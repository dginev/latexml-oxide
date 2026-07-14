# Windows Compatibility Plan

**Status: living worklist** (started 2026-07-12, `windows-compatibility` branch).
Mission: bring the single turnkey `latexml_oxide` executable to native Windows
(`x86_64-pc-windows-msvc`), and make `cargo test --release` (and the `ci`
profile) pass on Windows, with a `windows-latest` CI job and a zipped `.exe`
release artifact as first-class deliverables.

This plan operationalizes portability rung 5 ("Windows — deferred") of
[`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §portability. Strategic decisions
locked in with the maintainer (2026-07-12):

| Decision | Choice |
|---|---|
| Toolchain / target | **MSVC** (`x86_64-pc-windows-msvc`) with **vcpkg static** libxml2/libxslt. No MinGW/GNU bring-up phase. |
| TeX distribution | **TeX Live for Windows AND MiKTeX** must both work at runtime (subprocess-`kpsewhich` backend for both). |
| CI | `windows-latest` job running the test suite is a first-class deliverable. |
| Release | Zipped self-contained `latexml_oxide.exe` artifact alongside the Linux/macOS assets. |

## Approach principle: prefer portable ecosystem helpers over hand-rolled platform code

Where a well-maintained crate (or a `std` API) already solves a
cross-platform problem, use it instead of writing `cfg(windows)` branches by
hand — less code to audit per-platform, and the edge cases (UNC prefixes,
`PATHEXT`, per-user dirs) are someone else's regression suite. Concretely:

| Problem | Use | Status |
|---|---|---|
| Temp files/dirs | `std::env::temp_dir()` for fixed-name scratch; **`tempfile`** (already a dep of `latexml_post` + `latexml_oxide`) for unique/auto-cleaned files — prefer it over hand-built `SystemTime`-suffixed siblings when touching that code | in use; extend opportunistically |
| Locating delegate executables on `PATH` | **`which`** crate — handles `PATHEXT` (`.exe`/`.bat`/`.cmd`), canonical result paths; replaces the hand-rolled `program_on_path` probe in `graphics.rs` | adopt in Phase 2.4 |
| Home directory | **`home`** crate (the cargo team's own; correct `USERPROFILE`/`HOME` semantics) — replaces the env-var fallback in `pathname.rs::HOME_PATH` | adopt in Phase 2.1 |
| `fs::canonicalize` returning `\\?\C:\…` verbatim paths (break string-level code and subprocess args) | **`dunce`** — canonicalizes to legacy drive-letter form when safe | adopt in Phase 2.1 |
| `Path` → `/`-separated string at the pathname-layer boundary | **`path-slash`** (or a 5-line local helper if the dep isn't warranted) | decide in Phase 2.1 |
| `PATH` splitting/joining | `std::env::split_paths` / `join_paths` — never split on `:`/`;` manually | in use (Phase 0) |

Rules: new deps go through the existing gates (`deny.toml` licenses/advisories,
`cargo machete`); Phase-0 groundwork deliberately stayed `std`-only because it
landed without a compiling Windows toolchain to verify new deps against —
swap-ins happen in the phase where they're compile-tested. This mirrors the
project's existing choices (`tempfile`, `glob`) rather than a new policy.

The kpathsea story is already Windows-shaped: the `kpathsea` crate's
subprocess-`kpsewhich` backend (see `latexml_core/Cargo.toml` and
`latexml_core/src/util/pathname.rs`) removes the libkpathsea link requirement —
both TeX Live's `kpsewhich.exe` and MiKTeX's `kpsewhich.exe` serve it. The
self-contained-binary design (embedded dumps/XSLT/CSS/schema,
[`OXIDIZED_DESIGN.md`](OXIDIZED_DESIGN.md)) also transfers cleanly: the
embedded-dump disk cache already uses `std::env::temp_dir()`
(`latexml_engine/src/embedded_dumps.rs`).

> **TL2026 release-dump caveat (2026-07-12):** this branch's full TL2026
> install proved the *plain* dump is release-gate-clean but surfaced that
> `--init=latex.ltx` on TL2026 hits **137 raw-load expl3-catcode-gap errors**
> — so 2026 cannot join the release dump window until that (deep, pre-existing,
> Linux-equivalent) gap is closed, independently of the unpublished
> texlive-docker:2026 container. Details + measurements in `SYNC_STATUS.md`
> ("TL2026 `latex.ltx` dump init is NOT release-gate-clean"). Windows test-suite
> green (1531/0) does NOT imply 2026 release-readiness — different bars.

---

## Phase 0 — groundwork that needs no Windows toolchain (LANDED on this branch)

These are cross-platform-neutral fixes validated by the existing Linux/macOS CI:

- [x] **`.gitattributes` LF enforcement.** Golden-file comparisons
  (`latexml_oxide/src/util/test.rs::process_xmlfile`) split on `'\n'` over raw
  bytes; a default Windows git (`core.autocrlf=true`) checks out CRLF and fails
  every `.tex`/`.xml` regression test. `* text=auto eol=lf` pins LF on checkout
  for all text files on every platform.
- [x] **Gate the un-gated `libc::dlsym` in `latexml_post/src/xslt.rs`**
  (`set_xslt_max_depth` + its test). `libc` is a `cfg(unix)`-only dependency of
  `latexml_post`, so this was the one guaranteed compile error on Windows that
  was NOT tracked in `RELEASE_CRITERIA.md`. Non-unix interim behavior: skip the
  write; libxslt's built-in recursion cap of 3000 still bounds recursion
  (Perl-parity value 1000 restored in Phase 2).
- [x] **Platform-aware delegate program names in `latexml_post/src/graphics.rs`.**
  On Windows, `convert.exe` is the system FAT→NTFS utility in `System32` (which
  shadows ImageMagick's legacy name), and Ghostscript ships its console binary
  as `gswin64c.exe` (MiKTeX bundles `mgs.exe`). Bare `Command::new("convert")` /
  `Command::new("gs")` would either run the wrong program or fail. New helpers
  `im_convert_program()` / `gs_program()` resolve `magick` and
  `gswin64c`/`gswin32c`/`mgs` on Windows.
- [x] **`HOME` → `USERPROFILE` fallback** in `latexml_core/src/util/pathname.rs`
  (`HOME_PATH`), so tilde expansion works on Windows.
- [x] **`/tmp` hardcodes → `std::env::temp_dir()`** in
  `latexml_oxide/src/util/test.rs` (SIGSEGV-handler dump file,
  `LATEXML_SAVE_ACTUAL` outputs).
- [x] **Opt-in CI workflow** `.github/workflows/windows-ci-manual-trigger.yml`
  (`workflow_dispatch` + pushes to `windows-compatibility`), staged so each step
  reports its own failure. It is *expected* to fail in early phases — it exists
  to give every subsequent phase a live Windows probe without turning main CI red.

## Phase 1 — make the workspace COMPILE on `x86_64-pc-windows-msvc`

**Progress 2026-07-12 (local bring-up box):** rustup nightly-msvc, VS 2022
Build Tools (VCTools), LLVM (libclang for bindgen — a toolchain requirement
this plan originally missed), and vcpkg are installed. Findings that
re-shape the phase:

- **libmarpa cc-port: DONE and validated.** The dist tarball ships all
  generated sources, and of `config.h.in`'s macro set the code only reads
  `MARPA_LIB_{MAJOR,MINOR,MICRO}_VERSION` — so `build.rs` now synthesizes a
  3-line `config.h` from `LIB_VERSION` and compiles the `Makefile.am` source
  list (6 files) via `cc::Build`. Landed on `dginev/marpa` branch
  `windows-compatibility` (commit `64c045c`); the **entire marpa test suite
  passes on Windows MSVC**. The workspace consumes it via the `[patch]`
  mechanism until the marpa PR merges. The presumed long pole fell first.
- **`libxml 0.3.15` already has first-class vcpkg support** on windows-msvc
  (`vcpkg::find_package("libxml2")` + bindgen on the vcpkg headers). No
  upstream PR needed — just `VCPKG_ROOT` + `VCPKGRS_TRIPLET=x64-windows-static-md`.
- **`libxslt 0.1.4` has NO Windows path** (pkg-config, else bare
  `cargo:rustc-link-lib=dylib=xslt`+`exslt`). The bare fallback may resolve
  against vcpkg's lib dir if a link-search is already in scope from libxml2;
  otherwise upstream a small vcpkg arm.
- **Dependency policy (maintainer, 2026-07-12):** `rust-libxml`,
  `rust-libxslt`, `rust-kpathsea`, and `rust-marpa` are all maintained by
  this project's author. Windows-compatibility issues in them are fixed
  **cleanly upstream in the crate** (branch + `[patch]` while iterating,
  then merge/publish and re-point), never with workspace-side workarounds,
  vendored forks, or env-var hacks. The libmarpa cc-port above is the model.
- **`kpathsea 0.3`/`kpathsea_sys 0.2.1` are already Windows-aware**: graceful
  no-link fallback to the subprocess backend, `which`-based PATHEXT-correct
  `kpsewhich` probe, even TL-Windows `kpathsealibw64.dll` detection. Building
  before a TeX distro is on PATH requires `KPATHSEA_SKIP_TOOLCHAIN_CHECK=1`.

**Progress 2026-07-12, later the same session — `cargo check --workspace
--all-targets` PASSES on Windows.** Two more upstream one-liners were needed
beyond the marpa port:

- **rust-libxml** (`windows-compatibility` branch, `8e40ba00`): the vcpkg arm
  must emit `cargo:rustc-link-lib=bcrypt` (+`ws2_32`) — vcpkg links port
  libraries but not Windows SDK system libs, and libxml2 ≥ 2.12 calls
  `BCryptGenRandom` from `xmlInitRandom` (LNK2019 first surfaces in
  latexml_codegen's proc-macro dylib link, since `cargo check` of rlibs
  never links).
- **rust-libxslt** (`windows-compatibility` branch, `9fa9a6d8`): new
  vcpkg-resolution arm in build.rs mirroring rust-libxml's.

Machine-local wiring (documented so the CI job can replicate it): vcpkg
triplet `x64-windows-static-md` with `VCPKG_ROOT`/`VCPKGRS_TRIPLET`, plus
`LIBCLANG_PATH` for bindgen — set via a parent-dir `.cargo/config.toml`
`[env]` block outside the repo. TL-Windows validation: `kpsewhich
-var-value=SELFAUTOPARENT` returns forward-slashed `C:/texlive/2026`, so
year detection and the `/`-normalization strategy hold.

The three native C dependencies, in increasing order of difficulty:

1. **libxml2 via vcpkg** (`vcpkg install libxml2:x64-windows-static-md`).
   The `libxml 0.3.15` crate build script resolves via pkg-config or env vars.
   Plan: use the `vcpkg` Rust crate convention (`VCPKG_ROOT` +
   `VCPKGRS_TRIPLET=x64-windows-static-md`) or explicit `LIBXML2` env pointing
   at the vcpkg-installed `.lib`. If the crate's build script cannot be
   convinced, upstream a small PR (the crate is actively maintained by this
   project's author — low friction). Verify: `cargo check -p latexml_core`.
   - `x64-windows-static-md` (static libs, dynamic CRT) is the recommended
     triplet: static CRT (`x64-windows-static` + `-C target-feature=+crt-static`)
     is all-or-nothing across every C dep and complicates debugging; dynamic
     libs would break the turnkey single-exe goal.
2. **libxslt via vcpkg** (same triplet; `libxslt 0.1.4` crate). Same env/PR
   strategy. Watch for the pregenerated-bindings symbol-decoration issue class
   documented in `docs/archive/PORTABILITY_MACOS_PROBE_2026-06-07.md` — on
   x64 COFF, C symbols are undecorated, so the `#[link_name = "\u{1}…"]`
   pinned names should link as-is (unlike Mach-O), but this must be verified.
3. **libmarpa (`marpa` git dep → `libmarpa-sys`)** — the hardest blocker. The
   vendored `libmarpa-8.6.2.tar.gz` builds via `./configure && make` under
   `/bin/sh`, which does not exist on Windows. Plan, in preference order:
   a. **Port `libmarpa-sys/build.rs` to the `cc` crate**: libmarpa is plain
      C99 with a stable file list; compile the tarball's `.c` files directly
      with `cc::Build`, generating `config.h`/`marpa.h` from the tarball's
      templates (they are nearly static — the configure step mostly stamps
      version numbers). This fixes Windows AND simplifies Linux/macOS builds.
      Land in the `dginev/marpa` repo (master), then `cargo update -p marpa`.
   b. Fallback: prebuild `marpa.lib` in CI via an MSYS2 step and feed it to
      the build script via env var — workable for CI but hostile to local
      dev; only if (a) stalls.
4. **Workspace compile sweep.** With the C deps linking, drive
   `cargo check --workspace --all-targets` to zero errors. Known code-level
   items beyond Phase 0 (all have `#[cfg(not(unix))]` fallbacks already, so
   they are *expected* to compile — verify, don't assume):
   `getrusage` telemetry fallbacks (`latexml_oxide.rs:1058`,
   `cortex_worker.rs:556`), `graphics_cache.rs` prune-lock sentinel,
   `graphics.rs` timeout-kill PID fallback, `lsp_server/generic.rs`.
   The `cortex` feature (zmq) is out of scope for Windows — document as
   unsupported rather than porting `zeromq-src` builds.

**Exit criterion:** `cargo build --release --bin latexml_oxide` produces a
`latexml_oxide.exe` on a Windows machine with only `VCPKG_ROOT` set.

## Phase 2 — make the binary WORK on Windows (runtime correctness)

1. **Pathname layer** (`latexml_core/src/util/pathname.rs`) — the highest-risk
   runtime area. The `LaTeXML::Util::Pathname` port canonicalizes with
   string-level `/` operations (`canonical()`, `pathname_concat`, directory
   splits). On Windows, `Path`/`PathBuf` produce `\`-separated strings, so
   canonicalization silently no-ops and mixed-separator paths leak into
   kpsewhich queries, XML `imagesrc` attributes, and dest-dir writes. Decide
   and implement ONE policy:
   - **Recommended: normalize to `/` at the boundary.** Windows APIs accept
     `/` in almost all contexts; kpsewhich on Windows *returns* `/`-separated
     paths. Convert `\` → `/` when a path enters the string-pathname layer
     (single choke point), keep the existing `/`-based logic intact. Add
     drive-letter awareness (`C:/…` absolute-path detection: current
     `is_absolute`-style checks that test for leading `/` must also accept
     `[A-Za-z]:`) and UNC prefixes as explicit cases with unit tests.
   - Audit `PATH`-splitting (`:` vs `;`) — use `std::env::split_paths`
     everywhere.
2. **kpsewhich subprocess backend on both TeX distros.**
   - TeX Live for Windows: verify `kpsewhich -var-value=SELFAUTOPARENT` year
     detection (`dump_paths.rs`) handles `C:/texlive/2026` (drive letter +
     either separator).
   - MiKTeX: `kpsewhich --version` reports MiKTeX, not "TeX Live YYYY" —
     `detect_ambient_texlive_year` needs a MiKTeX arm (MiKTeX is rolling; map
     to the closest TL year or fall back to newest embedded dump,
     documented). MiKTeX's on-the-fly package install can block on a GUI
     prompt: document `--disable-installation`/`AutoInstall=no` guidance, and
     make sure a hung `kpsewhich` cannot hang a conversion (timeout).
   - Windows `Command` spawn overhead is ~10× Unix fork; the per-lookup
     subprocess cost may need the existing kpsewhich cache widened (measure
     first — see `perf-check` skill rules).
3. **Restore Perl-parity `xsltMaxDepth = 1000` on Windows.** With vcpkg
   *static* libxslt the symbol lives in our own image, so `GetProcAddress` is
   not applicable; declare `extern "C" { static mut xsltMaxDepth: c_int; }`
   directly (undecorated on x64 COFF) under `#[cfg(windows)]` and re-enable
   the `dlsym_sets_perl_parity_cap` test equivalent.
4. **Graphics delegate chain end-to-end.** With Phase 0 names in place,
   validate each delegate on Windows installs: `magick` (ImageMagick 7),
   `gswin64c`/`mgs`, `mutool`, `pdftocairo` (ships with TeX Live's tlpkg on
   Windows and MiKTeX), `ps2pdf` (a `.bat` on TL-Windows — Rust `Command`
   spawns `.bat` via `cmd.exe`; verify the post-CVE-2024-24576 strict arg
   escaping doesn't reject our args, else replace the `ps2pdf` call with a
   direct `gs -sDEVICE=pdfwrite` invocation).
   Also: the timeout kill path (`run_with_timeout`) has no process-*group*
   semantics on Windows; use Job Objects (or accept PID-only kill and document
   the orphaned-grandchild risk — decide when measuring, mirror of the
   `setsid`/`killpg` design).
5. **`latex_images.rs` pipeline** (`latex` + `dvips`/`dvipng` delegates):
   verify on both distros; MiKTeX names match.

   **Windows delegate availability matrix (2026-07-12 inventory):**

   | Delegate | Windows source | Status on bring-up box |
   |---|---|---|
   | `latex`, `dvips`, `ps2pdf`, `kpsewhich`, `pdflatex` | TeX Live bin/windows | ✅ installed |
   | `dvipng` | `tlmgr install dvipng` (not in scheme-medium) | ✅ installed |
   | Ghostscript | TL-Windows bundles it behind `rungs.exe` (in `gs_program()`'s probe list) — no separate install needed on a TL box; dedicated: winget `ArtifexSoftware.GhostScript` (needs UAC), MiKTeX: `mgs.exe` | ✅ via TL `rungs` |
   | ImageMagick 7 | **portable 7z from GitHub releases** (`ImageMagick/ImageMagick` release assets, extract, put `magick.exe` on PATH) — the winget installer needs a UAC accept, portable does not | ✅ portable at `C:\claude\tools\imagemagick` |
   | MuPDF (`mutool`) | zip from GitHub `ArtifexSoftware/mupdf-downloads` release assets | ✅ portable at `C:\claude\tools\mupdf` (1.28.0) |
   | poppler (`pdftocairo`) | NOT in TL-Windows; scoop/choco `poppler` or vcpkg `poppler[cairo]` — optional middle of the PDF chain | ⬜ not installed |
   | `xmllint` | NOT in TL-Windows; vcpkg's libxml2 port skips tools | ⬜ absent — Phase 3.3 removes the need |
   | MiKTeX (second TeX distro) | miktex.org installer | ⬜ deferred until the Phase 2 distro matrix (PATH ordering vs TL needs care) |
6. **Smoke matrix:** `latexml_oxide --format=html5 --dest=paper.html paper.tex`
   on (TeX Live, MiKTeX) × (plain doc, math-heavy doc, EPS/PDF graphics doc),
   diffed against Linux output. Divergences triaged per `canvas-triage` rules
   (fail toward flagging).

## Phase 3 — make `cargo test --release` (and `--profile ci`) PASS on Windows

**Build-cost finding (2026-07-12, first Windows suite attempt):** a plain
`cargo test --release --tests --workspace` is dominated by `[profile.release]
lto = "thin"` — every one of the ~60 test executables (48 integration-test
files in `latexml_oxide/tests` alone, plus per-crate unit-test binaries)
re-runs thin-LTO over the whole statically-linked workspace (~55 MB each,
~2 min per binary → ~2 h wall on a 16C/32T Threadripper; interrupted before
any test executed). This is profile physics, not a Windows defect — Linux
pays the same LTO cost. **Supported way to run the release suite:**

```
CARGO_PROFILE_RELEASE_LTO=false cargo test --release --tests --workspace
```

LTO is a distribution-artifact optimization (`maxperf`); test correctness is
unaffected. The future windows CI job must use the same override (or the `ci`
profile, which CI uses everywhere anyway). Committed build-speed help:
`.cargo/config.toml` now mirrors the Linux `-Zthreads=8` parallel-frontend
flags for `x86_64-pc-windows-msvc` (no mold equivalent; a machine-local
config can compose `-Clinker=lld-link` on top, since cargo merges rustflags
arrays across config files). Also recommended on Windows dev boxes: a
Windows Defender real-time-scanning exclusion for the checkout + cargo dirs
(user action, not automated — it's a security setting).

1. **Dump generation without bash.** Port `tools/make_formats.sh` to
   PowerShell (`tools/make_formats.ps1`), same contract: build, detect TL year
   (kpsewhich SELFAUTOPARENT → `pdflatex --version` fallback, plus the MiKTeX
   arm from Phase 2), run the two `--init` passes, verify
   `resources/dumps/{plain,latex}.YYYY.dump.txt`. Keep the `.sh` authoritative;
   the `.ps1` mirrors it (note the duplication in both headers).
2. **Test-discovery proc macro on `\` paths.** `latexml_codegen/src/testable.rs`
   globs `latexml_oxide/{dir}/*.tex` at compile time; verify the embedded
   relative paths (native `\` separators on Windows) flow correctly into
   `latexml_test_single` and into any golden-path comparisons. Fix at the
   macro (emit `/`-normalized strings) if not.
3. **`90_latexmlpost.rs` bash dependency.** The `bash -c "diff <(xmllint …)"`
   comparisons need a rewrite to run anywhere without a Unix userland:
   preferred — do the comparison in Rust (parse both files with the
   already-linked libxml2, pretty-print, line-diff in-process), which also
   removes the `xmllint`/`diff`/`grep`/`wc` CI dependency on Linux. Interim:
   `#[cfg_attr(windows, ignore = "requires unix userland — see WINDOWS_COMPATIBILITY_PLAN.md Phase 3")]`.
4. **Unix-flavored unit tests.** Inventory and fix or cfg-gate:
   `graphics.rs:2764` (fake-`convert` shell script + `0o755` + `PATH` `:`
   split — provide a `.bat`/`.cmd` twin under `cfg(windows)`),
   `latexml_post/src/collector.rs` / `processor.rs` `/tmp/...` golden-string
   assertions (use `Path`-built expectations),
   `latexml_core/tests/07_unit_relaxng_scan.rs` `/home/deyan` candidates
   (drop or generalize).
5. **Full-suite drive to green:** `cargo test --release --tests --workspace`
   on a Windows machine with TeX Live; then repeat with MiKTeX (expected:
   package-availability skips, not failures). Triaging order: compile errors →
   path/separator failures → delegate failures → genuine parity divergences
   (each of the last class gets a `SYNC_STATUS.md` entry, per project rules).
6. **Test-profile note:** `split-debuginfo = "unpacked"` in `[profile.test]`
   is `.dwo`-flavored; MSVC uses PDBs and cargo maps `unpacked` appropriately —
   verify no warning/error, else gate the setting per-platform via
   `[profile.test]` overrides in a `--config` layer (cargo profiles are not
   target-conditional; if it errors, the simplest fix is accepting `packed`
   semantics everywhere or documenting a Windows-local override).

**First full-suite triage (2026-07-12, ~1350 tests, 9 failures):** all
failures fell into predicted buckets. FIXED same-day: `overlay` mtime test
(read-only handle + `set_modified` — Windows needs write access),
`pathname::concat` (was `PathBuf::push` → `\`; now joins with `/` like
Perl's `pathname_concat`, plus `canonical()` normalizes `\`→`/` on Windows
at the single choke point), `pack` zip entry names (zip spec mandates `/`).
OPEN, reclassified as **ambient-TL-2026 drift suspects, not Windows bugs**
(this box runs TL 2026; Linux CI runs Ubuntu's older TL — verify on
Linux+TL2026 before treating as platform divergence): `greek_test`
(babel's new `locale/invalid/` deprecation shim for `polutonikogreek`
fires; `\text`/`\acc*` undefined downstream) and 86_tikz
`ac_drive_components_test` (SVG coordinate drift 12.4 → 12.68, pgf
version-scented). FIXED same-day: 4 × `90_latexmlpost` — the Phase 3.3 rewrite landed:
comparison now parses both sides in-process with the already-linked libxml2
(`no_blanks` parse + `format: true` serialization = exactly `xmllint
--format`) and line-diffs via the `similar` crate (LCS, same counts as
GNU diff `<`/`>` lines). No bash/xmllint/diff/grep/wc on ANY platform now,
and a missing/malformed file panics instead of vacuously passing.

**Suite status after day one: 1524 passed / 2 failed** — two ambient-TL-2026
drift suspects (`greek_test`, tikz `ac_drive_components_test`). `greek_test`
was root-caused and FIXED TL-independently via native `\Declare*caseMapping`
handlers (engine, cross-platform — verified on macOS CI too). The tikz one was
**circuitikz-version** drift, NOT a code defect: the drawn-plate coordinate
(12.4 → 12.68) tracks circuitikz's version, which is unpinnable and differs by
the platform's TeX (Linux/macOS apt/brew = older = 12.4 = golden; Windows CI
net-install + fresh `install-tl` = newest = 12.68). It's **compared on
Linux/macOS and skipped on Windows** (a `#[cfg(windows)]` `WINDOWS_GOLDEN_SKIP`
guard in `latexml_test_single`) — a Linux↔Windows portability difference, not
a code divergence. Full reasoning in the "tikz `ac_drive_components`" entry in
`SYNC_STATUS.md`.

## Phase 4 — CI: `windows-latest` job as a required leg

> **Resolved (2026-07-13): the `Repository access blocked` failure was a dead
> action, not billing.** the Windows CI job's early runs aborted at "Getting action
> download info" because `teatimeguest/setup-texlive-action@v3` — the TeX Live
> setup action — had its entire GitHub account go **404** (moved to the
> `TeX-Live` org). GitHub reports an un-fetchable action as `Repository access
> blocked`, which *looked* like a Windows-runner billing block but was not (the
> account runs metered jobs fine — the macOS 10× leg proves it; the Actions
> budget had headroom). Fixed by switching to `TeX-Live/setup-texlive-action@v4`
> (identical inputs). The workflow is kept **`workflow_dispatch`-only by
> choice** — deliberate manual runs to control this private repo's 2× Windows
> minutes — not because of any access block. Restore the on-push trigger any
> time before promoting it into `CI.yml` (below).

1. Promote the bring-up workflow into `CI.yml` as a `windows` job mirroring the
   `macos` job structure: rustup nightly (MSVC host default), vcpkg install
   (with `actions/cache` on the vcpkg tree keyed by vcpkg baseline + triplet),
   TeX Live install (the `teatimeguest/setup-texlive-action` gives a cached,
   scheme-configurable TL — the full apt package list maps to TL collections
   `collection-latexextra`, `collection-science`, `collection-publishers`,
   `collection-bibtexextra`, language packs de/fr/el/ru), `make_formats.ps1`
   with the same dump-cache key pattern (`runner.os` already separates caches),
   then `cargo test --profile ci --tests --workspace -- --test-threads=2`.
2. Runner budget: windows-latest = 4 vCPU / 16 GB, same as ubuntu — the `ci`
   profile's RAM tuning carries over; expect slower cold builds (MSVC link
   times); set `timeout-minutes` generously (120, like macOS).
3. MiKTeX leg: a second, **scheduled/dispatch-only** job (not per-PR) that runs
   the smoke matrix against MiKTeX — full suite × 2 TeX distros per PR is not
   worth the runner minutes.
4. Only after the job is stably green: mark it required, update the README
   badge/platform table.

### Fast TeX-ecosystem CI install (design record — DEFERRED, 2026-07-13)

Goal: make the TeX install in the Windows CI jobs **extremely quick and
deterministic**. The cold `TeX-Live/setup-texlive-action` download of
`scheme-medium` + 6 collections + language packs (~1–2 GB) is the slowest
step. Not changing it today; capturing the design so we implement the right
thing rather than react.

**Two constraints that rule the choice:**
- **Determinism / no golden churn (load-bearing).** The 1531/0 suite is
  validated against a specific package-version universe (TL2026). *Any* distro
  or version whose package files differ can shift package-version-sensitive
  fixtures (tikz metrics, `.tfm` fonts, babel) — exactly the circuitikz
  1.8.0 drift we already hit. So the fast install must be **pinned to the
  validated TL year**, not "whatever's latest." This is why a blind swap to
  MiKTeX (a different package universe) is risky: it trades install speed for
  a golden-triage tax, and LaTeXML's own Windows workflow warns MiKTeX-in-CI
  cost them "too many hours."
- **Per-job TeX footprint differs**, so one size need not fit all:
  - *release smoke* (`hello.tex`): trivial — a minimal scheme, or skip TeX
    and build with `KPATHSEA_SKIP_TOOLCHAIN_CHECK=1`.
  - *dump generation* (`make_formats`): only the kernel layer (plain.tex,
    latex.ltx, expl3, base fonts) — a small, stable set.
  - *test suite*: the union of packages the fixtures touch — broad, but
    **finite and knowable**.

**Options, ranked:**
1. **Pinned, cached, minimal texmf snapshot (recommended target).** Instrument
   one full test run to capture the exact `kpsewhich`-resolved file set, build
   a minimal texmf tree containing only those (pinned to the validated TL
   year), store it as a content-hash-keyed artifact/cache, and restore it in
   CI (download + extract = seconds). Fastest *and* deterministic *and*
   golden-stable — decouples CI speed from any distro's install machinery.
   Cost: a one-time tool to capture the file set + a refresh step when the
   suite's package footprint changes. Generalizes to the Linux/macOS legs too
   (they currently apt/brew-install a broad TL, also slow).
2. **`setup-texlive-action` cache + trimmed package set (low-effort interim).**
   The action already caches by default (cold run slow, warm runs fast), so
   most of the pain is one-time; trimming the collections to only what the
   suite needs shrinks the cold download. No golden risk (still TL). Smallest
   change if we want relief before (1) lands. Caveat: GitHub cache is 10 GB/
   repo with 7-day idle eviction, so a large TL tree can thrash the cache.
3. **MiKTeX via choco + cached `C:\miktex-repo` + on-the-fly install (Perl's
   approach).** Fetches only used packages; the Perl-specific `execsilent`/
   `LATEXML_KPSEWHICH*` glue is NOT needed for us (our subprocess-`kpsewhich`
   backend reads stdout and ignores MiKTeX's stderr nags — verified on a local
   `hello.tex`). But it's a different package universe → golden churn (see
   constraint above), plus the maintainers' own "too many hours" caution. Best
   reserved for a **separate, non-golden MiKTeX smoke leg** (Phase 4.3), not
   the golden-comparing suite.
4. **Self-hosted Windows runner with TeX pre-installed / a pre-baked runner
   image.** Zero install time; needs infra and interacts with the private-repo
   minute economics. Overkill unless CI volume grows.

**Decision:** target option (1) for the golden-comparing suite (fast +
deterministic), keep option (3) for the deliberately-non-golden MiKTeX smoke
leg. Consider (2) as a stopgap. Deferred — no CI change today.

**Self-containedness landmine (found 2026-07-12, MiKTeX smoke):** a Windows
build made with TeX Live on PATH silently LINKS TL's `kpathsealibw64.dll`
(kpathsea_sys's `try_windows_dll` probe), producing a binary that won't even
LAUNCH unless TL's bin dir is on PATH — with only MiKTeX installed it dies at
load time with "error while loading shared libraries". The Windows
**distribution** build must therefore set `KPATHSEA_NO_LINK=1` (subprocess
`kpsewhich` backend, works against both TL and MiKTeX) — the exact Windows
analogue of the Linux release-dumps' "kpathsea-UNLINKED dumper binary"
pattern. Local dev builds with TL on PATH may keep the in-process link (it's
faster); only the shipped artifact and the MiKTeX test leg need the unlinked
variant. `dumpbin /DEPENDENTS` in Phase 5.2 is the regression gate.

**MiKTeX runtime findings (Phase 2.2, same probe):** `kpsewhich
-var-value=SELFAUTOPARENT` returns EMPTY on MiKTeX (plus an update-nag line
on stderr), and no banner anywhere mentions a TL year — so
`detect_ambient_texlive_year` gained a MiKTeX arm mapping the rolling
`MiKTeX YY.MM` version stamp to `20YY` (25.12 → 2025; nearest-dump fallback
covers the rest). MiKTeX ships `mgs.exe` (already in `gs_program()`'s probe
list) and resolves files with forward slashes, like TL-Windows.

## Phase 5 — release artifact

**Prototype validated locally (2026-07-12).** The full Windows distribution
recipe, end to end:

```
KPATHSEA_NO_LINK=1 cargo build --no-default-features \
  --features runtime-bindings --profile maxperf --bin latexml_oxide
```

Results on the bring-up box: 46.9 MB exe (vs 58.9 MB release-profile);
`dumpbin /DEPENDENTS` shows OS DLLs + the VC runtime (`VCRUNTIME140*`) +
UCRT (`api-ms-win-crt-*`) — the `-md` dynamic-CRT triplet choice. **Caveat
(corrected 2026-07-14, see Phase 5.1): only the UCRT ships with Windows
10+; `VCRUNTIME140.dll`/`VCRUNTIME140_1.dll` are the VC++ *redistributable*,
NOT part of Windows** — so this `-md` build is not truly self-contained on a
clean box. Converts on a MiKTeX-only PATH and
produces full HTML5 + CSS on TL; embedded dumps verified by renaming
`resources/dumps` away (no degraded-mode warning). Packaged as
`latexml-oxide-<version>-x86_64-pc-windows-msvc.zip` + `.sha256` sidecar
(Compress-Archive + Get-FileHash), matching the existing asset naming.
Remaining for the real release leg: wire this into `release.yml`
(vcpkg + setup-texlive as in windows-ci-manual-trigger.yml, dumps from
release-dumps.yml embed unchanged) and the README platform table.

1. Extend `release.yml` with a `x86_64-pc-windows-msvc` leg: `maxperf` profile,
   `--no-default-features --features runtime-bindings` (same recipe as
   `tools/make_release.sh`), vcpkg static libs, embedded 5-year dump window
   (the `release-dumps.yml` dumps are platform-neutral text — generated on
   Linux containers, embedded into the Windows build unchanged; verify the
   `latexml_engine/build.rs` scan is separator-clean).
2. Self-containedness verification, Windows edition: the `ldd`/`otool` checks
   become `dumpbin /DEPENDENTS` (expect only OS DLLs + CRT); confirm no
   `resources/` reads via Process Monitor once, manually.
3. Package as `latexml-oxide-<version>-x86_64-pc-windows-msvc.zip` +
   `.sha256` sidecar, matching existing naming. Document runtime prerequisites
   in README (TeX Live or MiKTeX on PATH; ImageMagick/Ghostscript/MuPDF/poppler
   optional for graphics). Code-signing: out of scope for now (record in
   `RELEASE_CRITERIA.md` if users hit SmartScreen friction).
4. Update `RELEASE_CRITERIA.md` portability ladder (rung 5 → in progress → done)
   and `RELEASING.md` platform matrix.

## Phase 5.1 — fully-static `.exe` (static CRT) — LANDED 2026-07-14

**Problem.** The RC 0.7.4-rc1 `.exe` (the `-md` triplet) dynamically links
`VCRUNTIME140.dll` + `VCRUNTIME140_1.dll` (the MSVC C++ runtime). Those are **not**
part of a clean Windows install — they ship with the VC++ Redistributable — so
on a machine without it, the `.exe` fails to *launch* (`ERROR_BAD_EXE_FORMAT` /
missing-DLL) before `main`. It only ran on dev boxes because the VS toolchain put
`VCRUNTIME140.dll` there. (First-principles: a distributable `.exe`'s entire
transitive import closure must resolve to DLLs guaranteed present on the target;
the UCRT `api-ms-win-crt-*` set qualifies on Win10+, the VC runtime does not.)

**Fix (ripgrep's posture).** `-C target-feature=+crt-static` statically links the
VC runtime **and** the UCRT → the import closure collapses to core OS DLLs only
(`kernel32`, `ntdll`, `api-ms-win-core-*`), so the `.exe` runs on any Windows,
no redistributable. Verified locally: with `+crt-static`, `dumpbin /DEPENDENTS`
drops every `VCRUNTIME*` and `api-ms-win-crt-*` entry.

**The C-dependency wrinkle (why it's not a one-line `.cargo/config.toml` change
like ripgrep).** ripgrep is ~pure Rust, so `crt-static` is free. We link C
(`libxml2`/`libxslt` via vcpkg; `libmarpa` via `cc`), and the CRT model must be
**uniform** across the whole image — `/MT` (static) and `/MD` (dynamic) cannot mix
(two heaps → corruption). So `+crt-static` (Rust `/MT`) requires the vcpkg libs
built `/MT` too: triplet **`x64-windows-static`**, NOT `x64-windows-static-md`
(the `-md` = dynamic CRT, chosen originally to match Rust's default `/MD`). `cc`
auto-selects `/MT` under `crt-static`, so `libmarpa` follows automatically.

**Scoped to the release leg, not project-wide.** Because flipping the triplet
forces every build that inherits it onto the static-CRT vcpkg libs, `+crt-static`
+ `x64-windows-static` live **only** in `release.yml`'s `build-windows` leg (env
`RUSTFLAGS` + `VCPKG*_TRIPLET`), leaving daily dev / `cargo test` / the dispatch
`windows-ci-manual-trigger.yml` on the fast, already-working `-md` setup. The CRT
model is invisible to test *logic*, and the release leg self-validates the link.

**CI guard.** `--version` on the runner passes even for a non-portable `.exe` (the
runner has the redist), so the release leg now also asserts, via `dumpbin
/DEPENDENTS` (dumpbin located through `vswhere`), that the shipped `.exe` imports
**no** `VCRUNTIME` — the property that actually guarantees clean-Windows launch.

**Perf footnote (the "slow as debug" report).** Not a build-flags issue — the
binary is full `maxperf`; warm conversions are ~80 ms. The ~300 ms *first-run*
cost is Windows Defender scanning the ~47 MB binary on first execution (confirmed:
every fresh copy pays it; repeat runs ~80 ms), amplified by the embedded 5-year
dump window's size. Mitigations: Defender exclusion (dev), code-signing
(distribution), or trimming the dump window; and for a fair Linux-vs-Windows
benchmark, discard the cold run.

## Explicitly out of scope (Windows)

- `cortex_worker` / the `cortex` feature (zmq fleet — production runs are Linux).
- The unix-socket LSP transport (`lsp_server/unix.rs`) — `generic.rs` is the
  Windows path; feature-completeness check is a Phase 3 nice-to-have.
- In-process libkpathsea linking — subprocess `kpsewhich` is the shipped Windows
  backend for now, BUT no longer "permanent": a static, in-process MSVC build was
  prototyped and implemented as an opt-in `vendored` feature in `rust-kpathsea`
  (branch `msvc-static-scope`, `docs/MSVC_STATIC_LINK_SCOPE.md`). It composes with
  Phase 5.1's `+crt-static` into a single `.exe` importing only core OS DLLs +
  in-process lookups. Deferred: publish `kpathsea_sys`/`kpathsea` (or a git dep),
  then flip the release leg. Gains perf on lookup-heavy documents (Windows process
  spawn is costly); the subprocess backend stays the zero-skew fallback.
- ARM64 Windows, `x86_64-pc-windows-gnu`, code signing.

## Risks / open questions

1. **libmarpa `cc`-port** is the long pole; everything in Phases 2-5 is
   unreachable until it lands. Start it first.
2. **MiKTeX year mapping** for dump selection is a design decision (rolling
   release vs TL-year dumps) — needs a maintainer call when reached (Phase 2.2).
3. **kpsewhich subprocess latency on Windows** may need cache work — measure
   before optimizing.
4. **vcpkg-crate vs env-var wiring** for `libxml`/`libxslt` crates may need
   upstream PRs to those crates.
5. **CRLF in user documents**: `mouth.rs` tokenization of CRLF inputs should be
   verified against Perl semantics (TeX treats `\r` as end-of-line) — add a
   CRLF-input regression test (Phase 3).
