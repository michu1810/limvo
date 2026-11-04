/**
 * Line icons drawn on a 16 px grid with 1.5 px strokes, matching Windows 11
 * system glyphs. Each one animates a part of itself on hover (see `base.css`),
 * never the whole symbol.
 */

const stroke = {
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.5,
  strokeLinecap: "round",
  strokeLinejoin: "round",
} as const;

export function BackIcon() {
  return (
    <svg
      className="icon icon--back flip-rtl"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      aria-hidden="true"
    >
      <path className="icon__part" d="M10 3 5 8l5 5" {...stroke} />
    </svg>
  );
}

export function ChevronIcon() {
  return (
    <svg
      className="icon icon--chevron flip-rtl"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      aria-hidden="true"
    >
      <path className="icon__part" d="m6 3 5 5-5 5" {...stroke} />
    </svg>
  );
}

export function CheckIcon() {
  return (
    <svg className="icon icon--check" width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
      <path className="icon__part" d="m3.5 8.5 3 3 6-7" {...stroke} />
    </svg>
  );
}

export function RemoveIcon() {
  return (
    <svg
      className="icon icon--remove"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      aria-hidden="true"
    >
      <path className="icon__part" d="m4.5 4.5 7 7m0-7-7 7" {...stroke} />
    </svg>
  );
}

/** Three sliders: the knobs slide along their tracks on hover. */
export function SettingsIcon() {
  return (
    <svg
      className="icon icon--settings"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      aria-hidden="true"
    >
      <path d="M2.5 4.5h11M2.5 8h11M2.5 11.5h11" {...stroke} strokeOpacity={0.45} />
      <circle
        className="icon__part icon__knob icon__knob--1"
        cx="5.5"
        cy="4.5"
        r="1.6"
        {...stroke}
        fill="var(--icon-fill)"
      />
      <circle
        className="icon__part icon__knob icon__knob--2"
        cx="10.5"
        cy="8"
        r="1.6"
        {...stroke}
        fill="var(--icon-fill)"
      />
      <circle
        className="icon__part icon__knob icon__knob--3"
        cx="7"
        cy="11.5"
        r="1.6"
        {...stroke}
        fill="var(--icon-fill)"
      />
    </svg>
  );
}
