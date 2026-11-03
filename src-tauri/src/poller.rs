//! Reads the provider on a schedule and publishes every new report.
//!
//! One task owns all network access, so there is never more than one request
//! in flight. A manual refresh only wakes that task early.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;

use crate::events;
use crate::providers::{ProviderError, UsageProvider};
use crate::snapshot::{ProviderReport, ReportState};

const HEALTHY_INTERVAL: Duration = Duration::from_mins(4);
const SIGNED_OUT_INTERVAL: Duration = Duration::from_mins(2);
const FIRST_RETRY: Duration = Duration::from_secs(30);
const MAX_RETRY: Duration = Duration::from_mins(15);
/// Manual refreshes closer together than this reuse the last answer.
const MIN_GAP: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct PollerHandle {
    report: Arc<Mutex<ProviderReport>>,
    wake: Arc<Notify>,
}

impl PollerHandle {
    pub fn report(&self) -> ProviderReport {
        self.report
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn refresh_now(&self) {
        self.wake.notify_one();
    }
}

pub fn start(app: AppHandle, provider: Box<dyn UsageProvider>) -> PollerHandle {
    let handle = PollerHandle {
        report: Arc::new(Mutex::new(ProviderReport::loading())),
        wake: Arc::new(Notify::new()),
    };
    let task = handle.clone();
    tauri::async_runtime::spawn(async move { run(app, provider, task).await });
    handle
}

async fn run(app: AppHandle, provider: Box<dyn UsageProvider>, handle: PollerHandle) {
    let http = match http_client() {
        Ok(http) => http,
        Err(error) => {
            log::error!("could not create the HTTP client: {error}");
            return;
        }
    };
    let mut failures: u32 = 0;
    let mut last_fetch: Option<Instant> = None;

    loop {
        if last_fetch.is_none_or(|at| at.elapsed() >= MIN_GAP) {
            last_fetch = Some(Instant::now());
            let outcome = provider.fetch(&http).await;
            failures = if outcome.is_ok() {
                0
            } else {
                failures.saturating_add(1)
            };
            let wait = next_delay(&outcome, failures);
            publish(&app, &handle, outcome);
            wait_or_wake(&handle, wait).await;
        } else {
            // A manual refresh arrived right after a fetch: show the same answer again.
            publish_current(&app, &handle);
            wait_or_wake(&handle, HEALTHY_INTERVAL).await;
        }
    }
}

async fn wait_or_wake(handle: &PollerHandle, wait: Duration) {
    tokio::select! {
        () = tokio::time::sleep(wait) => {}
        () = handle.wake.notified() => {}
    }
}

fn publish(
    app: &AppHandle,
    handle: &PollerHandle,
    outcome: Result<crate::snapshot::UsageSnapshot, ProviderError>,
) {
    {
        let mut report = handle.report.lock().unwrap_or_else(PoisonError::into_inner);
        *report = merge(&report, outcome);
    }
    publish_current(app, handle);
}

fn publish_current(app: &AppHandle, handle: &PollerHandle) {
    if let Err(error) = app.emit(events::REPORT_UPDATED, handle.report()) {
        log::warn!("could not publish the report: {error}");
    }
}

/// Builds the next report. Failures keep the last good snapshot so the UI can show it as stale.
fn merge(
    previous: &ProviderReport,
    outcome: Result<crate::snapshot::UsageSnapshot, ProviderError>,
) -> ProviderReport {
    match outcome {
        Ok(snapshot) => ProviderReport {
            state: ReportState::Ok,
            snapshot: Some(snapshot),
            detail: None,
        },
        Err(error) => {
            let state = match &error {
                ProviderError::NoCredentials => ReportState::NoCredentials,
                ProviderError::AuthRequired => ReportState::AuthRequired,
                ProviderError::Unreachable(_) | ProviderError::RateLimited { .. } => {
                    ReportState::Offline
                }
                ProviderError::Unexpected(_) => ReportState::Error,
            };
            log::warn!("usage refresh failed: {error}");
            ProviderReport {
                state,
                snapshot: previous.snapshot.clone(),
                detail: Some(error.to_string()),
            }
        }
    }
}

fn next_delay(
    outcome: &Result<crate::snapshot::UsageSnapshot, ProviderError>,
    failures: u32,
) -> Duration {
    match outcome {
        Ok(_) => HEALTHY_INTERVAL,
        // Signing in happens outside the app; check again soon, without hammering.
        Err(ProviderError::NoCredentials | ProviderError::AuthRequired) => SIGNED_OUT_INTERVAL,
        Err(ProviderError::RateLimited {
            retry_after: Some(wait),
        }) => (*wait).clamp(FIRST_RETRY, MAX_RETRY),
        Err(_) => backoff(failures),
    }
}

/// 30 s, 60 s, 2 min, 4 min ... capped at 15 min.
fn backoff(failures: u32) -> Duration {
    let exponent = failures.saturating_sub(1).min(8);
    FIRST_RETRY.saturating_mul(1 << exponent).min(MAX_RETRY)
}

fn http_client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("Headroom/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::UsageSnapshot;

    fn snapshot() -> UsageSnapshot {
        UsageSnapshot {
            provider_id: "claude".into(),
            provider_name: "Claude".into(),
            fetched_at: "2026-11-08T14:57:00Z".into(),
            headline: None,
            windows: Vec::new(),
            breakdown: Vec::new(),
            usage_page_url: None,
        }
    }

    #[test]
    fn a_failure_keeps_the_last_good_snapshot() {
        let good = merge(&ProviderReport::loading(), Ok(snapshot()));
        let offline = merge(&good, Err(ProviderError::Unreachable("timeout".into())));
        assert_eq!(offline.state, ReportState::Offline);
        assert_eq!(offline.snapshot, good.snapshot);
    }

    #[test]
    fn rejected_credentials_ask_the_user_to_sign_in() {
        let report = merge(&ProviderReport::loading(), Err(ProviderError::AuthRequired));
        assert_eq!(report.state, ReportState::AuthRequired);
    }

    #[test]
    fn backs_off_exponentially_up_to_a_cap() {
        assert_eq!(backoff(1), Duration::from_secs(30));
        assert_eq!(backoff(3), Duration::from_secs(120));
        assert_eq!(backoff(30), MAX_RETRY);
    }

    #[test]
    fn honours_retry_after_within_bounds() {
        let limited = Err(ProviderError::RateLimited {
            retry_after: Some(Duration::from_secs(3)),
        });
        assert_eq!(next_delay(&limited, 1), FIRST_RETRY);
    }
}
