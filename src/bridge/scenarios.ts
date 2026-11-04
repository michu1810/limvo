import type { ProviderReport, UsageSnapshot } from "../core/snapshot";

const HOUR = 3600_000;
const WEEK_SECONDS = 7 * 24 * 3600;
const SESSION_SECONDS = 5 * 3600;

type ClaudeReport = Omit<ProviderReport, "providerId" | "providerName">;

/**
 * Claude readings for development and screenshots, relative to `now` so they
 * never go stale. Select one with `?scenario=<name>` in the browser; a Codex
 * reading is added unless `?codex=off`.
 */
export const SCENARIOS = {
  reference: (now) => ok(claude(now, { percent: 51, resetInHours: 40, updatedMinutesAgo: 3 })),
  fresh: (now) =>
    ok(claude(now, { percent: 2, resetInHours: 167, updatedMinutesAgo: 0, codeShare: 100 })),
  tight: (now) => ok(claude(now, { percent: 46, resetInHours: 84, updatedMinutesAgo: 1 })),
  hot: (now) =>
    ok(claude(now, { percent: 78, resetInHours: 70, updatedMinutesAgo: 1, extraSpentCents: 1240 })),
  reached: (now) => ok(claude(now, { percent: 100, resetInHours: 5, updatedMinutesAgo: 2 })),
  offline: (now) => ({
    state: "offline",
    snapshot: claude(now, { percent: 51, resetInHours: 40, updatedMinutesAgo: 42 }),
    detail: null,
  }),
  auth: () => ({ state: "auth_required", snapshot: null, detail: null }),
  signedOut: () => ({ state: "no_credentials", snapshot: null, detail: null }),
  error: () => ({ state: "error", snapshot: null, detail: "HTTP 500" }),
  loading: () => ({ state: "loading", snapshot: null, detail: null }),
} satisfies Record<string, (now: Date) => ClaudeReport>;

export type ScenarioName = keyof typeof SCENARIOS;

/** Every provider's report for a scenario, in the order the core lists them. */
export function scenarioReports(
  name: ScenarioName,
  now: Date,
  withCodex: boolean,
  /** Adds Cursor and Antigravity, for screenshots of every service together. */
  withMore = false,
): ProviderReport[] {
  const reports: ProviderReport[] = [
    { providerId: "claude", providerName: "Claude", ...SCENARIOS[name](now) },
  ];
  reports.push(
    withCodex
      ? {
          providerId: "codex",
          providerName: "Codex",
          state: "ok",
          snapshot: codex(now),
          detail: null,
        }
      : {
          providerId: "codex",
          providerName: "Codex",
          state: "no_credentials",
          snapshot: null,
          detail: null,
        },
  );
  for (const [providerId, providerName, snapshot] of [
    ["cursor", "Cursor", cursor(now)],
    ["antigravity", "Antigravity", antigravity(now)],
  ] as const)
    reports.push(
      withMore
        ? { providerId, providerName, state: "ok", snapshot, detail: null }
        : { providerId, providerName, state: "no_credentials", snapshot: null, detail: null },
    );
  return reports;
}

function ok(value: UsageSnapshot): ClaudeReport {
  return { state: "ok", snapshot: value, detail: null };
}

interface Shape {
  percent: number;
  resetInHours: number;
  updatedMinutesAgo: number;
  /** Share of Claude Code in the breakdown; the rest goes to Cowork and Chats. */
  codeShare?: number;
  /** Extra usage spent, in cents of a 50 USD cap; omitted means extra usage is off. */
  extraSpentCents?: number;
}

const at = (now: Date, hours: number) => new Date(now.getTime() + hours * HOUR).toISOString();

