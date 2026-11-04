/** "Later" on the update banner, kept between runs. */
export interface UpdateDismissal {
  /** The version the user put off, for example "0.2.0". */
  version: string;
  /** When they did, in milliseconds since the epoch. */
  at: number;
}

/**
 * Whether the flyout shows the banner offering `available`.
 *
 * `available` is the newer version the host found, or `null` when there is
 * none. `dismissal` is the last time the user chose "Later", or `null`.
 *
 * Limvo is open all day, so a banner that comes back too often nags, and one
 * that never comes back leaves people on an old version for good. "Later"
 * therefore hides one version for a few days; a newer version than the one
 * put off shows at once, since it may carry a fix the user is waiting for.
 */
export function showUpdateBanner(
  available: string | null,
  dismissal: UpdateDismissal | null,
  now: number,
): boolean {
  if (available === null) return false;
  if (dismissal === null || dismissal.version !== available) return true;
  // A clock set back would otherwise hide the banner for good.
  const elapsed = now - dismissal.at;
  return elapsed < 0 || elapsed >= LATER_HIDES_FOR;
}

/** How long "Later" hides the same version. */
export const LATER_HIDES_FOR = 3 * 24 * 60 * 60 * 1000;
