FROM rust:1.86.0 as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
  libpq5 \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/document-extraction-pipeline /app/document-extraction-pipeline
COPY --from=builder /usr/src/app/.env.example /app/.env.example

# Create uploads directory
RUN mkdir -p /app/uploads && chmod 777 /app/uploads

# Run as non-root user
RUN groupadd -r app && useradd -r -g app app
RUN chown -R app:app /app
USER app

EXPOSE 3000

CMD ["./document-extraction-pipeline"]