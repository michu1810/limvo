import type { ProviderReport } from "../core/snapshot";
import { SCENARIOS, type ScenarioName } from "./scenarios";
import type { Bridge, Settings } from "./types";

/**
 * Development host for `npm run dev` in a plain browser. Query parameters:
 * `scenario` (see `SCENARIOS`), `locale` (for example `he`) and `now`
 * (an ISO instant, for reproducible screenshots).
 */
export function createBrowserBridge(): Bridge {
  const params = new URLSearchParams(window.location.search);
  const scenario = (params.get("scenario") ?? "reference") as ScenarioName;
  const fixedNow = params.get("now");
  const now = () => (fixedNow ? new Date(fixedNow) : new Date());
  const build = SCENARIOS[scenario] ?? SCENARIOS.reference;

  let report: ProviderReport = build(now());
  let settings: Settings = { locale: params.get("locale") };
  const reportListeners = new Set<(report: ProviderReport) => void>();
  const settingsListeners = new Set<(settings: Settings) => void>();

  const subscribe = <T>(listeners: Set<(value: T) => void>, listener: (value: T) => void) => {
    listeners.add(listener);
    return Promise.resolve(() => {
      listeners.delete(listener);
    });
  };

  return {
    getReport: () => Promise.resolve(report),
    onReport: (listener) => subscribe(reportListeners, listener),
    async refresh() {
      report = build(now());
      for (const listener of reportListeners) listener(report);
    },

    getSettings: () => Promise.resolve(settings),
    onSettings: (listener) => subscribe(settingsListeners, listener),
    async setLocale(locale) {
      settings = { ...settings, locale };
      for (const listener of settingsListeners) listener(settings);
    },

    systemLocales: () => Promise.resolve(navigator.languages),
    hostInfo: () =>
      Promise.resolve({
        material: "none",
        taskbarTheme: matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light",
      }),
    onTaskbarTheme: () => Promise.resolve(() => {}),
    localizeTray: () => Promise.resolve(),

    async openUsagePage() {
      const url = report.snapshot?.usagePageUrl;
      if (url) window.open(url, "_blank", "noopener");
    },
    fitFlyout: () => Promise.resolve(),
    fitStrip: () => Promise.resolve(),
    toggleFlyout: () => Promise.resolve(),
    hideFlyout: () => Promise.resolve(),
    quit: () => Promise.resolve(),
  };
}
