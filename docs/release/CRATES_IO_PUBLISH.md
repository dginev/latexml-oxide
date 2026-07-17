# Publishing `latexml` to crates.io

Living checklist for the first crates.io release of the workspace and the
docs.rs / library-consumer story. Complements
[`RELEASING.md`](RELEASING.md) (the GitHub-Release binary flow); this file is
specifically about **`cargo publish` + docs.rs + library use**.

Status (2026-07-17): **all code blockers are cleared.** B1 (dep versions), B2
(forked git deps → published `marpa-asf` / `libmarpa-asf-sys` / `pericortex`),
B3a (XSLT/CSS/js → `latexml_post`) and **B3b (RelaxNG → `latexml_core`)** are
done; B5 (dumpless source install) is an accepted, documented limitation. The
only remaining steps are **account actions, not code**: make the repo public (every
crate's `repository`/`homepage` 404s until then), then `cargo publish --workspace`
(§5). B4 is verified: all 7 sibling names are free.

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

### B3 — workspace `resources/` are not in the package tarballs — ✅ DONE
`resources/` lived at the **workspace root**, outside every crate dir. `cargo
package` cannot include `../` paths, so the resources never reached the tarball.
Split into two independently-shippable halves — **B3a DONE (2026-07-16), B3b DONE
(2026-07-17)**. The same `../` rule bit the README too, separately: see B6.

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

#### B3b — RelaxNG schema/model — ✅ DONE (2026-07-17)
**Do NOT naively move `resources/RelaxNG` into a single crate — that breaks the
build,** which is what made this look hard. Proven 2026-07-16: a bare move to
`latexml_core/resources/RelaxNG` makes `cargo build -p latexml` fail with
`proc-macro derive panicked: Model "LaTeXML" not found`. RelaxNG is a
**compile-time input at two different depths of the dependency graph**:

1. **`latexml_core/build.rs`** walks `resources/RelaxNG` to emit the `include_str!`
   embed table (`common::relaxng::embedded::{FILES, lookup}`).
2. **The `load_model!` `macro_rules!`** (exported from `latexml_engine`, but
   *invoked* in **`latexml_oxide`** — `src/lib.rs`) expands a
   `#[derive(LoadModel)]` that compiles `LaTeXML.model` **into code at compile
   time**. It used `pathname::find(installation_subdir="resources/RelaxNG")`, which
   resolves **cwd-relative** (`<cwd>/resources/RelaxNG`, else one level up) — fine
   inside our checkout, fatal from a crates.io install where no `resources/` sits
   beside the cwd.

**What actually resolved it:** the two halves were never really in conflict —
`latexml_codegen` (the proc-macro crate) **already links `latexml_core`**, whose
build.rs **already** embeds the tree. So the derive can read the *embedded bytes*
instead of the filesystem, and the disk copy only needs to exist in one place:

1. `latexml_codegen/src/modelable.rs` → `relaxng::embedded::lookup("<name>.model")`;
   no `pathname::find`, no `File::open`. **Cwd-independent by construction.**
2. `latexml_core/build.rs` → resolve under `CARGO_MANIFEST_DIR` (drop `.parent()`),
   plus an **assert that the tree is non-empty**: `collect_files` swallows a missing
   dir, so a tree that failed to ship would compile clean, embed nothing, and
   resurface much later as a confusing derive panic pointing at the consumer.
3. `git mv resources/RelaxNG latexml_core/resources/RelaxNG` (108 files).

The runtime already preferred the embedded table (`model.rs` L437, `scan.rs` L97),
so nothing else moved. `latexml_core`'s default (no `include`/`exclude`) packaging
picks the tree up automatically.

**Verified 2026-07-17:** `cargo clean && cargo build -p latexml` green (the crate
where the derive expands — a `-p latexml_core`/`-p latexml_engine` build is a
**false green**, it never triggers it); conversion smoke emits a model-driven
`<para xml:id="p1">`; `cargo package --list -p latexml_core` ships all **108**
files incl. `resources/RelaxNG/LaTeXML.model`, while `-p latexml` now needs **0**;
self-containment smoke passes with the whole `resources/` tree moved aside.

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

### B4 — sub-crate names must be free on crates.io — ✅ VERIFIED FREE (2026-07-17)
`latexml` is registered to **dginev** (currently 0.0.2 — the placeholder the real
release overwrites). All 7 siblings checked against the crates.io API and **free**:
`latexml_core`, `latexml_codegen`, `latexml_math_parser`, `latexml_engine`,
`latexml_package`, `latexml_post`, `latexml_contrib`. (crates.io normalizes
`_`/`-`, but the exact `_` names the manifests use are what get reserved.)
Nothing to pre-reserve — the publish in §5 claims all 7 in one go. Re-check right
before publishing if any time has passed; a name can be taken by anyone.

### B6 — `readme` pointed outside the crate dir — ✅ DONE (2026-07-17)
Found by the first `cargo publish --workspace --dry-run`: `latexml_oxide` declared
`readme = "README.md"` while README.md lives at the **workspace root**, so
`cargo publish -p latexml` died with *"readme `README.md` does not appear to
exist"*. Same root cause as B3 — `cargo package` cannot follow `../`. The other 7
crates packaged fine; only the top crate declares a readme.

**Fix:** `latexml_oxide/README.md` is a **symlink** to `../README.md`. Cargo
dereferences it and packages the real content (verified: 12,601 bytes in the
package, byte-identical to the root file, stored as a regular file). A *copy* was
rejected — the root README is the single source of truth that GitHub serves, and a
second one is guaranteed drift. The manifest carries a comment saying so, because
the obvious "fix" for a symlink is to replace it with a copy.

*Caveat:* a Windows checkout without symlink support materialises this as a text
file containing `../README.md`, which would publish a 12-byte README. We publish
from Linux/CI, so this is noted rather than guarded.

### B5 — dumps absent from the crates.io crate (accepted; user-fixable in one step)
The per-TL-year kernel dumps are generated at release time and are large; they
are **not** shipped in the crates.io tarball. A from-source `cargo install
latexml` therefore starts **dumpless** → the engine reconstructs kernel state
from the base pool at startup (slower; the `LATEXML_NODUMP` parity path). The
prebuilt GitHub-Release binaries remain the fast, dump-embedded distribution.

**But the user can generate them once, and no code was needed — this already
works** (verified 2026-07-17 against the maxperf binary in a simulated install
layout, dev tree hidden). `--init` writes to `./resources/dumps/`
(`latexml_oxide/src/ini_tex.rs` L240) and the loader's chain step 3 is
`<exe_dir>/../resources/dumps` — which, for `cargo install`'s
`~/.cargo/bin/latexml_oxide`, is `~/.cargo/resources/dumps`. So:

```bash
cd ~/.cargo && latexml_oxide --init=plain.tex && latexml_oxide --init=latex.ltx
```

writes `~/.cargo/resources/dumps/{plain,latex}.YYYY.dump.txt` and every later run
finds them. (`LATEXML_DUMP_DIR` overrides the location; the engine's own
no-dump error already says *"run `latexml_oxide --init=latex.ltx` to generate"*.)
Documented in the README's crates.io section.

**Why not generate dumps from `build.rs` at `cargo install` time?** Asked
2026-07-17; it does not work, for four independent reasons:
1. **Ordering cycle.** A dump is produced by *running the fully linked binary*
   (`tools/make_formats.sh` = `cargo build --bin latexml_oxide`, *then* run it with
   `--init`). A build script runs *before* its own crate is compiled, and the
   binary is the last node in the graph — `latexml_engine/build.rs` cannot invoke
   the thing it is helping to build.
2. **The escape is worse.** `latexml_oxide/build.rs` + `[build-dependencies]
   latexml_engine` would compile the engine a *second* time for the host.
   `latexml_engine` is precisely the crate that was split out to cut compile-time
   RAM — doubling it makes `cargo install` far heavier. And the dumps would then
   have to flow back into `latexml_engine`'s runtime loader, which embeds from its
   *own* build.rs: the wiring inverts.
3. **It would require TeX Live at build time.** `--init` resolves `plain.tex` /
   `latex.ltx` through kpathsea, and `make_formats.sh` exits 3 when it cannot even
   detect a TL year. Today a missing TeX means *slower startup*; via build.rs it
   would mean a hard `cargo install` **failure** — strictly worse for a source
   install, and docs.rs (no network, no TeX) builds the crate too.
4. **Dumps are per-TL-year.** One baked at install time goes stale the moment the
   user upgrades TeX Live. The release binaries dodge this by embedding a 5-year
   window; a single install-time dump cannot, whereas the on-demand `--init` above
   is simply re-run.

The nicer version — **auto-generate on first run** into a persistent per-user dir,
slotted into the existing fallback chain — is a real feature and a reasonable
follow-up. It changes first-run behaviour (a surprise multi-second first
conversion plus a disk write), so it wants its own design + test cycle rather than
a release-day patch.

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

Code blockers: **all cleared** (B1, B2, B3a, B3b, B6 done; B4 verified free; B5 an
accepted limitation). Verified 2026-07-17: `cargo test --tests` **1581/0/0**,
`cargo clippy --workspace --all-targets -- -D warnings` clean, self-containment
smoke green for both XML and HTML with the whole `resources/` tree moved aside.

**One command does it** — `cargo publish --workspace` (cargo 1.99) topo-sorts and
publishes bottom-up, and handles the not-yet-published intra-workspace deps that
make a per-crate `cargo publish -p <sibling>` fail ("no matching package named
`latexml_codegen` found"). Rehearsed clean end-to-end:

```bash
cargo publish --workspace --dry-run     # all 8 package + verify; RC=0
cargo publish --workspace               # the real thing
```

The order it picks matches §1 exactly: core → codegen → math_parser → engine →
package → post → contrib → latexml.

0. [ ] **Publish a STABLE version, not an `-rc`.** crates.io currently holds only
       the placeholders `latexml` **0.0.1 / 0.0.2** (`max_stable_version: 0.0.2`).
       `cargo install` and `latexml = "0.7"` both **ignore pre-releases**, so
       publishing `0.7.4-rc4` would leave `cargo install latexml` resolving to the
       **0.0.2 placeholder** — while the README shipped inside that very crate tells
       people to run exactly that command. Publishing an rc is worse than not
       publishing. Tag `0.7.4-rc4` for the GitHub **draft prerelease** (what
       `release.yml` does for `-*` tags: cross-OS testing, not a public release) and
       do the crates.io publish on the **stable `0.7.4`** tag.
1. [ ] **Repo is public.** It was private as of 2026-07-17 (going public that day).
       `repository` / `homepage` on all 8 crates and every README badge point at
       `github.com/dginev/latexml-oxide` — all 404 while it is private, and a
       crates.io release is **irreversible** (yankable, never deletable; the version
       number is burned). Flip visibility *before* publishing. The gh-pages
       `documentation` link already serves (200).
2. [ ] Re-check **B4** name availability (§B4) — free is not a permanent state.
3. [ ] `cargo publish --workspace --dry-run` clean.
4. [ ] `cargo publish --workspace`.
5. [ ] Confirm docs.rs built `latexml` (the `documentation` link is the fallback).
6. [ ] README `cargo install latexml` note carries the nightly + build-dep + dumpless caveats (B5).
