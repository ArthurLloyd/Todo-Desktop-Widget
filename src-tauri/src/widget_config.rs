use anyhow::{anyhow, bail, Context};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::models::{
    WidgetInstanceDto, WidgetInstanceInput, WidgetLayoutMode, WidgetPaneDto, WidgetPaneInput,
    WidgetThemeConfig, WidgetThemeDto, WidgetThemeInput,
};

pub const DEFAULT_THEME_ID: &str = "default";
pub const OCEAN_THEME_ID: &str = "ocean";
pub const VIOLET_THEME_ID: &str = "violet";
pub const FOREST_THEME_ID: &str = "forest";
pub const DEFAULT_WIDGET_ID: &str = "main";
pub const DEFAULT_PANE_ID: &str = "main-pane";

pub fn init_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS widget_themes (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          config_json TEXT NOT NULL,
          is_builtin INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS widget_instances (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          layout_mode TEXT NOT NULL CHECK (layout_mode IN ('single', 'columns', 'rows', 'grid')),
          default_theme_id TEXT NULL REFERENCES widget_themes(id) ON DELETE SET NULL,
          is_primary INTEGER NOT NULL DEFAULT 0,
          is_open INTEGER NOT NULL DEFAULT 0,
          pinned INTEGER NOT NULL DEFAULT 0,
          window_x INTEGER NULL,
          window_y INTEGER NULL,
          window_width INTEGER NULL,
          window_height INTEGER NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS widget_panes (
          id TEXT PRIMARY KEY,
          widget_instance_id TEXT NOT NULL REFERENCES widget_instances(id) ON DELETE CASCADE,
          list_id TEXT NULL,
          title_override TEXT NULL,
          theme_id TEXT NULL REFERENCES widget_themes(id) ON DELETE SET NULL,
          sort_order INTEGER NOT NULL CHECK (sort_order >= 0),
          show_completed INTEGER NULL,
          max_tasks INTEGER NULL CHECK (max_tasks IS NULL OR max_tasks > 0),
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_widget_panes_instance_order
          ON widget_panes(widget_instance_id, sort_order, id);
        "#,
    )
    .context("failed to initialize widget configuration schema")?;
    for (name, definition) in [
        ("is_primary", "INTEGER NOT NULL DEFAULT 0"),
        ("is_open", "INTEGER NOT NULL DEFAULT 0"),
        ("pinned", "INTEGER NOT NULL DEFAULT 0"),
        ("window_x", "INTEGER NULL"),
        ("window_y", "INTEGER NULL"),
        ("window_width", "INTEGER NULL"),
        ("window_height", "INTEGER NULL"),
    ] {
        ensure_widget_instance_column(conn, name, definition)?;
    }
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_widget_instances_one_primary
         ON widget_instances(is_primary) WHERE is_primary = 1",
        [],
    )?;
    Ok(())
}

pub fn bootstrap_defaults(conn: &Connection) -> anyhow::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    let now = Utc::now().to_rfc3339();

    let mut created_themes = Vec::new();
    for (id, name, config) in builtin_theme_definitions() {
        if transaction.execute(
            "INSERT OR IGNORE INTO widget_themes
             (id, name, config_json, is_builtin, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1, ?4, ?4)",
            params![id, name, serde_json::to_string(&config)?, now],
        )? > 0
        {
            created_themes.push(name);
        }
    }

    let instance_count: i64 =
        transaction.query_row("SELECT COUNT(*) FROM widget_instances", [], |row| {
            row.get(0)
        })?;
    let widget_created = if instance_count == 0 {
        transaction.execute(
            "INSERT INTO widget_instances
             (id, name, layout_mode, default_theme_id, is_primary, is_open, pinned,
              created_at, updated_at)
             VALUES (?1, 'Main', 'single', ?2, 1, 1, 0, ?3, ?3)",
            params![DEFAULT_WIDGET_ID, DEFAULT_THEME_ID, now],
        )? > 0
    } else {
        false
    };

    let main_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM widget_instances WHERE id = ?1)",
        params![DEFAULT_WIDGET_ID],
        |row| row.get(0),
    )?;
    let pane_created = if main_exists {
        let pane_count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM widget_panes WHERE widget_instance_id = ?1",
            params![DEFAULT_WIDGET_ID],
            |row| row.get(0),
        )?;
        if pane_count == 0 {
            transaction.execute(
                "INSERT INTO widget_panes
                 (id, widget_instance_id, list_id, title_override, theme_id, sort_order,
                  show_completed, max_tasks, created_at, updated_at)
                 VALUES (?1, ?2, NULL, NULL, NULL, 0, NULL, NULL, ?3, ?3)",
                params![DEFAULT_PANE_ID, DEFAULT_WIDGET_ID, now],
            )? > 0
        } else {
            false
        }
    } else {
        false
    };

    ensure_primary_and_migrate_legacy_pinned(&transaction)?;
    transaction.commit()?;

    for theme_name in created_themes {
        println!("[widget-config] initialized built-in theme: {theme_name}");
    }
    if widget_created {
        println!("[widget-config] initialized default widget");
    }
    if pane_created {
        println!("[widget-config] initialized default pane");
    }
    Ok(())
}

pub fn default_theme_config() -> WidgetThemeConfig {
    WidgetThemeConfig {
        version: 1,
        background: "#F5F7FA".to_string(),
        foreground: "#1F2937".to_string(),
        secondary_text: "#6B7280".to_string(),
        accent: "#2563EB".to_string(),
        task_overdue: "#DC2626".to_string(),
        task_due_today: "#2563EB".to_string(),
        task_important: "#D97706".to_string(),
        task_pending: "#6B7280".to_string(),
        task_completed: "#64748B".to_string(),
        background_opacity: 0.78,
        blur_px: 24.0,
        border_radius_px: 18.0,
    }
}

fn builtin_theme_definitions() -> [(&'static str, &'static str, WidgetThemeConfig); 4] {
    [
        (DEFAULT_THEME_ID, "Default", default_theme_config()),
        (
            OCEAN_THEME_ID,
            "Ocean",
            WidgetThemeConfig {
                version: 1,
                background: "#102A3A".to_string(),
                foreground: "#EAF8FF".to_string(),
                secondary_text: "#A5C4D4".to_string(),
                accent: "#38BDF8".to_string(),
                task_overdue: "#FB7185".to_string(),
                task_due_today: "#38BDF8".to_string(),
                task_important: "#FBBF24".to_string(),
                task_pending: "#94A3B8".to_string(),
                task_completed: "#7FA7B8".to_string(),
                background_opacity: 0.88,
                blur_px: 20.0,
                border_radius_px: 16.0,
            },
        ),
        (
            VIOLET_THEME_ID,
            "Violet",
            WidgetThemeConfig {
                version: 1,
                background: "#281E38".to_string(),
                foreground: "#F7F2FF".to_string(),
                secondary_text: "#C0B3D4".to_string(),
                accent: "#A78BFA".to_string(),
                task_overdue: "#FB7185".to_string(),
                task_due_today: "#C4B5FD".to_string(),
                task_important: "#FBBF24".to_string(),
                task_pending: "#A9A0B8".to_string(),
                task_completed: "#A495B8".to_string(),
                background_opacity: 0.88,
                blur_px: 8.0,
                border_radius_px: 32.0,
            },
        ),
        (
            FOREST_THEME_ID,
            "Forest",
            WidgetThemeConfig {
                version: 1,
                background: "#153028".to_string(),
                foreground: "#ECFDF5".to_string(),
                secondary_text: "#A7CDBE".to_string(),
                accent: "#34D399".to_string(),
                task_overdue: "#FB7185".to_string(),
                task_due_today: "#5EEAD4".to_string(),
                task_important: "#FBBF24".to_string(),
                task_pending: "#94B8AA".to_string(),
                task_completed: "#86AA9C".to_string(),
                background_opacity: 0.88,
                blur_px: 0.0,
                border_radius_px: 8.0,
            },
        ),
    ]
}

