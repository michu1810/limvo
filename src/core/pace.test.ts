import { describe, expect, it } from "vitest";
import { computePace } from "./pace";

const WEEK = 7 * 24 * 3600;
const now = new Date("2026-11-08T17:00:00Z");
const hoursFromNow = (hours: number) => new Date(now.getTime() + hours * 3600_000).toISOString();

describe("computePace", () => {
  it("matches the reference reading: 51% with 1 day 16 hr left", () => {
    const pace = computePace({ percent: 51, resetsAt: hoursFromNow(40), periodSeconds: WEEK }, now);
    expect(pace?.elapsed).toBeCloseTo(128 / 168, 5);
    expect(Math.round(pace?.projected ?? 0)).toBe(67);
    expect(pace?.status).toBe("on_track");
  });

  it("warns when the projection gets close to the limit", () => {
    const pace = computePace({ percent: 45, resetsAt: hoursFromNow(84), periodSeconds: WEEK }, now);
    expect(pace?.status).toBe("tight");
  });

  it("flags a pace that would hit the limit before the reset", () => {
    const pace = computePace({ percent: 60, resetsAt: hoursFromNow(84), periodSeconds: WEEK }, now);
    expect(pace?.status).toBe("running_hot");
  });

  it("reports a reached limit regardless of pace", () => {
    const pace = computePace({ percent: 100, resetsAt: hoursFromNow(1), periodSeconds: WEEK }, now);
    expect(pace?.status).toBe("limit_reached");
  });

  it("keeps projections calm right after a reset", () => {
    const pace = computePace({ percent: 2, resetsAt: hoursFromNow(167), periodSeconds: WEEK }, now);
    expect(pace?.projected).toBeCloseTo(20, 5);
    expect(pace?.status).toBe("on_track");
  });

  it("treats a reset in the past as a fully elapsed window", () => {
    const pace = computePace({ percent: 30, resetsAt: hoursFromNow(-2), periodSeconds: WEEK }, now);
    expect(pace?.elapsed).toBe(1);
  });

  it("returns null without a clock to compare against", () => {
    expect(computePace({ percent: 10, resetsAt: null, periodSeconds: WEEK }, now)).toBeNull();
    expect(
      computePace({ percent: 10, resetsAt: hoursFromNow(5), periodSeconds: null }, now),
    ).toBeNull();
    expect(computePace({ percent: 10, resetsAt: "soon", periodSeconds: WEEK }, now)).toBeNull();
  });
});
