/**
 * Readings of one window over its current period, for the weekly chart.
 *
 * Only the running period is kept: when the reset time moves on, the history
 * starts over. Readings that repeat the previous value are folded together
 * unless enough time has passed, so a quiet day does not fill the store.
 */

export interface Reading {
  /** Unix milliseconds. */
  at: number;
  percent: number;
}

export interface WindowHistory {
  resetsAt: string;
  readings: Reading[];
}

/** Repeat a flat value at most this often, so the chart still shows idle stretches. */
const FLAT_SPACING_MS = 30 * 60_000;
/** A week of five-minute readings fits with room to spare. */
const MAX_READINGS = 600;

export function addReading(
  history: WindowHistory | undefined,
  resetsAt: string,
  reading: Reading,
): WindowHistory {
  const current = history && history.resetsAt === resetsAt ? history : { resetsAt, readings: [] };
  const last = current.readings.at(-1);

  if (last && reading.at <= last.at) return current;
  if (last && last.percent === reading.percent && reading.at - last.at < FLAT_SPACING_MS) {
    return current;
  }
  const readings = [...current.readings, reading].slice(-MAX_READINGS);
  return { resetsAt, readings };
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
