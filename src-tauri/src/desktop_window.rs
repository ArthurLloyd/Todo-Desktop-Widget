use std::sync::Mutex;

use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Clone, Debug)]
struct DesktopOwnerAttachment {
    main_hwnd: isize,
    host_hwnd: isize,
    def_view_hwnd: isize,
    original_owner: isize,
    dpi_before: u32,
}

#[derive(Default)]
struct DesktopWindowStatus {
    attachment: Option<DesktopOwnerAttachment>,
    pinned: bool,
}

#[derive(Default)]
pub struct DesktopWindowState {
    status: Mutex<DesktopWindowStatus>,
}

pub async fn set_main_window_mode(app: &AppHandle, mode: &str) -> Result<(), String> {
    eprintln!("[desktop] requested mode = {mode}");
    eprintln!("[desktop] state.pinned before = {}", pinned_state(app)?);

    let result = match mode {
        "pinned" => pin_main_window(app).await,
        "normal" => restore_normal_window(app).await,
        "hidden" => hide_main_window(app),
        _ => Err(format!("unsupported window mode: {mode}")),
    };

    match pinned_state(app) {
        Ok(pinned) => eprintln!("[desktop] state.pinned after = {pinned}"),
        Err(error) => eprintln!("[desktop] failed to read state after mode change: {error}"),
    }

    result
}

pub fn is_owner_attached(app: &AppHandle) -> bool {
    app.try_state::<DesktopWindowState>()
        .and_then(|state| {
            state
                .status
                .lock()
                .ok()
                .map(|status| status.attachment.is_some())
        })
        .unwrap_or(false)
}

pub fn is_pinned(app: &AppHandle) -> bool {
    pinned_state(app).unwrap_or(false)
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window.show().map_err(to_string)?;

    if let Some(attachment) = current_attachment(app)? {
        if let Err(error) = platform::validate_desktop_owner(&attachment) {
            eprintln!("[desktop] owner validation failed after show: {error}");
            if pinned_state(app)? {
                return enable_bottom_fallback(&window);
            }
        }
    }

    enforce_main_window_z_order(app)
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    main_window(app)?.hide().map_err(to_string)
}

pub fn enable_bottom_fallback(window: &WebviewWindow) -> Result<(), String> {
    window.set_always_on_top(false).map_err(to_string)?;
    window.set_always_on_bottom(true).map_err(to_string)?;
    window.show().map_err(to_string)?;
    eprintln!("[desktop] using always-on-bottom fallback");
    Ok(())
}

pub fn enforce_main_window_z_order(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    let pinned = pinned_state(app)?;
    let attachment_exists = current_attachment(app)?.is_some();

    eprintln!("[desktop] enforce requested");
    eprintln!("[desktop] pinned = {pinned}");
    eprintln!("[desktop] attachment exists = {attachment_exists}");

    if pinned {
        window.set_always_on_top(false).map_err(to_string)?;
        window.set_always_on_bottom(true).map_err(to_string)
    } else {
        window.set_always_on_bottom(false).map_err(to_string)?;
        window.set_always_on_top(false).map_err(to_string)
    }
}

async fn pin_main_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;

    if is_owner_attached(app) {
        mark_pinned(app, true)?;
        return show_main_window(app);
    }

    window.set_always_on_top(false).map_err(to_string)?;

    let native_window = window.clone();
    let attach_result = run_on_main_thread(&window, move || {
        platform::attach_desktop_owner(&native_window)
    })
    .await;

    match attach_result {
        Ok(attachment) => {
            let dpi_attachment = attachment.clone();
            let state = app
                .try_state::<DesktopWindowState>()
                .ok_or_else(|| "desktop window state is not initialized".to_string())?;
            let mut status = state
                .status
                .lock()
                .map_err(|_| "desktop window state lock poisoned".to_string())?;
            status.attachment = Some(attachment);
            status.pinned = true;
            drop(status);

            enforce_main_window_z_order(app)?;
            window.show().map_err(to_string)?;
            platform::log_dpi_after_always_on_bottom(&dpi_attachment)
        }
        Err(error) => {
            eprintln!("[desktop] owner attach failed: {error}");
            mark_pinned(app, true)?;
            enable_bottom_fallback(&window)
        }
    }
}

