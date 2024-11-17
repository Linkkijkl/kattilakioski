# https://github.com/devcontainers/images/tree/main/src/rust

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="bullseye"
FROM rust:1-${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    # Remove imagemagick due to https://security-tracker.debian.org/tracker/CVE-2019-10131
    && apt-get purge -y imagemagick imagemagick-6-common

# Install cargo watch
RUN cargo install cargo-watch

# Install Diesel cli tools
RUN cargo install diesel_cli

# Install clippy
RUN rustup component add clippy-preview

# Install postgres cli
RUN apt-get install -y postgresql-client
