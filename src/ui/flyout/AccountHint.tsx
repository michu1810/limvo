import { roomierAccount } from "../../core/accounts";
import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";
import { ChevronIcon } from "../icons";
import { ProviderMark } from "../ProviderMark";

/**
 * Points a hot or spent account to another account of the same service that
 * lasts to its reset. A click opens that account's tab; nothing is switched
 * in the service itself, which Limvo only reads.
 */
export function AccountHint({
  report,
  reports,
  now,
  onSelect,
}: {
  report: ProviderReport;
  reports: readonly ProviderReport[];
  now: Date;
  onSelect: (providerId: string) => void;
}) {
  const { t, format } = useI18n();
  const roomier = roomierAccount(report, reports, now);
  if (!roomier) return null;
  const { report: other, window } = roomier;
  const resetsAt = window.resetsAt ? new Date(window.resetsAt) : null;
  const reading =
    resetsAt && resetsAt > now
      ? t("stripUntil", { percent: format.percent(window.percent), day: format.weekday(resetsAt) })
      : format.percent(window.percent);

  return (
    <button
      type="button"
      className="account-hint"
      data-plan={other.snapshot?.plan?.id}
      onClick={() => onSelect(other.providerId)}
    >
      <ProviderMark providerId={other.serviceId} size={16} level={window.percent} />
      <span className="account-hint__text">
        <span className="account-hint__title">
          {t("roomierAccount", { name: other.providerName })}
        </span>
        <span className="account-hint__detail">{reading}</span>
      </span>
      <ChevronIcon />
    </button>
  );
}
