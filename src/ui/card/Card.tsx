import type { PointerEvent } from "react";
import { bridge, type Settings } from "../../bridge";
import { pinnedLimits, visibleReports } from "../../core/pins";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { Headline } from "../flyout/Headline";
import { planAttribute, useNow } from "../hooks";
import { ProviderMark } from "../ProviderMark";

/**
 * The reading pinned to the desktop: the service of the first taskbar pin.
 * The whole card is a drag handle; a double click opens the full flyout.
 */
export function Card({
  reports,
  settings,
}: {
  reports: ProviderReport[];
  settings: Settings | null;
}) {
  const report =
    pinnedLimits(reports, settings?.pinned ?? null)[0]?.report ??
    visibleReports(reports)[0] ??
    reports[0];
  const { t, format } = useI18n();
  const now = useNow();
  const snapshot = report?.snapshot ?? null;
  const fetchedAt = snapshot ? new Date(snapshot.fetchedAt) : null;

  const startDrag = (event: PointerEvent) => {
    if (event.button === 0 && event.detail === 1) bridge.dragCard();
  };

  return (
    <main
      className="flyout card"
      data-plan={planAttribute(report, settings)}
      data-stale={report?.state !== "ok" || undefined}
      onPointerDown={startDrag}
      onDoubleClick={() => bridge.toggleFlyout()}
    >
      <header className="flyout__header">
        <ProviderMark
          providerId={report?.serviceId ?? "claude"}
          size={16}
          level={snapshot?.headline?.percent ?? null}
        />
        <h1 className="flyout__title">
          {t("usageTitle", { provider: report?.providerName ?? "Claude" })}
        </h1>
        {fetchedAt && (
          <span className="flyout__updated">
            {now.getTime() - fetchedAt.getTime() < 60_000
              ? t("updatedJustNow")
              : format.relativeTime(fetchedAt, now)}
          </span>
        )}
      </header>
      {snapshot?.headline ? (
        <Headline window={snapshot.headline} now={now} plan={snapshot.plan} compact />
      ) : (
        <p className="card__empty">{t("stateLoading")}</p>
      )}
    </main>
  );
}
