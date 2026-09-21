use anyhow::Context;
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use uuid::Uuid;

use crate::models::{SettingsDto, TaskDto, TaskListDto, TaskPatchDto};

const TASK_COLUMNS: &str = "id, graph_id, list_id, title, status, importance, note, due_date_time, reminder_date_time, completed_date_time, time_zone, is_reminder_on, recurrence_json, created_at, modified_at, dirty";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteTaskDisposition {
    LocalOnly,
    RemotePending,
}

pub fn init_db(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
          key TEXT PRIMARY KEY,
          value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS task_lists (
          id TEXT PRIMARY KEY,
          display_name TEXT NOT NULL,
          role TEXT,
          updated_at TEXT
        );

        CREATE TABLE IF NOT EXISTS tasks (
          id TEXT PRIMARY KEY,
          graph_id TEXT UNIQUE,
          list_id TEXT NOT NULL,
          title TEXT NOT NULL,
          status TEXT NOT NULL,
          importance TEXT,
          note TEXT,
          due_date_time TEXT,
          reminder_date_time TEXT,
          completed_date_time TEXT,
          time_zone TEXT,
          is_reminder_on INTEGER NOT NULL DEFAULT 0,
          recurrence_json TEXT,
          created_at TEXT,
          modified_at TEXT,
          local_created_at TEXT NOT NULL,
          dirty INTEGER NOT NULL DEFAULT 0,
          pending_delete INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS pending_ops (
          op_id TEXT PRIMARY KEY,
          op_type TEXT NOT NULL,
          task_id TEXT,
          list_id TEXT,
          payload_json TEXT NOT NULL,
          created_at TEXT NOT NULL,
          retry_count INTEGER NOT NULL DEFAULT 0,
          last_error TEXT
        );
        "#,
    )
    .context("failed to initialize sqlite schema")?;

    ensure_task_column(conn, "note", "TEXT")?;
    ensure_task_column(conn, "due_date_time", "TEXT")?;
    ensure_task_column(conn, "reminder_date_time", "TEXT")?;
    ensure_task_column(conn, "completed_date_time", "TEXT")?;
    ensure_task_column(conn, "time_zone", "TEXT")?;
    ensure_task_column(conn, "is_reminder_on", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_task_column(conn, "recurrence_json", "TEXT")?;
    crate::widget_config::init_schema(conn)?;

    Ok(())
}

fn ensure_task_column(conn: &Connection, name: &str, definition: &str) -> anyhow::Result<()> {
    let exists = conn
        .prepare("PRAGMA table_info(tasks)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|column| column == name);

    if !exists {
        conn.execute(
            &format!("ALTER TABLE tasks ADD COLUMN {name} {definition}"),
            [],
        )?;
    }

    Ok(())
}

pub fn ensure_local_core_lists(conn: &Connection) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    for (id, display_name, role) in [
        ("local-today", "Today", "today"),
        ("local-inbox", "Inbox", "inbox"),
        ("local-this-week", "This Week", "this_week"),
    ] {
        conn.execute(
            "INSERT OR IGNORE INTO task_lists (id, display_name, role, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, display_name, role, now],
        )?;
    }
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> anyhow::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .context("failed to read setting")
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn load_settings(conn: &Connection) -> anyhow::Result<SettingsDto> {
    let value = get_setting(conn, "settings")?;
    match value {
        Some(raw) => Ok(serde_json::from_str(&raw).unwrap_or_default()),
        None => Ok(SettingsDto::default()),
    }
}

pub fn save_settings(conn: &Connection, settings: &SettingsDto) -> anyhow::Result<()> {
    set_setting(conn, "settings", &serde_json::to_string(settings)?)?;
    Ok(())
}

pub fn last_synced_at(conn: &Connection) -> anyhow::Result<Option<String>> {
    get_setting(conn, "last_synced_at")
}

pub fn set_last_synced_at(conn: &Connection) -> anyhow::Result<String> {
    let now = Utc::now().to_rfc3339();
    set_setting(conn, "last_synced_at", &now)?;
    Ok(now)
}

pub fn save_auth_refresh_token(conn: &Connection, refresh_token: &str) -> anyhow::Result<()> {
    set_setting(conn, "auth_refresh_token", refresh_token)
}

pub fn load_auth_refresh_token(conn: &Connection) -> anyhow::Result<Option<String>> {
    get_setting(conn, "auth_refresh_token")
}

pub fn clear_auth_refresh_token(conn: &Connection) -> anyhow::Result<()> {
    conn.execute(
        "DELETE FROM settings WHERE key IN ('auth_refresh_token', 'auth_account')",
        [],
    )?;
    Ok(())
}

pub fn save_auth_account(conn: &Connection, account: &str) -> anyhow::Result<()> {
    set_setting(conn, "auth_account", account)
}

pub fn load_auth_account(conn: &Connection) -> anyhow::Result<Option<String>> {
    get_setting(conn, "auth_account")
}

pub fn list_id_for_role(conn: &Connection, role: &str) -> anyhow::Result<String> {
    conn.query_row(
        "SELECT id FROM task_lists WHERE role = ?1 ORDER BY updated_at DESC LIMIT 1",
        params![role],
        |row| row.get(0),
    )
    .context("failed to resolve task list role")
}

pub fn upsert_task_list(
    conn: &Connection,
    id: &str,
    display_name: &str,
    role: Option<&str>,
) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO task_lists (id, display_name, role, updated_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET display_name = excluded.display_name, role = excluded.role, updated_at = excluded.updated_at",
        params![id, display_name, role, now],
    )?;
    Ok(())
}

pub fn replace_remote_task_lists(
    conn: &Connection,
    lists: &[(String, String, Option<String>)],
) -> anyhow::Result<()> {
    let remote_ids = lists
        .iter()
        .map(|(id, _, _)| id.as_str())
        .collect::<HashSet<_>>();

    for (id, display_name, role) in lists {
        upsert_task_list(conn, id, display_name, role.as_deref())?;
    }

    let mut stmt = conn.prepare("SELECT id FROM task_lists WHERE id NOT LIKE 'local-%'")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut stale_ids = Vec::new();
    for row in rows {
        let id = row?;
        if !remote_ids.contains(id.as_str()) {
            stale_ids.push(id);
        }
    }

    for id in stale_ids {
        conn.execute("DELETE FROM task_lists WHERE id = ?1", params![id])?;
    }

    Ok(())
}

pub fn list_task_lists(conn: &Connection) -> anyhow::Result<Vec<TaskListDto>> {
    let remote = read_task_lists(conn, "id NOT LIKE 'local-%'")?;
    if remote.is_empty() {
        read_task_lists(conn, "id LIKE 'local-%'")
    } else {
        Ok(remote)
    }
}

fn read_task_lists(conn: &Connection, where_clause: &str) -> anyhow::Result<Vec<TaskListDto>> {
    let sql = format!(
        "SELECT id, display_name, role
         FROM task_lists
         WHERE {where_clause}
         ORDER BY
           CASE role WHEN 'today' THEN 0 WHEN 'inbox' THEN 1 WHEN 'this_week' THEN 2 ELSE 3 END,
           lower(display_name)"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(TaskListDto {
            id: row.get(0)?,
            display_name: row.get(1)?,
            role: row.get(2)?,
        })
    })?;

    let mut lists = Vec::new();
    for row in rows {
        lists.push(row?);
    }
    Ok(lists)
}

