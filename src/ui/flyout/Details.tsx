import type { ReactNode } from "react";
import type { BreakdownRow, ExtraUsage, UsageWindow } from "../../core/snapshot";
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
  extraUsage,
  replay,
}: {
  windows: readonly UsageWindow[];
  extraUsage: ExtraUsage | null;
  replay: number;
}) {
  const i18n = useI18n();
  const { t, format } = i18n;
  if (windows.length === 0 && !extraUsage) return null;

  return (
    <Group title={t("otherLimits")}>
      {extraUsage && <ExtraUsageRow extra={extraUsage} replay={replay} />}
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

/** Paid credits: what was spent, against the monthly cap when there is one. */
function ExtraUsageRow({ extra, replay }: { extra: ExtraUsage; replay: number }) {
  const { t, locale } = useI18n();
  const money = new Intl.NumberFormat(locale.code, {
    style: "currency",
    currency: extra.currency,
    minimumFractionDigits: Math.min(extra.exponent, 2),
    maximumFractionDigits: extra.exponent,
  });
  const amount = (minor: number) => money.format(minor / 10 ** extra.exponent);
  const used = amount(extra.usedMinor);
  const label = t("extraUsage");

  return (
    <li
      className="limit limit--money"
      title={
        extra.limitMinor === null
          ? undefined
          : t("extraUsageOf", { used, limit: amount(extra.limitMinor) })
      }
    >
      <span className="limit__label">{label}</span>
      {extra.percent === null ? (
        <span />
      ) : (
        <Meter
          size="small"
          value={extra.percent}
          severity={extra.severity}
          label={label}
          replay={replay}
        />
      )}
      <span className="limit__value">{used}</span>
    </li>
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
