# Releasing

The release process for latexml-oxide is automated end-to-end: bump the
version, write the changelog, push a tag. GitHub Actions builds the
artifacts and publishes them to a new GitHub Release. The Linux assets
build on `ubuntu-22.04` (glibc 2.35, broadest binary compatibility); the
macOS asset builds natively on `macos-15` (Apple Silicon).

Currently published platforms: **`x86_64-unknown-linux-gnu`** and
**`aarch64-apple-darwin`** (macOS Apple Silicon). Intel macOS
(`x86_64-apple-darwin`), `aarch64` Linux, Windows, and musl are
out of scope for now — see "Release asset strategy" below.

## Release asset strategy

**A native binary is never cross-OS.** Linux compiles to ELF linked
against glibc; macOS compiles to Mach-O linked against libSystem. They
are not interchangeable at any level, so there is exactly **one artifact
per `(OS, arch)` target triple** — no single download serves both. The
asset filenames encode the triple (`…-<triple>.tar.gz`), so the scheme
scales to new targets without renaming.

What this means concretely:

- **Per-triple build legs, not cross-compilation.** Each platform builds
  on its own native runner (`release.yml`: the Linux `release` job, the
  `build-linux-arm64` job, and the `build-macos` / `build-macos-intel` jobs).
  We do *not* cross-compile: each leg source-builds its own PIC static
  libxml2/libxslt/libkpathsea for the native toolchain (ELF vs Mach-O) and
  statically links them in.
- **Linux = x86_64 + aarch64, as both a tarball and a `.deb` each.** The
  arm64 leg (`build-linux-arm64`, `ubuntu-22.04-arm`) is a full build+gate
  peer of the x86_64 `release` leg — same static linkage, `ldd`
  self-contained check, conversion + embedded-resource smokes, and 64 MB size
  budget — for AWS Graviton / Ampere / Raspberry Pi OS 64-bit / Apple-Silicon
  Linux VMs. `cargo deb` derives the control-file `Architecture:` from the
  native host; `make_release.sh` labels the filename `arm64` (vs `amd64`) from
  `RELEASE_TARGET`.
- **macOS = Apple Silicon (arm64) + Intel (x86_64), as separate tarballs.**
  arm64 is the arch the CI suite validates (`CI.yml` macOS job runs on arm64
  `macos-15`, #217). An arm64 binary will **not** run on an Intel Mac (Rosetta
  only translates the other direction), so Intel gets its own native leg
  (`build-macos-intel`) rather than a cross-compile or `lipo` universal binary.
  - **Intel runner:** `macos-15-intel`. GitHub retired the `macos-13` Intel
    image on 2025-12-04; `macos-15-intel` is the **last free-tier x86_64 macOS
    image**, available until ~Fall 2027, after which GitHub Actions drops Intel
    macOS entirely. **When that lands, revisit:** switch to a `lipo` universal
    binary built by cross-compiling x86_64 on the arm64 runner (the static C
    deps would need `-arch x86_64` in their `CFLAGS`/`--host`), or a
    self-hosted Intel Mac.
  - **Deployment target:** the Intel leg pins `MACOSX_DEPLOYMENT_TARGET=10.13`
    so the binary runs on older Intel Macs (e.g. a 2018 MacBook Air, which
    shipped with 10.14 and tops out at Sonoma 14) even though the runner's SDK
    is macOS 15.
- **Distribution linkage (self-contained):** the CLI assets STATICALLY link
  libxml2 + libxslt + libexslt (source-built PIC,
  `tools/build_static_libxml.sh`) and — on Linux — libkpathsea
  (`tools/build_static_kpathsea.sh`, in-process lookups). The binary carries
  NO versioned libxml2/libxslt SONAME dependency, so it is independent of the
  host's libxml2 era: libxml2 2.14 bumped the SONAME `.so.2` → `.so.16`, and a
  dynamically-linked binary loads on only one side of that split. On macOS,
  kpathsea stays the **subprocess-`kpsewhich` backend** of `kpathsea` 0.3 —
  mandatory on MacTeX (ships no libkpathsea). Only the glibc/libSystem family
  remains dynamic. Our *own* resources (XSLT/CSS/JS/schema/dumps) are always
  embedded; see the portability note below. A `release.yml` step
  `ldd`/`otool`-asserts the absence of dynamic libxml2/libxslt/kpathsea and
  fails the release otherwise.
