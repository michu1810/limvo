import { useState } from "react";

/**
 * The provider's own mark if the user supplied one in `public/brand/<id>.svg`,
 * otherwise the Headroom mark. Brand files are not shipped with the project:
 * they belong to their owners, so each user adds the official asset locally.
 */
export function ProviderMark({ providerId, size }: { providerId: string; size: number }) {
  const [missing, setMissing] = useState(false);

  if (missing) return <HeadroomMark size={size} />;
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

/** An H whose crossbar sits low: the space above it is the headroom you have left. */
export function HeadroomMark({ size }: { size: number }) {
  return (
    <svg className="mark" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
      <rect x="4" y="3" width="4" height="18" rx="2" fill="var(--accent)" />
      <rect x="16" y="3" width="4" height="18" rx="2" fill="var(--accent)" />
      <rect x="6" y="14" width="12" height="4" rx="2" fill="var(--accent)" />
      <rect x="10" y="7" width="4" height="2.5" rx="1.25" fill="var(--accent)" opacity="0.35" />
    </svg>
  );
}
