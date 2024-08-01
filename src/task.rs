use serde::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub status: String,
    pub task: String,
    pub periodic_details: Option<PeriodicDetails>
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Interval {
    Daily,
    Weekly,
    Monthly,
}

impl Interval {
    fn as_str(&self) -> &'static str {
        match self {
            Interval::Daily => "daily",
            Interval::Weekly => "weekly",
            Interval::Monthly => "monthly",
        }
    }

    fn to_duration(&self) -> Duration {
        match self {
            Interval::Daily => Duration::from_secs(60 * 60 * 24),
            Interval::Weekly => Duration::from_secs(60 * 60 * 24 * 7),
            Interval::Monthly => Duration::from_secs(60 * 60 * 24 * 30), // Approximate duration for a month
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeriodicDetails {
    pub interval: Interval,
    pub last_run: Option<String>,
    pub next_run: String,
}


// Define a struct to match the incoming JSON payload
#[derive(Serialize, Deserialize, Debug)]
pub struct BaseTaskPayload {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub status: String,
    pub task: String,
    pub interval: Option<Interval>,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
}

impl From<BaseTaskPayload> for BaseTask {
    fn from(payload: BaseTaskPayload) -> Self {
        let periodic_details = if let (Some(interval), Some(next_run)) = (payload.interval, payload.next_run) {
            Some(PeriodicDetails {
                interval,
                last_run: payload.last_run,
                next_run,
            })
        } else {
            None
        };

        BaseTask {
            id: payload.id,
            name: payload.name,
            description: payload.description,
            category: payload.category,
            priority: payload.priority,
            status: payload.status,
            task: payload.task,
            periodic_details
        }
    }
}
