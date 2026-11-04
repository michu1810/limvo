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

  extraUsage: string;
  /** Placeholders `{used}` and `{limit}` are formatted amounts of money. */
  extraUsageOf: string;
  /** Placeholder `{when}`, a weekday and time before the reset. */
  forecastLimitAt: string;
  /** Accessible description of the weekly chart. */
  weekChart: string;

  shortcut: string;
  /** Placeholder `{keys}`, for example "Win+Shift+H". */
  shortcutHint: string;
  shortcutTaken: string;
  planColors: string;
  planColorsHint: string;
  services: string;
  taskbarPins: string;
  taskbarPinsHint: string;
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