pub fn remote_task_list_exists(conn: &Connection, list_id: &str) -> anyhow::Result<bool> {
    let exists: Option<String> = conn
        .query_row(
            "SELECT id FROM task_lists WHERE id = ?1 AND id NOT LIKE 'local-%'",
            params![list_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(exists.is_some())
}

pub fn list_today_tasks(conn: &Connection) -> anyhow::Result<Vec<TaskDto>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT tasks.{}
         FROM tasks
         JOIN task_lists ON task_lists.id = tasks.list_id
         WHERE task_lists.role = 'today' AND pending_delete = 0
         ORDER BY status = 'completed' ASC, dirty DESC, COALESCE(modified_at, local_created_at) DESC",
        TASK_COLUMNS.replace(", ", ", tasks.")
    ))?;

    let rows = stmt.query_map([], row_to_task)?;
    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

pub fn list_tasks_for_list(conn: &Connection, list_id: &str) -> anyhow::Result<Vec<TaskDto>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {TASK_COLUMNS}
         FROM tasks
         WHERE list_id = ?1 AND pending_delete = 0
         ORDER BY status = 'completed' ASC, dirty DESC, COALESCE(modified_at, local_created_at) DESC",
    ))?;

    let rows = stmt.query_map(params![list_id], row_to_task)?;
    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

