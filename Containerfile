# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM docker.io/rust:1.52-alpine as cargo-build

RUN rustup default nightly && rustup update

WORKDIR /work

COPY . .

RUN apk add --no-cache musl-dev
RUN cargo build --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:3.13

COPY --from=cargo-build /work/target/release/catinator /usr/local/bin

ENV CATINATOR_CONFIG="/config.toml"
ENV CATINATOR_PASSWORD=""

CMD ["/usr/local/bin/catinator"]
