use serde::{Deserialize, Serialize};
use astrolabe::{CronSchedule as Schedule, DateUtilities};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub task: String,
    pub scheduled_at: u64,
    pub cron_sheduled_at: String,
    pub args: Option<std::collections::HashMap<String, serde_json::Value>>,
}


// Define a struct to match the incoming JSON payload
#[derive(Serialize, Deserialize, Debug)]
pub struct BaseTaskPayload {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub task: String,
    pub scheduled_at: u64,
    pub cron_sheduled_at: String,
    pub args: Option<std::collections::HashMap<String, serde_json::Value>>,

}

impl From<BaseTaskPayload> for BaseTask {
    fn from(payload: BaseTaskPayload) -> Self {
        BaseTask {
            id: payload.id,
            name: payload.name,
            description: payload.description,
            category: payload.category,
            priority: payload.priority,
            task: payload.task,
            scheduled_at: payload.scheduled_at,
            cron_sheduled_at: payload.cron_sheduled_at,
            args: payload.args,
        }
    }
}

/// Converts a cron string to a Unix timestamp.
///
/// # Returns
///
/// The Unix timestamp representing the next occurrence of the cron schedule.
///
/// # Example
///
/// ```
/// use thermite::task::BaseTask;
///
/// let task = BaseTask::new();
/// let timestamp = task.cron_string_to_unix_timestamp();
/// println!("Next occurrence: {}", timestamp);
/// ```
impl BaseTask {
    pub fn cron_string_to_unix_timestamp(&self) -> i64 {
        let cron_schedule = self.cron_sheduled_at.as_str();
        let mut schedule = Schedule::parse(cron_schedule).unwrap();
        let next = schedule.next().unwrap();
        next.timestamp()
    }
}