pub fn inbox_count(conn: &Connection) -> anyhow::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*)
         FROM tasks
         JOIN task_lists ON task_lists.id = tasks.list_id
         WHERE task_lists.role = 'inbox' AND status != 'completed' AND pending_delete = 0",
        [],
        |row| row.get(0),
    )
    .context("failed to count inbox tasks")
}

pub fn insert_local_task(conn: &Connection, title: &str, role: &str) -> anyhow::Result<TaskDto> {
    let list_id = list_id_for_role(conn, role)?;
    insert_local_task_for_list(conn, title, &list_id)
}

pub fn insert_local_task_for_list(
    conn: &Connection,
    title: &str,
    list_id: &str,
) -> anyhow::Result<TaskDto> {
    let id = format!("local-{}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO tasks (id, graph_id, list_id, title, status, local_created_at, dirty)
         VALUES (?1, NULL, ?2, ?3, 'notStarted', ?4, 1)",
        params![id, list_id, title, now],
    )?;

    insert_pending_op(
        conn,
        "create_task",
        Some(&id),
        Some(&list_id),
        &serde_json::json!({ "title": title }).to_string(),
    )?;

    get_task(conn, &id)
}

pub fn complete_local_task(conn: &Connection, task_id: &str) -> anyhow::Result<()> {
    let list_id: String = conn.query_row(
        "SELECT list_id FROM tasks WHERE id = ?1",
        params![task_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE tasks SET status = 'completed', completed_date_time = ?2, dirty = 1, modified_at = ?2 WHERE id = ?1",
        params![task_id, Utc::now().to_rfc3339()],
    )?;

    insert_pending_op(
        conn,
        "complete_task",
        Some(task_id),
        Some(&list_id),
        &serde_json::json!({ "status": "completed" }).to_string(),
    )?;

    Ok(())
}

pub fn update_local_task(
    conn: &Connection,
    task_id: &str,
    patch: &TaskPatchDto,
) -> anyhow::Result<TaskDto> {
    let current = get_task(conn, task_id)?;
    let title = patch.title.trim();
    let title = if title.is_empty() {
        current.title.as_str()
    } else {
        title
    };
    let recurrence_json = patch.recurrence.clone().map(|value| value.to_string());
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE tasks
         SET title = ?2,
             importance = ?3,
             note = ?4,
             due_date_time = ?5,
             reminder_date_time = ?6,
             time_zone = ?7,
             is_reminder_on = ?8,
             recurrence_json = ?9,
             dirty = 1,
             modified_at = ?10
         WHERE id = ?1",
        params![
            task_id,
            title,
            patch.importance.as_deref(),
            patch.note.as_deref(),
            patch.due_date_time.as_deref(),
            patch.reminder_date_time.as_deref(),
            patch.time_zone.as_deref(),
            if patch.is_reminder_on { 1 } else { 0 },
            recurrence_json,
            now
        ],
    )?;

    insert_pending_op(
        conn,
        "update_task",
        Some(task_id),
        Some(&current.list_id),
        &serde_json::to_string(patch)?,
    )?;

    get_task(conn, task_id)
}

