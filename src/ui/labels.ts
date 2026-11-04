import type { PaceStatus } from "../core/pace";
import type { UsageWindow } from "../core/snapshot";
import type { I18n } from "../i18n/I18nProvider";
import type { MessageKey } from "../i18n/resolve";

export function headlineLabel(window: UsageWindow, { t }: I18n): string {
  if (window.kind === "session") return t("headlineSession");
  if (window.kind === "monthly") return t("headlineMonthly");
  if (window.kind === "scoped" && window.label) return t("windowScoped", { name: window.label });
  return t("headlineWeekly");
}

export function windowLabel(window: UsageWindow, { t }: I18n): string {
  switch (window.kind) {
    case "session":
      return t("windowSession");
    case "weekly":
      return window.label ?? t("windowWeekly");
    case "monthly":
      return window.label ? t("windowMonthlyNamed", { name: window.label }) : t("windowMonthly");
    case "scoped":
      return t("windowScoped", { name: window.label ?? "" });
  }
}

export const PACE_MESSAGE: Record<PaceStatus, MessageKey> = {
  on_track: "paceOnTrack",
  tight: "paceTight",
  running_hot: "paceHot",
  limit_reached: "paceReached",
};
