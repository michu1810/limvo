import { describe, expect, it } from "vitest";
import { budgetWorthShowing, computePace, limitReachedAt, usageBudget } from "./pace";

const WEEK = 7 * 24 * 3600;
const SESSION = 5 * 3600;
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

describe("usageBudget", () => {
  const weekly = (percent: number, hoursLeft: number) =>
    ({ kind: "weekly", percent, resetsAt: hoursFromNow(hoursLeft), periodSeconds: WEEK }) as const;

  it("splits what is left over the days to the reset", () => {
    // 16% used, 6 days 1 hr left: 23 hours in, 84% for 145 hours.
    const budget = usageBudget(weekly(16, 145), now);
    expect(budget?.unit).toBe("day");
    expect(budget?.allowed).toBeCloseTo((84 / 145) * 24, 5);
    expect(budget?.current).toBeCloseTo((16 / 23) * 24, 5);
  });

  it("agrees with the pace status on whether the week lasts", () => {
    for (const [percent, hoursLeft] of [
      [51, 40],
      [45, 84],
      [60, 84],
      [16, 145],
      [99, 30],
    ] as const) {
      const pace = computePace(weekly(percent, hoursLeft), now);
      const budget = usageBudget(weekly(percent, hoursLeft), now);
      const label = `${percent}% with ${hoursLeft} hr left`;
      if (!pace || !budget) throw new Error(`no pace or budget for ${label}`);
      expect(budget.current > budget.allowed, label).toBe(pace.projected > 100);
    }
  });

  it("calms only the rate so far in the first tenth of the window", () => {
    // 9.5% in the first hour: projected 95% (tight), yet 13.6% a day so far
    // against 13.0% allowed. The flyout colours the line by the pace label.
    const budget = usageBudget(weekly(9.5, 167), now);
    expect(computePace(weekly(9.5, 167), now)?.status).toBe("tight");
    expect(budget?.current).toBeCloseTo((9.5 / 16.8) * 24, 5);
    expect(budget?.allowed).toBeCloseTo((90.5 / 167) * 24, 5);
  });

  it("counts a session per hour", () => {
    const budget = usageBudget(
      { kind: "session", percent: 40, resetsAt: hoursFromNow(3), periodSeconds: SESSION },
      now,
    );
    expect(budget?.unit).toBe("hour");
    expect(budget?.allowed).toBeCloseTo(20, 5);
    expect(budget?.current).toBeCloseTo(20, 5);
  });

  it("counts per hour on the last day, so it never promises more than is left", () => {
    const budget = usageBudget(weekly(70, 10), now);
    expect(budget?.unit).toBe("hour");
    expect(budget?.allowed).toBeCloseTo(3, 5);
  });

  it("keeps the current rate calm right after a reset", () => {
    // 2% in the first hour would read as 48% a day without the floor.
    const budget = usageBudget(weekly(2, 167), now);
    expect(budget?.current).toBeCloseTo((2 / 16.8) * 24, 5);
  });

  it("says nothing in the last hour, for a spent window or without a clock", () => {
    expect(usageBudget(weekly(50, 0.5), now)).toBeNull();
    expect(usageBudget(weekly(50, -2), now)).toBeNull();
    expect(usageBudget(weekly(100, 40), now)).toBeNull();
    expect(usageBudget({ ...weekly(10, 40), resetsAt: null }, now)).toBeNull();
    expect(usageBudget({ ...weekly(10, 40), periodSeconds: null }, now)).toBeNull();
    expect(usageBudget({ ...weekly(10, 40), resetsAt: "soon" }, now)).toBeNull();
  });
});

describe("budgetWorthShowing", () => {
  const weekly = (percent: number, hoursLeft: number) =>
    ({ kind: "weekly", percent, resetsAt: hoursFromNow(hoursLeft), periodSeconds: WEEK }) as const;

  it("shows the budget only while the window could run out", () => {
    expect(budgetWorthShowing(weekly(51, 40), now)).toBeNull(); // on track
    expect(budgetWorthShowing(weekly(45, 84), now)?.unit).toBe("day"); // tight
    expect(budgetWorthShowing(weekly(60, 84), now)?.unit).toBe("day"); // running hot
    expect(budgetWorthShowing(weekly(100, 40), now)).toBeNull(); // reached
  });
});
