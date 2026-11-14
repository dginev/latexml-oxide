# Building, testing and developing latexml-oxide

[← README](../README.md) · [Install](INSTALL.md) · [Usage](USAGE.md) · [docs index](README.md)

## Build from source

**1. Prerequisites.** Rust via [rustup](https://rustup.rs) — the checked-in
`rust-toolchain.toml` selects the nightly toolchain (with rustfmt and clippy) the project
needs — plus libxml2, libxslt and a TeX Live installation:

* **Ubuntu / Debian**
  ```
  $ sudo apt install pkg-config libxml2-dev libxslt1-dev libkpathsea-dev mold \
                     texlive texlive-latex-extra texlive-science texlive-bibtex-extra \
                     texlive-publishers imagemagick ghostscript poppler-utils
  ```
  On x86_64 Linux the build links with `mold` (`.cargo/config.toml`), so install it on
  any distribution.
* **macOS** (Homebrew)
  ```
  $ brew install pkgconf libxml2 libxslt texlive
  $ export PKG_CONFIG_PATH="$(brew --prefix libxml2)/lib/pkgconfig:$(brew --prefix libxslt)/lib/pkgconfig"
  ```
  libxml2 and libxslt are keg-only, hence the export (put it in your shell profile).
  MacTeX/BasicTeX works too: it ships no `libkpathsea`, so the build falls back to its
  `kpsewhich` (`/Library/TeX/texbin` on `PATH`), with slightly slower cold file lookups.
* **Windows** (MSVC toolchain, PowerShell)
  ```
  > vcpkg install libxml2:x64-windows-static-md libxslt:x64-windows-static-md
  > $env:VCPKG_ROOT = "<your vcpkg directory>"
  > $env:VCPKGRS_TRIPLET = "x64-windows-static-md"
  ```
  plus TeX Live for Windows, and LLVM for `bindgen` (set `LIBCLANG_PATH` if it is not
  found) — as in `.github/workflows/windows-ci-manual-trigger.yml`.

**2. Build and test**, from the repository root:

```
cargo build
tools/make_formats.sh          # Windows: tools\make_formats.ps1
cargo build
cargo test
```

`make_formats` runs the new binary with `--init=plain.tex` and `--init=latex.ltx` against
your TeX Live, writing the kernel format dumps to `resources/dumps/`; the second
`cargo build` embeds them in the binary. **Rerun the last three steps after updating TeX
Live:** a dump is used only with the TeX files it was built from, and without one every
executable — each test binary included — first builds its own format into a per-user
cache, which is slow.

[`cargo-nextest`](https://nexte.st) (`cargo install cargo-nextest`) runs the same suite
several times faster: `cargo nextest run --workspace`.

With [EPUBCheck](https://www.w3.org/publishing/epubcheck/) installed (`apt install epubcheck`), the EPUB tests
also validate the `.epub` they write; without it they print `SKIP epubcheck …`, and
`LATEXML_EPUBCHECK_TESTS=1` turns that skip into a failure.

## Build profiles

`Cargo.toml` defines named profiles for day-to-day development (`test`), the
RAM-bounded CI runner (`ci`), local perf measurement (`release`), and the shipped
distribution binaries (`maxperf` / `maxperf-cortex`). Use the default profile
(`cargo build` / `cargo test`, no flag) for everyday work; see
[CLAUDE.md](../CLAUDE.md) for the full profile table.

## Development Tips

The first build points this checkout's `core.hooksPath` at `.githooks/`; its pre-push
hook runs `tools/lint.sh` — rustfmt, clippy with `-D warnings`, rustdoc and the other
checks CI's `lint` job runs — and blocks the push on a failure. Run it directly to check
before pushing.

This workspace is heavy for rust-analyzer (large proc-macro bodies); the checked-in `.vscode/settings.json`
ships a stability profile that keeps the IDE responsive.
Terminal `cargo build` / `cargo test` are unaffected.

To generate the project documentation locally, run:
```bash
$ cargo doc --workspace --no-deps --open
```

Test fixtures (`[name].tex` + `[name].xml` pairs) are discovered at compile time: after
adding one, run `cargo clean` so the build finds it.

A test that needs something outside the base TeX Live set — a package or class that some
TeX Live installs lack (a distro's partial package set, an older year), a minimal TeX Live
year, a program such as `jing` — declares it, and is skipped where it is not met. This is
Perl LaTeXML's `requires` (`LaTeXML/Util/Test.pm`, `check_requirements`): a fixture
directory's `tex_tests!` takes a table of `(test, Requirements)` rows (`"*"` for every
test), and a guard starts with `if !check_requirements(&["tcolorbox.sty"]) { return; }`, listing
every file of its load set a TeX Live may lack (or with
`Requirements::packages(…).texlive_min(2021).programs(&["jing"]).met()`). Files are looked
up the way the conversion finds them. A skip prints `Skip: <reason>` and appends
`<test>\t<reason>` to `test-skips.log` in the cargo target directory (or
`$LATEXML_TEST_SKIP_LOG`), which CI lists after the tests; a met requirement runs the test.

## Build features

Cargo features, and what the shipped binary turns on. The releases are built with
`--no-default-features --features runtime-bindings` — that drops `test-utils` but
deliberately **keeps** the Rhai runtime bindings below.

| Feature | Crate | In releases? | What it does |
|---|---|---|---|
| `runtime-bindings` | `latexml` → `latexml_contrib` | **yes** (also a default) | Embeds the [Rhai](https://rhai.rs) interpreter so you can add or override TeX package/class bindings **at runtime** — no Rust toolchain, no recompile. See the [Rhai guide](parity/RHAI_BINDINGS_GUIDE.md). |
| `kpathsea` | `latexml_core` | **yes** (a default) | Resolves `.sty`/`.cls`/`.tfm` from your TeX tree via libkpathsea in-process, falling back to the `kpsewhich` subprocess when the library is absent (e.g. MacTeX). |
| `test-utils` | `latexml` | no (dropped) | Test-suite discovery helpers; drops `glob` and the A/B comparator's `roxmltree` + `similar` from the distribution build. |
| `token-locators` | `latexml` (+4 crates) | no | Widens `Token` from 8 to 12 bytes so each token carries its exact source offset through expansion into digestion — the precision build behind `--source-map`. Off everywhere else so normal conversions don't pay. |
| `cortex` | `latexml` | no (worker image only) | The `cortex_worker` binary for the arXiv-scale fleet. |
| `jemalloc`, `dhat-heap` | `latexml` | no | Allocator swap / heap profiling, for performance work. |
