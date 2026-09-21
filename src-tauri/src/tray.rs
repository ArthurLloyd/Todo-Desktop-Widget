use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn init(app: &AppHandle) -> anyhow::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Widgets", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Widgets", true, None::<&str>)?;
    let sync = MenuItem::with_id(app, "sync", "Sync Now", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &sync, &settings, &quit])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    crate::widget_window::show_open_widget_windows(&app).await;
                });
            }
            "hide" => hide_all_widgets(app),
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
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    match crate::widget_window::activate_primary_from_tray(&app).await {
                        Ok(window) => {
                            if let Err(error) = window.emit("open-settings", ()) {
                                eprintln!("[tray] failed to open primary settings: {error}");
                            }
                        }
                        Err(error) => eprintln!("[tray] failed to activate primary: {error}"),
                    }
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = crate::widget_window::activate_primary_from_tray(&app).await
                    {
                        eprintln!("[tray] failed to activate primary: {error}");
                    }
                });
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn hide_all_widgets(app: &AppHandle) {
    for window in crate::widget_window::list_runtime_widget_windows(app) {
        let _ = crate::desktop_window::hide_widget_window(&window);
    }
}
