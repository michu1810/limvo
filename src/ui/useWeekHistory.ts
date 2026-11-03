import { useEffect, useState } from "react";
import { addReading, type WindowHistory } from "../core/history";
import type { ProviderReport } from "../core/snapshot";

const STORAGE_KEY = "limvo.history.v1";

/**
 * Remembers the headline readings of the running period, for the weekly
 * chart. Kept in this window's storage: it lives as long as the app and
 * never leaves the machine.
 */
export function useWeekHistory(report: ProviderReport): WindowHistory | undefined {
  const [history, setHistory] = useState<WindowHistory | undefined>(load);

  useEffect(() => {
    const headline = report.snapshot?.headline;
    if (report.state !== "ok" || !headline?.resetsAt || !report.snapshot) return;
    const at = Date.parse(report.snapshot.fetchedAt);
    if (Number.isNaN(at)) return;
    setHistory((previous) => {
      const next = addReading(previous, headline.resetsAt ?? "", { at, percent: headline.percent });
      if (next !== previous) save(next);
      return next;
    });
  }, [report]);

  return history;
}

function load(): WindowHistory | undefined {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null");
    if (parsed && typeof parsed === "object" && "readings" in parsed)
      return parsed as WindowHistory;
  } catch {
    // Unreadable storage only costs the chart its past.
  }
  return undefined;
}

function save(history: WindowHistory) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history));
  } catch {
    // Storage can be unavailable; the chart then shows the current period only.
  }
}