pub fn validate_theme_config(config: &WidgetThemeConfig) -> anyhow::Result<()> {
    if config.version != 1 {
        bail!("unsupported widget theme config version {}", config.version);
    }

    for (field, value) in [
        ("background", config.background.as_str()),
        ("foreground", config.foreground.as_str()),
        ("secondaryText", config.secondary_text.as_str()),
        ("accent", config.accent.as_str()),
        ("taskOverdue", config.task_overdue.as_str()),
        ("taskDueToday", config.task_due_today.as_str()),
        ("taskImportant", config.task_important.as_str()),
        ("taskPending", config.task_pending.as_str()),
        ("taskCompleted", config.task_completed.as_str()),
    ] {
        if !is_hex_color(value) {
            bail!("invalid {field} color; expected #RRGGBB or #RRGGBBAA");
        }
    }

    validate_range("backgroundOpacity", config.background_opacity, 0.0, 1.0)?;
    validate_range("blurPx", config.blur_px, 0.0, 64.0)?;
    validate_range("borderRadiusPx", config.border_radius_px, 0.0, 64.0)?;
    Ok(())
}

pub fn parse_layout_mode(value: &str) -> anyhow::Result<WidgetLayoutMode> {
    match value {
        "single" => Ok(WidgetLayoutMode::Single),
        "columns" => Ok(WidgetLayoutMode::Columns),
        "rows" => Ok(WidgetLayoutMode::Rows),
        "grid" => Ok(WidgetLayoutMode::Grid),
        _ => bail!("invalid widget layout mode: {value}"),
    }
}

pub fn list_widget_instances(conn: &Connection) -> anyhow::Result<Vec<WidgetInstanceDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, layout_mode, default_theme_id, is_primary, is_open, pinned,
                window_x, window_y, window_width, window_height, created_at, updated_at
         FROM widget_instances ORDER BY is_primary DESC, created_at, id",
    )?;
    let mut rows = stmt.query([])?;
    let mut instances = Vec::new();
    while let Some(row) = rows.next()? {
        instances.push(widget_instance_from_row(row)?);
    }
    Ok(instances)
}

pub fn get_widget_instance(conn: &Connection, id: &str) -> anyhow::Result<WidgetInstanceDto> {
    let mut stmt = conn.prepare(
        "SELECT id, name, layout_mode, default_theme_id, is_primary, is_open, pinned,
                window_x, window_y, window_width, window_height, created_at, updated_at
         FROM widget_instances WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    let row = rows.next()?.context("widget instance not found")?;
    widget_instance_from_row(row)
}

pub fn get_primary_widget_instance(conn: &Connection) -> anyhow::Result<WidgetInstanceDto> {
    let id: String = conn
        .query_row(
            "SELECT id FROM widget_instances WHERE is_primary = 1 LIMIT 1",
            [],
            |row| row.get(0),
        )
        .context("primary widget instance not found")?;
    get_widget_instance(conn, &id)
}

pub fn list_open_secondary_widgets(conn: &Connection) -> anyhow::Result<Vec<WidgetInstanceDto>> {
    Ok(list_widget_instances(conn)?
        .into_iter()
        .filter(|instance| !instance.is_primary && instance.is_open)
        .collect())
}

pub fn create_widget_with_default_pane(
    conn: &Connection,
    initial_list_id: Option<&str>,
) -> anyhow::Result<WidgetInstanceDto> {
    let transaction = conn.unchecked_transaction()?;
    let list_id = normalize_optional_text(initial_list_id);
    let list_name = if let Some(list_id) = list_id.as_deref() {
        transaction
            .query_row(
                "SELECT display_name FROM task_lists WHERE id = ?1",
                params![list_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .context("initial task list not found")?
            .into()
    } else {
        None
    };
    let base_name = list_name.unwrap_or_else(|| "Widget".to_string());
    let name = next_available_widget_name(&transaction, &base_name)?;
    let id = Uuid::new_v4().to_string();
    let pane_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    transaction.execute(
        "INSERT INTO widget_instances
         (id, name, layout_mode, default_theme_id, is_primary, is_open, pinned,
          created_at, updated_at)
         VALUES (?1, ?2, 'single', ?3, 0, 1, 0, ?4, ?4)",
        params![id, name, DEFAULT_THEME_ID, now],
    )?;
    transaction.execute(
        "INSERT INTO widget_panes
         (id, widget_instance_id, list_id, title_override, theme_id, sort_order,
          show_completed, max_tasks, created_at, updated_at)
         VALUES (?1, ?2, ?3, NULL, NULL, 0, NULL, NULL, ?4, ?4)",
        params![pane_id, id, list_id, now],
    )?;
    transaction.commit()?;
    get_widget_instance(conn, &id)
}

pub fn set_widget_open(conn: &Connection, id: &str, is_open: bool) -> anyhow::Result<()> {
    let instance = get_widget_instance(conn, id)?;
    let is_open = instance.is_primary || is_open;
    conn.execute(
        "UPDATE widget_instances SET is_open = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, i64::from(is_open), Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn set_widget_pinned(conn: &Connection, id: &str, pinned: bool) -> anyhow::Result<()> {
    if conn.execute(
        "UPDATE widget_instances SET pinned = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, i64::from(pinned), Utc::now().to_rfc3339()],
    )? == 0
    {
        bail!("widget instance not found");
    }
    Ok(())
}

pub fn save_widget_geometry(
    conn: &Connection,
    id: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> anyhow::Result<()> {
    if width < 1 || height < 1 {
        bail!("widget window size must be positive");
    }
    if conn.execute(
        "UPDATE widget_instances
         SET window_x = ?2, window_y = ?3, window_width = ?4, window_height = ?5,
             updated_at = ?6
         WHERE id = ?1",
        params![
            id,
            x,
            y,
            i64::from(width),
            i64::from(height),
            Utc::now().to_rfc3339()
        ],
    )? == 0
    {
        bail!("widget instance not found");
    }
    Ok(())
}

pub fn update_first_pane_list(
    conn: &Connection,
    widget_instance_id: &str,
    list_id: &str,
) -> anyhow::Result<WidgetPaneDto> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM task_lists WHERE id = ?1)",
        params![list_id],
        |row| row.get(0),
    )?;
    if !exists {
        bail!("task list not found");
    }
    let pane_id: String = conn
        .query_row(
            "SELECT id FROM widget_panes WHERE widget_instance_id = ?1
             ORDER BY sort_order, id LIMIT 1",
            params![widget_instance_id],
            |row| row.get(0),
        )
        .context("widget pane not found")?;
    conn.execute(
        "UPDATE widget_panes SET list_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![pane_id, list_id, Utc::now().to_rfc3339()],
    )?;
    get_widget_pane(conn, &pane_id)
}

pub fn create_pane_for_widget(
    conn: &Connection,
    widget_instance_id: &str,
    list_id: Option<&str>,
) -> anyhow::Result<WidgetPaneDto> {
    ensure_widget_exists(conn, widget_instance_id)?;
    if let Some(list_id) = list_id {
        ensure_task_list_exists(conn, list_id)?;
    }
    let next_sort_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1
         FROM widget_panes WHERE widget_instance_id = ?1",
        params![widget_instance_id],
        |row| row.get(0),
    )?;
    create_widget_pane(
        conn,
        &WidgetPaneInput {
            widget_instance_id: widget_instance_id.to_string(),
            list_id: list_id.map(str::to_string),
            title_override: None,
            theme_id: None,
            sort_order: next_sort_order,
            show_completed: None,
            max_tasks: None,
        },
    )
}

pub fn set_widget_pane_list_for_widget(
    conn: &Connection,
    widget_instance_id: &str,
    pane_id: &str,
    list_id: Option<&str>,
) -> anyhow::Result<WidgetPaneDto> {
    ensure_pane_belongs_to_widget(conn, pane_id, widget_instance_id)?;
    if let Some(list_id) = list_id {
        ensure_task_list_exists(conn, list_id)?;
    }
    conn.execute(
        "UPDATE widget_panes SET list_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![
            pane_id,
            normalize_optional_text(list_id),
            Utc::now().to_rfc3339()
        ],
    )?;
    get_widget_pane(conn, pane_id)
}

