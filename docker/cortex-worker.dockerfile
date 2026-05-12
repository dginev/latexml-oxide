# Multi-stage Dockerfile for the cortex-worker binary
# Builds latexml-oxide's CorTeX worker for distributed TeX-to-HTML conversion
#
# Build:
#   docker build -f docker/cortex-worker.dockerfile -t cortex-worker .
#
# Run (worker mode):
#   docker run cortex-worker --source-address tcp://dispatcher:51695 --sink-address tcp://dispatcher:51696
#
# Run (standalone mode):
#   docker run -v /data:/data cortex-worker --standalone --input /data/paper.zip --output /data/result.zip

# --- Stage 1: Build ---
FROM rust:1.83-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libxml2-dev libxslt1-dev libkpathsea-dev \
    pkg-config libzmq3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN cargo build --release --bin cortex_worker

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libxml2 libxslt1.1 libkpathsea6 libzmq5 \
    texlive-latex-base texlive-latex-extra texlive-science \
    imagemagick poppler-utils mupdf-tools \
    && rm -rf /var/lib/apt/lists/*
# poppler-utils provides pdftocairo (default fast PDF→PNG/SVG path).
# mupdf-tools provides `mutool draw`, the first-choice PDF rasterizer
# (~2× faster than pdftocairo on matplotlib/pgfplots scatter PDFs and
# ~4× more gzip-compressible SVG output). Graceful fallback to
# pdftocairo if missing, but cortex workers process the canvas
# slow-tail PDFs where the speedup matters most.

# Copy binary
COPY --from=builder /build/target/release/cortex_worker /usr/local/bin/

# Copy resources (XSLT stylesheets, CSS, RelaxNG schemas)
COPY --from=builder /build/resources/ /usr/local/share/latexml-oxide/resources/

# Default: worker mode connecting to localhost dispatcher
ENTRYPOINT ["cortex_worker"]
CMD ["--source-address", "tcp://dispatcher:51695", "--sink-address", "tcp://dispatcher:51696"]
