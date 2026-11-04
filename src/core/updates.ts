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
 * that never comes back leaves people on an old version for good.
 */
export function showUpdateBanner(
  available: string | null,
  dismissal: UpdateDismissal | null,
  now: number,
): boolean {
  // TODO(michu1810): decide when "Later" stops hiding the banner.
  // Placeholder until then: show it whenever an update exists.
  void dismissal;
  void now;
  return available !== null;
}
