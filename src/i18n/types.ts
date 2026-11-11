/**
 * Every string the UI shows. Placeholders in `{braces}` are filled by `fill()`.
 * Numbers, dates and durations are not part of these strings: they are
 * formatted with `Intl` for the active locale, so translators never handle them.
 */
export interface Messages {
  /** Window title, for example "Claude Usage". Placeholder: `{provider}`. */
  usageTitle: string;
  /** Placeholder `{time}` is a relative time such as "3 min ago". */
  updated: string;
  updatedJustNow: string;

  headlineWeekly: string;
  headlineSession: string;
  headlineMonthly: string;
  /** Placeholder `{duration}`, for example "1 day 16 hr". */
  resetsIn: string;
  resetsSoon: string;

  paceOnTrack: string;
  paceTight: string;
  paceHot: string;
  paceReached: string;
  /** Placeholders `{elapsed}` and `{projected}` are formatted percentages. */
  paceDetailWeekly: string;
  paceDetailSession: string;
  paceDetailMonthly: string;

  byProductWeekly: string;
  otherLimits: string;
  windowSession: string;
  windowWeekly: string;
  /** Placeholder `{name}` is the model name, for example "Fable". */
  windowScoped: string;
  windowMonthly: string;
  windowMonthlyNamed: string;

  openUsagePage: string;
  refresh: string;
  quit: string;
  language: string;
  systemLanguage: string;
  back: string;

  /** Taskbar label. Placeholders `{percent}` and `{day}`, for example "51% until Tue". */
  stripUntil: string;

  stateLoading: string;
  /** Placeholder `{time}` is a relative time. */
  stateOffline: string;
  /** Placeholder `{provider}`. */
  stateAuthRequired: string;
  /** Placeholder `{provider}`. */
  stateNoCredentials: string;
  stateError: string;
  /** The service asked to slow down before a first reading. Placeholder `{provider}`. */
  stateRateLimited: string;
  retry: string;

  settings: string;
  launchAtLogin: string;
  showStrip: string;
  stripAllMonitors: string;
  stripAllMonitorsHint: string;
  desktopCard: string;
  notifications: string;
  notificationsHint: string;
  refreshFrequency: string;
  refreshAdaptive: string;
  refreshAdaptiveHint: string;
  /** Placeholder `{interval}`, for example "5 min". */
  refreshEvery: string;

  /** Placeholders `{window}` and `{percent}`. */
  notifyThresholdTitle: string;
  /** Placeholder `{when}`, a weekday and time. Also used when a limit is reached. */
  notifyThresholdBody: string;
  /** Placeholder `{window}`. */
  notifyReachedTitle: string;
  /** Placeholder `{window}`. */
  notifyResetTitle: string;
  notifyResetBody: string;
  /** The pace would empty the window before its reset. Placeholders `{window}` and `{when}`, a weekday and time. */
  notifyForecastTitle: string;
  /** Placeholders `{day}`, as in `budgetUntil`, and `{rate}`, a percentage. */
  notifyForecastBodyDay: string;
  /** Placeholders `{day}` and `{rate}`. */
  notifyForecastBodyHour: string;

  extraUsage: string;
  /** Placeholders `{used}` and `{limit}` are formatted amounts of money. */
  extraUsageOf: string;
  /** Placeholder `{when}`, a weekday and time before the reset. */
  forecastLimitAt: string;
  /** Accessible description of the weekly chart. */
  weekChart: string;
  /** Opens the budget line. Placeholder `{day}`, a weekday, or a weekday and time on the last day. */
  budgetUntil: string;
  /** Placeholder `{rate}`, a percentage that may have one decimal, for example "13.9%". */
  budgetPerDay: string;
  /** Placeholder `{rate}`. */
  budgetPerHour: string;
  /** The rate so far, beside the budget. Placeholder `{rate}`. */
  budgetNow: string;

  shortcut: string;
  /** Placeholder `{keys}`, for example "Win+Shift+H". */
  shortcutHint: string;
  shortcutTaken: string;
  planColors: string;
  planColorsHint: string;
  services: string;
  taskbarPins: string;
  taskbarPinsHint: string;

  checkUpdates: string;
  checkUpdatesHint: string;
  /** Placeholder `{version}`, for example "0.2.0". */
  updateAvailable: string;
  updateInstall: string;
  updateLater: string;
  updateInstalling: string;
  updateFailed: string;

  accounts: string;
  accountsHint: string;
  addAccount: string;
  accountName: string;
  chooseFolder: string;
  /** Title of the system folder dialog. Placeholder `{service}`, "Claude" or "Codex". */
  accountFolderTitle: string;
  /** Placeholder `{name}`, for example "Claude · Work". */
  removeAccount: string;
  /** Placeholder `{count}`. */
  accountErrorTooMany: string;
  accountErrorDefault: string;
  accountErrorDuplicate: string;
  /** Placeholder `{service}`. */
  accountErrorNoSignIn: string;
}

export interface LocaleDefinition {
  /** BCP 47 tag used for `Intl`, for example `pt-BR`. */
  code: string;
  /** Name of the language in itself, for the language menu. */
  nativeName: string;
  /** Short label on the language button, for example "PL" or "עב". */
  shortLabel: string;
  dir: "ltr" | "rtl";
  messages: Messages;
}
