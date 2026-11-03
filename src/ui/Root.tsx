import type { ReactNode } from "react";
import { I18nProvider } from "../i18n/I18nProvider";
import { useActiveLocale } from "./hooks";

/** Locale plumbing shared by every window. */
export function Root({ children }: { children: (chosenLocale: string | null) => ReactNode }) {
  const { locale, chosen } = useActiveLocale();
  return <I18nProvider locale={locale}>{children(chosen)}</I18nProvider>;
}
