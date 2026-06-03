# syntax=docker/dockerfile:1.7

# =============================================================================
# Stage: base — Rust toolchain + rustup targets for the 6-target matrix.
# Kept deliberately small: this is what `docker compose run --rm test` boots,
# so we don't drag the multi-GB MSVC SDK into the test/dev gate.
# -----------------------------------------------------------------------------
# Targets (per ADR-001 §1.1 D1.3, TASK-007-N8a/N8b/I):
#   x86_64-unknown-linux-musl     — native rustup target + musl-tools
#   aarch64-unknown-linux-musl    — native rustup target + rust-lld
#   x86_64-pc-windows-msvc        — via cargo-xwin (added in cross-tools stage)
#   aarch64-pc-windows-msvc       — via cargo-xwin (added in cross-tools stage)
#   x86_64-apple-darwin           — via cargo-zigbuild (added in cross-tools)
#   aarch64-apple-darwin          — via cargo-zigbuild (added in cross-tools)
# =============================================================================
FROM rust:1.95-bookworm AS base

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        file \
        musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add \
        x86_64-unknown-linux-musl \
        aarch64-unknown-linux-musl \
        x86_64-pc-windows-msvc \
        aarch64-pc-windows-msvc \
        x86_64-apple-darwin \
        aarch64-apple-darwin \
    && rustup component add rustfmt clippy llvm-tools-preview

# cargo-llvm-cov for coverage gating per lawbook §7.1. Kept in the base stage
# so `docker compose run --rm test cargo llvm-cov ...` works without the
# heavier cross-tools layer.
RUN cargo install --locked cargo-llvm-cov

# cargo-audit for advisory scanning per lawbook §7 + org pattern (TASK-007-A
# finding line 226). Pinned indirectly via `--locked`, mirroring the
# cargo-llvm-cov pattern above.
RUN cargo install --locked cargo-audit

WORKDIR /src

# =============================================================================
# Stage: cross-tools — adds zig, cargo-zigbuild, cargo-xwin, and a splatted
# MSVC SDK. Only the `build` stage depends on this; the test/dev gate does not.
# -----------------------------------------------------------------------------
# License note (MSVC SDK):
#   The Microsoft Visual C++ Build Tools and the Windows SDK headers and
#   import libraries downloaded by `xwin splat` are governed by Microsoft's
#   "Visual Studio Build Tools" EULA. We accept that license programmatically
#   via `xwin --accept-license`; users building this image accept it
#   transitively. See https://visualstudio.microsoft.com/license-terms/ for
#   the full text.
# =============================================================================
FROM base AS cross-tools

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        curl \
        xz-utils \
        clang \
        lld \
        llvm \
    && rm -rf /var/lib/apt/lists/*

# Install Zig (pinned 0.13.0) for cargo-zigbuild. SHA256 verified against the
# value published in https://ziglang.org/download/index.json.
ARG ZIG_VERSION=0.13.0
ARG ZIG_SHA256=d45312e61ebcc48032b77bc4cf7fd6915c11fa16e4aad116b66c9468211230ea
RUN set -eu; \
    cd /tmp; \
    curl -fsSLo zig.tar.xz "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-x86_64-${ZIG_VERSION}.tar.xz"; \
    echo "${ZIG_SHA256}  zig.tar.xz" | sha256sum -c -; \
    tar -xf zig.tar.xz; \
    mv "zig-linux-x86_64-${ZIG_VERSION}" /opt/zig; \
    ln -s /opt/zig/zig /usr/local/bin/zig; \
    rm zig.tar.xz; \
    zig version

# Cargo subcommands for cross-compilation drivers.
RUN cargo install --locked cargo-xwin \
    && cargo install --locked cargo-zigbuild \
    && cargo install --locked xwin

# Populate the MSVC SDK cache via xwin splat. --accept-license records
# acceptance of Microsoft's EULA (see header comment for URL). Both arches
# are splatted in a single invocation; the layer caches the ~1-2 GiB payload.
#
# xwin 0.9.0 moved --arch from the `splat` subcommand to a global option
# (placed BEFORE the subcommand) and accepts a comma-separated list rather
# than repeated flags. Verified via `xwin --help` / `xwin splat --help` in
# a throwaway `rust:1.95-bookworm` container (xwin v0.9.0 installed via
# `cargo install --locked xwin`); the global `--arch <ARCH>` option lists
# `[possible values: x86, x86_64, aarch, aarch64]`. See BUG-005.
ENV XWIN_CACHE_DIR=/xwin
RUN xwin --accept-license \
        --arch x86_64,aarch64 \
        --timeout 300s \
        --http-retry 3 \
        splat \
        --output /xwin

# =============================================================================
# Stage: build — compile real source for ${TARGET}.
# Routes to the right driver:
#   *windows-msvc*  -> cargo xwin build
#   *apple-darwin   -> cargo zigbuild
#   default         -> cargo build
# =============================================================================
FROM cross-tools AS build

ARG TARGET=x86_64-unknown-linux-musl
ENV TARGET=${TARGET}

COPY . .

RUN set -eu; \
    case "${TARGET}" in \
      *windows-msvc*) cargo xwin build --release --target "${TARGET}" --locked ;; \
      *apple-darwin)  cargo zigbuild   --release --target "${TARGET}" --locked ;; \
      *)              cargo build      --release --target "${TARGET}" --locked ;; \
    esac

RUN set -eu; \
    case "${TARGET}" in \
      *windows*) ext=".exe" ;; \
      *)         ext="" ;; \
    esac; \
    mkdir -p "/dist/${TARGET}"; \
    cp "target/${TARGET}/release/xdu${ext}" "/dist/${TARGET}/"; \
    file "/dist/${TARGET}/xdu${ext}"; \
    wc -c "/dist/${TARGET}/xdu${ext}"

# =============================================================================
# Stage: artifact — scratch image with just the binary at /
# Useful for `docker create + docker cp` extraction in CI.
# =============================================================================
FROM scratch AS artifact

ARG TARGET=x86_64-unknown-linux-musl
COPY --from=build /dist/${TARGET}/ /
