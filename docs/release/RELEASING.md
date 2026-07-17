# Releasing

The release process for latexml-oxide is automated end-to-end: bump the
version, write the changelog, push a tag. GitHub Actions builds the
artifacts and publishes them to a new GitHub Release. The Linux assets
build on `ubuntu-22.04` (glibc 2.35, broadest binary compatibility); the
macOS asset builds natively on `macos-15` (Apple Silicon).

Currently published platforms: **`x86_64-unknown-linux-gnu`**,
**`aarch64-unknown-linux-gnu`**, **`aarch64-apple-darwin`** (macOS Apple
Silicon), and **`x86_64-apple-darwin`** (macOS Intel). **`x86_64-pc-windows-msvc`**
joins at **`0.7.4`** as a single self-contained `.exe`, shipped in a `.zip`. The
`.exe` itself validated in the `0.7.4-rc2` RC draft; the `.zip` packaging around it
is new in 0.7.4 and **first exercised at the next RC tag** — no RC has published
one yet. See [`WINDOWS_COMPATIBILITY_PLAN.md`](WINDOWS_COMPATIBILITY_PLAN.md). Only musl
remains out of scope for now — see "Release asset strategy" below.

## Release targets & order

Five targets. Only the first is triggered by the tag; the rest hang off the
**published GitHub Release**, which is why the order is forced rather than a
preference — Homebrew and crates.io both need the release's assets/sha256s to exist.

```
tag X.Y.Z  →  release.yml  →  GitHub Release (public)          ← the only tag-triggered step
                                  │
                                  ├─ AUTOMATIC  docker.yml → ghcr.io (cli + cortex-worker)
                                  │             fires on `release: types: [published]`
                                  ├─ MANUAL     dginev/homebrew-tap: ./update-formula.sh X.Y.Z
                                  │             (reads the release's macOS .sha256 sidecars)
                                  ├─ MANUAL     cargo publish --workspace   (needs the repo PUBLIC)
                                  └─ MANUAL     ar5iv-editor deploy → latexml.rs
```

| # | Target | How | Doc |
|---|---|---|---|
| 1 | **GitHub Release** (8 assets) | tag push → `release.yml` | this file |
| 2 | **Container images** (ghcr.io) | **automatic** on Release *publish* → `docker.yml` | "Container images (GHCR)" below |
| 3 | **Homebrew tap** | manual: `dginev/homebrew-tap` → `./update-formula.sh X.Y.Z`, commit, push | that repo's README |
| 4 | **crates.io** (8 crates) | manual: `cargo publish --workspace` | [`CRATES_IO_PUBLISH.md`](CRATES_IO_PUBLISH.md) |
| 5 | **latexml.rs** (ar5iv-editor) | manual: that repo's `deploy/build-and-push.sh` + cloud rollout | ar5iv-editor `deploy/` |

Also automatic, no action needed: `rustdoc.yml` republishes the gh-pages rustdoc on
every push to `main` — that is what crates.io's `documentation` link points at.

**Don't forget the tap.** The README tells macOS users `brew install
dginev/tap/latexml-oxide`. If the formula isn't bumped, every Homebrew user silently
keeps getting the *previous* version while the release page advertises the new one.

## RC tags vs final tags

`release.yml` decides purely on whether the tag contains a `-`:

```yaml
draft:      ${{ contains(github.ref_name, '-') }}
prerelease: ${{ contains(github.ref_name, '-') }}
```

* **`X.Y.Z-rcN`** → a **DRAFT prerelease**. Artifacts only, for cross-OS testing.
* **`X.Y.Z`** → a **public** Release, auto-published.

Two consequences worth knowing before you plan an RC:

* **Targets 2-5 do not happen for an RC.** `docker.yml` listens for `release:
  published`, which does **not** fire for a draft. To exercise the container path from
  an RC, use its `workflow_dispatch` with the tag.
* **Never publish an RC to crates.io.** `cargo install` and `latexml = "0.7"` both
  ignore pre-releases, so an `-rcN` upload leaves `cargo install latexml` resolving to
  whatever stable version exists (today: the ancient `0.0.2` placeholder) — while the
  README *inside that crate* tells people to run exactly that.

### "Can I just promote the RC draft to latest?"

**No — cut a fresh `X.Y.Z` tag.** The version string is baked into the *artifacts*, not
just the Release object, so flipping draft→published re-labels nothing:

* assets are named `…-X.Y.Z-rcN-…`; the README's `VERSION=X.Y.Z` instructions 404,
* the `.deb` carries `X.Y.Z-rcN-1`, so a later real `X.Y.Z` is a *newer* apt version,
* the binary reports `X.Y.Z-rcN` from `--version` — which **fails the Homebrew
  formula's** `assert_match version.to_s, shell_output("#{bin}/latexml_oxide --version")`,
* the tap's formula fetches `…-#{version}-…tar.gz`, which does not exist for a plain
  `X.Y.Z` against RC assets,