async fn restore_normal_window(app: &AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    let attachment = current_attachment(app)?;
    mark_pinned(app, false)?;

    if let Some(attachment) = attachment {
        if let Err(error) =
            run_on_main_thread(&window, move || platform::detach_desktop_owner(&attachment)).await
        {
            mark_pinned(app, true)?;
            if let Err(enforce_error) = enforce_main_window_z_order(app) {
                eprintln!(
                    "[desktop] failed to restore pinned Z-order after detach error: {enforce_error}"
                );
            }
            return Err(error);
        }

        let state = app
            .try_state::<DesktopWindowState>()
            .ok_or_else(|| "desktop window state is not initialized".to_string())?;
        let mut status = state
            .status
            .lock()
            .map_err(|_| "desktop window state lock poisoned".to_string())?;
        status.attachment = None;
    }

    window.set_always_on_bottom(false).map_err(to_string)?;
    window.set_always_on_top(false).map_err(to_string)?;
    window.show().map_err(to_string)?;

    let native_window = window.clone();
    let actual_owner = run_on_main_thread(&window, move || {
        platform::restore_normal_z_order(&native_window)
    })
    .await?;

    eprintln!("[desktop] normal mode final:");
    eprintln!("[desktop] owner = {actual_owner}");
    eprintln!("[desktop] pinned = {}", pinned_state(app)?);
    eprintln!("[desktop] bottom requested = false");
    Ok(())
}

fn current_attachment(app: &AppHandle) -> Result<Option<DesktopOwnerAttachment>, String> {
    let state = app
        .try_state::<DesktopWindowState>()
        .ok_or_else(|| "desktop window state is not initialized".to_string())?;
    let status = state
        .status
        .lock()
        .map_err(|_| "desktop window state lock poisoned".to_string())?;
    Ok(status.attachment.clone())
}

fn mark_pinned(app: &AppHandle, pinned: bool) -> Result<(), String> {
    let state = app
        .try_state::<DesktopWindowState>()
        .ok_or_else(|| "desktop window state is not initialized".to_string())?;
    let mut status = state
        .status
        .lock()
        .map_err(|_| "desktop window state lock poisoned".to_string())?;
    status.pinned = pinned;
    Ok(())
}

fn pinned_state(app: &AppHandle) -> Result<bool, String> {
    let state = app
        .try_state::<DesktopWindowState>()
        .ok_or_else(|| "desktop window state is not initialized".to_string())?;
    let status = state
        .status
        .lock()
        .map_err(|_| "desktop window state lock poisoned".to_string())?;
    Ok(status.pinned)
}

fn main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    if window.label() != "main" {
        return Err("only the main window can enter desktop owner mode".to_string());
    }

    Ok(window)
}

async fn run_on_main_thread<T, F>(window: &WebviewWindow, operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    window
        .run_on_main_thread(move || {
            let _ = sender.send(operation());
        })
        .map_err(to_string)?;

    receiver
        .await
        .map_err(|_| "desktop window operation was cancelled".to_string())?
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(target_os = "windows")]
mod platform {
    use std::ptr::{null, null_mut};

    use tauri::WebviewWindow;
    use windows_sys::Win32::{
        Foundation::{GetLastError, SetLastError, HWND, LPARAM},
        UI::{
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{
                EnumWindows, FindWindowExW, GetClassNameW, GetParent, GetWindowLongPtrW, IsWindow,
                SetWindowLongPtrW, SetWindowPos, GWLP_HWNDPARENT, HWND_TOP, SWP_NOACTIVATE,
                SWP_NOMOVE, SWP_NOSIZE,
            },
        },
    };

    use super::DesktopOwnerAttachment;

    #[derive(Clone, Copy)]
    struct DesktopHost {
        host: HWND,
        def_view: HWND,
    }

