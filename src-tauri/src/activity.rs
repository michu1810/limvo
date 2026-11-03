//! Notices when the user is working with the service, so readings can follow along.
//!
//! The usage endpoint has no push channel. Instead of polling it fast all
//! day, Headroom watches files the provider's own tools write while they
//! work (for Claude, the Claude Code transcripts) and reads the usage soon
//! after something happens there. Only modification events are observed;
//! the files are never opened.

use std::fmt;
use std::path::PathBuf;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::poller::PollerHandle;

/// Keeps the operating system watch alive for as long as it is held.
pub struct ActivityWatch {
    watchers: Vec<RecommendedWatcher>,
}

impl fmt::Debug for ActivityWatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActivityWatch")
            .field("watchers", &self.watchers.len())
            .finish()
    }
}

pub fn watch(paths: Vec<PathBuf>, poller: &PollerHandle) -> ActivityWatch {
    let watchers = paths
        .into_iter()
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            let poller = poller.clone();
            let mut watcher =
                notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                    if let Ok(event) = event
                        && matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
                    {
                        poller.note_activity();
                    }
                })
                .map_err(|error| log::warn!("activity watch unavailable: {error}"))
                .ok()?;
            watcher
                .watch(&path, RecursiveMode::Recursive)
                .map_err(|error| log::warn!("cannot watch {}: {error}", path.display()))
                .ok()?;
            log::info!("following activity in {}", path.display());
            Some(watcher)
        })
        .collect();
    ActivityWatch { watchers }
}