* crates.io needs a stable version regardless (above).

`make_release.sh` enforces this from the other side: it refuses to build when the tag
does not equal `latexml_oxide/Cargo.toml`'s version. So an RC is a **test artifact**.
When happy: bump `Cargo.toml` `X.Y.Z-rcN` → `X.Y.Z`, tag `X.Y.Z`, and delete the RC
draft. The CHANGELOG's `## [X.Y.Z]` header then also starts matching, so the release
body carries real notes instead of the generic fallback.

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
  `tools/build_static_libxml.sh`) and libkpathsea on **every** leg (Linux + macOS
  `tools/build_static_kpathsea.sh`; Windows `kpathsea_sys` `build_from_source` —
  all in-process lookups). The binary carries NO versioned
  libxml2/libxslt SONAME dependency, so it is independent of the host's libxml2
  era (libxml2 2.14 bumped the SONAME `.so.2` → `.so.16`; a dynamically-linked
  binary loads on only one side of that split). At *runtime* `select_kpaths` may still fall back to the
  **subprocess-`kpsewhich` backend** where the linked-in kpathsea can't serve the
  host (e.g. MiKTeX, whose fndb a static libkpathsea can't read) — a runtime
  choice, which does **not** change the fact that the library is linked in. Only the glibc/libSystem
  family remains dynamic — and on Windows even the CRT is static (`+crt-static`),
  so the `.exe` imports only core OS DLLs. (libkpathsea is LGPL-2.1, so **every** published binary carries the §6 relink
  obligation — discharged per `LICENSE_INVENTORY.md` §D.3 + `THIRD-PARTY-NOTICES` §7.) Our
  *own* resources (XSLT/CSS/JS/schema/dumps) are always embedded; see the
  portability note below. A `release.yml` step `ldd`/`otool`/`dumpbin`-asserts the
  absence of dynamic libxml2/libxslt/kpathsea (and, on Windows, VCRUNTIME) and
  fails the release otherwise.
- **Editor-distributed binary:** the stricter "no host libxml2/libxslt" bar
  (`RELEASE_CRITERIA.md` §11) is now MET by these same CLI assets, so a VSCode
  extension can ship the binary directly.
- **Deferred matrix rungs:** a macOS `lipo` universal binary (folding the two
  macOS tarballs into one — see the Intel runner sunset note above), then musl
  (`RELEASE_CRITERIA.md` §3). (`aarch64-unknown-linux-gnu` landed 2026-07-09 as
  `build-linux-arm64`; `x86_64-pc-windows-msvc` landed 2026-07-14 as
  `build-windows` — a single fully-static `.exe`.) Each new triple is one more
  native build leg in `release.yml` plus a `RELEASE_TARGET=<triple>` invocation
  of `tools/make_release.sh`.

## What ships in a release

Assets attached to each `X.Y.Z` GitHub Release — five platform builds (two
Linux, two macOS, one Windows). Linux/macOS ship a tarball (+ `.sha256`);
Windows ships a `.zip` (+ `.sha256`) — no `.deb` equivalent. Every archive
carries the binary plus `THIRD-PARTY-NOTICES`, `LICENSE`, `README.md` and
`CHANGELOG.md`. Plus a `.deb` (+ `.sha256`) for each Linux arch, plus the
aggregate `THIRD-PARTY-NOTICES` as a standalone asset:

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
| `latexml-oxide-X.Y.Z-aarch64-apple-darwin.tar.gz` | Portable macOS (Apple Silicon) archive: same contents as the Linux tarball. No `.deb` on macOS; the Homebrew tap (`dginev/homebrew-tap`) is the analogue — bump it per release, see "Release targets & order". |
| `latexml-oxide-X.Y.Z-aarch64-apple-darwin.tar.gz.sha256` | SHA-256 sidecar. |
| `latexml-oxide-X.Y.Z-x86_64-apple-darwin.tar.gz` | Portable macOS (Intel) archive: built with a macOS 10.13 deployment target so it runs on older Intel Macs. |
| `latexml-oxide-X.Y.Z-x86_64-apple-darwin.tar.gz.sha256` | SHA-256 sidecar. |
| `latexml-oxide-X.Y.Z-x86_64-pc-windows-msvc.zip` | Windows (x86_64) archive: a single fully-static `latexml_oxide.exe` (`+crt-static`; static libxml2/libxslt/libkpathsea via `build_from_source`) — imports only core OS DLLs, no VC++ redistributable — plus the same `README.md`/`CHANGELOG.md`/`LICENSE`/`THIRD-PARTY-NOTICES` as the tarballs. Unzip and run; TeX Live or MiKTeX on PATH for host TeX resolution. |
| `latexml-oxide-X.Y.Z-x86_64-pc-windows-msvc.zip.sha256` | SHA-256 sidecar. |
| `THIRD-PARTY-NOTICES` | Aggregate license notices: hand-authored §1–4 (embedded TeX dumps; Perl-LaTeXML assets; the linked native libs incl. the vendored libmarpa/mimalloc and static-LGPL kpathsea) + the cargo-about Rust-crate appendix (§5) + the verbatim copyleft texts (§6) + the per-artifact source provenance for LGPL relinking (§7). Assembled once by the `notices` job and bundled byte-identically into every archive **and both `.deb`s** (the `.deb` gets it via `make_release.sh` swapping the path `cargo deb` reads, then reading the notices back out of the built package to prove it). The **container images** carry their own copy, generated by the Dockerfile's `notices` stage per-image — the CLI and cortex-worker link different feature graphs, so their §5 differs. |

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
Intel, Windows x86_64) downloads the full window into `resources/dumps/` and
verifies completeness before building. The dumps are OS/arch-agnostic gzipped
text, so all five binaries embed the exact same bytes. Every leg builds with
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
[`OXIDIZED_DESIGN.md`](../parity/OXIDIZED_DESIGN.md).

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

