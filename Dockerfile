FROM rust:1.65 AS env
ARG RUST_TOOLCHAIN=nightly-2022-11-08
RUN rustup toolchain install \
  --allow-downgrade \
  --no-self-update \
  --profile minimal \
  ${RUST_TOOLCHAIN}

FROM env AS builder
WORKDIR /app
RUN cargo init --quiet

# build dependencies
COPY ./Cargo.toml ./Cargo.lock ./rust-toolchain.toml ./
RUN cargo build --release

COPY ./build.rs ./
COPY ./contracts ./contracts
RUN cargo build --release

# build binaries
COPY ./ ./
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
HEALTHCHECK --interval=15s --timeout=5s \
  CMD curl -sf http://127.0.0.1:9000/metrics || exit 1
