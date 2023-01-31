# Niumside Population Tracker

This is a population Tracker for PlanetSide 2. It stores population in this format (world > zone > faction > loadout) into a Postgres database. The database still has some empty tables and is designed in a way that I can integrate other modules into it. These modules eventually will replace my [old Discord bot](https://github.com/Emerald-Immersion/vanu-s-enlightenment), which was also my first ever coding project.

## Runtime dependencies

- [Postgres](https://www.postgresql.org/)

## Installation

This project is still in development and not ready for production. Either use the development environment setup or make sure you do the following:

- Install the runtime dependencies.
- Install Rust nightly.
- Override the configuration by copying [config/default.yaml](config/default.yaml) to `config/local.yaml`.
  - We copy the default configuration to a local configuration to allow local changes to be independent of the default configuration.
- Initialize the database with the schema in [niumside-database.sql](niumside-database.sql).
- Run the application by using any of the options below.
  - Without compilation:

    ```bash
    cargo run --release
    ```

  - With compilation (Executable will be different depending on your platform):

    ```bash
    cargo build --release
    ./target/release/niumside-population-tracker
    ```

## Development

### Environment

- [Docker](https://www.docker.com/) or an alternative runtime like [podman](https://podman.io/)
- [Docker compose](https://docs.docker.com/compose/) or an alternative like [podman-compose](https://github.com/containers/podman-compose)
- [Rust](https://www.rust-lang.org/) (nightly)
- My fork of [auraxis-rs](https://github.com/brakenium/auraxis-rs) (until my pull requests are merged)
- A development environment which supports the Rust language server.

### Credentials

Credentials can be found in the [docker-compose.yaml](docker-compose.yaml) file. The default credentials are:
| Application | Username | Password |
| ----------- | -------- | -------- |
| Postgres    | postgres | P@ssw0rd |
| PGAdmin4    | niumside@example.com | P@ssw0rd |

When changing, these have to be changed in [docker-compose.yaml](docker-compose.yaml) and [config/local.yaml](config/local.yaml) (read below for configuration setup) as well as [.env](.env).

### Usage

In order to use the development environment you have to run the following commands for a basic setup after installing the [dependencies](#environment):

```bash
mkdir niumside
git clone https://github.com/brakenium/niumside-poptracker.git
git clone https://github.com/brakenium/auraxis-rs.git
cd niumside-poptracker
cp example.sqlx.env .env
# Edit .env to your liking. The default values are setup for docker-compose.yaml.
nano .env
cp config/default.yaml config/local.yaml
# Edit config/local.yaml to your liking. The commented values are setup for docker-compose.yaml. You will have to edit census.service_id with your own service_id.
nano config/local.yaml
# Run Docker compose. Depending on the setup sudo may be required and in the case of the standalone install the command might be `docker-compose` instead.
docker compose up -d
```

After changes to the database you have to run the following command to update the schema:

```bash
docker compose stop db
docker compose rm db
docker compose up -d db
```

## Database

This project uses a Postgres database which has been configured with PGAdmin4. The generated schema can be found in [niumside-database.sql](niumside-database.sql). The PGAdmin4 project can be found in [niumside-database.pgerd](niumside-database.pgerd) and be opened with PGAdmin4 Desktop or a server install like the docker-compose.yaml sets up at [localhost:8080](https://localhost:8080). The local install will add the development server to the application, however the password will have to be entered manually.

The database is designed to be modular. This means that it can be extended with other modules. The current modules are:

- Population Tracker

Possible future modules are:

- Outfit Tracker
- Session Tracker
- Discord Bot
