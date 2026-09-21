use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
    time::Duration,
};

use crate::{
    models::{WidgetConfigChangedDto, WidgetContextDto, WidgetInstanceDto},
    widget_config,
    widget_window_identity::{widget_id_from_window_label, widget_window_label, OpeningRegistry},
    AppState,
};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

const PRIMARY_WINDOW_LABEL: &str = "main";
const GEOMETRY_DEBOUNCE: Duration = Duration::from_millis(400);

#[derive(Default)]
pub struct WidgetWindowRuntimeState {
    opening: OpeningRegistry,
    closing: Mutex<HashSet<String>>,
    geometry_generation: Mutex<HashMap<String, u64>>,
}

pub fn resolve_widget_context(
    window: &WebviewWindow,
    state: &AppState,
) -> Result<WidgetContextDto, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let instance = match widget_id_from_window_label(window.label())? {
        None => widget_config::get_primary_widget_instance(&conn),
        Some(id) => widget_config::get_widget_instance(&conn, &id),
    }
    .map_err(to_string)?;
    let expected_label = widget_window_label(&instance)?;
    if expected_label != window.label() {
        return Err("window label does not match the widget instance".to_string());
    }
    let panes = widget_config::list_widget_panes(&conn, &instance.id).map_err(to_string)?;
    Ok(WidgetContextDto { instance, panes })
}

pub async fn open_widget_window(app: &AppHandle, instance_id: &str) -> Result<(), String> {
    let instance = {
        let state = app
            .try_state::<AppState>()
            .ok_or_else(|| "app state is not initialized".to_string())?;
        let conn = state
            .db
            .lock()
            .map_err(|_| "database lock poisoned".to_string())?;
        widget_config::get_widget_instance(&conn, instance_id).map_err(to_string)?
    };
    if instance.is_primary {
        return Err("the primary widget uses the existing main window".to_string());
    }
    let label = widget_window_label(&instance)?;
    if let Some(existing) = app.get_webview_window(&label) {
        mark_open(app, &instance.id, true)?;
        crate::desktop_window::show_widget_window(&existing)?;
        return Ok(());
    }

    let runtime = app
        .try_state::<WidgetWindowRuntimeState>()
        .ok_or_else(|| "widget window runtime state is not initialized".to_string())?;
    if !runtime.opening.begin(&label)? {
        return Ok(());
    }

    let result = create_widget_window(app, &instance, &label).await;
    runtime.opening.finish(&label);
    result
}

async fn create_widget_window(
    app: &AppHandle,
    instance: &WidgetInstanceDto,
    label: &str,
) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title(&instance.name)
        .inner_size(340.0, 560.0)
        .min_inner_size(300.0, 360.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .skip_taskbar(true)
        .always_on_top(false)
        .always_on_bottom(false)
        .visible(false)
        .build()
        .map_err(to_string)?;

    restore_geometry(&window, instance)?;
    mark_open(app, &instance.id, true)?;
    window.show().map_err(to_string)?;
    if instance.pinned {
        crate::desktop_window::set_widget_window_mode(&window, "pinned").await?;
    } else {
        crate::desktop_window::set_widget_window_mode(&window, "normal").await?;
    }
    eprintln!("[widget-window] opened {label} for {}", instance.id);
    Ok(())
}

pub async fn close_secondary_widget(window: &WebviewWindow) -> Result<(), String> {
    let Some(instance_id) = widget_id_from_window_label(window.label())? else {
        return Err("the primary widget cannot be closed independently".to_string());
    };
    let runtime = window
        .app_handle()
        .try_state::<WidgetWindowRuntimeState>()
        .ok_or_else(|| "widget window runtime state is not initialized".to_string())?;
    {
        let mut closing = runtime
            .closing
            .lock()
            .map_err(|_| "widget closing state lock poisoned".to_string())?;
        closing.insert(window.label().to_string());
    }
    let close_result = async {
        persist_geometry(window)?;
        crate::desktop_window::set_widget_window_mode(window, "normal").await?;
        mark_open(window.app_handle(), &instance_id, false)?;
        emit_widget_config_changed(window.app_handle(), &instance_id);
        crate::desktop_window::remove_window_state(window.app_handle(), window.label());
        window.close().map_err(to_string)
    }
    .await;
    if close_result.is_err() {
        if let Ok(mut closing) = runtime.closing.lock() {
            closing.remove(window.label());
        }
    }
    close_result
}

