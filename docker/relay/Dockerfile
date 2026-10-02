# syntax=docker/dockerfile:1.7
#
# Self-hosted iroh relay for papo (PAPO_RELAY), built from the same iroh-relay version
# papo links against so client and server speak the same relay protocol.
#
# DEV MODE ONLY: plain HTTP, no TLS, no access control. Use it on a LAN, in tests, or
# behind a TLS-terminating proxy you trust. For production, run iroh-relay with TLS (see
# https://github.com/n0-computer/iroh/tree/main/iroh-relay).

ARG RUST_IMAGE=rust:1.98-alpine3.22
ARG RUNTIME_IMAGE=gcr.io/distroless/static-debian12:nonroot
ARG IROH_RELAY_VERSION=1.3.0

FROM ${RUST_IMAGE} AS build
ARG IROH_RELAY_VERSION
RUN apk add --no-cache musl-dev
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install iroh-relay --version "${IROH_RELAY_VERSION}" --features server --locked --root /out

FROM ${RUNTIME_IMAGE}
ARG IROH_RELAY_VERSION
LABEL org.opencontainers.image.title="papo-relay" \
      org.opencontainers.image.description="iroh-relay ${IROH_RELAY_VERSION} em modo dev (HTTP) para usar com PAPO_RELAY. Não é para produção." \
      org.opencontainers.image.source="https://github.com/Kelvin-Jesus/papo" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"
COPY --from=build /out/bin/iroh-relay /usr/local/bin/iroh-relay
COPY iroh-relay.toml /etc/iroh-relay.toml
USER 65532:65532
EXPOSE 3340/tcp
ENTRYPOINT ["iroh-relay", "--dev", "--config-path", "/etc/iroh-relay.toml"]
