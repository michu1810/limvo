import type { ProviderReport, UsageSnapshot } from "../core/snapshot";

const HOUR = 3600_000;
const WEEK_SECONDS = 7 * 24 * 3600;

/**
 * Readings for development and screenshots, relative to `now` so they never
 * go stale. Select one with `?scenario=<name>` in the browser.
 */
export const SCENARIOS = {
  reference: (now) => ok(snapshot(now, { percent: 51, resetInHours: 40, updatedMinutesAgo: 3 })),
  fresh: (now) =>
    ok(snapshot(now, { percent: 2, resetInHours: 167, updatedMinutesAgo: 0, codeShare: 100 })),
  tight: (now) => ok(snapshot(now, { percent: 46, resetInHours: 84, updatedMinutesAgo: 1 })),
  hot: (now) => ok(snapshot(now, { percent: 78, resetInHours: 70, updatedMinutesAgo: 1 })),
  reached: (now) => ok(snapshot(now, { percent: 100, resetInHours: 5, updatedMinutesAgo: 2 })),
  offline: (now) => ({
    state: "offline",
    snapshot: snapshot(now, { percent: 51, resetInHours: 40, updatedMinutesAgo: 42 }),
    detail: null,
  }),
  auth: () => ({ state: "auth_required", snapshot: null, detail: null }),
  signedOut: () => ({ state: "no_credentials", snapshot: null, detail: null }),
  error: () => ({ state: "error", snapshot: null, detail: "HTTP 500" }),
  loading: () => ({ state: "loading", snapshot: null, detail: null }),
} satisfies Record<string, (now: Date) => ProviderReport>;

export type ScenarioName = keyof typeof SCENARIOS;

function ok(value: UsageSnapshot): ProviderReport {
  return { state: "ok", snapshot: value, detail: null };
}

interface Shape {
  percent: number;
  resetInHours: number;
  updatedMinutesAgo: number;
  /** Share of Claude Code in the breakdown; the rest goes to Cowork and Chats. */
  codeShare?: number;
}

function snapshot(now: Date, shape: Shape): UsageSnapshot {
  const at = (hours: number) => new Date(now.getTime() + hours * HOUR).toISOString();
  const codeShare = shape.codeShare ?? 51;
  const coworkShare = codeShare === 100 ? 0 : 100 - codeShare - 1;

  return {
    providerId: "claude",
    providerName: "Claude",
    fetchedAt: at(-shape.updatedMinutesAgo / 60),
    headline: {
      id: "weekly_all",
      kind: "weekly",
      label: null,
      percent: shape.percent,
      resetsAt: at(shape.resetInHours),
      periodSeconds: WEEK_SECONDS,
      severity: shape.percent >= 100 ? "critical" : shape.percent >= 80 ? "warning" : "normal",
    },
    windows: [
      {
        id: "scoped:fable",
        kind: "scoped",
        label: "Fable",
        percent: 0,
        resetsAt: at(shape.resetInHours),
        periodSeconds: WEEK_SECONDS,
        severity: "normal",
      },
    ],
    breakdown: [
      { key: "claude_code", label: "Claude Code", percent: codeShare },
      { key: "cowork", label: "Cowork", percent: coworkShare },
      { key: "chat", label: "Chats", percent: codeShare === 100 ? 0 : 1 },
    ],
    usagePageUrl: "https://claude.ai/settings/usage",
  };
}