pub fn emit_widget_config_changed(app: &AppHandle, widget_instance_id: &str) {
    if let Err(error) = app.emit(
        "widget-config-changed",
        WidgetConfigChangedDto {
            widget_instance_id: widget_instance_id.to_string(),
        },
    ) {
        eprintln!("[widget-window] failed to emit widget-config-changed: {error}");
    }
}

pub fn should_intercept_close(app: &AppHandle, label: &str) -> bool {
    if !matches!(widget_id_from_window_label(label), Ok(Some(_))) {
        return false;
    }
    app.try_state::<WidgetWindowRuntimeState>()
        .and_then(|state| {
            state
                .closing
                .lock()
                .ok()
                .map(|closing| !closing.contains(label))
        })
        .unwrap_or(false)
}

pub fn clear_closed_window_state(app: &AppHandle, label: &str) {
    if let Some(runtime) = app.try_state::<WidgetWindowRuntimeState>() {
        if let Ok(mut closing) = runtime.closing.lock() {
            closing.remove(label);
        }
        if let Ok(mut generations) = runtime.geometry_generation.lock() {
            generations.remove(label);
        }
    }
    crate::desktop_window::remove_window_state(app, label);
}

pub fn schedule_geometry_persist(app: &AppHandle, label: &str) {
    if widget_id_from_window_label(label).is_err() {
        return;
    }
    let Some(runtime) = app.try_state::<WidgetWindowRuntimeState>() else {
        return;
    };
    let generation = {
        let Ok(mut generations) = runtime.geometry_generation.lock() else {
            return;
        };
        let generation = generations.entry(label.to_string()).or_default();
        *generation = generation.wrapping_add(1);
        *generation
    };
    let app = app.clone();
    let label = label.to_string();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(GEOMETRY_DEBOUNCE).await;
        let Some(runtime) = app.try_state::<WidgetWindowRuntimeState>() else {
            return;
        };
        let is_latest = runtime
            .geometry_generation
            .lock()
            .ok()
            .and_then(|generations| generations.get(&label).copied())
            == Some(generation);
        if !is_latest {
            return;
        }
        if let Some(window) = app.get_webview_window(&label) {
            if let Err(error) = persist_geometry(&window) {
                eprintln!("[widget-window] failed to persist geometry for {label}: {error}");
            }
        }
    });
}

pub fn persist_geometry(window: &WebviewWindow) -> Result<(), String> {
    let context_id = resolve_instance_id(window)?;
    let position = window.outer_position().map_err(to_string)?;
    let size = window.inner_size().map_err(to_string)?;
    let state = window
        .app_handle()
        .try_state::<AppState>()
        .ok_or_else(|| "app state is not initialized".to_string())?;
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::save_widget_geometry(
        &conn,
        &context_id,
        position.x,
        position.y,
        size.width,
        size.height,
    )
    .map_err(to_string)
}

pub fn list_runtime_widget_windows(app: &AppHandle) -> Vec<WebviewWindow> {
    app.webview_windows()
        .into_values()
        .filter(|window| widget_id_from_window_label(window.label()).is_ok())
        .collect()
}

