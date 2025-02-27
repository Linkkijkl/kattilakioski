#!/bin/sh

# Ensure the database is reachable
pg_isready -h postgres || exit 1

# Build backend
cargo build || exit 1

# Start backend server in background
cargo run &
# Store backend process id for later use
BACKEND_PID=$!
# Allow some time for the backend to start
sleep 1

# Wrapper function for backend tests, for stopping the backend process even if they fail
backend_tests() {
    cargo test -- --test-threads 1 || return 1
}
backend_tests
BACKEND_TESTS_STATUS=$?

# Stop backend server
kill $BACKEND_PID
if [ "$BACKEND_TESTS_STATUS" -ne 0 ]; then
    exit 1
fi

# Check frontend
yarn run check || exit 1
