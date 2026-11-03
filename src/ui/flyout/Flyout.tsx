import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { bridge } from "../../bridge";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { useNow } from "../hooks";
import { ProviderMark } from "../ProviderMark";
import { Breakdown, OtherLimits } from "./Details";
import { Headline } from "./Headline";
import { LanguagePanel } from "./LanguagePanel";
import { StateNotice } from "./StateNotice";

type View = "usage" | "language";

export function Flyout({
  report,
  chosenLocale,
}: {
  report: ProviderReport;
  chosenLocale: string | null;
}) {
  const { t, format } = useI18n();
  const now = useNow();
  const [view, setView] = useState<View>("usage");
  const [refreshing, setRefreshing] = useState(false);
  const root = useFitToContent();
  useEscapeToHide(view, () => setView("usage"));
  useTrayTexts(report, now);

  const snapshot = report.snapshot;
  const providerName = snapshot?.providerName ?? "Claude";
  const fetchedAt = snapshot ? new Date(snapshot.fetchedAt) : null;
  const updated =
    fetchedAt === null
      ? null
      : now.getTime() - fetchedAt.getTime() < 60_000
        ? t("updatedJustNow")
        : t("updated", { time: format.relativeTime(fetchedAt, now) });

  const refresh = async () => {
    setRefreshing(true);
    try {
      await bridge.refresh();
    } finally {
      setRefreshing(false);
    }
  };

  if (view === "language") {
    return (
      <main ref={root} className="flyout">
        <LanguagePanel chosen={chosenLocale} onClose={() => setView("usage")} />
      </main>
    );
  }

  return (
    <main ref={root} className="flyout" aria-busy={report.state === "loading" || refreshing}>
      <header className="flyout__header">
        <ProviderMark providerId={snapshot?.providerId ?? "claude"} size={20} />
        <h1 className="flyout__title">{t("usageTitle", { provider: providerName })}</h1>
        {updated && <span className="flyout__updated">{updated}</span>}
      </header>

      <StateNotice report={report} now={now} onRetry={refresh} />

      {snapshot?.headline && <Headline window={snapshot.headline} now={now} />}
      {snapshot && <Breakdown rows={snapshot.breakdown} />}
      {snapshot && <OtherLimits windows={snapshot.windows} />}

      <footer className="flyout__footer">
        <button
          type="button"
          className="action"
          disabled={!snapshot?.usagePageUrl}
          onClick={() => bridge.openUsagePage()}
        >
          {t("openUsagePage")}
        </button>
        <button type="button" className="action" disabled={refreshing} onClick={refresh}>
          {t("refresh")}
        </button>
        <LanguageButton onOpen={() => setView("language")} />
        <button type="button" className="action" onClick={() => bridge.quit()}>
          {t("quit")}
        </button>
      </footer>
    </main>
  );
}

function LanguageButton({ onOpen }: { onOpen: () => void }) {
  const { t, locale } = useI18n();
  return (
    <button
      type="button"
      className="action action--language"
      aria-label={`${t("language")}: ${locale.nativeName}`}
      title={t("language")}
      onClick={onOpen}
    >
      <span lang={locale.code}>{locale.shortLabel}</span>
    </button>
  );
}

/**
 * The tray icon and its menu are drawn by the host, which has no translations.
 * The flyout window lives for the whole session, so it keeps them current.
 */
function useTrayTexts(report: ProviderReport, now: Date) {
  const { t, format, locale } = useI18n();
  const headline = report.snapshot?.headline;
  const title = t("usageTitle", { provider: report.snapshot?.providerName ?? "Claude" });
  const reading =
    headline?.resetsAt && new Date(headline.resetsAt) > now
      ? t("stripUntil", {
          percent: format.percent(headline.percent),
          day: format.weekday(new Date(headline.resetsAt)),
        })
      : headline
        ? format.percent(headline.percent)
        : null;
  const tooltip = reading ? `${title}: ${reading}` : title;
  const refresh = t("refresh");
  const quit = t("quit");

  // biome-ignore lint/correctness/useExhaustiveDependencies: `locale` re-sends the texts after a language switch.
  useEffect(() => {
    bridge.localizeTray({ tooltip, refresh, quit });
  }, [tooltip, refresh, quit, locale]);
}

/** Reports the content height to the host so the native window hugs it. */
function useFitToContent() {
  const root = useRef<HTMLElement>(null);
  useLayoutEffect(() => {
    const element = root.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry)
        bridge.fitFlyout(Math.ceil(entry.borderBoxSize[0]?.blockSize ?? element.offsetHeight));
    });
    observer.observe(element);
    return () => observer.disconnect();
  });
  return root;
}

/** Escape closes the language panel first, then hides the flyout. */
function useEscapeToHide(view: View, closePanel: () => void) {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      if (view === "language") closePanel();
      else bridge.hideFlyout();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [view, closePanel]);
}
