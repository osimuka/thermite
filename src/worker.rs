use reqwest::{Client, Error, Response};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

use crate::task::BaseTask;

pub async fn execute_task(client: Arc<Client>, task: BaseTask) -> Result<Response, Error> {
    let task_id = task.id.clone();
    let task_name = task.name.clone();

    info!(task_id = %task_id, task_name = %task_name, "executing task");

    let response = client
        .post(&task.task)
        .json(&json!({ "task_id": task.id, "args": task.args }))
        .send()
        .await?;

    if response.status().is_success() {
        info!(task_id = %task_id, status = response.status().as_u16(), "task execution completed successfully");
        Ok(response)
    } else {
        error!(task_id = %task_id, status = response.status().as_u16(), "task execution returned a non-success status");
        response.error_for_status()
    }
}
