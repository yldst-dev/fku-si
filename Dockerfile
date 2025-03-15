# 빌드 스테이지
FROM rust:latest as builder

WORKDIR /app

# 의존성을 먼저 빌드하여 캐싱 효과를 높임
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/fku_si* && \
    rm -rf src

# 소스 코드 복사 및 빌드
COPY src src
COPY .env.example .
RUN cargo build --release

# 실행 스테이지
FROM debian:bookworm-slim

WORKDIR /app

# 실행에 필요한 라이브러리 설치
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# 빌드된 실행 파일 및 필요한 파일들만 복사
COPY --from=builder /app/target/release/fku-si .
COPY --from=builder /app/.env.example .env.example

# .env 파일이 존재하지 않을 경우 .env.example을 .env로 복사하는 스크립트
RUN echo '#!/bin/bash\n\
if [ ! -f .env ]; then\n\
    cp .env.example .env\n\
    echo "Created .env file from .env.example. Please edit .env with your settings."\n\
fi\n\
./fku-si\n' > /app/start.sh && \
    chmod +x /app/start.sh

CMD ["/app/start.sh"] 