function claude(now: Date, shape: Shape): UsageSnapshot {
  const codeShare = shape.codeShare ?? 51;
  const coworkShare = codeShare === 100 ? 0 : 100 - codeShare - 1;

  return {
    providerId: "claude",
    providerName: "Claude",
    plan: { id: "max_20x", label: "Max 20×" },
    fetchedAt: at(now, -shape.updatedMinutesAgo / 60),
    headline: {
      id: "weekly_all",
      kind: "weekly",
      label: null,
      percent: shape.percent,
      resetsAt: at(now, shape.resetInHours),
      periodSeconds: WEEK_SECONDS,
      severity: shape.percent >= 100 ? "critical" : shape.percent >= 80 ? "warning" : "normal",
    },
    windows: [
      {
        id: "scoped:fable",
        kind: "scoped",
        label: "Fable",
        percent: 0,
        resetsAt: at(now, shape.resetInHours),
        periodSeconds: WEEK_SECONDS,
        severity: "normal",
      },
    ],
    breakdown: [
      { key: "claude_code", label: "Claude Code", percent: codeShare },
      { key: "cowork", label: "Cowork", percent: coworkShare },
      { key: "chat", label: "Chats", percent: codeShare === 100 ? 0 : 1 },
    ],
    extraUsage:
      shape.extraSpentCents === undefined
        ? null
        : {
            usedMinor: shape.extraSpentCents,
            limitMinor: 5000,
            currency: "USD",
            exponent: 2,
            percent: shape.extraSpentCents / 50,
            severity: shape.extraSpentCents >= 4000 ? "warning" : "normal",
          },
    usagePageUrl: "https://claude.ai/settings/usage",
  };
}

const MONTH_SECONDS = 31 * 24 * 3600;

function cursor(now: Date): UsageSnapshot {
  const month = (id: string, label: string | null, percent: number) => ({
    id,
    kind: "monthly" as const,
    label,
    percent,
    resetsAt: at(now, 22 * 24),
    periodSeconds: MONTH_SECONDS,
    severity: percent >= 80 ? ("warning" as const) : ("normal" as const),
  });
  return {
    providerId: "cursor",
    providerName: "Cursor",
    plan: { id: "cursor_pro", label: "Pro" },
    fetchedAt: at(now, -1 / 60),
    headline: month("monthly_included", null, 34),
    windows: [month("monthly:auto", "Auto", 27), month("monthly:api", "API", 48)],
    breakdown: [],
    extraUsage: null,
    usagePageUrl: "https://cursor.com/dashboard?tab=usage",
  };
}

function antigravity(now: Date): UsageSnapshot {
  const model = (name: string, percent: number) => ({
    id: `scoped:${name.toLowerCase().replace(/[^a-z0-9]/g, "_")}`,
    kind: "scoped" as const,
    label: name,
    percent,
    resetsAt: at(now, 96),
    periodSeconds: WEEK_SECONDS,
    severity: percent >= 80 ? ("warning" as const) : ("normal" as const),
  });
  return {
    providerId: "antigravity",
    providerName: "Antigravity",
    plan: { id: "antigravity_g1_plus_tier", label: "Google AI Plus" },
    fetchedAt: at(now, -3 / 60),
    headline: model("Gemini 3 Flash", 41),
    windows: [
      model("Claude Sonnet 4.6 (Thinking)", 18),
      model("Claude Opus 4.6 (Thinking)", 6),
      model("GPT-OSS 120B (Medium)", 0),
    ],
    breakdown: [],
    extraUsage: null,
    usagePageUrl: null,
  };
}

function codex(now: Date): UsageSnapshot {
  return {
    providerId: "codex",
    providerName: "Codex",
    plan: { id: "chatgpt_plus", label: "Plus" },
    fetchedAt: at(now, -2 / 60),
    headline: {
      id: "weekly_all",
      kind: "weekly",
      label: null,
      percent: 23,
      resetsAt: at(now, 120),
      periodSeconds: WEEK_SECONDS,
      severity: "normal",
    },
    windows: [
      {
        id: "session",
        kind: "session",
        label: null,
        percent: 61,
        resetsAt: at(now, 2),
        periodSeconds: SESSION_SECONDS,
        severity: "normal",
      },
    ],
    breakdown: [],
    extraUsage: null,
    usagePageUrl: "https://chatgpt.com/codex/settings/usage",
  };
}
