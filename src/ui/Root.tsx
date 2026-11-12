import { type ReactNode, useEffect } from "react";
import { bridge, type Settings } from "../bridge";
import { I18nProvider, useI18n } from "../i18n/I18nProvider";
import type { MessageKey } from "../i18n/resolve";
import { useActiveLocale, useSettings } from "./hooks";

/**
 * Settings and locale plumbing shared by every window. `title` names the
 * window in the user's language, as Task Manager shows it; without one the
 * window is plain "Limvo".
 */
export function Root({
  title,
  children,
}: {
  title?: MessageKey;
  children: (settings: Settings | null) => ReactNode;
}) {
  const settings = useSettings();
  const locale = useActiveLocale(settings);
  return (
    <I18nProvider locale={locale}>
      <WindowName title={title} />
      {children(settings)}
    </I18nProvider>
  );
}

function WindowName({ title }: { title: MessageKey | undefined }) {
  const { t } = useI18n();
  const name = title ? t(title) : "Limvo";
  useEffect(() => {
    bridge.nameWindow(name).catch(() => {
      // A window that cannot rename itself keeps its plain name.
    });
  }, [name]);
  return null;
}
