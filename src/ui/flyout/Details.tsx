import type { ReactNode } from "react";
import type { BreakdownRow, UsageWindow } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { windowLabel } from "../labels";
import { Meter } from "../Meter";

/** Products that used part of the window; untouched ones would only add noise. */
export function Breakdown({ rows }: { rows: readonly BreakdownRow[] }) {
  const { t } = useI18n();
  const used = rows.filter((row) => row.percent > 0);
  if (used.length === 0) return null;

  return (
    <Group title={t("byProductWeekly")}>
      {used.map((row) => (
        <LimitRow key={row.key} label={row.label} percent={row.percent} />
      ))}
    </Group>
  );
}

export function OtherLimits({ windows }: { windows: readonly UsageWindow[] }) {
  const i18n = useI18n();
  if (windows.length === 0) return null;

  return (
    <Group title={i18n.t("otherLimits")}>
      {windows.map((window) => (
        <LimitRow
          key={window.id}
          label={windowLabel(window, i18n)}
          percent={window.percent}
          severity={window.severity}
        />
      ))}
    </Group>
  );
}

function Group({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="group" aria-label={title}>
      <h2 className="group__title">{title}</h2>
      <ul className="group__rows">{children}</ul>
    </section>
  );
}

function LimitRow({
  label,
  percent,
  severity = "normal",
}: {
  label: string;
  percent: number;
  severity?: UsageWindow["severity"];
}) {
  const { format } = useI18n();
  return (
    <li className="limit">
      <span className="limit__label">{label}</span>
      <Meter size="small" value={percent} severity={severity} label={label} />
      <span className="limit__value">{format.percent(percent)}</span>
    </li>
  );
}
