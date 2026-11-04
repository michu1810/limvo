import { useEffect, useState } from "react";
import { bridge } from "../../bridge";
import type { AvailableUpdate } from "../../bridge/types";
import { showUpdateBanner, type UpdateDismissal } from "../../core/updates";
import { useI18n } from "../../i18n/I18nProvider";

const DISMISSAL_KEY = "limvo.updateDismissal";

/** Offers the newer version the host found. Nothing downloads until "Update". */
export function UpdateBanner({ now }: { now: Date }) {
  const { t } = useI18n();
  const update = useAvailableUpdate();
  const [dismissal, setDismissal] = useState(readDismissal);
  const [phase, setPhase] = useState<"idle" | "installing" | "failed">("idle");

  if (!update || !showUpdateBanner(update.version, dismissal, now.getTime())) return null;

  const later = () => {
    // The flyout's clock, not Date.now(): it ticks less often, and a later
    // stamp would look like a clock set back and keep the banner up.
    const next = { version: update.version, at: now.getTime() };
    setDismissal(next);
    saveDismissal(next);
  };
  const install = async () => {
    setPhase("installing");
    try {
      await bridge.installUpdate();
    } catch {
      setPhase("failed");
    }
  };

  return (
    <div className="notice notice--update" role="status" aria-live="polite">
      <p className="notice__text">
        {phase === "failed" ? t("updateFailed") : t("updateAvailable", { version: update.version })}
      </p>
      {phase !== "installing" && (
        <button type="button" className="action action--inline" onClick={later}>
          {t("updateLater")}
        </button>
      )}
      <button
        type="button"
        className="action action--inline action--accent"
        disabled={phase === "installing"}
        onClick={install}
      >
        {phase === "installing" ? t("updateInstalling") : t("updateInstall")}
      </button>
    </div>
  );
}

function useAvailableUpdate() {
  const [update, setUpdate] = useState<AvailableUpdate | null>(null);
  useEffect(() => {
    let active = true;
    bridge.getUpdate().then((found) => active && setUpdate(found));
    const unsubscribe = bridge.onUpdate((found) => active && setUpdate(found));
    return () => {
      active = false;
      unsubscribe.then((stop) => stop());
    };
  }, []);
  return update;
}

function readDismissal(): UpdateDismissal | null {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(DISMISSAL_KEY) ?? "null");
    if (
      value &&
      typeof value === "object" &&
      typeof (value as UpdateDismissal).version === "string" &&
      typeof (value as UpdateDismissal).at === "number"
    ) {
      return value as UpdateDismissal;
    }
  } catch {
    // Unreadable storage counts as never dismissed.
  }
  return null;
}

function saveDismissal(dismissal: UpdateDismissal) {
  try {
    localStorage.setItem(DISMISSAL_KEY, JSON.stringify(dismissal));
  } catch {
    // Without storage "Later" lasts until the flyout reloads.
  }
}
