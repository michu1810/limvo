import { useEffect, useState } from "react";
import { addReading, type WindowHistory } from "../core/history";
import type { ProviderReport } from "../core/snapshot";

const STORAGE_KEY = "limvo.history.v2";
/** Before several services the history held Claude's readings alone. */
const LEGACY_KEY = "limvo.history.v1";

type Histories = Record<string, WindowHistory>;

/**
 * Remembers the headline readings of the running period of every service,
 * for the weekly chart. Kept in this window's storage: it lives as long as
 * the app and never leaves the machine.
 */
export function useWeekHistory(reports: readonly ProviderReport[]): Histories {
  const [histories, setHistories] = useState<Histories>(load);

  useEffect(() => {
    setHistories((previous) => {
      let next = previous;
      for (const report of reports) {
        const headline = report.snapshot?.headline;
        if (report.state !== "ok" || !headline?.resetsAt || !report.snapshot) continue;
        const at = Date.parse(report.snapshot.fetchedAt);
        if (Number.isNaN(at)) continue;
        const before = next[report.providerId];
        const after = addReading(before, headline.resetsAt, { at, percent: headline.percent });
        if (after !== before) next = { ...next, [report.providerId]: after };
      }
      if (next !== previous) save(next);
      return next;
    });
  }, [reports]);

  return histories;
}

function load(): Histories {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null");
    if (parsed && typeof parsed === "object") return parsed as Histories;
    const legacy: unknown = JSON.parse(localStorage.getItem(LEGACY_KEY) ?? "null");
    if (legacy && typeof legacy === "object" && "readings" in legacy)
      return { claude: legacy as WindowHistory };
  } catch {
    // Unreadable storage only costs the chart its past.
  }
  return {};
}

function save(histories: Histories) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(histories));
  } catch {
    // Storage can be unavailable; the chart then shows the current period only.
  }
}
