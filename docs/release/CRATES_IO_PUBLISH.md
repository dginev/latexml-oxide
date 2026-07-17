# Publishing `latexml` to crates.io

Living checklist for the first crates.io release of the workspace and the
docs.rs / library-consumer story. Complements
[`RELEASING.md`](RELEASING.md) (the GitHub-Release binary flow); this file is
specifically about **`cargo publish` + docs.rs + library use**.

Status (2026-07-16): the two **forked git-dep** blockers are cleared — the
dginev/marpa fork is published as **`marpa-asf`** 0.3.0 + **`libmarpa-asf-sys`**
0.3.0, and **`pericortex`** 0.2.8 is published; `latexml`'s deps are repointed
off git onto those crates (workspace builds from crates.io, `--features cortex`
included). B1 (dep versions) + docs/library groundwork landed. **Remaining
before the `latexml` crates can publish: B3b** (RelaxNG relocation — the one hard
blocker) and **B4** (reserve the 7 sibling names).

---

## 1. Publish order (dependency-first)

`cargo publish` requires every dependency to already be on crates.io, so the
8 crates publish **bottom-up**. Publishing `latexml` drags all 7 siblings onto
crates.io permanently — **their names must be available too** (reserve them).

| # | crate (dir) | version | crates.io deps it needs first |
|---|-------------|---------|-------------------------------|
| 1 | `latexml_core` | 0.4.0 | — |
| 2 | `latexml_codegen` | 0.4.0 | core |
| 3 | `latexml_math_parser` | 0.3.0 | core |
| 4 | `latexml_engine` | 0.5.0 | codegen, core |
| 5 | `latexml_package` | 0.5.0 | codegen, core, engine |
| 6 | `latexml_post` | 0.3.0 | core, engine |
| 7 | `latexml_contrib` | 0.3.0 | core, codegen, engine, package |
| 8 | `latexml` (`latexml_oxide`) | 0.7.4-rc3 | all 7 + `pericortex` |

**Published prerequisites (all on crates.io, deps repointed off git — ✅ DONE):**
`libmarpa-asf-sys` 0.3.0 → `marpa-asf` 0.3.0 (the dginev/marpa fork; consumed by
`latexml_core`/`latexml_math_parser` via `marpa = { package = "marpa-asf" }`),
and `pericortex` 0.2.8 (behind the optional `cortex` feature). crates.io rejects
git deps, so these were hard blockers — see B2.

---

## 2. Blockers & resolutions

### B1 — path deps had no `version` — ✅ DONE
crates.io rejects a `{ path = "…" }` dep with no `version`. All 21 intra-workspace
dep lines now carry `version = "x.y.z"` alongside `path` (local builds still use
the path; the version is only consulted when published). Verified: `cargo
metadata` + `cargo check` clean.

### B2 — forked **git** deps (`marpa`, `pericortex`) — ✅ DONE
crates.io **rejects git deps even when optional**, and the workspace had two:
* `marpa = { git = "https://github.com/dginev/marpa" }` in `latexml_core` +
  `latexml_math_parser` (a core dep — surfaced by the `latexml_core` publish
  dry-run: *"dependency `marpa` does not specify a version"*).
* `pericortex = { git = "…cortex-peripherals…", optional = true }` in
  `latexml_oxide`, behind the off-by-default `cortex` feature.

**Resolution (done 2026-07-16):** both forks published under crates.io-free
names, then the deps repointed:
* dginev/marpa fork → **`libmarpa-asf-sys` 0.3.0** + **`marpa-asf` 0.3.0**
  (upstream `marpa`/`libmarpa-sys` names are taken). Consumers use the cargo
  `package` alias so `marpa::` / `libmarpa_sys::` code is unchanged:
  `marpa = { package = "marpa-asf", version = "0.3.0" }`.
* cortex-peripherals → **`pericortex` 0.2.8**;
  `pericortex = { version = "0.2.8", optional = true }`.

Verified: `cargo check -p latexml` and `cargo check -p latexml --features cortex`
both resolve from crates.io. **Lesson (feature-gated drift):** the `cortex`
feature isn't built by CI/tests by default, so a `Config` literal in
`cortex_worker.rs` that the `--inputencoding` work left missing a field went
unnoticed until this switch built it — worth adding a `--features cortex`
check to CI.

