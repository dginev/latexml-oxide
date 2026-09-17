# Windows workflow validation — 2026-09-17

Point-in-time check that the perfect-kernel workflow (build → dump generation →
raw-interpretation conversion) is healthy on Windows, on a host carrying **both**
TeX Live 2026 (`C:\texlive\2026`) and MiKTeX 25.12. Binary: `latexml_oxide 0.7.6`
built from the `perfect_kernel` tip (`0796e0cb54`), `x86_64-pc-windows-msvc`,
release profile.

## Verdict

The Windows workflow is healthy. Nothing on the branch is Windows-incompatible:
the 431-commit kernel diff is platform-neutral (no new `std::os::unix`/`libc`,
no `/usr/local/texlive` or `/tmp` literals), and it inherits every Windows-port
PR already in `main` (#278, #286/#287, #464–#466, #752).

## What was validated

1. **Build** — `cargo build --release --bin latexml_oxide` succeeds on
   Windows/MSVC (8m38s cold-ish; the C deps libxml/libmarpa compile fine).

2. **Dump generation on Windows is healthy (new — releases have only ever built
   dumps on Linux CI).** `latexml_oxide --init=plain.tex` and `--init=latex.ltx`
   run against TL2026 both complete at the zero-`Error:`/`Fatal:` parity bar
   (CLAUDE.md rule 4), writing `resources/dumps/{plain,latex}.2026.dump.txt`.
   The Windows-built `latex.2026.dump.txt` matches a known-good Linux-origin dump
   within ~0.2%: **24,387 vs 24,355 entries, 24,330 keys in common**; the 57
   Windows-only keys are benign environment-sensitive bookkeeping (file-path
   tracking — `\CurrentFile`, `\CurrentFilePath`, `\filename@path`,
   `\@currnamestack` — and alignment/interaction state `LAST_ALIGNMENT_COLS/ROWS`,
   `INTERACTION_MODE`); the 3 reference-only keys are `%00`-charcode artifacts. No
   core-kernel or expl3 definition differs.

3. **Both TeX toolchains resolve files** (`rust-kpathsea` fallback ladder:
   in-process libkpathsea → subprocess `kpsewhich` on PATH → error; plus the
   dual-TL agreement probe in `pathname.rs`). Clean (0-error) conversions with
   TL2026 on: a hyperref/xcolor/amsmath article (13s); a **bindingless** raw-load
   doc (`ninecolors`+`spverbatim`, confirming real `.sty` source is read through
   the kernel); and the nicematrix exemplar (38s). MiKTeX also converts installed
   packages (nicematrix 0 errors, 97s) but is slower (subprocess `kpsewhich` per
   file; no libkpathsea) and only sees packages it has already installed on demand
   — an uninstalled package surfaces as `Warning:missing_file … no raw file found
   on disk`, not a bug.

4. **Windows-built dump ≡ Linux-origin dump, behaviorally.** The full KOMA-Script
   guide (`scrguide-en.tex`, ~450pp; source lives in TL's `source/latex/koma-script/doc/`)
   converts to a **byte-identical 133,660-byte XML** under both dumps, hitting the
   **same `Fatal:Stomach:Recursion` at the same file** (`common-options-en.tex`).
   That fatal is a document-intrinsic difficult case — scrguide.cls's recursive
   `\@index`/`\@indexphrase` machinery — reproduced identically with either dump,
   so it is neither Windows- nor dump-related. (The Windows run also logged one
   extra `Error:undefined:\@nil`, an output-neutral non-determinism in the
   fatal-recovery path; `\@nil` is a delimiter token, defined in neither dump.)
   Note scrguide is **not** a formal corpus entry — `enumerate_corpus.sh` requires
   the `.tex` beside its PDF under `doc/latex/`, and this manual's source ships
   under `source/`, so the Linux sweeps do not include it either.

5. **Harness runs on Git Bash** — `tools/perfect_kernel/run_doc.sh` produced a
   valid `verdict.tsv` (`status 0`); `ulimit -v` is accepted (enforcement is moot,
   `--max-memory` covers the RAM guard, #466).

## Reproduction recipe (TL2026, Windows/Git Bash)

`tools/make_formats.sh` is **Linux-only by design** — do not run it on Windows.
Generate the dump by invoking the two `--init` passes directly (this is exactly
what `make_formats.sh` does after its build step), with TeX Live 2026 first on
PATH and `TEXMF*` pinned to it, then convert:

```bash
export PATH="/c/texlive/2026/bin/windows:$PATH"
export TEXMFROOT=C:/texlive/2026 TEXMFDIST=C:/texlive/2026/texmf-dist \
       TEXMFCNF=C:/texlive/2026/texmf-dist/web2c
# Dump generation (writes resources/dumps/{plain,latex}.2026.dump.txt relative to CWD):
latexml_oxide.exe --init=plain.tex
latexml_oxide.exe --init=latex.ltx
# Convert with raw interpretation (dump auto-found via the dev-tree path next to
# the binary; or set LATEXML_DUMP_DIR to the dump directory explicitly):
latexml_oxide.exe --preload='[rawstyles,rawclasses]latexml.sty' --xml \
  --dest=out.xml manual.tex
```

## Gotchas found

- **A fresh `git worktree` has no `resources/dumps/`** (it is git-ignored and
  generated), so a binary built there embeds NO dump → the degraded raw path →
  `\providecommand`/`\AddToHook` undefined and a `\c__codepoint_nfd__tl already
  defined` ×N expl3 storm on every document. Fix: build in a checkout that has
  `resources/dumps/` populated (generate it on Linux via `make_formats.sh`, on
  Windows via the direct `--init` recipe above), or point the runtime at an
  existing dump directory with `LATEXML_DUMP_DIR`.
- **`make_formats.sh` is Linux-only by design — do not "fix" it for Windows.** On
  Windows, generate the dump with the two `--init` passes directly (recipe above);
  they produce a dump validated healthy here. For reference, the reasons it does not
  run as-is on Windows are that it invokes the binary by the hardcoded relative path
  `target/$PROFILE/latexml_oxide` (no `.exe`, no `CARGO_TARGET_DIR` awareness) — but
  the direct `--init` path is the supported Windows flow, not a patched script.
- **The doc corpus itself is not installed here** — TL2026 was installed without
  the `doc/latex` tree and MiKTeX fetches docs on demand, so
  `enumerate_corpus.sh` returns 0. The corpus-scale S0∧S1 number cannot be swept on
  this box without installing TeX Live's full doc scheme; per-document behavior,
  however, is deterministic and matches Linux given the same sources + a
  matching-year dump.
