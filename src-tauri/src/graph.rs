use anyhow::Context;
use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::models::{GraphCollection, GraphTask, GraphTaskList, GraphUserProfile, TaskDto};

const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

pub async fn list_task_lists(
    http: &Client,
    access_token: &str,
) -> anyhow::Result<Vec<GraphTaskList>> {
    let url = format!("{GRAPH_BASE}/me/todo/lists");
    collect_pages(http, access_token, url).await
}

pub async fn get_profile(http: &Client, access_token: &str) -> anyhow::Result<GraphUserProfile> {
    let url = format!("{GRAPH_BASE}/me");
    let result = http
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<GraphUserProfile>()
        .await?;
    Ok(result)
}

pub async fn list_tasks(
    http: &Client,
    access_token: &str,
    list_id: &str,
) -> anyhow::Result<Vec<GraphTask>> {
    let url = format!("{GRAPH_BASE}/me/todo/lists/{list_id}/tasks");
    collect_pages(http, access_token, url)
        .await
        .with_context(|| format!("failed to list tasks for {list_id}"))
}

pub async fn create_task(
    http: &Client,
    access_token: &str,
    list_id: &str,
    task: &TaskDto,
) -> anyhow::Result<GraphTask> {
    let url = format!("{GRAPH_BASE}/me/todo/lists/{list_id}/tasks");
    let result = http
        .post(url)
        .bearer_auth(access_token)
        .json(&task_payload(task))
        .send()
        .await?
        .error_for_status()?
        .json::<GraphTask>()
        .await?;
    Ok(result)
}

pub async fn update_task(
    http: &Client,
    access_token: &str,
    list_id: &str,
    task_id: &str,
    task: &TaskDto,
) -> anyhow::Result<GraphTask> {
    let url = format!("{GRAPH_BASE}/me/todo/lists/{list_id}/tasks/{task_id}");
    let result = http
        .patch(url)
        .bearer_auth(access_token)
        .json(&task_payload(task))
        .send()
        .await?
        .error_for_status()?
        .json::<GraphTask>()
        .await?;
    Ok(result)
}

pub async fn complete_task(
    http: &Client,
    access_token: &str,
    list_id: &str,
    task_id: &str,
) -> anyhow::Result<()> {
    let url = format!("{GRAPH_BASE}/me/todo/lists/{list_id}/tasks/{task_id}");
    http.patch(url)
        .bearer_auth(access_token)
        .json(&json!({ "status": "completed" }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn delete_task(
    http: &Client,
    access_token: &str,
    list_id: &str,
    task_id: &str,
) -> anyhow::Result<()> {
    let url = format!("{GRAPH_BASE}/me/todo/lists/{list_id}/tasks/{task_id}");
    let response = http.delete(url).bearer_auth(access_token).send().await?;
    let status = response.status();

    if delete_status_is_success(status) {
        return Ok(());
    }

    response
        .error_for_status()
        .with_context(|| format!("failed to delete task {task_id} from list {list_id}"))?;
    Ok(())
}

fn delete_status_is_success(status: StatusCode) -> bool {
    status.is_success() || status == StatusCode::NOT_FOUND
}

async fn collect_pages<T>(
    http: &Client,
    access_token: &str,
    first_url: String,
) -> anyhow::Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let mut url = Some(first_url);
    let mut values = Vec::new();

    while let Some(current_url) = url {
        let result = http
            .get(current_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .error_for_status()?
            .json::<GraphCollection<T>>()
            .await?;
        values.extend(result.value);
        url = result.next_link;
    }

    Ok(values)
}

fn task_payload(task: &TaskDto) -> serde_json::Value {
    let mut payload = json!({
        "title": task.title,
        "importance": task.importance.as_deref().unwrap_or("normal"),
        "body": {
            "content": task.note.as_deref().unwrap_or(""),
            "contentType": "text"
        },
        "isReminderOn": task.is_reminder_on,
    });

    payload["dueDateTime"] = task
        .due_date_time
        .as_deref()
        .map(|value| date_time_time_zone(value, task.time_zone.as_deref()))
        .unwrap_or(serde_json::Value::Null);
    payload["reminderDateTime"] = if task.is_reminder_on {
        task.reminder_date_time
            .as_deref()
            .map(|value| date_time_time_zone(value, task.time_zone.as_deref()))
            .unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    payload["recurrence"] = task.recurrence.clone().unwrap_or(serde_json::Value::Null);

    payload
}

fn date_time_time_zone(value: &str, time_zone: Option<&str>) -> serde_json::Value {
    json!({
        "dateTime": value,
        "timeZone": time_zone.unwrap_or("UTC")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_accepts_no_content_and_not_found() {
        assert!(delete_status_is_success(StatusCode::NO_CONTENT));
        assert!(delete_status_is_success(StatusCode::NOT_FOUND));
    }

    #[test]
    fn delete_retries_auth_throttle_server_and_network_statuses() {
        assert!(!delete_status_is_success(StatusCode::UNAUTHORIZED));
        assert!(!delete_status_is_success(StatusCode::FORBIDDEN));
        assert!(!delete_status_is_success(StatusCode::TOO_MANY_REQUESTS));
        assert!(!delete_status_is_success(StatusCode::INTERNAL_SERVER_ERROR));
    }
}
