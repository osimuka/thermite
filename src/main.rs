use std::env;
use redis::Client;
use tokio::sync::mpsc;
use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use std::sync::Mutex;
use reqwest::Client as HttpClient;

// local package imports
use thermite::task::BaseTask;
use thermite::worker;
use thermite::queue;
use thermite::handlers::{AppState, submit_task, submit_tasks, not_found};


async fn start_receiver(redis_client: Client, http_client: HttpClient, data: web::Data<Mutex<AppState>>, tx: mpsc::Sender<BaseTask>, mut rx: mpsc::Receiver<BaseTask>) -> std::io::Result<()> {
    // Clear the task queue before starting/restarting the server
    let _ = queue::clear_task_queue(&redis_client).await;

    // Spawning a task to fetch tasks from the Redis queue
    tokio::spawn(async move {
        loop {
            if let Ok(Some(task)) = queue::dequeue_task(&redis_client).await {
                tx.send(task).await.unwrap_or_default();
            } else {
                // Sleep for a second if there are no tasks in the queue
                println!("No tasks in the queue");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    });

    // Spawning a task to process received tasks using the HTTPS client
    let cloned_http_client = Arc::new(http_client.clone());
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
            .route("/submit-tasks",web::post().to(submit_tasks))
            .default_service(web::route().to(not_found))
    })
    .bind(env::var("THERMITE_TASKS_URL").unwrap_or_else(|_| "127.0.0.1:8080".to_string())) {
        Ok(server) => server.run().await,
        Err(e) => {
            eprintln!("Failed to bind server: {}", e);
            Err(e)
        }
    }
}

async fn start_fetcher(redis_client: Client, http_client: HttpClient, data: web::Data<Mutex<AppState>>, tx: mpsc::Sender<BaseTask>, mut rx: mpsc::Receiver<BaseTask>) -> std::io::Result<()> {
    // Get the URL to fetch tasks from
    let fetch_url = env::var("THERMITE_FETCH_URL").expect("THERMITE_FETCH_URL must be set");

    // Clear the task queue before starting/restarting the server
    let _ = queue::clear_task_queue(&redis_client).await;

    // Spawning a task to fetch tasks from the Redis queue
    tokio::spawn(async move {
        loop {
            if let Ok(Some(task)) = queue::dequeue_task(&redis_client).await {
                tx.send(task).await.unwrap_or_default();
            } else {
                // Sleep for a second if there are no tasks in the queue
                println!("No tasks in the queue");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    });

    // Spawning a task to process received tasks using the HTTPS client
    let cloned_http_client = Arc::new(http_client.clone());
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

    // Fetch tasks from the URL and enqueue them
    // Spawning a task to fetch tasks from the given URL every second
    loop {
        let res = http_client.get(fetch_url.as_str()).send().await;
        match res {
            Ok(res) => {
                if let Ok(tasks) = res.json::<Vec<BaseTask>>().await {
                    for task in tasks {
                        let redis_client = data.lock().unwrap().redis_client.clone();
                        match queue::enqueue_task(&redis_client, &task).await {
                            Ok(_) => println!("Task enqueued: {}", task.id),
                            Err(e) => eprintln!("Failed to enqueue task: {}", e),
                        }
                    }
                }
            }
            Err(e) => println!("Failed to fetch tasks: {}", e),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Get the mode from the environment variable
    let mode = env::var("THERMITE_MODE").unwrap_or_else(|_| "receiver".to_string());
    // Get the Redis URL from the environment variable
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    // Create the Redis client
    let redis_client = Client::open(redis_url.as_str()).expect("Invalid Redis URL");
    // Create a new HTTP client allow for http requests
    let http_client = HttpClient::new();
    let data = web::Data::new(Mutex::new(AppState {
        redis_client: redis_client.clone(),
    }));

    let (tx, rx): (mpsc::Sender<BaseTask>, mpsc::Receiver<BaseTask>) = mpsc::channel(32);

    if mode == "receiver" {
        let _ = start_receiver(redis_client, http_client, data, tx, rx).await;
    } else if mode == "fetcher" {
        let _ = start_fetcher(redis_client, http_client, data, tx, rx).await;
    } else {
        eprintln!("Invalid APP_MODE. Must be 'receiver' or 'fetcher'.");
    }
    Ok(())
}