pub fn delete_local_task(
    conn: &Connection,
    task_id: &str,
) -> anyhow::Result<DeleteTaskDisposition> {
    let transaction = conn.unchecked_transaction()?;
    let (graph_id, list_id): (Option<String>, String) = transaction
        .query_row(
            "SELECT graph_id, list_id FROM tasks WHERE id = ?1",
            params![task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .context("failed to resolve task for deletion")?;

    transaction.execute(
        "DELETE FROM pending_ops WHERE task_id = ?1",
        params![task_id],
    )?;

    let disposition = if let Some(graph_id) = graph_id {
        transaction.execute(
            "UPDATE tasks
             SET pending_delete = 1, dirty = 1, modified_at = ?2
             WHERE id = ?1",
            params![task_id, Utc::now().to_rfc3339()],
        )?;
        insert_pending_op(
            &transaction,
            "delete_task",
            Some(task_id),
            Some(&list_id),
            &serde_json::json!({ "graphId": graph_id }).to_string(),
        )?;
        DeleteTaskDisposition::RemotePending
    } else {
        transaction.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
        DeleteTaskDisposition::LocalOnly
    };

    transaction.commit()?;
    Ok(disposition)
}

pub fn finalize_remote_task_delete(
    conn: &Connection,
    task_id: &str,
    op_id: &str,
) -> anyhow::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    transaction.execute(
        "DELETE FROM pending_ops WHERE task_id = ?1 OR op_id = ?2",
        params![task_id, op_id],
    )?;
    transaction.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
    transaction.commit()?;
    Ok(())
}

pub fn queue_inflight_created_task_delete(
    conn: &Connection,
    task: &TaskDto,
    graph_id: &str,
) -> anyhow::Result<()> {
    let transaction = conn.unchecked_transaction()?;
    transaction.execute(
        "INSERT INTO tasks (
           id, graph_id, list_id, title, status, local_created_at, dirty, pending_delete
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 1)",
        params![
            task.id,
            graph_id,
            task.list_id,
            task.title,
            task.status,
            Utc::now().to_rfc3339()
        ],
    )?;
    insert_pending_op(
        &transaction,
        "delete_task",
        Some(&task.id),
        Some(&task.list_id),
        &serde_json::json!({ "graphId": graph_id }).to_string(),
    )?;
    transaction.commit()?;
    Ok(())
}

pub fn get_task(conn: &Connection, task_id: &str) -> anyhow::Result<TaskDto> {
    conn.query_row(
        &format!("SELECT {TASK_COLUMNS} FROM tasks WHERE id = ?1"),
        params![task_id],
        row_to_task,
    )
    .context("failed to get task")
}

pub fn upsert_graph_task(
    conn: &Connection,
    list_id: &str,
    graph_id: &str,
    title: &str,
    status: &str,
    importance: Option<&str>,
    note: Option<&str>,
    due_date_time: Option<&str>,
    reminder_date_time: Option<&str>,
    completed_date_time: Option<&str>,
    time_zone: Option<&str>,
    is_reminder_on: bool,
    recurrence: Option<&serde_json::Value>,
    created_at: Option<&str>,
    modified_at: Option<&str>,
) -> anyhow::Result<()> {
    let existing_local_id: Option<String> = conn
        .query_row(
            "SELECT id FROM tasks WHERE graph_id = ?1",
            params![graph_id],
            |row| row.get(0),
        )
        .optional()?;
    let id = existing_local_id.unwrap_or_else(|| graph_id.to_string());
    let now = Utc::now().to_rfc3339();
    let recurrence_json = recurrence.map(|value| value.to_string());

    conn.execute(
        "INSERT INTO tasks (id, graph_id, list_id, title, status, importance, note, due_date_time, reminder_date_time, completed_date_time, time_zone, is_reminder_on, recurrence_json, created_at, modified_at, local_created_at, dirty)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 0)
         ON CONFLICT(id) DO UPDATE SET
           graph_id = excluded.graph_id,
           list_id = excluded.list_id,
           title = excluded.title,
           status = excluded.status,
           importance = excluded.importance,
           note = excluded.note,
           due_date_time = excluded.due_date_time,
           reminder_date_time = excluded.reminder_date_time,
           completed_date_time = excluded.completed_date_time,
           time_zone = excluded.time_zone,
           is_reminder_on = excluded.is_reminder_on,
           recurrence_json = excluded.recurrence_json,
           created_at = excluded.created_at,
           modified_at = excluded.modified_at,
           dirty = 0",
        params![
            id,
            graph_id,
            list_id,
            title,
            status,
            importance,
            note,
            due_date_time,
            reminder_date_time,
            completed_date_time,
            time_zone,
            if is_reminder_on { 1 } else { 0 },
            recurrence_json,
            created_at,
            modified_at,
            now
        ],
    )?;
    Ok(())
}

