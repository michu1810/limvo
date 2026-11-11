import { useEffect, useState } from "react";
import { bridge, type NotifyLevels, type RefreshMode, type Settings } from "../../bridge";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { Segmented, SwitchRow } from "../controls";
import { BackIcon, ChevronIcon } from "../icons";
import { AccountsPanel } from "./AccountsPanel";
import { TaskbarPins } from "./TaskbarPins";

const MINUTES: Record<Exclude<RefreshMode, "adaptive">, number> = { "2m": 2, "5m": 5, "15m": 15 };

export function SettingsPanel({
  settings,
  reports,
  onBack,
  onLanguage,
}: {
  settings: Settings | null;
  reports: readonly ProviderReport[];
  onBack: () => void;
  onLanguage: () => void;
}) {
  const { t, locale, format } = useI18n();
  const [launchAtLogin, setLaunchAtLogin] = useState<boolean | null>(null);
  const [shortcutKeys, setShortcutKeys] = useState<string | null>(null);
  // The Store delivers its own updates, so the Store build has no update switch.
  const [store, setStore] = useState(false);

  useEffect(() => {
    bridge.getLaunchAtLogin().then(setLaunchAtLogin);
    bridge.hostInfo().then((host) => setStore(host.store));
  }, []);

  // The host registers the shortcut when the setting changes; ask which one took.
  // biome-ignore lint/correctness/useExhaustiveDependencies: re-read whenever the switch moves.
  useEffect(() => {
    bridge.shortcutLabel().then(setShortcutKeys);
  }, [settings?.shortcut]);
  const shortcutOn = settings?.shortcut ?? true;

  const update = (patch: Partial<Settings>) => bridge.updateSettings(patch);
  const minutes = (count: number) =>
    new Intl.NumberFormat(locale.code, {
      style: "unit",
      unit: "minute",
      unitDisplay: "short",
    }).format(count);
  const refreshOptions = [
    { value: "adaptive" as const, label: t("refreshAdaptive") },
    ...(Object.entries(MINUTES) as [RefreshMode, number][]).map(([value, count]) => ({
      value,
      label: minutes(count),
    })),
  ];
  const refresh = settings?.refresh ?? "adaptive";
  // "80 · 95%": the numbers alone, with the locale's percent sign on the last.
  const levels = (values: readonly number[]) =>
    values
      .map((value, index) =>
        index === values.length - 1 ? format.percent(value) : format.number(value),
      )
      .join(" · ");
  const levelOptions: { value: NotifyLevels; label: string }[] = [
    { value: "early", label: levels([50, 75, 90]) },
    { value: "standard", label: levels([80, 95]) },
    { value: "reached", label: t("notifyLevelsReached") },
  ];

  return (
    <>
      <header className="flyout__header">
        <button type="button" className="icon-button" aria-label={t("back")} onClick={onBack}>
          <BackIcon />
        </button>
        <h1 className="flyout__title">{t("settings")}</h1>
        {/* The running version, in view without scrolling, to compare with an update. */}
        <span className="flyout__updated" dir="ltr">
          Limvo {__APP_VERSION__}
        </span>
      </header>

      <div className="settings">
        <button type="button" className="setting" onClick={onLanguage}>
          <span className="setting__text">
            <span className="setting__label">{t("language")}</span>
            <span className="setting__hint">
              {settings?.locale ? locale.nativeName : t("systemLanguage")}
            </span>
          </span>
          <ChevronIcon />
        </button>

        <SwitchRow
          label={t("launchAtLogin")}
          checked={launchAtLogin ?? false}
          disabled={launchAtLogin === null}
          onChange={async (enabled) => setLaunchAtLogin(await bridge.setLaunchAtLogin(enabled))}
        />
        <SwitchRow
          label={t("showStrip")}
          checked={settings?.showStrip ?? true}
          onChange={(showStrip) => update({ showStrip })}
        />
        {(settings?.showStrip ?? true) && (
          <SwitchRow
            label={t("stripAllMonitors")}
            hint={t("stripAllMonitorsHint")}
            checked={settings?.stripAllMonitors ?? true}
            onChange={(stripAllMonitors) => update({ stripAllMonitors })}
          />
        )}
        {(settings?.showStrip ?? true) && <TaskbarPins reports={reports} settings={settings} />}
        <AccountsPanel settings={settings} />
        <SwitchRow
          label={t("desktopCard")}
          checked={settings?.desktopCard ?? false}
          onChange={(desktopCard) => update({ desktopCard })}
        />
        <SwitchRow
          label={t("notifications")}
          hint={t("notificationsHint")}
          checked={settings?.notifications ?? true}
          onChange={(notifications) => update({ notifications })}
        />
        {(settings?.notifications ?? true) && (
          <>
            <div className="setting setting--stacked">
              <span className="setting__text">
                <span className="setting__label">{t("notifyLevels")}</span>
              </span>
              <Segmented
                label={t("notifyLevels")}
                value={settings?.notifyLevels ?? "standard"}
                options={levelOptions}
                onChange={(notifyLevels) => update({ notifyLevels })}
              />
            </div>
            <SwitchRow
              label={t("notifyForecast")}
              hint={t("notifyForecastHint")}
              checked={settings?.notifyForecast ?? true}
              onChange={(notifyForecast) => update({ notifyForecast })}
            />
          </>
        )}
        <SwitchRow
          label={t("shortcut")}
          hint={
            !shortcutOn
              ? undefined
              : shortcutKeys
                ? t("shortcutHint", { keys: shortcutKeys })
                : t("shortcutTaken")
          }
          checked={shortcutOn}
          onChange={(shortcut) => update({ shortcut })}
        />
        <SwitchRow
          label={t("planColors")}
          hint={t("planColorsHint")}
          checked={settings?.planColors ?? true}
          onChange={(planColors) => update({ planColors })}
        />
        {!store && (
          <SwitchRow
            label={t("checkUpdates")}
            hint={t("checkUpdatesHint")}
            checked={settings?.checkUpdates ?? true}
            onChange={(checkUpdates) => update({ checkUpdates })}
          />
        )}

        <div className="setting setting--stacked">
          <span className="setting__text">
            <span className="setting__label">{t("refreshFrequency")}</span>
            <span className="setting__hint">
              {refresh === "adaptive"
                ? t("refreshAdaptiveHint")
                : t("refreshEvery", { interval: minutes(MINUTES[refresh] ?? 5) })}
            </span>
          </span>
          <Segmented
            label={t("refreshFrequency")}
            value={refresh}
            options={refreshOptions}
            onChange={(value) => update({ refresh: value })}
          />
        </div>
      </div>
    </>
  );
}
