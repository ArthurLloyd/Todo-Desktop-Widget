use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: String,
    pub graph_id: Option<String>,
    pub list_id: String,
    pub title: String,
    pub status: String,
    pub importance: Option<String>,
    pub note: Option<String>,
    pub due_date_time: Option<String>,
    pub reminder_date_time: Option<String>,
    pub completed_date_time: Option<String>,
    pub time_zone: Option<String>,
    pub is_reminder_on: bool,
    pub recurrence: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatchDto {
    pub title: String,
    pub importance: Option<String>,
    pub note: Option<String>,
    pub due_date_time: Option<String>,
    pub reminder_date_time: Option<String>,
    pub time_zone: Option<String>,
    pub is_reminder_on: bool,
    pub recurrence: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListDto {
    pub id: String,
    pub display_name: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusDto {
    pub state: String,
    pub last_synced_at: Option<String>,
    pub message: Option<String>,
}

impl SyncStatusDto {
    pub fn idle() -> Self {
        Self {
            state: "idle".to_string(),
            last_synced_at: None,
            message: None,
        }
    }

    pub fn auth_required(message: impl Into<String>) -> Self {
        Self {
            state: "auth_required".to_string(),
            last_synced_at: None,
            message: Some(message.into()),
        }
    }

    pub fn error(message: impl Into<String>, last_synced_at: Option<String>) -> Self {
        Self {
            state: "error".to_string(),
            last_synced_at,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct SettingsDto {
    pub always_on_top: bool,
    pub opacity: f64,
    pub sync_interval_minutes: i64,
    pub autostart: bool,
    pub debug_mode: bool,
    pub show_completed: bool,
    pub language: String,
    pub font_family: String,
    pub font_scale: f64,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            always_on_top: false,
            opacity: 0.78,
            sync_interval_minutes: 3,
            autostart: false,
            debug_mode: false,
            show_completed: false,
            language: "system".to_string(),
            font_family: "system".to_string(),
            font_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub always_on_top: Option<bool>,
    pub opacity: Option<f64>,
    pub sync_interval_minutes: Option<i64>,
    pub autostart: Option<bool>,
    pub debug_mode: Option<bool>,
    pub show_completed: Option<bool>,
    pub language: Option<String>,
    pub font_family: Option<String>,
    pub font_scale: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WidgetLayoutMode {
    Single,
    Columns,
    Rows,
    Grid,
}

impl WidgetLayoutMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Columns => "columns",
            Self::Rows => "rows",
            Self::Grid => "grid",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WidgetThemeConfig {
    pub version: u32,
    pub background: String,
    pub foreground: String,
    pub secondary_text: String,
    pub accent: String,
    pub task_overdue: String,
    pub task_due_today: String,
    pub task_important: String,
    pub task_pending: String,
    pub task_completed: String,
    pub background_opacity: f64,
    pub blur_px: f64,
    pub border_radius_px: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetInstanceDto {
    /// Stable identity for persistence; 4B may derive `widget-{id}` labels from it.
    /// The existing primary window remains labeled `main`.
    pub id: String,
    pub name: String,
    pub layout_mode: WidgetLayoutMode,
    pub default_theme_id: Option<String>,
    pub is_primary: bool,
    pub is_open: bool,
    pub pinned: bool,
    pub pane_count: i64,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetInstanceInput {
    pub name: String,
    pub layout_mode: WidgetLayoutMode,
    pub default_theme_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetPaneDto {
    pub id: String,
    pub widget_instance_id: String,
    pub list_id: Option<String>,
    pub title_override: Option<String>,
    pub theme_id: Option<String>,
    pub sort_order: i64,
    pub show_completed: Option<bool>,
    pub max_tasks: Option<i64>,
    pub size_weight: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetPaneInput {
    pub widget_instance_id: String,
    pub list_id: Option<String>,
    pub title_override: Option<String>,
    pub theme_id: Option<String>,
    pub sort_order: i64,
    pub show_completed: Option<bool>,
    pub max_tasks: Option<i64>,
    pub size_weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetPaneWeightInput {
    pub pane_id: String,
    pub size_weight: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetContextDto {
    pub instance: WidgetInstanceDto,
    pub panes: Vec<WidgetPaneDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetTaskSnapshotDto {
    pub task_lists: Vec<TaskListDto>,
    pub tasks: Vec<TaskDto>,
    pub inbox_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetConfigChangedDto {
    pub widget_instance_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetThemeDto {
    pub id: String,
    pub name: String,
    pub config: WidgetThemeConfig,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetThemeInput {
    pub name: String,
    pub config: WidgetThemeConfig,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatusDto {
    pub logged_in: bool,
    pub account: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginStartDto {
    pub auth_url: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::WidgetConfigChangedDto;

    #[test]
    fn widget_config_event_targets_one_instance() {
        let payload = WidgetConfigChangedDto {
            widget_instance_id: "widget-a".to_string(),
        };
        let value = serde_json::to_value(payload).expect("serialize event payload");
        assert_eq!(value["widgetInstanceId"], "widget-a");
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphTaskList {
    pub id: String,
    pub display_name: String,
    pub wellknown_list_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphUserProfile {
    pub display_name: Option<String>,
    pub user_principal_name: Option<String>,
    pub mail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphCollection<T> {
    pub value: Vec<T>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphTask {
    pub id: String,
    pub title: String,
    pub status: Option<String>,
    pub importance: Option<String>,
    pub body: Option<GraphItemBody>,
    pub due_date_time: Option<GraphDateTimeTimeZone>,
    pub reminder_date_time: Option<GraphDateTimeTimeZone>,
    pub completed_date_time: Option<GraphDateTimeTimeZone>,
    pub is_reminder_on: Option<bool>,
    pub recurrence: Option<serde_json::Value>,
    pub created_date_time: Option<String>,
    pub last_modified_date_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphItemBody {
    pub content: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphDateTimeTimeZone {
    pub date_time: Option<String>,
    pub time_zone: Option<String>,
}
