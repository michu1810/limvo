import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { bridge, PRIMARY_PROVIDER, type Settings } from "../../bridge";
import { pinnedLimits, visibleReports } from "../../core/pins";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { planAttribute, useNow, useShowCount } from "../hooks";
import { SettingsIcon } from "../icons";
import { ProviderMark } from "../ProviderMark";
import { useAlerts } from "../useAlerts";
import { useWeekHistory } from "../useWeekHistory";
import { Breakdown, OtherLimits } from "./Details";
import { Headline } from "./Headline";
import { LanguagePanel } from "./LanguagePanel";
import { ProviderTabs } from "./ProviderTabs";
import { SettingsPanel } from "./SettingsPanel";
import { StateNotice } from "./StateNotice";

type View = "usage" | "settings" | "language";

/** Depth of each view, so moving deeper slides forward and going back slides back. */
const DEPTH: Record<View, number> = { usage: 0, settings: 1, language: 2 };
const TAB_KEY = "limvo.tab";

export function Flyout({
  reports,
  settings,
}: {
  reports: ProviderReport[];
  settings: Settings | null;
}) {
  const { t, format } = useI18n();
  const now = useNow();
  const shown = useShowCount();
  const [view, setView] = useState<View>("usage");
  const [direction, setDirection] = useState<"enter" | "forward" | "back">("enter");
  const [languageFrom, setLanguageFrom] = useState<View>("usage");
  const [refreshing, setRefreshing] = useState(false);
  const [tab, setTab] = useState(() => readTab());
  const root = useFitToContent();

  const visible = visibleReports(reports);
  const report =
    visible.find((candidate) => candidate.providerId === tab) ??
    visible.find((candidate) => candidate.providerId === PRIMARY_PROVIDER) ??
    visible[0] ??
    reports[0];

  useTrayTexts(reports, settings, now);
  useAlerts(reports, settings);
  const histories = useWeekHistory(reports);

  const go = (next: View) => {
    setDirection(DEPTH[next] >= DEPTH[view] ? "forward" : "back");
    setView(next);
  };
  const openLanguage = () => {
    setLanguageFrom(view);
    go("language");
  };
  const back = () => go(view === "language" ? languageFrom : "usage");
  const selectTab = (providerId: string) => {
    setDirection("forward");
    setTab(providerId);
    saveTab(providerId);
  };
  useEscape(() => (view === "usage" ? bridge.hideFlyout() : back()));

  // Every time the flyout opens it starts on the readings and plays its arrival.
  // biome-ignore lint/correctness/useExhaustiveDependencies: runs on each show only.
  useEffect(() => {
    setDirection("enter");
    setView("usage");
  }, [shown]);

  // The button only asks; the busy state ends when the new reports arrive.
  // biome-ignore lint/correctness/useExhaustiveDependencies: new reports end the refresh.
  useEffect(() => setRefreshing(false), [reports]);
  const refresh = () => {
    setRefreshing(true);
    bridge.refresh();
  };

  if (!report) return null;
  const snapshot = report.snapshot;
  const fetchedAt = snapshot ? new Date(snapshot.fetchedAt) : null;
  const updated =
    fetchedAt === null
      ? null
      : now.getTime() - fetchedAt.getTime() < 60_000
        ? t("updatedJustNow")
        : t("updated", { time: format.relativeTime(fetchedAt, now) });

  return (
    <main
      ref={root}
      className="flyout"
      data-plan={planAttribute(report, settings)}
      aria-busy={report.state === "loading" || refreshing}
    >
      <div
        key={`${view}:${shown}:${report.providerId}`}
        className="view"
        data-direction={direction}
      >
        {view === "settings" && (
          <SettingsPanel
            settings={settings}
            reports={reports}
            onBack={back}
            onLanguage={openLanguage}
          />
        )}
        {view === "language" && <LanguagePanel chosen={settings?.locale ?? null} onClose={back} />}
        {view === "usage" && (
          <>
            {visible.length > 1 && (
              <ProviderTabs
                reports={visible}
                settings={settings}
                selected={report.providerId}
                onSelect={selectTab}
              />
            )}
            <header className="flyout__header">
              <ProviderMark
                providerId={report.providerId}
                size={20}
                level={snapshot?.headline?.percent ?? null}
              />
              <h1 className="flyout__title">
                {t("usageTitle", { provider: report.providerName })}
              </h1>
              {updated && (
                <span className="flyout__updated" data-busy={refreshing || undefined}>
                  {updated}
                </span>
              )}
              <button
                type="button"
                className="icon-button icon-button--end"
                aria-label={t("settings")}
                title={t("settings")}
                onClick={() => go("settings")}
              >
                <SettingsIcon />
              </button>
              <span className="flyout__progress" data-active={refreshing || undefined} />
            </header>

            <StateNotice report={report} now={now} onRetry={refresh} />

            {snapshot?.headline && (
              <Headline
                window={snapshot.headline}
                now={now}
                replay={shown}
                plan={snapshot.plan}
                history={histories[report.providerId]}
              />
            )}
            {snapshot && <Breakdown rows={snapshot.breakdown} replay={shown} />}
            {snapshot && (
              <OtherLimits
                windows={snapshot.windows}
                extraUsage={snapshot.extraUsage}
                replay={shown}
              />
            )}

            <footer className="flyout__footer">
              <button
                type="button"
                className="action"
                disabled={!snapshot?.usagePageUrl}
                onClick={() => bridge.openUsagePage(report.providerId)}
              >
                {t("openUsagePage")}
              </button>
              <button type="button" className="action" disabled={refreshing} onClick={refresh}>
                {t("refresh")}
              </button>
              <LanguageButton onOpen={openLanguage} />
              <button type="button" className="action" onClick={() => bridge.quit()}>
                {t("quit")}
              </button>
            </footer>
          </>
        )}
      </div>
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
 * The tooltip lists the pinned limits, one service per line.
 */
function useTrayTexts(reports: ProviderReport[], settings: Settings | null, now: Date) {
  const { t, format, locale } = useI18n();
  const lines = pinnedLimits(reports, settings?.pinned ?? null).map(({ report, window }) => {
    const reading =
      window.resetsAt && new Date(window.resetsAt) > now
        ? t("stripUntil", {
            percent: format.percent(window.percent),
            day: format.weekday(new Date(window.resetsAt)),
          })
        : format.percent(window.percent);
    return `${report.providerName}: ${reading}`;
  });
  const tooltip = lines.length > 0 ? ["Limvo", ...lines].join("\n") : "Limvo";
  const refresh = t("refresh");
  const quit = t("quit");

  // biome-ignore lint/correctness/useExhaustiveDependencies: `locale` re-sends the texts after a language switch.
  useEffect(() => {
    bridge.localizeTray({ tooltip, refresh, quit });
  }, [tooltip, refresh, quit, locale]);
}

function readTab(): string {
  try {
    return localStorage.getItem(TAB_KEY) ?? PRIMARY_PROVIDER;
  } catch {
    return PRIMARY_PROVIDER;
  }
}

function saveTab(providerId: string) {
  try {
    localStorage.setItem(TAB_KEY, providerId);
  } catch {
    // Without storage the flyout simply opens on the first service.
  }
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
  }, []);
  return root;
}

function useEscape(onEscape: () => void) {
  const handler = useRef(onEscape);
  handler.current = onEscape;
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") handler.current();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
