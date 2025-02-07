FROM rust:1.79 as builder

WORKDIR /usr/src/solana-mirror-api

COPY Cargo.lock Cargo.toml ./

COPY . .

RUN rustup update stable
RUN cargo build --release

FROM debian:latest
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    libc6 \
    libc-bin \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/solana-mirror-api

COPY --from=builder /usr/src/solana-mirror-api/target/release/solana-mirror-api .

COPY --from=builder /usr/src/solana-mirror-api/lib/src/coingecko.json ./lib/src/coingecko.json

RUN chmod +x ./solana-mirror-api

EXPOSE 8000

CMD ["./solana-mirror-api"]
