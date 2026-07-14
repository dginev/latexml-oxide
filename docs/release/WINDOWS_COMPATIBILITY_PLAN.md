# Windows Compatibility — SHIPPED

**Status:** shipped. The native Windows `latexml_oxide.exe` was cut as tag
`0.7.4-rc2` (2026-07-14) on the `windows-compatibility` branch. This document
was the phased bring-up plan (Phases 0–5); it is now a SHIPPED summary plus a
short REMAINING worklist. It operationalized portability rung 5 ("Windows —
deferred") of [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §portability.

Decisions locked with the maintainer (2026-07-12): **MSVC** target
(`x86_64-pc-windows-msvc`); **vcpkg-static** libxml2/libxslt; **TeX Live AND
MiKTeX** must both work at runtime; a self-contained `.exe` alongside the
Linux/macOS assets. No MinGW/GNU bring-up phase.

> **Release-dump caveat:** Windows test-suite green does NOT imply TL2026
> release-readiness — `--init=latex.ltx` on TL2026 is not dump-gate-clean (137
> expl3-catcode errors; see REMAINING and `SYNC_STATUS.md`). 2026 is out of the
> release dump window; TL2026 hosts fall back to the newest embedded dump.

## Toolchain & native deps (reproduce the build from here)

| Piece | Choice |
|---|---|
| Target | `x86_64-pc-windows-msvc` (MSVC; no MinGW/GNU) |
| vcpkg triplet | **release:** `x64-windows-static` (static C libs + static CRT `/MT`) · **dev/CI:** `x64-windows-static-md` (static libs, dynamic CRT `/MD`) |
| Build env | `VCPKG_ROOT`, `LIBCLANG_PATH` (bindgen), nightly MSVC toolchain |
| C deps | libxml2 + libxslt via vcpkg; libmarpa via a `cc`-port; kpathsea via `build_from_source`. **All four are the project author's own crates — Windows fixes land upstream in the crate (branch + `[patch]` while iterating, then publish + re-point), never as workspace-side forks/hacks.** |

Upstream one-liners that unblocked the compile (witnesses): libmarpa `cc`-port
(`dginev/marpa`, `64c045c` — synthesizes `config.h` from `LIB_VERSION`,
compiles the source list with `cc::Build`, whole marpa suite green on MSVC);
rust-libxml vcpkg arm emitting `bcrypt`+`ws2_32` (`8e40ba00` — libxml2 ≥ 2.12
calls `BCryptGenRandom`); rust-libxslt vcpkg arm (`9fa9a6d8`).

Prefer portable ecosystem helpers over hand-rolled `cfg(windows)` branches
(`std::env::split_paths`/`join_paths` for `PATH`; `tempfile` / `temp_dir()` for
scratch) — the established policy, not a new one.

## SHIPPED

### Build & toolchain

- [x] **`.gitattributes` LF enforcement** (`* text=auto eol=lf`) on every
  platform. Golden-file tests split on `\n` over raw bytes; a default Windows
  git (`core.autocrlf=true`) checks out CRLF and would fail every `.tex`/`.xml`
  regression.
- [x] **Workspace compiles + links on MSVC** (`cargo check --workspace
  --all-targets` clean). Phase-0 `cfg(unix)` gaps closed: the ungated
  `libc::dlsym` in `latexml_post/src/xslt.rs`, `/tmp`→`std::env::temp_dir()`,
  and the `getrusage` / process-group / timeout-kill fallbacks.
- [x] **Fully-static release `.exe`** (was "Phase 5.1"): `-C
  target-feature=+crt-static` + the `x64-windows-static` triplet collapse the
  import closure to core OS DLLs only — no `VCRUNTIME140*`, no VC++
  redistributable needed on a clean Windows. The CRT model must be **uniform**
  across the image — `/MT` (static) and `/MD` (dynamic) cannot mix (two heaps →
  corruption) — so `+crt-static` (Rust `/MT`) forces the vcpkg libs to `/MT`
  too, hence `x64-windows-static` (NOT `-md`). Scoped to `release.yml`'s
  `build-windows` leg only; dev / `cargo test` / dispatch CI stay on the fast,
  already-working `-md` setup.

### kpathsea / TeX resolution

- [x] **In-process static libkpathsea** — `kpathsea 0.3.2` / `kpathsea_sys
  0.2.2` (crates.io), feature `kpathsea-build-from-source` fetches + compiles a
  STATIC libkpathsea from source. No runtime `kpathsealibw64.dll`. This
  SUPERSEDES the old subprocess-only `KPATHSEA_NO_LINK=1` distribution recipe
  and the earlier "in-process libkpathsea is out of scope" position. (Origin
  landmine, now moot: a build made with TeX Live on PATH used to silently link
  TL's `kpathsealibw64.dll` and then fail to *launch* on a MiKTeX-only box — a
  static in-process link removes the DLL dependency entirely.)
- [x] **Runtime backend auto-selection** —
  `latexml_core/src/util/pathname.rs::select_kpaths`: in-process on TeX Live
  (fast, no per-lookup subprocess), subprocess fallback on MiKTeX (whose MPM
  `fndb` a static libkpathsea can't read — MiKTeX ships no `ls-R`). Detected by
  the `kpsewhich --version` "MiKTeX" banner plus a universal `cmr10.tfm`
  sentinel probe. One binary works on no-TeX / MiKTeX / TeX Live.
- [x] **MiKTeX on-the-fly installer suppression** (`--miktex-disable-installer`,
  kpathsea 0.3.2) — a not-installed package now resolves not-found fast instead
  of raising a BLOCKING GUI install prompt that hung conversions into a ~60 s
  wall-clock fatal.
- [x] **One `kpsewhich` per process** — `ambient_kpsewhich_version()` memoizes a
  single shared `kpsewhich --version` banner; ambient-TeX-year detection is
  memoized too (MiKTeX's rolling `MiKTeX YY.MM` stamp maps `25.12`→`2025`,
  nearest-dump fallback otherwise). Took MiKTeX `--whatsin=math` from ~3.2 s to
  ~0.5 s; TeX Live ~0.31 s.
- [x] **Search-path parity with Perl `Core.pm:50`** — cwd is searched first for
  relative input (fixes no-TeX relative input); a `--path` ending in `//` is
  searched recursively (kpsewhich convention), symlink-cycle-guarded by deduping
  on the canonical path.

### Runtime correctness

- [x] **Pathname layer** normalizes `\`→`/` at the single choke point
  (`canonical()`), joins with `/` like Perl `pathname_concat`, and adds
  drive-letter (`C:/…`) absolute-path awareness. Fixed same-day: `concat` (was
  `PathBuf::push`→`\`), zip/pack entry names (`/` per spec), overlay mtime
  (Windows needs a write handle for `set_modified`).
- [x] **Perl-parity `xsltMaxDepth = 1000`** restored on Windows via a direct
  `extern "C"` static (static libxslt → symbol in our own image, undecorated on
  x64 COFF; `GetProcAddress`/`dlsym` not applicable).
- [x] **Graphics delegates** resolved by Windows names (matrix below).
- [x] **CRLF-input regression guard landed** (was tracked as risk #5):
  `latexml_core/src/mouth.rs::split_raw_lines_universal_newlines` asserts
  CRLF/CR/LF all split identically and the terminator never leaks a raw `\r`.

Windows graphics-delegate names:

| Role | Windows binary(ies) |
|---|---|
| ImageMagick | `magick` — NOT `convert` (Windows `convert.exe` is the System32 filesystem tool) |
| Ghostscript | `gswin64c` / `gswin32c` / `mgs` (MiKTeX) / TL's bundled `rungs` |
| PDF | `mutool` (MuPDF), `pdftocairo` (poppler) |
| ps2pdf | a `.bat` on TL-Windows (spawned via `cmd.exe`; watch CVE-2024-24576 strict arg-escaping — fall back to direct `gs -sDEVICE=pdfwrite` if rejected) |

### Tests

- [x] **Full suite green on Windows MSVC.** Run the RELEASE suite with
  `CARGO_PROFILE_RELEASE_LTO=false cargo test --release --tests --workspace` —
  otherwise thin-LTO re-links each of ~60 test binaries over the whole
  workspace (~2 h wall). LTO is a distribution-only optimization; correctness is
  unaffected. The `ci` profile (which CI uses) sidesteps it. Profile physics,
  not a Windows defect — Linux pays the same cost.
- [x] **Test-fixes landed:** 4× `90_latexmlpost` rewritten to parse both sides
  in-process with the linked libxml2 + line-diff via `similar` (removes the
  bash/`xmllint`/`diff`/`grep`/`wc` dependency on ALL platforms); `greek_test`
  fixed TL-independently via native `\Declare*caseMapping` handlers; the
  test-discovery proc macro (`latexml_codegen`) emits `/`-normalized paths. The
  tikz `ac_drive_components_test` is circuitikz-version drift (12.4→12.68), so
  it is compared on Linux/macOS and skipped on Windows (`WINDOWS_GOLDEN_SKIP`) —
  a portability difference, not a code divergence (`SYNC_STATUS.md`).
- [x] **`tools/make_formats.ps1`** — bash-free dump generation mirroring
  `make_formats.sh` (the `.sh` stays authoritative).

### Release

- [x] **`release.yml` `build-windows` leg** (`windows-latest`): builds the
  maxperf `.exe` with `RELEASE_EXTRA_FEATURES=kpathsea-build-from-source` +
  `RUSTFLAGS=-C target-feature=+crt-static`, `VCPKG*_TRIPLET=x64-windows-static`,
  `KPATHSEA_SKIP_TOOLCHAIN_CHECK=1` (no TeX on the runner). The OS-agnostic
  TL-window dumps are embedded from the shared `dumps` job. Guard: a `--version`
  launch smoke **plus** `dumpbin /DEPENDENTS` (located via `vswhere`) asserting
  the import table carries no `vcruntime`/`kpathsea`/`libxml`/`libxslt` — the
  property that actually guarantees clean-Windows launch (`--version` alone
  passes on the redist-equipped runner). Published as
  `latexml-oxide-<ver>-x86_64-pc-windows-msvc.exe` (+ `.sha256`); RC tags
  publish a draft prerelease.
- [x] **Verified locally:** 61.6 MB (release-profile), OS-DLLs-only, converts
  under no-TeX / MiKTeX / TeX Live; embedded dumps confirmed by renaming
  `resources/dumps` away (no degraded-mode warning).

### Perf note (corrects the earlier "slow as debug" report)

The dominant Windows/MiKTeX first-run cost was **unmemoized ambient-TeX-year
detection** spawning `kpsewhich`+`pdflatex` from ~5 sites per conversion
(~340 ms each on MiKTeX) — fixed by the one-kpsewhich-per-process memoization
above (~3.2 s → ~0.5 s). Windows Defender scanning the multi-tens-of-MB binary
on first execution is a *secondary*, cold-start factor (repeat runs ~80 ms);
mitigate with a Defender exclusion (dev) or code-signing (dist), and discard the
cold run for a fair Linux-vs-Windows benchmark.

## REMAINING / OPEN

- [ ] **Promote a `windows` job into `CI.yml` as a required leg.** CI.yml
  currently has NO windows job (lint / miri / test / macos only); Windows CI is
  dispatch-only (`.github/workflows/windows-ci-manual-trigger.yml`). Mirror the
  `macos` job (rustup MSVC, cached vcpkg, `setup-texlive-action`,
  `make_formats.ps1`, `cargo test --profile ci`), then mark it required. Keep
  the MiKTeX smoke as a separate scheduled/dispatch leg (full suite × 2 distros
  per PR is not worth the 2× Windows minutes).
- [ ] **README + `RELEASING.md` platform status.** README omits Windows
  entirely; `RELEASING.md` still lists Windows as "out of scope for now" and a
  "deferred matrix rung." Update both to reflect the shipped `.exe` (and its
  runtime prereqs: TeX Live or MiKTeX on PATH; ImageMagick / Ghostscript /
  MuPDF / poppler optional for graphics).
- [ ] **kpathsea license entry.** kpathsea is LGPL-2.1 and is now STATICALLY
  linked (Windows `build_from_source`; Linux/macOS
  `tools/build_static_kpathsea.sh`). It is missing from `LICENSE_INVENTORY.md`
  and absent from `THIRD-PARTY-NOTICES` §3 (no kpathsea entry; no §6 relink note
  found). Add it and confirm the static-LGPL relink obligation is covered.
- [ ] **expl3 / TL2026 `latex.ltx` dump gate.** `--init=latex.ltx` on TL2026
  emits 137 raw-load expl3-catcode-gap errors
  (`EXPL3_CATCODE_GAP_2026-06-08.md`), so 2026 cannot join the release dump
  window (`SYNC_STATUS.md`: "TL2026 latex.ltx dump init is NOT
  release-gate-clean"). Deep, pre-existing, Linux-equivalent gap — first
  post-release work.
- [ ] **Fast TeX-ecosystem CI install** (DEFERRED). The cold
  `setup-texlive-action` download (~1–2 GB) is the slowest CI step. Target: a
  pinned, content-hash-keyed minimal texmf snapshot (fast + deterministic +
  golden-stable, pinned to the validated TL year — a different distro's package
  universe shifts version-sensitive fixtures). Keep the deliberately-non-golden
  MiKTeX smoke on a separate leg. No CI change today.

## Out of scope

- `cortex_worker` / the `cortex` feature (zmq fleet — production runs are Linux).
- Unix-socket LSP transport (`lsp_server/unix.rs`) — `generic.rs` is the Windows
  path.
- ARM64 Windows, `x86_64-pc-windows-gnu`.
- Code-signing (record in `RELEASE_CRITERIA.md` if users hit SmartScreen
  friction).
