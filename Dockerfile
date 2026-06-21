# syntax=docker/dockerfile:1
#
# Dockerfile — reproducible cortex_worker fleet image (latexml-oxide).
#
# The Rust port of the Perl LaTeXML-Plugin-Cortex worker image. Like the Perl
# fleet (deployed across ~5 physical machines), the worker is NOT self-contained:
# at convert time latexml-oxide resolves real TeX assets (.sty/.cls/.tfm/fonts)
# from the ambient TeX Live tree via libkpathsea, and it reads year-stamped
# format *dumps* that must be generated against THAT SAME TeX Live. So fan-out
# reproducibility needs the exact toolchain pinned — which is what this image is
# for: identical texlive + matching dumps + the maxperf-cortex binary, baked once
# and run anywhere.
#
# This image COMPILES the worker from the repo it is built in (COPY . + maxperf
# build) and REGENERATES the dumps against the image's texlive — no host-built
# artifacts, fully self-contained.
#
# Build (BuildKit required, for the cache mounts):
#   DOCKER_BUILDKIT=1 docker build --tag cortex-worker:0.6 .
#
# Run — point it at the dispatcher's ZMQ ports (51695 ventilator / 51696 sink).
# Scratch (TMPDIR) is disk-backed at /opt/cortex-scratch: bind-mount a host dir on
# a SEPARATE physical disk from the OS. Do NOT stage on a ramdisk — /dev/shm
# exhaustion under a large fleet truncates inputs -> empty results (CorTeX D-18).
# Give each container a unique --hostname so worker identities don't collide.
#
#   # remote fleet machine -> the public/Tailscale dispatcher:
#   docker run --cpus=72 --memory=96g -v /opt/cortex-scratch:/opt/cortex-scratch \
#     --hostname "$(hostname)" -e DISPATCHER_HOST=corpora.latexml.rs \
#     -e WORKERS=72 -e SERVICE=oxidized-tex-to-html cortex-worker:0.6
#
#   # co-located on the dispatcher host (loopback, skips the bridge/NAT):
#   docker run --network host -v /opt/cortex-scratch:/opt/cortex-scratch \
#     --hostname "$(hostname)" -e WORKERS=72 cortex-worker:0.6

# ---------------------------------------------------------------------------
# texbase: the EXACT TeX Live + runtime libraries shared by the build and the
# runtime stages. Sharing one base is what guarantees the dumps (year-stamped
# against this texlive in the build stage) match what the runtime detects. The
# package set mirrors the proven Perl fleet image's arXiv coverage.
# ---------------------------------------------------------------------------
FROM ubuntu:24.04 AS texbase
ENV DEBIAN_FRONTEND=noninteractive TZ=America/New_York
RUN set -ex && apt-get update -qq && apt-get install -qy --no-install-recommends \
      texlive texlive-fonts-extra texlive-lang-all texlive-latex-extra \
      texlive-bibtex-extra texlive-science texlive-pictures texlive-pstricks \
      texlive-publishers \
      libxml2 libxslt1.1 libkpathsea6 \
      imagemagick mupdf-tools poppler-utils ghostscript \
      ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# ---------------------------------------------------------------------------
# build: toolchain + dev headers on top of texbase. Compiles the worker with the
# maxperf-cortex profile and regenerates the dumps against texbase's texlive.
# ---------------------------------------------------------------------------
FROM texbase AS build
RUN set -ex && apt-get update -qq && apt-get install -qy --no-install-recommends \
      build-essential pkg-config mold git curl \
      libxml2-dev libxslt1-dev libkpathsea-dev \
    && rm -rf /var/lib/apt/lists/*
# rustup with NO default toolchain — the repo's rust-toolchain.toml pins the exact
# nightly (+ rust-src/clippy/rustfmt), installed automatically on the first cargo
# invocation inside /src. Pinned nightly = reproducible codegen (see the toolchain
# file's rationale).
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --no-modify-path --default-toolchain none

WORKDIR /src
COPY . .
# One layer: build the production binary, regenerate dumps against this texlive,
# then stage both outside the cache-mounted target/ so the runtime stage can COPY
# them. make_formats rebuilds the latexml_oxide bin unconditionally (cargo is the
# staleness authority — a stale cache-mount binary would emit yearless dumps).
RUN --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -ex && \
    cargo build --profile maxperf-cortex --features cortex --bin cortex_worker && \
    tools/make_formats.sh && \
    mkdir -p /out/dumps && \
    cp target/maxperf-cortex/cortex_worker /out/cortex_worker && \
    cp resources/dumps/*.dump.txt /out/dumps/ && \
    cp resources/dumps/*.version /out/dumps/ 2>/dev/null || true

# ---------------------------------------------------------------------------
# runtime: texbase (same texlive) + the staged binary and dumps. No toolchain,
# no -dev headers — a lean fleet image.
# ---------------------------------------------------------------------------
FROM texbase
COPY --from=build /out/cortex_worker /usr/local/bin/cortex_worker
COPY --from=build /out/dumps /opt/latexml/dumps
# Dumps are read from disk at runtime (not embedded); point the worker at them.
ENV LATEXML_DUMP_DIR=/opt/latexml/dumps
# Disk-backed scratch — bind-mount a host dir here at run (see header). Created so
# the worker can stage even if the operator forgets the -v (degraded, not broken).
ENV TMPDIR=/opt/cortex-scratch
RUN mkdir -p /opt/cortex-scratch

# Entrypoint mirrors scripts/run_worker.sh: harness flags from env, with the
# dispatcher addresses derived from DISPATCHER_HOST (override SOURCE/SINK directly
# for split hosts). WORKERS defaults to the container's visible CPUs — size it with
# --cpus, or pin WORKERS to PHYSICAL cores + 1/8 for the battle-tested sweet spot.
RUN printf '%s\n' \
  '#!/bin/sh' \
  'set -eu' \
  ': "${DISPATCHER_HOST:=127.0.0.1}"' \
  ': "${SOURCE_ADDRESS:=tcp://${DISPATCHER_HOST}:51695}"' \
  ': "${SINK_ADDRESS:=tcp://${DISPATCHER_HOST}:51696}"' \
  ': "${WORKERS:=$(nproc)}"' \
  ': "${SERVICE:=oxidized-tex-to-html}"' \
  ': "${PROFILE:=ar5iv}"' \
  'exec cortex_worker --harness \' \
  '  --workers "$WORKERS" --service "$SERVICE" \' \
  '  --source-address "$SOURCE_ADDRESS" --sink-address "$SINK_ADDRESS" \' \
  '  --profile "$PROFILE"' \
  > /usr/local/bin/run-worker && chmod +x /usr/local/bin/run-worker

ENTRYPOINT ["/usr/local/bin/run-worker"]
