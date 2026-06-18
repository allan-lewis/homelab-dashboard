FROM rust:1-bookworm AS builder

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked

COPY Cargo.toml Cargo.lock ./
COPY index.html ./
COPY authentik.png ./
COPY src ./src

RUN trunk build --release
RUN cargo build --release --bin server


FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/dist /app/dist

EXPOSE 3000

CMD ["/app/server"]