## macOS Gatekeeper & code signing

**Decision (2026-07-13): ripgrep-style — ad-hoc signature only, NOT
notarized.** We deliberately do *not* enroll in the Apple Developer Program
or notarize the macOS tarballs. This matches ripgrep and the vast majority
of open-source Rust CLIs, and it is correct because of how Gatekeeper
actually works:

- **The "unidentified developer" prompt fires only on files carrying the
  `com.apple.quarantine` xattr**, which is set by *quarantine-aware* apps
  (browsers, Mail, AirDrop) — **not** by `curl`, `git`, or Homebrew. Our
  install instructions (`make_release.sh` release body) use `curl`, so the
  downloaded binary has no quarantine bit and Gatekeeper never prompts. A
  user who instead downloads the tarball *in a browser* will see the prompt;
  they clear it once with `xattr -d com.apple.quarantine <file>` or
  right-click → Open.
- **Code signing ≠ notarization.** A Developer ID signature alone no longer
  clears Gatekeeper for quarantined files (since Catalina); *notarization*
  (uploading to Apple's notary service) is what removes the browser-download
  warning. Both require the $99/yr program. We do neither.
- **arm64 needs *a* signature just to execute.** Apple Silicon kills any
  arm64 Mach-O without at least an ad-hoc signature. Because our macOS legs
  build **natively** (`macos-15` / `macos-15-intel`), the linker ad-hoc-signs
  automatically — but `strip` can invalidate that. So `make_release.sh`
  **re-applies an ad-hoc signature after strip** (`codesign --sign - --force`,
  dash = ad-hoc, no cert), and both macOS legs gate on
  `codesign --verify` (`verify code signature present` step). This costs
  nothing and prevents a `Killed: 9` regression.

**Primary macOS channel = Homebrew** (like ripgrep's homebrew-core). `brew`
strips quarantine, so a tap install is warning-free by construction — no
Apple Developer account, no notarization, aligned with the CC0 / public-domain
posture (a paid Apple identity would contradict the no-strings ethos, and
Homebrew removes the warning on the channel Mac users actually reach for). Not
yet published; the per-release `.sha256` sidecars make an auto-bump trivial.

The bigger win: for a LaTeX converter the real setup friction is **TeX + the
graphics toolchain**, not the executable. The formula encodes the small
always-needed graphics tools as `depends_on`, so `brew install …` sets up the
whole runtime in one command, and caveats the (large, sometimes
already-present via MacTeX) TeX distribution rather than force-installing a
redundant copy. A tap lives in its own `dginev/homebrew-tap` repo as
`Formula/latexml-oxide.rb`:

```ruby
class LatexmlOxide < Formula
  desc "Rust port of LaTeXML — LaTeX to HTML/XML/MathML"
  homepage "https://github.com/dginev/latexml-oxide"
  version "0.7.3"                       # bump per release
  license "CC0-1.0"
  # 1:1 with the .deb's graphics Depends (imagemagick, mupdf-tools,
  # poppler-utils, ghostscript, dvisvgm) — all are real homebrew-core formulae.
  # `dvipng` (a .deb Depends) has NO standalone brew formula; on macOS it ships
  # only inside TeX Live / MacTeX, so it is covered via the TeX distribution
  # (caveats below), not a `depends_on` line.
  depends_on "dvisvgm"
  depends_on "ghostscript"
  depends_on "imagemagick"
  depends_on "mupdf-tools"
  depends_on "poppler"
  on_macos do
    on_arm do
      url "https://github.com/dginev/latexml-oxide/releases/download/#{version}/latexml-oxide-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "…"                        # from the .tar.gz.sha256 sidecar
    end
    on_intel do
      url "https://github.com/dginev/latexml-oxide/releases/download/#{version}/latexml-oxide-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "…"
    end
  end
  def install
    bin.install "latexml_oxide"
  end
  # TeX Live is the one heavy runtime dep. Caveat rather than `depends_on
  # "texlive"` so users with MacTeX/BasicTeX aren't forced into a redundant
  # ~5 GB brew copy. (Swap to `depends_on "texlive"` if you'd rather the
  # zero-TeX audience get a truly one-command setup.)
  def caveats
    <<~EOS
      latexml-oxide needs a TeX distribution at runtime (kpsewhich, pdflatex,
      and dvipng — TeX Live bundles dvipng, which has no standalone brew formula).
      Install one of:
        brew install texlive                 # Homebrew's (~5 GB, full TeX Live)
        # …or MacTeX / BasicTeX: https://tug.org/mactex/
      With MacTeX/BasicTeX, put /Library/TeX/texbin on PATH.
    EOS
  end
  test do
    # Runs the binary — also proves the (ad-hoc) code signature is valid,
    # since an unsigned/broken arm64 Mach-O is killed at exec.
    assert_match version.to_s, shell_output("#{bin}/latexml_oxide --version")
  end
end
```

`brew install dginev/tap/latexml-oxide` then gives Mac users a warning-free,
on-PATH, auto-upgradeable install with the graphics runtime already wired up —
the "easy start" the plain tarball can't match. Path to the front door
(`brew install latexml-oxide`, no tap): submit to **homebrew-core** once the
project clears its notability bar; until then the personal tap is the
pragmatic channel and README should lead with it on macOS.

**If browser downloads ever become a support burden**, the upgrade is a
Developer ID sign + `xcrun notarytool submit --wait` job (needs the $99/yr
program + 5 GitHub secrets: base64 `.p12` cert, cert password, sign identity,
App Store Connect key/issuer). Note a **bare CLI binary cannot be
`stapler staple`d** (only `.app`/`.dmg`/`.pkg`), so notarization would rely on
Gatekeeper's *online* check unless wrapped in a `.pkg`. Not currently
warranted.

## Container images (GHCR)

Two images ship from a **single, unified root `Dockerfile`** selected with
`--target` (DRY: TeX Live, graphics tools, and the build toolchain are declared
once in shared `texbase` + `toolchain` stages; only the per-binary build command
and entrypoint differ):

- **`--target cli` → `ghcr.io/dginev/latexml-oxide`** — the plain `latexml_oxide`
  entrypoint plus a reproducible TeX Live + graphics environment, so a user
  needs no local TeX Live:
  ```
  docker run --rm -v "$PWD:/work" ghcr.io/dginev/latexml-oxide:X.Y.Z paper.tex
  ```
- **`--target worker` → `ghcr.io/dginev/latexml-oxide/cortex-worker`** — the
  turnkey CorTeX fleet harness (ZMQ, `cortex-worker-entrypoint.sh`; see
  `docker/README.md`).

Unlike the tarball/.deb (a prebuilt static binary), each image **builds its own**
binary from source and regenerates the kernel dumps against its own TeX Live. The
CLI **embeds** them (a runtime-stage self-test converts a document with no repo
tree present, proving self-containment before push); the worker reads them from
disk via `LATEXML_DUMP_DIR`. Both link the system libxml2/libxslt/kpathsea
dynamically — static linkage is only for the portable tarball/.deb; inside a
fixed image the dynamic libs are always present.

`.github/workflows/docker.yml` builds + pushes both on `release: published` (so
the containers track a published tag, never a draft; since the `Release`
workflow now auto-publishes, this fires automatically on each release; also
`workflow_dispatch`-able for a given tag). The CLI is multi-arch (amd64 + arm64) on **native runners** — no
QEMU emulation of the fat-LTO compile — merged into one manifest list tagged
`:X.Y.Z` + `:latest`; the worker is amd64-only (x86_64 fleet). The first push of
each package creates it private — make it public once in the repo's package
settings.

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
    git push origin main
    git push origin X.Y.Z
    ```

5. **Watch the workflow.** `Release` on the Actions tab. It runs seven
   jobs: `dumps` (TL-window generation) → `notices` (assembles THIRD-PARTY-NOTICES
   once, handed to every leg) → `build-macos` (Apple Silicon, `macos-15`) ‖
   `build-macos-intel` (Intel, `macos-15-intel`) ‖ `build-linux-arm64`
   (`ubuntu-22.04-arm`) ‖ `build-windows` (`windows-latest`) → `release`
   (Linux x86_64 tarball + `.deb`, then collects every other leg's artifacts and
   attaches all **eight** assets). The Intel-macOS leg's fat-LTO `maxperf` build on the
   slower `macos-15-intel` runner is the long pole (up to ~120 min budget).

6. **Nothing — it auto-publishes.** The workflow publishes a **public**
   Release directly (`release.yml` `draft: false`), so a tag push is the last
   manual step. This is safe because every asset is gated in-CI before publish:
   static-linkage checks (`ldd`/`otool`), the size budget, a real conversion
   smoke on the Linux legs, and a code-signature + `--version` launch smoke on
   both macOS legs — plus CI.yml's macOS test job covers arm64 conversion paths
   on the same arch. If a published asset is later found broken, use *Failure
   recovery* below. (To re-add a human review gate, set `draft: true` and
   publish manually.)

   > Coverage note: the **Intel-macOS** asset (`…-x86_64-apple-darwin.tar.gz`)
   > is built on `macos-15-intel`, which no CI test job exercises — its only
   > automated gate is the launch/signature smoke. If you have an Intel Mac,
   > a periodic spot-check of a real conversion there is worthwhile.

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
