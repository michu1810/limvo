import type { ProviderReport, UsageWindow } from "./snapshot";

/** A limit somewhere in the reports: one window of one provider. */
export interface Limit {
  /** `<provider>:<window>`, the form pins are stored in. */
  key: string;
  report: ProviderReport;
  window: UsageWindow;
  /** The provider's main limit, the one the flyout shows large. */
  headline: boolean;
}

export const limitKey = (providerId: string, windowId: string) => `${providerId}:${windowId}`;

/**
 * Providers worth showing: the ones with a sign-in on this machine. When none
 * has one, the first provider stays, so the flyout can explain how to sign in.
 */
export function visibleReports(reports: readonly ProviderReport[]): ProviderReport[] {
  const signedIn = reports.filter(
    (report) => report.state !== "no_credentials" || report.snapshot !== null,
  );
  if (signedIn.length > 0) return signedIn;
  return reports.slice(0, 1);
}

/** Every limit the visible providers report, main limits first within each provider. */
export function availableLimits(reports: readonly ProviderReport[]): Limit[] {
  return visibleReports(reports).flatMap((report) => {
    const snapshot = report.snapshot;
    if (!snapshot) return [];
    const windows = snapshot.headline ? [snapshot.headline, ...snapshot.windows] : snapshot.windows;
    return windows.map((window) => ({
      key: limitKey(report.providerId, window.id),
      report,
      window,
      headline: window === snapshot.headline,
    }));
  });
}

/**
 * The limits for the taskbar. With nothing chosen, each provider shows its
 * main limit. Pins whose provider or window is gone are skipped, so a pin
 * survives a provider being signed out for a while without breaking.
 */
export function pinnedLimits(
  reports: readonly ProviderReport[],
  pinned: readonly string[] | null,
): Limit[] {
  const limits = availableLimits(reports);
  const defaults = limits.filter((limit) => limit.headline);
  if (pinned === null) return defaults;
  const chosen = pinned.flatMap((key) => limits.filter((limit) => limit.key === key));
  // Every pin is away (say, only Codex pinned and Codex signed out): show the defaults, never an empty plate.
  return chosen.length > 0 ? chosen : defaults;
}
