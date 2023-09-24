# REST API

## Running the API
- Start the docker service *(If on Windows, run Docker Desktop)*
- Run the mongo database with `docker-compose up -d mongo`
- `cd server/`
- run `cargo run -- -e ../.env`

> [Access the automatically generated documentation](http://127.0.0.1:8080/rapidoc/index.html)


## Running tests

- Start the docker service *(If on Windows, run Docker Desktop)*
- Go in the directory `server/crates/api`
- Run `cargo test`
