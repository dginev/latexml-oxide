# syntax=docker/dockerfile:1
#
# Unified Dockerfile for BOTH published latexml-oxide images — select with --target:
#
#   # general-purpose CLI  →  ghcr.io/dginev/latexml-oxide  (DEFAULT target)
#   docker build --target cli -t ghcr.io/dginev/latexml-oxide .
#   docker run --rm -v "$PWD:/work" ghcr.io/dginev/latexml-oxide paper.tex
#
#   # CorTeX fleet worker  →  ghcr.io/dginev/latexml-oxide/cortex-worker
#   export HOSTTIME=$(date -Iminute)
#   docker build --target worker --build-arg HOSTTIME=$HOSTTIME -t cortex-worker .
#   docker run --network host -v /opt/cortex-scratch:/opt/cortex-scratch \
#     --hostname="$(hostname)" cortex-worker 127.0.0.1
#
# DRY consolidation of the former root Dockerfile + docker/cli.dockerfile +
# docker/cortex-worker.dockerfile: every dependency is declared ONCE in the
# shared `texbase` (runtime) + `toolchain` (build) stages; only the per-binary
# build command, runtime COPY, and entrypoint differ per target. Both binaries
# ride the same TeX Live + graphics environment, so a paper converts identically
# whether run through the CLI or the fleet worker.
#
# Fleet worker details: the Rust counterpart of the legacy Perl fleet image
# (LaTeXML-Plugin-CorTeX/Dockerfile) — same dispatcher, ZMQ ports (51695/51696),
# and result-archive contract; different engine + service name
# (`oxidized_tex_to_html`). Scratch (TMPDIR) is disk-backed at /opt/cortex-scratch:
# bind-mount a host dir on a SEPARATE physical disk from the OS. Do NOT stage on a
# ramdisk — /dev/shm exhaustion under a large fleet truncates inputs → empty
# results (CorTeX D-18). Give each container a unique --hostname.

# ===========================================================================
# texbase — the SINGLE place runtime dependencies live. Full arXiv-capable TeX
# Live + graphics tools + runtime C libs, shared by both images. The
# comprehensive TeX Live (lang-all / fonts-extra / pstricks / …) matters for the
# arXiv long tail: latexml-oxide reads the texmf tree via kpathsea at convert
# time, so a minimal `texlive-latex-base` would fail real papers' exotic
# `\usepackage`s and fonts. libzmq5 is used only by the worker (harmless, small,
# in the CLI image). Sharing ONE base guarantees the worker's baked dumps match
# the runtime TeX Live (TL2023) — a Perl-vs-Rust delta is then the engine, not
# the texmf tree.
# ===========================================================================
FROM ubuntu:24.04 AS texbase
ENV TZ=America/New_York DEBIAN_FRONTEND=noninteractive
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN set -ex && apt-get update -qq && apt-get install -qy --no-install-recommends \
      texlive texlive-fonts-extra texlive-lang-all texlive-latex-extra \
      texlive-bibtex-extra texlive-science texlive-pictures texlive-pstricks \
      texlive-publishers \
      libxml2 libxslt1.1 libkpathsea6 libzmq5 \
      ghostscript imagemagick poppler-utils mupdf-tools dvipng dvisvgm \
      ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# ghostscript is EXPLICIT (only a *Recommends* of imagemagick, so
# `--no-install-recommends` would drop it): `gs` is the EPS/PS rasterizer AND
# ImageMagick's PS/EPS/PDF delegate — without it every EPS/PS figure fails
# `imageprocessing:failed_to_convert` (PDFs survive via mutool/pdftocairo).
# poppler-utils → pdftocairo; mupdf-tools → `mutool draw` (first-choice PDF
# rasterizer, faster + more gzip-compressible on the canvas slow-tail).

# Let ImageMagick read/write PDF/EPS and raise its resource ceilings — arXiv
# figures need it (Debian's default policy.xml blocks PDF/EPS). Same patch the
# legacy Perl image applies; benefits both the CLI and the worker.
RUN set -e; P=/etc/ImageMagick-6/policy.xml; if [ -f "$P" ]; then \
      sed -i -E 's/rights="none" pattern="([XE]?PS[0-9]?|PDF)"/rights="read|write" pattern="\1"/g' "$P"; \
      sed -i -E 's/(name="width" value=)"[^"]*"/\1"256KP"/'  "$P"; \
      sed -i -E 's/(name="height" value=)"[^"]*"/\1"256KP"/' "$P"; \
      sed -i -E 's/(name="area" value=)"[^"]*"/\1"4GiB"/'    "$P"; \
      sed -i -E 's/(name="disk" value=)"[^"]*"/\1"6GiB"/'    "$P"; \
      sed -i -E 's/(name="memory" value=)"[^"]*"/\1"4GiB"/'  "$P"; \
      sed -i -E 's/(name="map" value=)"[^"]*"/\1"4GiB"/'     "$P"; \
    fi

