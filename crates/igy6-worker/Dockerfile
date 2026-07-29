FROM rust:1.88-slim AS build

WORKDIR /workspace/project
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p igy6-worker

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        poppler-utils \
        tesseract-ocr \
        tesseract-ocr-eng \
        ffmpeg \
        python3 \
        python3-pip \
    && pip3 install --no-cache-dir --break-system-packages openai-whisper \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace/project
COPY --from=build /workspace/project/target/release/igy6-worker /usr/local/bin/igy6-worker
CMD ["igy6-worker", "--check"]
