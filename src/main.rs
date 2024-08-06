use std::env;
use redis::Client;
use serde_json::json;
use tokio::sync::mpsc;
use std::sync::Arc;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use reqwest::Client as HttpClient;

// local package imports
use thermite::task::BaseTask;
use thermite::worker;
use thermite::queue;

struct AppState {
    redis_client: Client,
}

async fn submit_task(_data: web::Data<Mutex<AppState>>, task: web::Json<BaseTask>) -> impl Responder {
    format!("Task received: {:?}", task);
    let redis_client = _data.lock().unwrap().redis_client.clone();
    match queue::enqueue_task(&redis_client, &task.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "Task submitted"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(json!({"error": "Not Found"}))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Get the Redis URL from the environment variable
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    // Create the Redis client
    let redis_client = Client::open(redis_url.as_str()).expect("Invalid Redis URL");
    // Create a new HTTP client allow for http requests
    let http_client = HttpClient::new();
    let data = web::Data::new(Mutex::new(AppState {
        redis_client: redis_client.clone(),
    }));

    let (tx, mut rx): (mpsc::Sender<BaseTask>, mpsc::Receiver<BaseTask>) = mpsc::channel(32);

    // Clear the task queue before starting/restarting the server
    queue::clear_task_queue(&redis_client).await.unwrap();

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

    // Spawning a task to process received tasks using the HTTPS client
    let cloned_http_client = Arc::new(http_client);
    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            let client = Arc::clone(&cloned_http_client);
            let handle = tokio::spawn(async move {
                let _ = worker::execute_task(client, task).await?;
                Ok::<(), reqwest::Error>(())
            });
            match handle.await {
                Ok(res) => println!("Task executed successfully {:?}", res),
                Err(e) => eprintln!("Failed to execute task: {}", e),
            }
        }
    });

    match HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/submit-task",web::post().to(submit_task))
            .default_service(web::route().to(not_found))
    })
    .bind(env::var("TASKS_URL").unwrap_or_else(|_| "127.0.0.1:8080".to_string())) {
        Ok(server) => server.run().await,
        Err(e) => {
            eprintln!("Failed to bind server: {}", e);
            return Err(e.into());
        }
    }
}
