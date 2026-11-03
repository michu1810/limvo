import { useEffect, useRef, useState } from "react";
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

/** The saved settings, or `null` until the host has answered. */
export function useSettings(): Settings | null {
  const [settings, setSettings] = useState<Settings | null>(null);

  useEffect(() => {
    let active = true;
    const unsubscribe = bridge.onSettings((next) => {
      if (active) setSettings(next);
    });
    bridge.getSettings().then((initial) => {
      if (active) setSettings(initial);
    });
    return () => {
      active = false;
      unsubscribe.then((stop) => stop());
    };
  }, []);

  return settings;
}

/** The locale the user picked, or the best match for the system languages. */
export function useActiveLocale(settings: Settings | null): LocaleDefinition {
  const [system, setSystem] = useState<readonly string[]>(navigator.languages);

  useEffect(() => {
    let active = true;
    bridge.systemLocales().then((languages) => {
      if (active && languages.length > 0) setSystem(languages);
    });
    return () => {
      active = false;
    };
  }, []);

  const chosen = settings?.locale ?? null;
  return (chosen ? findLocale(chosen) : undefined) ?? resolveLocale(system);
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

/**
 * Counts each time the flyout comes on screen. Components key their entrance
 * on it, so the motion plays when the user looks, not when the hidden window
 * happened to mount.
 */
export function useShowCount(): number {
  const [count, setCount] = useState(0);
  useEffect(() => {
    const unsubscribe = bridge.onFlyoutShown(() => setCount((value) => value + 1));
    return () => {
      unsubscribe.then((stop) => stop());
    };
  }, []);
  return count;
}

export function usePrefersReducedMotion(): boolean {
  const [reduced, setReduced] = useState(
    () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );
  useEffect(() => {
    const query = window.matchMedia("(prefers-reduced-motion: reduce)");
    const onChange = () => setReduced(query.matches);
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);
  return reduced;
}

const easeOutCubic = (t: number) => 1 - (1 - t) ** 3;

/**
 * A number that glides to `target` instead of jumping. `replay` restarts the
 * glide from zero, which the flyout uses each time it opens. Runs only while
 * the value changes, so an idle window costs nothing.
 */
export function useAnimatedNumber(target: number, replay: number, duration = 650): number {
  const reduced = usePrefersReducedMotion();
  const [value, setValue] = useState(target);
  const shown = useRef(target);
  const lastReplay = useRef(replay);

  useEffect(() => {
    if (reduced) {
      shown.current = target;
      setValue(target);
      return;
    }
    const from = lastReplay.current !== replay ? 0 : shown.current;
    lastReplay.current = replay;
    if (from === target) return;

    const start = performance.now();
    let frame = 0;
    const step = (time: number) => {
      const progress = Math.min(1, (time - start) / duration);
      const next = from + (target - from) * easeOutCubic(progress);
      shown.current = next;
      setValue(next);
      if (progress < 1) frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => cancelAnimationFrame(frame);
  }, [target, replay, duration, reduced]);

  return value;
}
