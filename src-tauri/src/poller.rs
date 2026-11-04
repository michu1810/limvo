//! Reads every provider on a schedule and publishes every new report.
//!
//! One task owns all network access, so there is never more than one request
//! in flight. Each provider keeps its own due time, back-off and last good
//! reading. Everything else (a manual refresh, opening the flyout, activity
//! in a provider's tools) only wakes that task early.

use std::sync::atomic::{AtomicI64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

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
/// A service that asked to slow down waits at least this long, longer each time.
const FIRST_RATE_LIMIT: Duration = Duration::from_mins(2);
const MAX_RETRY: Duration = Duration::from_mins(15);
/// Requests to one provider closer together than this reuse the last answer.
const MIN_GAP: Duration = Duration::from_secs(10);

/// The provider list waiting for the poller to take over.
type Pending = Arc<Mutex<Option<Vec<Box<dyn UsageProvider>>>>>;

#[derive(Clone)]
pub struct PollerHandle {
    reports: Arc<Mutex<Vec<ProviderReport>>>,
    wake: Arc<Notify>,
    schedule: Arc<Schedule>,
    pending: Pending,
}

impl std::fmt::Debug for PollerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PollerHandle")
            .field("reports", &self.reports().len())
            .finish_non_exhaustive()
    }
}

/// Lock-free state shared between the poller and whoever nudges it.
#[derive(Debug)]
struct Schedule {
    mode: AtomicU8,
    last_activity_ms: AtomicI64,
    last_fetch_ms: AtomicI64,
}

