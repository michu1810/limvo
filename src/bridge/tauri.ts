import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ProviderReport } from "../core/snapshot";
import type { Bridge, HostInfo, Settings, Theme } from "./types";

/** Event names shared with `src-tauri/src/events.rs`. */
const REPORT_EVENT = "report-updated";
const SETTINGS_EVENT = "settings-updated";
const TASKBAR_THEME_EVENT = "taskbar-theme";

export const tauriBridge: Bridge = {
  getReport: () => invoke<ProviderReport>("get_report"),
  onReport: (listener) => listen<ProviderReport>(REPORT_EVENT, (event) => listener(event.payload)),
  refresh: () => invoke("refresh"),

  getSettings: () => invoke<Settings>("get_settings"),
  onSettings: (listener) => listen<Settings>(SETTINGS_EVENT, (event) => listener(event.payload)),
  setLocale: (locale) => invoke("set_locale", { locale }),

  systemLocales: () => invoke<string[]>("system_locales"),
  hostInfo: () => invoke<HostInfo>("host_info"),
  onTaskbarTheme: (listener) =>
    listen<Theme>(TASKBAR_THEME_EVENT, (event) => listener(event.payload)),
  localizeTray: (texts) => invoke("localize_tray", { ...texts }),

  openUsagePage: () => invoke("open_usage_page"),
  fitFlyout: (height) => invoke("fit_flyout", { height }),
  fitStrip: (width) => invoke("fit_strip", { width }),
  toggleFlyout: () => invoke("toggle_flyout"),
  hideFlyout: () => invoke("hide_flyout"),
  quit: () => invoke("quit"),
};
