FROM rust:1.99-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY sql ./sql
COPY migrations ./migrations
RUN cargo build --release --locked --bins

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home nusavela
WORKDIR /app
COPY --from=build /app/target/release/nusavela-backend /app/target/release/seed_wilayah /app/target/release/seed_osm /app/target/release/mint_admin_token /usr/local/bin/
USER nusavela
EXPOSE 8080
CMD ["nusavela-backend"]
