/**
 * Provider-neutral usage model.
 *
 * Every provider (Claude today, others later) maps its own API into these
 * shapes in the Rust core, so the UI never knows which service it is showing.
 * The Rust side mirrors these types in `src-tauri/src/snapshot.rs`; both sides
 * are checked against `fixtures/report.sample.json`.
 */

export type WindowKind = "session" | "weekly" | "scoped";

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
  usagePageUrl: string | null;
}

export type ReportState =
  | "loading"
  | "ok"
  | "offline"
  | "auth_required"
  | "no_credentials"
  | "error";

export interface ProviderReport {
  state: ReportState;
  /** The last good reading. Kept when a later refresh fails, so the UI can show it as stale. */
  snapshot: UsageSnapshot | null;
  /** Diagnostic detail for `error`; never contains credentials. */
  detail: string | null;
}
