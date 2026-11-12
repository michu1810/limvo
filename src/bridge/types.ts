import type { ProviderReport } from "../core/snapshot";

export type RefreshMode = "adaptive" | "2m" | "5m" | "15m";

/** Levels a main limit is announced at: 80/95%, 50/75/90%, or only when reached. */
export type NotifyLevels = "standard" | "early" | "reached";

/** A service that can have extra accounts, each signed in from its own folder. */
export type AccountService = "claude" | "codex";

/** Mirrors `Account` in `src-tauri/src/accounts.rs`. */
export interface Account {
  /** `<service>~<n>`, the account's provider id. */
  id: string;
  service: AccountService;
  /** Shown so the user recognises the account; the interface never sends a path. */
  folder: string;
  label: string;
}

/** Why the host refused a folder; `cancelled` means the dialog was closed. */
export type AccountError = "cancelled" | "too_many" | "default_folder" | "duplicate" | "no_sign_in";

/** Mirrors `Settings` in `src-tauri/src/settings.rs`. */
export interface Settings {
  /** BCP 47 code chosen by the user, or `null` to follow the system language. */
  locale: string | null;
  showStrip: boolean;
  /** A reading on the taskbar of every monitor. */
  stripAllMonitors: boolean;
  notifications: boolean;
  /** When the main limit is announced on its way up. */
  notifyLevels: NotifyLevels;
  /** The early warning when the pace would empty the main limit before its reset. */
  notifyForecast: boolean;
  refresh: RefreshMode;
  desktopCard: boolean;
  /** The system-wide shortcut that opens the flyout. */
  shortcut: boolean;
  /** Tint the interface with the colour of the account's plan. */
  planColors: boolean;
  /** Limits in the taskbar as `<provider>:<window>`; `null` shows each provider's main limit. */
  pinned: string[] | null;
  /** Look for a newer Limvo on GitHub Releases now and then. */
  checkUpdates: boolean;
  /** Extra accounts. Changed only through the account calls below. */
  accounts: Account[];
}

/** Only the fields named in a patch change; the host keeps everything else. */
export type SettingsPatch = Partial<Omit<Settings, "accounts">>;

export type Theme = "light" | "dark";

export interface HostInfo {
  /** `mica` when the native window material shows through; `none` means paint a solid surface. */
  material: "mica" | "none";
  /** The taskbar follows the Windows mode, which can differ from the app mode. */
  taskbarTheme: Theme;
  /** Installed from the Microsoft Store, which delivers the updates itself. */
  store: boolean;
}

/** Mirrors `CardData` in `src-tauri/src/widgets/mod.rs`: the Widgets board card, already worded. */
export interface WidgetCard {
  items: { name: string; value: string; detail: string; hot: boolean }[];
  /** Shown when no service is signed in. */
  empty: string;
  /** Label of the card's refresh button. */
  refresh: string;
}

export interface TrayTexts {
  tooltip: string;
  refresh: string;
  quit: string;
}

/** Mirrors `AvailableUpdate` in `src-tauri/src/updater.rs`. */
export interface AvailableUpdate {
  version: string;
  currentVersion: string;
}

export interface Toast {
  title: string;
  body: string;
}

export type Unsubscribe = () => void;

/**
 * Everything the UI may ask of the host. The Tauri implementation talks to the
 * Rust core; the browser implementation serves scenarios for development.
 */
export interface Bridge {
  /** One report per provider, in the order the interface lists them. */
  getReports(): Promise<ProviderReport[]>;
  onReports(listener: (reports: ProviderReport[]) => void): Promise<Unsubscribe>;
  refresh(): Promise<void>;

  getSettings(): Promise<Settings>;
  onSettings(listener: (settings: Settings) => void): Promise<Unsubscribe>;
  updateSettings(patch: SettingsPatch): Promise<Settings>;
  getLaunchAtLogin(): Promise<boolean>;
  setLaunchAtLogin(enabled: boolean): Promise<boolean>;
  /** The shortcut that registered, such as "Win+Shift+H", or `null` when none is active. */
  shortcutLabel(): Promise<string | null>;

  /** Preferred UI languages of the operating system, most preferred first. */
  systemLocales(): Promise<readonly string[]>;
  hostInfo(): Promise<HostInfo>;
  onTaskbarTheme(listener: (theme: Theme) => void): Promise<Unsubscribe>;
  /** Hands translated texts to the tray icon and its menu, which the host draws. */
  localizeTray(texts: TrayTexts): Promise<void>;
  /**
   * Names the current window, for Task Manager and the window list: the
   * window's title and its page's title, which also names its WebView2 process.
   */
  nameWindow(title: string): Promise<void>;
  /** Hands the Widgets board card its content; only the Store build shows it. */
  updateWidget(card: WidgetCard): Promise<void>;
  /** Shows a Windows notification. */
  notify(toast: Toast): Promise<void>;
  /** Fires each time the flyout is brought on screen, so it can play its entrance. */
  onFlyoutShown(listener: () => void): Promise<Unsubscribe>;

  /**
   * Opens the system folder dialog titled `title` and adds the account signed
   * in there. Rejects with an `AccountError` code.
   */
  addAccount(service: AccountService, label: string, title: string): Promise<Settings>;
  removeAccount(id: string): Promise<Settings>;
  renameAccount(id: string, label: string): Promise<Settings>;

  /** The newer version found by the host's last check, or `null`. */
  getUpdate(): Promise<AvailableUpdate | null>;
  onUpdate(listener: (update: AvailableUpdate) => void): Promise<Unsubscribe>;
  /** Downloads and runs the verified installer; Limvo restarts as the new version. */
  installUpdate(): Promise<void>;

  openUsagePage(providerId: string): Promise<void>;
  /** Tells the host how tall the flyout content is, so the window can fit it. */
  fitFlyout(height: number): Promise<void>;
  /** Tells the host how wide the taskbar strip content is. */
  fitStrip(width: number): Promise<void>;
  toggleFlyout(): Promise<void>;
  hideFlyout(): Promise<void>;
  /** Moves the desktop card while the pointer is held down on it. */
  dragCard(): Promise<void>;
  quit(): Promise<void>;
}
