# Stage 1: Build
FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/app

# Install required packages (sqlite3 is needed to create the build-time DB)
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev build-essential sqlite3

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# 🌟 THE TRICK: Create a live SQLite DB instantly using your migration files 
# so sqlx::query! can verify the schema during compilation.
# Using `cat migrations/*.sql` ensures it applies all future migrations too.
RUN cat migrations/*.sql | sqlite3 build.db
ENV DATABASE_URL="sqlite://build.db"

# Build the application (SQLx will use build.db to verify queries)
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# 🌟 CRITICAL FIX: Upgrade base OS packages first to patch Trivy CVEs (zlib, perl, sqlite3)
RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y ca-certificates sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary and migrations from the builder
COPY --from=builder /usr/src/app/target/release/comment-service ./
COPY --from=builder /usr/src/app/migrations ./migrations

# Expose the Axum port
EXPOSE 3000

# Run the server
CMD ["./comment-service"]