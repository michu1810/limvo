import type { Account } from "../bridge/types";

/** The label of an extra account, or `null` for a service's default account. */
export function accountLabel(
  accounts: readonly Account[] | undefined,
  providerId: string,
): string | null {
  return accounts?.find((account) => account.id === providerId)?.label ?? null;
}

/**
 * One or two characters that tell two accounts of a service apart where only
 * the service's mark fits: the compact tabs and the taskbar. "Praca" gives
 * "P", "Side project" gives "SP", "2" stays "2".
 */
export function accountTag(label: string): string {
  const words = label.trim().split(/\s+/).filter(Boolean);
  const initial = (word: string) => (Array.from(word)[0] ?? "").toLocaleUpperCase();
  if (words.length === 0) return "";
  if (words.length === 1) return initial(words[0] ?? "");
  return words.slice(0, 2).map(initial).join("");
}
