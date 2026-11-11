import { useEffect } from "react";
import { bridge, type Settings } from "../bridge";
import type { WidgetCard } from "../bridge/types";
import { computePace } from "../core/pace";
import { pinnedLimits } from "../core/pins";
import type { ProviderReport } from "../core/snapshot";
import { useI18n } from "../i18n/I18nProvider";
import { PACE_MESSAGE, windowLabel } from "./labels";

/**
 * Words the content of the card on the Windows Widgets board: the pinned
 * limits, as the taskbar shows them. The host draws the card and has no
 * translations, so the flyout window (alive for the whole session) sends
 * it whenever it changes. Only the Store build has a board to send it to.
 */
export function useWidgetCard(reports: ProviderReport[], settings: Settings | null, now: Date) {
  const i18n = useI18n();
  const { t, format } = i18n;

  const items = pinnedLimits(reports, settings?.pinned ?? null).map(
    ({ report, window, headline }) => {
      const pace = computePace(window, now);
      const resetsAt = window.resetsAt ? new Date(window.resetsAt) : null;
      const detail = [
        pace ? t(PACE_MESSAGE[pace.status]) : null,
        resetsAt && resetsAt > now ? format.weekdayTime(resetsAt) : null,
      ]
        .filter((part): part is string => part !== null)
        .join(" · ");
      return {
        name: headline
          ? report.providerName
          : `${report.providerName} · ${windowLabel(window, i18n)}`,
        value: format.percent(window.percent),
        detail,
        hot: pace?.status === "running_hot" || pace?.status === "limit_reached",
      };
    },
  );
  const card: WidgetCard = { items, empty: t("widgetEmpty"), refresh: t("refresh") };
  // Compared as text, so a new clock minute without a new reading sends nothing.
  const content = JSON.stringify(card);

  useEffect(() => {
    bridge.updateWidget(JSON.parse(content) as WidgetCard);
  }, [content]);
}
