# thermite

A lightweight **Redis-backed task scheduler and HTTP worker** written in Rust.

Thermite accepts tasks over HTTP or fetches them from another service, stores them in Redis, and executes them when they are due by sending an HTTP `POST` request to the target task URL. It also supports **periodic jobs** using cron expressions.

## What this tool does

Thermite runs in one of two modes:

- **`receiver` mode**: exposes an HTTP API for submitting one or more tasks.
- **`fetcher` mode**: periodically pulls tasks from a remote endpoint and enqueues them.

Once a task is in Redis, Thermite:

1. stores it in a sorted set using `scheduled_at` as the score,
2. polls for due tasks,
3. executes each due task by `POST`ing to its `task` URL,
4. re-enqueues periodic tasks with their next cron-based run time.

## Task model

Each task contains:

| Field | Description |
|---|---|
| `id` | Unique task identifier |
| `name` | Human-readable task name |
| `description` | Task description |
| `category` | `non_periodic` or `periodic` |
| `priority` | Priority label |
| `task` | Target URL to call when the task runs |
| `scheduled_at` | Unix timestamp for the next run |
| `cron_scheduled_at` | Cron expression used for periodic jobs |
| `args` | Optional JSON payload passed through to the target URL |

When a task is executed, Thermite sends a request like:

```json
{
  "task_id": "123",
  "args": {
    "email": "user@example.com"
  }
}
```

## HTTP API

### `POST /submit-task`
Submit a single task.

### `POST /submit-tasks`
Submit multiple tasks in one request.

### Example task payload

```json
{
  "id": "task-1",
  "name": "Send reminder",
  "description": "Send a reminder email",
  "category": "non_periodic",
  "priority": "high",
  "task": "http://localhost:9000/jobs/reminder",
  "scheduled_at": 1893456000,
  "cron_scheduled_at": "0 0 * * *",
  "args": {
    "user_id": 42,
    "channel": "email"
  }
}
```

## Running locally

### With Docker Compose

```bash
docker compose up --build
```

This starts:

- the Thermite worker on `http://localhost:8080`
- Redis on `localhost:6379`

### With Cargo

```bash
cargo run -- --mode receiver --redis-url redis://localhost:6379
```

To run in fetcher mode:

```bash
export FETCH_URL=http://localhost:8000/api/tasks/periodic
cargo run -- --mode fetcher --redis-url redis://localhost:6379
```

## Configuration

Thermite uses these environment variables and CLI options:

| Name | Purpose | Default |
|---|---|---|
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `TASKS_URL` | Address the HTTP server binds to in `receiver` mode | `127.0.0.1:8080` |
| `FETCH_URL` | Endpoint to poll for tasks in `fetcher` mode | required for fetcher |
| `THERMITE_API_KEY` | Optional API key required on `POST /submit-task` and `POST /submit-tasks` via `x-api-key` or `Authorization: Bearer ...` | unset |
| `--mode` | Run mode: `receiver` or `fetcher` | `receiver` |

## Typical workflow

1. Submit or fetch tasks.
2. Thermite stores them in Redis.
3. When `scheduled_at` is due, Thermite executes the target URL.
4. If the task is `periodic`, it computes the next run from `cron_scheduled_at` and requeues it.

## Heroku container deployment

1. Install the Heroku CLI and log in:

```bash
heroku login
heroku container:login
```

2. Create the app:

```bash
heroku create thermite
```

3. Build and push the image:

```bash
docker build --platform linux/amd64 -t registry.heroku.com/thermite/worker .
docker push registry.heroku.com/thermite/worker
```

4. Set the stack and release:

```bash
heroku stack:set container -a thermite
heroku container:release worker -a thermite
```

5. Open the app or inspect logs:

```bash
heroku open -a thermite
heroku logs --tail -a thermite
```
