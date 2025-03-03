# Kattilakioski

<p align="center">
  <img src="frontend/public/img/logo.svg" alt="Kattilakioski logo" />
</p>

## Deploying in Linux environment
[Docker](https://www.docker.com/) and [Docker compose](https://docs.docker.com/compose/) required.

Adapt [`docker-compose.example.yml`](docker-compose.example.yml) to your liking and rename it to `docker-compose.yml`. Then launch server stack with `docker compose up`.

## Development

Get up and running right away, without installing or configuring much locally, with [Development Containers](https://containers.dev/). Install Docker, Docker Compose and VSCode. In VSCode, install [VSCode Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) and open this directory. A notification popup should ask if you want to reopen the project inside a container.

The development container has Rust tooling, Yarn along with Npm, and PostgreSQL CLI installed. Here are a few choise commands to execute inside vscode's terminal:
- `cargo run` - run backend
- `yarn install` - install frontend dependencies
- `yarn prepare` - build themes
- `yarn dev` - run frontend development server
- `yarn run check` - check frontend for errors
- `./tests.sh` - run all tests
- `psql postgres://postgres:mypasswd@postgres -c "UPDATE users SET is_admin = true WHERE username = 'yourusername';"` - promote your user to admin

Debug backend and frontend ( even at the same time! ) from VSCode's debug side bar. Debug backend tests by first starting backend debugging session, then starting backend tests debugging session.

## Hardcore Development

If you want to get **all** this running locally without containers: take a look how services in [`docker-compose.example.yml`](docker-compose.example.yml) are configured, and what tools are installed in [`.devcontainer/Dockerfile`](.devcontainer/Dockerfile). [`.devcontainer/devcontainer.json`](.devcontainer/devcontainer.json) might also provide some information. Install things as they are required. May the search engine be with you.
