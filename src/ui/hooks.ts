import { useEffect, useRef, useState } from "react";
import { bridge, type Settings } from "../bridge";
import type { ProviderReport } from "../core/snapshot";
import { findLocale, resolveLocale } from "../i18n/resolve";
import type { LocaleDefinition } from "../i18n/types";

const LOADING: ProviderReport[] = [
  {
    providerId: "claude",
    serviceId: "claude",
    providerName: "Claude",
    state: "loading",
    snapshot: null,
    detail: null,
  },
];

/** The latest report of every provider, kept current through the host's events. */
export function useReports(): ProviderReport[] {
  const [reports, setReports] = useState<ProviderReport[]>(LOADING);

  useEffect(() => {
    let active = true;
    const unsubscribe = bridge.onReports((next) => {
      if (active) setReports(next);
    });
    bridge.getReports().then((initial) => {
      if (active) setReports(initial);
    });
    return () => {
      active = false;
      unsubscribe.then((stop) => stop());
    };
  }, []);

  return reports;
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

/**
 * The `data-plan` value that tints an element with the colour of the
 * account's plan; `tokens.css` maps each plan to an accent. Set per element,
 * so a taskbar showing two services gives each its own colour. Status
 * colours (on track, tight, reached) never change with the plan.
 */
export function planAttribute(
  report: ProviderReport | undefined,
  settings: Settings | null,
): string | undefined {
  if (!(settings?.planColors ?? true)) return undefined;
  return report?.snapshot?.plan?.id ?? undefined;
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

export const easeOutCubic = (t: number) => 1 - (1 - t) ** 3;

/** Spins up briskly and coasts into place, like a mechanical counter winding down. */
export const easeOutQuint = (t: number) => 1 - (1 - t) ** 5;

/**
 * A number that glides to `target` instead of jumping. `replay` restarts the
 * glide from zero, which the flyout uses each time it opens. Runs only while
 * the value changes, so an idle window costs nothing.
 */
export function useAnimatedNumber(
  target: number,
  replay: number,
  duration = 650,
  ease: (t: number) => number = easeOutCubic,
): number {
  const reduced = usePrefersReducedMotion();
  // Mounted as part of a replay (the flyout remounts its view when it opens): start from zero.
  const initial = replay > 0 && !reduced ? 0 : target;
  const [value, setValue] = useState(initial);
  const shown = useRef(initial);
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
      const next = from + (target - from) * ease(progress);
      shown.current = next;
      setValue(next);
      if (progress < 1) frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => cancelAnimationFrame(frame);
  }, [target, replay, duration, ease, reduced]);

  return value;
}
