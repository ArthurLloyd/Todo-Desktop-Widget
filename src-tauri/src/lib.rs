mod auth;
mod autostart;
#[cfg(not(test))]
mod commands;
#[cfg(not(test))]
mod config;
mod db;
#[cfg(not(test))]
mod debug_console;
#[cfg(not(test))]
mod desktop_window;
mod graph;
#[cfg(not(test))]
mod hotkey;
mod models;
mod settings;
mod sync;
mod sync_coordinator;
#[cfg(not(test))]
mod tray;
mod tray_activation;
mod widget_config;
mod widget_snapshot;
#[cfg(not(test))]
mod widget_window;
mod widget_window_identity;
mod window_state_map;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use models::SyncStatusDto;
use rusqlite::Connection;
#[cfg(not(test))]
use {anyhow::Context, tauri::Manager};

pub struct AppState {
    pub list_mutation_lock: tokio::sync::Mutex<()>,
    pub settings_window_lock: tokio::sync::Mutex<()>,
    pub db: Mutex<Connection>,
    pub http: reqwest::Client,
    pub sync_status: Mutex<SyncStatusDto>,
    pub detail_windows: Mutex<HashMap<String, String>>,
    pub client_id: Option<String>,
    pub tenant: String,
    pub macos_app_group_id: String,
    pub sync_coordinator: Arc<sync_coordinator::SyncCoordinator>,
}

#[cfg(not(test))]
impl AppState {
    fn new(
        app: &tauri::App,
        sync_coordinator: Arc<sync_coordinator::SyncCoordinator>,
    ) -> anyhow::Result<Self> {
        let app_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data directory")?;
        std::fs::create_dir_all(&app_dir).context("failed to create app data directory")?;

        let db_path = app_dir.join("cache.sqlite3");
        let conn = Connection::open(db_path).context("failed to open sqlite cache")?;
        db::init_db(&conn)?;
        db::ensure_local_core_lists(&conn)?;
        widget_config::bootstrap_defaults(&conn)?;
        let settings = db::load_settings(&conn)?;
        debug_console::apply(settings.debug_mode);
        let macos_app_group_id = config::macos_app_group_id(app);
        let _ = widget_snapshot::export_today(&conn, &macos_app_group_id);

        Ok(Self {
            list_mutation_lock: tokio::sync::Mutex::new(()),
            settings_window_lock: tokio::sync::Mutex::new(()),
            db: Mutex::new(conn),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .context("failed to build http client")?,
            sync_status: Mutex::new(SyncStatusDto::idle()),
            detail_windows: Mutex::new(HashMap::new()),
            client_id: config::client_id(app),
            tenant: config::tenant(app),
            macos_app_group_id,
            sync_coordinator,
        })
    }
}

#[cfg(not(test))]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = widget_window::activate_primary_from_tray(&app).await {
                    eprintln!("[single-instance] failed to activate primary widget: {error}");
                }
            });
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let sync_coordinator = Arc::new(sync_coordinator::SyncCoordinator::new());
            let state = AppState::new(app, Arc::clone(&sync_coordinator))?;
            app.manage(state);
            app.manage(Arc::clone(&sync_coordinator));
            app.manage(desktop_window::DesktopWindowState::default());
            app.manage(widget_window::WidgetWindowRuntimeState::default());
            tray::init(app.handle())?;
            hotkey::init(app.handle())?;
            sync_coordinator.start_worker(app.handle().clone());
            sync_coordinator.request_global_sync(sync_coordinator::GlobalSyncReason::Startup);
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(widget_window::restore_widget_windows(app_handle));
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                widget_window::schedule_geometry_persist(window.app_handle(), window.label());
            }
            tauri::WindowEvent::CloseRequested { api, .. }
                if widget_window::should_intercept_close(window.app_handle(), window.label()) =>
            {
                api.prevent_close();
                if let Some(webview) = window.app_handle().get_webview_window(window.label()) {
                    tauri::async_runtime::spawn(async move {
                        if let Err(error) = widget_window::close_secondary_widget(&webview).await {
                            eprintln!("[widget-window] close failed: {error}");
                        }
                    });
                }
            }
            tauri::WindowEvent::Destroyed => {
                widget_window::clear_closed_window_state(window.app_handle(), window.label());
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_today_tasks,
            commands::get_task_lists,
            commands::create_task_list,
            commands::set_widget_opacity,
            commands::open_settings_window,
            commands::close_settings_window,
            commands::get_tasks_for_list,
            commands::get_local_widget_snapshot,
            commands::get_task,
            commands::get_task_id_for_window,
            commands::close_task_details_window,
            commands::add_task,
            commands::add_task_to_list,
            commands::complete_task,
            commands::update_task,
            commands::delete_task,
            commands::open_task_details,
            commands::sync_now,
            commands::sync_list_now,
            commands::login,
            commands::logout,
            commands::get_sync_status,
            commands::get_auth_status,
            commands::get_inbox_count,
            commands::get_settings,
            commands::update_settings,
            commands::get_app_info,
            commands::list_widget_instances,
            commands::get_widget_instance,
            commands::get_widget_context,
            commands::update_managed_widget,
            commands::create_managed_widget_pane,
            commands::update_managed_widget_pane,
            commands::delete_managed_widget_pane,
            commands::reorder_managed_widget_panes,
            commands::reset_managed_widget_pane_weights,
            commands::create_widget_instance,
            commands::update_widget_instance,
            commands::delete_widget_instance,
            commands::list_widget_panes,
            commands::create_widget_pane,
            commands::update_widget_pane,
            commands::delete_widget_pane,
            commands::set_current_widget_pane_weights,
            commands::list_widget_themes,
            commands::get_widget_theme,
            commands::create_widget_theme,
            commands::update_widget_theme,
            commands::delete_widget_theme,
            commands::resolve_widget_pane_theme,
            commands::get_current_widget_context,
            commands::create_current_widget_pane,
            commands::set_current_widget_pane_list,
            commands::delete_current_widget_pane,
            commands::update_current_widget_layout,
            commands::update_current_widget_name,
            commands::set_current_widget_theme,
            commands::set_current_widget_pane_theme,
            commands::create_widget_window,
            commands::duplicate_widget_window,
            commands::open_widget_window,
            commands::show_managed_widget_window,
            commands::close_managed_widget_window,
            commands::set_managed_widget_pinned,
            commands::delete_managed_widget,
            commands::close_current_widget_window,
            commands::update_current_widget_list,
            commands::set_window_mode,
            commands::start_drag,
            commands::start_window_drag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
