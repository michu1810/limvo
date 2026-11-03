import type { ProviderReport } from "../core/snapshot";

export interface Settings {
  /** BCP 47 code chosen by the user, or `null` to follow the system language. */
  locale: string | null;
}

export type Theme = "light" | "dark";

export interface HostInfo {
  /** `mica` when the native window material shows through; `none` means paint a solid surface. */
  material: "mica" | "none";
  /** The taskbar follows the Windows mode, which can differ from the app mode. */
  taskbarTheme: Theme;
}

export interface TrayTexts {
  tooltip: string;
  refresh: string;
  quit: string;
}

export type Unsubscribe = () => void;

/**
 * Everything the UI may ask of the host. The Tauri implementation talks to the
 * Rust core; the browser implementation serves scenarios for development.
 */
export interface Bridge {
  getReport(): Promise<ProviderReport>;
  onReport(listener: (report: ProviderReport) => void): Promise<Unsubscribe>;
  refresh(): Promise<void>;

  getSettings(): Promise<Settings>;
  onSettings(listener: (settings: Settings) => void): Promise<Unsubscribe>;
  setLocale(locale: string | null): Promise<void>;

  /** Preferred UI languages of the operating system, most preferred first. */
  systemLocales(): Promise<readonly string[]>;
  hostInfo(): Promise<HostInfo>;
  onTaskbarTheme(listener: (theme: Theme) => void): Promise<Unsubscribe>;
  /** Hands translated texts to the tray icon and its menu, which the host draws. */
  localizeTray(texts: TrayTexts): Promise<void>;

  openUsagePage(): Promise<void>;
  /** Tells the host how tall the flyout content is, so the window can fit it. */
  fitFlyout(height: number): Promise<void>;
  /** Tells the host how wide the taskbar strip content is. */
  fitStrip(width: number): Promise<void>;
  toggleFlyout(): Promise<void>;
  hideFlyout(): Promise<void>;
  quit(): Promise<void>;
}