pub fn delete_widget_pane_for_widget(
    conn: &Connection,
    widget_instance_id: &str,
    pane_id: &str,
) -> anyhow::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    ensure_pane_belongs_to_widget(&transaction, pane_id, widget_instance_id)?;
    let pane_count: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM widget_panes WHERE widget_instance_id = ?1",
        params![widget_instance_id],
        |row| row.get(0),
    )?;
    if pane_count <= 1 {
        bail!("widget must contain at least one pane");
    }
    delete_widget_pane(&transaction, pane_id)?;
    transaction.commit()?;
    Ok(())
}

pub fn set_widget_layout_mode(
    conn: &Connection,
    widget_instance_id: &str,
    layout_mode: WidgetLayoutMode,
) -> anyhow::Result<WidgetInstanceDto> {
    if conn.execute(
        "UPDATE widget_instances SET layout_mode = ?2, updated_at = ?3 WHERE id = ?1",
        params![
            widget_instance_id,
            layout_mode.as_str(),
            Utc::now().to_rfc3339()
        ],
    )? == 0
    {
        bail!("widget instance not found");
    }
    get_widget_instance(conn, widget_instance_id)
}

pub fn set_widget_name(
    conn: &Connection,
    widget_instance_id: &str,
    name: &str,
) -> anyhow::Result<WidgetInstanceDto> {
    let name = validate_name(name, "widget name")?;
    if conn.execute(
        "UPDATE widget_instances SET name = ?2, updated_at = ?3 WHERE id = ?1",
        params![widget_instance_id, name, Utc::now().to_rfc3339()],
    )? == 0
    {
        bail!("widget instance not found");
    }
    get_widget_instance(conn, widget_instance_id)
}

pub fn set_widget_default_theme(
    conn: &Connection,
    widget_instance_id: &str,
    theme_id: Option<&str>,
) -> anyhow::Result<WidgetInstanceDto> {
    ensure_theme_exists(conn, theme_id)?;
    if conn.execute(
        "UPDATE widget_instances SET default_theme_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![widget_instance_id, theme_id, Utc::now().to_rfc3339()],
    )? == 0
    {
        bail!("widget instance not found");
    }
    get_widget_instance(conn, widget_instance_id)
}

pub fn set_widget_pane_theme_for_widget(
    conn: &Connection,
    widget_instance_id: &str,
    pane_id: &str,
    theme_id: Option<&str>,
) -> anyhow::Result<WidgetPaneDto> {
    ensure_pane_belongs_to_widget(conn, pane_id, widget_instance_id)?;
    ensure_theme_exists(conn, theme_id)?;
    conn.execute(
        "UPDATE widget_panes SET theme_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![pane_id, theme_id, Utc::now().to_rfc3339()],
    )?;
    get_widget_pane(conn, pane_id)
}

#[cfg(test)]
fn effective_show_completed(pane_value: Option<bool>, global_value: bool) -> bool {
    pane_value.unwrap_or(global_value)
}

pub fn create_widget_instance(
    conn: &Connection,
    input: &WidgetInstanceInput,
) -> anyhow::Result<WidgetInstanceDto> {
    let name = validate_name(&input.name, "widget name")?;
    ensure_theme_exists(conn, input.default_theme_id.as_deref())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO widget_instances
         (id, name, layout_mode, default_theme_id, is_primary, is_open, pinned,
          created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, 0, 0, ?5, ?5)",
        params![
            id,
            name,
            input.layout_mode.as_str(),
            input.default_theme_id,
            now
        ],
    )?;
    get_widget_instance(conn, &id)
}

pub fn update_widget_instance(
    conn: &Connection,
    id: &str,
    input: &WidgetInstanceInput,
) -> anyhow::Result<WidgetInstanceDto> {
    let name = validate_name(&input.name, "widget name")?;
    ensure_theme_exists(conn, input.default_theme_id.as_deref())?;
    let changed = conn.execute(
        "UPDATE widget_instances
         SET name = ?2, layout_mode = ?3, default_theme_id = ?4, updated_at = ?5
         WHERE id = ?1",
        params![
            id,
            name,
            input.layout_mode.as_str(),
            input.default_theme_id,
            Utc::now().to_rfc3339()
        ],
    )?;
    if changed == 0 {
        bail!("widget instance not found");
    }
    get_widget_instance(conn, id)
}

pub fn delete_widget_instance(conn: &Connection, id: &str) -> anyhow::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    let is_primary: bool = transaction
        .query_row(
            "SELECT is_primary != 0 FROM widget_instances WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?
        .context("widget instance not found")?;
    if is_primary {
        bail!("the primary widget instance cannot be deleted");
    }
    transaction.execute(
        "DELETE FROM widget_panes WHERE widget_instance_id = ?1",
        params![id],
    )?;
    let changed = transaction.execute("DELETE FROM widget_instances WHERE id = ?1", params![id])?;
    if changed == 0 {
        bail!("widget instance not found");
    }
    transaction.commit()?;
    Ok(())
}

pub fn list_widget_panes(
    conn: &Connection,
    widget_instance_id: &str,
) -> anyhow::Result<Vec<WidgetPaneDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, widget_instance_id, list_id, title_override, theme_id, sort_order,
                show_completed, max_tasks, created_at, updated_at
         FROM widget_panes WHERE widget_instance_id = ?1 ORDER BY sort_order, id",
    )?;
    let mut rows = stmt.query(params![widget_instance_id])?;
    let mut panes = Vec::new();
    while let Some(row) = rows.next()? {
        panes.push(widget_pane_from_row(row)?);
    }
    Ok(panes)
}

