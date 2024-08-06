use reqwest::{Client, Error, Response};
use serde_json::json;
use std::sync::Arc;
use crate::task::BaseTask;


pub async fn execute_task(client: Arc<Client>, task: BaseTask) -> Result<Response, Error>{

    println!("Executing task: {}", task.name);
    let client_clone = Arc::clone(&client);
    let task_clone = task.clone();

    let res = client_clone.post(&task_clone.task)
        .json(&json!({ "task_id": task_clone.id }))
        .send()
        .await;
    res
}
