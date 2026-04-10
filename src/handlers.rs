use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Mutex;
use tracing::{error, info, warn};

use crate::errors::TaskQueueError;
use crate::queue;
use crate::task::BaseTask;

pub struct AppState {
    pub redis_client: redis::Client,
}

fn authorize_request(req: &HttpRequest) -> Result<(), HttpResponse> {
    let configured_api_key = match std::env::var("THERMITE_API_KEY") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return Ok(()),
    };

    let provided_api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .or_else(|| {
            req.headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
        });

    match provided_api_key {
        Some(value) if value == configured_api_key => Ok(()),
        _ => {
            warn!(path = %req.path(), "unauthorized request rejected");
            Err(HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"})))
        }
    }
}

fn task_error_response(error: TaskQueueError) -> HttpResponse {
    match error {
        TaskQueueError::InvalidCronExpression(_) | TaskQueueError::InvalidTaskTarget(_) => {
            warn!(error = %error, "task request validation failed");
            HttpResponse::BadRequest().json(json!({"error": error.to_string()}))
        }
        _ => {
            error!(error = %error, "task request failed due to a server-side issue");
            HttpResponse::InternalServerError().json(json!({"error": error.to_string()}))
        }
    }
}

pub async fn submit_task(
    req: HttpRequest,
    data: web::Data<Mutex<AppState>>,
    task: web::Json<BaseTask>,
) -> impl Responder {
    if let Err(response) = authorize_request(&req) {
        return response;
    }

    let redis_client = match data.lock() {
        Ok(state) => state.redis_client.clone(),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Application state unavailable: {e}")}));
        }
    };

    let task = task.into_inner();
    info!(task_id = %task.id, category = %task.category, path = %req.path(), "received task submission");

    match queue::enqueue_task(&redis_client, &task).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "Task submitted"})),
        Err(e) => task_error_response(e),
    }
}

pub async fn submit_tasks(
    req: HttpRequest,
    data: web::Data<Mutex<AppState>>,
    tasks: web::Json<Vec<BaseTask>>,
) -> impl Responder {
    if let Err(response) = authorize_request(&req) {
        return response;
    }

    let redis_client = match data.lock() {
        Ok(state) => state.redis_client.clone(),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Application state unavailable: {e}")}));
        }
    };

    let tasks = tasks.into_inner();
    info!(count = tasks.len(), path = %req.path(), "received batch task submission");

    let mut submitted = 0usize;
    let mut failures = Vec::new();
    let mut has_server_error = false;

    for task in tasks {
        match queue::enqueue_task(&redis_client, &task).await {
            Ok(_) => {
                info!(task_id = %task.id, "task enqueued from batch request");
                submitted += 1;
            }
            Err(e) => {
                warn!(task_id = %task.id, error = %e, "failed to enqueue task from batch request");
                if !matches!(e, TaskQueueError::InvalidCronExpression(_) | TaskQueueError::InvalidTaskTarget(_)) {
                    has_server_error = true;
                }
                failures.push(json!({"id": task.id, "error": e.to_string()}));
            }
        }
    }

    if failures.is_empty() {
        HttpResponse::Ok().json(json!({"status": "Tasks submitted", "submitted": submitted}))
    } else if has_server_error {
        HttpResponse::InternalServerError().json(json!({
            "status": "Some tasks failed",
            "submitted": submitted,
            "failed": failures
        }))
    } else {
        HttpResponse::BadRequest().json(json!({
            "status": "Some tasks failed validation",
            "submitted": submitted,
            "failed": failures
        }))
    }
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": "ok"}))
}

pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(json!({"error": "Not Found"}))
}