pub fn mark_missing_remote_tasks_deleted(
    conn: &Connection,
    list_id: &str,
    remote_graph_ids: &[String],
) -> anyhow::Result<()> {
    let cutoff = (Utc::now() - Duration::seconds(90)).to_rfc3339();
    let mut stmt = conn.prepare(
        "SELECT graph_id FROM tasks
         WHERE list_id = ?1
           AND graph_id IS NOT NULL
           AND dirty = 0
           AND pending_delete = 0
           AND COALESCE(modified_at, local_created_at) < ?2",
    )?;
    let rows = stmt.query_map(params![list_id, cutoff], |row| row.get::<_, String>(0))?;

    for row in rows {
        let graph_id = row?;
        if !remote_graph_ids.iter().any(|id| id == &graph_id) {
            conn.execute(
                "UPDATE tasks
                 SET pending_delete = 1, status = 'completed', completed_date_time = ?3, modified_at = ?3
                 WHERE list_id = ?1 AND graph_id = ?2 AND dirty = 0",
                params![list_id, graph_id, Utc::now().to_rfc3339()],
            )?;
        }
    }
    Ok(())
}

pub fn insert_pending_op(
    conn: &Connection,
    op_type: &str,
    task_id: Option<&str>,
    list_id: Option<&str>,
    payload_json: &str,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO pending_ops (op_id, op_type, task_id, list_id, payload_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            Uuid::new_v4().to_string(),
            op_type,
            task_id,
            list_id,
            payload_json,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskDto> {
    let recurrence_json: Option<String> = row.get(12)?;
    Ok(TaskDto {
        id: row.get(0)?,
        graph_id: row.get(1)?,
        list_id: row.get(2)?,
        title: row.get(3)?,
        status: row.get(4)?,
        importance: row.get(5)?,
        note: row.get(6)?,
        due_date_time: row.get(7)?,
        reminder_date_time: row.get(8)?,
        completed_date_time: row.get(9)?,
        time_zone: row.get(10)?,
        is_reminder_on: row.get::<_, i64>(11)? != 0,
        recurrence: recurrence_json.and_then(|raw| serde_json::from_str(&raw).ok()),
        created_at: row.get(13)?,
        modified_at: row.get(14)?,
        dirty: row.get::<_, i64>(15)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open test database");
        init_db(&conn).expect("initialize test database");
        ensure_local_core_lists(&conn).expect("initialize local lists");
        conn
    }

    fn insert_remote_task(conn: &Connection, task_id: &str) {
        upsert_task_list(conn, "remote-list", "Tasks", Some("inbox")).expect("insert remote list");
        upsert_graph_task(
            conn,
            "remote-list",
            task_id,
            "Remote task",
            "notStarted",
            Some("normal"),
            None,
            None,
            None,
            None,
            Some("UTC"),
            false,
            None,
            None,
            None,
        )
        .expect("insert remote task");
    }

    fn pending_types(conn: &Connection, task_id: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(
                "SELECT op_type FROM pending_ops WHERE task_id = ?1 ORDER BY created_at, op_id",
            )
            .expect("prepare pending query");
        stmt.query_map(params![task_id], |row| row.get(0))
            .expect("query pending operations")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect pending operations")
    }

    #[test]
    fn local_only_delete_removes_task_and_all_pending_operations() {
        let conn = test_db();
        let task = insert_local_task(&conn, "Local task", "inbox").expect("insert local task");
        complete_local_task(&conn, &task.id).expect("complete local task");

        let disposition = delete_local_task(&conn, &task.id).expect("delete local task");

        assert_eq!(disposition, DeleteTaskDisposition::LocalOnly);
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM tasks WHERE id = ?1",
                params![task.id],
                |row| row.get::<_, i64>(0)
            )
            .expect("count tasks"),
            0
        );
        assert!(pending_types(&conn, &task.id).is_empty());
    }

    #[test]
    fn remote_delete_replaces_update_and_complete_with_one_delete() {
        let conn = test_db();
        insert_remote_task(&conn, "remote-task");
        let patch = TaskPatchDto {
            title: "Updated".to_string(),
            importance: Some("high".to_string()),
            note: None,
            due_date_time: None,
            reminder_date_time: None,
            time_zone: Some("UTC".to_string()),
            is_reminder_on: false,
            recurrence: None,
        };
        update_local_task(&conn, "remote-task", &patch).expect("update remote task locally");
        complete_local_task(&conn, "remote-task").expect("complete remote task locally");

        let disposition = delete_local_task(&conn, "remote-task").expect("delete remote task");

        assert_eq!(disposition, DeleteTaskDisposition::RemotePending);
        assert_eq!(pending_types(&conn, "remote-task"), vec!["delete_task"]);
        assert_eq!(
            conn.query_row(
                "SELECT pending_delete FROM tasks WHERE id = ?1",
                params!["remote-task"],
                |row| row.get::<_, i64>(0)
            )
            .expect("read tombstone"),
            1
        );
        assert!(list_tasks_for_list(&conn, "remote-list")
            .expect("list visible tasks")
            .is_empty());
    }

    #[test]
    fn graph_pull_does_not_reveal_pending_delete_tombstone() {
        let conn = test_db();
        insert_remote_task(&conn, "remote-task");
        delete_local_task(&conn, "remote-task").expect("delete remote task");

        upsert_graph_task(
            &conn,
            "remote-list",
            "remote-task",
            "Still remote",
            "notStarted",
            Some("normal"),
            None,
            None,
            None,
            None,
            Some("UTC"),
            false,
            None,
            None,
            None,
        )
        .expect("pull remote task");

        assert!(list_tasks_for_list(&conn, "remote-list")
            .expect("list visible tasks")
            .is_empty());
    }

    #[test]
    fn successful_remote_delete_finalizes_tombstone_and_outbox() {
        let conn = test_db();
        insert_remote_task(&conn, "remote-task");
        delete_local_task(&conn, "remote-task").expect("delete remote task");
        let op_id: String = conn
            .query_row(
                "SELECT op_id FROM pending_ops WHERE task_id = ?1",
                params!["remote-task"],
                |row| row.get(0),
            )
            .expect("read delete operation");

        finalize_remote_task_delete(&conn, "remote-task", &op_id).expect("finalize remote delete");

        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM tasks WHERE id = ?1",
                params!["remote-task"],
                |row| row.get::<_, i64>(0)
            )
            .expect("count tasks"),
            0
        );
        assert!(pending_types(&conn, "remote-task").is_empty());
    }

    #[test]
    fn in_flight_create_completion_is_queued_for_compensating_delete() {
        let conn = test_db();
        let task = insert_local_task(&conn, "Local task", "inbox").expect("insert local task");
        delete_local_task(&conn, &task.id).expect("delete local task");

        queue_inflight_created_task_delete(&conn, &task, "new-remote-id")
            .expect("queue compensating delete");

        assert_eq!(pending_types(&conn, &task.id), vec!["delete_task"]);
        assert!(list_tasks_for_list(&conn, &task.list_id)
            .expect("list visible tasks")
            .is_empty());
    }
}
