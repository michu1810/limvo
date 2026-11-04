/**
 * Provider-neutral usage model.
 *
 * Every provider (Claude today, others later) maps its own API into these
 * shapes in the Rust core, so the UI never knows which service it is showing.
 * The Rust side mirrors these types in `src-tauri/src/snapshot.rs`; both sides
 * are checked against `fixtures/report.sample.json`.
 */

/** `monthly` follows a billing cycle; `scoped` is a weekly cap of one model, named by `label`. */
export type WindowKind = "session" | "weekly" | "monthly" | "scoped";

export type Severity = "normal" | "warning" | "critical";

export interface UsageWindow {
  /** Stable id, for example `weekly_all`, `session` or `scoped:fable`. */
  id: string;
  kind: WindowKind;
  /** Name supplied by the provider (a model such as "Fable"); `null` means the UI labels it by kind. */
  label: string | null;
  /** Share of the allowance used, 0 to 100. */
  percent: number;
  /** ISO 8601 instant of the next reset, if the provider reports one. */
  resetsAt: string | null;
  /** Length of the rolling window, used for pace. */
  periodSeconds: number | null;
  severity: Severity;
}

export interface BreakdownRow {
  key: string;
  label: string;
  /** Share of this window's usage, 0 to 100. Rows add up to about 100. */
  percent: number;
}

/** The plan the account is on. Plan names are product names and are not translated. */
export interface PlanBadge {
  /** Stable id to style by, for example `max_20x`. */
  id: string;
  label: string;
}

/** Paid credits beyond the plan, when the user turned them on. */
export interface ExtraUsage {
  /** Amount spent in minor units of `currency`: the value is `usedMinor / 10 ** exponent`. */
  usedMinor: number;
  /** The monthly cap in the same units, or `null` when uncapped. */
  limitMinor: number | null;
  /** ISO 4217 code. */
  currency: string;
  exponent: number;
  /** Share of the cap spent, 0 to 100, when there is a cap. */
  percent: number | null;
  severity: Severity;
}

export interface UsageSnapshot {
  providerId: string;
  /** Name of the service, shown as "<name> usage". */
  providerName: string;
  plan: PlanBadge | null;
  /** ISO 8601 instant the provider answered. */
  fetchedAt: string;
  /** The window that matters most, shown large. */
  headline: UsageWindow | null;
  /** Every other limit, in the order the provider reports them. */
  windows: UsageWindow[];
  /** How the headline window splits across products. */
  breakdown: BreakdownRow[];
  extraUsage: ExtraUsage | null;
  usagePageUrl: string | null;
}

export type ReportState =
  | "loading"
  | "ok"
  | "offline"
  | "rate_limited"
  | "auth_required"
  | "no_credentials"
  | "error";

export interface ProviderReport {
  /**
   * Which account this is about, known even before the first reading: the
   * service id for the default account, `<service>~<n>` for an extra one.
   */
  providerId: string;
  /** The service, for its mark. */
  serviceId: string;
  providerName: string;
  state: ReportState;
  /** The last good reading. Kept when a later refresh fails, so the UI can show it as stale. */
  snapshot: UsageSnapshot | null;
  /** Diagnostic detail for `error`; never contains credentials. */
  detail: string | null;
}
