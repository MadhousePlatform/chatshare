FROM rust:bookworm as builder

WORKDIR /src
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install libssl3 && apt clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/chatshare /usr/local/bin/chatshare
CMD ["chatshare"]