# ===========================================================================
# toolchain — the SINGLE place build dependencies live. Build deps for BOTH
# binaries (union): mold (the linker pinned by .cargo/config.toml), the engine's
# libxml2/libxslt/kpathsea headers, plus the worker-only clang+libclang (a
# `cortex`-feature dep runs bindgen) and libzmq3-dev (pericortex → zmq). The
# worker-only deps are harmless for the CLI build and cost nothing in the final
# CLI image (build stages are discarded). rustup installs the exact nightly
# pinned by rust-toolchain.toml on the first cargo invocation.
# ===========================================================================
FROM texbase AS toolchain
RUN set -ex && apt-get update -qq && apt-get install -qy --no-install-recommends \
      build-essential pkg-config mold git curl \
      libxml2-dev libxslt1-dev libkpathsea-dev \
      clang libclang-dev libzmq3-dev \
    && rm -rf /var/lib/apt/lists/*
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --no-modify-path --default-toolchain none
WORKDIR /src

# ===========================================================================
# notices — the THIRD-PARTY-NOTICES both images ship.
# ===========================================================================
# These images are a DISTRIBUTION CHANNEL: docker.yml pushes them to GHCR on
# release-publish, so each one hands a user the binary with statically linked LGPL
# code in it (libkpathsea; libmarpa's libavl/obstack files) and, in the worker, the
# W3C/Mozilla SVG schemas whose license requires their notice travel with every
# copy. They shipped neither the notices nor even our own LICENSE until 0.7.4.
#
# The lesson F7 (the bare Windows .exe) and F9 (the .deb, assembled by cargo-deb)
# taught twice: an artifact built by a DIFFERENT tool does not inherit the staging
# done for the others. `tools/make_release.sh` never runs here, so this stage is
# where the container's notices have to come from.
#
# Generated ONCE for both targets: this stage shares `toolchain` + `COPY . .` with
# build-cli/build-worker, so the context layer is cached, and cargo-about is built
# once rather than in each build stage. The two images link DIFFERENT graphs
# (`runtime-bindings` vs `cortex` — the worker adds zmq/pericortex), so section 5
# is generated per-target rather than shared: one file each, attributing exactly
# what that image contains.
FROM toolchain AS notices
COPY . .
RUN cargo install cargo-about --locked --features cli
# .git is out of context (see .dockerignore), so gen_notices.sh cannot resolve our
# own commit for section 7's relink pointer; docker.yml passes the real sha here.
ARG GITSHA=unknown
ENV LATEXML_SELF_REV=${GITSHA}
RUN set -ex \
 && tools/gen_notices.sh /tmp/THIRD-PARTY-NOTICES.cli \
 && NOTICES_CARGO_FEATURES="--no-default-features --features cortex" \
      tools/gen_notices.sh /tmp/THIRD-PARTY-NOTICES.worker
# Prove it rather than assume it -- the same readback make_release.sh does on the
# .deb. A truncated notice is invisible from outside: the image runs fine either way.
RUN set -ex; for f in /tmp/THIRD-PARTY-NOTICES.cli /tmp/THIRD-PARTY-NOTICES.worker; do \
      for needle in "3.2 libkpathsea" "3.3 libmarpa" "5. RUST DEPENDENCY LICENSES" \
                    "6. COPYLEFT LICENSE TEXTS" "7. SOURCE PROVENANCE"; do \
        grep -qF "$needle" "$f" || { echo "$f is missing: $needle" >&2; exit 1; }; \
      done; \
    done

# ===========================================================================
# CorTeX fleet worker (--target worker)
# ===========================================================================
# build-worker: cortex_worker carries `required-features = ["cortex"]`, so the
# feature set is MANDATORY. `--no-default-features` drops test-utils; the worker
# uses compiled-in bindings, so `runtime-bindings` is unused. Profile
# `maxperf-cortex` = maxperf's real levers (opt-level=3, fat LTO, CGU=1) but with
# `panic = "unwind"` restored — MANDATORY, because the worker relies on
# `catch_unwind` to isolate per-paper panics (maxperf's abort would crash the
# whole fleet child on one bad paper). Also build latexml_oxide: its `--init`
# generates the kernel dumps baked into the runtime stage (cortex_worker has no
# --init of its own); it rides the same profile / target dir / dep graph.
FROM toolchain AS build-worker
COPY . .
# Deterministic telemetry/version provenance: the build context excludes `.git`
# (see `.dockerignore`), so build.rs can't derive the sha itself — stamp it via a
# build-arg that build.rs reads first (`LATEXML_GIT_SHA_OVERRIDE`). `unknown` is the
# honest default for a bare `docker build`; CI passes the real short sha.
ARG GITSHA=unknown
ENV LATEXML_GIT_SHA_OVERRIDE=${GITSHA}
RUN cargo build --profile maxperf-cortex --no-default-features --features cortex \
      --bin cortex_worker --bin latexml_oxide

FROM texbase AS worker
ARG HOSTTIME
ENV DOCKER_BUILD_TIME=$HOSTTIME
# The worker binary + its resources (XSLT/CSS/RelaxNG), plus latexml_oxide (only
# to generate dumps). maxperf-cortex outputs live in target/maxperf-cortex/.
COPY --from=build-worker /src/target/maxperf-cortex/cortex_worker /usr/local/bin/
COPY --from=build-worker /src/target/maxperf-cortex/latexml_oxide /usr/local/bin/
COPY --from=build-worker /src/resources/ /usr/local/share/latexml-oxide/resources/
# Redistributing the binary (and, via resources/, the W3C/Mozilla SVG schemas)
# obliges us to carry their terms with them. Generated for THIS image's feature set.
COPY --from=build-worker /src/LICENSE /usr/local/share/doc/latexml-oxide/LICENSE
COPY --from=notices /tmp/THIRD-PARTY-NOTICES.worker \
     /usr/local/share/doc/latexml-oxide/THIRD-PARTY-NOTICES
# Bake the ambient-year TeX kernel dumps into the image (the Rust analog of the
# Perl image's `make formats`). `.gitignore`/`.dockerignore` ship no dumps, so
# without this every one-conversion child re-bootstraps the kernel (~2 s/paper).
# `latexml_oxide --init` writes plain/latex.<year>.dump.txt under CWD/resources/
# dumps; the worker reads them via LATEXML_DUMP_DIR (disk, not embedded — fine
# inside a fixed image).
ENV LATEXML_DUMP_DIR=/usr/local/share/latexml-oxide/resources/dumps
RUN cd /usr/local/share/latexml-oxide && mkdir -p resources/dumps \
    && latexml_oxide --init=plain.tex \
    && latexml_oxide --init=latex.ltx \
    && ls -la resources/dumps/*.dump.txt
COPY docker/cortex-worker-entrypoint.sh /usr/local/bin/cortex-worker-entrypoint.sh
RUN chmod +x /usr/local/bin/cortex-worker-entrypoint.sh
# Disk-backed scratch — bind-mount a host dir here at run (see header).
RUN mkdir -p /opt/cortex-scratch
ENV TMPDIR=/opt/cortex-scratch
ENTRYPOINT ["cortex-worker-entrypoint.sh"]

# ===========================================================================
# General-purpose CLI (--target cli) — DEFAULT target (last stage)
# ===========================================================================
# build-cli: the distribution recipe (drop test-utils, keep runtime-bindings,
# maxperf). make_formats runs FIRST — it builds a debug latexml_oxide and emits
# the two --init dumps (plain.tex + latex.ltx) into resources/dumps/; the maxperf
# build that follows EMBEDS them (latexml_engine/build.rs scans resources/dumps/).
# Order matters: dumps must exist BEFORE the embedding build, so the shipped
# single-file binary is self-contained — nothing read from our own resources.
FROM toolchain AS build-cli
COPY . .
# Deterministic provenance (see build-worker) — `.git` is out of context.
ARG GITSHA=unknown
ENV LATEXML_GIT_SHA_OVERRIDE=${GITSHA}
RUN set -ex \
 && tools/make_formats.sh \
 && cargo build --no-default-features --features runtime-bindings \
      --profile maxperf --bin latexml_oxide \
 && strip --strip-all target/maxperf/latexml_oxide \
 && cp target/maxperf/latexml_oxide /usr/local/bin/latexml_oxide

FROM texbase AS cli
COPY --from=build-cli /usr/local/bin/latexml_oxide /usr/local/bin/latexml_oxide
# This image redistributes the binary, so it carries the binary's terms: our CC0
# LICENSE plus the third-party notices (incl. the section 6 copyleft texts the
# statically linked LGPL code obliges and the section 7 relink pointer).
COPY --from=build-cli /src/LICENSE /usr/local/share/doc/latexml-oxide/LICENSE
COPY --from=notices /tmp/THIRD-PARTY-NOTICES.cli \
     /usr/local/share/doc/latexml-oxide/THIRD-PARTY-NOTICES
# Build-time self-test: this stage carries NO repo/resources tree, so a
# successful HTML5 conversion proves the shipped binary is self-contained —
# embedded kernel dump + embedded XSLT/CSS/schema — and resolves article.cls from
# the image's TeX Live. Fails the build if the image cannot convert.
RUN set -ex \
 && printf '\\documentclass{article}\n\\begin{document}\nHello from the latexml-oxide container.\n\\end{document}\n' > /tmp/hello.tex \
 && latexml_oxide --dest=/tmp/hello.html /tmp/hello.tex \
 && grep -q 'Hello from the latexml-oxide container' /tmp/hello.html \
 && rm -f /tmp/hello.tex /tmp/hello.html
# Bind-mount your document tree here: `-v "$PWD:/work"`.
WORKDIR /work
ENTRYPOINT ["latexml_oxide"]
