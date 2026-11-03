import type { ProviderReport } from "../core/snapshot";
import { SCENARIOS, type ScenarioName } from "./scenarios";
import type { Bridge, Settings } from "./types";

/**
 * Development host for `npm run dev` in a plain browser. Query parameters:
 * `scenario` (see `SCENARIOS`), `locale` (for example `he`) and `now`
 * (an ISO instant, for reproducible screenshots). Refresh cycles through a
 * small change so motion can be checked without the app.
 */
export function createBrowserBridge(): Bridge {
  const params = new URLSearchParams(window.location.search);
  const scenario = (params.get("scenario") ?? "reference") as ScenarioName;
  const fixedNow = params.get("now");
  const now = () => (fixedNow ? new Date(fixedNow) : new Date());
  const build = SCENARIOS[scenario] ?? SCENARIOS.reference;

  let report: ProviderReport = build(now());
  let settings: Settings = {
    locale: params.get("locale"),
    showStrip: true,
    notifications: true,
    refresh: "adaptive",
    desktopCard: false,
  };
  let launchAtLogin = true;
  let refreshes = 0;
  const reportListeners = new Set<(report: ProviderReport) => void>();
  const settingsListeners = new Set<(settings: Settings) => void>();
  const shownListeners = new Set<() => void>();

  const subscribe = <T>(listeners: Set<(value: T) => void>, listener: (value: T) => void) => {
    listeners.add(listener);
    return Promise.resolve(() => {
      listeners.delete(listener);
    });
  };

  // A browser tab is "shown" once, when it loads.
  queueMicrotask(() => {
    for (const listener of shownListeners) listener();
  });

  return {
    getReport: () => Promise.resolve(report),
    onReport: (listener) => subscribe(reportListeners, listener),
    async refresh() {
      refreshes += 1;
      const next = build(now());
      const headline = next.snapshot?.headline;
      if (headline) headline.percent = Math.min(100, headline.percent + (refreshes % 3) * 4);
      if (next.snapshot) next.snapshot.fetchedAt = now().toISOString();
      report = next;
      for (const listener of reportListeners) listener(report);
    },

    getSettings: () => Promise.resolve(settings),
    onSettings: (listener) => subscribe(settingsListeners, listener),
    async updateSettings(patch) {
      settings = { ...settings, ...patch };
      for (const listener of settingsListeners) listener(settings);
      return settings;
    },
    getLaunchAtLogin: () => Promise.resolve(launchAtLogin),
    async setLaunchAtLogin(enabled) {
      launchAtLogin = enabled;
      return enabled;
    },

    systemLocales: () => Promise.resolve(navigator.languages),
    hostInfo: () =>
      Promise.resolve({
        material: "none",
        taskbarTheme: matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light",
      }),
    onTaskbarTheme: () => Promise.resolve(() => {}),
    localizeTray: () => Promise.resolve(),
    async notify(toast) {
      console.info("[notification]", toast.title, toast.body);
    },
    onFlyoutShown: (listener) => {
      shownListeners.add(listener);
      return Promise.resolve(() => {
        shownListeners.delete(listener);
      });
    },

    async openUsagePage() {
      const url = report.snapshot?.usagePageUrl;
      if (url) window.open(url, "_blank", "noopener");
    },
    fitFlyout: () => Promise.resolve(),
    fitStrip: () => Promise.resolve(),
    toggleFlyout: () => Promise.resolve(),
    hideFlyout: () => Promise.resolve(),
    dragCard: () => Promise.resolve(),
    quit: () => Promise.resolve(),
  };
}
