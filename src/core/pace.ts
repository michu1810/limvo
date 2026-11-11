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

/**
 * When the window would run out if usage carries on at its average rate so
 * far, or `null` when it lasts until the reset (or has already run out).
 * Uses the same elapsed floor as the projection, so one early burst does not
 * predict an empty week.
 */
export function limitReachedAt(
  window: Pick<UsageWindow, "percent" | "resetsAt" | "periodSeconds">,
  now: Date,
  thresholds: PaceThresholds = DEFAULT_THRESHOLDS,
): Date | null {
  if (window.resetsAt === null || window.periodSeconds === null || window.periodSeconds <= 0) {
    return null;
  }
  const resetsAt = Date.parse(window.resetsAt);
  if (Number.isNaN(resetsAt) || window.percent <= 0 || window.percent >= 100) return null;

  const periodMs = window.periodSeconds * 1000;
  const elapsedMs = Math.max(
    periodMs - (resetsAt - now.getTime()),
    thresholds.elapsedFloor * periodMs,
  );
  const perMs = window.percent / elapsedMs;
  const at = now.getTime() + (100 - window.percent) / perMs;
  return at < resetsAt ? new Date(at) : null;
}

export type BudgetUnit = "day" | "hour";

export interface Budget {
  /** What one step of the budget is. */
  unit: BudgetUnit;
  /** Share of the limit that can be used per unit from now on and still last until the reset, in percent. */
  allowed: number;
  /** Average share used per unit so far in this window, in percent. */
  current: number;
}

const HOUR_MS = 3600_000;
const DAY_MS = 24 * HOUR_MS;

/**
 * How fast the rest of the window can be used and still last until the
 * reset, next to how fast it has been used so far.
 *
 * Counted per hour for a session, and for any window with less than a day
 * left, where a daily figure would promise more than remains. Uses the same
 * elapsed floor as the projection, so once the first tenth of the window has
 * passed, `current` exceeds `allowed` exactly when the projection is over
 * 100%. Before that the floor calms `current` only. Returns `null` without a clock, for a
 * spent window, and in the last hour, when the reset is close enough to
 * make a budget pointless.
 */
export function usageBudget(
  window: Pick<UsageWindow, "kind" | "percent" | "resetsAt" | "periodSeconds">,
  now: Date,
  thresholds: PaceThresholds = DEFAULT_THRESHOLDS,
): Budget | null {
  if (window.resetsAt === null || window.periodSeconds === null || window.periodSeconds <= 0) {
    return null;
  }
  const resetsAt = Date.parse(window.resetsAt);
  if (Number.isNaN(resetsAt) || window.percent >= 100) return null;

  const periodMs = window.periodSeconds * 1000;
  const remainingMs = Math.min(resetsAt - now.getTime(), periodMs);
  if (remainingMs < HOUR_MS) return null;

  const unit: BudgetUnit = window.kind === "session" || remainingMs < DAY_MS ? "hour" : "day";
  const unitMs = unit === "day" ? DAY_MS : HOUR_MS;
  const elapsedMs = Math.max(periodMs - remainingMs, thresholds.elapsedFloor * periodMs);
  const percent = Math.max(0, window.percent);

  return {
    unit,
    allowed: ((100 - percent) / remainingMs) * unitMs,
    current: (percent / elapsedMs) * unitMs,
  };
}

/**
 * The budget, when it is worth showing: only while the window could run out.
 * While the pace is calm it would be noise, and once the limit is reached
 * there is nothing left to budget. The flyout and the tray tooltip share it.
 */
export function budgetWorthShowing(
  window: Pick<UsageWindow, "kind" | "percent" | "resetsAt" | "periodSeconds">,
  now: Date,
): Budget | null {
  const status = computePace(window, now)?.status;
  return status === "tight" || status === "running_hot" ? usageBudget(window, now) : null;
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
