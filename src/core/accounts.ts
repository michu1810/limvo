import type { Account } from "../bridge/types";
import { computePace } from "./pace";
import type { ProviderReport, UsageWindow } from "./snapshot";

/** The label of an extra account, or `null` for a service's default account. */
export function accountLabel(
  accounts: readonly Account[] | undefined,
  providerId: string,
): string | null {
  return accounts?.find((account) => account.id === providerId)?.label ?? null;
}

/**
 * One or two characters that tell two accounts of a service apart where only
 * the service's mark fits: the compact tabs and the taskbar. "Praca" gives
 * "P", "Side project" gives "SP", "2" stays "2".
 */
export function accountTag(label: string): string {
  const words = label.trim().split(/\s+/).filter(Boolean);
  const initial = (word: string) => (Array.from(word)[0] ?? "").toLocaleUpperCase();
  if (words.length === 0) return "";
  if (words.length === 1) return initial(words[0] ?? "");
  return words.slice(0, 2).map(initial).join("");
}

/**
 * Another account of the same service that would last where this one does
 * not: the current account is running hot or spent, and the other one, at
 * its own pace, makes it to its reset. Of several, the one projected to use
 * the least. `null` when there is nothing better to suggest.
 */
export function roomierAccount(
  current: ProviderReport,
  reports: readonly ProviderReport[],
  now: Date,
): { report: ProviderReport; window: UsageWindow } | null {
  const headline = current.state === "ok" ? current.snapshot?.headline : null;
  if (!headline) return null;
  const status = computePace(headline, now)?.status;
  if (status !== "running_hot" && status !== "limit_reached") return null;

  let best: { report: ProviderReport; window: UsageWindow; projected: number } | null = null;
  for (const report of reports) {
    if (report.providerId === current.providerId || report.serviceId !== current.serviceId)
      continue;
    const window = report.state === "ok" ? report.snapshot?.headline : null;
    if (!window || window.percent >= 100) continue;
    const pace = computePace(window, now);
    if (!pace || pace.projected >= 100) continue;
    if (!best || pace.projected < best.projected)
      best = { report, window, projected: pace.projected };
  }
  return best && { report: best.report, window: best.window };
}
