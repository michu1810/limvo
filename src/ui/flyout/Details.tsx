import type { ReactNode } from "react";
import type { BreakdownRow, UsageWindow } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { useAnimatedNumber } from "../hooks";
import { windowLabel } from "../labels";
import { Meter } from "../Meter";

/** Products that used part of the window; untouched ones would only add noise. */
export function Breakdown({ rows, replay }: { rows: readonly BreakdownRow[]; replay: number }) {
  const { t } = useI18n();
  const used = rows.filter((row) => row.percent > 0);
  if (used.length === 0) return null;

  return (
    <Group title={t("byProductWeekly")}>
      {used.map((row) => (
        <LimitRow key={row.key} label={row.label} percent={row.percent} replay={replay} />
      ))}
    </Group>
  );
}

export function OtherLimits({
  windows,
  replay,
}: {
  windows: readonly UsageWindow[];
  replay: number;
}) {
  const i18n = useI18n();
  const { t, format } = i18n;
  if (windows.length === 0) return null;

  return (
    <Group title={t("otherLimits")}>
      {windows.map((window) => (
        <LimitRow
          key={window.id}
          label={windowLabel(window, i18n)}
          percent={window.percent}
          severity={window.severity}
          replay={replay}
          detail={
            window.resetsAt
              ? t("notifyThresholdBody", { when: format.weekdayTime(new Date(window.resetsAt)) })
              : undefined
          }
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
  replay,
  detail,
}: {
  label: string;
  percent: number;
  severity?: UsageWindow["severity"];
  replay: number;
  /** Extra context shown when the row is hovered, such as the reset time. */
  detail?: string | undefined;
}) {
  const { format } = useI18n();
  const counted = useAnimatedNumber(percent, replay, 750);
  return (
    <li className="limit" title={detail}>
      <span className="limit__label">{label}</span>
      <Meter size="small" value={percent} severity={severity} label={label} replay={replay} />
      <span className="limit__value">{format.percent(Math.round(counted))}</span>
    </li>
  );
}
