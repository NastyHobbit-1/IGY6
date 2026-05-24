FROM rust:1.85-slim AS build

WORKDIR /workspace/project
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p igy6-worker

FROM debian:bookworm-slim

WORKDIR /workspace/project
COPY --from=build /workspace/project/target/release/igy6-worker /usr/local/bin/igy6-worker
CMD ["igy6-worker", "--check"]
