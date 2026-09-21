use anyhow::{anyhow, bail, Context};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::models::{
    WidgetInstanceDto, WidgetInstanceInput, WidgetLayoutMode, WidgetPaneDto, WidgetPaneInput,
    WidgetThemeConfig, WidgetThemeDto, WidgetThemeInput,
};

pub const DEFAULT_THEME_ID: &str = "default";
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
    Ok(())
}

pub fn bootstrap_defaults(conn: &Connection) -> anyhow::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    let now = Utc::now().to_rfc3339();

    let theme_created = transaction.execute(
        "INSERT OR IGNORE INTO widget_themes
         (id, name, config_json, is_builtin, created_at, updated_at)
         VALUES (?1, 'Default', ?2, 1, ?3, ?3)",
        params![
            DEFAULT_THEME_ID,
            serde_json::to_string(&default_theme_config())?,
            now
        ],
    )? > 0;

    let instance_count: i64 =
        transaction.query_row("SELECT COUNT(*) FROM widget_instances", [], |row| {
            row.get(0)
        })?;
    let widget_created = if instance_count == 0 {
        transaction.execute(
            "INSERT INTO widget_instances
             (id, name, layout_mode, default_theme_id, created_at, updated_at)
             VALUES (?1, 'Main', 'single', ?2, ?3, ?3)",
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

    transaction.commit()?;

    if theme_created {
        println!("[widget-config] initialized default theme");
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
        "SELECT id, name, layout_mode, default_theme_id, created_at, updated_at
         FROM widget_instances ORDER BY created_at, id",
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
        "SELECT id, name, layout_mode, default_theme_id, created_at, updated_at
         FROM widget_instances WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    let row = rows.next()?.context("widget instance not found")?;
    widget_instance_from_row(row)
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
         (id, name, layout_mode, default_theme_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
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
    let name = validate_name(&input.name, "theme name")?;
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
    let name = validate_name(&input.name, "theme name")?;
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
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
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

        assert_eq!(list_widget_themes(&conn).expect("list themes").len(), 1);
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

        assert_eq!(list_widget_themes(&conn).expect("list themes").len(), 1);
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

        delete_widget_instance(&conn, DEFAULT_WIDGET_ID).expect("delete widget");

        assert!(list_widget_panes(&conn, DEFAULT_WIDGET_ID)
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
}
