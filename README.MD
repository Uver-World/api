# REST API

## Dependencies

- [build-essential](https://packages.debian.org/bookworm/build-essential)
- [pkg-config](https://packages.debian.org/bookworm/pkg-config)
- [libssl-dev](https://packages.debian.org/bookworm/libssl-dev)
- [libudev-dev](https://packages.debian.org/bookworm/libudev-dev)
- [rust](https://sh.rustup.rs)

## Running the database
- Start the docker service *(If on Windows, run Docker Desktop)*
- Run the mongo database with `docker-compose up -d mongo`

## Running the API
- `cd server/`
- run `cargo run -- -e ../.env`

> [Click to access the automatically generated documentation](http://127.0.0.1:8080/rapidoc/index.html)

## Running tests

- Start the docker service *(If on Windows, run Docker Desktop)*
- Go in the directory `server/crates/api`
- Run `cargo test`
