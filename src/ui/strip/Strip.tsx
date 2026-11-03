import { useLayoutEffect, useRef } from "react";
import { bridge, type Settings } from "../../bridge";
import { computePace } from "../../core/pace";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { splitTemplate } from "../../i18n/resolve";
import { useNow, usePlanTheme } from "../hooks";
import { Meter } from "../Meter";
import { ProviderMark } from "../ProviderMark";
import { RollingPercent } from "../RollingNumber";

/** The compact reading docked in the taskbar. Clicking it opens the flyout. */
export function Strip({ report, settings }: { report: ProviderReport; settings: Settings | null }) {
  usePlanTheme(report, settings);
  const { t, format, locale } = useI18n();
  const now = useNow();
  const snapshot = report.snapshot;
  const headline = snapshot?.headline ?? null;
  const pace = headline ? computePace(headline, now) : null;
  const resetsAt = headline?.resetsAt ? new Date(headline.resetsAt) : null;
  const percent = headline ? format.percent(headline.percent) : "--";

  // "{percent} until {day}": the number is emphasised, the rest stays quiet.
  const parts = resetsAt ? splitTemplate(locale.messages.stripUntil) : [{ placeholder: "percent" }];
  const button = useFitWidth();

  return (
    <button
      ref={button}
      type="button"
      className="strip"
      data-stale={report.state !== "ok" || undefined}
      aria-label={t("usageTitle", { provider: snapshot?.providerName ?? "Claude" })}
      onClick={() => bridge.toggleFlyout()}
    >
      <ProviderMark
        providerId={snapshot?.providerId ?? "claude"}
        size={18}
        level={headline?.percent ?? null}
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
              return (
                // A new reading rolls in on the counter instead of jumping.
                headline ? (
                  <RollingPercent key={key} className="strip__value" value={headline.percent} />
                ) : (
                  <span key={key} className="strip__value">
                    {percent}
                  </span>
                )
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
          value={headline?.percent ?? 0}
          marker={pace?.elapsed ?? null}
          severity={headline?.severity ?? "normal"}
          label={percent}
        />
      </span>
    </button>
  );
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
