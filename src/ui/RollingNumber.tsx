import type { CSSProperties } from "react";
import { wheelCount, wheelPositions } from "../core/odometer";
import { useI18n } from "../i18n/I18nProvider";
import { easeOutQuint, useAnimatedNumber } from "./hooks";

/** A full roll from zero takes this long; the counter coasts for most of it. */
const ROLL_MS = 1800;
/** Digits on each wheel, top to bottom. A second 0 on top lets 9 roll on to 0 seamlessly. */
const STRIP = [0, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

/**
 * A percentage drawn as a mechanical counter. Each digit is a wheel; a new
 * digit drops in from above, and a wheel only moves on when the one to its
 * right rolls past nine. The percent sign and spacing come from `Intl`, so
 * "51 %" and "%51" keep their local form.
 */
export function RollingPercent({
  value,
  replay = 0,
  className,
}: {
  value: number;
  /** Changing it rolls the counter up from zero again. */
  replay?: number;
  className?: string;
}) {
  const { locale } = useI18n();
  const target = Math.round(Math.min(100, Math.max(0, value)));
  const rolled = useAnimatedNumber(target, replay, ROLL_MS, easeOutQuint);
  const wheels = wheelCount(Math.max(target, Math.ceil(rolled)));
  const positions = wheelPositions(rolled, wheels);

  const parts = new Intl.NumberFormat(locale.code, {
    style: "percent",
    maximumFractionDigits: 0,
  }).formatToParts(target / 100);

  return (
    <span className={`rolling ${className ?? ""}`}>
      <span className="sr-only">{parts.map((part) => part.value).join("")}</span>
      <span className="rolling__face" aria-hidden="true">
        {parts.map((part, index) =>
          part.type === "integer" ? (
            // A formatted percentage has a single integer part.
            <span key="wheels" className="rolling__digits">
              {positions
                .map((position, place) => (
                  <Wheel
                    // biome-ignore lint/suspicious/noArrayIndexKey: wheels are identified by their place.
                    key={place}
                    position={position}
                    // A leading wheel fades in as it rolls from 0 to 1, so there are no leading zeros.
                    opacity={place === 0 ? 1 : Math.min(1, positions[place] ?? 0)}
                  />
                ))
                .reverse()}
            </span>
          ) : (
            // biome-ignore lint/suspicious/noArrayIndexKey: parts keep their order.
            <span key={index} className="rolling__literal">
              {part.value}
            </span>
          ),
        )}
      </span>
    </span>
  );
}

function Wheel({ position, opacity }: { position: number; opacity: number }) {
  const turn = ((position % 10) + 10) % 10;
  // Index of the shown digit counted from the top of STRIP; growing values slide the strip down.
  const style = {
    "--wheel-offset": 10 - turn,
    opacity,
  } as CSSProperties;
  return (
    <span className="rolling__wheel" style={style}>
      <span className="rolling__strip">
        {STRIP.map((digit, index) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: the strip is fixed.
          <span key={index} className="rolling__digit">
            {digit}
          </span>
        ))}
      </span>
    </span>
  );
}
