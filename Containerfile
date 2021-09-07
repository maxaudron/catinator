# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1.54.0-alpine AS chef
WORKDIR /work

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /work/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM scratch

COPY --from=builder /work/target/release/catinator /usr/local/bin

ENV CATINATOR_CONFIG="/config.toml"
ENV CATINATOR_PASSWORD=""

ENTRYPOINT ["/usr/local/bin/catinator"]
