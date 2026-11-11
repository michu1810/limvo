const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

export interface Formatters {
  /** 51 becomes "51%" in English and "51 %" in German. */
  percent(value: number): string;
  /** A percentage to one decimal, for rates: 13.94 becomes "13.9%", 14.02 "14%". */
  percentTenths(value: number): string;
  /** "3 min ago", relative to `now`. */
  relativeTime(instant: Date, now: Date): string;
  /** A positive span as its two largest units, for example "1 day 16 hr". */
  duration(milliseconds: number): string;
  /** Short weekday and clock time, for example "Tue 09:00". */
  weekdayTime(instant: Date): string;
  /** Short weekday only, for example "Tue". */
  weekday(instant: Date): string;
}

export function createFormatters(locale: string): Formatters {
  const percent = new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: 0 });
  const percentTenths = new Intl.NumberFormat(locale, {
    style: "percent",
    maximumFractionDigits: 1,
  });
  const relative = new Intl.RelativeTimeFormat(locale, { numeric: "auto", style: "short" });
  const weekdayTime = new Intl.DateTimeFormat(locale, {
    weekday: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
  const weekday = new Intl.DateTimeFormat(locale, { weekday: "short" });
  const unit = (name: "day" | "hour" | "minute") =>
    new Intl.NumberFormat(locale, {
      style: "unit",
      unit: name,
      unitDisplay: name === "day" ? "long" : "short",
    });
  const days = unit("day");
  const hours = unit("hour");
  const minutes = unit("minute");

  return {
    percent: (value) => percent.format(value / 100),
    percentTenths: (value) => percentTenths.format(value / 100),

    relativeTime(instant, now) {
      const elapsed = now.getTime() - instant.getTime();
      if (elapsed < HOUR)
        return relative.format(-Math.max(1, Math.round(elapsed / MINUTE)), "minute");
      if (elapsed < DAY) return relative.format(-Math.round(elapsed / HOUR), "hour");
      return relative.format(-Math.round(elapsed / DAY), "day");
    },

    duration(milliseconds) {
      const total = Math.max(0, milliseconds);
      const d = Math.floor(total / DAY);
      const h = Math.floor((total % DAY) / HOUR);
      const m = Math.floor((total % HOUR) / MINUTE);
      const parts =
        d > 0
          ? [days.format(d), h > 0 ? hours.format(h) : null]
          : h > 0
            ? [hours.format(h), m > 0 ? minutes.format(m) : null]
            : [minutes.format(Math.max(1, m))];
      return parts.filter((part): part is string => part !== null).join(" ");
    },

    weekdayTime: (instant) => weekdayTime.format(instant),
    weekday: (instant) => weekday.format(instant),
  };
}
