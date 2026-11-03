import { createContext, type ReactNode, useContext, useEffect, useMemo } from "react";
import { createFormatters, type Formatters } from "./format";
import { fill, type MessageKey } from "./resolve";
import type { LocaleDefinition } from "./types";

export interface I18n {
  locale: LocaleDefinition;
  t(key: MessageKey, values?: Readonly<Record<string, string>>): string;
  format: Formatters;
}

const I18nContext = createContext<I18n | null>(null);

export function I18nProvider({
  locale,
  children,
}: {
  locale: LocaleDefinition;
  children: ReactNode;
}) {
  const value = useMemo<I18n>(
    () => ({
      locale,
      t: (key, values) => fill(locale.messages[key], values),
      format: createFormatters(locale.code),
    }),
    [locale],
  );

  useEffect(() => {
    document.documentElement.lang = locale.code;
    document.documentElement.dir = locale.dir;
  }, [locale]);

  return <I18nContext value={value}>{children}</I18nContext>;
}

export function useI18n(): I18n {
  const context = useContext(I18nContext);
  if (!context) throw new Error("useI18n must be used inside <I18nProvider>");
  return context;
}
