import { computePace } from "../../core/pace";
import type { UsageWindow } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { useAnimatedNumber } from "../hooks";
import { headlineLabel, PACE_MESSAGE } from "../labels";
import { Meter } from "../Meter";

export function Headline({
  window,
  now,
  replay = 0,
  compact = false,
}: {
  window: UsageWindow;
  now: Date;
  /** Changing it counts the number up again, as when the flyout opens. */
  replay?: number;
  /** The desktop card drops the pace detail to stay small. */
  compact?: boolean;
}) {
  const i18n = useI18n();
  const { t, format } = i18n;
  const pace = computePace(window, now);
  const resetsAt = window.resetsAt ? new Date(window.resetsAt) : null;
  const remaining = resetsAt ? resetsAt.getTime() - now.getTime() : null;
  const label = headlineLabel(window, i18n);
  const counted = useAnimatedNumber(window.percent, replay);

  return (
    <section className="headline" data-compact={compact || undefined} aria-label={label}>
      <p className="eyebrow">{label}</p>
      <div className="headline__row">
        {/* Screen readers get the final value, not every frame of the count. */}
        <span className="headline__value">
          <span aria-hidden="true">{format.percent(Math.round(counted))}</span>
          <span className="sr-only">{format.percent(window.percent)}</span>
        </span>
        {resetsAt && remaining !== null && (
          <p className="headline__reset">
            <strong>
              {remaining > 60_000
                ? t("resetsIn", { duration: format.duration(remaining) })
                : t("resetsSoon")}
            </strong>
            <time dateTime={resetsAt.toISOString()}>{format.weekdayTime(resetsAt)}</time>
          </p>
        )}
      </div>

      <Meter
        size="large"
        value={window.percent}
        marker={pace?.elapsed ?? null}
        severity={window.severity}
        label={label}
        replay={replay}
      />

      {pace && (
        <p className="pace">
          {/* Keyed on the status so a change plays the pill's arrival again. */}
          <span key={pace.status} className="pill" data-status={pace.status}>
            {t(PACE_MESSAGE[pace.status])}
          </span>
          {!compact && (
            <span className="pace__detail">
              {t(window.kind === "session" ? "paceDetailSession" : "paceDetailWeekly", {
                elapsed: format.percent(pace.elapsed * 100),
                projected: format.percent(pace.projected),
              })}
            </span>
          )}
        </p>
      )}
    </section>
  );
}
