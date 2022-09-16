FROM rust AS env
ARG RUST_TOOLCHAIN=nightly-2022-09-15
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
# build binaries
COPY ./ ./
RUN cargo build --release --bin sandwitch

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/sandwitch /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/sandwitch", "--config", "/etc/sandwitch/sandwitch.toml"]
EXPOSE 9000/tcp
HEALTHCHECK --interval=15s --timeout=5s \
  CMD curl -sf http://127.0.0.1:9000/metrics || exit 1
