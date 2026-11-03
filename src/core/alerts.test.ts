import { describe, expect, it } from "vitest";
import { type AlertMemory, evaluate, HEADLINE_POLICY, SECONDARY_POLICY } from "./alerts";

const WEEK_A = "2026-11-10T07:00:00Z";
const WEEK_B = "2026-11-17T07:00:00Z";

function run(readings: [number, string][], policy = HEADLINE_POLICY) {
  let memory: AlertMemory | undefined;
  return readings.map(([percent, resetsAt]) => {
    const result = evaluate({ percent, resetsAt }, memory, policy);
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
