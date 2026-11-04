import type { ProviderReport } from "../core/snapshot";
import { SCENARIOS, type ScenarioName, scenarioReports } from "./scenarios";
import type { Bridge, Settings } from "./types";

/**
 * Development host for `npm run dev` in a plain browser. Query parameters:
 * `scenario` (see `SCENARIOS`), `codex=off`, `more` (adds Cursor and
 * Antigravity), `pins`, `locale` (for example `he`) and
 * `now` (an ISO instant, for reproducible screenshots) and `update` (a
 * version to offer, for example `0.2.0`). Refresh cycles
 * through a small change so motion can be checked without the app.
 */
export function createBrowserBridge(): Bridge {
  const params = new URLSearchParams(window.location.search);
  const requested = params.get("scenario") ?? "reference";
  const name: ScenarioName = requested in SCENARIOS ? (requested as ScenarioName) : "reference";
  const withCodex = params.get("codex") !== "off";
  const fixedNow = params.get("now");
  const now = () => (fixedNow ? new Date(fixedNow) : new Date());
  const withMore = params.has("more");
  const build = () => scenarioReports(name, now(), withCodex, withMore);

  let reports: ProviderReport[] = build();
  let settings: Settings = {
    locale: params.get("locale"),
    showStrip: true,
    stripAllMonitors: true,
    notifications: true,
    refresh: "adaptive",
    desktopCard: false,
    shortcut: true,
    planColors: true,
    pinned: params.get("pins")?.split(",") ?? null,
    checkUpdates: true,
  };
  const offered = params.get("update");
  const update = offered ? { version: offered, currentVersion: __APP_VERSION__ } : null;
  let launchAtLogin = true;
  let refreshes = 0;
  const reportListeners = new Set<(reports: ProviderReport[]) => void>();
  const settingsListeners = new Set<(settings: Settings) => void>();
  const shownListeners = new Set<() => void>();

  const subscribe = <T>(listeners: Set<(value: T) => void>, listener: (value: T) => void) => {
    listeners.add(listener);
    return Promise.resolve(() => {
      listeners.delete(listener);
    });
  };

  // A browser tab is "shown" once, after it loads and React has subscribed.
  window.addEventListener("load", () => {
    window.setTimeout(() => {
      for (const listener of shownListeners) listener();
    }, 60);
  });

  return {
    getReports: () => Promise.resolve(reports),
    onReports: (listener) => subscribe(reportListeners, listener),
    async refresh() {
      refreshes += 1;
      const next = build();
      for (const report of next) {
        const headline = report.snapshot?.headline;
        if (headline) headline.percent = Math.min(100, headline.percent + (refreshes % 3) * 4);
        if (report.snapshot) report.snapshot.fetchedAt = now().toISOString();
      }
      reports = next;
      for (const listener of reportListeners) listener(reports);
    },

    getSettings: () => Promise.resolve(settings),
    onSettings: (listener) => subscribe(settingsListeners, listener),
    async updateSettings(patch) {
      settings = { ...settings, ...patch };
      for (const listener of settingsListeners) listener(settings);
      return settings;
    },
    getLaunchAtLogin: () => Promise.resolve(launchAtLogin),
    shortcutLabel: () => Promise.resolve(settings.shortcut ? "Win+Shift+H" : null),
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

    getUpdate: () => Promise.resolve(update),
    onUpdate: () => Promise.resolve(() => {}),
    async installUpdate() {
      console.info("[update] would install", update?.version);
    },

    async openUsagePage(providerId) {
      const url = reports.find((report) => report.providerId === providerId)?.snapshot
        ?.usagePageUrl;
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
