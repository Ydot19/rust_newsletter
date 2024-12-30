# Newsletter Application

## Purpose and Goals

Learning how to develop, build, test and continuous integrate a backend service in Rust.

### Goals

- [ ] Build a backend service that can communicate with postgres.
    - [ ] Backend should support basic crud operations (GET/POST/DELETE).
    - [ ] Backend should have built in telemetry and logging
    - [ ] Support testing at various levels
- [ ] Containerize the application
- [ ] Add github actions for the application

### Non-Goals

- Deploy application to the cloud.
> Because this is a toy project and the fact that deployment is not difficult besides the security hardening and network access controls, I will forgo this for the sake of this project. In a more serious project, I will likely showcase how this is done in Google Cloud or AWS
- Benchmark the performance of the application
> This application is mainly for learning purposes. Benchmarking would not be fruitful because most of the time is spent on the call to the database (Postgres)

## Tools

This project uses the following technologies

- Rust (v1.83)
    - Cargo nextest
    - Cargo bininstall
    - Axum + Tokio for the Backend API
    - Diesel.rs for the ORM and database migration
- Postgres
- Podman (OCI Container)
    - Podman Compose (mimick the functionalities of docker compose)

### Installing tools

#### Rust
#
Use [rustup](https://rustup.rs/) to install rust and (for this project) Rust v1.83

#### Cargo Bininstall

Install cargo packages as binaries. Install with the following command

```zsh
cargo install cargo-binstall
```

Project Version: `v1.10.x`

#### Cargo Nextest
This is the test runner mainly used by this project. 
Install nextest with the following:

```zsh
cargo binstall cargo-nextest --secure
```

Project version `v0.9.x`

#### Install Diesel 

Install the diesel cli with the following:

```zsh
cargo binstall diesel_cli
```

Project version `v2.2.x`

## Project Structure

```
db/ <- diesel generated migration files and the database schema
    ...
lib/ <-- houses all the application logic
    adapter/ <-- logic to communicate with postgres
    domain/ <-- internal models and errors application
    model/ <-- request and response objects for external use
    routes/ <-- different routes
    libs.rs
src/ 
    main.rs <-- application binary is built here
tests/
    common/ <-- common test utilities
    integration/ <-- integration tests (calls libs.rs directly)
    end_to_end/ <-- runs tests against a running docker container initialized from source
```
## Setup

### Start up database

```zsh
make start-database migrate-database
```

#### Stop database

Inverse operation

```zsh
make stop-database
```

### Execute database schema dump

Dump the database schema to a file

```zsh
make dump-schema
```

Note: assumes the database has started and all relevant migrations are complete

### Start backend service

```zsh
make start-datbase
```

#### Stop back service

```zsh
make stop-database
```

### Start up all services

```zsh
make up
```

#### Tear down all services

```zsh
make down
```

## Migrations

Before using diesel, you need to set the connection string as an environment variable:
```
DATABASE_URL=postgresql://localhost:5432/newsletter?sslmode=disable&user=postgres&password=postgres
```

Fortunately, diesel comes pre-packaged with `dotenvy` crate so you can rely on projects local `.env` with-thought having to worry about setting the above environment variable.

### Creating Migration Files

For local development, create a new migration file by running

```zsh
diesel migration --migration-dir ./db/migrations generate create_subscriptions
```

Note:
- Afterwards, run the `make dump-schema` to view the latest database schema for this application
- This command assumes the database is started, healthy and able to receive connections.

### Dumping and Loading schema

As the last note eluded to, you can dump the database schema using `make dump-schema`. This section goes over how to prepare the database using the dumped schema file.

```zsh
make start-database
make load-schema
```

With these two make commands, we can start the application without having to rely on the diesel cli to manually execute the migrations.

## Testing

### Run Lib Tests

Pre-requisites:
- Start Database 
- Execute Migrations / Load Schema
- Cargo Nextest installed

Run lib tests :

```zsh
make test-lib
```

Note: Lib tests are all the tests located in the lib directory

### Run Integration Tests


Pre-requisites:
- Start Database 
- Execute Migrations / Load Schema
- Cargo Nextest installed

Run integration tests using the following command:

```zsh
make test-integration
```

In this project, integration tests are:
- Test that are located in `tests/integration` directory
- Call methods in the root of lib/libs.rs file.
- The test calls code that you can step through via a debugger.

### Run End To End Tests


Pre-requisites:
- Run the following
```zsh
make up
make migrate-database
```


In this project there are two ways to run end to end tests

1. Run End To End Tests by executing cargo from the terminal

```zsh
make test-endtoend 
```

2. Run End To End Tests using the test container in the compose file

```zsh
make run-endtoend
```

In this project, endtoend tests are:
- Tests that test the actual running services via http.
- A sanity check that the different docker services are working together.
