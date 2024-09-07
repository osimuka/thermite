use serde_json::json;
use std::sync::Mutex;
use actix_web::{web, HttpResponse, Responder};
use crate::queue;
use crate::task::BaseTask;


pub struct AppState {
    pub redis_client: redis::Client,
}

pub async fn submit_task(_data: web::Data<Mutex<AppState>>, task: web::Json<BaseTask>) -> impl Responder {
    let redis_client = _data.lock().unwrap().redis_client.clone();
    match queue::enqueue_task(&redis_client, &task.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "Task submitted"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

pub async fn submit_tasks(_data: web::Data<Mutex<AppState>>, tasks: web::Json<Vec<BaseTask>>) -> impl Responder {
    for task in tasks.into_inner() {
        let redis_client = _data.lock().unwrap().redis_client.clone();
        match queue::enqueue_task(&redis_client, &task).await {
            Ok(_) => println!("Task enqueued: {}", task.id),
            Err(e) => eprintln!("Failed to enqueue task: {}", e),
        }
    }
    HttpResponse::Ok().json(json!({"status": "Tasks submitted"}))
}

pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(json!({"error": "Not Found"}))
}
