import { describe, expect, it } from "vitest";
import { computePace, limitReachedAt } from "./pace";

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

describe("limitReachedAt", () => {
  it("finds when a hot week runs out", () => {
    // 60% used halfway through the week: 100% after 5/6 of it, 28 hours before the reset.
    const at = limitReachedAt(
      { percent: 60, resetsAt: hoursFromNow(84), periodSeconds: WEEK },
      now,
    );
    expect(at?.getTime()).toBe(now.getTime() + 56 * 3600_000);
  });

  it("says nothing when the week lasts until the reset", () => {
    expect(
      limitReachedAt({ percent: 51, resetsAt: hoursFromNow(40), periodSeconds: WEEK }, now),
    ).toBeNull();
  });

  it("says nothing for an empty or an exhausted window", () => {
    expect(
      limitReachedAt({ percent: 0, resetsAt: hoursFromNow(40), periodSeconds: WEEK }, now),
    ).toBeNull();
    expect(
      limitReachedAt({ percent: 100, resetsAt: hoursFromNow(40), periodSeconds: WEEK }, now),
    ).toBeNull();
  });

  it("does not panic after an early burst", () => {
    // 9% in the first hour would empty the week in 11 hours without the floor.
    const at = limitReachedAt(
      { percent: 9, resetsAt: hoursFromNow(167), periodSeconds: WEEK },
      now,
    );
    expect(at).toBeNull();
  });
});
