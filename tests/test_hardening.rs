#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test as actix_test, web, App};
    use std::sync::Mutex;
    use thermite::handlers::{health_check, submit_task, AppState};
    use thermite::task::BaseTask;

    #[actix_web::test]
    async fn health_check_returns_ok() {
        let app = actix_test::init_service(
            App::new().route("/healthz", web::get().to(health_check)),
        )
        .await;

        let req = actix_test::TestRequest::get().uri("/healthz").to_request();
        let resp = actix_test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn submit_task_requires_api_key_when_configured() {
        std::env::set_var("THERMITE_API_KEY", "test-secret");

        let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(Mutex::new(AppState { redis_client })))
                .route("/submit-task", web::post().to(submit_task)),
        )
        .await;

        let payload = serde_json::json!({
            "id": "task-1",
            "name": "Test Task",
            "description": "desc",
            "category": "non_periodic",
            "priority": "high",
            "task": "http://example.com",
            "scheduled_at": 1893456000_u64,
            "cron_scheduled_at": "",
            "args": null
        });

        let req = actix_test::TestRequest::post()
            .uri("/submit-task")
            .set_json(&payload)
            .to_request();
        let resp = actix_test::call_service(&app, req).await;

        std::env::remove_var("THERMITE_API_KEY");
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn submit_task_rejects_localhost_target() {
        let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(Mutex::new(AppState { redis_client })))
                .route("/submit-task", web::post().to(submit_task)),
        )
        .await;

        let payload = serde_json::json!({
            "id": "task-ssrf",
            "name": "Blocked Task",
            "description": "desc",
            "category": "non_periodic",
            "priority": "high",
            "task": "http://127.0.0.1:8080/internal",
            "scheduled_at": 1893456000_u64,
            "cron_scheduled_at": "",
            "args": null
        });

        let req = actix_test::TestRequest::post()
            .uri("/submit-task")
            .insert_header(("x-api-key", "test-secret"))
            .set_json(&payload)
            .to_request();
        let resp = actix_test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn allowlist_enforces_configured_hosts() {
        std::env::set_var("THERMITE_ALLOWED_HOSTS", "jobs.example.com");

        let allowed_task = BaseTask {
            task: "https://jobs.example.com/hooks/run".to_string(),
            ..Default::default()
        };
        let blocked_task = BaseTask {
            task: "https://evil.example.net/hooks/run".to_string(),
            ..Default::default()
        };

        let allowed_result = allowed_task.validate_target_url();
        let blocked_result = blocked_task.validate_target_url();
        std::env::remove_var("THERMITE_ALLOWED_HOSTS");

        assert!(allowed_result.is_ok());
        assert!(blocked_result.is_err());
    }

    #[test]
    fn invalid_periodic_cron_returns_error() {
        let task = BaseTask {
            category: "periodic".to_string(),
            cron_scheduled_at: "not a valid cron".to_string(),
            ..Default::default()
        };

        assert!(task.get_next_unix_datetime().is_err());
    }
}
