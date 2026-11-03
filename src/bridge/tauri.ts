import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import type { ProviderReport } from "../core/snapshot";
import type { Bridge, HostInfo, Settings, Theme } from "./types";

/** Event names shared with `src-tauri/src/events.rs`. */
const REPORT_EVENT = "report-updated";
const SETTINGS_EVENT = "settings-updated";
const TASKBAR_THEME_EVENT = "taskbar-theme";
const FLYOUT_SHOWN_EVENT = "flyout-shown";

export const tauriBridge: Bridge = {
  getReport: () => invoke<ProviderReport>("get_report"),
  onReport: (listener) => listen<ProviderReport>(REPORT_EVENT, (event) => listener(event.payload)),
  refresh: () => invoke("refresh"),

  getSettings: () => invoke<Settings>("get_settings"),
  onSettings: (listener) => listen<Settings>(SETTINGS_EVENT, (event) => listener(event.payload)),
  updateSettings: (patch) => invoke<Settings>("update_settings", { patch }),
  getLaunchAtLogin: () => invoke<boolean>("get_launch_at_login"),
  setLaunchAtLogin: (enabled) => invoke<boolean>("set_launch_at_login", { enabled }),
  shortcutLabel: () => invoke<string | null>("shortcut_label"),

  systemLocales: () => invoke<string[]>("system_locales"),
  hostInfo: () => invoke<HostInfo>("host_info"),
  onTaskbarTheme: (listener) =>
    listen<Theme>(TASKBAR_THEME_EVENT, (event) => listener(event.payload)),
  localizeTray: (texts) => invoke("localize_tray", { ...texts }),
  async notify(toast) {
    const granted = (await isPermissionGranted()) || (await requestPermission()) === "granted";
    if (granted) sendNotification(toast);
  },
  onFlyoutShown: (listener) => listen(FLYOUT_SHOWN_EVENT, () => listener()),

  openUsagePage: () => invoke("open_usage_page"),
  fitFlyout: (height) => invoke("fit_flyout", { height }),
  fitStrip: (width) => invoke("fit_strip", { width }),
  toggleFlyout: () => invoke("toggle_flyout"),
  hideFlyout: () => invoke("hide_flyout"),
  dragCard: () => getCurrentWindow().startDragging(),
  quit: () => invoke("quit"),
};
