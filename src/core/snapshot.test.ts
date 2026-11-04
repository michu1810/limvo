import { describe, expect, it } from "vitest";
import fixture from "../../fixtures/report.sample.json";
import type { BreakdownRow, ProviderReport, UsageSnapshot, UsageWindow } from "./snapshot";

// `satisfies` makes each list complete and exact for its type, so these lists
// fail to compile when a field is added or renamed in `snapshot.ts`.
const reportKeys = {
  providerId: true,
  serviceId: true,
  providerName: true,
  state: true,
  snapshot: true,
  detail: true,
} satisfies Record<keyof ProviderReport, true>;
const snapshotKeys = {
  providerId: true,
  providerName: true,
  plan: true,
  fetchedAt: true,
  headline: true,
  windows: true,
  breakdown: true,
  extraUsage: true,
  usagePageUrl: true,
} satisfies Record<keyof UsageSnapshot, true>;
const windowKeys = {
  id: true,
  kind: true,
  label: true,
  percent: true,
  resetsAt: true,
  periodSeconds: true,
  severity: true,
} satisfies Record<keyof UsageWindow, true>;
const rowKeys = { key: true, label: true, percent: true } satisfies Record<
  keyof BreakdownRow,
  true
>;

const keysOf = (value: object) => Object.keys(value).sort();

describe("report contract shared with the Rust core", () => {
  it("has exactly the fields the UI expects", () => {
    expect(keysOf(fixture)).toEqual(keysOf(reportKeys));
    expect(keysOf(fixture.snapshot)).toEqual(keysOf(snapshotKeys));
    expect(keysOf(fixture.snapshot.headline)).toEqual(keysOf(windowKeys));
    for (const window of fixture.snapshot.windows)
      expect(keysOf(window)).toEqual(keysOf(windowKeys));
    for (const row of fixture.snapshot.breakdown) expect(keysOf(row)).toEqual(keysOf(rowKeys));
  });
});
