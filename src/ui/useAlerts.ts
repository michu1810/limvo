import { useEffect } from "react";
import { bridge, type Settings } from "../bridge";
import {
  type Alert,
  type AlertMemory,
  evaluate,
  HEADLINE_POLICY,
  SECONDARY_POLICY,
} from "../core/alerts";
import type { ProviderReport, UsageWindow } from "../core/snapshot";
import { type I18n, useI18n } from "../i18n/I18nProvider";
import { headlineLabel, windowLabel } from "./labels";

const STORAGE_KEY = "limvo.alerts.v1";

/**
 * Turns fresh readings into Windows notifications. Runs in the flyout
 * window, which lives for the whole session whether or not it is visible.
 */
export function useAlerts(report: ProviderReport, settings: Settings | null) {
  const i18n = useI18n();

  useEffect(() => {
    const snapshot = report.snapshot;
    if (report.state !== "ok" || !snapshot || !settings) return;

    const memory = load();
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
      const result = evaluate(window, memory[window.id], policy);
      memory[window.id] = result.memory;
      // Memory is always updated, so turning notifications back on never replays old news.
      if (result.alert && settings.notifications)
        bridge.notify(toast(result.alert, window, label, i18n));
    }
    save(memory);
  }, [report, settings, i18n]);
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
