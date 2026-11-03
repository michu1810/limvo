import { useEffect } from "react";
import { bridge, type Settings } from "../bridge";
import {
  type Alert,
  type AlertMemory,
  evaluate,
  HEADLINE_POLICY,
  SECONDARY_POLICY,
} from "../core/alerts";
import { limitKey, visibleReports } from "../core/pins";
import type { ProviderReport, UsageWindow } from "../core/snapshot";
import { type I18n, useI18n } from "../i18n/I18nProvider";
import { headlineLabel, windowLabel } from "./labels";

const STORAGE_KEY = "limvo.alerts.v1";

/**
 * Turns fresh readings into Windows notifications. Runs in the flyout
 * window, which lives for the whole session whether or not it is visible.
 * With several services signed in, each title names its service.
 */
export function useAlerts(reports: readonly ProviderReport[], settings: Settings | null) {
  const i18n = useI18n();

  useEffect(() => {
    if (!settings) return;
    const memory = load();
    const named = visibleReports(reports).length > 1;

    for (const report of reports) {
      const snapshot = report.snapshot;
      if (report.state !== "ok" || !snapshot) continue;
      const prefix = named ? `${report.providerName}: ` : "";
      const watched = [
        ...(snapshot.headline
          ? [
              {
                window: snapshot.headline,
                policy: HEADLINE_POLICY,
                label: headlineLabel(snapshot.headline, i18n),
              },
            ]
          : []),
        ...snapshot.windows.map((window) => ({
          window,
          policy: SECONDARY_POLICY,
          label: windowLabel(window, i18n),
        })),
      ];

      for (const { window, policy, label } of watched) {
        const key = limitKey(report.providerId, window.id);
        // Before several services, memory was keyed by window alone; that was Claude.
        const previous =
          memory[key] ?? (report.providerId === "claude" ? memory[window.id] : undefined);
        const result = evaluate(window, previous, policy);
        memory[key] = result.memory;
        // Memory is always updated, so turning notifications back on never replays old news.
        if (result.alert && settings.notifications)
          bridge.notify(toast(result.alert, window, `${prefix}${label}`, i18n));
      }
    }
    save(memory);
  }, [reports, settings, i18n]);
}

function toast(alert: Alert, window: UsageWindow, label: string, { t, format }: I18n) {
  const when = window.resetsAt ? format.weekdayTime(new Date(window.resetsAt)) : "";
  switch (alert.kind) {
    case "threshold":
      return {
        title: t("notifyThresholdTitle", { window: label, percent: format.percent(alert.level) }),
        body: when ? t("notifyThresholdBody", { when }) : "",
      };
    case "reached":
      return {
        title: t("notifyReachedTitle", { window: label }),
        body: when ? t("notifyThresholdBody", { when }) : "",
      };
    case "reset":
      return { title: t("notifyResetTitle", { window: label }), body: t("notifyResetBody") };
  }
}

function load(): Record<string, AlertMemory> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    return parsed && typeof parsed === "object" ? (parsed as Record<string, AlertMemory>) : {};
  } catch {
    return {};
  }
}

function save(memory: Record<string, AlertMemory>) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(memory));
  } catch {
    // Storage can be unavailable; the worst case is one repeated notification.
  }
}
