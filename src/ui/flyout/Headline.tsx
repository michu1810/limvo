import { computePace } from "../../core/pace";
import type { UsageWindow } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { headlineLabel, PACE_MESSAGE } from "../labels";
import { Meter } from "../Meter";

export function Headline({ window, now }: { window: UsageWindow; now: Date }) {
  const i18n = useI18n();
  const { t, format } = i18n;
  const pace = computePace(window, now);
  const resetsAt = window.resetsAt ? new Date(window.resetsAt) : null;
  const remaining = resetsAt ? resetsAt.getTime() - now.getTime() : null;
  const label = headlineLabel(window, i18n);

  return (
    <section className="headline" aria-label={label}>
      <p className="eyebrow">{label}</p>
      <div className="headline__row">
        <span className="headline__value">{format.percent(window.percent)}</span>
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
      />

      {pace && (
        <p className="pace">
          <span className="pill" data-status={pace.status}>
            {t(PACE_MESSAGE[pace.status])}
          </span>
          <span className="pace__detail">
            {t(window.kind === "session" ? "paceDetailSession" : "paceDetailWeekly", {
              elapsed: format.percent(pace.elapsed * 100),
              projected: format.percent(pace.projected),
            })}
          </span>
        </p>
      )}
    </section>
  );
}
