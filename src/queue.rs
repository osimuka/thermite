use redis::AsyncCommands;
use crate::task::BaseTask;
use crate::errors::TaskQueueError;
use serde_json;


pub async fn enqueue_task(client: &redis::Client, task: &BaseTask) -> Result<(), TaskQueueError> {
    let mut conn = client.get_async_connection().await.expect("Failed to connect to Redis");
    let task_id_key = format!("task:{}", task.id);
    // Check if the task is already in the queue
    let exists: bool = conn.exists(&task_id_key).await.unwrap_or(false);
    if !exists {
        // set the task id key to true
        conn.set(&task_id_key, true).await?;
        let task_json = serde_json::to_string(&task).expect("Failed to serialize task");
        conn.lpush("task_queue", task_json).await?;
    }else {
        println!("Task {} already in queue", task.id);
    }
    Ok(())
}

pub async fn dequeue_task(client: &redis::Client) -> Result<Option<BaseTask>, TaskQueueError> {
    let mut con = client.get_async_connection().await?;
    let task_json: Option<String> = con.rpop("task_queue").await?;
    if let Some(task_str) = task_json {
        let task: BaseTask = serde_json::from_str(&task_str)?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}
