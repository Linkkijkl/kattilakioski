# Kattilakioski

Quick readme

## Deploying in Linux environment
Run `./deploy.sh`. [Docker](https://www.docker.com/) and [Docker compose](https://docs.docker.com/compose/) required.

## Development

Get up and running right away, without installing or configuring much locally, with [Development Containers](https://containers.dev/). Install Docker, Docker Compose and VSCode. In VSCode, install [VSCode Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) and open this directory. A notification popup should ask if you want to reopen the project inside a container.

The development container has Rust tooling, Yarn along with Npm, and PostgreSQL cli installed. Here are a few choise commands to execute inside vscode's terminal:
- `cargo run` - run backend
- `yarn install` - install frontend dependencies
- `yarn prepare` - build themes
- `yarn dev` - run frontend development server

Debug backend and frontend ( even at the same time! ) from vscode's debug side bar. Debug backend tests by first starting backend debugging session, then starting backend tests debugging session.

## Hardcore Development

If you want to get **all** this running locally without containers: take a look how services in `docker-compose.yml`, `docker-compose.production.yml` and `.env` are configured, and how `deploy.sh` invokes them. Install things as they are required. May the search engine be with you.
