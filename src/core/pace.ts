import type { UsageWindow } from "./snapshot";

export type PaceStatus = "on_track" | "tight" | "running_hot" | "limit_reached";

export interface PaceThresholds {
  /** Projected end-of-window usage at which the status turns to `tight`. */
  tight: number;
  /** Projected end-of-window usage at which the status turns to `running_hot`. */
  hot: number;
  /**
   * Smallest share of the window treated as elapsed when projecting. Right
   * after a reset a single request would otherwise project to thousands of
   * percent; the floor keeps early projections calm without hiding real spikes.
   */
  elapsedFloor: number;
}

export const DEFAULT_THRESHOLDS: PaceThresholds = { tight: 85, hot: 100, elapsedFloor: 0.1 };

export interface Pace {
  /** Share of the window already elapsed, 0 to 1. */
  elapsed: number;
  /** Usage projected for the end of the window at the current rate, in percent. */
  projected: number;
  status: PaceStatus;
}

/**
 * Linear projection: if `percent` was used in the elapsed part of the window,
 * the same rate carries on until the reset.
 *
 * Returns `null` when the window has no reset time or length, because there
 * is no clock to measure the pace against.
 */
export function computePace(
  window: Pick<UsageWindow, "percent" | "resetsAt" | "periodSeconds">,
  now: Date,
  thresholds: PaceThresholds = DEFAULT_THRESHOLDS,
): Pace | null {
  if (window.resetsAt === null || window.periodSeconds === null || window.periodSeconds <= 0) {
    return null;
  }
  const resetsAt = Date.parse(window.resetsAt);
  if (Number.isNaN(resetsAt)) {
    return null;
  }

  const periodMs = window.periodSeconds * 1000;
  const remaining = clamp((resetsAt - now.getTime()) / periodMs, 0, 1);
  const elapsed = 1 - remaining;
  const projected = window.percent / Math.max(elapsed, thresholds.elapsedFloor);

  return { elapsed, projected, status: classify(window.percent, projected, thresholds) };
}

function classify(percent: number, projected: number, thresholds: PaceThresholds): PaceStatus {
  if (percent >= 100) return "limit_reached";
  if (projected >= thresholds.hot) return "running_hot";
  if (projected >= thresholds.tight) return "tight";
  return "on_track";
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
