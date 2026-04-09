use reqwest::{Client, Error, Response};
use serde_json::json;
use std::sync::Arc;
use crate::task::BaseTask;


pub async fn execute_task(client: Arc<Client>, task: BaseTask) -> Result<Response, Error> {
    println!("Executing task: {}", task.name);

    client
        .post(&task.task)
        .json(&json!({ "task_id": task.id, "args": task.args }))
        .send()
        .await?
        .error_for_status()
}
