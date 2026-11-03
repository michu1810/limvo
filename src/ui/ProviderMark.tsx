import { type CSSProperties, useState } from "react";

/**
 * The provider's own mark if the user supplied one in `public/brand/<id>.svg`,
 * otherwise the Headroom mark. Brand files are not shipped with the project:
 * they belong to their owners, so each user adds the official asset locally.
 */
export function ProviderMark({
  providerId,
  size,
  level = null,
}: {
  providerId: string;
  size: number;
  /** Usage in percent; the Headroom mark shows it. */
  level?: number | null;
}) {
  const [missing, setMissing] = useState(false);

  if (missing) return <HeadroomMark size={size} level={level} />;
  return (
    <img
      className="mark"
      src={`/brand/${providerId}.svg`}
      width={size}
      height={size}
      alt=""
      draggable={false}
      onError={() => setMissing(true)}
    />
  );
}

/**
 * An H whose crossbar is a gauge: it rises as the plan fills up, and the
 * space left above it is the headroom. With no reading it rests low.
 */
export function HeadroomMark({ size, level = null }: { size: number; level?: number | null }) {
  const used = Math.min(100, Math.max(0, level ?? 0)) / 100;
  // The crossbar travels between the feet (y 16) and the top (y 4) of the posts.
  const style = { "--bar-y": `${16 - used * 12}px` } as CSSProperties;
  return (
    <svg
      className="mark mark--headroom"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      aria-hidden="true"
      style={style}
    >
      <rect x="4" y="3" width="4" height="18" rx="2" fill="var(--accent)" />
      <rect x="16" y="3" width="4" height="18" rx="2" fill="var(--accent)" />
      <rect className="mark__bar" x="6" width="12" height="4" rx="2" fill="var(--accent)" />
    </svg>
  );
}
