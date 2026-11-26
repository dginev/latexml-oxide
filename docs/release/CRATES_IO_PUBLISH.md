# Publishing `latexml` to crates.io

The crates.io side of a release: `cargo publish`, docs.rs, and use as a library. The GitHub Release binaries are
[`RELEASING.md`](RELEASING.md)'s.

**Status.** All 8 workspace crates are on crates.io and are published at every stable release, first at 0.7.4; the last
published line is 0.7.6 (2026-08-23). From 0.8.0, an install from crates.io carries no format dumps and builds them on
its first conversion into a per-user cache (`LATEXML_FORMAT_CACHE`, `latexml_oxide/src/format_dumps.rs`;
`docs/INSTALL.md` "From crates.io").

**Each release bumps every changed crate and runs `cargo publish --workspace`** (`RELEASING.md` step 1). crates.io is
immutable, so a changed sibling left at its old version publishes the top crate against stale siblings. The per-release
checklist is §5. **Rule 0: never publish an `-rc`.** `cargo install` and `latexml = "0.8"` ignore pre-releases, so
`cargo install latexml` would keep resolving to the previous stable while the README inside the crate tells people to
run exactly that.

---

## 1. Publish order (dependency-first)

`cargo publish` needs every dependency on crates.io already, so the crates publish bottom-up. `cargo publish
--workspace` sorts them itself, in this order:

| # | crate (dir) | version (0.8.0 line) | workspace crates it needs first |
|---|-------------|----------------------|---------------------------------|
| 1 | `latexml_core` | 0.7.0 | — |
| 2 | `latexml_codegen` | 0.5.0 | core |
| 3 | `latexml_math_parser` | 0.5.0 | core |
| 4 | `latexml_engine` | 0.8.0 | codegen, core |
| 5 | `latexml_package` | 0.8.0 | codegen, core, engine |
| 6 | `latexml_post` | 0.6.0 | core |
| 7 | `latexml_contrib` | 0.6.0 | core, codegen, engine, package |
| 8 | `latexml` (`latexml_oxide`) | 0.8.0 | all 7 (+ `pericortex`, optional) |

External prerequisites, both on crates.io: `marpa-asf` 0.4.0 (over `libmarpa-asf-sys` 0.3.x; the `dginev/marpa` fork),
consumed by `latexml_math_parser` as `marpa = { package = "marpa-asf", version = "0.4.0" }`; and `pericortex` 0.2.8+,
behind the optional `cortex` feature.

---

## 2. Packaging rules

These keep the crates publishable; each was a blocker once, and code comments cite their labels (B1–B8).

- **B1 — every intra-workspace dependency carries `version` beside `path`.** crates.io rejects a bare `{ path = "…" }`;
  local builds still use the path.
- **B2 — no git dependencies, even optional ones.** crates.io rejects them. The marpa fork publishes as
  `libmarpa-asf-sys`/`marpa-asf` (the upstream names are taken), and the `package` alias keeps the `marpa::` /
  `libmarpa_sys::` code unchanged. `marpa-asf` 0.4.0 compiles a byte class (`byte_set`, `byte_range`,
  `inverse_byte_set`) to one terminal, so a token glade's symbol is the class and its byte is `token_value() - 1`: a
  breaking change for an ASF consumer, and `latexml_math_parser` requires it.
- **Check the `cortex` feature by hand.** CI does not build it (only the worker image does), so before a publish run
  `cargo check -p latexml --features cortex` as well as `cargo check -p latexml`.
- **B3 — each crate carries the resources it embeds.** `cargo package` cannot include `../` paths. B3a: XSLT, CSS and
  JavaScript live in `latexml_post/resources/` (`include_str!` resolves relative to the source file, so the build is
  independent of the working directory). B3b: the RelaxNG tree lives in `latexml_core/resources/RelaxNG/` (108 files):
  `latexml_core/build.rs` embeds it, and the `load_model!` derive (exported from `latexml_engine`, invoked in
  `latexml_oxide`) reads `LaTeXML.model` from that embedded table, not from the working directory. **Gate: `cargo build
  -p latexml`**, the crate where the derive expands; a `-p latexml_core` or `-p latexml_engine` build is a false green.
- **B5 — release-only inputs stay at the workspace root, outside every crate.** The per-TL-year dumps
  (`resources/dumps/`, gitignored, built by `release-dumps.yml` / `make_formats.sh`, embedded by
  `latexml_engine/build.rs`) and the bundled `ar5iv.css` (`resources/ar5iv-css/`; RELEASING.md "The bundled ar5iv.css")
  never reach a crate, so a crates.io install or a docs.rs build has neither: `latexml_post/build.rs` embeds no
  `ar5iv.css`, and `--css=ar5iv.css` warns and asks for a path to a local copy. Moving the dumps into a crate would buy
  nothing and entangle the release and CI cache paths, the Dockerfile, `.gitignore`/`.gitattributes` and `ini_tex.rs`'s
  `--init` write path. Rejected: a prepublish copy-and-rewrite script (the `include_str!` paths are compile-time
  literals) and per-crate symlinks into `../resources` (Windows-hostile).
- **B6 — `latexml_oxide/README.md` is a symlink to `../README.md`,** not a copy (a copy would drift from the README
  GitHub serves). Cargo packages the real content. A Windows checkout without symlink support turns it into a 12-byte
  text file, so publish from Linux.