pub fn create_widget_pane(
    conn: &Connection,
    input: &WidgetPaneInput,
) -> anyhow::Result<WidgetPaneDto> {
    validate_pane_input(conn, input)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO widget_panes
         (id, widget_instance_id, list_id, title_override, theme_id, sort_order,
          show_completed, max_tasks, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            id,
            input.widget_instance_id,
            normalize_optional_text(input.list_id.as_deref()),
            normalize_optional_text(input.title_override.as_deref()),
            input.theme_id,
            input.sort_order,
            input.show_completed.map(i64::from),
            input.max_tasks,
            now
        ],
    )?;
    get_widget_pane(conn, &id)
}

pub fn update_widget_pane(
    conn: &Connection,
    id: &str,
    input: &WidgetPaneInput,
) -> anyhow::Result<WidgetPaneDto> {
    validate_pane_input(conn, input)?;
    let changed = conn.execute(
        "UPDATE widget_panes
         SET widget_instance_id = ?2, list_id = ?3, title_override = ?4, theme_id = ?5,
             sort_order = ?6, show_completed = ?7, max_tasks = ?8, updated_at = ?9
         WHERE id = ?1",
        params![
            id,
            input.widget_instance_id,
            normalize_optional_text(input.list_id.as_deref()),
            normalize_optional_text(input.title_override.as_deref()),
            input.theme_id,
            input.sort_order,
            input.show_completed.map(i64::from),
            input.max_tasks,
            Utc::now().to_rfc3339()
        ],
    )?;
    if changed == 0 {
        bail!("widget pane not found");
    }
    get_widget_pane(conn, id)
}

pub fn update_widget_pane_for_widget(
    conn: &Connection,
    widget_instance_id: &str,
    pane_id: &str,
    input: &WidgetPaneInput,
) -> anyhow::Result<WidgetPaneDto> {
    ensure_pane_belongs_to_widget(conn, pane_id, widget_instance_id)?;
    if input.widget_instance_id != widget_instance_id {
        bail!("widget pane does not belong to the current widget");
    }
    update_widget_pane(conn, pane_id, input)
}

pub fn delete_widget_pane(conn: &Connection, id: &str) -> anyhow::Result<()> {
    if conn.execute("DELETE FROM widget_panes WHERE id = ?1", params![id])? == 0 {
        bail!("widget pane not found");
    }
    Ok(())
}

pub fn list_widget_themes(conn: &Connection) -> anyhow::Result<Vec<WidgetThemeDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, config_json, is_builtin, created_at, updated_at
         FROM widget_themes ORDER BY is_builtin DESC, created_at, id",
    )?;
    let mut rows = stmt.query([])?;
    let mut themes = Vec::new();
    while let Some(row) = rows.next()? {
        themes.push(widget_theme_from_row(row)?);
    }
    Ok(themes)
}

pub fn get_widget_theme(conn: &Connection, id: &str) -> anyhow::Result<WidgetThemeDto> {
    get_widget_theme_optional(conn, id)?.context("widget theme not found")
}

pub fn create_widget_theme(
    conn: &Connection,
    input: &WidgetThemeInput,
) -> anyhow::Result<WidgetThemeDto> {
    let name = validate_theme_name(&input.name)?;
    validate_theme_config(&input.config)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO widget_themes
         (id, name, config_json, is_builtin, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, ?4, ?4)",
        params![id, name, serde_json::to_string(&input.config)?, now],
    )?;
    get_widget_theme(conn, &id)
}

pub fn update_widget_theme(
    conn: &Connection,
    id: &str,
    input: &WidgetThemeInput,
) -> anyhow::Result<WidgetThemeDto> {
    let is_builtin: Option<bool> = conn
        .query_row(
            "SELECT is_builtin != 0 FROM widget_themes WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    match is_builtin {
        None => bail!("widget theme not found"),
        Some(true) => bail!("built-in widget themes cannot be updated"),
        Some(false) => {}
    }

    let name = validate_theme_name(&input.name)?;
    validate_theme_config(&input.config)?;
    let changed = conn.execute(
        "UPDATE widget_themes SET name = ?2, config_json = ?3, updated_at = ?4 WHERE id = ?1",
        params![
            id,
            name,
            serde_json::to_string(&input.config)?,
            Utc::now().to_rfc3339()
        ],
    )?;
    if changed == 0 {
        bail!("widget theme not found");
    }
    get_widget_theme(conn, id)
}

pub fn delete_widget_theme(conn: &Connection, id: &str) -> anyhow::Result<()> {
    let is_builtin: Option<bool> = conn
        .query_row(
            "SELECT is_builtin != 0 FROM widget_themes WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    match is_builtin {
        None => bail!("widget theme not found"),
        Some(true) => bail!("built-in widget themes cannot be deleted"),
        Some(false) => {}
    }

    let transaction = conn.unchecked_transaction()?;
    transaction.execute(
        "UPDATE widget_panes SET theme_id = NULL, updated_at = ?2 WHERE theme_id = ?1",
        params![id, Utc::now().to_rfc3339()],
    )?;
    transaction.execute(
        "UPDATE widget_instances SET default_theme_id = NULL, updated_at = ?2
         WHERE default_theme_id = ?1",
        params![id, Utc::now().to_rfc3339()],
    )?;
    transaction.execute("DELETE FROM widget_themes WHERE id = ?1", params![id])?;
    transaction.commit()?;
    Ok(())
}

pub fn resolve_widget_pane_theme(
    conn: &Connection,
    pane_id: &str,
) -> anyhow::Result<WidgetThemeDto> {
    let (widget_id, pane_theme_id): (String, Option<String>) = conn
        .query_row(
            "SELECT widget_instance_id, theme_id FROM widget_panes WHERE id = ?1",
            params![pane_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .context("widget pane not found")?;

    if let Some(theme_id) = pane_theme_id {
        if let Some(theme) = get_widget_theme_optional(conn, &theme_id)? {
            return Ok(theme);
        }
    }

    let widget = get_widget_instance(conn, &widget_id)?;
    if let Some(theme_id) = widget.default_theme_id {
        if let Some(theme) = get_widget_theme_optional(conn, &theme_id)? {
            return Ok(theme);
        }
    }

    get_widget_theme(conn, DEFAULT_THEME_ID)
}

fn get_widget_pane(conn: &Connection, id: &str) -> anyhow::Result<WidgetPaneDto> {
    let mut stmt = conn.prepare(
        "SELECT id, widget_instance_id, list_id, title_override, theme_id, sort_order,
                show_completed, max_tasks, created_at, updated_at
         FROM widget_panes WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    let row = rows.next()?.context("widget pane not found")?;
    widget_pane_from_row(row)
}

fn next_available_widget_name(conn: &Connection, base: &str) -> anyhow::Result<String> {
    let exists = |candidate: &str| -> anyhow::Result<bool> {
        Ok(conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM widget_instances WHERE name = ?1)",
            params![candidate],
            |row| row.get(0),
        )?)
    };
    if !exists(base)? {
        return Ok(base.to_string());
    }
    for suffix in 2..=10_000 {
        let candidate = format!("{base} {suffix}");
        if !exists(&candidate)? {
            return Ok(candidate);
        }
    }
    bail!("unable to allocate a unique widget name")
}

fn ensure_widget_instance_column(
    conn: &Connection,
    name: &str,
    definition: &str,
) -> anyhow::Result<()> {
    let exists = conn
        .prepare("PRAGMA table_info(widget_instances)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|column| column == name);
    if !exists {
        conn.execute(
            &format!("ALTER TABLE widget_instances ADD COLUMN {name} {definition}"),
            [],
        )?;
    }
    Ok(())
}

fn ensure_primary_and_migrate_legacy_pinned(conn: &Connection) -> anyhow::Result<()> {
    let primary_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM widget_instances WHERE is_primary = 1",
        [],
        |row| row.get(0),
    )?;
    if primary_count == 0 {
        let primary_id: String = conn
            .query_row(
                "SELECT id FROM widget_instances
                 ORDER BY CASE WHEN id = ?1 THEN 0 ELSE 1 END, created_at, id LIMIT 1",
                params![DEFAULT_WIDGET_ID],
                |row| row.get(0),
            )
            .context("cannot select a primary widget from an empty configuration")?;
        conn.execute(
            "UPDATE widget_instances SET is_primary = 1, is_open = 1 WHERE id = ?1",
            params![primary_id],
        )?;
    }
    conn.execute(
        "UPDATE widget_instances SET is_open = 1 WHERE is_primary = 1",
        [],
    )?;

    let migrated: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM settings WHERE key = 'widget_window_state_v1_migrated')",
        [],
        |row| row.get(0),
    )?;
    if !migrated {
        let legacy_pinned = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'settings'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|value| value.get("alwaysOnTop").and_then(|item| item.as_bool()))
            .unwrap_or(false);
        conn.execute(
            "UPDATE widget_instances SET pinned = ?1 WHERE is_primary = 1",
            params![i64::from(legacy_pinned)],
        )?;
        conn.execute(
            "INSERT INTO settings(key, value) VALUES ('widget_window_state_v1_migrated', '1')",
            [],
        )?;
    }
    Ok(())
}

