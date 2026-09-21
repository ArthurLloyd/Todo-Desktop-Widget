use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

use crate::{
    auth, db, debug_console,
    models::{
        AuthStatusDto, LoginStartDto, SettingsDto, SettingsPatch, SyncStatusDto, TaskDto,
        TaskListDto, TaskPatchDto, WidgetContextDto, WidgetInstanceDto, WidgetInstanceInput,
        WidgetLayoutMode, WidgetPaneDto, WidgetPaneInput, WidgetTaskSnapshotDto, WidgetThemeDto,
        WidgetThemeInput,
    },
    settings,
    sync_coordinator::GlobalSyncReason,
    widget_config, widget_snapshot, widget_window, AppState,
};

#[tauri::command]
pub fn get_today_tasks(state: State<'_, AppState>) -> Result<Vec<TaskDto>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    db::list_today_tasks(&conn).map_err(to_string)
}

#[tauri::command]
pub fn get_task_lists(state: State<'_, AppState>) -> Result<Vec<TaskListDto>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    db::list_task_lists(&conn).map_err(to_string)
}

#[tauri::command]
pub fn get_tasks_for_list(
    state: State<'_, AppState>,
    list_id: String,
) -> Result<Vec<TaskDto>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    db::list_tasks_for_list(&conn, &list_id).map_err(to_string)
}

#[tauri::command]
pub fn get_local_widget_snapshot(
    state: State<'_, AppState>,
) -> Result<WidgetTaskSnapshotDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    Ok(WidgetTaskSnapshotDto {
        task_lists: db::list_task_lists(&conn).map_err(to_string)?,
        tasks: db::list_all_tasks(&conn).map_err(to_string)?,
        inbox_count: db::inbox_count(&conn).map_err(to_string)?,
    })
}

#[tauri::command]
pub fn get_task(state: State<'_, AppState>, task_id: String) -> Result<TaskDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    db::get_task(&conn, &task_id).map_err(to_string)
}

#[tauri::command]
pub fn get_task_id_for_window(
    state: State<'_, AppState>,
    label: String,
) -> Result<Option<String>, String> {
    let windows = state
        .detail_windows
        .lock()
        .map_err(|_| "detail window lock poisoned".to_string())?;
    Ok(windows.get(&label).cloned())
}

#[tauri::command]
pub fn close_task_details_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(to_string)?;
    }

    if let Some(state) = app.try_state::<AppState>() {
        let mut windows = state
            .detail_windows
            .lock()
            .map_err(|_| "detail window lock poisoned".to_string())?;
        windows.remove(&label);
    }

    Ok(())
}

#[tauri::command]
pub fn add_task(
    app: AppHandle,
    state: State<'_, AppState>,
    title: String,
    list_role: String,
) -> Result<TaskDto, String> {
    let role = match list_role.as_str() {
        "today" | "inbox" | "this_week" => list_role,
        _ => "today".to_string(),
    };

    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let task = db::insert_local_task(&conn, title.trim(), &role).map_err(to_string)?;
    let _ = widget_snapshot::export_today(&conn, &state.macos_app_group_id);
    drop(conn);
    state
        .sync_coordinator
        .request_global_sync(GlobalSyncReason::Mutation);
    emit_local_tasks_changed(&app);
    Ok(task)
}

#[tauri::command]
pub fn add_task_to_list(
    app: AppHandle,
    state: State<'_, AppState>,
    title: String,
    list_id: String,
) -> Result<TaskDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let task = db::insert_local_task_for_list(&conn, title.trim(), &list_id).map_err(to_string)?;
    let _ = widget_snapshot::export_today(&conn, &state.macos_app_group_id);
    drop(conn);
    state
        .sync_coordinator
        .request_global_sync(GlobalSyncReason::Mutation);
    emit_local_tasks_changed(&app);
    Ok(task)
}