    struct EnumContext {
        def_view_class: Vec<u16>,
        found: Option<DesktopHost>,
    }

    pub(super) fn attach_desktop_owner(
        window: &WebviewWindow,
    ) -> Result<DesktopOwnerAttachment, String> {
        if window.label() != "main" {
            return Err("only the main window can enter desktop owner mode".to_string());
        }

        let main = window.hwnd().map_err(|error| error.to_string())?.0 as HWND;
        ensure_window(main, "main window")?;

        let host = find_desktop_host()?;
        let host_class = class_name(host.host)?;
        eprintln!(
            "[desktop] found owner host: {} ({})",
            format_hwnd(host.host),
            host_class
        );
        eprintln!("[desktop] defview: {}", format_hwnd(host.def_view));
        eprintln!("[desktop] main top-level HWND: {}", format_hwnd(main));

        let original_owner =
            get_window_long(main, GWLP_HWNDPARENT, "read original main window owner")?;
        let dpi_before = unsafe { GetDpiForWindow(main) };
        eprintln!("[desktop] main DPI before owner attach: {dpi_before}");

        let attachment = DesktopOwnerAttachment {
            main_hwnd: main as isize,
            host_hwnd: host.host as isize,
            def_view_hwnd: host.def_view as isize,
            original_owner,
            dpi_before,
        };

        if let Err(error) = set_window_long(
            main,
            GWLP_HWNDPARENT,
            host.host as isize,
            "set desktop host as main window owner",
        ) {
            return Err(error);
        }

        if let Err(error) = validate_owner_attachment(main, host.host, host.def_view) {
            return rollback_owner(&attachment, error);
        }

        let dpi_after = unsafe { GetDpiForWindow(main) };
        eprintln!("[desktop] main DPI after owner attach: {dpi_after}");
        if dpi_before != dpi_after {
            eprintln!(
                "[desktop] warning: main DPI changed during owner attach: {dpi_before} -> {dpi_after}"
            );
        }

        eprintln!("[desktop] owner attach succeeded");
        Ok(attachment)
    }

    pub(super) fn detach_desktop_owner(attachment: &DesktopOwnerAttachment) -> Result<(), String> {
        let main = as_hwnd(attachment.main_hwnd);
        let host = as_hwnd(attachment.host_hwnd);
        let def_view = as_hwnd(attachment.def_view_hwnd);
        ensure_window(main, "main window during owner restore")?;

        if let Err(error) = validate_desktop_host(host, def_view) {
            eprintln!("[desktop] owner host changed before detach: {error}");
        }

        if attachment.original_owner != 0 {
            ensure_window(
                as_hwnd(attachment.original_owner),
                "original main window owner",
            )?;
        }

        set_window_long(
            main,
            GWLP_HWNDPARENT,
            attachment.original_owner,
            "restore original main window owner",
        )?;

        let restored_owner =
            get_window_long(main, GWLP_HWNDPARENT, "verify restored main window owner")?;
        if restored_owner != attachment.original_owner {
            return Err(format!(
                "restored owner mismatch for main HWND {}: expected {}, found {}",
                format_hwnd(main),
                format_handle_value(attachment.original_owner),
                format_handle_value(restored_owner)
            ));
        }

        let dpi_after_detach = unsafe { GetDpiForWindow(main) };
        eprintln!(
            "[desktop] main DPI after owner detach: {dpi_after_detach} (before attach: {})",
            attachment.dpi_before
        );
        eprintln!("[desktop] owner detached successfully");
        Ok(())
    }

    pub(super) fn validate_desktop_owner(
        attachment: &DesktopOwnerAttachment,
    ) -> Result<(), String> {
        validate_owner_attachment(
            as_hwnd(attachment.main_hwnd),
            as_hwnd(attachment.host_hwnd),
            as_hwnd(attachment.def_view_hwnd),
        )
    }

