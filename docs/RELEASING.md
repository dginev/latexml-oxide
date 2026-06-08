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
  on its own native runner (`release.yml`: the Linux `release` job +
  the `build-macos` job). We do *not* cross-compile, because the C deps
  (libxml2/libxslt) link against the host's libraries.
- **macOS = Apple Silicon (arm64) only, for now.** That's the arch the
  CI suite validates (`CI.yml` macOS job runs on arm64 `macos-15`, #217).
  An arm64 binary will **not** run on an Intel Mac (Rosetta only
  translates the other direction). Adding Intel means either a separate
  `x86_64-apple-darwin` tarball or a `lipo` universal binary — both
  require a `macos-13` (Intel) build leg to validate, deferred until
  there's demand.
- **Distribution linkage:** the CLI assets dynamically link host-provided
  libxml2/libxslt (Linux: `.deb` `Depends:` / apt; macOS: `brew install
  libxml2 libxslt`) and resolve TeX paths via the **subprocess-`kpsewhich`
  backend** of `kpathsea` 0.3 — ABI-decoupled across TeX Live years and
  mandatory on MacTeX (ships no libkpathsea). Our *own* resources
  (XSLT/CSS/JS/schema/dumps) are always embedded; see the portability
  note below.
- **Editor-distributed binary is a stricter bar** (VSCode extension): it
  cannot assume host libxml2/libxslt and must statically link/vendor
  them. That is a separate track (`RELEASE_CRITERIA.md` §11), not these
  CLI assets.
- **Deferred matrix rungs:** `aarch64-unknown-linux-gnu`, a macOS
  universal/Intel slice, then Windows/musl (`RELEASE_CRITERIA.md` §3).
  Each new triple is one more native build leg in `release.yml` plus a
  `RELEASE_TARGET=<triple>` invocation of `tools/make_release.sh`.

## What ships in a release

Six files attached to each `X.Y.Z` GitHub Release:

| Asset | Purpose |
|---|---|
| `latexml-oxide-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz` | Portable Linux archive: stripped `latexml_oxide` binary, `README.md`, `CHANGELOG.md`, `LICENSE`. |
| `latexml-oxide-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz.sha256` | SHA-256 sidecar (ripgrep-style). |
| `latexml-oxide_X.Y.Z-1_amd64.deb` | Debian package with declared runtime `Depends:` on `libxml2`, `libxslt1.1`, `libkpathsea6`, `texlive-latex-{base,extra}`, `texlive-science`. |
| `latexml-oxide_X.Y.Z-1_amd64.deb.sha256` | SHA-256 sidecar. |
| `latexml-oxide-X.Y.Z-aarch64-apple-darwin.tar.gz` | Portable macOS (Apple Silicon) archive: same contents as the Linux tarball. No `.deb` on macOS (a Homebrew tap is the natural future analogue). |
| `latexml-oxide-X.Y.Z-aarch64-apple-darwin.tar.gz.sha256` | SHA-256 sidecar. |

The shipped `latexml_oxide` binary is fully self-contained — XSLT
stylesheets, CSS, JavaScript, and the RelaxNG schema tree are
embedded at build time (`include_str!` / `include_bytes!`). Format
dumps for a **5-year moving TeX Live window** (currently 2022–2026)
are also embedded. They are NOT in git: `release.yml` first calls
`release-dumps.yml`, which generates each year's
`{plain,latex}.YYYY.dump.txt` + `texlive.YYYY.version` inside a pinned
TL-year container (`ghcr.io/tkw1536/texlive-docker:YYYY` — the image
family behind Perl LaTeXML's CI) under a strict zero-error `--init`
gate, then **both** the Linux `release` job and the `build-macos` job
download the full window into `resources/dumps/` and verify completeness
before building. The dumps are OS-agnostic gzipped text, so the macOS
binary embeds the exact same bytes. Both legs build with `--profile
maxperf` (`release.yml`), so each platform's download is one optimized,
self-contained artifact.

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

System libraries remain dynamically linked, host-provided:

- **Linux**: `libxml2`, `libxslt1.1`, `libkpathsea6` — installed by the
  `.deb`'s `Depends:` or the tarball user's own apt invocation.
- **macOS**: `libxml2`, `libxslt` via `brew install libxml2 libxslt`.
  `kpathsea` is *not* linked on macOS — the binary resolves TeX paths
  through the subprocess-`kpsewhich` backend (works with both Homebrew
  TeX Live and MacTeX/BasicTeX, neither of which the binary needs to
  link against).

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

5. **Watch the workflow.** `Release` on the Actions tab. It runs three
   jobs: `dumps` (TL-window generation) → `build-macos` (Apple Silicon
   tarball) ‖ `release` (Linux tarball + `.deb`, then collects the macOS
   artifact and publishes all six assets). Typical duration: 15–25 min
   cold (the macOS leg's fat-LTO `maxperf` build on the 7 GB `macos-15`
   runner is the long pole), faster warm. On success the new release
   appears at <https://github.com/dginev/latexml-oxide/releases/tag/X.Y.Z>.

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
