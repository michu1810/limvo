import type { CSSProperties, KeyboardEvent } from "react";
import type { Settings } from "../../bridge";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { planAttribute } from "../hooks";
import { ProviderMark } from "../ProviderMark";

/**
 * One tab per signed-in service. A thin indicator slides to the chosen tab;
 * each tab shows the service's mark and its main reading, so a glance at the
 * tabs alone compares them.
 */
export function ProviderTabs({
  reports,
  settings,
  selected,
  onSelect,
}: {
  reports: readonly ProviderReport[];
  settings: Settings | null;
  selected: string;
  onSelect: (providerId: string) => void;
}) {
  const { t, format } = useI18n();
  const index = Math.max(
    0,
    reports.findIndex((report) => report.providerId === selected),
  );

  // Arrow keys move between tabs, as in any Windows tab list.
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key];
    if (!step) return;
    const direction = document.documentElement.dir === "rtl" ? -step : step;
    const next = reports[(index + direction + reports.length) % reports.length];
    if (next) onSelect(next.providerId);
  };

  return (
    <div
      className="tabs"
      // Four services leave no room for names: marks and readings carry the tabs.
      data-compact={reports.length > 3 || undefined}
      role="tablist"
      aria-label={t("services")}
      style={{ "--tabs": reports.length, "--tab": index } as CSSProperties}
      onKeyDown={onKeyDown}
    >
      <span className="tabs__indicator" aria-hidden="true" />
      {reports.map((report) => {
        const active = report.providerId === selected;
        const headline = report.snapshot?.headline;
        return (
          <button
            key={report.providerId}
            type="button"
            role="tab"
            className="tabs__tab"
            data-plan={planAttribute(report, settings)}
            aria-selected={active}
            aria-label={
              headline
                ? `${report.providerName} ${format.percent(headline.percent)}`
                : report.providerName
            }
            title={report.providerName}
            tabIndex={active ? 0 : -1}
            onClick={() => onSelect(report.providerId)}
          >
            <ProviderMark
              providerId={report.providerId}
              size={16}
              level={headline?.percent ?? null}
            />
            <span className="tabs__name">{report.providerName}</span>
            {headline && <span className="tabs__value">{format.percent(headline.percent)}</span>}
          </button>
        );
      })}
    </div>
  );
}
