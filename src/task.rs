use std::str::FromStr;

use serde::{Deserialize, Serialize};
use chrono::Utc;
use cron::Schedule;

/// A structure holding two public integers.
///
/// Example:
///
/// ```
/// use thermite::task::BaseTask;
///
/// let task = BaseTask {
///    id: "1".to_string(),
///    name: "Task 1".to_string(),
///    description: "Task 1 description".to_string(),
///    category: "non_periodic".to_string(),
///    priority: "high".to_string(),
///    task: "http://localhost:8080/task".to_string(),
///    scheduled_at: 1628764800,
///    cron_scheduled_at: "* 0 0 * * *".to_string(),
///    args: None,
/// };
///
/// assert_eq!(task.id, "1");
/// ```
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

impl Default for BaseTask {
    fn default() -> Self {
        BaseTask {
            id: "".to_string(),
            name: "".to_string(),
            description: "".to_string(),
            category: "".to_string(),
            priority: "".to_string(),
            task: "".to_string(),
            scheduled_at: 0,
            cron_scheduled_at: "".to_string(),
            args: None,
        }
    }
}


impl BaseTask {

    /// Returns the next occurrence of a Unix datetime based on the task's cron schedule.
    /// If the task's category is not "periodic", the method returns the scheduled datetime as an i64.
    /// If the task's category is "periodic", the method parses the cron schedule and calculates the next occurrence.
    /// The cron schedule is expected to have either 5 or 6 fields separated by spaces.
    /// If the cron schedule has 5 fields, the method prepends '0' to make it a 6-field cron string.
    /// The method then creates a `Schedule` instance from the cron string and retrieves the next occurrence using the `upcoming` method.
    /// If no upcoming dates are found, the method returns an error.
    /// The next occurrence is returned as a Unix datetime (i64).
    ///
    /// # Examples
    ///
    /// ```
    /// use thermite::task::BaseTask;
    ///
    /// let task = BaseTask {
    ///   id: "1".to_string(),
    ///   name: "Task 1".to_string(),
    ///   description: "Task 1 description".to_string(),
    ///   category: "periodic".to_string(),
    ///   priority: "high".to_string(),
    ///   task: "http://localhost:8080/task".to_string(),
    ///   scheduled_at: 1628764800,
    ///   cron_scheduled_at: "* 0 0 * * *".to_string(),
    ///   args: None,
    /// };
    ///
    /// let next_datetime = task.get_next_unix_datetime();
    /// assert!(next_datetime > 1628764800);
    ///
    /// ```
    pub fn get_next_unix_datetime(&self) -> i64 {

        if self.category != "periodic" {
            return self.scheduled_at as i64;
        }
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

        println!("Next occurrence: {}", next_occurrence);

        // Return the next occurrence as DateTime<Utc>
        next_occurrence.timestamp()
    }


    /// Set the next scheduled Unix datetime based on the task's cron schedule.
    /// If the task's category is not "periodic", the method sets the scheduled_at value to the current value.
    /// If the task's category is "periodic", the method calculates the next occurrence based on the cron schedule and sets the scheduled_at value to it.
    /// The method internally calls the `get_next_unix_datetime` method to calculate the next occurrence.
    ///
    /// # Examples
    ///
    /// ```
    /// use thermite::task::BaseTask;
    ///
    /// let mut task = BaseTask {
    ///     id: "1".to_string(),
    ///     name: "Task 1".to_string(),
    ///     description: "Task 1 description".to_string(),
    ///     category: "periodic".to_string(),
    ///     priority: "high".to_string(),
    ///     task: "http://localhost:8080/task".to_string(),
    ///     scheduled_at: 1628764800,
    ///     cron_scheduled_at: "* 0 0 * * *".to_string(),
    ///     args: None,
    /// };
    ///
    /// task.set_next_unix_datetime();
    /// assert!(task.scheduled_at > 1628764800);
    /// ```
    pub fn set_next_unix_datetime(&mut self) {
        self.scheduled_at = self.get_next_unix_datetime() as u64;
    }
}
