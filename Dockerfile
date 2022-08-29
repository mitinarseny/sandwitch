FROM rustlang/rust:nightly AS chef
RUN cargo install cargo-chef
WORKDIR /app


FROM chef AS planner
COPY ./ ./
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json ./
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./ ./
RUN cargo build --release --bin sandwitch


FROM debian:buster-slim
RUN apt-get update && apt-get install --yes --no-install-recommends \
  openssl \
  ca-certificates

COPY --from=builder /app/target/release/sandwitch /usr/local/bin/
COPY ./sandwitch.toml /etc/sandwitch/
ENTRYPOINT ["/usr/local/bin/sandwitch", "--config", "/etc/sandwitch/sandwitch.toml"]
