FROM rust:1.99.1-bookworm AS build
WORKDIR /build
COPY authorization-registry/ ./
RUN cargo build --locked --release --bin authorization_registry

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /build/target/release/authorization_registry /usr/local/bin/authorization_registry
USER 10001:10001
EXPOSE 4000
ENTRYPOINT ["/usr/local/bin/authorization_registry"]
CMD ["--config-path", "/etc/ar/config.json"]
