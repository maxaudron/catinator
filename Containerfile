# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM kube.cat/cocainefarm/rust:1.55.0 AS chef
RUN apk add openssl-dev
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
FROM docker.io/alpine:3.14 as alpine

COPY --from=builder /work/target/release/catinator /usr/local/bin

ENV CATINATOR_CONFIG="/config.toml"
ENV CATINATOR_PASSWORD=""

ENTRYPOINT ["/usr/local/bin/catinator"]
