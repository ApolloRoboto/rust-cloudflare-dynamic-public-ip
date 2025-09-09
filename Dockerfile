FROM rust:1.89.0-bookworm AS build

WORKDIR /usr/src/rust-cloudflare-dynamic-public-ip
COPY ./src/ ./src/
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo install --path .

FROM debian:bookworm-slim

RUN apt-get update
RUN apt-get install -y openssl ca-certificates
RUN rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/cargo/bin/rust-cloudflare-dynamic-public-ip /usr/local/bin/cfdpip
ENTRYPOINT ["cfdpip"]
