import { useEffect, useState } from "react";
import { bridge, type Settings } from "../bridge";
import type { ProviderReport } from "../core/snapshot";
import { findLocale, resolveLocale } from "../i18n/resolve";
import type { LocaleDefinition } from "../i18n/types";

const LOADING: ProviderReport = { state: "loading", snapshot: null, detail: null };

/** The latest report from the host, kept current through its events. */
export function useReport(): ProviderReport {
  const [report, setReport] = useState<ProviderReport>(LOADING);

  useEffect(() => {
    let active = true;
    const unsubscribe = bridge.onReport((next) => {
      if (active) setReport(next);
    });
    bridge.getReport().then((initial) => {
      if (active) setReport(initial);
    });
    return () => {
      active = false;
      unsubscribe.then((stop) => stop());
    };
  }, []);

  return report;
}

/** The locale the user picked, or the best match for the system languages. */
export function useActiveLocale(): { locale: LocaleDefinition; chosen: string | null } {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [system, setSystem] = useState<readonly string[]>(navigator.languages);

  useEffect(() => {
    let active = true;
    const unsubscribe = bridge.onSettings((next) => {
      if (active) setSettings(next);
    });
    Promise.all([bridge.getSettings(), bridge.systemLocales()]).then(([initial, languages]) => {
      if (!active) return;
      setSettings(initial);
      setSystem(languages);
    });
    return () => {
      active = false;
      unsubscribe.then((stop) => stop());
    };
  }, []);

  const chosen = settings?.locale ?? null;
  const locale = (chosen ? findLocale(chosen) : undefined) ?? resolveLocale(system);
  return { locale, chosen };
}

/** The current time, refreshed every `intervalMs` so relative labels stay true. */
export function useNow(intervalMs = 30_000): Date {
  const fixed = new URLSearchParams(window.location.search).get("now");
  const [now, setNow] = useState(() => (fixed ? new Date(fixed) : new Date()));

  useEffect(() => {
    if (fixed) return;
    const timer = window.setInterval(() => setNow(new Date()), intervalMs);
    return () => window.clearInterval(timer);
  }, [fixed, intervalMs]);

  return now;
}
