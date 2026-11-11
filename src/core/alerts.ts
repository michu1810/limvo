import { computePace, DEFAULT_THRESHOLDS, limitReachedAt } from "./pace";
import type { UsageWindow } from "./snapshot";

/** What the user is told about one window. */
export type Alert =
  | { kind: "threshold"; level: number }
  | { kind: "reached" }
  | { kind: "reset" }
  /** At the current pace the window runs out at `runsOutAt`, an ISO instant before its reset. */
  | { kind: "forecast"; runsOutAt: string };

/** What was last said about a window, kept between runs so restarts never repeat a notification. */
export interface AlertMemory {
  resetsAt: string | null;
  /** Highest level already announced in the current window, 0 when none. */
  announced: number;
  /** Highest usage seen in the current window. */
  peak: number;
  /** Whether the forecast was announced in the current window. Missing in memory kept by 0.4. */
  forecast?: boolean;
}

export interface AlertPolicy {
  /** Levels announced on the way up; 100 is announced as "reached". */
  levels: readonly number[];
  /** Announce the reset only when the window got at least this full, so quiet weeks stay quiet. */
  resetWorthAnnouncing: number | null;
  /**
   * Warn once per window when the projection reaches this many percent, or
   * `null` for no forecast. Set a little above 100 so a week that only just
   * runs out in its last hours does not raise an alarm.
   */
  forecastAbove: number | null;
}

export const HEADLINE_POLICY: AlertPolicy = {
  levels: [80, 95, 100],
  resetWorthAnnouncing: 80,
  forecastAbove: 105,
};
export const SECONDARY_POLICY: AlertPolicy = {
  levels: [100],
  resetWorthAnnouncing: null,
  forecastAbove: null,
};

/**
 * Compares a fresh reading with what was announced before and decides what,
 * if anything, to tell the user. Returns the memory to keep either way.
 */
export function evaluate(
  window: Pick<UsageWindow, "percent" | "resetsAt" | "periodSeconds">,
  memory: AlertMemory | undefined,
  policy: AlertPolicy,
  now: Date,
): { alert: Alert | null; memory: AlertMemory } {
  const percent = window.percent;
  const sameWindow = memory !== undefined && memory.resetsAt === window.resetsAt;

  // A new window started: the reset time moved on and usage fell back.
  if (memory !== undefined && !sameWindow && percent < memory.peak) {
    const worth = policy.resetWorthAnnouncing;
    return {
      alert: worth !== null && memory.peak >= worth ? { kind: "reset" } : null,
      memory: {
        resetsAt: window.resetsAt,
        announced: highestBelow(policy.levels, percent),
        peak: percent,
        forecast: false,
      },
    };
  }

  const announced = memory?.announced ?? 0;
  const peak = Math.max(memory?.peak ?? 0, percent);
  const level = highestBelow(policy.levels, percent);
  const next: AlertMemory = {
    resetsAt: window.resetsAt,
    announced: Math.max(announced, level),
    peak,
    // Once a level is announced, those notifications carry the news; a late forecast would repeat it.
    forecast: (memory?.forecast ?? false) || level > 0,
  };

  if (level > announced) {
    return {
      alert: level >= 100 ? { kind: "reached" } : { kind: "threshold", level },
      memory: next,
    };
  }
  if (next.forecast) return { alert: null, memory: next };

  const runsOutAt = forecast(window, policy, now);
  if (runsOutAt === null) return { alert: null, memory: next };
  return {
    alert: { kind: "forecast", runsOutAt: runsOutAt.toISOString() },
    memory: { ...next, forecast: true },
  };
}

/**
 * When the window runs out at the current pace, if that is worth a warning:
 * the policy asks for one, the projection is over its mark, and the first
 * tenth of the window has passed, so one busy hour after a reset says nothing.
 */
function forecast(
  window: Pick<UsageWindow, "percent" | "resetsAt" | "periodSeconds">,
  policy: AlertPolicy,
  now: Date,
): Date | null {
  if (policy.forecastAbove === null) return null;
  const pace = computePace(window, now);
  if (!pace || pace.elapsed < DEFAULT_THRESHOLDS.elapsedFloor) return null;
  if (pace.projected < policy.forecastAbove) return null;
  return limitReachedAt(window, now);
}

/** The highest policy level that `percent` has reached, or 0. */
function highestBelow(levels: readonly number[], percent: number): number {
  return levels.reduce((best, level) => (percent >= level && level > best ? level : best), 0);
}
