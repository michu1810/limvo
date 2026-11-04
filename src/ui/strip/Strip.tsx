import { useLayoutEffect, useRef } from "react";
import { bridge, type Settings } from "../../bridge";
import { computePace } from "../../core/pace";
import { type Limit, pinnedLimits, visibleReports } from "../../core/pins";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { splitTemplate } from "../../i18n/resolve";
import { planAttribute, useNow } from "../hooks";
import { headlineLabel, windowLabel } from "../labels";
import { Meter } from "../Meter";
import { ProviderMark } from "../ProviderMark";
import { RollingPercent } from "../RollingNumber";

/**
 * The compact reading docked in the taskbar. One pinned limit reads as a
 * sentence ("23% until Sat"); several sit side by side as mark, number and
 * meter, each in its own service's colour. Clicking opens the flyout.
 */
export function Strip({
  reports,
  settings,
}: {
  reports: ProviderReport[];
  settings: Settings | null;
}) {
  const i18n = useI18n();
  const now = useNow();
  const button = useFitWidth();
  const limits = pinnedLimits(reports, settings?.pinned ?? null);
  const fallback = visibleReports(reports)[0] ?? reports[0];
  const single = limits.length <= 1;

  const description = limits
    .map(({ report, window, headline }) => {
      const label = headline ? headlineLabel(window, i18n) : windowLabel(window, i18n);
      return `${report.providerName}, ${label}: ${i18n.format.percent(window.percent)}`;
    })
    .join("; ");

  return (
    <button
      ref={button}
      type="button"
      className="strip"
      data-layout={single ? "single" : "multi"}
      aria-label={description || i18n.t("usageTitle", { provider: fallback?.providerName ?? "" })}
      title={limits.length > 1 ? description : undefined}
      onClick={() => bridge.toggleFlyout()}
    >
      {single ? (
        <SingleReading
          limit={limits[0]}
          report={limits[0]?.report ?? fallback}
          settings={settings}
          now={now}
        />
      ) : (
        limits.map((limit) => (
          <CompactReading key={limit.key} limit={limit} settings={settings} now={now} />
        ))
      )}
    </button>
  );
}

/** One limit, as the original Windows taskbar widget reads it. */
function SingleReading({
  limit,
  report,
  settings,
  now,
}: {
  limit: Limit | undefined;
  report: ProviderReport | undefined;
  settings: Settings | null;
  now: Date;
}) {
  const { format, locale } = useI18n();
  const window = limit?.window ?? null;
  const pace = window ? computePace(window, now) : null;
  const resetsAt = window?.resetsAt ? new Date(window.resetsAt) : null;
  const percent = window ? format.percent(window.percent) : "--";
  // "{percent} until {day}": the number is emphasised, the rest stays quiet.
  const parts = resetsAt ? splitTemplate(locale.messages.stripUntil) : [{ placeholder: "percent" }];

  return (
    <span
      className="strip__item"
      data-plan={planAttribute(report, settings)}
      data-stale={report?.state !== "ok" || undefined}
    >
      <ProviderMark
        providerId={report?.providerId ?? "claude"}
        size={18}
        level={window?.percent ?? null}
      />
      <span className="strip__body">
        <span className="strip__text">
          {parts.map((part, index) => {
            const key = `${index}`;
            if ("text" in part)
              return (
                <span key={key} className="strip__muted">
                  {part.text}
                </span>
              );
            if (part.placeholder === "percent")
              return window ? (
                // A new reading rolls in on the counter instead of jumping.
                <RollingPercent key={key} className="strip__value" value={window.percent} />
              ) : (
                <span key={key} className="strip__value">
                  {percent}
                </span>
              );
            return (
              <span key={key} className="strip__muted">
                {resetsAt ? format.weekday(resetsAt) : ""}
              </span>
            );
          })}
        </span>
        <Meter
          size="strip"
          value={window?.percent ?? 0}
          marker={pace?.elapsed ?? null}
          severity={window?.severity ?? "normal"}
          label={percent}
        />
      </span>
    </span>
  );
}

/** One of several limits: the mark tells the service, the number and meter the reading. */
function CompactReading({
  limit,
  settings,
  now,
}: {
  limit: Limit;
  settings: Settings | null;
  now: Date;
}) {
  const { format } = useI18n();
  const { report, window } = limit;
  const pace = computePace(window, now);
  return (
    <span
      className="strip__item strip__item--compact"
      data-plan={planAttribute(report, settings)}
      data-stale={report.state !== "ok" || undefined}
    >
      <ProviderMark providerId={report.providerId} size={16} level={window.percent} />
      <span className="strip__body">
        <span className="strip__text">
          <RollingPercent className="strip__value" value={window.percent} />
          {!limit.headline && <span className="strip__muted strip__tag">{shortTag(window)}</span>}
        </span>
        <Meter
          size="strip"
          value={window.percent}
          marker={pace?.elapsed ?? null}
          severity={window.severity}
          label={format.percent(window.percent)}
        />
      </span>
    </span>
  );
}

/** A second limit of the same service gets a hint of which one it is. */
function shortTag(window: Limit["window"]): string {
  if (window.kind === "session") return "5h";
  if (window.kind === "monthly" && !window.label) return "";
  return (window.label ?? "").slice(0, 6);
}

/** Reports the natural width of the strip so the host sizes its window to fit the text. */
function useFitWidth() {
  const element = useRef<HTMLButtonElement>(null);
  useLayoutEffect(() => {
    const button = element.current;
    if (!button) return;
    const observer = new ResizeObserver(() => {
      bridge.fitStrip(Math.ceil(button.scrollWidth));
    });
    observer.observe(button);
    return () => observer.disconnect();
  }, []);
  return element;
}
