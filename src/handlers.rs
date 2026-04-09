use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Mutex;

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
        _ => Err(HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))),
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

    match queue::enqueue_task(&redis_client, &task.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "Task submitted"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
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

    let mut submitted = 0usize;
    let mut failures = Vec::new();

    for task in tasks.into_inner() {
        match queue::enqueue_task(&redis_client, &task).await {
            Ok(_) => {
                println!("Task enqueued: {}", task.id);
                submitted += 1;
            }
            Err(e) => {
                eprintln!("Failed to enqueue task {}: {}", task.id, e);
                failures.push(json!({"id": task.id, "error": e.to_string()}));
            }
        }
    }

    if failures.is_empty() {
        HttpResponse::Ok().json(json!({"status": "Tasks submitted", "submitted": submitted}))
    } else {
        HttpResponse::InternalServerError().json(json!({
            "status": "Some tasks failed",
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
