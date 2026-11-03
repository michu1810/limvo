import { describe, expect, it } from "vitest";
import { scenarioReports } from "../bridge/scenarios";
import { availableLimits, pinnedLimits, visibleReports } from "./pins";

const now = new Date("2026-11-08T15:00:00Z");
const both = scenarioReports("reference", now, true);
const claudeOnly = scenarioReports("reference", now, false);

describe("visibleReports", () => {
  it("hides providers without a sign-in", () => {
    expect(visibleReports(claudeOnly).map((r) => r.providerId)).toEqual(["claude"]);
    expect(visibleReports(both).map((r) => r.providerId)).toEqual(["claude", "codex"]);
  });

  it("keeps the first provider when nobody is signed in", () => {
    const nobody = scenarioReports("signedOut", now, false);
    expect(visibleReports(nobody).map((r) => r.providerId)).toEqual(["claude"]);
  });
});

describe("availableLimits", () => {
  it("lists main limits first within each provider", () => {
    expect(availableLimits(both).map((limit) => limit.key)).toEqual([
      "claude:weekly_all",
      "claude:scoped:fable",
      "codex:weekly_all",
      "codex:session",
    ]);
  });
});

describe("pinnedLimits", () => {
  it("shows each provider's main limit when nothing is chosen", () => {
    expect(pinnedLimits(both, null).map((limit) => limit.key)).toEqual([
      "claude:weekly_all",
      "codex:weekly_all",
    ]);
  });

  it("keeps the chosen order", () => {
    const pins = ["codex:session", "claude:weekly_all"];
    expect(pinnedLimits(both, pins).map((limit) => limit.key)).toEqual(pins);
  });

  it("skips pins whose provider is signed out for now", () => {
    expect(
      pinnedLimits(claudeOnly, ["codex:session", "claude:weekly_all"]).map((l) => l.key),
    ).toEqual(["claude:weekly_all"]);
  });
});
