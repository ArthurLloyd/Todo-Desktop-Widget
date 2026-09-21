#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutostartAction {
    NoChange,
    Enable,
    Disable,
}

pub fn action_for_update(current: bool, requested: bool) -> AutostartAction {
    match (current, requested) {
        (false, true) => AutostartAction::Enable,
        (true, false) => AutostartAction::Disable,
        _ => AutostartAction::NoChange,
    }
}

#[cfg(not(test))]
pub fn is_enabled(app: &tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(to_string)
}

#[cfg(not(test))]
pub fn set_enabled(app: &tauri::AppHandle, requested: bool) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;

    let manager = app.autolaunch();
    let current = manager.is_enabled().map_err(to_string)?;
    match action_for_update(current, requested) {
        AutostartAction::Enable => manager.enable().map_err(to_string)?,
        AutostartAction::Disable => manager.disable().map_err(to_string)?,
        AutostartAction::NoChange => {}
    }
    manager.is_enabled().map_err(to_string)
}

#[cfg(not(test))]
fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_autostart_setting_changes_to_os_actions() {
        assert_eq!(action_for_update(false, true), AutostartAction::Enable);
        assert_eq!(action_for_update(true, false), AutostartAction::Disable);
        assert_eq!(action_for_update(false, false), AutostartAction::NoChange);
        assert_eq!(action_for_update(true, true), AutostartAction::NoChange);
    }
}