fn get_widget_theme_optional(
    conn: &Connection,
    id: &str,
) -> anyhow::Result<Option<WidgetThemeDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, config_json, is_builtin, created_at, updated_at
         FROM widget_themes WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    rows.next()?.map(widget_theme_from_row).transpose()
}

fn widget_instance_from_row(row: &Row<'_>) -> anyhow::Result<WidgetInstanceDto> {
    let layout_mode: String = row.get(2)?;
    Ok(WidgetInstanceDto {
        id: row.get(0)?,
        name: row.get(1)?,
        layout_mode: parse_layout_mode(&layout_mode)?,
        default_theme_id: row.get(3)?,
        is_primary: row.get::<_, i64>(4)? != 0,
        is_open: row.get::<_, i64>(5)? != 0,
        pinned: row.get::<_, i64>(6)? != 0,
        window_x: row.get(7)?,
        window_y: row.get(8)?,
        window_width: row
            .get::<_, Option<i64>>(9)?
            .and_then(|value| u32::try_from(value).ok()),
        window_height: row
            .get::<_, Option<i64>>(10)?
            .and_then(|value| u32::try_from(value).ok()),
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn widget_pane_from_row(row: &Row<'_>) -> anyhow::Result<WidgetPaneDto> {
    Ok(WidgetPaneDto {
        id: row.get(0)?,
        widget_instance_id: row.get(1)?,
        list_id: row.get(2)?,
        title_override: row.get(3)?,
        theme_id: row.get(4)?,
        sort_order: row.get(5)?,
        show_completed: row.get::<_, Option<i64>>(6)?.map(|value| value != 0),
        max_tasks: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn widget_theme_from_row(row: &Row<'_>) -> anyhow::Result<WidgetThemeDto> {
    let raw_config: String = row.get(2)?;
    let config: WidgetThemeConfig =
        serde_json::from_str(&raw_config).context("invalid widget theme config JSON")?;
    validate_theme_config(&config)?;
    Ok(WidgetThemeDto {
        id: row.get(0)?,
        name: row.get(1)?,
        config,
        is_builtin: row.get::<_, i64>(3)? != 0,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn validate_pane_input(conn: &Connection, input: &WidgetPaneInput) -> anyhow::Result<()> {
    ensure_widget_exists(conn, &input.widget_instance_id)?;
    ensure_theme_exists(conn, input.theme_id.as_deref())?;
    if input.sort_order < 0 {
        bail!("pane sort order must be non-negative");
    }
    if let Some(max_tasks) = input.max_tasks {
        if !(1..=500).contains(&max_tasks) {
            bail!("pane maxTasks must be between 1 and 500");
        }
    }
    if let Some(title) = input.title_override.as_deref() {
        if title.trim().len() > 120 {
            bail!("pane title override must be at most 120 characters");
        }
    }
    Ok(())
}

fn ensure_widget_exists(conn: &Connection, id: &str) -> anyhow::Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM widget_instances WHERE id = ?1)",
        params![id],
        |row| row.get(0),
    )?;
    if !exists {
        bail!("widget instance not found");
    }
    Ok(())
}

fn ensure_pane_belongs_to_widget(
    conn: &Connection,
    pane_id: &str,
    widget_instance_id: &str,
) -> anyhow::Result<()> {
    let owner: Option<String> = conn
        .query_row(
            "SELECT widget_instance_id FROM widget_panes WHERE id = ?1",
            params![pane_id],
            |row| row.get(0),
        )
        .optional()?;
    match owner {
        Some(owner) if owner == widget_instance_id => Ok(()),
        Some(_) => bail!("widget pane does not belong to the current widget"),
        None => bail!("widget pane not found"),
    }
}

fn ensure_task_list_exists(conn: &Connection, list_id: &str) -> anyhow::Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM task_lists WHERE id = ?1)",
        params![list_id],
        |row| row.get(0),
    )?;
    if !exists {
        bail!("task list not found");
    }
    Ok(())
}

fn ensure_theme_exists(conn: &Connection, id: Option<&str>) -> anyhow::Result<()> {
    let Some(id) = id else {
        return Ok(());
    };
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM widget_themes WHERE id = ?1)",
        params![id],
        |row| row.get(0),
    )?;
    if !exists {
        bail!("widget theme not found");
    }
    Ok(())
}

fn validate_name(value: &str, field: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{field} cannot be empty");
    }
    if value.len() > 120 {
        bail!("{field} must be at most 120 characters");
    }
    Ok(value.to_string())
}

fn validate_theme_name(value: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("theme name cannot be empty");
    }
    if value.chars().count() > 64 {
        bail!("theme name must be at most 64 characters");
    }
    Ok(value.to_string())
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value.and_then(|text| {
        let text = text.trim();
        (!text.is_empty()).then(|| text.to_string())
    })
}

