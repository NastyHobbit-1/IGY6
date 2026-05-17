FROM rust:1.85-slim AS build

WORKDIR /workspace/project
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p igy6-gateway

FROM debian:bookworm-slim

WORKDIR /workspace/project
COPY --from=build /workspace/project/target/release/igy6-gateway /usr/local/bin/igy6-gateway
EXPOSE 8000
CMD ["igy6-gateway", "--bind", "0.0.0.0:8000"]
