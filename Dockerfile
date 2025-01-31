# ----- BUILD BACKEND -----

# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature. This also leverages the docker build cache to avoid
# re-downloading dependencies if they have not changed.
FROM rust:alpine AS chef
RUN apk add --no-cache musl-dev openssl-dev libpq-dev

# Comment out if you don't need nightly
RUN rustup default nightly

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin kattilakioski

# ----- BUILD FRONTEND -----

RUN apk add --no-cache nodejs npm
RUN npm install --global yarn vite
RUN yarn install
RUN yarn run build

# ----- FINAL CONTAINER ----- 
# Copy the statically-linked binary into a scratch container
FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/kattilakioski /usr/local/bin/kattilakioski
COPY --from=builder /app/dist /usr/local/bin/dist
USER 1000
CMD [ "/usr/local/bin/kattilakioski" ]
