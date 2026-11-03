import type { CSSProperties } from "react";
import { type ChartPoint, chartPoints, type WindowHistory } from "../../core/history";
import type { UsageWindow } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";

/** Drawing space in CSS pixels: the flyout content is 332 px wide, so nothing is stretched. */
const WIDTH = 332;
const HEIGHT = 44;
const TOP = 4;
const BOTTOM = 41;
const y = (percent: number) => BOTTOM - (Math.min(percent, 112) / 100) * (BOTTOM - TOP);

/**
 * The period at a glance. The dashed diagonal is an even pace that ends the
 * period exactly at the limit; above it means faster than that. The filled
 * line is what actually happened, and the dotted tail carries today's rate
 * on to the reset.
 */
export function WeekChart({
  window,
  history,
  now,
  projected,
  replay,
}: {
  window: UsageWindow;
  history: WindowHistory | undefined;
  now: Date;
  projected: number;
  replay: number;
}) {
  const { t } = useI18n();
  if (!window.resetsAt || !window.periodSeconds) return null;

  const points = chartPoints(history, window.resetsAt, window.periodSeconds, {
    at: now.getTime(),
    percent: window.percent,
  });
  const last = points.at(-1);
  if (!last || points.length < 2) return null;

  const line = points.map((point) => `${point.x * WIDTH},${y(point.y)}`).join(" ");
  const area = `M0,${BOTTOM} L${line.replaceAll(" ", " L")} L${last.x * WIDTH},${BOTTOM} Z`;
  const dot = {
    "--dot-x": `${last.x * 100}%`,
    "--dot-y": `${(y(last.y) / HEIGHT) * 100}%`,
  } as CSSProperties;

  return (
    <figure key={replay} className="chart" role="img" aria-label={t("weekChart")}>
      <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} aria-hidden="true">
        <defs>
          <linearGradient id="chart-fill" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="var(--accent)" stopOpacity="0.28" />
            <stop offset="1" stopColor="var(--accent)" stopOpacity="0" />
          </linearGradient>
        </defs>
        <line className="chart__grid" x1="0" y1={TOP} x2={WIDTH} y2={TOP} />
        <line className="chart__pace" x1="0" y1={BOTTOM} x2={WIDTH} y2={TOP} />
        <path className="chart__area" d={area} fill="url(#chart-fill)" />
        <polyline className="chart__line" points={line} pathLength={1} />
        <Projection from={last} projected={projected} />
      </svg>
      <span className="chart__dot" style={dot} aria-hidden="true" />
    </figure>
  );
}

function Projection({ from, projected }: { from: ChartPoint; projected: number }) {
  if (from.x >= 1) return null;
  return (
    <line
      className="chart__projection"
      x1={from.x * WIDTH}
      y1={y(from.y)}
      x2={WIDTH}
      y2={y(projected)}
    />
  );
}