    pub(super) fn restore_normal_z_order(window: &WebviewWindow) -> Result<String, String> {
        if window.label() != "main" {
            return Err("only the main window can restore normal Z-order".to_string());
        }

        let main = window.hwnd().map_err(|error| error.to_string())?.0 as HWND;
        ensure_window(main, "main window during normal Z-order restore")?;
        let owner = get_window_long(main, GWLP_HWNDPARENT, "read owner after normal restore")?;
        eprintln!(
            "[desktop] actual owner after normal = {}",
            format_handle_value(owner)
        );

        unsafe {
            SetLastError(0);
            if SetWindowPos(
                main,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            ) == 0
            {
                return Err(last_error("restore normal top-level Z-order", main));
            }
        }

        Ok(format_handle_value(owner))
    }

    pub(super) fn log_dpi_after_always_on_bottom(
        attachment: &DesktopOwnerAttachment,
    ) -> Result<(), String> {
        let main = as_hwnd(attachment.main_hwnd);
        ensure_window(main, "main window after always-on-bottom")?;
        let dpi = unsafe { GetDpiForWindow(main) };
        eprintln!(
            "[desktop] main DPI after always-on-bottom: {dpi} (before owner attach: {})",
            attachment.dpi_before
        );
        if dpi != attachment.dpi_before {
            eprintln!(
                "[desktop] warning: main DPI changed after always-on-bottom: {} -> {dpi}",
                attachment.dpi_before
            );
        }
        Ok(())
    }

    fn rollback_owner(
        attachment: &DesktopOwnerAttachment,
        attach_error: String,
    ) -> Result<DesktopOwnerAttachment, String> {
        let main = as_hwnd(attachment.main_hwnd);
        match set_window_long(
            main,
            GWLP_HWNDPARENT,
            attachment.original_owner,
            "rollback original main window owner",
        ) {
            Ok(_) => {
                eprintln!("[desktop] owner rollback succeeded");
                Err(attach_error)
            }
            Err(rollback_error) => Err(format!(
                "{attach_error}; owner rollback also failed: {rollback_error}"
            )),
        }
    }

    fn find_desktop_host() -> Result<DesktopHost, String> {
        let mut context = EnumContext {
            def_view_class: wide("SHELLDLL_DefView"),
            found: None,
        };

        unsafe {
            SetLastError(0);
            let enumerated = EnumWindows(
                Some(enum_desktop_windows),
                &mut context as *mut EnumContext as LPARAM,
            );
            if let Some(host) = context.found {
                validate_desktop_host(host.host, host.def_view)?;
                return Ok(host);
            }

            let error = GetLastError();
            if enumerated == 0 && error != 0 {
                return Err(win32_error("enumerate desktop windows", null_mut(), error));
            }
        }

        Err("no existing Progman or WorkerW containing SHELLDLL_DefView was found".to_string())
    }

    unsafe extern "system" fn enum_desktop_windows(hwnd: HWND, lparam: LPARAM) -> i32 {
        if lparam == 0 || IsWindow(hwnd) == 0 {
            return 1;
        }

        let context = &mut *(lparam as *mut EnumContext);
        let host_class = match class_name(hwnd) {
            Ok(value) => value,
            Err(_) => return 1,
        };
        if host_class != "Progman" && host_class != "WorkerW" {
            return 1;
        }

        let def_view = FindWindowExW(hwnd, null_mut(), context.def_view_class.as_ptr(), null());
        if def_view.is_null() || IsWindow(def_view) == 0 {
            return 1;
        }
        if GetParent(def_view) != hwnd {
            return 1;
        }
        if class_name(def_view).ok().as_deref() != Some("SHELLDLL_DefView") {
            return 1;
        }

        context.found = Some(DesktopHost {
            host: hwnd,
            def_view,
        });
        0
    }

    fn validate_desktop_host(host: HWND, def_view: HWND) -> Result<(), String> {
        ensure_window(host, "desktop icon host")?;
        ensure_window(def_view, "SHELLDLL_DefView")?;

        let host_class = class_name(host)?;
        if host_class != "Progman" && host_class != "WorkerW" {
            return Err(format!(
                "desktop icon host {} has unexpected class {host_class}",
                format_hwnd(host)
            ));
        }
        if class_name(def_view)? != "SHELLDLL_DefView" {
            return Err(format!(
                "desktop view {} is not SHELLDLL_DefView",
                format_hwnd(def_view)
            ));
        }
        if unsafe { GetParent(def_view) } != host {
            return Err(
                "SHELLDLL_DefView is no longer a direct child of the icon host".to_string(),
            );
        }
        Ok(())
    }

