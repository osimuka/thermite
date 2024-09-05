use std::str::FromStr;
use chrono::Utc;
use cron::Schedule;
use serde::{Deserialize, Serialize};

type CRONShedule = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub task: String,
    pub scheduled_at: Option<u64>,
    pub cron_sheduled_at: Option<CRONShedule>,
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
    pub scheduled_at: Option<u64>,
    pub cron_sheduled_at: Option<CRONShedule>,
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

impl BaseTask {
    pub fn cron_string_to_unix_timestamp(&self) -> i64 {
        if let Some(cron) = &self.cron_sheduled_at {
            let schedule_parts: Vec<&str> = cron.split_whitespace().collect();
            if schedule_parts.len() != 5 {
                return Utc::now().timestamp();
            }

            // Insert "0" at the beginning for seconds
            let mut schedule_with_seconds: Vec<String> = schedule_parts.iter().map(|s| s.to_string()).collect();
            schedule_with_seconds.insert(0, "0".to_string());

            // Combine the parts back into a single string
            let cron_expression = schedule_with_seconds.join(" ");

            // Parse the CRON expression into a Schedule
            let schedule = match Schedule::from_str(&cron_expression) {
                Ok(s) => s,
                Err(_) => return Utc::now().timestamp(), // Default to current time on error
            };

            // Find the next time the schedule will trigger
            match schedule.upcoming(Utc).next() {
                Some(next_time) => next_time.timestamp(),
                None => Utc::now().timestamp(), // Default to current time if no next time is found
            }
        } else {
            Utc::now().timestamp()
        }
    }
}
