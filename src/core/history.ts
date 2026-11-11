/**
 * Readings of one window over its current period, for the weekly chart, and
 * the peak of each period before it, for the history of past periods.
 *
 * Only the running period keeps its readings: when the reset time moves on,
 * they are folded into one past entry and the history starts over. Readings
 * that repeat the previous value are folded together unless enough time has
 * passed, so a quiet day does not fill the store.
 */

export interface Reading {
  /** Unix milliseconds. */
  at: number;
  percent: number;
}

/** A finished period: when it reset and the most it reached while Limvo watched. */
export interface PastPeriod {
  resetsAt: string;
  peak: number;
}

export interface WindowHistory {
  resetsAt: string;
  readings: Reading[];
  /** Finished periods, oldest first. Missing until one has finished. */
  past?: PastPeriod[];
}

/** Repeat a flat value at most this often, so the chart still shows idle stretches. */
const FLAT_SPACING_MS = 30 * 60_000;
/** A week of five-minute readings fits with room to spare. */
const MAX_READINGS = 600;
/** Past periods kept: two months of weeks. */
export const MAX_PAST = 8;
/**
 * A reset time that moves on by less than this is the service adjusting it,
 * not a new period, and leaves no past entry behind.
 */
const NEW_PERIOD_MS = 3600_000;

export function addReading(
  history: WindowHistory | undefined,
  resetsAt: string,
  reading: Reading,
): WindowHistory {
  const current =
    history && history.resetsAt === resetsAt
      ? history
      : { resetsAt, readings: [], ...pastAfter(history, resetsAt, reading.at) };
  const last = current.readings.at(-1);

  if (last && reading.at <= last.at) return current;
  if (last && last.percent === reading.percent && reading.at - last.at < FLAT_SPACING_MS) {
    return current;
  }
  const readings = [...current.readings, reading].slice(-MAX_READINGS);
  return { ...current, resetsAt, readings };
}

/**
 * The past periods once `history` gives way to the period ending at
 * `resetsAt`. The old period counts only if it has ended by `now`: a service
 * whose headline can switch to another quota mid-period (Antigravity leads
 * with its fullest model) would otherwise leave a period behind that never
 * finished.
 */
function pastAfter(
  history: WindowHistory | undefined,
  resetsAt: string,
  now: number,
): Pick<WindowHistory, "past"> {
  if (!history) return {};
  const past = history.past ?? [];
  const oldReset = Date.parse(history.resetsAt);
  const moved = Date.parse(resetsAt) - oldReset;
  const ended = now >= oldReset - NEW_PERIOD_MS;
  if (!(moved >= NEW_PERIOD_MS) || !ended || history.readings.length === 0) {
    return past.length > 0 ? { past } : {};
  }
  const peak = Math.max(...history.readings.map((reading) => reading.percent));
  return { past: [...past, { resetsAt: history.resetsAt, peak }].slice(-MAX_PAST) };
}

export interface ChartPoint {
  /** Share of the period, 0 to 1. */
  x: number;
  /** Usage, 0 to 100. */
  y: number;
}

/**
 * Points for the chart: the period starts empty, then every reading, then
 * the reading the user is looking at now.
 */
export function chartPoints(
  history: WindowHistory | undefined,
  resetsAt: string,
  periodSeconds: number,
  current: Reading,
): ChartPoint[] {
  const end = Date.parse(resetsAt);
  const start = end - periodSeconds * 1000;
  const span = end - start;
  if (!Number.isFinite(span) || span <= 0) return [];

  const toPoint = (reading: Reading): ChartPoint => ({
    x: Math.min(1, Math.max(0, (reading.at - start) / span)),
    y: Math.min(100, Math.max(0, reading.percent)),
  });
  const kept =
    history && history.resetsAt === resetsAt
      ? history.readings.filter((reading) => reading.at >= start && reading.at < current.at)
      : [];
  return [{ x: 0, y: 0 }, ...kept.map(toPoint), toPoint(current)];
}
