use reqwest::Client;
use serde_json::json;
use tokio::task as tokio_task;
use std::sync::Arc;
use crate::task::BaseTask;


pub async fn execute_task(client: Arc<Client>, task: BaseTask) {
    let client_clone = Arc::clone(&client);
    let task_clone = task.clone();

    tokio_task::spawn(async move {
        let res = client_clone.post(&task_clone.task)
            .json(&json!({ "task_id": task_clone.id }))
            .send()
            .await;

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Successfully executed task: {}", task_clone.name);

                    // Print additional information if the task is periodic
                    if let Some(periodic_details) = task_clone.periodic_details {
                        println!("Task is periodic with interval: {:?}", periodic_details.interval);
                        println!("Last run: {:?}", periodic_details.last_run);
                        println!("Next run: {}", periodic_details.next_run);
                    }
                } else {
                    println!("Failed to execute task: {}: {:?}", task_clone.name, response.status());
                }
            },
            Err(err) => {
                println!("Error sending request for task {}: {:?}", task_clone.name, err);
            }
        }
    }).await.expect("Task execution failed");
}
