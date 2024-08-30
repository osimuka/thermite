#!/bin/bash

# Set default values if not provided in the environment
export REDIS_URL=${REDIS_URL:-"redis://thermite-redis-1:6379"}
export TASKS_URL=${TASKS_URL:-"thermite-worker-1:8080"}
export FETCH_URL=${FETCH_URL:-"http://host.docker.internal:8000/api/tasks/periodic"}


# Now run the main application

# Run the binary
./thermite --mode "receiver" --redis-url $REDIS_URL
