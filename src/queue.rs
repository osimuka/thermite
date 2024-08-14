use redis::AsyncCommands;
use crate::task::BaseTask;
use crate::errors::TaskQueueError;
use chrono::Utc;


pub async fn enqueue_task(client: &redis::Client, task: &BaseTask) -> Result<(), TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect to Redis");
    let task_json = serde_json::to_string(&task).expect("Failed to serialize task");

    println!("Enqueuing task: {}", task_json);

    // Add the task to the queue
    let was_set: bool = conn.zadd("task_queue", task_json, task.cron_string_to_unix_timestamp()).await?;
    if !was_set {
        println!("Task {} already exists, not enqueued again", task.id);
    } else {
        println!("Enqueued task: {}", task.id);
    }
    Ok(())
}

pub async fn dequeue_task(client: &redis::Client) -> Result<Option<BaseTask>, TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    // Get the current time as a Unix timestamp
    let now = Utc::now().timestamp() as u64;
    // get first task from the queue based on the score (timestamp)
    let task_json: Option<String> = conn
        .zrangebyscore_withscores("task_queue", "-inf", now)
        .await
        .unwrap_or_else(|_| vec![]).first().cloned();

    // Remove the task from the queue
    if let Some(task_str) = &task_json {
        let _: () = conn.zrem("task_queue", task_str).await?;
    }

    println!("Dequeued task: {}", task_json.as_deref().unwrap_or("None"));

    // Deserialize and return the task if found
    match task_json {
        Some(task_str) => {
            let task: BaseTask = serde_json::from_str(&task_str)?;
            Ok(Some(task))
        },
        None => Ok(None)
    }
}

pub async fn clear_task_queue(client: &redis::Client) -> Result<(), TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: () = conn.del("task_queue").await?;
    Ok(())
}
