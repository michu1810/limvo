import { FALLBACK_LOCALE, LOCALES } from "./locales";
import type { LocaleDefinition, Messages } from "./types";

/**
 * Picks the best supported locale for a list of BCP 47 preferences: an exact
 * match first (`pt-BR`), then the same language (`pt-PT` falls back to `pt-BR`).
 */
export function resolveLocale(preferences: readonly string[]): LocaleDefinition {
  for (const preference of preferences) {
    const wanted = preference.toLowerCase();
    const exact = LOCALES.find((locale) => locale.code.toLowerCase() === wanted);
    if (exact) return exact;
    const language = wanted.split("-")[0];
    const sameLanguage = LOCALES.find(
      (locale) => locale.code.toLowerCase().split("-")[0] === language,
    );
    if (sameLanguage) return sameLanguage;
  }
  return FALLBACK_LOCALE;
}

export function findLocale(code: string): LocaleDefinition | undefined {
  return LOCALES.find((locale) => locale.code === code);
}

export type MessageKey = keyof Messages;

export type TemplatePart = { text: string } | { placeholder: string };

/** Splits a template into text and placeholders, for when placeholders get their own styling. */
export function splitTemplate(template: string): TemplatePart[] {
  return template
    .split(/(\{\w+\})/g)
    .filter((piece) => piece !== "")
    .map((piece) =>
      /^\{\w+\}$/.test(piece) ? { placeholder: piece.slice(1, -1) } : { text: piece },
    );
}

/** Replaces `{name}` placeholders. Unknown placeholders stay visible, which makes gaps easy to spot. */
export function fill(template: string, values: Readonly<Record<string, string>> = {}): string {
  return template.replace(/\{(\w+)\}/g, (match, name: string) => values[name] ?? match);
}
