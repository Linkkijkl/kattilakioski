FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev libpq-dev nodejs npm
WORKDIR /app

# Backend
RUN rustup default nightly
ENV RUSTFLAGS="-C target-feature=-crt-static"
# Future debugger: add your additional Rust specific files to the following
# line. Specifying the files makes Docker use cache if they are not modified,
# skipping the tidiously long build.
COPY Cargo.lock Cargo.toml diesel.toml .
COPY src src
COPY migrations migrations
RUN cargo build --release --bin kattilakioski

# Frontend
RUN apk add --no-cache nodejs npm
RUN npm install --global yarn vite
COPY . .
RUN mkdir -p frontend/public
RUN yarn install
RUN yarn run build

# Final container
FROM alpine AS runtime
RUN apk add --no-cache libpq-dev libgcc
WORKDIR /app
COPY --from=builder /app/target/release/kattilakioski .
COPY --from=builder /app/dist dist
USER 1000
CMD [ "./kattilakioski" ]
