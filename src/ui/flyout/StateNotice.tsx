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
      case "rate_limited":
        return t("stateRateLimited", { provider });
      case "auth_required":
        return t("stateAuthRequired", { provider });
      case "no_credentials":
        return t("stateNoCredentials", { provider });
      case "error":
        return t("stateError");
    }
  })();

  if (message === null) return null;
  // Retrying a rate limit early would only renew it; the host tries again itself.
  const canRetry = report.state !== "loading" && report.state !== "rate_limited";

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