impl PollerHandle {
    /// One report per provider, in the order the interface lists them.
    pub fn reports(&self) -> Vec<ProviderReport> {
        self.reports
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The report of one provider, if it is known.
    pub fn report(&self, provider_id: &str) -> Option<ProviderReport> {
        self.reports()
            .into_iter()
            .find(|report| report.provider_id == provider_id)
    }

    pub fn refresh_now(&self) {
        self.wake.notify_one();
    }

    /// Replaces the providers, for an account added or removed. Providers
    /// that stay keep their schedule and last reading; new ones are read now.
    pub fn set_providers(&self, providers: Vec<Box<dyn UsageProvider>>) {
        *self.pending.lock().unwrap_or_else(PoisonError::into_inner) = Some(providers);
        self.wake.notify_one();
    }

    pub fn set_mode(&self, mode: RefreshMode) {
        self.schedule
            .mode
            .store(mode_to_u8(mode), Ordering::Relaxed);
        self.wake.notify_one();
    }

    /// A provider's tools just wrote something. Read soon, but never more
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

/// What the poller knows about one provider between readings.
struct Slot {
    provider: Box<dyn UsageProvider>,
    due: Instant,
    failures: u32,
    last_fetch: Option<Instant>,
    /// Set when the service asked to slow down: no refresh, opened flyout or
    /// activity brings the next request before it. Waking it early would
    /// only renew the limit, and the reading would never come back.
    held_until: Option<Instant>,
}

pub fn start(
    app: AppHandle,
    providers: Vec<Box<dyn UsageProvider>>,
    mode: RefreshMode,
) -> PollerHandle {
    let reports = providers
        .iter()
        .map(|provider| loading(provider.as_ref()))
        .collect();
    let handle = PollerHandle {
        reports: Arc::new(Mutex::new(reports)),
        pending: Arc::new(Mutex::new(None)),
        wake: Arc::new(Notify::new()),
        schedule: Arc::new(Schedule {
            mode: AtomicU8::new(mode_to_u8(mode)),
            last_activity_ms: AtomicI64::new(i64::MIN / 2),
            last_fetch_ms: AtomicI64::new(i64::MIN / 2),
        }),
    };
    let task = handle.clone();
    tauri::async_runtime::spawn(async move { run(app, providers, task).await });
    handle
}

async fn run(app: AppHandle, providers: Vec<Box<dyn UsageProvider>>, handle: PollerHandle) {
    let http = match http_client() {
        Ok(http) => http,
        Err(error) => {
            log::error!("could not create the HTTP client: {error}");
            return;
        }
    };
    let start = Instant::now();
    let mut slots: Vec<Slot> = providers
        .into_iter()
        .map(|provider| Slot {
            provider,
            due: start,
            failures: 0,
            last_fetch: None,
            held_until: None,
        })
        .collect();

    loop {
        let replacement = handle
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(providers) = replacement {
            slots = replace_slots(&handle, slots, providers);
        }
        let now = Instant::now();
        for (index, slot) in slots.iter_mut().enumerate() {
            let too_soon = slot.last_fetch.is_some_and(|at| at.elapsed() < MIN_GAP);
            if slot.due > now || too_soon {
                continue;
            }
            slot.last_fetch = Some(Instant::now());
            handle
                .schedule
                .last_fetch_ms
                .store(now_ms(), Ordering::Relaxed);
            let outcome = slot.provider.fetch(&http).await;
            slot.failures = if outcome.is_ok() {
                0
            } else {
                slot.failures.saturating_add(1)
            };
            let wait = match &outcome {
                Ok(_) => handle.schedule.healthy_interval(now_ms()),
                Err(error) => retry_delay(error, slot.failures),
            };
            slot.due = Instant::now() + wait;
            slot.held_until =
                matches!(outcome, Err(ProviderError::RateLimited { .. })).then_some(slot.due);
            store(&handle, index, outcome);
        }
        publish(&app, &handle);

        let next = slots.iter().map(|slot| slot.due).min().unwrap_or(now);
        let woken = wait_or_wake(&handle, next.saturating_duration_since(Instant::now())).await;
        if woken {
            // A refresh, an opened flyout or new activity: everyone is due
            // now, except a service that asked to slow down.
            let now = Instant::now();
            for slot in &mut slots {
                slot.due = woken_due(slot.held_until, now);
            }
        }
    }
}

fn loading(provider: &dyn UsageProvider) -> ProviderReport {
    ProviderReport::loading(provider.id(), provider.service(), provider.name())
}

/// The slots and reports for a new provider list, reusing those of providers
/// that stay so their schedule and last reading carry over.
fn replace_slots(
    handle: &PollerHandle,
    mut old: Vec<Slot>,
    providers: Vec<Box<dyn UsageProvider>>,
) -> Vec<Slot> {
    let mut reports = handle
        .reports
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let mut next_reports = Vec::with_capacity(providers.len());
    let mut next_slots = Vec::with_capacity(providers.len());
    for provider in providers {
        let kept = old
            .iter()
            .position(|slot| slot.provider.id() == provider.id())
            .map(|index| old.swap_remove(index));
        let report = reports
            .iter()
            .find(|report| report.provider_id == provider.id())
            .filter(|_| kept.is_some())
            .cloned()
            .map_or_else(
                || loading(provider.as_ref()),
                |mut report| {
                    // A renamed account keeps its readings under its new name.
                    provider.name().clone_into(&mut report.provider_name);
                    report
                },
            );
        next_reports.push(report);
        next_slots.push(match kept {
            Some(slot) => Slot { provider, ..slot },
            None => Slot {
                provider,
                due: Instant::now(),
                failures: 0,
                last_fetch: None,
                held_until: None,
            },
        });
    }
    *reports = next_reports;
    next_slots
}

/// Waits for `wait`, or less when someone asks for a fresh reading; returns whether they did.
async fn wait_or_wake(handle: &PollerHandle, wait: Duration) -> bool {
    tokio::select! {
        () = tokio::time::sleep(wait) => false,
        () = handle.wake.notified() => true,
    }
}

fn store(handle: &PollerHandle, index: usize, outcome: Result<UsageSnapshot, ProviderError>) {
    let mut reports = handle
        .reports
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(report) = reports.get_mut(index) {
        *report = merge(report, outcome);
    }
}

fn publish(app: &AppHandle, handle: &PollerHandle) {
    let reports = handle.reports();
    // The tray icon follows the first provider that has a reading.
    let level = reports
        .iter()
        .find_map(|report| report.snapshot.as_ref()?.headline.as_ref())
        .map(|headline| headline.percent);
    crate::tray::show_level(app, level);
    if let Err(error) = app.emit(events::REPORT_UPDATED, reports) {
        log::warn!("could not publish the reports: {error}");
    }
}

/// Builds the next report. Failures keep the last good snapshot so the UI can show it as stale.
fn merge(
    previous: &ProviderReport,
    outcome: Result<UsageSnapshot, ProviderError>,
) -> ProviderReport {
    match outcome {
        Ok(snapshot) => ProviderReport {
            provider_id: previous.provider_id.clone(),
            service_id: previous.service_id.clone(),
            provider_name: previous.provider_name.clone(),
            state: ReportState::Ok,
            snapshot: Some(snapshot),
            detail: None,
        },
        Err(error) => {
            let state = match &error {
                ProviderError::NoCredentials => ReportState::NoCredentials,
                ProviderError::AuthRequired => ReportState::AuthRequired,
                ProviderError::Unreachable(_) => ReportState::Offline,
                // Not a connection problem: the last reading stands, and its
                // age in the header tells how fresh it is.
                ProviderError::RateLimited { .. } if previous.snapshot.is_some() => ReportState::Ok,
                ProviderError::RateLimited { .. } => ReportState::RateLimited,
                ProviderError::Unexpected(_) => ReportState::Error,
            };
            if !matches!(error, ProviderError::NoCredentials) {
                log::warn!("{} refresh failed: {error}", previous.provider_name);
            }
            ProviderReport {
                provider_id: previous.provider_id.clone(),
                service_id: previous.service_id.clone(),
                provider_name: previous.provider_name.clone(),
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
        ProviderError::RateLimited { retry_after } => retry_after
            .unwrap_or_else(|| rate_limit_backoff(failures))
            .clamp(FIRST_RATE_LIMIT, MAX_RETRY),
        _ => backoff(failures),
    }
}

/// 2 min, 4 min, 8 min, then 15 min.
fn rate_limit_backoff(failures: u32) -> Duration {
    let exponent = failures.saturating_sub(1).min(8);
    FIRST_RATE_LIMIT
        .saturating_mul(1 << exponent)
        .min(MAX_RETRY)
}

/// When a woken slot is due: now, or when its rate limit ends.
fn woken_due(held_until: Option<Instant>, now: Instant) -> Instant {
    held_until.map_or(now, |until| until.max(now))
}

/// 30 s, 60 s, 2 min, 4 min ... capped at 15 min.
fn backoff(failures: u32) -> Duration {
    let exponent = failures.saturating_sub(1).min(8);
    FIRST_RETRY.saturating_mul(1 << exponent).min(MAX_RETRY)
}

fn http_client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("Limvo/", env!("CARGO_PKG_VERSION")))
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
            plan: None,
            fetched_at: "2026-11-08T14:57:00Z".into(),
            headline: None,
            windows: Vec::new(),
            breakdown: Vec::new(),
            extra_usage: None,
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
        let good = merge(
            &ProviderReport::loading("claude", "claude", "Claude"),
            Ok(snapshot()),
        );
        let offline = merge(&good, Err(ProviderError::Unreachable("timeout".into())));
        assert_eq!(offline.state, ReportState::Offline);
        assert_eq!(offline.snapshot, good.snapshot);
    }

    #[test]
    fn rejected_credentials_ask_the_user_to_sign_in() {
        let report = merge(
            &ProviderReport::loading("claude", "claude", "Claude"),
            Err(ProviderError::AuthRequired),
        );
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
        assert_eq!(retry_delay(&limited, 1), FIRST_RATE_LIMIT);
        let long = ProviderError::RateLimited {
            retry_after: Some(Duration::from_hours(2)),
        };
        assert_eq!(retry_delay(&long, 1), MAX_RETRY);
    }

    #[test]
    fn a_rate_limit_without_retry_after_waits_longer_each_time() {
        let limited = ProviderError::RateLimited { retry_after: None };
        assert_eq!(retry_delay(&limited, 1), Duration::from_mins(2));
        assert_eq!(retry_delay(&limited, 2), Duration::from_mins(4));
        assert_eq!(retry_delay(&limited, 9), MAX_RETRY);
    }

    #[test]
    fn waking_never_cuts_a_rate_limit_short() {
        let now = Instant::now();
        let later = now + Duration::from_mins(3);
        assert_eq!(woken_due(None, now), now);
        assert_eq!(woken_due(Some(later), now), later);
        // A limit that has run out no longer holds anything back.
        assert_eq!(woken_due(now.checked_sub(Duration::from_secs(1)), now), now);
    }

    #[test]
    fn a_rate_limit_keeps_the_last_reading_on_screen() {
        let good = merge(
            &ProviderReport::loading("claude", "claude", "Claude"),
            Ok(snapshot()),
        );
        let limited = merge(&good, Err(ProviderError::RateLimited { retry_after: None }));
        assert_eq!(limited.state, ReportState::Ok);
        assert_eq!(limited.snapshot, good.snapshot);

        let first = merge(
            &ProviderReport::loading("claude", "claude", "Claude"),
            Err(ProviderError::RateLimited { retry_after: None }),
        );
        assert_eq!(first.state, ReportState::RateLimited);
    }

    struct Fake(&'static str, &'static str);

    impl UsageProvider for Fake {
        fn service(&self) -> &'static str {
            "claude"
        }
        fn id(&self) -> &str {
            self.0
        }
        fn name(&self) -> &str {
            self.1
        }
        fn fetch<'a>(&'a self, _: &'a reqwest::Client) -> crate::providers::FetchFuture<'a> {
            Box::pin(async { Err(ProviderError::NoCredentials) })
        }
    }

