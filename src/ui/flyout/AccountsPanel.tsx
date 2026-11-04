import { useState } from "react";
import { bridge, type Settings } from "../../bridge";
import type { AccountError, AccountService } from "../../bridge/types";
import { useI18n } from "../../i18n/I18nProvider";
import type { Messages } from "../../i18n/types";
import { Segmented } from "../controls";
import { RemoveIcon } from "../icons";
import { ProviderMark } from "../ProviderMark";

const SERVICES: readonly { value: AccountService; label: string }[] = [
  { value: "claude", label: "Claude" },
  { value: "codex", label: "Codex" },
];
/** Mirrors `MAX_ACCOUNTS` and `MAX_LABEL` in `src-tauri/src/accounts.rs`. */
const MAX_ACCOUNTS = 4;
const MAX_LABEL = 24;

const ERRORS: Record<Exclude<AccountError, "cancelled">, keyof Messages> = {
  too_many: "accountErrorTooMany",
  default_folder: "accountErrorDefault",
  duplicate: "accountErrorDuplicate",
  no_sign_in: "accountErrorNoSignIn",
};

/**
 * Extra Claude Code and Codex accounts, each signed in from its own folder.
 * The folder is chosen in the system dialog the host opens; this panel only
 * names the service and the label.
 */
export function AccountsPanel({ settings }: { settings: Settings | null }) {
  const { t } = useI18n();
  const [service, setService] = useState<AccountService>("claude");
  const [label, setLabel] = useState("");
  const [error, setError] = useState<keyof Messages | null>(null);
  const [busy, setBusy] = useState(false);
  const accounts = settings?.accounts ?? [];
  const serviceName = SERVICES.find((option) => option.value === service)?.label ?? service;

  const add = async () => {
    setBusy(true);
    setError(null);
    try {
      await bridge.addAccount(service, label, t("accountFolderTitle", { service: serviceName }));
      setLabel("");
    } catch (code) {
      const key = ERRORS[code as keyof typeof ERRORS];
      // Closing the dialog is not a failure; anything unknown reads as "no sign-in".
      if (code !== "cancelled") setError(key ?? "accountErrorNoSignIn");
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="settings__group" aria-label={t("accounts")}>
      <div className="setting setting--heading">
        <span className="setting__text">
          <span className="setting__label">{t("accounts")}</span>
          <span className="setting__hint">{t("accountsHint")}</span>
        </span>
      </div>

      {accounts.map((account) => {
        const name = `${account.service === "codex" ? "Codex" : "Claude"} · ${account.label}`;
        return (
          <div key={account.id} className="setting account">
            <ProviderMark providerId={account.service} size={16} />
            <span className="setting__text">
              <span className="setting__label">{name}</span>
              <span className="setting__hint account__folder" dir="ltr" title={account.folder}>
                {account.folder}
              </span>
            </span>
            <button
              type="button"
              className="icon-button"
              aria-label={t("removeAccount", { name })}
              title={t("removeAccount", { name })}
              onClick={() => bridge.removeAccount(account.id)}
            >
              <RemoveIcon />
            </button>
          </div>
        );
      })}

      {accounts.length < MAX_ACCOUNTS && (
        <div className="setting setting--stacked account-form">
          <Segmented
            label={t("addAccount")}
            value={service}
            options={SERVICES}
            onChange={setService}
          />
          <div className="account-form__row">
            <input
              className="account-form__name"
              type="text"
              value={label}
              maxLength={MAX_LABEL}
              placeholder={t("accountName")}
              aria-label={t("accountName")}
              onChange={(event) => setLabel(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !busy) add();
              }}
            />
            <button
              type="button"
              className="action action--inline action--accent"
              disabled={busy}
              onClick={add}
            >
              {t("chooseFolder")}
            </button>
          </div>
          {error && (
            <p className="account-form__error" role="alert">
              {t(error, { service: serviceName, count: String(MAX_ACCOUNTS) })}
            </p>
          )}
        </div>
      )}
    </section>
  );
}