#[tauri::command]
pub fn complete_task(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    db::complete_local_task(&conn, &task_id).map_err(to_string)?;
    let _ = widget_snapshot::export_today(&conn, &state.macos_app_group_id);
    drop(conn);
    state
        .sync_coordinator
        .request_global_sync(GlobalSyncReason::Mutation);
    emit_local_tasks_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn update_task(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
    patch: TaskPatchDto,
) -> Result<TaskDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let task = db::update_local_task(&conn, &task_id, &patch).map_err(to_string)?;
    let _ = widget_snapshot::export_today(&conn, &state.macos_app_group_id);
    drop(conn);
    state
        .sync_coordinator
        .request_global_sync(GlobalSyncReason::Mutation);
    emit_local_tasks_changed(&app);
    Ok(task)
}

#[tauri::command]
pub fn delete_task(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let disposition = db::delete_local_task(&conn, &task_id).map_err(to_string)?;
    let _ = widget_snapshot::export_today(&conn, &state.macos_app_group_id);
    drop(conn);

    if disposition == db::DeleteTaskDisposition::RemotePending {
        state
            .sync_coordinator
            .request_global_sync(GlobalSyncReason::Mutation);
    }

    emit_local_tasks_changed(&app);

    Ok(())
}

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>) -> Result<SyncStatusDto, String> {
    Ok(state.sync_coordinator.run_global_sync_now(&state).await)
}

#[tauri::command]
pub async fn sync_list_now(
    state: State<'_, AppState>,
    list_id: String,
) -> Result<SyncStatusDto, String> {
    Ok(state
        .sync_coordinator
        .run_list_sync_now(&state, &list_id)
        .await)
}

#[tauri::command]
pub async fn login(app: AppHandle, state: State<'_, AppState>) -> Result<LoginStartDto, String> {
    let session = auth::prepare_browser_login(&state)
        .await
        .map_err(to_string)?;
    let client_id = state
        .client_id
        .clone()
        .ok_or_else(|| "MICROSOFT_CLIENT_ID is not configured".to_string())?;
    let tenant = state.tenant.clone();
    let http = state.http.clone();
    let auth_url = session.auth_url.clone();

    app.opener()
        .open_url(auth_url.clone(), None::<&str>)
        .map_err(to_string)?;

    {
        let mut status = state
            .sync_status
            .lock()
            .map_err(|_| "sync status lock poisoned".to_string())?;
        status.state = "auth_required".to_string();
        status.message = Some("Waiting for Microsoft login".to_string());
    }

    let app_for_poll = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = auth::complete_browser_login(http, tenant, client_id, session).await;
        let login_succeeded = result.is_ok();

        if let Some(state) = app_for_poll.try_state::<AppState>() {
            if let Ok(mut status) = state.sync_status.lock() {
                match result {
                    Ok(ref refresh_token) => {
                        if let Ok(conn) = state.db.lock() {
                            let _ = db::save_auth_refresh_token(&conn, refresh_token);
                        }
                        status.state = "syncing".to_string();
                        status.message =
                            Some("Login complete. Syncing Microsoft To Do".to_string());
                    }
                    Err(error) => {
                        status.state = "auth_required".to_string();
                        status.message = Some(error.to_string());
                    }
                }
            }

            if login_succeeded && auth::has_refresh_token(&state) {
                state
                    .sync_coordinator
                    .request_global_sync(GlobalSyncReason::Login);
            }
        }
    });

    Ok(LoginStartDto {
        auth_url,
        message: "Microsoft sign-in window opened".to_string(),
    })
}

#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> Result<(), String> {
    auth::clear_refresh_token(&state).map_err(to_string)?;
    let mut status = state
        .sync_status
        .lock()
        .map_err(|_| "sync status lock poisoned".to_string())?;
    *status = SyncStatusDto::auth_required("Logged out");
    Ok(())
}

#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatusDto, String> {
    let mut status = state
        .sync_status
        .lock()
        .map_err(|_| "sync status lock poisoned".to_string())?
        .clone();

    if status.last_synced_at.is_none() {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        status.last_synced_at = db::last_synced_at(&conn).map_err(to_string)?;
    }

    if !auth::has_refresh_token(&state) && status.state == "idle" {
        status = SyncStatusDto::auth_required("Login required");
    }

    Ok(status)
}

