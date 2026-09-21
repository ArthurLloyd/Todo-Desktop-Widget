use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, Notify};

use crate::{auth, db, models::SyncStatusDto, sync, AppState};

const DEFAULT_SYNC_INTERVAL_MINUTES: u64 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum GlobalSyncReason {
    Startup = 1,
    Periodic = 2,
    Login = 3,
    Mutation = 4,
    External = 5,
}

impl GlobalSyncReason {
    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Startup,
            2 => Self::Periodic,
            3 => Self::Login,
            4 => Self::Mutation,
            _ => Self::External,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Periodic => "periodic",
            Self::Login => "login",
            Self::Mutation => "mutation",
            Self::External => "external",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum BackgroundSkipReason {
    None = 0,
    MissingClientId = 1,
    NotAuthenticated = 2,
}

pub struct SyncCoordinator {
    sync_lock: Mutex<()>,
    global_requested: AtomicBool,
    requested_reason: AtomicU8,
    notify: Notify,
    worker_started: AtomicBool,
    sync_active: AtomicBool,
    last_skip_reason: AtomicU8,
}

impl Default for SyncCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncCoordinator {
    pub fn new() -> Self {
        Self {
            sync_lock: Mutex::new(()),
            global_requested: AtomicBool::new(false),
            requested_reason: AtomicU8::new(GlobalSyncReason::External as u8),
            notify: Notify::new(),
            worker_started: AtomicBool::new(false),
            sync_active: AtomicBool::new(false),
            last_skip_reason: AtomicU8::new(BackgroundSkipReason::None as u8),
        }
    }

