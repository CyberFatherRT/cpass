FROM rust:1.78.0-buster AS chef
WORKDIR /app

RUN cargo install cargo-chef
RUN cargo init --name cpass

COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json


FROM rust:1.78.0-buster AS planner
WORKDIR /app

RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install cargo-chef

COPY --from=chef /app/recipe.json recipe.json
RUN RUSTFLAGS="-C target-feature=+crt-static" cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl


FROM rust:1.78.0-buster AS builder
ENV RUSTFLAGS="-C target-feature=+crt-static"
WORKDIR /app

RUN rustup target add x86_64-unknown-linux-musl

COPY . .
COPY --from=planner /app/target target
COPY --from=planner /app/Cargo.lock Cargo.lock
RUN cargo build --release --target x86_64-unknown-linux-musl


FROM alpine AS final
WORKDIR /app

COPY migrations migrations
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cpass cpass

CMD ["/app/cpass"]
