#!/bin/bash

# Set default values if not provided in the environment
export REDIS_URL=${REDIS_URL:-"redis://localhost:6379"}
export TASKS_URL=${TASKS_URL:-"0.0.0.0:8080"}

# Now run the main application
exec "$@"
