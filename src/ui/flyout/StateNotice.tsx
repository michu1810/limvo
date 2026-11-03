import type { ProviderReport } from "../../core/snapshot";
import { useI18n } from "../../i18n/I18nProvider";

/** Explains why the numbers are missing or old. Silent when everything is fine. */
export function StateNotice({
  report,
  now,
  onRetry,
}: {
  report: ProviderReport;
  now: Date;
  onRetry: () => void;
}) {
  const { t, format } = useI18n();
  const provider = report.snapshot?.providerName ?? "Claude";

  const message = (() => {
    switch (report.state) {
      case "ok":
        return null;
      case "loading":
        return t("stateLoading");
      case "offline":
        return report.snapshot
          ? t("stateOffline", {
              time: format.relativeTime(new Date(report.snapshot.fetchedAt), now),
            })
          : t("stateError");
      case "auth_required":
        return t("stateAuthRequired", { provider });
      case "no_credentials":
        return t("stateNoCredentials", { provider });
      case "error":
        return t("stateError");
    }
  })();

  if (message === null) return null;
  const canRetry = report.state !== "loading";

  return (
    <div className="notice" data-state={report.state} role="status" aria-live="polite">
      <p className="notice__text">{message}</p>
      {canRetry && (
        <button type="button" className="action action--inline" onClick={onRetry}>
          {t("retry")}
        </button>
      )}
    </div>
  );
}