#[tauri::command]
pub fn get_auth_status(state: State<'_, AppState>) -> AuthStatusDto {
    let account = state
        .db
        .lock()
        .ok()
        .and_then(|conn| db::load_auth_account(&conn).ok())
        .flatten();

    AuthStatusDto {
        logged_in: auth::has_refresh_token(&state),
        account,
    }
}

#[tauri::command]
pub fn get_inbox_count(state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    db::inbox_count(&conn).map_err(to_string)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    db::load_settings(&conn).map_err(to_string)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: SettingsPatch,
) -> Result<SettingsDto, String> {
    let interval_changed = settings.sync_interval_minutes.is_some();
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let current = db::load_settings(&conn).map_err(to_string)?;
    let next = settings::apply_patch(current, settings);
    db::save_settings(&conn, &next).map_err(to_string)?;
    debug_console::apply(next.debug_mode);
    drop(conn);
    if interval_changed {
        state
            .sync_coordinator
            .notify_interval_changed(next.sync_interval_minutes);
    }
    Ok(next)
}

#[tauri::command]
pub fn list_widget_instances(state: State<'_, AppState>) -> Result<Vec<WidgetInstanceDto>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::list_widget_instances(&conn).map_err(to_string)
}

#[tauri::command]
pub fn get_widget_instance(
    state: State<'_, AppState>,
    id: String,
) -> Result<WidgetInstanceDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::get_widget_instance(&conn, &id).map_err(to_string)
}

#[tauri::command]
pub fn get_widget_context(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<WidgetContextDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let instance = widget_config::get_widget_instance(&conn, &instance_id).map_err(to_string)?;
    let panes = widget_config::list_widget_panes(&conn, &instance_id).map_err(to_string)?;
    Ok(WidgetContextDto { instance, panes })
}

#[tauri::command]
pub fn update_managed_widget(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    input: WidgetInstanceInput,
) -> Result<WidgetInstanceDto, String> {
    let instance = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::update_widget_instance(&conn, &instance_id, &input).map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(&app, &instance_id);
    Ok(instance)
}

#[tauri::command]
pub fn create_managed_widget_pane(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    list_id: Option<String>,
) -> Result<WidgetPaneDto, String> {
    let pane = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::create_pane_for_widget(&conn, &instance_id, list_id.as_deref())
            .map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(&app, &instance_id);
    Ok(pane)
}

#[tauri::command]
pub fn update_managed_widget_pane(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    pane_id: String,
    input: WidgetPaneInput,
) -> Result<WidgetPaneDto, String> {
    let pane = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::update_widget_pane_for_widget(&conn, &instance_id, &pane_id, &input)
            .map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(&app, &instance_id);
    Ok(pane)
}

#[tauri::command]
pub fn delete_managed_widget_pane(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    pane_id: String,
) -> Result<(), String> {
    {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::delete_widget_pane_for_widget(&conn, &instance_id, &pane_id)
            .map_err(to_string)?;
    }
    widget_window::emit_widget_config_changed(&app, &instance_id);
    Ok(())
}

#[tauri::command]
pub fn reorder_managed_widget_panes(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    ordered_pane_ids: Vec<String>,
) -> Result<Vec<WidgetPaneDto>, String> {
    let panes = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::reorder_widget_panes(&conn, &instance_id, &ordered_pane_ids)
            .map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(&app, &instance_id);
    Ok(panes)
}

#[tauri::command]
pub fn create_widget_instance(
    state: State<'_, AppState>,
    input: WidgetInstanceInput,
) -> Result<WidgetInstanceDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::create_widget_instance(&conn, &input).map_err(to_string)
}

#[tauri::command]
pub fn update_widget_instance(
    state: State<'_, AppState>,
    id: String,
    input: WidgetInstanceInput,
) -> Result<WidgetInstanceDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::update_widget_instance(&conn, &id, &input).map_err(to_string)
}

#[tauri::command]
pub fn delete_widget_instance(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::delete_widget_instance(&conn, &id).map_err(to_string)
}