- **Editor-distributed binary:** the stricter "no host libxml2/libxslt" bar
  (`RELEASE_CRITERIA.md` §11) is now MET by these same CLI assets, so a VSCode
  extension can ship the binary directly.
- **Deferred matrix rungs:** a macOS `lipo` universal binary (folding the two
  macOS tarballs into one — see the Intel runner sunset note above), then
  Windows/musl (`RELEASE_CRITERIA.md` §3). (`aarch64-unknown-linux-gnu`
  landed 2026-07-09 as the `build-linux-arm64` leg.) Each new triple is one
  more native build leg in `release.yml` plus a `RELEASE_TARGET=<triple>`
  invocation of `tools/make_release.sh`.

## What ships in a release

Assets attached to each `X.Y.Z` GitHub Release — four platform builds (two
Linux, two macOS), each a tarball (+ `.sha256`), plus a `.deb` (+ `.sha256`)
for each Linux arch, plus the aggregate `THIRD-PARTY-NOTICES`:

| Asset | Purpose |
|---|---|
| `latexml-oxide-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz` | Portable Linux (x86_64) archive: stripped `latexml_oxide` binary, `README.md`, `CHANGELOG.md`, `LICENSE`, `THIRD-PARTY-NOTICES`. |
| `latexml-oxide-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz.sha256` | SHA-256 sidecar (ripgrep-style). |
| `latexml-oxide_X.Y.Z-1_amd64.deb` | Debian package (x86_64). With libxml2/libxslt/kpathsea statically linked, `$auto` resolves to just the glibc family — `Depends:` carries NO libxml2/libxslt SONAME, only the graphics/TeX tools (`imagemagick`, `mupdf-tools`, `poppler-utils`, `ghostscript`, `dvipng`, `dvisvgm`, `texlive-latex-{base,extra}`, `texlive-science`). |
| `latexml-oxide_X.Y.Z-1_amd64.deb.sha256` | SHA-256 sidecar. |
| `latexml-oxide-X.Y.Z-aarch64-unknown-linux-gnu.tar.gz` | Portable Linux (aarch64 / arm64) archive: same contents + self-contained linkage as the x86_64 tarball. For AWS Graviton, Ampere, Raspberry Pi OS 64-bit, Apple-Silicon Linux VMs. |
| `latexml-oxide-X.Y.Z-aarch64-unknown-linux-gnu.tar.gz.sha256` | SHA-256 sidecar. |
| `latexml-oxide_X.Y.Z-1_arm64.deb` | Debian package (aarch64). Same `Depends:` as the amd64 `.deb`; `cargo deb` sets `Architecture: arm64` from the native arm64 build host. |
| `latexml-oxide_X.Y.Z-1_arm64.deb.sha256` | SHA-256 sidecar. |
| `latexml-oxide-X.Y.Z-aarch64-apple-darwin.tar.gz` | Portable macOS (Apple Silicon) archive: same contents as the Linux tarball. No `.deb` on macOS (a Homebrew tap is the natural future analogue). |
| `latexml-oxide-X.Y.Z-aarch64-apple-darwin.tar.gz.sha256` | SHA-256 sidecar. |
| `latexml-oxide-X.Y.Z-x86_64-apple-darwin.tar.gz` | Portable macOS (Intel) archive: built with a macOS 10.13 deployment target so it runs on older Intel Macs. |
| `latexml-oxide-X.Y.Z-x86_64-apple-darwin.tar.gz.sha256` | SHA-256 sidecar. |
| `THIRD-PARTY-NOTICES` | Aggregate license notices (hand-authored §1–4 + the cargo-about Rust-crate appendix). |

