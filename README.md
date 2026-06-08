# Thermite

A high-performance task queue worker and scheduler written in Rust. Thermite processes distributed tasks from a Redis queue with support for both one-time and periodic (cron-scheduled) task execution.

## Features

- **Two Operating Modes:**
  - **Receiver Mode**: Listens for incoming tasks via HTTP POST endpoints and queues them in Redis
  - **Fetcher Mode**: Periodically fetches tasks from an external source and processes them
  
- **Task Management:**
  - Support for one-time and periodic (cron-scheduled) tasks
  - Task prioritization and categorization
  - Asynchronous task execution with configurable arguments
  - FIFO queue processing via Redis

- **Architecture:**
  - Built with Actix-web for high-performance HTTP handling
  - Redis-backed distributed task queue
  - Tokio async runtime for concurrent task execution
  - Cron expression parsing and scheduling support
  - RESTful API for task submission

## API Endpoints

- `POST /submit-task` - Submit a single task
- `POST /submit-tasks` - Submit multiple tasks in batch

## Configuration

The application can be configured via environment variables or CLI arguments:

- `REDIS_URL` - Redis server connection URL (default: `redis://localhost:6379`)
- `TASKS_URL` - HTTP server binding address (default: `127.0.0.1:8080`)
- `FETCH_URL` - External URL to fetch tasks from (required in fetcher mode)

## Running Thermite

```bash
# Receiver mode (listen for HTTP requests)
cargo run -- --mode receiver --redis-url redis://localhost:6379 --tasks-url 0.0.0.0:8080

# Fetcher mode (periodically fetch from external source)
cargo run -- --mode fetcher --redis-url redis://localhost:6379
```

## Heroku Container Deployment Steps

1. Install Heroku CLI
2. Login to Heroku
3. Login to Heroku Container Registry

```bash
heroku login
heroku container:login
```

4. Create a Heroku app

```bash
heroku create thermite
```

5. Build the Docker image

```bash
docker build --platform linux/amd64 -t registry.heroku.com/thermite/worker .
```

6. Push the Docker image to Heroku Container Registry

```bash
docker push registry.heroku.com/thermite/worker
```

7. Set the stack to container

```bash
heroku stack:set container -a thermite
```

7. Release the Docker image

```bash
heroku container:release worker -a thermite
```

8. Open the Heroku app

```bash
heroku open -a thermite
```

9. View the logs

```bash
heroku logs --tail -a thermite
```
