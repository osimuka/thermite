use std::str::FromStr;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use cron::Schedule;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub task: String,
    pub scheduled_at: u64,
    pub cron_scheduled_at: String,
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
    pub cron_scheduled_at: String,
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
            cron_scheduled_at: payload.cron_scheduled_at,
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
    pub fn cron_string_to_unix_datetime(&self) -> DateTime<Utc> {
        let mut cron_schedule = self.cron_scheduled_at.as_str();

        println!("Original Cron schedule: {}", cron_schedule);

        // Check if the cron string has only 5 fields (assumes space-separated fields)
        let modified_cron_schedule = if cron_schedule.trim().split_whitespace().count() == 5 {
            // Prepend '0 ' to make it a 6-field cron string
            format!("0 {}", cron_schedule)
        } else {
            cron_schedule.to_owned()
        };
        cron_schedule = &modified_cron_schedule;
        println!("Modified Cron schedule for compatibility: {}", cron_schedule);

        // Create a Schedule instance from the cron string
        let schedule = Schedule::from_str(cron_schedule).expect("Failed to parse CRON expression");

        // Get the next occurrence from the schedule
        let next_occurrence = schedule.upcoming(Utc).next().expect("No upcoming dates found");

        // Return the next occurrence as DateTime<Utc>
        next_occurrence
    }
}
