# syntax=docker/dockerfile:1.4
FROM rust:latest AS env
RUN rm -f /etc/apt/apt.conf.d/docker-clean \
  && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt --mount=type=cache,target=/var/lib/apt \
  apt update \
  && apt install --yes --no-install-recommends \
  protobuf-compiler
RUN wget -qO- "https://github.com/mozilla/sccache/releases/download/v0.4.2/sccache-v0.4.2-$(uname -m)-unknown-linux-musl.tar.gz" \
  | tar xzC /usr/local/bin/ --strip-components 1 "sccache-v0.4.2-$(uname -m)-unknown-linux-musl/sccache" \
  && chmod +x /usr/local/bin/sccache
ENV RUSTC_WRAPPER=/usr/local/bin/sccache
ENV SCCACHE_DIR=/var/cache/sccache
WORKDIR /app

FROM env AS contracts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=${SCCACHE_DIR} \
  cargo install \
  --git https://github.com/foundry-rs/foundry foundry-cli \
  --profile local \
  --bin forge \
  --locked
COPY --link ./foundry.toml ./remappings.txt ./
COPY --link ./contracts/ ./contracts/
RUN --mount=type=cache,target=./contracts/cache forge build

FROM env AS builder
COPY --link ./rust-toolchain.toml ./
RUN --mount=type=cache,target=/usr/local/rustup \
  rustup set profile minimal \
  && rustup show active-toolchain
COPY --link . .
COPY --from=contracts --link /app/contracts/out/ ./contracts/out/
RUN --mount=type=cache,target=/usr/local/rustup \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=${SCCACHE_DIR} \
  --mount=type=cache,target=./target \
  cargo build --locked --release --bin sandwitch \
  && mv ./target/release/sandwitch /usr/local/bin/

FROM gcr.io/distroless/cc
COPY --from=builder /usr/local/bin/sandwitch /usr/local/bin/
VOLUME [ "/etc/sandwitch" ]
ENTRYPOINT [\
  "/usr/local/bin/sandwitch",\
  "--config", "/etc/sandwitch/sandwitch.toml"\
  ]
EXPOSE 9000/tcp
HEALTHCHECK --interval=15s --timeout=5s \
  CMD curl -sf http://127.0.0.1:9000/metrics || exit 1
