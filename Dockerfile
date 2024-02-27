FROM rust:1.74 as builder

WORKDIR /crates
COPY ./.cargo ./.cargo
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
RUN --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/crates/target \
    cargo install --path .

FROM debian:bookworm-slim
WORKDIR /server
COPY --from=builder /usr/local/cargo/bin/liquid-address .
CMD ["/server/liquid-address"]
