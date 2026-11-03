import type { LocaleDefinition } from "../types";
import { de } from "./de";
import { en } from "./en";
import { es } from "./es";
import { fr } from "./fr";
import { he } from "./he";
import { it } from "./it";
import { ja } from "./ja";
import { pl } from "./pl";
import { ptBR } from "./pt-BR";
import { uk } from "./uk";
import { zhCN } from "./zh-CN";

/** English first: it is the fallback and the reference for every other locale. */
export const LOCALES: readonly LocaleDefinition[] = [
  en,
  pl,
  de,
  es,
  fr,
  it,
  ptBR,
  uk,
  he,
  ja,
  zhCN,
];

export const FALLBACK_LOCALE: LocaleDefinition = en;
