import type { CSSProperties } from "react";
import type { Severity } from "../core/snapshot";
import { useAnimatedNumber } from "./hooks";

interface MeterProps {
  /** Share used, 0 to 100. */
  value: number;
  /** Share of the window elapsed, 0 to 1. Draws the "where the clock is" tick. */
  marker?: number | null;
  size: "large" | "small" | "strip";
  severity?: Severity;
  label: string;
  /** Changing this replays the fill from empty, as the flyout does when it opens. */
  replay?: number;
}

/** A light passes along the fill once whenever the value or the replay changes. */
const glintKey = (value: number, replay: number) => `${replay}:${Math.round(value)}`;

export function Meter({
  value,
  marker = null,
  size,
  severity = "normal",
  label,
  replay = 0,
}: MeterProps) {
  const clamped = Math.min(100, Math.max(0, value));
  const shown = useAnimatedNumber(clamped, replay, size === "small" ? 750 : 650);
  const style = {
    "--meter-value": `${shown}%`,
    "--meter-marker": marker === null ? undefined : `${Math.min(1, Math.max(0, marker)) * 100}%`,
  } as CSSProperties;

  return (
    // biome-ignore lint/a11y/useSemanticElements: a native <meter> cannot hold the pace marker.
    <div
      className="meter"
      data-size={size}
      data-severity={severity}
      data-empty={clamped === 0 || undefined}
      role="meter"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(clamped)}
      style={style}
    >
      <span key={glintKey(clamped, replay)} className="meter__fill" />
      {marker !== null && <span key={replay} className="meter__marker" aria-hidden="true" />}
    </div>
  );
}
