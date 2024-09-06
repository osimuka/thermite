use redis::AsyncCommands;
use crate::task::BaseTask;
use crate::errors::TaskQueueError;
use chrono::Utc;


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
    // Get tasks from the queue based on the score (timestamp), taking the highest score up to 'now'
    let tasks: Vec<(String, f64)> = conn.zrevrangebyscore_withscores("task_queue", now as f64, "-inf").await?;
    // Retrieve the first task in the list, which will have the highest score within the range
    Ok(tasks.into_iter().next().map(|(task, _score)| task))
}

pub async fn dequeue_task(client: &redis::Client) -> Result<Option<BaseTask>, TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect to Redis");
    // Get the current time as a Unix timestamp
    let now = Utc::now().timestamp() as u64;
    println!("Current timestamp: {}", now);
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

    // Remove the task from the queue
    let task_str_clone = task_str.take().unwrap_or_default();
    let _: () = conn.zrem("task_queue", &task_str_clone).await?;
    println!("Dequeued task: {:?}", task);
    // Return the task
    Ok(Some(task.unwrap()))
}

pub async fn clear_task_queue(client: &redis::Client) -> Result<(), TaskQueueError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: () = conn.del("task_queue").await?;
    Ok(())
}
