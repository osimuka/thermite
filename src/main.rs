mod task;
mod worker;
mod queue;
mod errors;

use std::env;
use redis::Client;
use serde_json::json;
use tokio::sync::mpsc;
use std::sync::Arc;
use task::BaseTask;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use reqwest::Client as HttpClient; // Import reqwest Client

struct AppState {
    redis_client: Client,
}

async fn submit_task(data: web::Data<Mutex<AppState>>, task: web::Json<BaseTask>) -> impl Responder {
    let redis_client = data.lock().unwrap().redis_client.clone();
    match queue::enqueue_task(&redis_client, &task.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "Task submitted"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Get the Redis URL from the environment variable
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    // Create the Redis client
    let redis_client = Client::open(redis_url.as_str()).expect("Invalid Redis URL");
    let http_client = HttpClient::new(); // Create a new HTTP client
    let data = web::Data::new(Mutex::new(AppState {
        redis_client: redis_client.clone(),
    }));

    let (tx, mut rx): (mpsc::Sender<BaseTask>, mpsc::Receiver<BaseTask>) = mpsc::channel(32);

    // Spawning a task to process received tasks using the HTTPS client
    let cloned_http_client = Arc::new(http_client);
    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            let client = Arc::clone(&cloned_http_client);
            tokio::spawn(async move {
                worker::execute_task(client, task).await; // Updated to use HTTPS client
            });
        }
    });

    // Spawning a task to fetch tasks from the Redis queue
    tokio::spawn(async move {
        loop {
            if let Ok(Some(task)) = queue::dequeue_task(&redis_client).await {
                tx.send(task).await.unwrap();
            } else {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route(&env::var("TASKS_URI").unwrap_or_else(|_| "/submit-task".to_string()), web::post().to(submit_task))
    })
    .bind(env::var("TASKS_URL").unwrap_or_else(|_| "127.0.0.1:8080".to_string()))?
    .run()
    .await
}
