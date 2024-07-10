FROM rust:1.79 as builder
WORKDIR /code/

COPY Cargo.lock .
COPY Cargo.toml .
COPY src ./src
COPY .sqlx ./.sqlx

ENV SQLX_OFFLINE=true
RUN cargo install --path .


FROM debian:stable-slim

RUN apt-get update \
 && apt-get install -y openssl  ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/simple-restaurant-api /usr/local/bin/

CMD ["simple-restaurant-api"]