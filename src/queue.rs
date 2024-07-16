use redis::AsyncCommands;
use crate::task::Task;
use crate::errors::TaskQueueError;
use serde_json;


pub async fn enqueue_task(client: &redis::Client, task: &Task) -> Result<(), TaskQueueError> {
    let mut con = client.get_async_connection().await?;
    let task_json = serde_json::to_string(task)?;
    con.lpush("task_queue", task_json).await?;
    Ok(())
}

pub async fn dequeue_task(client: &redis::Client) -> Result<Option<Task>, TaskQueueError> {
    let mut con = client.get_async_connection().await?;
    let task_json: Option<String> = con.rpop("task_queue").await?;
    if let Some(task_str) = task_json {
        let task: Task = serde_json::from_str(&task_str)?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}