### B3 — workspace `resources/` are not in the package tarballs
`resources/` lives at the **workspace root**, outside every crate dir. `cargo
package` cannot include `../` paths, so the resources never reach the tarball.
Split into two independently-shippable halves; **B3a is DONE, B3b is the
remaining hard blocker.**

#### B3a — `latexml_post` XSLT/CSS/javascript — ✅ DONE (2026-07-16)
`src/xslt.rs` embedded **36 files** via `include_str!("../../resources/…")`; a
missing `include_str!` target is a **compile error**, so this was the one hard
blocker that was cleanly fixable. Relocated `resources/{XSLT,CSS,javascript}` →
`latexml_post/resources/…` and rewrote the 36 embed paths `../../resources/…` →
`../resources/…`. **`include_str!` resolves relative to the source file, not the
process cwd**, so this is robust regardless of where the compiler runs. Verified:
`cargo build -p latexml_post` green; the workspace-root self-containment smoke
(`mv resources aside`) is unaffected because the runtime CSS/JS disk-search is
still cwd-relative with the embedded table as the real source.

#### B3b — RelaxNG schema/model — ❌ HARD BLOCKER (needs its own branch)
**Do NOT naively move `resources/RelaxNG` into a single crate — it breaks the
build.** Proven 2026-07-16: moving it to `latexml_core/resources/RelaxNG` makes
`cargo build -p latexml` fail with `proc-macro derive panicked: Model "LaTeXML"
not found`. Why it's entangled — RelaxNG is a **compile-time input to two crates
at different depths of the dependency graph**:

1. **`latexml_core/build.rs`** walks `../resources/RelaxNG` to emit the runtime
   embed (temp-extracted for `--validate` / `.model` loading).
2. **The `load_model!` `macro_rules!`** (exported from `latexml_engine`, but
   *invoked* in **`latexml_oxide`** — `src/lib.rs`, see `core_interface.rs:359`)
   expands a `#[derive(LoadModel)]` that compiles `LaTeXML.model` **into code at
   compile time**. Its `pathname::find(installation_subdir="resources/RelaxNG")`
   resolves **cwd-relative** (`<cwd>/resources/RelaxNG` or `<cwd>/../…`), and the
   cwd during a `cargo build` is the **workspace root**. So the model derive
   needs RelaxNG reachable from the *root/consumer* crate, while the build.rs
   embed needs it inside `latexml_core`. A single move can't satisfy both, and
   the `load_model!` expansion happens in the top `latexml` crate, so a crates.io
   consumer would need RelaxNG in that crate's tarball too.

**Fix (deferred, own branch):** make the model resolution independent of process
cwd — resolve `LaTeXML.model` relative to `CARGO_MANIFEST_DIR` (or, better, have
`#[derive(LoadModel)]` consume `latexml_core`'s embedded RelaxNG bytes instead of
re-reading the filesystem), then relocate `resources/RelaxNG → latexml_core` and
package it once. Gate with a full `cargo clean && cargo build -p latexml` (the
crate where `load_model!` actually expands — a `-p latexml_core`/`-p
latexml_engine` build is a **false green**, it never triggers the derive) plus
`cargo test --tests` and a release-binary self-containment smoke.

#### `resources/dumps` — intentionally NOT relocated (see B5)
The per-TL-year dumps are git-ignored, generated by the release pipeline
(`release-dumps.yml`, `make_formats.sh`), embedded by `latexml_engine/build.rs`,
and **already excluded from the crates.io tarball by design (B5)** — a
from-source install starts dumpless and reconstructs kernel state. Relocating
them buys nothing for the publish and entangles `release.yml`/`CI.yml` cache
paths, the Dockerfile, `.gitignore`/`.gitattributes`, and `ini_tex.rs`'s
`--init` write path. Left at the workspace root.

Rejected alternatives: a prepublish copy-and-path-rewrite script (fragile — the
`include_str!` paths are compile-time literal), and per-crate symlinks into
`../resources` (Windows-hostile, brittle).

