import { useRef } from "react";
import { bridge, type Settings } from "../../bridge";
import { availableLimits, pinnedLimits } from "../../core/pins";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { SwitchRow } from "../controls";
import { planAttribute } from "../hooks";
import { headlineLabel, windowLabel } from "../labels";
import { ProviderMark } from "../ProviderMark";

/** The host keeps at most this many; the taskbar has room for little more. */
export const MAX_PINS = 6;

/**
 * Which limits the taskbar shows, grouped by service. Pins keep the order in
 * which they were turned on, and the last one cannot be turned off, so the
 * taskbar never shows an empty plate.
 */
export function TaskbarPins({
  reports,
  settings,
}: {
  reports: readonly ProviderReport[];
  settings: Settings | null;
}) {
  const i18n = useI18n();
  // Pins sent but not yet saved: a second click before the first is saved
  // counts from it, instead of from the settings the panel last drew.
  const sending = useRef<string[] | null>(null);
  const limits = availableLimits(reports);
  if (limits.length < 2) return null;

  const current = pinnedLimits(reports, settings?.pinned ?? null).map((limit) => limit.key);
  const toggle = (key: string, on: boolean) => {
    const base = sending.current ?? current;
    const pinned = on ? [...base, key] : base.filter((pin) => pin !== key);
    sending.current = pinned;
    bridge.updateSettings({ pinned }).finally(() => {
      if (sending.current === pinned) sending.current = null;
    });
  };

  const providers = [...new Set(limits.map((limit) => limit.report.providerId))];
  return (
    <section className="settings__group" aria-label={i18n.t("taskbarPins")}>
      <div className="setting setting--heading">
        <span className="setting__text">
          <span className="setting__label">{i18n.t("taskbarPins")}</span>
          <span className="setting__hint">
            {i18n.t("taskbarPinsHint", { count: String(MAX_PINS) })}
          </span>
        </span>
      </div>
      {providers.map((providerId) => {
        const own = limits.filter((limit) => limit.report.providerId === providerId);
        const name = own[0]?.report.providerName ?? providerId;
        return (
          <div
            key={providerId}
            className="pins"
            data-plan={planAttribute(own[0]?.report, settings)}
          >
            <span className="pins__provider">
              <ProviderMark
                providerId={own[0]?.report.serviceId ?? providerId}
                size={14}
                level={own[0]?.window.percent ?? null}
              />
              {name}
            </span>
            {own.map((limit) => {
              const on = current.includes(limit.key);
              const label = limit.headline
                ? headlineLabel(limit.window, i18n)
                : windowLabel(limit.window, i18n);
              return (
                <SwitchRow
                  key={limit.key}
                  label={label}
                  hint={i18n.format.percent(limit.window.percent)}
                  checked={on}
                  disabled={on ? current.length === 1 : current.length >= MAX_PINS}
                  onChange={(next) => toggle(limit.key, next)}
                />
              );
            })}
          </div>
        );
      })}
    </section>
  );
}