#[tauri::command]
pub fn list_widget_panes(
    state: State<'_, AppState>,
    widget_instance_id: String,
) -> Result<Vec<WidgetPaneDto>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::list_widget_panes(&conn, &widget_instance_id).map_err(to_string)
}

#[tauri::command]
pub fn create_widget_pane(
    window: WebviewWindow,
    state: State<'_, AppState>,
    input: WidgetPaneInput,
) -> Result<WidgetPaneDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    if input.widget_instance_id != context.instance.id {
        return Err("widget pane does not belong to the current widget".to_string());
    }
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let pane = widget_config::create_widget_pane(&conn, &input).map_err(to_string)?;
    drop(conn);
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(pane)
}

#[tauri::command]
pub fn update_widget_pane(
    window: WebviewWindow,
    state: State<'_, AppState>,
    id: String,
    input: WidgetPaneInput,
) -> Result<WidgetPaneDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let pane =
        widget_config::update_widget_pane_for_widget(&conn, &context.instance.id, &id, &input)
            .map_err(to_string)?;
    drop(conn);
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(pane)
}

#[tauri::command]
pub fn delete_widget_pane(
    window: WebviewWindow,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::delete_widget_pane_for_widget(&conn, &context.instance.id, &id)
        .map_err(to_string)?;
    drop(conn);
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(())
}

#[tauri::command]
pub fn list_widget_themes(state: State<'_, AppState>) -> Result<Vec<WidgetThemeDto>, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::list_widget_themes(&conn).map_err(to_string)
}

#[tauri::command]
pub fn get_widget_theme(state: State<'_, AppState>, id: String) -> Result<WidgetThemeDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::get_widget_theme(&conn, &id).map_err(to_string)
}

#[tauri::command]
pub fn create_widget_theme(
    app: AppHandle,
    state: State<'_, AppState>,
    input: WidgetThemeInput,
) -> Result<WidgetThemeDto, String> {
    let theme = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::create_widget_theme(&conn, &input).map_err(to_string)?
    };
    emit_theme_definitions_changed(&app);
    Ok(theme)
}

#[tauri::command]
pub fn update_widget_theme(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: WidgetThemeInput,
) -> Result<WidgetThemeDto, String> {
    let theme = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::update_widget_theme(&conn, &id, &input).map_err(to_string)?
    };
    emit_theme_definitions_changed(&app);
    Ok(theme)
}

#[tauri::command]
pub fn delete_widget_theme(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::delete_widget_theme(&conn, &id).map_err(to_string)?;
    }
    emit_theme_definitions_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn resolve_widget_pane_theme(
    state: State<'_, AppState>,
    pane_id: String,
) -> Result<WidgetThemeDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::resolve_widget_pane_theme(&conn, &pane_id).map_err(to_string)
}

#[tauri::command]
pub fn get_current_widget_context(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<WidgetContextDto, String> {
    widget_window::resolve_widget_context(&window, &state)
}

#[tauri::command]
pub fn create_current_widget_pane(
    window: WebviewWindow,
    state: State<'_, AppState>,
    list_id: Option<String>,
) -> Result<WidgetPaneDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let pane = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::create_pane_for_widget(&conn, &context.instance.id, list_id.as_deref())
            .map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(pane)
}

#[tauri::command]
pub fn set_current_widget_pane_list(
    window: WebviewWindow,
    state: State<'_, AppState>,
    pane_id: String,
    list_id: Option<String>,
) -> Result<WidgetPaneDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let pane = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::set_widget_pane_list_for_widget(
            &conn,
            &context.instance.id,
            &pane_id,
            list_id.as_deref(),
        )
        .map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(pane)
}

#[tauri::command]
pub fn delete_current_widget_pane(
    window: WebviewWindow,
    state: State<'_, AppState>,
    pane_id: String,
) -> Result<(), String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::delete_widget_pane_for_widget(&conn, &context.instance.id, &pane_id)
            .map_err(to_string)?;
    }
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(())
}

#[tauri::command]
pub fn update_current_widget_layout(
    window: WebviewWindow,
    state: State<'_, AppState>,
    layout_mode: WidgetLayoutMode,
) -> Result<WidgetInstanceDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let instance = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::set_widget_layout_mode(&conn, &context.instance.id, layout_mode)
            .map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(instance)
}

