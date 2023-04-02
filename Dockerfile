FROM rust:latest AS env
WORKDIR /app
RUN cargo install --git https://github.com/foundry-rs/foundry foundry-cli --bin forge --locked

FROM env AS contracts
COPY ./foundry.toml ./remappings.txt ./
COPY ./contracts/ ./contracts/
RUN forge build

FROM env AS builder
COPY ./rust-toolchain.toml ./
RUN rustup set profile minimal \
  && rustup show active-toolchain
# build dependencies
RUN cargo init --quiet --lib \
  && cargo new --quiet --lib ./bindings
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./bindings/Cargo.toml ./bindings/
RUN cargo check --release
RUN cargo build --release
# build contract bindings
COPY ./bindings/ ./bindings/
RUN mkdir -p ./contracts/out
COPY --from=contracts ./contracts/out/ ./contracts/out/
RUN cargo build --release
# build binaries
COPY ./src/ ./src/
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