fn is_hex_color(value: &str) -> bool {
    matches!(value.len(), 7 | 9)
        && value.starts_with('#')
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_range(field: &str, value: f64, min: f64, max: f64) -> anyhow::Result<()> {
    if !value.is_finite() || !(min..=max).contains(&value) {
        return Err(anyhow!("{field} must be between {min} and {max}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open test database");
        db::init_db(&conn).expect("initialize database");
        db::ensure_local_core_lists(&conn).expect("initialize local lists");
        conn
    }

    fn widget_input(name: &str, theme_id: Option<&str>) -> WidgetInstanceInput {
        WidgetInstanceInput {
            name: name.to_string(),
            layout_mode: WidgetLayoutMode::Single,
            default_theme_id: theme_id.map(str::to_string),
        }
    }

    fn pane_input(widget_id: &str, list_id: Option<&str>, order: i64) -> WidgetPaneInput {
        WidgetPaneInput {
            widget_instance_id: widget_id.to_string(),
            list_id: list_id.map(str::to_string),
            title_override: None,
            theme_id: None,
            sort_order: order,
            show_completed: None,
            max_tasks: None,
        }
    }

    #[test]
    fn fresh_database_bootstraps_default_records() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");

        let themes = list_widget_themes(&conn).expect("list themes");
        assert_eq!(themes.len(), 4);
        assert_eq!(
            themes
                .iter()
                .map(|theme| theme.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                DEFAULT_THEME_ID,
                FOREST_THEME_ID,
                OCEAN_THEME_ID,
                VIOLET_THEME_ID
            ]
        );
        assert!(themes.iter().all(|theme| theme.is_builtin));
        assert_eq!(list_widget_instances(&conn).expect("list widgets").len(), 1);
        assert_eq!(
            list_widget_panes(&conn, DEFAULT_WIDGET_ID)
                .expect("list panes")
                .len(),
            1
        );
    }

    #[test]
    fn repeated_bootstrap_is_idempotent() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("first bootstrap");
        bootstrap_defaults(&conn).expect("second bootstrap");

        assert_eq!(list_widget_themes(&conn).expect("list themes").len(), 4);
        assert_eq!(list_widget_instances(&conn).expect("list widgets").len(), 1);
        assert_eq!(
            list_widget_panes(&conn, DEFAULT_WIDGET_ID)
                .expect("list panes")
                .len(),
            1
        );
    }

    #[test]
    fn creates_second_widget_without_touching_tasks() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let task = db::insert_local_task(&conn, "Keep me", "inbox").expect("insert task");

        create_widget_instance(&conn, &widget_input("Second", Some(DEFAULT_THEME_ID)))
            .expect("create widget");

        assert_eq!(list_widget_instances(&conn).expect("list widgets").len(), 2);
        assert!(db::get_task(&conn, &task.id).is_ok());
    }

    #[test]
    fn lists_three_panes_in_sort_order() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        delete_widget_pane(&conn, DEFAULT_PANE_ID).expect("remove default pane");
        create_widget_pane(&conn, &pane_input(DEFAULT_WIDGET_ID, Some("list-c"), 2))
            .expect("create pane c");
        create_widget_pane(&conn, &pane_input(DEFAULT_WIDGET_ID, Some("list-a"), 0))
            .expect("create pane a");
        create_widget_pane(&conn, &pane_input(DEFAULT_WIDGET_ID, Some("list-b"), 1))
            .expect("create pane b");

        let panes = list_widget_panes(&conn, DEFAULT_WIDGET_ID).expect("list panes");
        assert_eq!(
            panes
                .iter()
                .map(|pane| pane.list_id.as_deref())
                .collect::<Vec<_>>(),
            vec![Some("list-a"), Some("list-b"), Some("list-c")]
        );
    }

    #[test]
    fn same_list_can_be_referenced_by_multiple_widgets() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let second = create_widget_instance(&conn, &widget_input("Second", None))
            .expect("create second widget");

        create_widget_pane(
            &conn,
            &pane_input(DEFAULT_WIDGET_ID, Some("shared-list"), 1),
        )
        .expect("create main pane");
        create_widget_pane(&conn, &pane_input(&second.id, Some("shared-list"), 0))
            .expect("create second pane");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM widget_panes WHERE list_id = 'shared-list'",
                [],
                |row| row.get(0),
            )
            .expect("count shared references");
        assert_eq!(count, 2);
    }

    #[test]
    fn deleting_pane_does_not_delete_tasks_or_lists() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let task = db::insert_local_task(&conn, "Keep me", "inbox").expect("insert task");

        delete_widget_pane(&conn, DEFAULT_PANE_ID).expect("delete pane");

        assert!(db::get_task(&conn, &task.id).is_ok());
        assert!(db::list_id_for_role(&conn, "inbox").is_ok());
    }

    #[test]
    fn deleting_widget_cleans_panes_without_deleting_task_data() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let task = db::insert_local_task(&conn, "Keep me", "inbox").expect("insert task");
        let secondary = create_widget_with_default_pane(&conn, Some("local-inbox"))
            .expect("create secondary widget");

        delete_widget_instance(&conn, &secondary.id).expect("delete widget");

        assert!(list_widget_panes(&conn, &secondary.id)
            .expect("list panes")
            .is_empty());
        assert!(db::get_task(&conn, &task.id).is_ok());
        assert!(db::list_id_for_role(&conn, "inbox").is_ok());
    }

    #[test]
    fn theme_config_json_round_trips() {
        let config = default_theme_config();
        let encoded = serde_json::to_string(&config).expect("encode theme");
        let decoded: WidgetThemeConfig = serde_json::from_str(&encoded).expect("decode theme");
        assert_eq!(decoded, config);
    }

    #[test]
    fn invalid_layout_mode_is_rejected() {
        assert!(parse_layout_mode("colums").is_err());
    }

    #[test]
    fn invalid_theme_values_are_rejected() {
        let mut config = default_theme_config();
        config.task_due_today = "red".to_string();
        assert!(validate_theme_config(&config).is_err());

        let mut config = default_theme_config();
        config.background_opacity = 1.5;
        assert!(validate_theme_config(&config).is_err());
    }

    #[test]
    fn built_in_theme_cannot_be_updated_or_deleted() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let input = WidgetThemeInput {
            name: "Changed".to_string(),
            config: default_theme_config(),
        };

        let update_error = update_widget_theme(&conn, OCEAN_THEME_ID, &input).unwrap_err();
        let delete_error = delete_widget_theme(&conn, OCEAN_THEME_ID).unwrap_err();

        assert!(update_error.to_string().contains("cannot be updated"));
        assert!(delete_error.to_string().contains("cannot be deleted"));
        assert_eq!(
            get_widget_theme(&conn, OCEAN_THEME_ID).unwrap().name,
            "Ocean"
        );
    }

    #[test]
    fn custom_theme_can_be_created_and_updated() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let created = create_widget_theme(
            &conn,
            &WidgetThemeInput {
                name: "Study".to_string(),
                config: default_theme_config(),
            },
        )
        .expect("create custom theme");
        let mut changed_config = created.config.clone();
        changed_config.accent = "#A78BFA".to_string();

        let updated = update_widget_theme(
            &conn,
            &created.id,
            &WidgetThemeInput {
                name: "Study Purple".to_string(),
                config: changed_config,
            },
        )
        .expect("update custom theme");

        assert!(!updated.is_builtin);
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "Study Purple");
        assert_eq!(updated.config.accent, "#A78BFA");
    }

    #[test]
    fn theme_name_must_be_between_one_and_sixty_four_characters() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");

        let empty = create_widget_theme(
            &conn,
            &WidgetThemeInput {
                name: "   ".to_string(),
                config: default_theme_config(),
            },
        )
        .unwrap_err();
        assert!(empty.to_string().contains("cannot be empty"));

        let sixty_four = create_widget_theme(
            &conn,
            &WidgetThemeInput {
                name: "a".repeat(64),
                config: default_theme_config(),
            },
        )
        .expect("accept 64-character theme name");
        assert_eq!(sixty_four.name.chars().count(), 64);

        let too_long = create_widget_theme(
            &conn,
            &WidgetThemeInput {
                name: "a".repeat(65),
                config: default_theme_config(),
            },
        )
        .unwrap_err();
        assert!(too_long.to_string().contains("at most 64"));
    }

    #[test]
    fn pane_theme_overrides_widget_theme() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let custom = create_widget_theme(
            &conn,
            &WidgetThemeInput {
                name: "Custom".to_string(),
                config: default_theme_config(),
            },
        )
        .expect("create theme");
        let pane = create_widget_pane(
            &conn,
            &WidgetPaneInput {
                theme_id: Some(custom.id.clone()),
                ..pane_input(DEFAULT_WIDGET_ID, None, 1)
            },
        )
        .expect("create pane");

        assert_eq!(
            resolve_widget_pane_theme(&conn, &pane.id)
                .expect("resolve theme")
                .id,
            custom.id
        );
    }

    #[test]
    fn pane_without_theme_uses_widget_theme() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let custom = create_widget_theme(
            &conn,
            &WidgetThemeInput {
                name: "Widget theme".to_string(),
                config: default_theme_config(),
            },
        )
        .expect("create theme");
        update_widget_instance(
            &conn,
            DEFAULT_WIDGET_ID,
            &widget_input("Main", Some(&custom.id)),
        )
        .expect("set widget theme");

        assert_eq!(
            resolve_widget_pane_theme(&conn, DEFAULT_PANE_ID)
                .expect("resolve widget theme")
                .id,
            custom.id
        );
    }

    #[test]
    fn widget_theme_assignment_is_isolated() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let second = create_widget_with_default_pane(&conn, None).expect("create second widget");

        set_widget_default_theme(&conn, DEFAULT_WIDGET_ID, Some(OCEAN_THEME_ID))
            .expect("set main ocean theme");
        set_widget_default_theme(&conn, &second.id, Some(FOREST_THEME_ID))
            .expect("set second forest theme");
        set_widget_default_theme(&conn, DEFAULT_WIDGET_ID, Some(VIOLET_THEME_ID))
            .expect("change main violet theme");

        assert_eq!(
            get_widget_instance(&conn, DEFAULT_WIDGET_ID)
                .unwrap()
                .default_theme_id
                .as_deref(),
            Some(VIOLET_THEME_ID)
        );
        assert_eq!(
            get_widget_instance(&conn, &second.id)
                .unwrap()
                .default_theme_id
                .as_deref(),
            Some(FOREST_THEME_ID)
        );
    }

    #[test]
    fn pane_theme_assignment_is_isolated_and_preserves_other_fields() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        set_widget_pane_list_for_widget(
            &conn,
            DEFAULT_WIDGET_ID,
            DEFAULT_PANE_ID,
            Some("local-inbox"),
        )
        .expect("bind main pane");
        let second = create_pane_for_widget(&conn, DEFAULT_WIDGET_ID, Some("local-today"))
            .expect("create second pane");

        set_widget_pane_theme_for_widget(
            &conn,
            DEFAULT_WIDGET_ID,
            DEFAULT_PANE_ID,
            Some(OCEAN_THEME_ID),
        )
        .expect("set first pane ocean");
        set_widget_pane_theme_for_widget(
            &conn,
            DEFAULT_WIDGET_ID,
            &second.id,
            Some(VIOLET_THEME_ID),
        )
        .expect("set second pane violet");
        set_widget_pane_theme_for_widget(
            &conn,
            DEFAULT_WIDGET_ID,
            DEFAULT_PANE_ID,
            Some(FOREST_THEME_ID),
        )
        .expect("change first pane forest");

        let first = get_widget_pane(&conn, DEFAULT_PANE_ID).unwrap();
        let unchanged_second = get_widget_pane(&conn, &second.id).unwrap();
        assert_eq!(first.theme_id.as_deref(), Some(FOREST_THEME_ID));
        assert_eq!(first.list_id.as_deref(), Some("local-inbox"));
        assert_eq!(first.sort_order, 0);
        assert_eq!(unchanged_second.theme_id.as_deref(), Some(VIOLET_THEME_ID));
        assert_eq!(unchanged_second.list_id.as_deref(), Some("local-today"));
    }

    #[test]
    fn pane_theme_assignment_rejects_another_widget() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let second = create_widget_with_default_pane(&conn, None).expect("create second widget");
        let second_pane = list_widget_panes(&conn, &second.id).unwrap()[0].clone();

        let error = set_widget_pane_theme_for_widget(
            &conn,
            DEFAULT_WIDGET_ID,
            &second_pane.id,
            Some(OCEAN_THEME_ID),
        )
        .unwrap_err();

        assert!(error.to_string().contains("current widget"));
        assert!(get_widget_pane(&conn, &second_pane.id)
            .unwrap()
            .theme_id
            .is_none());
    }

    #[test]
    fn missing_widget_theme_falls_back_to_default() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        update_widget_instance(&conn, DEFAULT_WIDGET_ID, &widget_input("Main", None))
            .expect("clear widget theme");

        assert_eq!(
            resolve_widget_pane_theme(&conn, DEFAULT_PANE_ID)
                .expect("resolve default theme")
                .id,
            DEFAULT_THEME_ID
        );
    }

    #[test]
    fn deleting_custom_theme_clears_references_and_preserves_fallback() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let custom = create_widget_theme(
            &conn,
            &WidgetThemeInput {
                name: "Temporary".to_string(),
                config: default_theme_config(),
            },
        )
        .expect("create theme");
        update_widget_instance(
            &conn,
            DEFAULT_WIDGET_ID,
            &widget_input("Main", Some(&custom.id)),
        )
        .expect("set widget theme");
        update_widget_pane(
            &conn,
            DEFAULT_PANE_ID,
            &WidgetPaneInput {
                theme_id: Some(custom.id.clone()),
                ..pane_input(DEFAULT_WIDGET_ID, None, 0)
            },
        )
        .expect("set pane theme");

        delete_widget_theme(&conn, &custom.id).expect("delete custom theme");

        assert!(get_widget_instance(&conn, DEFAULT_WIDGET_ID)
            .expect("get widget")
            .default_theme_id
            .is_none());
        assert!(get_widget_pane(&conn, DEFAULT_PANE_ID)
            .expect("get pane")
            .theme_id
            .is_none());
        assert_eq!(
            resolve_widget_pane_theme(&conn, DEFAULT_PANE_ID)
                .expect("resolve default theme")
                .id,
            DEFAULT_THEME_ID
        );
    }

    #[test]
    fn bootstrap_has_exactly_one_open_primary() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        create_widget_with_default_pane(&conn, None).expect("create secondary");
        let instances = list_widget_instances(&conn).expect("list instances");
        assert_eq!(instances.iter().filter(|item| item.is_primary).count(), 1);
        assert!(
            instances
                .iter()
                .find(|item| item.is_primary)
                .unwrap()
                .is_open
        );
    }

    #[test]
    fn primary_widget_cannot_be_deleted() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let error = delete_widget_instance(&conn, DEFAULT_WIDGET_ID).unwrap_err();
        assert!(error.to_string().contains("primary"));
    }

    #[test]
    fn high_level_secondary_creation_is_open_with_one_pane() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let secondary =
            create_widget_with_default_pane(&conn, Some("local-inbox")).expect("create secondary");
        assert!(!secondary.is_primary);
        assert!(secondary.is_open);
        let panes = list_widget_panes(&conn, &secondary.id).expect("list panes");
        assert_eq!(panes.len(), 1);
        assert_eq!(panes[0].list_id.as_deref(), Some("local-inbox"));
    }

    #[test]
    fn window_state_fields_persist_independently() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let secondary = create_widget_with_default_pane(&conn, None).expect("create secondary");
        set_widget_open(&conn, &secondary.id, false).expect("close secondary");
        set_widget_pinned(&conn, &secondary.id, true).expect("pin secondary");
        save_widget_geometry(&conn, &secondary.id, 120, -40, 420, 650).expect("save geometry");

        let stored = get_widget_instance(&conn, &secondary.id).expect("load secondary");
        assert!(!stored.is_open);
        assert!(stored.pinned);
        assert_eq!((stored.window_x, stored.window_y), (Some(120), Some(-40)));
        assert_eq!(
            (stored.window_width, stored.window_height),
            (Some(420), Some(650))
        );
        assert!(!get_primary_widget_instance(&conn).unwrap().pinned);
    }

    #[test]
    fn changing_secondary_pane_list_does_not_change_primary() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        update_first_pane_list(&conn, DEFAULT_WIDGET_ID, "local-inbox").expect("bind primary");
        let secondary =
            create_widget_with_default_pane(&conn, Some("local-inbox")).expect("create secondary");
        update_first_pane_list(&conn, &secondary.id, "local-today").expect("change secondary");

        assert_eq!(
            list_widget_panes(&conn, DEFAULT_WIDGET_ID).unwrap()[0]
                .list_id
                .as_deref(),
            Some("local-inbox")
        );
        assert_eq!(
            list_widget_panes(&conn, &secondary.id).unwrap()[0]
                .list_id
                .as_deref(),
            Some("local-today")
        );
    }

    #[test]
    fn current_widget_helpers_create_and_order_panes() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");

        let second =
            create_pane_for_widget(&conn, DEFAULT_WIDGET_ID, None).expect("create second pane");
        let third = create_pane_for_widget(&conn, DEFAULT_WIDGET_ID, Some("local-inbox"))
            .expect("create third pane");

        assert_eq!(second.sort_order, 1);
        assert_eq!(third.sort_order, 2);
        assert_eq!(third.list_id.as_deref(), Some("local-inbox"));
    }

    #[test]
    fn current_widget_helpers_reject_another_widgets_pane() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let secondary =
            create_widget_with_default_pane(&conn, None).expect("create secondary widget");
        let secondary_pane = list_widget_panes(&conn, &secondary.id).unwrap()[0].clone();

        let error = set_widget_pane_list_for_widget(
            &conn,
            DEFAULT_WIDGET_ID,
            &secondary_pane.id,
            Some("local-inbox"),
        )
        .unwrap_err();

        assert!(error.to_string().contains("current widget"));
        assert!(get_widget_pane(&conn, &secondary_pane.id)
            .unwrap()
            .list_id
            .is_none());
    }

    #[test]
    fn current_widget_cannot_delete_its_last_pane() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");

        let error =
            delete_widget_pane_for_widget(&conn, DEFAULT_WIDGET_ID, DEFAULT_PANE_ID).unwrap_err();

        assert!(error.to_string().contains("at least one pane"));
        assert_eq!(
            list_widget_panes(&conn, DEFAULT_WIDGET_ID).unwrap().len(),
            1
        );
    }

    #[test]
    fn current_widget_pane_delete_keeps_tasks_and_lists() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let task = db::insert_local_task(&conn, "Keep me", "inbox").expect("insert task");
        let second = create_pane_for_widget(&conn, DEFAULT_WIDGET_ID, Some("local-inbox"))
            .expect("create second pane");

        delete_widget_pane_for_widget(&conn, DEFAULT_WIDGET_ID, &second.id)
            .expect("delete second pane");

        assert!(db::get_task(&conn, &task.id).is_ok());
        assert!(db::list_id_for_role(&conn, "inbox").is_ok());
    }

    #[test]
    fn layout_mode_persists_for_current_widget() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");

        set_widget_layout_mode(&conn, DEFAULT_WIDGET_ID, WidgetLayoutMode::Grid)
            .expect("set grid layout");

        assert_eq!(
            get_widget_instance(&conn, DEFAULT_WIDGET_ID)
                .unwrap()
                .layout_mode,
            WidgetLayoutMode::Grid
        );
    }

    #[test]
    fn show_completed_inherits_global_value() {
        assert!(!effective_show_completed(None, false));
        assert!(effective_show_completed(None, true));
        assert!(effective_show_completed(Some(true), false));
        assert!(!effective_show_completed(Some(false), true));
    }

    #[test]
    fn pane_max_tasks_validation_still_applies() {
        let conn = test_db();
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        let error = create_widget_pane(
            &conn,
            &WidgetPaneInput {
                max_tasks: Some(0),
                ..pane_input(DEFAULT_WIDGET_ID, None, 1)
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("maxTasks"));
    }

    #[test]
    fn legacy_always_on_top_seeds_primary_pinned_once() {
        let conn = test_db();
        db::save_settings(
            &conn,
            &crate::models::SettingsDto {
                always_on_top: true,
                ..Default::default()
            },
        )
        .expect("save legacy settings");
        bootstrap_defaults(&conn).expect("bootstrap defaults");
        assert!(get_primary_widget_instance(&conn).unwrap().pinned);

        db::save_settings(&conn, &crate::models::SettingsDto::default())
            .expect("change legacy settings");
        bootstrap_defaults(&conn).expect("repeat bootstrap");
        assert!(get_primary_widget_instance(&conn).unwrap().pinned);
    }

    #[test]
    fn four_a_schema_migrates_idempotently() {
        let conn = Connection::open_in_memory().expect("open legacy database");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO settings(key, value) VALUES
              ('settings', '{"alwaysOnTop":true}');
            CREATE TABLE widget_themes (
              id TEXT PRIMARY KEY, name TEXT NOT NULL, config_json TEXT NOT NULL,
              is_builtin INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE widget_instances (
              id TEXT PRIMARY KEY, name TEXT NOT NULL, layout_mode TEXT NOT NULL,
              default_theme_id TEXT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE widget_panes (
              id TEXT PRIMARY KEY, widget_instance_id TEXT NOT NULL, list_id TEXT NULL,
              title_override TEXT NULL, theme_id TEXT NULL, sort_order INTEGER NOT NULL,
              show_completed INTEGER NULL, max_tasks INTEGER NULL,
              created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            INSERT INTO widget_instances
              (id, name, layout_mode, default_theme_id, created_at, updated_at)
              VALUES ('main', 'Main', 'single', NULL, '2026-01-01', '2026-01-01');
            "#,
        )
        .expect("create 4A schema");

        init_schema(&conn).expect("first migration");
        bootstrap_defaults(&conn).expect("first bootstrap");
        init_schema(&conn).expect("second migration");
        bootstrap_defaults(&conn).expect("second bootstrap");

        let primary = get_primary_widget_instance(&conn).expect("load primary");
        assert_eq!(primary.id, DEFAULT_WIDGET_ID);
        assert!(primary.is_open);
        assert!(primary.pinned);
        assert_eq!(
            list_widget_instances(&conn)
                .unwrap()
                .iter()
                .filter(|instance| instance.is_primary)
                .count(),
            1
        );
    }
}