    pub fn start_worker(self: &Arc<Self>, app: AppHandle) {
        if self.worker_started.swap(true, Ordering::AcqRel) {
            eprintln!("[sync-coordinator] worker already started");
            return;
        }

        let coordinator = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            coordinator.worker_loop(app).await;
        });
    }

    pub fn request_global_sync(&self, reason: GlobalSyncReason) {
        self.mark_global_requested(reason, true);
    }

    pub fn notify_interval_changed(&self, interval_minutes: i64) {
        eprintln!(
            "[sync-coordinator] interval changed to {} minutes",
            interval_minutes.clamp(1, 60)
        );
        self.notify.notify_one();
    }

    pub async fn run_global_sync_now(&self, state: &AppState) -> SyncStatusDto {
        self.run_global_sync(state, "manual-global").await
    }

    pub async fn run_list_sync_now(&self, state: &AppState, list_id: &str) -> SyncStatusDto {
        self.run_serialized(async {
            eprintln!("[sync-coordinator] sync started: reason=manual-list");
            set_syncing_status(state, "Syncing current list");
            let next = sync::sync_list_now(state, list_id).await;
            store_sync_status(state, &next);
            eprintln!(
                "[sync-coordinator] sync completed: reason=manual-list state={}",
                next.state
            );
            next
        })
        .await
    }

    pub(crate) async fn run_direct_global_sync(&self, state: &AppState) -> SyncStatusDto {
        self.run_global_sync(state, "direct").await
    }

    async fn worker_loop(self: Arc<Self>, app: AppHandle) {
        eprintln!("[sync-coordinator] worker started");
        let mut logged_interval = None;

        loop {
            let interval_minutes = read_sync_interval_minutes(&app);
            if logged_interval != Some(interval_minutes) {
                eprintln!("[sync-coordinator] periodic interval = {interval_minutes} minutes");
                logged_interval = Some(interval_minutes);
            }

            let timer = tokio::time::sleep(Duration::from_secs(interval_minutes * 60));
            tokio::pin!(timer);

            tokio::select! {
                _ = self.notify.notified() => {}
                _ = &mut timer => {
                    self.mark_global_requested(GlobalSyncReason::Periodic, false);
                }
            }

            while self.global_requested.load(Ordering::Acquire) {
                let Some(state) = app.try_state::<AppState>() else {
                    eprintln!("[sync-coordinator] sync skipped: app state unavailable");
                    break;
                };

                if let Some(skip_reason) = background_skip_reason(&state) {
                    if let Some(reason) = self.take_global_request() {
                        self.log_background_skip(reason, skip_reason);
                    }
                    continue;
                }

                let Some((_guard, reason)) = self.lock_for_requested_sync().await else {
                    continue;
                };
                self.sync_active.store(true, Ordering::Release);
                let _activity = SyncActivityGuard {
                    active: &self.sync_active,
                };
                self.last_skip_reason
                    .store(BackgroundSkipReason::None as u8, Ordering::Release);
                let _ = self.execute_global_sync(&state, reason.as_str()).await;
            }
        }
    }

    async fn run_global_sync(&self, state: &AppState, reason: &str) -> SyncStatusDto {
        self.run_serialized(self.execute_global_sync(state, reason))
            .await
    }

    async fn execute_global_sync(&self, state: &AppState, reason: &str) -> SyncStatusDto {
        eprintln!("[sync-coordinator] sync started: reason={reason}");
        set_syncing_status(state, "Syncing");
        let next = sync::execute_global_sync(state).await;
        store_sync_status(state, &next);
        eprintln!(
            "[sync-coordinator] sync completed: reason={reason} state={}",
            next.state
        );
        next
    }

    async fn run_serialized<T>(&self, operation: impl Future<Output = T>) -> T {
        let _guard = self.sync_lock.lock().await;
        self.sync_active.store(true, Ordering::Release);
        let _activity = SyncActivityGuard {
            active: &self.sync_active,
        };
        operation.await
    }

    fn mark_global_requested(&self, reason: GlobalSyncReason, notify_worker: bool) {
        self.requested_reason.store(reason as u8, Ordering::Release);
        let already_requested = self.global_requested.swap(true, Ordering::AcqRel);
        if already_requested {
            // The pending follow-up already represents this request.
        } else if self.sync_active.load(Ordering::Acquire) {
            eprintln!(
                "[sync-coordinator] request coalesced while sync active: reason={}",
                reason.as_str()
            );
        } else {
            eprintln!(
                "[sync-coordinator] request global sync: reason={}",
                reason.as_str()
            );
        }
        if notify_worker {
            self.notify.notify_one();
        }
    }

    fn take_global_request(&self) -> Option<GlobalSyncReason> {
        self.global_requested
            .swap(false, Ordering::AcqRel)
            .then(|| GlobalSyncReason::from_u8(self.requested_reason.load(Ordering::Acquire)))
    }

    async fn lock_for_requested_sync(
        &self,
    ) -> Option<(tokio::sync::MutexGuard<'_, ()>, GlobalSyncReason)> {
        let guard = self.sync_lock.lock().await;
        self.take_global_request().map(|reason| (guard, reason))
    }

    fn log_background_skip(
        &self,
        request_reason: GlobalSyncReason,
        skip_reason: BackgroundSkipReason,
    ) {
        let previous = self
            .last_skip_reason
            .swap(skip_reason as u8, Ordering::AcqRel);
        if previous == skip_reason as u8 && request_reason != GlobalSyncReason::Startup {
            return;
        }

        let detail = match skip_reason {
            BackgroundSkipReason::MissingClientId => "Microsoft client ID is not configured",
            BackgroundSkipReason::NotAuthenticated => "not authenticated",
            BackgroundSkipReason::None => return,
        };
        eprintln!(
            "[sync-coordinator] {} sync skipped: {detail}",
            request_reason.as_str()
        );
    }
}

struct SyncActivityGuard<'a> {
    active: &'a AtomicBool,
}

impl Drop for SyncActivityGuard<'_> {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

fn set_syncing_status(state: &AppState, message: &str) {
    match state.sync_status.lock() {
        Ok(mut status) => {
            status.state = "syncing".to_string();
            status.message = Some(message.to_string());
        }
        Err(_) => eprintln!("[sync-coordinator] failed to mark sync status as syncing"),
    }
}

