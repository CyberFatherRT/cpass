FROM rust:1.78.0-buster AS chef

WORKDIR /app

RUN cargo install cargo-chef
RUN cargo init --name cpass

COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.78.0-buster AS planner
WORKDIR /root

RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v27.1/protoc-27.1-linux-x86_64.zip -O ~/protoc.zip \
    && unzip ~/protoc.zip \
    && ln -sf ~/bin/protoc /usr/bin/protoc

WORKDIR /app

RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install cargo-chef
RUN cargo init --name cpass

COPY --from=chef /app/recipe.json recipe.json
COPY proto proto
COPY build.rs build.rs

RUN cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl


FROM rust:1.78.0-buster AS builder
WORKDIR /root

RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v27.1/protoc-27.1-linux-x86_64.zip -O ~/protoc.zip \
    && unzip ~/protoc.zip \
    && ln -sf ~/bin/protoc /usr/bin/protoc

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY .sqlx .sqlx
COPY migrations migrations
COPY src src
COPY proto proto
COPY build.rs build.rs
COPY Cargo.toml Cargo.toml

COPY --from=planner /app/target target
COPY --from=planner /app/Cargo.lock Cargo.lock

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine AS final
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cpass app
ENTRYPOINT [ "/app" ]