#[tauri::command]
pub fn update_current_widget_name(
    window: WebviewWindow,
    state: State<'_, AppState>,
    name: String,
) -> Result<WidgetInstanceDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let instance = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::set_widget_name(&conn, &context.instance.id, &name).map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(instance)
}

#[tauri::command]
pub fn set_current_widget_theme(
    window: WebviewWindow,
    state: State<'_, AppState>,
    theme_id: Option<String>,
) -> Result<WidgetInstanceDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let instance = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::set_widget_default_theme(&conn, &context.instance.id, theme_id.as_deref())
            .map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(instance)
}

#[tauri::command]
pub fn set_current_widget_pane_theme(
    window: WebviewWindow,
    state: State<'_, AppState>,
    pane_id: String,
    theme_id: Option<String>,
) -> Result<WidgetPaneDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let pane = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::set_widget_pane_theme_for_widget(
            &conn,
            &context.instance.id,
            &pane_id,
            theme_id.as_deref(),
        )
        .map_err(to_string)?
    };
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(pane)
}

#[tauri::command]
pub async fn create_widget_window(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    initial_list_id: Option<String>,
) -> Result<WidgetInstanceDto, String> {
    let source_position = window.outer_position().ok();
    let source_size = window.inner_size().ok();
    let instance = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        let instance =
            widget_config::create_widget_with_default_pane(&conn, initial_list_id.as_deref())
                .map_err(to_string)?;
        if let (Some(position), Some(size)) = (source_position, source_size) {
            widget_config::save_widget_geometry(
                &conn,
                &instance.id,
                position.x.saturating_add(32),
                position.y.saturating_add(32),
                size.width,
                size.height,
            )
            .map_err(to_string)?;
        }
        widget_config::get_widget_instance(&conn, &instance.id).map_err(to_string)?
    };
    if let Err(error) = widget_window::open_widget_window(&app, &instance.id).await {
        if let Ok(conn) = state.db.lock() {
            let _ = widget_config::set_widget_open(&conn, &instance.id, false);
        }
        widget_window::emit_widget_config_changed(&app, &instance.id);
        return Err(error);
    }
    widget_window::emit_widget_config_changed(&app, &instance.id);
    Ok(instance)
}

#[tauri::command]
pub async fn duplicate_widget_window(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    source_instance_id: String,
) -> Result<WidgetInstanceDto, String> {
    let fallback_position = window.outer_position().ok();
    let fallback_size = window.inner_size().ok();
    let duplicated = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        let source =
            widget_config::get_widget_instance(&conn, &source_instance_id).map_err(to_string)?;
        let duplicated = widget_config::duplicate_widget_instance(&conn, &source_instance_id)
            .map_err(to_string)?;
        let geometry = match (
            source.window_x,
            source.window_y,
            source.window_width,
            source.window_height,
        ) {
            (Some(x), Some(y), Some(width), Some(height)) => Some((x, y, width, height)),
            _ => fallback_position
                .zip(fallback_size)
                .map(|(position, size)| (position.x, position.y, size.width, size.height)),
        };
        if let Some((x, y, width, height)) = geometry {
            if let Err(error) = widget_config::save_widget_geometry(
                &conn,
                &duplicated.id,
                x.saturating_add(32),
                y.saturating_add(32),
                width,
                height,
            ) {
                eprintln!("[widget-manager] failed to save duplicate geometry: {error}");
            }
        }
        widget_config::get_widget_instance(&conn, &duplicated.id).map_err(to_string)?
    };

    if let Err(error) = widget_window::open_widget_window(&app, &duplicated.id).await {
        if let Ok(conn) = state.db.lock() {
            let _ = widget_config::set_widget_open(&conn, &duplicated.id, false);
        }
        widget_window::emit_widget_config_changed(&app, &duplicated.id);
        return Err(error);
    }
    widget_window::emit_widget_config_changed(&app, &duplicated.id);
    Ok(duplicated)
}

