import type { CSSProperties } from "react";
import type { WindowHistory } from "../../core/history";
import type { UsageWindow } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageKey } from "../../i18n/resolve";

/** The periods a window's history is told in; a five-hour session has none worth keeping. */
const TITLE: Partial<Record<UsageWindow["kind"], MessageKey>> = {
  weekly: "pastWeeks",
  scoped: "pastWeeks",
  monthly: "pastMonths",
};

/**
 * The peak of each finished period beside the running one, so a glance says
 * whether the plan is too small or too large. Shown once a period has
 * finished while Limvo was running; the history never leaves the machine.
 */
export function PastPeriods({
  window,
  history,
  replay,
}: {
  window: UsageWindow;
  history: WindowHistory | undefined;
  replay: number;
}) {
  const { t, format } = useI18n();
  const title = TITLE[window.kind];
  const past = history?.past ?? [];
  if (!title || past.length === 0) return null;

  const average = past.reduce((sum, period) => sum + period.peak, 0) / past.length;
  const bars = [
    ...past.map((period) => ({
      key: period.resetsAt,
      peak: period.peak,
      label: format.dayMonth(new Date(period.resetsAt)),
      current: false,
    })),
    { key: "now", peak: window.percent, label: t("pastNow"), current: true },
  ];

  return (
    <section className="group past" aria-label={t(title)}>
      <div className="past__head">
        <h2 className="group__title">{t(title)}</h2>
        <span className="past__average">
          {t("pastAverage", { percent: format.percent(average) })}
        </span>
      </div>
      {/* Keyed on `replay` so the bars grow again each time the flyout opens. */}
      <ol key={replay} className="past__bars">
        {bars.map((bar, index) => {
          const reading = bar.current
            ? `${bar.label}: ${format.percent(bar.peak)}`
            : t("pastBar", { period: bar.label, percent: format.percent(bar.peak) });
          return (
            <li
              key={bar.key}
              className="past__bar"
              data-current={bar.current || undefined}
              data-full={bar.peak >= 100 || undefined}
              title={reading}
              aria-label={reading}
              style={
                {
                  "--value": Math.min(100, Math.max(0, bar.peak)) / 100,
                  "--index": index,
                } as CSSProperties
              }
            >
              <span className="past__fill" />
              <span className="past__label" aria-hidden="true">
                {bar.label}
              </span>
            </li>
          );
        })}
      </ol>
    </section>
  );
}
