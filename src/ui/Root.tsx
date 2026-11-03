import type { ReactNode } from "react";
import type { Settings } from "../bridge";
import { I18nProvider } from "../i18n/I18nProvider";
import { useActiveLocale, useSettings } from "./hooks";

/** Settings and locale plumbing shared by every window. */
export function Root({ children }: { children: (settings: Settings | null) => ReactNode }) {
  const settings = useSettings();
  const locale = useActiveLocale(settings);
  return <I18nProvider locale={locale}>{children(settings)}</I18nProvider>;
}
