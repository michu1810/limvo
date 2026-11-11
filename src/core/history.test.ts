import { describe, expect, it } from "vitest";
import { addReading, chartPoints } from "./history";

const RESET = "2026-11-10T07:00:00Z";
const NEXT = "2026-11-17T07:00:00Z";
const WEEK = 7 * 24 * 3600;
const start = Date.parse(RESET) - WEEK * 1000;
const hour = 3600_000;

describe("addReading", () => {
  it("keeps readings of the running period", () => {
    let history = addReading(undefined, RESET, { at: start + hour, percent: 2 });
    history = addReading(history, RESET, { at: start + 2 * hour, percent: 5 });
    expect(history.readings.map((r) => r.percent)).toEqual([2, 5]);
  });

  it("starts over when the window resets", () => {
    const old = addReading(undefined, RESET, { at: start + hour, percent: 80 });
    const fresh = addReading(old, NEXT, { at: Date.parse(RESET) + hour, percent: 1 });
    expect(fresh).toEqual({
      resetsAt: NEXT,
      readings: [{ at: Date.parse(RESET) + hour, percent: 1 }],
      past: [{ resetsAt: RESET, peak: 80 }],
    });
  });

  it("keeps the peak of each finished period, the last eight", () => {
    let history: ReturnType<typeof addReading> | undefined;
    for (let week = 0; week < 10; week++) {
      const reset = new Date(Date.parse(RESET) + week * WEEK * 1000).toISOString();
      const begin = Date.parse(reset) - WEEK * 1000;
      history = addReading(history, reset, { at: begin + hour, percent: week });
      history = addReading(history, reset, { at: begin + 2 * hour, percent: week * 10 });
    }
    expect(history?.past?.map((period) => period.peak)).toEqual([10, 20, 30, 40, 50, 60, 70, 80]);
    expect(history?.readings.at(-1)?.percent).toBe(90);
  });

  it("does not take a nudged reset time for a new period", () => {
    const old = addReading(undefined, RESET, { at: start + hour, percent: 40 });
    const nudged = new Date(Date.parse(RESET) + 60_000).toISOString();
    const next = addReading(old, nudged, { at: start + 2 * hour, percent: 41 });
    expect(next.past).toBeUndefined();
  });

  it("folds repeated values that come close together", () => {
    let history = addReading(undefined, RESET, { at: start, percent: 5 });
    history = addReading(history, RESET, { at: start + 5 * 60_000, percent: 5 });
    history = addReading(history, RESET, { at: start + 45 * 60_000, percent: 5 });
    expect(history.readings).toHaveLength(2);
  });

  it("ignores readings that arrive out of order", () => {
    const history = addReading(undefined, RESET, { at: start + 2 * hour, percent: 5 });
    expect(addReading(history, RESET, { at: start + hour, percent: 4 })).toBe(history);
  });
});

describe("chartPoints", () => {
  it("runs from an empty start through the history to now", () => {
    const history = addReading(undefined, RESET, { at: start + 42 * hour, percent: 20 });
    const points = chartPoints(history, RESET, WEEK, { at: start + 84 * hour, percent: 50 });
    expect(points).toEqual([
      { x: 0, y: 0 },
      { x: 0.25, y: 20 },
      { x: 0.5, y: 50 },
    ]);
  });

  it("ignores history from another period", () => {
    const history = addReading(undefined, NEXT, { at: start + hour, percent: 9 });
    expect(chartPoints(history, RESET, WEEK, { at: start + 84 * hour, percent: 50 })).toHaveLength(
      2,
    );
  });
});