#[tauri::command]
pub async fn open_widget_window(app: AppHandle, instance_id: String) -> Result<(), String> {
    widget_window::open_widget_window(&app, &instance_id).await?;
    widget_window::emit_widget_config_changed(&app, &instance_id);
    Ok(())
}

#[tauri::command]
pub async fn show_managed_widget_window(app: AppHandle, instance_id: String) -> Result<(), String> {
    widget_window::show_widget_instance(&app, &instance_id).await?;
    widget_window::emit_widget_config_changed(&app, &instance_id);
    Ok(())
}

#[tauri::command]
pub async fn close_managed_widget_window(
    app: AppHandle,
    instance_id: String,
) -> Result<(), String> {
    widget_window::close_widget_instance(&app, &instance_id).await
}

#[tauri::command]
pub async fn set_managed_widget_pinned(
    app: AppHandle,
    instance_id: String,
    pinned: bool,
) -> Result<(), String> {
    widget_window::set_widget_instance_pinned(&app, &instance_id, pinned).await
}

#[tauri::command]
pub async fn delete_managed_widget(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instance = {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::get_widget_instance(&conn, &instance_id).map_err(to_string)?
    };
    if instance.is_primary {
        return Err("the primary widget instance cannot be deleted".to_string());
    }
    widget_window::close_widget_instance(&app, &instance_id).await?;
    {
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::delete_widget_instance(&conn, &instance_id).map_err(to_string)?;
    }
    widget_window::emit_widget_config_changed(&app, &instance_id);
    Ok(())
}

#[tauri::command]
pub async fn close_current_widget_window(window: WebviewWindow) -> Result<(), String> {
    widget_window::close_secondary_widget(&window).await?;
    Ok(())
}

#[tauri::command]
pub fn update_current_widget_list(
    window: WebviewWindow,
    state: State<'_, AppState>,
    list_id: String,
) -> Result<WidgetPaneDto, String> {
    let context = widget_window::resolve_widget_context(&window, &state)?;
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let pane = widget_config::update_first_pane_list(&conn, &context.instance.id, &list_id)
        .map_err(to_string)?;
    drop(conn);
    widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    Ok(pane)
}

#[tauri::command]
pub async fn open_task_details(app: AppHandle, task_id: String) -> Result<(), String> {
    let hash = format!("{:x}", Sha256::digest(task_id.as_bytes()));
    let label = format!("task-details-{}", &hash[..16]);

    if let Some(state) = app.try_state::<AppState>() {
        let mut windows = state
            .detail_windows
            .lock()
            .map_err(|_| "detail window lock poisoned".to_string())?;
        windows.insert(label.clone(), task_id);
    }

    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(to_string)?;
        window.set_focus().map_err(to_string)?;
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, label, WebviewUrl::App("index.html".into()))
        .title("Task details")
        .inner_size(430.0, 640.0)
        .min_inner_size(360.0, 480.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .visible(true)
        .build()
        .map_err(to_string)?;

    Ok(())
}

#[tauri::command]
pub async fn set_window_mode(
    window: WebviewWindow,
    state: State<'_, AppState>,
    mode: String,
) -> Result<(), String> {
    crate::desktop_window::set_widget_window_mode(&window, &mode).await?;
    if mode == "pinned" || mode == "normal" {
        let context = widget_window::resolve_widget_context(&window, &state)?;
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::set_widget_pinned(&conn, &context.instance.id, mode == "pinned")
            .map_err(to_string)?;
        widget_window::emit_widget_config_changed(window.app_handle(), &context.instance.id);
    }
    Ok(())
}

#[tauri::command]
pub fn start_drag(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(to_string)
}

#[tauri::command]
pub fn start_window_drag(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(to_string)
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn emit_local_tasks_changed(app: &AppHandle) {
    if let Err(error) = app.emit("local-tasks-changed", ()) {
        eprintln!("[tasks] failed to emit local-tasks-changed: {error}");
    }
}

fn emit_theme_definitions_changed(app: &AppHandle) {
    if let Err(error) = app.emit("theme-definitions-changed", ()) {
        eprintln!("[theme] failed to emit theme-definitions-changed: {error}");
    }
}
