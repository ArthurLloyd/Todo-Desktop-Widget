use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn init(app: &AppHandle) -> anyhow::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Widget", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Widget", true, None::<&str>)?;
    let pin = MenuItem::with_id(app, "pin", "Pin / Unpin", true, None::<&str>)?;
    let sync = MenuItem::with_id(app, "sync", "Sync Now", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &pin, &sync, &settings, &quit])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if let Some(window) = app.get_webview_window("main") {
                match event.id.as_ref() {
                    "show" => {
                        let _ = crate::desktop_window::show_main_window(app);
                    }
                    "hide" => {
                        let _ = crate::desktop_window::hide_main_window(app);
                    }
                    "pin" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let mode = if crate::desktop_window::is_pinned(&app) {
                                "normal"
                            } else {
                                "pinned"
                            };
                            if let Err(error) =
                                crate::desktop_window::set_main_window_mode(&app, mode).await
                            {
                                eprintln!("[desktop] tray mode change failed: {error}");
                            }
                        });
                    }
                    "sync" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(state) = app.try_state::<crate::AppState>() {
                                let next = crate::sync::sync_now(&state).await;
                                if let Ok(mut status) = state.sync_status.lock() {
                                    *status = next;
                                }
                            }
                        });
                    }
                    "settings" => {
                        let _ = crate::desktop_window::show_main_window(app);
                        let _ = window.set_focus();
                        let _ = crate::desktop_window::enforce_main_window_z_order(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = crate::desktop_window::hide_main_window(tray.app_handle());
                    } else {
                        let _ = crate::desktop_window::show_main_window(tray.app_handle());
                    }
                }
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}
