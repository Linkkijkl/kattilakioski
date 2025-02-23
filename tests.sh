#!/bin/sh

# Check that database is reachable
pg_isready -h postgres || exit 1

# Build backend
cargo build || exit 1
# Start backend server in background 
cargo run &
sleep 0.5
# Store backend process id for later use
BACKEND_PID=$!
cargo test -- --test-threads 1 || exit 1
# Kill backend server
kill $BACKEND_PID

# Check frontend
yarn run check || exit 1
