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
- [x] **Opt-in CI bring-up workflow** `.github/workflows/windows-bringup.yml`
  (`workflow_dispatch` + pushes to `windows-compatibility`), staged so each step
  reports its own failure. It is *expected* to fail in early phases — it exists
  to give every subsequent phase a live Windows probe without turning main CI red.

## Phase 1 — make the workspace COMPILE on `x86_64-pc-windows-msvc`

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
6. **Smoke matrix:** `latexml_oxide --format=html5 --dest=paper.html paper.tex`
   on (TeX Live, MiKTeX) × (plain doc, math-heavy doc, EPS/PDF graphics doc),
   diffed against Linux output. Divergences triaged per `canvas-triage` rules
   (fail toward flagging).

## Phase 3 — make `cargo test --release` (and `--profile ci`) PASS on Windows

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

## Phase 4 — CI: `windows-latest` job as a required leg

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

## Phase 5 — release artifact

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

## Explicitly out of scope (Windows)

- `cortex_worker` / the `cortex` feature (zmq fleet — production runs are Linux).
- The unix-socket LSP transport (`lsp_server/unix.rs`) — `generic.rs` is the
  Windows path; feature-completeness check is a Phase 3 nice-to-have.
- In-process libkpathsea linking (subprocess `kpsewhich` is the permanent
  Windows backend).
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
