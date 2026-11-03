import type { UsageWindow } from "./snapshot";

/** What the user is told about one window. */
export type Alert = { kind: "threshold"; level: number } | { kind: "reached" } | { kind: "reset" };

/** What was last said about a window, kept between runs so restarts never repeat a notification. */
export interface AlertMemory {
  resetsAt: string | null;
  /** Highest level already announced in the current window, 0 when none. */
  announced: number;
  /** Highest usage seen in the current window. */
  peak: number;
}

export interface AlertPolicy {
  /** Levels announced on the way up; 100 is announced as "reached". */
  levels: readonly number[];
  /** Announce the reset only when the window got at least this full, so quiet weeks stay quiet. */
  resetWorthAnnouncing: number | null;
}

export const HEADLINE_POLICY: AlertPolicy = { levels: [80, 95, 100], resetWorthAnnouncing: 80 };
export const SECONDARY_POLICY: AlertPolicy = { levels: [100], resetWorthAnnouncing: null };

/**
 * Compares a fresh reading with what was announced before and decides what,
 * if anything, to tell the user. Returns the memory to keep either way.
 */
export function evaluate(
  window: Pick<UsageWindow, "percent" | "resetsAt">,
  memory: AlertMemory | undefined,
  policy: AlertPolicy,
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
  };

  if (level <= announced) return { alert: null, memory: next };
  return { alert: level >= 100 ? { kind: "reached" } : { kind: "threshold", level }, memory: next };
}

/** The highest policy level that `percent` has reached, or 0. */
function highestBelow(levels: readonly number[], percent: number): number {
  return levels.reduce((best, level) => (percent >= level && level > best ? level : best), 0);
}
