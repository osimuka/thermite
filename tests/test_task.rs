#[cfg(test)]
mod tests {
    use chrono::Utc;
    use thermite::task::BaseTask;

    #[test]
    fn test_get_next_unix_datetime_non_periodic() {
        let task = BaseTask {
            category: "non_periodic".to_string(),
            scheduled_at: 1628764800, // Unix timestamp for August 13, 2021 00:00:00 UTC
            cron_scheduled_at: "0 0 * * *".to_string(), // Cron schedule for daily at midnight
            ..Default::default()
        };

        let next_datetime = task.get_next_unix_datetime();
        assert_eq!(next_datetime, 1628764800); // Scheduled datetime should remain the same
    }

    #[test]
    fn test_get_next_unix_datetime_periodic() {
        let current = 1628764800 as i64; // Unix timestamp for August 13, 2021 00:00:00 UTC
        let task = BaseTask {
            category: "periodic".to_string(),
            scheduled_at: current as u64, // Unix timestamp for August 13, 2021 00:00:00 UTC
            cron_scheduled_at: "0 0 * * *".to_string(), // Cron schedule for daily at midnight
            ..Default::default()
        };

        let next_datetime = task.get_next_unix_datetime();
        // today at midnight

        assert!(next_datetime > Utc::now().timestamp()); // Unix timestamp for August 14, 2021 00:00:00 UTC
    }

    #[test]
    fn test_set_next_unix_datetime_non_periodic() {
        let mut task = BaseTask {
            category: "non_periodic".to_string(),
            scheduled_at: 1628764800, // Unix timestamp for August 13, 2021 00:00:00 UTC
            cron_scheduled_at: "0 0 * * *".to_string(), // Cron schedule for daily at midnight
            ..Default::default()
        };

        task.set_next_unix_datetime();
        assert_eq!(task.scheduled_at, 1628764800); // Scheduled datetime should remain the same
    }

    #[test]
    fn test_set_next_unix_datetime_periodic() {
        let mut task = BaseTask {
            category: "periodic".to_string(),
            scheduled_at: 1628764800, // Unix timestamp for August 13, 2021 00:00:00 UTC
            cron_scheduled_at: "0 0 * * *".to_string(), // Cron schedule for daily at midnight
            ..Default::default()
        };

        task.set_next_unix_datetime();
        assert!(task.scheduled_at as i64 > Utc::now().timestamp());
    }
}
