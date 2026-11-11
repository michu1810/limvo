import { describe, expect, it } from "vitest";
import {
  type AlertMemory,
  evaluate,
  HEADLINE_POLICY,
  headlinePolicy,
  SECONDARY_POLICY,
} from "./alerts";

const WEEK_A = "2026-11-10T07:00:00Z";
const WEEK_B = "2026-11-17T07:00:00Z";
const NOW = new Date("2026-11-05T07:00:00Z");

function run(readings: [number, string][], policy = HEADLINE_POLICY) {
  let memory: AlertMemory | undefined;
  return readings.map(([percent, resetsAt]) => {
    // No period: these readings test the levels, never the forecast.
    const result = evaluate({ percent, resetsAt, periodSeconds: null }, memory, policy, NOW);
    memory = result.memory;
    return result.alert;
  });
}

describe("evaluate", () => {
  it("announces each level once on the way up", () => {
    expect(
      run([
        [50, WEEK_A],
        [81, WEEK_A],
        [85, WEEK_A],
        [96, WEEK_A],
        [100, WEEK_A],
      ]),
    ).toEqual([
      null,
      { kind: "threshold", level: 80 },
      null,
      { kind: "threshold", level: 95 },
      { kind: "reached" },
    ]);
  });

  it("announces only the highest level when several are crossed at once", () => {
    expect(
      run([
        [10, WEEK_A],
        [97, WEEK_A],
      ]),
    ).toEqual([null, { kind: "threshold", level: 95 }]);
  });

  it("announces a reset after a busy week, then starts counting again", () => {
    expect(
      run([
        [90, WEEK_A],
        [2, WEEK_B],
        [82, WEEK_B],
      ]),
    ).toEqual([
      { kind: "threshold", level: 80 },
      { kind: "reset" },
      { kind: "threshold", level: 80 },
    ]);
  });

  it("keeps a quiet week quiet", () => {
    expect(
      run([
        [30, WEEK_A],
        [1, WEEK_B],
      ]),
    ).toEqual([null, null]);
  });

  it("does not repeat itself when the same reading comes back", () => {
    expect(
      run([
        [85, WEEK_A],
        [85, WEEK_A],
        [85, WEEK_A],
      ]),
    ).toEqual([{ kind: "threshold", level: 80 }, null, null]);
  });

  it("only reports a reached limit for secondary windows", () => {
    expect(
      run(
        [
          [85, WEEK_A],
          [100, WEEK_A],
        ],
        SECONDARY_POLICY,
      ),
    ).toEqual([null, { kind: "reached" }]);
  });
});

describe("evaluate, forecast", () => {
  const WEEK = 7 * 24 * 3600;
  const start = new Date("2026-11-03T07:00:00Z");
  const resetsAt = "2026-11-10T07:00:00Z";
  const at = (hours: number) => new Date(start.getTime() + hours * 3600_000);

  /** Readings of one week, each `[percent, hours since the reset]`. */
  function week(readings: [number, number][], policy = HEADLINE_POLICY) {
    let memory: AlertMemory | undefined;
    return readings.map(([percent, hours]) => {
      const result = evaluate(
        { percent, resetsAt, periodSeconds: WEEK },
        memory,
        policy,
        at(hours),
      );
      memory = result.memory;
      return result.alert;
    });
  }

  it("warns once when the week would run out before the reset", () => {
    // 16% after 23 hours projects to ~117% and runs out after ~144 hours.
    const alerts = week([
      [10, 20],
      [16, 23],
      [20, 30],
    ]);
    expect(alerts[0]).toBeNull();
    expect(alerts[1]).toEqual({
      kind: "forecast",
      runsOutAt: at((23 / 16) * 100).toISOString(),
    });
    expect(alerts[2]).toBeNull();
  });

  it("stays quiet in the first tenth of the week, however busy", () => {
    // 15% in the first 10 hours: 16.8 hours make the first tenth.
    expect(week([[15, 10]])).toEqual([null]);
  });

  it("stays quiet when the week only just runs out", () => {
    // Projected ~104%: under the mark of 105%.
    expect(week([[52, 84]])).toEqual([null]);
  });

  it("leaves the news to the levels once one is announced", () => {
    expect(
      week([
        [82, 60],
        [90, 70],
      ]),
    ).toEqual([{ kind: "threshold", level: 80 }, null]);
  });

  it("warns again in the next week", () => {
    let memory: AlertMemory | undefined;
    const read = (percent: number, reset: string, now: Date) => {
      const result = evaluate(
        { percent, resetsAt: reset, periodSeconds: WEEK },
        memory,
        HEADLINE_POLICY,
        now,
      );
      memory = result.memory;
      return result.alert?.kind ?? null;
    };
    expect(read(30, resetsAt, at(30))).toBe("forecast");
    expect(read(1, "2026-11-17T07:00:00Z", at(170))).toBe(null);
    expect(read(30, "2026-11-17T07:00:00Z", at(198))).toBe("forecast");
  });

  it("never warns about a secondary limit", () => {
    expect(week([[30, 30]], SECONDARY_POLICY)).toEqual([null]);
  });
});

describe("headlinePolicy", () => {
  it("announces the levels the user chose", () => {
    const early = headlinePolicy("early", true);
    expect(
      run(
        [
          [55, WEEK_A],
          [76, WEEK_A],
          [91, WEEK_A],
        ],
        early,
      ),
    ).toEqual([
      { kind: "threshold", level: 50 },
      { kind: "threshold", level: 75 },
      { kind: "threshold", level: 90 },
    ]);
    expect(
      run(
        [
          [85, WEEK_A],
          [100, WEEK_A],
        ],
        headlinePolicy("reached", true),
      ),
    ).toEqual([null, { kind: "reached" }]);
  });

  it("drops the forecast when it is turned off", () => {
    expect(headlinePolicy("standard", true).forecastAbove).toBe(105);
    expect(headlinePolicy("standard", false).forecastAbove).toBeNull();
  });
});