### B4 — sub-crate names must be free on crates.io
`latexml` is registered. Before publishing, confirm/reserve: `latexml_core`,
`latexml_codegen`, `latexml_math_parser`, `latexml_engine`, `latexml_package`,
`latexml_post`, `latexml_contrib`. (crates.io normalizes `_`/`-`, but reserve
the exact `_` names the manifests use.)

### B5 — dumps absent from the crates.io crate (accepted limitation)
The per-TL-year kernel dumps are generated at release time and are large; they
are **not** shipped in the crates.io tarball. A from-source `cargo install
latexml` therefore starts **dumpless** → the engine reconstructs kernel state
from the base pool at startup (slower; the `LATEXML_NODUMP` parity path). This
is acceptable for the source-install path; the prebuilt GitHub-Release binaries
remain the fast, dump-embedded distribution. Document it in the README's
`cargo install` note.

---

## 3. docs.rs & the `documentation` link — ✅ DONE (a)

docs.rs **auto-builds** docs on publish (nothing to upload) with a **nightly**
compiler (so `#![feature(…)]` is fine) in a **no-network** sandbox that sets
`DOCS_RS=1`. Our `build.rs` scripts are docs.rs-safe (git calls no-op without
`.git`; committed `latex_dump.rs`/`plain_dump.rs`; missing resource dirs → empty
compiling embeds). System `libxml2`/`libxslt` come from the docs.rs image;
`kpathsea` 0.3 falls back to its subprocess backend without `libkpathsea`.

`latexml/Cargo.toml` now has:
```toml
documentation = "https://dginev.github.io/latexml-oxide/latexml/"   # self-hosted rustdoc, always works
[package.metadata.docs.rs]
targets = ["x86_64-unknown-linux-gnu"]
```
The `documentation` link points at the **gh-pages rustdoc** (deployed by
`.github/workflows/rustdoc.yml`) so the crate page links to working docs even if
a docs.rs build ever regresses. (gh-pages, like the README badges, only serves
publicly once the repo is public.)

---

## 4. Using `latexml` as a library — ✅ DONE (d)

`latexml::api` is the batteries-included entrypoint — no binary, no manual
`Config`/dispatch wiring:
```rust
let xml  = latexml::api::convert_to_xml(tex)?;   // TeX → LaTeXML XML
let html = latexml::api::convert_to_html(tex)?;  // TeX → HTML5 + Presentation MathML
```
Each call runs on its own 256 MiB-stack worker thread and frees the
thread-local engine (`reset_thread_engine`) before the thread exits. For finer
control (preloads, search paths, `--whatsin`, split, encoding, …) drive
`latexml::converter::Converter` + `latexml::post` directly. Runtime needs a TeX
distribution on `PATH` just like the binary. **Note:** downstream use is only
*functional from crates.io* once **B3** lands (until then the published
`latexml_post` won't build).

**Drift / unification.** `api` and the binary share the actual engine
(`Converter` + `post::run_post_processing`) and the per-format stylesheet choice
(`post::default_stylesheet`, the single source of truth). `PostOptions` has no
`Default`, so a new field is a compile error in `api.rs` — it can't silently
drift. The only high-level logic still living in the binary (not the library) is
`bin/latexml_oxide.rs::real_main`'s CLI orchestration (archive/dir detection,
split, zip packing, telemetry, watchdog, xml-input, whatsout).
**TODO (tracked follow-up):** hoist that CLI-agnostic core into
`latexml::api::run(config)` so the binary becomes a thin CLI shell and there is
exactly one high-level conversion path. Deferred from the 2026-07-16 release
prep as too broad to land safely against the release binary at that time.

---

## 5. Publish checklist

1. [ ] **B3** resource relocation landed + `cargo test --tests` green + release-binary smoke.
2. [ ] **B2** `cortex-peripherals` published; `latexml`'s `pericortex` switched to `version`.
3. [ ] **B4** all 7 sibling crate names reserved.
4. [ ] `cargo publish -p <crate> --dry-run` clean for each, in the order of §1.
5. [ ] Publish bottom-up (§1), waiting for each to index before the next.
6. [ ] Confirm docs.rs built `latexml` (or that the `documentation` link resolves).
7. [ ] README `cargo install latexml` note carries the nightly + build-dep + dumpless caveats (B5).
