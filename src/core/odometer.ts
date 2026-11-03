/**
 * Wheel positions of a mechanical counter showing `value`.
 *
 * The units wheel turns continuously with the value. Every higher wheel stays
 * put and only advances while the wheel below it rolls from 9 to 0, the way a
 * Geneva drive in an old odometer carries one digit to the next.
 *
 * Returns one position per wheel, units first. A position is in digit steps:
 * 3 shows a 3, 3.5 is halfway between 3 and 4.
 */
export function wheelPositions(value: number, wheels: number): number[] {
  const v = Math.max(0, value);
  return Array.from({ length: wheels }, (_, place) => {
    if (place === 0) return v;
    const span = 10 ** place;
    const below = v % span;
    // The carry happens during the last unit step of the lower wheels.
    const carry = Math.max(0, below - (span - 1));
    return Math.floor(v / span) + carry;
  });
}

/** How many wheels the counter needs to show every value up to `max`. */
export function wheelCount(max: number): number {
  return Math.max(1, Math.floor(Math.max(0, max)).toString().length);
}