The shipped `latexml_oxide` binary is fully self-contained — XSLT
stylesheets, CSS, JavaScript, and the RelaxNG schema tree are
embedded at build time (`include_str!` / `include_bytes!`). Format
dumps for a **5-year moving TeX Live window** (currently 2022–2026)
are also embedded. They are NOT in git: `release.yml` first calls
`release-dumps.yml`, which generates each year's
`{plain,latex}.YYYY.dump.txt` + `texlive.YYYY.version` inside a pinned
TL-year container (`ghcr.io/tkw1536/texlive-docker:YYYY` — the image
family behind Perl LaTeXML's CI) under a strict zero-error `--init`
gate, then **every** build leg (Linux x86_64 + aarch64, macOS arm64 +
Intel) downloads the full window into `resources/dumps/` and verifies
completeness before building. The dumps are OS/arch-agnostic gzipped text,
so all four binaries embed the exact same bytes. Every leg builds with
`--profile maxperf` (`release.yml`), so each platform's download is one
optimized, self-contained artifact.

**Design requirement — portability.** A conversion must not *read* any of
latexml_oxide's *own* resources from disk during its main operation: the
binary carries them all and serves them from memory (XSLT/CSS/JS/schema via
the `embed:///` libxml2 input callback, format dumps via `include_str!`).
*Writing* files into the destination directory is expected and fine. This is
verified end-to-end: XSLT resolves with zero `.xsl` disk reads (`strace`),
and renaming the dev-tree `resources/dumps/` away still converts successfully
from the embedded dumps. The host **TeX Live ecosystem is out of scope** —
reading `.sty`/`.cls`/`.tfm` from the user's texmf tree via `kpathsea` is
allowed and expected (see the runtime-dependency note below). See the
"Self-contained, portable binary" principle in
[`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md).

The churn-prone C libraries are STATICALLY linked, so only the platform's
core runtime stays dynamic:

- **Linux**: `libc`, `libm`, `libgcc_s`, `ld-linux` (the glibc family) —
  nothing else. libxml2/libxslt/libexslt and libkpathsea are baked in
  (verified by `ldd` in `release.yml`), so the binary runs on any
  glibc-2.35+ host regardless of its libxml2 SONAME (or the absence of one).
- **macOS**: `libSystem` plus the bundled libxml2/libxslt. `kpathsea` is
  *not* linked on macOS — the binary resolves TeX paths through the
  subprocess-`kpsewhich` backend (works with both Homebrew TeX Live and
  MacTeX/BasicTeX).

TeX Live (`kpsewhich`, `pdflatex`) is required at runtime and not
bundled on any platform.

## Release procedure

1. **Bump the version** in `latexml_oxide/Cargo.toml`:

    ```toml
    [package]
    version = "X.Y.Z"   # was: X.Y.(Z-1)
    ```

   Other workspace crates use their own version cadence and do not need
   bumping in lockstep.

2. **Add a CHANGELOG entry** at the top of `CHANGELOG.md`:

    ```markdown
    ## [X.Y.Z] YYYY-MM-DD

    - …summary of user-visible changes…
    ```

   `tools/make_release.sh` slices this section into the GitHub release
   body, falling back to a plain link to the full changelog if the
   matching header is missing.

3. **Sanity-check the release build locally** (catches build-script,
   `cargo-deb` metadata, and embedded-resource regressions before they
   reach CI):

    ```bash
    cargo install cargo-deb     # one-time per developer
    bash tools/make_release.sh
    ```

   On success the script prints the artifact paths and their SHA-256
   hashes. On Linux it produces the tarball + `.deb` (+ sidecars) and the
   shared `RELEASE_BODY.md`; spot-check the `.deb`:

    ```bash
    dpkg-deb -I target/release-artifacts/latexml-oxide_X.Y.Z-1_amd64.deb
    dpkg-deb -c target/release-artifacts/latexml-oxide_X.Y.Z-1_amd64.deb
    ```

   The macOS leg is produced by CI's `build-macos` job, not this local
   Linux dry-run. To dry-run it on an Apple Silicon Mac (no `.deb`, no
   release body — that's emitted only by the Linux publishing leg):

    ```bash
    RELEASE_TARGET=aarch64-apple-darwin bash tools/make_release.sh
    ```

4. **Commit, tag, push.** Tag format is bare `X.Y.Z` — no `v` prefix —
   matching the existing tag history.

    ```bash
    git commit -am "release: X.Y.Z"
    git tag X.Y.Z
    git push origin master
    git push origin X.Y.Z
    ```

5. **Watch the workflow.** `Release` on the Actions tab. It runs four
   jobs: `dumps` (TL-window generation) → `build-macos` (Apple Silicon,
   `macos-15`) ‖ `build-macos-intel` (Intel, `macos-15-intel`) → `release`
   (Linux tarball + `.deb`, then collects both macOS artifacts and attaches
   all **eight** assets). The Intel-macOS leg's fat-LTO `maxperf` build on the
   slower `macos-15-intel` runner is the long pole (up to ~120 min budget).

6. **Publish the draft.** The workflow attaches the assets to a **draft**
   Release (not public — see `release.yml` `draft: true`). Open it under
   *Releases* → download and sanity-check each tarball on its target hardware
   **before** publishing. In particular the **Intel-macOS** asset
   (`…-x86_64-apple-darwin.tar.gz`) is built on a different runner than the
   arm64 leg — verify `latexml_oxide --version` and a real conversion run on an
   actual Intel Mac. When satisfied, click **Publish release**. (Flip
   `release.yml` back to `draft: false` for a target once it's proven, if you
   prefer auto-publish.)

## Failure recovery

* **Workflow fails before publishing.** Fix the underlying issue,
  delete the tag (`git tag -d X.Y.Z && git push origin :refs/tags/X.Y.Z`),
  push the fix, re-tag.

* **Workflow publishes a broken release.** Don't reuse the tag. Bump
  to `X.Y.(Z+1)` and start over — the deleted-then-replaced tag flow
  is confusing for downstream installers that may have already cached
  the broken assets.

* **`make_release.sh` rejects `GITHUB_REF_NAME`-vs-Cargo.toml mismatch.**
  This means you tagged before bumping. Bump `latexml_oxide/Cargo.toml`,
  amend the commit (or add a follow-up), and re-tag.

* **`build-macos` OOMs / times out.** The fat-LTO `maxperf` link is the
  RAM peak; the free-tier `macos-15` runner is only 7 GB. If it OOMs,
  move that leg to a larger runner (`macos-15-xlarge`, paid) or relax the
  macOS leg's `codegen-units` for the link step. Do **not** drop the
  whole macOS asset silently — a missing target is a regression.

## Extending the workflow

* **New target triple.** Native binaries are never cross-OS, so each new
  triple is its own native build leg, not a cross-compile. Add a job in
  `.github/workflows/release.yml` modeled on `build-macos` (right runner,
  platform deps, dump download, `RELEASE_TARGET=<triple> bash
  tools/make_release.sh`, upload artifact), then collect its artifact in
  the `release` job and add its files to the publish `files:` list.
  `tools/make_release.sh` already accepts `RELEASE_TARGET` and derives
  per-OS behavior (strip flags, `.deb` only on Linux, sha256 tool); it
  builds **natively** (no `--target`), so `RELEASE_TARGET` must match the
  runner's host arch. For a macOS universal binary, add a `macos-13`
  (Intel) leg and `lipo`-merge its slice with the arm64 slice before
  staging.

* **Skip the .deb**: comment out the `cargo deb …` block in
  `tools/make_release.sh` and drop the `.deb` lines from the
  `files:` list in `.github/workflows/release.yml`.

* **Embed additional resources**: add `include_str!` / `include_bytes!`
  hooks at the call site, then refresh the smoke test
  (`latexml_oxide/tests/001_single_binary_smoke.rs`) to assert the
  new asset appears.
