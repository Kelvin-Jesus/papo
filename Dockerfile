# syntax=docker/dockerfile:1.7
#
# papo in a container: a static musl binary on a distroless base, running as a non-root user.
#
#   docker build -t papo .                       # runtime image for this machine's architecture
#   docker build --target test .                 # runs the hermetic test suite (cargo test)
#   docker buildx build --platform linux/amd64,linux/arm64 -t papo .
#
# The builder always runs on the build machine's own architecture ($BUILDPLATFORM) and
# cross-compiles with zig when the target differs, so a multi-arch build never compiles
# Rust under QEMU emulation.

ARG RUST_IMAGE=rust:1.98-alpine3.22
ARG RUNTIME_IMAGE=gcr.io/distroless/static-debian12:nonroot

FROM --platform=$BUILDPLATFORM ${RUST_IMAGE} AS base
RUN apk add --no-cache musl-dev
WORKDIR /src

# ---------------------------------------------------------------------------------------
# test: the same hermetic suite CI runs (local relay, no internet), in a pinned Linux.
FROM base AS test
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target,id=papo-target-test \
    cargo test --locked

# ---------------------------------------------------------------------------------------
# builder: release binary for $TARGETARCH, statically linked against musl.
FROM base AS builder
ARG TARGETARCH
ARG BUILDARCH
# zig + cargo-zigbuild provide the cross C toolchain ring needs; only installed when
# cross-compiling, so the common native build stays lean.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    if [ "$TARGETARCH" != "$BUILDARCH" ]; then \
      apk add --no-cache zig && cargo install cargo-zigbuild --locked; \
    fi
RUN case "$TARGETARCH" in \
      amd64) echo x86_64-unknown-linux-musl ;; \
      arm64) echo aarch64-unknown-linux-musl ;; \
      *) echo "unsupported architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac > /rust-target \
 && rustup target add "$(cat /rust-target)"
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target,id=papo-target-$TARGETARCH \
    set -eu; \
    target="$(cat /rust-target)"; \
    if [ "$TARGETARCH" = "$BUILDARCH" ]; then build=build; else build=zigbuild; fi; \
    cargo "$build" --release --locked --target "$target"; \
    mkdir -p /out/data; \
    cp "target/$target/release/papo" /out/papo

# ---------------------------------------------------------------------------------------
# runtime: distroless static has CA certificates (reqwest verifies the n0 DNS/pkarr
# endpoints against the system store), tzdata (papo log prints local time; set TZ) and a
# "nonroot" user (65532). No shell, no package manager.
FROM ${RUNTIME_IMAGE} AS runtime
ARG VERSION=dev
LABEL org.opencontainers.image.title="papo" \
      org.opencontainers.image.description="Linha direta P2P entre o seu Claude Code e o do seu colega (servidor MCP + CLI)." \
      org.opencontainers.image.source="https://github.com/Kelvin-Jesus/papo" \
      org.opencontainers.image.url="https://kelvin-jesus.github.io/papo/" \
      org.opencontainers.image.documentation="https://kelvin-jesus.github.io/papo/docs/guias/docker.html" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="${VERSION}"
COPY --from=builder /out/papo /usr/local/bin/papo
# /data must exist and belong to the runtime user, so a fresh named volume mounted
# there inherits a writable directory.
COPY --from=builder --chown=65532:65532 /out/data /data
ENV PAPO_HOME=/data
VOLUME ["/data"]
USER 65532:65532
# No WORKDIR on purpose: papo uses the working directory's name as the default "about"
# shown to peers, and "/" has none. Set it per profile with --about instead.
ENTRYPOINT ["papo"]
CMD ["--help"]
