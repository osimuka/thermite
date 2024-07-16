mod task;
mod worker;
mod notification;
mod queue;
mod errors;

use notification::{call_api, send_email};
use redis::Client;
use serde_json::json;
use tokio::sync::mpsc;
use worker::execute_task;
use task::Task;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;

struct AppState {
    redis_client: Client,
}

async fn submit_task(data: web::Data<Mutex<AppState>>, task: web::Json<Task>) -> impl Responder {
    let client = data.lock().unwrap().redis_client.clone();
    match queue::enqueue_task(&client, &task.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "Task submitted"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let redis_client = Client::open("redis://127.0.0.1/").expect("Invalid Redis URL");
    let data = web::Data::new(Mutex::new(AppState { redis_client: redis_client.clone() }));

    let (tx, mut rx) = mpsc::channel(32);

    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            let result = execute_task(&task);
            match result {
                Ok(output) => {
                    if let Some(email) = &task.notification_email {
                        send_email(email, "Task Completed", &output);
                    }
                    if let Some(url) = &task.callback_url {
                        call_api(url, &output).await;
                    }
                },
                Err(err) => println!("Task failed: {}", err),
            }
        }
    });

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
            .route("/submit_task", web::post().to(submit_task))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
