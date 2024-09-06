use redis::AsyncCommands;
use crate::task::BaseTask;
use crate::errors::TaskQueueError;
use chrono::{Utc, Local};


pub async fn enqueue_task(client: &redis::Client, task: &BaseTask) -> Result<(), TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect to Redis");
    let task_json = serde_json::to_string(&task).expect("Failed to serialize task");

    println!("Enqueuing task: {}", task_json);

    // Add the task to the queue
    let was_set: bool = conn.zadd("task_queue", task_json, task.scheduled_at).await?;
    if !was_set {
        println!("Task {} already exists, not enqueued again", task.id);
    } else {
        println!("Enqueued task: {}", task.id);
    }
    Ok(())
}

async fn get_task(mut conn: redis::aio::MultiplexedConnection, now: i64) -> redis::RedisResult<Option<String>> {
    let tasks: Vec<(String, f64)> = conn.zrangebyscore_withscores("task_queue", "-inf", now as f64).await?;
    Ok(tasks.into_iter().next().map(|(task, _score)| task))
}

pub async fn dequeue_task(client: &redis::Client) -> Result<Option<BaseTask>, TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect to Redis");
    // Get the current time as a Unix timestamp
    let now = Utc::now().timestamp() as u64;
    // get first task from the queue based on the score (timestamp)
    let mut task_str = get_task(conn.clone(), now as i64).await?;
    let task: Option<BaseTask> = match task_str {
        Some(ref task_str) => {
            println!("Task string: {:?}", task_str);
            let task: BaseTask = serde_json::from_str(&task_str).expect("Failed to deserialize task");
            Some(task)
        }
        None => None,
    };

    println!("Task: {:?}", task);

    if task.is_none() {
        return Ok(None)
    }

    // check if task is less than or equal to the current time
    let task_time = task.as_ref().unwrap().cron_string_to_unix_datetime();
    let local_time = Local::now();

    println!("Task time: {}", task_time.to_string());
    println!("Now time: {}", local_time.to_string());

    // check if task is ready to be executed
    if task_time <= local_time {
        // Remove the task from the queue
        let task_str_clone = task_str.take().unwrap_or_default();
        let _: () = conn.zrem("task_queue", &task_str_clone).await?;
        println!("Dequeued task: {:?}", task);
        // Return the task
        return Ok(Some(task.unwrap()))
    }
    // Return None if the task is not ready to be executed
    Ok(None)
}

pub async fn clear_task_queue(client: &redis::Client) -> Result<(), TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: () = conn.del("task_queue").await?;
    Ok(())
}