    fn validate_owner_attachment(main: HWND, host: HWND, def_view: HWND) -> Result<(), String> {
        ensure_window(main, "main window after owner attach")?;
        validate_desktop_host(host, def_view)?;

        let owner = get_window_long(main, GWLP_HWNDPARENT, "verify main window owner")?;
        if owner != host as isize {
            return Err(format!(
                "main window owner mismatch: expected {}, found {}",
                format_hwnd(host),
                format_handle_value(owner)
            ));
        }
        Ok(())
    }

    fn ensure_window(hwnd: HWND, context: &str) -> Result<(), String> {
        if hwnd.is_null() || unsafe { IsWindow(hwnd) } == 0 {
            Err(format!(
                "{context} is not a valid HWND ({})",
                format_hwnd(hwnd)
            ))
        } else {
            Ok(())
        }
    }

    fn class_name(hwnd: HWND) -> Result<String, String> {
        let mut buffer = [0u16; 256];
        unsafe {
            SetLastError(0);
            let length = GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
            if length == 0 {
                return Err(last_error("read window class", hwnd));
            }
            Ok(String::from_utf16_lossy(&buffer[..length as usize]))
        }
    }

    fn get_window_long(hwnd: HWND, index: i32, context: &str) -> Result<isize, String> {
        unsafe {
            SetLastError(0);
            let value = GetWindowLongPtrW(hwnd, index);
            let error = GetLastError();
            if value == 0 && error != 0 {
                Err(win32_error(context, hwnd, error))
            } else {
                Ok(value)
            }
        }
    }

    fn set_window_long(
        hwnd: HWND,
        index: i32,
        value: isize,
        context: &str,
    ) -> Result<isize, String> {
        unsafe {
            SetLastError(0);
            let previous = SetWindowLongPtrW(hwnd, index, value);
            let error = GetLastError();
            if previous == 0 && error != 0 {
                Err(win32_error(context, hwnd, error))
            } else {
                Ok(previous)
            }
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn as_hwnd(value: isize) -> HWND {
        value as HWND
    }

    fn format_hwnd(hwnd: HWND) -> String {
        format!("0x{:X}", hwnd as usize)
    }

    fn format_handle_value(value: isize) -> String {
        format_hwnd(as_hwnd(value))
    }

    fn last_error(context: &str, hwnd: HWND) -> String {
        let error = unsafe { GetLastError() };
        win32_error(context, hwnd, error)
    }

    fn win32_error(context: &str, hwnd: HWND, error: u32) -> String {
        let detail = std::io::Error::from_raw_os_error(error as i32);
        format!(
            "{context} failed for HWND {}: Win32 error {error} ({detail})",
            format_hwnd(hwnd)
        )
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use tauri::WebviewWindow;

    use super::DesktopOwnerAttachment;

    pub(super) fn attach_desktop_owner(
        _window: &WebviewWindow,
    ) -> Result<DesktopOwnerAttachment, String> {
        Err("Windows desktop owner mode is only available on Windows".to_string())
    }

    pub(super) fn detach_desktop_owner(_attachment: &DesktopOwnerAttachment) -> Result<(), String> {
        Err("Windows desktop owner mode is only available on Windows".to_string())
    }

    pub(super) fn validate_desktop_owner(
        _attachment: &DesktopOwnerAttachment,
    ) -> Result<(), String> {
        Err("Windows desktop owner mode is only available on Windows".to_string())
    }

    pub(super) fn log_dpi_after_always_on_bottom(
        _attachment: &DesktopOwnerAttachment,
    ) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn restore_normal_z_order(_window: &WebviewWindow) -> Result<String, String> {
        Ok("0x0".to_string())
    }
}
