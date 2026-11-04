import { type CSSProperties, useEffect, useState } from "react";

/** `?nobrand` forces the Limvo mark, for screenshots that must not show third-party logos. */
const BRANDS_OFF = new URLSearchParams(window.location.search).has("nobrand");

/**
 * The provider's own mark if the user supplied one in `public/brand/<id>.svg`,
 * otherwise the Limvo mark. A logo drawn in dark ink can come with a light
 * variant, `<id>-dark.svg`, used on dark surfaces. Brand files are not shipped
 * with the project: they belong to their owners, so each user adds the
 * official asset locally.
 */
export function ProviderMark({
  providerId,
  size,
  level = null,
}: {
  providerId: string;
  size: number;
  /** Usage in percent; the Limvo mark shows it. */
  level?: number | null;
}) {
  const dark = useDarkSurface();
  // Which files turned out to be missing: the dark variant, then the mark itself.
  const [missing, setMissing] = useState({ dark: false, base: false });

  if (missing.base || BRANDS_OFF) return <LimvoMark size={size} level={level} />;
  const useDark = dark && !missing.dark;
  return (
    <img
      key={useDark ? "dark" : "base"}
      className="mark"
      src={`/brand/${providerId}${useDark ? "-dark" : ""}.svg`}
      width={size}
      height={size}
      alt=""
      draggable={false}
      onError={() => setMissing((previous) => ({ ...previous, [useDark ? "dark" : "base"]: true }))}
    />
  );
}

/**
 * Whether the window is drawn dark: the theme the host forced on the root
 * (the strip follows the taskbar), otherwise the system scheme.
 */
function useDarkSurface(): boolean {
  const [dark, setDark] = useState(readDark);

  useEffect(() => {
    const update = () => setDark(readDark());
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    query.addEventListener("change", update);
    const observer = new MutationObserver(update);
    observer.observe(document.documentElement, { attributeFilter: ["data-theme"] });
    return () => {
      query.removeEventListener("change", update);
      observer.disconnect();
    };
  }, []);

  return dark;
}

function readDark(): boolean {
  const forced = document.documentElement.dataset.theme;
  if (forced === "dark" || forced === "light") return forced === "dark";
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/**
 * An L whose foot is a meter: it fills from the post outwards as the plan is
 * used. With no reading it shows a resting level.
 */
export function LimvoMark({ size, level = null }: { size: number; level?: number | null }) {
  const used = Math.min(100, Math.max(0, level ?? 60)) / 100;
  // The fill starts as wide as the post (4 units) and reaches the end of the foot (16 units).
  const style = { "--fill-width": `${4 + used * 12}px` } as CSSProperties;
  return (
    <svg
      className="mark mark--limvo"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      aria-hidden="true"
      style={style}
    >
      <rect x="4" y="3" width="4" height="18" rx="2" fill="var(--accent)" />
      <rect x="4" y="17" width="16" height="4" rx="2" fill="var(--accent)" opacity="0.28" />
      <rect className="mark__fill" x="4" y="17" height="4" rx="2" fill="var(--accent)" />
    </svg>
  );
}
