# Publishing `latexml` to crates.io

Living checklist for the first crates.io release of the workspace and the
docs.rs / library-consumer story. Complements
[`RELEASING.md`](RELEASING.md) (the GitHub-Release binary flow); this file is
specifically about **`cargo publish` + docs.rs + library use**.

Status (2026-07-16): **not yet publishable** — one hard blocker remains (**B3**,
workspace resources). B1 (dep versions) and the docs/library groundwork are
landed.

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

Plus **`pericortex`** (repo `dginev/cortex-peripherals`) must be published
before #8 — see B2.

---

## 2. Blockers & resolutions

### B1 — path deps had no `version` — ✅ DONE
crates.io rejects a `{ path = "…" }` dep with no `version`. All 21 intra-workspace
dep lines now carry `version = "x.y.z"` alongside `path` (local builds still use
the path; the version is only consulted when published). Verified: `cargo
metadata` + `cargo check` clean.

### B2 — `pericortex` is a **git** dependency — resolve by publishing it
`latexml/Cargo.toml`: `pericortex = { git = "https://github.com/dginev/cortex-peripherals.git", optional = true }`.
crates.io **rejects git deps even when optional**. It's behind the off-by-default
`cortex` feature (the `cortex_worker` binary), but it's still in the manifest.

**Resolution** (chosen): publish `cortex-peripherals` to crates.io first, then
change the line to:
```toml
pericortex = { version = "X.Y.Z", optional = true }
```
(Alternative, if we'd rather not ship the worker: move `cortex_worker` +
`pericortex` into a separate unpublished workspace member so the published
`latexml` crate has no `cortex` feature at all.)

### B3 — workspace `resources/` are not in the package tarballs — ❌ HARD BLOCKER
`resources/` lives at the **workspace root**, outside every crate dir. `cargo
package` cannot include `../` paths, so the resources never reach the tarball.
Two failure modes:

* **`latexml_post` — compile failure (hard).** `src/xslt.rs` embeds **37 files**
  via `include_str!("../../resources/{XSLT,CSS,javascript}/…")`. A missing
  `include_str!` target is a **compile error**, so `cargo publish -p
  latexml_post` fails its verify build. Proven: `cargo package -p latexml_post
  --list` ships only `src/` (48 files) — **zero** resource files.
* **`latexml_core` / `latexml_engine` — empty embeds (soft).** Their `build.rs`
  scripts *walk* `../resources/RelaxNG` and `../resources/dumps`; a missing dir
  yields an empty manifest that still compiles, but the published crate would
  have **no schema / no dumps** → runtime-broken (see B5 for dumps).

**Fix (recommended): physically relocate each resource set into the crate that
embeds it.**
| resource | move to | update |
|----------|---------|--------|
| `resources/XSLT`, `resources/CSS`, `resources/javascript` | `latexml_post/resources/…` | `xslt.rs` paths `../../resources/…` → `../resources/…` |
| `resources/RelaxNG` | `latexml_core/resources/RelaxNG` | `latexml_core/build.rs` walk dir → in-crate |
| `resources/dumps` | `latexml_engine/resources/dumps` | `latexml_engine/build.rs` walk dir; release-dumps write here (see B5) |

Runtime is unaffected: the embedded lookups are keyed by **basename/logical
name**, not the physical include path, so only the compile-time `include_*`
paths change. Blast radius to audit before landing (grep `resources/`):
`.github/workflows/release.yml` (the "move resources/ aside" self-containment
smoke), `release-dumps.yml`, `tools/make_release.sh`, `tools/make_formats.sh`,
and any test fixtures that read `resources/…` from disk. **Do this on its own
branch with full `cargo test --tests` + a release-binary smoke** — it is the
gating item for a *functional* publish, not a quick edit.

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