pub async fn show_open_widget_windows(app: &AppHandle) {
    if let Some(main) = app.get_webview_window(PRIMARY_WINDOW_LABEL) {
        if let Err(error) = crate::desktop_window::show_widget_window(&main) {
            eprintln!("[widget-window] failed to show primary: {error}");
        }
    }
    let instances = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .db
                .lock()
                .ok()
                .and_then(|conn| widget_config::list_open_secondary_widgets(&conn).ok())
        })
        .unwrap_or_default();
    for instance in instances {
        if let Err(error) = open_widget_window(app, &instance.id).await {
            eprintln!("[widget-window] failed to show {}: {error}", instance.id);
        }
    }
}

pub async fn restore_widget_windows(app: AppHandle) {
    let (primary, secondary) = {
        let Some(state) = app.try_state::<AppState>() else {
            return;
        };
        let Ok(conn) = state.db.lock() else {
            eprintln!("[widget-window] startup restore skipped: database lock poisoned");
            return;
        };
        let primary = widget_config::get_primary_widget_instance(&conn);
        let secondary = widget_config::list_open_secondary_widgets(&conn);
        (primary, secondary)
    };

    match primary {
        Ok(instance) => {
            if let Some(main) = app.get_webview_window(PRIMARY_WINDOW_LABEL) {
                if let Err(error) = restore_geometry(&main, &instance) {
                    eprintln!("[widget-window] failed to restore main geometry: {error}");
                }
                let mode = if instance.pinned { "pinned" } else { "normal" };
                if let Err(error) = crate::desktop_window::set_widget_window_mode(&main, mode).await
                {
                    eprintln!("[widget-window] failed to restore main mode: {error}");
                }
            }
        }
        Err(error) => eprintln!("[widget-window] primary restore failed: {error}"),
    }

    match secondary {
        Ok(instances) => {
            for instance in instances {
                if let Err(error) = open_widget_window(&app, &instance.id).await {
                    eprintln!(
                        "[widget-window] failed to restore secondary {}: {error}",
                        instance.id
                    );
                }
            }
        }
        Err(error) => eprintln!("[widget-window] secondary restore failed: {error}"),
    }
}

fn restore_geometry(window: &WebviewWindow, instance: &WidgetInstanceDto) -> Result<(), String> {
    let (Some(x), Some(y), Some(width), Some(height)) = (
        instance.window_x,
        instance.window_y,
        instance.window_width,
        instance.window_height,
    ) else {
        return Ok(());
    };
    if !platform::rect_intersects_monitor(x, y, width, height) {
        eprintln!(
            "[widget-window] stored geometry for {} is off-screen; using platform default",
            instance.id
        );
        return Ok(());
    }
    window
        .set_size(PhysicalSize::new(width, height))
        .map_err(to_string)?;
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(to_string)
}

fn resolve_instance_id(window: &WebviewWindow) -> Result<String, String> {
    match widget_id_from_window_label(window.label())? {
        Some(id) => Ok(id),
        None => {
            let state = window
                .app_handle()
                .try_state::<AppState>()
                .ok_or_else(|| "app state is not initialized".to_string())?;
            let conn = state
                .db
                .lock()
                .map_err(|_| "database lock poisoned".to_string())?;
            Ok(widget_config::get_primary_widget_instance(&conn)
                .map_err(to_string)?
                .id)
        }
    }
}

fn mark_open(app: &AppHandle, id: &str, is_open: bool) -> Result<(), String> {
    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| "app state is not initialized".to_string())?;
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    widget_config::set_widget_open(&conn, id, is_open).map_err(to_string)
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(target_os = "windows")]
mod platform {
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::{MonitorFromRect, MONITOR_DEFAULTTONULL},
    };

    pub(super) fn rect_intersects_monitor(x: i32, y: i32, width: u32, height: u32) -> bool {
        let rect = RECT {
            left: x,
            top: y,
            right: i64::from(x)
                .saturating_add(i64::from(width))
                .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
            bottom: i64::from(y)
                .saturating_add(i64::from(height))
                .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        };
        !unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONULL) }.is_null()
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    pub(super) fn rect_intersects_monitor(_x: i32, _y: i32, _width: u32, _height: u32) -> bool {
        true
    }
}
