use std::str::FromStr;
use chrono::Utc;
use cron::Schedule;
use serde::{Deserialize, Serialize};
use astrolabe::CronSchedule;

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
        let cron_schedule = self.cron_sheduled_at.as_ref().unwrap();
        let schedule = Schedule::from_str(cron_schedule).unwrap();
        let next = schedule.upcoming(Utc).next().unwrap();
        next.timestamp()
    }
}