fn store_sync_status(state: &AppState, next: &SyncStatusDto) {
    match state.sync_status.lock() {
        Ok(mut status) => *status = next.clone(),
        Err(_) => eprintln!("[sync-coordinator] failed to store sync status"),
    }
}

fn read_sync_interval_minutes(app: &AppHandle) -> u64 {
    let Some(state) = app.try_state::<AppState>() else {
        return DEFAULT_SYNC_INTERVAL_MINUTES;
    };
    let Ok(conn) = state.db.lock() else {
        eprintln!("[sync-coordinator] failed to read interval: database lock poisoned");
        return DEFAULT_SYNC_INTERVAL_MINUTES;
    };

    match db::load_settings(&conn) {
        Ok(settings) => settings.sync_interval_minutes.clamp(1, 60) as u64,
        Err(error) => {
            eprintln!("[sync-coordinator] failed to read interval: {error}");
            DEFAULT_SYNC_INTERVAL_MINUTES
        }
    }
}

fn background_skip_reason(state: &AppState) -> Option<BackgroundSkipReason> {
    if state.client_id.as_deref().is_none_or(str::is_empty) {
        return Some(BackgroundSkipReason::MissingClientId);
    }
    if !auth::has_refresh_token(state) {
        return Some(BackgroundSkipReason::NotAuthenticated);
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use super::{GlobalSyncReason, SyncCoordinator};

    #[test]
    fn repeated_requests_are_coalesced() {
        let coordinator = SyncCoordinator::new();
        coordinator.request_global_sync(GlobalSyncReason::Mutation);
        coordinator.request_global_sync(GlobalSyncReason::Mutation);
        coordinator.request_global_sync(GlobalSyncReason::Mutation);

        assert!(coordinator.take_global_request().is_some());
        assert!(coordinator.take_global_request().is_none());
    }

    #[tokio::test]
    async fn serialized_work_never_overlaps() {
        let coordinator = Arc::new(SyncCoordinator::new());
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();

        for _ in 0..8 {
            let coordinator = Arc::clone(&coordinator);
            let active = Arc::clone(&active);
            let maximum = Arc::clone(&maximum);
            tasks.push(tokio::spawn(async move {
                coordinator
                    .run_serialized(async {
                        let current = active.fetch_add(1, Ordering::AcqRel) + 1;
                        maximum.fetch_max(current, Ordering::AcqRel);
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        active.fetch_sub(1, Ordering::AcqRel);
                    })
                    .await;
            }));
        }

        for task in tasks {
            assert!(task.await.is_ok());
        }
        assert_eq!(maximum.load(Ordering::Acquire), 1);
    }

    #[tokio::test]
    async fn requests_while_sync_active_schedule_one_follow_up() {
        let coordinator = Arc::new(SyncCoordinator::new());
        let started = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());

        let active_coordinator = Arc::clone(&coordinator);
        let active_started = Arc::clone(&started);
        let active_release = Arc::clone(&release);
        let active_sync = tokio::spawn(async move {
            active_coordinator
                .run_serialized(async {
                    active_started.notify_one();
                    active_release.notified().await;
                })
                .await;
        });

        started.notified().await;
        for _ in 0..20 {
            coordinator.request_global_sync(GlobalSyncReason::Mutation);
        }

        let waiting_coordinator = Arc::clone(&coordinator);
        let follow_up = tokio::spawn(async move {
            waiting_coordinator
                .lock_for_requested_sync()
                .await
                .is_some()
        });
        coordinator.request_global_sync(GlobalSyncReason::Mutation);
        release.notify_one();

        assert!(active_sync.await.is_ok());
        assert!(matches!(follow_up.await, Ok(true)));
        assert!(coordinator.take_global_request().is_none());
    }

    #[tokio::test]
    async fn interval_change_wakes_waiter() {
        let coordinator = SyncCoordinator::new();
        coordinator.notify_interval_changed(1);

        let notified = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            coordinator.notify.notified(),
        )
        .await;
        assert!(notified.is_ok());
    }
}
