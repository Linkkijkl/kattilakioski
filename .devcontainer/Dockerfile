# https://github.com/devcontainers/images/tree/main/src/rust

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="bullseye"
FROM rust:1-${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    # Remove imagemagick due to https://security-tracker.debian.org/tracker/CVE-2019-10131
    && apt-get purge -y imagemagick imagemagick-6-common

# Install cargo watch
# (commented out because it's currently not building)
#RUN cargo install cargo-watch

# Install Diesel cli tools
RUN cargo install diesel_cli

# Install clippy and rustfmt
RUN rustup component add clippy-preview
RUN rustup component add rustfmt

# Install postgres cli
RUN apt-get install -y postgresql-client

# Install node and yarn
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && apt install nodejs -y
RUN npm install --global yarn vite
