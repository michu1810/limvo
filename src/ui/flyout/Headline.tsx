import { computePace } from "../../core/pace";
import type { PlanBadge, UsageWindow } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { headlineLabel, PACE_MESSAGE } from "../labels";
import { Meter } from "../Meter";
import { RollingPercent } from "../RollingNumber";

export function Headline({
  window,
  now,
  replay = 0,
  compact = false,
  plan = null,
}: {
  window: UsageWindow;
  /** The account's plan, shown as a badge beside the label. */
  plan?: PlanBadge | null;
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

  return (
    <section className="headline" data-compact={compact || undefined} aria-label={label}>
      <div className="headline__label">
        <p className="eyebrow">{label}</p>
        {plan && (
          <span className="plan" data-plan={plan.id}>
            {plan.label}
          </span>
        )}
      </div>
      <div className="headline__row">
        <RollingPercent className="headline__value" value={window.percent} replay={replay} />
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
