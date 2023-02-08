FROM ethereum/solc:stable AS solc

FROM rust:latest AS env
WORKDIR /app
COPY ./rust-toolchain.toml ./
RUN rustup set profile minimal \
  && rustup show active-toolchain
COPY --from=solc /usr/bin/solc /usr/bin/

FROM env AS builder
# build dependencies
COPY ./Cargo.toml ./Cargo.lock ./
RUN cargo new --quiet ./src/sandwitch \
  && cargo new --quiet --lib ./src/contracts \
  && echo 'fn main() {}' > ./src/contracts/build.rs
COPY ./src/sandwitch/Cargo.toml ./src/sandwitch/
COPY ./src/contracts/Cargo.toml ./src/contracts/
RUN cargo build --release
# build contract bindings
COPY ./contracts/ ./contracts/
COPY ./src/contracts/ ./src/contracts/
RUN cargo build --release
# build binaries
COPY ./src/sandwitch/ ./src/sandwitch/
RUN cargo build --release --bin sandwitch

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/sandwitch /usr/local/bin/
ENTRYPOINT [\
  "/usr/local/bin/sandwitch",\
  "--metrics-host", "0.0.0.0",\
  "--metrics-port", "9000",\
  "--config", "/etc/sandwitch/sandwitch.toml",\
  "--accounts-dir", "/etc/sandwitch/accounts"\
]
EXPOSE 9000/tcp
VOLUME ["/etc/sandwitch/accounts"]
HEALTHCHECK --interval=15s --timeout=5s \
  CMD curl -sf http://127.0.0.1:9000/metrics || exit 1
