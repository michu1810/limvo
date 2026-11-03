import { useEffect, useRef } from "react";
import { bridge } from "../../bridge";
import { useI18n } from "../../i18n/I18nProvider";
import { LOCALES } from "../../i18n/locales";
import { BackIcon, CheckIcon } from "../icons";

/** Full-height list instead of a popup menu, so it always fits the small flyout. */
export function LanguagePanel({ chosen, onClose }: { chosen: string | null; onClose: () => void }) {
  const { t } = useI18n();
  const selected = useRef<HTMLInputElement>(null);

  useEffect(() => selected.current?.focus(), []);

  const pick = async (code: string | null) => {
    await bridge.updateSettings({ locale: code });
    onClose();
  };

  const options: { code: string | null; name: string; lang?: string }[] = [
    { code: null, name: t("systemLanguage") },
    ...LOCALES.map((locale) => ({ code: locale.code, name: locale.nativeName, lang: locale.code })),
  ];

  return (
    <>
      <header className="flyout__header">
        <button type="button" className="icon-button" aria-label={t("back")} onClick={onClose}>
          <BackIcon />
        </button>
        <h1 className="flyout__title">{t("language")}</h1>
      </header>
      <fieldset className="languages">
        <legend className="sr-only">{t("language")}</legend>
        {options.map((option) => {
          const checked = option.code === chosen;
          return (
            <label key={option.code ?? "system"} className="languages__option">
              <input
                ref={checked ? selected : undefined}
                type="radio"
                name="locale"
                checked={checked}
                onChange={() => pick(option.code)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") pick(option.code);
                }}
              />
              <span lang={option.lang}>{option.name}</span>
              {checked && <CheckIcon />}
            </label>
          );
        })}
      </fieldset>
    </>
  );
}
