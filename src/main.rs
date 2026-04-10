use std::env;
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::{web, App, HttpServer};
use clap::{Arg, ArgAction, Command};
use redis::Client;
use reqwest::Client as HttpClient;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

// local package imports
use thermite::task::BaseTask;
use thermite::worker;
use thermite::queue;
use thermite::handlers::{dead_letter_tasks, health_check, not_found, submit_task, submit_tasks, AppState};

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,thermite=info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .try_init();
}

fn spawn_queue_dispatcher(redis_client: Client, tx: mpsc::Sender<BaseTask>) {
    tokio::spawn(async move {
        loop {
            match queue::dequeue_task(&redis_client).await {
                Ok(Some(task)) => {
                    if tx.send(task).await.is_err() {
                        warn!("worker channel closed while dispatching task");
                        break;
                    }
                }
                Ok(None) => {
                    debug!("no tasks in the queue");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!(error = %e, "failed to dequeue task");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    });
}

fn spawn_task_processor(
    redis_client: Client,
    http_client: HttpClient,
    mut rx: mpsc::Receiver<BaseTask>,
) {
    let http_client = Arc::new(http_client);

    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            let client = Arc::clone(&http_client);
            let failure_client = redis_client.clone();
            let handle = tokio::spawn(async move {
                let failed_task = task.clone();

                match worker::execute_task(client, task).await {
                    Ok(_) => Ok::<(), reqwest::Error>(()),
                    Err(error) => {
                        if let Err(queue_error) =
                            queue::handle_task_failure(&failure_client, &failed_task, &error.to_string()).await
                        {
                            error!(
                                task_id = %failed_task.id,
                                error = %queue_error,
                                "failed to persist retry or dead-letter state"
                            );
                        }

                        Err(error)
                    }
                }
            });

            match handle.await {
                Ok(Ok(())) => info!("task executed successfully"),
                Ok(Err(e)) => error!(error = %e, "task execution failed"),
                Err(e) => error!(error = %e, "task worker join failed"),
            }
        }
    });
}

async fn start_receiver(
    redis_client: Client,
    http_client: HttpClient,
    data: web::Data<Mutex<AppState>>,
    tx: mpsc::Sender<BaseTask>,
    rx: mpsc::Receiver<BaseTask>
) -> std::io::Result<()> {

    spawn_queue_dispatcher(redis_client.clone(), tx);
    spawn_task_processor(redis_client, http_client, rx);

    let bind_address = env::var("TASKS_URL").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    info!(bind_address = %bind_address, "starting receiver HTTP server");

    match HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/healthz", web::get().to(health_check))
            .route("/dead-letter-tasks", web::get().to(dead_letter_tasks))
            .route("/submit-task",web::post().to(submit_task))
            .route("/submit-tasks",web::post().to(submit_tasks))
            .default_service(web::route().to(not_found))
    })
    .bind(&bind_address) {
        Ok(server) => server.run().await,
        Err(e) => {
            error!(error = %e, bind_address = %bind_address, "failed to bind server");
            Err(e)
        }
    }
}

async fn start_fetcher(
    redis_client: Client,
    http_client: HttpClient,
    data: web::Data<Mutex<AppState>>,
    tx: mpsc::Sender<BaseTask>,
    rx: mpsc::Receiver<BaseTask>
) -> std::io::Result<()> {

    // Get the URL to fetch tasks from
    let fetch_url = env::var("FETCH_URL").map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("FETCH_URL must be set: {e}"),
        )
    })?;
    info!("starting fetcher loop");

    spawn_queue_dispatcher(redis_client.clone(), tx);
    spawn_task_processor(redis_client, http_client.clone(), rx);

    // Fetch tasks from the URL and enqueue them
    // Spawning a task to fetch tasks from the given URL every second
    loop {
        let res = http_client.get(fetch_url.as_str()).send().await;
        match res {
            Ok(res) => {
                if let Ok(tasks) = res.json::<Vec<BaseTask>>().await {
                    for task in tasks {
                        let redis_client = match data.lock() {
                            Ok(state) => state.redis_client.clone(),
                            Err(e) => {
                                error!(error = %e, "application state unavailable while enqueueing fetched tasks");
                                continue;
                            }
                        };
                        match queue::enqueue_task(&redis_client, &task).await {
                            Ok(_) => info!(task_id = %task.id, "task enqueued from fetcher"),
                            Err(e) => warn!(task_id = %task.id, error = %e, "failed to enqueue fetched task"),
                        }
                    }
                }
            }
            Err(e) => warn!(error = %e, "failed to fetch tasks"),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}


fn cli() -> Command {
    Command::new("thermite")
        .version("1.0")
        .author("Uka Osim <hsojo91@gmail.com>")
        .about("Runs tasks either as a receiver or fetcher")
        .arg(Arg::new("mode")
            .short('m')
            .long("mode")
            .help("Sets the operation mode of the application")
            .action(ArgAction::Set)
            .value_name("MODE")
            .value_parser(["receiver", "fetcher"])
            .default_value("receiver")
            .required(true))
        .arg(Arg::new("redis-url")
            .short('r')
            .long("redis-url")
            .help("Sets the Redis server URL")
            .action(ArgAction::Set)
            .value_name("REDIS_URL")
            // .required(true)  // If you want this to be always required
            .default_value("redis://localhost:6379"))
        .arg(Arg::new("tasks-url")
            .short('t')
            .long("tasks-url")
            .help("Sets the URL to listen for tasks")
            .action(ArgAction::Set)
            .value_name("TASKS_URL")
            .requires_if("receiver", "mode")
            .default_value("localhost:8080"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let matches = cli().get_matches();

    let mode = matches.get_one::<String>("mode").unwrap();
    info!(mode = %mode, "starting thermite");

    let default_redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_url = matches.get_one::<String>("redis-url").unwrap_or(&default_redis_url);

    // Create the Redis client
    let redis_client = Client::open(redis_url.as_str()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid Redis URL: {e}"),
        )
    })?;
    // Create a new HTTP client allow for http requests
    let http_client = HttpClient::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| std::io::Error::other(format!("Failed to build HTTP client: {e}")))?;
    let data = web::Data::new(Mutex::new(AppState {
        redis_client: redis_client.clone(),
    }));

    let (tx, rx): (mpsc::Sender<BaseTask>, mpsc::Receiver<BaseTask>) = mpsc::channel(32);

    if mode == "receiver" {
        let _ = start_receiver(redis_client, http_client, data, tx, rx).await;
    } else if mode == "fetcher" {
        let _ = start_fetcher(redis_client, http_client, data, tx, rx).await;
    } else {
        error!(mode = %mode, "invalid APP_MODE; must be 'receiver' or 'fetcher'");
    }
    Ok(())
}