- **B7 — `latexml_oxide/tests/` is excluded** (`exclude = ["tests/"]`). crates.io caps an upload at 10 MiB, and the
  regression corpus was ~19.5 of the crate's ~20 MiB. Nothing in `src/` or `bin/` reads `tests/` at build time, and
  `build.rs` only wires git hooks (a no-op without `.git`). Do **not** exclude the fixture subdirectories while keeping
  `tests/*.rs`: the `testable` proc-macro scans the corpus at compile time, so the packaged crate's `cargo test` would
  fail. The two explicit `[[test]]` targets (`cortex_worker_exit_reports`, `cortex_worker_untrusted_bindings`, both
  `cortex`-gated) are new since the last publish; `cargo publish --workspace --dry-run` shows how cargo treats them with
  their files excluded.

**Why the dumps are not built in `build.rs`.** A dump comes from running the fully linked binary (`make_formats.sh`
builds `latexml_oxide`, then runs `--init`), but a build script runs before its own crate compiles: an ordering cycle.
Breaking it would compile the engine twice, require TeX Live at build time (docs.rs has none, so `cargo install` would
fail outright), and bake a dump that goes stale on the next TeX Live upgrade. The first-run build into the per-user
cache avoids all three. (A 0.7.6 install has no first-run build; `cd ~/.cargo && latexml_oxide --init=plain.tex &&
latexml_oxide --init=latex.ltx` writes `~/.cargo/resources/dumps/`, the loader's `<exe_dir>/../resources/dumps`;
`LATEXML_DUMP_DIR` overrides the location.)

### Two traps `--dry-run` cannot catch

`cargo publish --workspace --dry-run` packages and verifies, then stops *before* uploading. A green dry-run says the
*contents* are sound, not that the publish will go through:

1. **B7 — the 10 MiB size cap is enforced only on upload.** The CDN in front of crates.io reports an oversized upload as
   a bare `503 backend write error` with an empty body, not a `413 Payload Too Large`. If a publish 503s, suspect the
   size first: `cargo package --list | wc -l`.
2. **B8 — new crate names are rate-limited:** a burst of 5, then about one per 10 minutes, and `cargo publish
   --workspace` aborts on the `429` instead of waiting. Finish with `cargo publish -p <crate>` per remaining crate as
   the limit refills (the `429` names the retry time; `latexml` depends on `latexml_contrib`, so it goes last). This
   applies only to new names; a version bump of an existing crate has a far looser limit.

---

## 3. docs.rs and the `documentation` link

docs.rs builds the docs on publish (nothing to upload), with a nightly compiler (so `#![feature(…)]` is fine), in a
no-network sandbox that sets `DOCS_RS=1`. Our `build.rs` scripts are docs.rs-safe: git calls are no-ops without `.git`,
`latex_dump.rs`/`plain_dump.rs` are committed, and a missing resource directory compiles to an empty embed.
libxml2/libxslt come from the docs.rs image, and `kpathsea` 0.3 falls back to its subprocess backend without
`libkpathsea`.

`latexml_oxide/Cargo.toml` has:
```toml
documentation = "https://dginev.github.io/latexml-oxide/latexml/"   # self-hosted rustdoc, always works
[package.metadata.docs.rs]
targets = ["x86_64-unknown-linux-gnu"]
```
The `documentation` link points at the gh-pages rustdoc (deployed by `.github/workflows/rustdoc.yml` on every push to
`main`), so the crate page links to working docs even if a docs.rs build regresses.

---

## 4. Using `latexml` as a library

`latexml::api` is the batteries-included entry point, with no binary and no `Config` wiring:
```rust
let xml  = latexml::api::convert_to_xml(tex)?;   // TeX → LaTeXML XML
let html = latexml::api::convert_to_html(tex)?;  // TeX → HTML5 + Presentation MathML
let opts = latexml::api::HtmlOptions::default().with_default_resources(false);
let html = latexml::api::convert_to_html_with(tex, &opts)?; // no CSS/JS links, nothing written
```
`convert_to_html` links the bundled CSS/JS and, with no destination, writes them into the working directory;
`with_default_resources(false)` is the `--nodefaultresources` equivalent. Each call runs on its own 256 MiB-stack worker
thread and frees the thread-local engine (`reset_thread_engine`) before the thread exits. For finer control (preloads,
search paths, `--whatsin`, split, encoding, …) drive `latexml::converter::Converter` and `latexml::post` directly. A TeX
distribution must be on `PATH` at runtime, as for the binary.

**One engine, two front ends.** `api` and the binary share the engine (`Converter` + `post::run_post_processing`) and
the per-format stylesheet choice (`post::default_stylesheet`). `PostOptions` implements `Default`, so a new field does
not force `api.rs` to set it: check `api.rs` when adding one. The CLI orchestration still lives only in the binary
(`bin/latexml_oxide.rs::real_main`: archive and directory detection, split, zip packing, telemetry, the watchdog, XML
input, `--whatsout`). **TODO:** hoist that CLI-agnostic core into `latexml::api::run(config)`, so the binary becomes a
thin CLI shell and there is one high-level conversion path.

---

## 5. Publish checklist

Once per project (done): the repository is public, since `repository`/`homepage` on all 8 crates and every README badge
point at `github.com/dginev/latexml-oxide` and a crates.io release is irreversible (yankable, never deletable); all 8
names are registered to dginev.

Per release, on the **stable** `X.Y.Z` tag, never an `-rc`:

1. Bump every crate changed since the last release and every intra-workspace `path`+`version` pin (`RELEASING.md` step
   1).
2. `cargo check -p latexml --features cortex` (§2).
3. `cargo publish --workspace --dry-run` clean.
4. `cargo publish --workspace` (background it: the index waits are long), then confirm each crate's `max_version`
   against the crates.io API — exit 0 alone is not proof.
5. Confirm docs.rs built `latexml` (the `documentation` link is the fallback).
6. Check that the `cargo install latexml` note (`docs/INSTALL.md` "From crates.io") still carries the nightly,
   build-dependency and first-run dump caveats.
