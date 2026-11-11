import { describe, expect, it } from "vitest";
import { scenarioReports } from "../bridge/scenarios";
import type { Account } from "../bridge/types";
import { accountLabel, accountTag, roomierAccount } from "./accounts";

const work: Account = { id: "claude~2", service: "claude", folder: "D:\\work", label: "Praca" };

describe("accountLabel", () => {
  it("names extra accounts only", () => {
    expect(accountLabel([work], "claude~2")).toBe("Praca");
    expect(accountLabel([work], "claude")).toBeNull();
    expect(accountLabel(undefined, "claude~2")).toBeNull();
  });
});

describe("accountTag", () => {
  it("takes the first letter of one word and the initials of two", () => {
    expect(accountTag("praca")).toBe("P");
    expect(accountTag("  side   project  x ")).toBe("SP");
  });

  it("keeps numbers and whole characters", () => {
    expect(accountTag("2")).toBe("2");
    expect(accountTag("Łódź")).toBe("Ł");
    expect(accountTag("עבודה")).toBe("ע");
  });

  it("is empty for an empty label", () => {
    expect(accountTag("   ")).toBe("");
  });
});

describe("roomierAccount", () => {
  const now = new Date("2026-11-08T17:00:00Z");
  // Claude running hot at 78%, and a second Claude account, "Praca", at a tight 46%.
  const reports = scenarioReports("hot", now, true, false, true);
  const main = reports.find((report) => report.providerId === "claude");
  const work = reports.find((report) => report.providerId === "claude~2");
  if (!main || !work) throw new Error("the scenario lost an account");

  it("points a hot account to one of the same service that lasts", () => {
    expect(roomierAccount(main, reports, now)?.report.providerId).toBe("claude~2");
  });

  it("says nothing while the current account lasts", () => {
    expect(roomierAccount(work, reports, now)).toBeNull();
  });

  it("never suggests another service", () => {
    const withoutWork = reports.filter((report) => report.providerId !== "claude~2");
    expect(roomierAccount(main, withoutWork, now)).toBeNull();
  });

  it("skips an account that would run out too", () => {
    const hotWork = { ...main, providerId: "claude~2" };
    expect(roomierAccount(main, [main, hotWork], now)).toBeNull();
  });

  it("skips an account without a reading", () => {
    const signedOut = { ...work, state: "auth_required" as const, snapshot: null };
    expect(roomierAccount(main, [main, signedOut], now)).toBeNull();
  });
});
