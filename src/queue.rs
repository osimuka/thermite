use chrono::Utc;
use redis::AsyncCommands;
use tracing::{debug, info};

use crate::errors::TaskQueueError;
use crate::task::BaseTask;


pub async fn enqueue_task(client: &redis::Client, task: &BaseTask) -> Result<(), TaskQueueError> {
    task.validate()?;

    let mut conn = client.get_multiplexed_async_connection().await?;
    let task_json = serde_json::to_string(task)?;

    info!(task_id = %task.id, scheduled_at = task.scheduled_at, category = %task.category, "enqueuing task");

    let was_set: bool = conn.zadd("task_queue", task_json, task.scheduled_at).await?;
    if !was_set {
        info!(task_id = %task.id, "task already existed in queue");
    } else {
        info!(task_id = %task.id, "task enqueued");
    }
    Ok(())
}

async fn get_task(mut conn: redis::aio::MultiplexedConnection, now: i64) -> redis::RedisResult<Option<String>> {
    // Get tasks from the queue based on the score (timestamp), taking the highest score up to 'now'
    let tasks: Vec<(String, f64)> = conn.zrevrangebyscore_withscores("task_queue", now as f64, "-inf").await?;
    // Retrieve the first task in the list, which will have the highest score within the range
    Ok(tasks.into_iter().next().map(|(task, _score)| task))
}

pub async fn dequeue_task(client: &redis::Client) -> Result<Option<BaseTask>, TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let now = Utc::now().timestamp() as u64;
    debug!(now, "checking queue for due tasks");

    let task_str = match get_task(conn.clone(), now as i64).await? {
        Some(task_str) => task_str,
        None => return Ok(None),
    };

    let task: BaseTask = serde_json::from_str(&task_str)?;

    let _: () = conn.zrem("task_queue", &task_str).await?;
    info!(task_id = %task.id, category = %task.category, "dequeued task");

    if task.category == "periodic" {
        let mut rescheduled_task = task.clone();
        rescheduled_task.set_next_unix_datetime()?;
        let task_json = serde_json::to_string(&rescheduled_task)?;
        let _: () = conn.zadd("task_queue", task_json, rescheduled_task.scheduled_at).await?;
        info!(task_id = %rescheduled_task.id, next_scheduled_at = rescheduled_task.scheduled_at, "rescheduled periodic task");
    }

    Ok(Some(task))
}

pub async fn clear_task_queue(client: &redis::Client) -> Result<(), TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: () = conn.del("task_queue").await?;
    Ok(())
}
