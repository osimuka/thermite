#!/bin/bash

# Set default values if not provided in the environment
export REDIS_URL=${REDIS_URL:-"redis://thermite-redis-1:6379"}
export THERMITE_TASKS_URL=${THERMITE_TASKS_URL:-"thermite-worker-1:8080"}
export THERMITE_FETCH_URL=${THERMITE_FETCH_URL:-"http://host.docker.internal:8000/api/tasks/periodic"}
export THERMITE_MODE=${THERMITE_MODE:-"receiver"}


# Now run the main application
exec "$@"
