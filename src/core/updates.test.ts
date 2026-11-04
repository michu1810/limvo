import { describe, expect, it } from "vitest";
import { LATER_HIDES_FOR, showUpdateBanner } from "./updates";

const NOW = Date.parse("2026-11-04T12:00:00Z");
const HOUR = 60 * 60 * 1000;

describe("showUpdateBanner", () => {
  it("stays hidden without an update", () => {
    expect(showUpdateBanner(null, null, NOW)).toBe(false);
    expect(showUpdateBanner(null, { version: "0.2.0", at: NOW }, NOW)).toBe(false);
  });

  it("shows an update that was never put off", () => {
    expect(showUpdateBanner("0.2.0", null, NOW)).toBe(true);
  });

  it("hides a version put off a moment ago", () => {
    expect(showUpdateBanner("0.2.0", { version: "0.2.0", at: NOW - HOUR }, NOW)).toBe(false);
  });

  it("brings the same version back after a few days", () => {
    const at = NOW - LATER_HIDES_FOR;
    expect(showUpdateBanner("0.2.0", { version: "0.2.0", at }, NOW)).toBe(true);
  });

  it("shows a newer version at once", () => {
    expect(showUpdateBanner("0.2.1", { version: "0.2.0", at: NOW - HOUR }, NOW)).toBe(true);
  });

  it("does not hide for good when the clock went back", () => {
    expect(showUpdateBanner("0.2.0", { version: "0.2.0", at: NOW + HOUR }, NOW)).toBe(true);
  });
});
