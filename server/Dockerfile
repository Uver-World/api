FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR server

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /server/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin server

# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
RUN apt-get -qy update && apt-get -qy upgrade
RUN apt-get install ca-certificates -qy
WORKDIR server
COPY --from=builder /server/target/release/server /usr/local/bin/
COPY Rocket.toml .
ENTRYPOINT ["/usr/local/bin/server"]
