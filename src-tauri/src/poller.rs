//! Reads the provider on a schedule and publishes every new report.
//!
//! One task owns all network access, so there is never more than one request
//! in flight. Everything else (a manual refresh, opening the flyout, activity
//! in the provider's tools) only wakes that task early.

use std::sync::atomic::{AtomicI64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;

use crate::events;
use crate::providers::{ProviderError, UsageProvider};
use crate::settings::RefreshMode;
use crate::snapshot::{ProviderReport, ReportState, UsageSnapshot};

/// While the user is working: about once a minute.
const ACTIVE_INTERVAL: Duration = Duration::from_mins(1);
/// Adaptive mode when nothing is happening.
const IDLE_INTERVAL: Duration = Duration::from_mins(5);
/// How long after the last activity the user still counts as working.
const ACTIVE_WINDOW: Duration = Duration::from_mins(5);
const SIGNED_OUT_INTERVAL: Duration = Duration::from_mins(2);
const FIRST_RETRY: Duration = Duration::from_secs(30);
const MAX_RETRY: Duration = Duration::from_mins(15);
/// Requests closer together than this reuse the last answer.
const MIN_GAP: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct PollerHandle {
    report: Arc<Mutex<ProviderReport>>,
    wake: Arc<Notify>,
    schedule: Arc<Schedule>,
}

/// Lock-free state shared between the poller and whoever nudges it.
#[derive(Debug)]
struct Schedule {
    mode: AtomicU8,
    last_activity_ms: AtomicI64,
    last_fetch_ms: AtomicI64,
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

    pub fn set_mode(&self, mode: RefreshMode) {
        self.schedule
            .mode
            .store(mode_to_u8(mode), Ordering::Relaxed);
        self.wake.notify_one();
    }

    /// The provider's tools just wrote something. Read soon, but never more
    /// than once per [`ACTIVE_INTERVAL`], however busy the user is.
    pub fn note_activity(&self) {
        let now = now_ms();
        self.schedule.last_activity_ms.store(now, Ordering::Relaxed);
        if self.schedule.mode() != RefreshMode::Adaptive {
            return;
        }
        let since_fetch = now - self.schedule.last_fetch_ms.load(Ordering::Relaxed);
        if since_fetch >= millis(ACTIVE_INTERVAL) {
            self.wake.notify_one();
        }
    }
}

impl Schedule {
    fn mode(&self) -> RefreshMode {
        mode_from_u8(self.mode.load(Ordering::Relaxed))
    }

    fn healthy_interval(&self, now_ms: i64) -> Duration {
        match self.mode() {
            RefreshMode::Adaptive => {
                let idle_for = now_ms - self.last_activity_ms.load(Ordering::Relaxed);
                if idle_for < millis(ACTIVE_WINDOW) {
                    ACTIVE_INTERVAL
                } else {
                    IDLE_INTERVAL
                }
            }
            RefreshMode::Every2Minutes => Duration::from_mins(2),
            RefreshMode::Every5Minutes => Duration::from_mins(5),
            RefreshMode::Every15Minutes => Duration::from_mins(15),
        }
    }
}

pub fn start(app: AppHandle, provider: Box<dyn UsageProvider>, mode: RefreshMode) -> PollerHandle {
    let handle = PollerHandle {
        report: Arc::new(Mutex::new(ProviderReport::loading())),
        wake: Arc::new(Notify::new()),
        schedule: Arc::new(Schedule {
            mode: AtomicU8::new(mode_to_u8(mode)),
            last_activity_ms: AtomicI64::new(i64::MIN / 2),
            last_fetch_ms: AtomicI64::new(i64::MIN / 2),
        }),
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
    let mut wait = Duration::ZERO;

    loop {
        let since_fetch = now_ms() - handle.schedule.last_fetch_ms.load(Ordering::Relaxed);
        if since_fetch >= millis(MIN_GAP) {
            handle
                .schedule
                .last_fetch_ms
                .store(now_ms(), Ordering::Relaxed);
            let outcome = provider.fetch(&http).await;
            failures = if outcome.is_ok() {
                0
            } else {
                failures.saturating_add(1)
            };
            wait = match &outcome {
                Ok(_) => handle.schedule.healthy_interval(now_ms()),
                Err(error) => retry_delay(error, failures),
            };
            publish(&app, &handle, outcome);
        } else {
            // Woken right after a fetch: show the same answer again instead of asking twice.
            publish_current(&app, &handle);
        }
        wait_or_wake(&handle, wait).await;
    }
}

async fn wait_or_wake(handle: &PollerHandle, wait: Duration) {
    tokio::select! {
        () = tokio::time::sleep(wait) => {}
        () = handle.wake.notified() => {}
    }
}

fn publish(app: &AppHandle, handle: &PollerHandle, outcome: Result<UsageSnapshot, ProviderError>) {
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
    outcome: Result<UsageSnapshot, ProviderError>,
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

fn retry_delay(error: &ProviderError, failures: u32) -> Duration {
    match error {
        // Signing in happens outside the app; check again soon, without hammering.
        ProviderError::NoCredentials | ProviderError::AuthRequired => SIGNED_OUT_INTERVAL,
        ProviderError::RateLimited {
            retry_after: Some(wait),
        } => (*wait).clamp(FIRST_RETRY, MAX_RETRY),
        _ => backoff(failures),
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

fn mode_to_u8(mode: RefreshMode) -> u8 {
    match mode {
        RefreshMode::Adaptive => 0,
        RefreshMode::Every2Minutes => 1,
        RefreshMode::Every5Minutes => 2,
        RefreshMode::Every15Minutes => 3,
    }
}

fn mode_from_u8(value: u8) -> RefreshMode {
    match value {
        1 => RefreshMode::Every2Minutes,
        2 => RefreshMode::Every5Minutes,
        3 => RefreshMode::Every15Minutes,
        _ => RefreshMode::Adaptive,
    }
}

fn now_ms() -> i64 {
    jiff::Timestamp::now().as_millisecond()
}

fn millis(duration: Duration) -> i64 {
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn schedule(mode: RefreshMode, last_activity_ms: i64) -> Schedule {
        Schedule {
            mode: AtomicU8::new(mode_to_u8(mode)),
            last_activity_ms: AtomicI64::new(last_activity_ms),
            last_fetch_ms: AtomicI64::new(0),
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
    fn adaptive_mode_speeds_up_while_the_user_works() {
        let now = 10_000_000;
        let working = schedule(RefreshMode::Adaptive, now - 30_000);
        let idle = schedule(RefreshMode::Adaptive, now - millis(ACTIVE_WINDOW) - 1);
        assert_eq!(working.healthy_interval(now), ACTIVE_INTERVAL);
        assert_eq!(idle.healthy_interval(now), IDLE_INTERVAL);
    }

    #[test]
    fn fixed_modes_ignore_activity() {
        let now = 10_000_000;
        let fixed = schedule(RefreshMode::Every15Minutes, now);
        assert_eq!(fixed.healthy_interval(now), Duration::from_mins(15));
    }

    #[test]
    fn modes_survive_the_atomic_round_trip() {
        for mode in [
            RefreshMode::Adaptive,
            RefreshMode::Every2Minutes,
            RefreshMode::Every5Minutes,
            RefreshMode::Every15Minutes,
        ] {
            assert_eq!(mode_from_u8(mode_to_u8(mode)), mode);
        }
    }

    #[test]
    fn backs_off_exponentially_up_to_a_cap() {
        assert_eq!(backoff(1), Duration::from_secs(30));
        assert_eq!(backoff(3), Duration::from_secs(120));
        assert_eq!(backoff(30), MAX_RETRY);
    }

    #[test]
    fn honours_retry_after_within_bounds() {
        let limited = ProviderError::RateLimited {
            retry_after: Some(Duration::from_secs(3)),
        };
        assert_eq!(retry_delay(&limited, 1), FIRST_RETRY);
    }
}
