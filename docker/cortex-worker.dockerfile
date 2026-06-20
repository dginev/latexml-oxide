# Turnkey Docker image for the latexml-oxide CorTeX worker (`cortex_worker`).
#
# The Rust counterpart of the legacy Perl fleet image
# (../LaTeXML-Plugin-CorTeX/Dockerfile): build once, then `docker run` the
# self-supervising `--harness` fleet pointed at a CorTeX dispatcher. Same
# dispatcher, ZMQ ports, and result-archive contract — different engine and
# service name (`oxidized-tex-to-html` vs the Perl `tex_to_html`).
#
# Build:
#   export HOSTTIME=$(date -Iminute)
#   docker build -f docker/cortex-worker.dockerfile --build-arg HOSTTIME=$HOSTTIME -t cortex-worker:latest .
#
# Run the fleet (turnkey — pass just the dispatcher host; `--harness` auto-sizes the box):
#   # local dispatcher (loopback, skips the Docker NAT):
#   docker run --network host --shm-size=32g --hostname=$(hostname) cortex-worker 127.0.0.1
#   # remote dispatcher:
#   docker run --shm-size=32g --hostname=$(hostname) cortex-worker DISPATCHER_IP
#   # positional args: <dispatcher-host> [ventilator-port=51695] [sink-port=51696] [service=oxidized-tex-to-html]
#   # env: PROFILE=ar5iv (default) | generic ; WORKERS=<n> to pin the fleet size
#   #      (battle-hardened sweet spot = physical cores + 1/8; default = harness CPU-derived).
#
# Advanced — pass cortex_worker flags straight through (e.g. a one-off standalone conversion):
#   docker run -v /data:/data cortex-worker --standalone --input /data/paper.zip --output /data/result.zip

# --- Stage 1: build the cortex_worker binary (nightly pinned by rust-toolchain.toml) ---
FROM rust:1.83-slim-bookworm AS builder

# Build deps: libxml2/libxslt/kpathsea (runtime-bindings), system ZeroMQ for the `cortex`
# feature's pericortex→zmq, pkg-config, and git+TLS for the pericortex git dependency.
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential pkg-config git ca-certificates \
      libxml2-dev libxslt1-dev libkpathsea-dev libzmq3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

# cortex_worker carries `required-features = ["cortex"]`, so the feature set is MANDATORY — a plain
# `cargo build --bin cortex_worker` refuses to build it. This matches the host build recipe
# (tools/ + docs/CORTEX_WORKER_HARNESS.md). Also build latexml_oxide: its `--init` generates the
# kernel dumps baked into the runtime stage below (cortex_worker has no --init of its own).
RUN cargo build --release --no-default-features --features cortex,runtime-bindings \
      --bin cortex_worker --bin latexml_oxide

# --- Stage 2: slim runtime with the full arXiv-capable TeX Live ---
FROM debian:bookworm-slim
ENV TZ=America/New_York
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

ARG HOSTTIME
ENV DOCKER_BUILD_TIME=$HOSTTIME

# Full TeX Live — the arXiv long tail needs the extra fonts/langs/packages (matches the legacy
# Perl image). latexml-oxide reads the same texmf tree via kpathsea at conversion time, so a
# minimal `texlive-latex-base` would fail on real papers' exotic `\usepackage`s and fonts.
RUN apt-get update && apt-get install -y --no-install-recommends \
      texlive texlive-fonts-extra texlive-lang-all texlive-latex-extra \
      texlive-bibtex-extra texlive-science texlive-pictures texlive-pstricks \
      texlive-publishers \
      libxml2 libxslt1.1 libkpathsea6 libzmq5 \
      imagemagick poppler-utils mupdf-tools \
      ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# poppler-utils → pdftocairo; mupdf-tools → `mutool draw`, the first-choice PDF rasterizer
# (faster + more gzip-compressible than pdftocairo on the canvas slow-tail PDFs; graceful
# fallback to pdftocairo if absent).

# Let ImageMagick read/write PDF/EPS and raise its resource ceilings — arXiv figures need it
# (the same patch the legacy Perl image applies; Debian's default policy.xml blocks PDF/EPS).
RUN set -e; P=/etc/ImageMagick-6/policy.xml; if [ -f "$P" ]; then \
      sed -i -E 's/rights="none" pattern="([XE]?PS[0-9]?|PDF)"/rights="read|write" pattern="\1"/g' "$P"; \
      sed -i -E 's/(name="width" value=)"[^"]*"/\1"256KP"/'  "$P"; \
      sed -i -E 's/(name="height" value=)"[^"]*"/\1"256KP"/' "$P"; \
      sed -i -E 's/(name="area" value=)"[^"]*"/\1"4GiB"/'    "$P"; \
      sed -i -E 's/(name="disk" value=)"[^"]*"/\1"6GiB"/'    "$P"; \
      sed -i -E 's/(name="memory" value=)"[^"]*"/\1"4GiB"/'  "$P"; \
      sed -i -E 's/(name="map" value=)"[^"]*"/\1"4GiB"/'     "$P"; \
    fi

# The worker binary + its resources (XSLT/CSS/RelaxNG), plus latexml_oxide (only to generate dumps).
COPY --from=builder /build/target/release/cortex_worker /usr/local/bin/
COPY --from=builder /build/target/release/latexml_oxide /usr/local/bin/
COPY --from=builder /build/resources/ /usr/local/share/latexml-oxide/resources/

# Bake the ambient-year TeX kernel dumps into the image — the Rust analog of the legacy Perl image's
# `cpanm --build-args formats` (`make formats`). `.gitignore` excludes resources/dumps, so a fresh
# checkout ships none; without a cached dump every one-conversion child re-bootstraps the kernel
# (~2 s/paper). `latexml_oxide --init` writes plain/latex.<year>.dump.txt under CWD/resources/dumps;
# LATEXML_DUMP_DIR points the runtime resolver at them.
ENV LATEXML_DUMP_DIR=/usr/local/share/latexml-oxide/resources/dumps
RUN cd /usr/local/share/latexml-oxide && mkdir -p resources/dumps \
    && latexml_oxide --init=plain.tex \
    && latexml_oxide --init=latex.ltx \
    && ls -la resources/dumps/*.dump.txt

COPY docker/cortex-worker-entrypoint.sh /usr/local/bin/cortex-worker-entrypoint.sh
RUN chmod +x /usr/local/bin/cortex-worker-entrypoint.sh

# Scratch on the ramdisk (run with --shm-size); matches the legacy worker's TMPDIR=/dev/shm.
ENV TMPDIR=/dev/shm

ENTRYPOINT ["cortex-worker-entrypoint.sh"]
# Default: harness fleet against a localhost dispatcher, service oxidized-tex-to-html.
CMD ["127.0.0.1"]
