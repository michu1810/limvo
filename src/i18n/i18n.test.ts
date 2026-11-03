import { describe, expect, it } from "vitest";
import { createFormatters } from "./format";
import { FALLBACK_LOCALE, LOCALES } from "./locales";
import { fill, resolveLocale } from "./resolve";

const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe("locales", () => {
  it("have unique codes", () => {
    const codes = LOCALES.map((locale) => locale.code);
    expect(new Set(codes).size).toBe(codes.length);
  });

  for (const locale of LOCALES) {
    it(`${locale.code} uses the same placeholders as English`, () => {
      for (const [key, reference] of Object.entries(FALLBACK_LOCALE.messages)) {
        const translated = locale.messages[key as keyof typeof locale.messages];
        expect(translated.trim(), `${locale.code}.${key} is empty`).not.toBe("");
        expect(placeholders(translated), `${locale.code}.${key}`).toEqual(placeholders(reference));
      }
    });
  }
});

describe("resolveLocale", () => {
  it("prefers an exact match, then the same language", () => {
    expect(resolveLocale(["pt-BR"]).code).toBe("pt-BR");
    expect(resolveLocale(["pt-PT"]).code).toBe("pt-BR");
    expect(resolveLocale(["de-AT", "en"]).code).toBe("de");
    expect(resolveLocale(["xx", "pl-PL"]).code).toBe("pl");
  });

  it("falls back to English", () => {
    expect(resolveLocale(["xx"]).code).toBe("en");
    expect(resolveLocale([]).code).toBe("en");
  });
});

describe("fill", () => {
  it("replaces known placeholders and keeps unknown ones visible", () => {
    expect(fill("{a} and {b}", { a: "1" })).toBe("1 and {b}");
  });
});

describe("createFormatters", () => {
  const en = createFormatters("en");

  it("formats the reference reset span like the original widget", () => {
    expect(en.duration((24 + 16) * 3600_000)).toBe("1 day 16 hr");
    expect(en.duration(3 * 3600_000 + 5 * 60_000)).toBe("3 hr 5 min");
    expect(en.duration(20_000)).toBe("1 min");
  });

  it("formats percentages per locale", () => {
    expect(en.percent(51)).toBe("51%");
    expect(createFormatters("de").percent(51)).toMatch(/^51\s%$/u);
  });

  it("formats relative times in the past", () => {
    const now = new Date("2026-11-08T12:00:00Z");
    expect(en.relativeTime(new Date("2026-11-08T11:57:00Z"), now)).toBe("3 min. ago");
  });
});