    fn fake(id: &'static str, name: &'static str) -> Box<dyn UsageProvider> {
        Box::new(Fake(id, name))
    }

    #[test]
    fn a_new_account_list_keeps_readings_and_schedules_of_the_accounts_that_stay() {
        let handle = PollerHandle {
            reports: Arc::new(Mutex::new(vec![
                merge(
                    &ProviderReport::loading("claude", "claude", "Claude"),
                    Ok(snapshot()),
                ),
                ProviderReport::loading("claude~2", "claude", "Claude · Old"),
            ])),
            wake: Arc::new(Notify::new()),
            schedule: Arc::new(schedule(RefreshMode::Adaptive, 0)),
            pending: Arc::new(Mutex::new(None)),
        };
        let later = Instant::now() + Duration::from_mins(10);
        let old = vec![
            Slot {
                provider: fake("claude", "Claude"),
                due: later,
                failures: 2,
                last_fetch: None,
                held_until: None,
            },
            Slot {
                provider: fake("claude~2", "Claude · Old"),
                due: later,
                failures: 0,
                last_fetch: None,
                held_until: None,
            },
        ];

        // claude~2 is renamed, claude~3 is new, nothing else changes.
        let slots = replace_slots(
            &handle,
            old,
            vec![
                fake("claude", "Claude"),
                fake("claude~2", "Claude · Work"),
                fake("claude~3", "Claude · Home"),
            ],
        );

        let ids: Vec<_> = slots
            .iter()
            .map(|slot| slot.provider.id().to_owned())
            .collect();
        assert_eq!(ids, ["claude", "claude~2", "claude~3"]);
        assert_eq!((slots[0].due, slots[0].failures), (later, 2));
        assert!(slots[2].due <= Instant::now());

        let reports = handle.reports();
        assert_eq!(reports[0].state, ReportState::Ok);
        assert_eq!(reports[1].provider_name, "Claude · Work");
        assert_eq!(reports[2].state, ReportState::Loading);
        assert_eq!(reports[2].service_id, "claude");
    }

    #[test]
    fn a_removed_account_leaves_no_report_behind() {
        let handle = PollerHandle {
            reports: Arc::new(Mutex::new(vec![
                ProviderReport::loading("claude", "claude", "Claude"),
                ProviderReport::loading("codex~2", "codex", "Codex · Work"),
            ])),
            wake: Arc::new(Notify::new()),
            schedule: Arc::new(schedule(RefreshMode::Adaptive, 0)),
            pending: Arc::new(Mutex::new(None)),
        };
        let now = Instant::now();
        let old = vec![
            Slot {
                provider: fake("claude", "Claude"),
                due: now,
                failures: 0,
                last_fetch: None,
                held_until: None,
            },
            Slot {
                provider: fake("codex~2", "Codex · Work"),
                due: now,
                failures: 0,
                last_fetch: None,
                held_until: None,
            },
        ];
        let slots = replace_slots(&handle, old, vec![fake("claude", "Claude")]);
        assert_eq!(slots.len(), 1);
        assert_eq!(handle.reports().len(), 1);
    }
}
