import { describe, expect, it } from "vitest";
import type { Account } from "../bridge/types";
import { accountLabel, accountTag } from "./accounts";

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
