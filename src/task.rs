use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: String,
    pub language: String,
    pub script: String,
    pub notification_email: Option<String>,
    pub callback_url: Option<String>,
}
