# Publishing `latexml` to crates.io

Living checklist for the first crates.io release of the workspace and the
docs.rs / library-consumer story. Complements
[`RELEASING.md`](RELEASING.md) (the GitHub-Release binary flow); this file is
specifically about **`cargo publish` + docs.rs + library use**.

Status (2026-07-17): ✅ **PUBLISHED.** All 8 crates are live on crates.io —
`latexml` **0.7.4** (`max_stable_version` 0.7.4, published 2026-07-17T14:44Z),
`latexml_core` 0.4.0 at 14:20Z, and the six siblings; the repo is **public**.
B1 (dep versions), B2 (forked git deps → published `marpa-asf` /
`libmarpa-asf-sys` / `pericortex`), B3a (XSLT/CSS/js → `latexml_post`) and
**B3b (RelaxNG → `latexml_core`)** are done; B4 verified; B5 (dumpless source
install) is an accepted, documented limitation.

**Subsequent releases are just a version bump + `cargo publish --workspace`.**
The checklist in §5 is retained as the procedure and, more importantly, for the
durable traps B7/B8 — read those before the next publish. Note the working tree
is already at `0.7.5-rc1`, and rule 0 below still binds: **never publish an
`-rc` to crates.io** (it does not become `max_stable_version`, so
`cargo install latexml` would silently keep resolving to 0.7.4).

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
| 8 | `latexml` (`latexml_oxide`) | published 0.7.4 | all 7 + `pericortex` |

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
RelaxNG is a compile-time input at two depths: `latexml_core/build.rs` walks it to
emit the `include_str!` embed table (`common::relaxng::embedded`), and
`load_model!`'s `#[derive(LoadModel)]` (exported from `latexml_engine`, *invoked* in
`latexml_oxide`) compiles `LaTeXML.model` into code. The derive used
`pathname::find`, which resolves **cwd-relative** — fine in our checkout, fatal from
a crates.io install. Hence a bare move breaks the build: `Model "LaTeXML" not found`.

Fixed by removing the conflict, not splitting the tree: `latexml_codegen` already
links `latexml_core`, whose build.rs already embeds it, so the derive now reads the
embedded bytes (`modelable.rs` → `embedded::lookup`) and is cwd-independent. Then
build.rs resolves under `CARGO_MANIFEST_DIR` (plus an assert the tree is non-empty)
and `git mv resources/RelaxNG latexml_core/` (108 files). The runtime already
preferred the embed table (`model.rs` L437, `scan.rs` L97); default packaging picks
the tree up.

**Gate: `cargo build -p latexml`** — the crate where the derive expands. A
`-p latexml_core` / `-p latexml_engine` build is a **false green**.
**Verified:** clean build green; conversion emits a model-driven `<para xml:id>`;
`cargo package --list -p latexml_core` ships 108 files incl. `LaTeXML.model`, and
`-p latexml` needs 0; self-containment smoke green with `resources/` moved aside.

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

**Fix:** `latexml_oxide/README.md` is a **symlink** to `../README.md` — not a copy,
which would be guaranteed drift from the root README GitHub serves. Cargo
dereferences it and packages the real content (verified: 12,601 bytes, byte-identical,
stored as a regular file). The manifest says so, since the obvious "fix" is to replace
a symlink with a copy.

*Caveat:* a Windows checkout without symlink support materialises it as a 12-byte text
file containing `../README.md`. We publish from Linux/CI — noted, not guarded.

### B7 — `latexml` exceeded the 10 MiB upload cap — ✅ DONE (2026-07-17)
The first real publish of the top crate was rejected:

```
413 Payload Too Large: max upload size is: 10485760
```

The tarball was **13.6 MiB** against a hard **10 MiB** cap. `latexml_oxide/tests` —
the regression corpus — was **1342 of the crate's 1372** packageable files and ~19.5
of its ~20 MiB. The other 7 crates are far smaller and were unaffected.

**Fix:** `exclude = ["tests/"]` in `latexml_oxide/Cargo.toml` → **30 files, 173 KiB
compressed**. Safe because nothing in `src/`/`bin/` reads `tests/` at build time,
there are no explicit `[[test]]` targets (the 50 harnesses are auto-discovered, so
the packaged crate simply has none — cargo prints a benign `ignoring test …` per
harness), and `build.rs` only wires git hooks and no-ops without a `.git`. Do **not**
exclude the fixture subdirs while keeping `tests/*.rs`: the `testable` proc-macro
scans the corpus at compile time, so a half-excluded tree fails `cargo test` on the
packaged crate.

> **Two traps, and `--dry-run` catches neither.**
>
> 1. **The size cap is only enforced on upload.** `cargo publish --workspace
>    --dry-run` packages and verifies, then aborts *before* uploading — so it went
>    fully green on a 13.6 MiB tarball that could never have been accepted. A green
>    dry-run says nothing about size.
> 2. **The CDN masks the error.** Fastly/Varnish in front of crates.io reports the
>    oversized write as a bare `503 backend write error` with an empty body, not a
>    413. If a publish 503s, suspect size first and check
>    `cargo package --list | wc -l` before assuming an outage.

### B8 — new-crate rate limit aborts `--workspace` mid-run — ⚠️ KNOWN (2026-07-17)
crates.io allows a **burst of 5 new crates**, then refills roughly one per 10
minutes. This workspace publishes **7 new crates at once**, so a first-time
`cargo publish --workspace` *cannot* finish in one pass. Observed: five landed in 7
seconds (14:20:29–14:20:36), the sixth got

```
429: You have published too many new crates in a short period of time.
     Please try again after Fri, 17 Jul 2026 14:32:10 GMT
```

and **cargo aborted rather than waiting**, so the remaining crates were never
attempted. Recovery is just `cargo publish -p <crate>` per remaining crate as tokens
refill; the 429 body states the exact retry time. Note the ordering constraint —
`latexml` depends on `latexml_contrib`, so it cannot go first.

Only bites when publishing **new** names (a version bump of an existing crate uses a
far looser limit), so this is a one-time cost now that all 8 names exist. Recorded
because a green dry-run makes it look like one `--workspace` invocation suffices.

### B5 — dumps absent from the crates.io crate (accepted; user-fixable in one step)
The per-TL-year kernel dumps are generated at release time and are large; they
are **not** shipped in the crates.io tarball. A from-source `cargo install
latexml` therefore starts **dumpless** → the engine reconstructs kernel state
from the base pool at startup (slower; the `LATEXML_NODUMP` parity path). The
prebuilt GitHub-Release binaries remain the fast, dump-embedded distribution.

**The user can generate them once — this already works, no code needed** (verified
2026-07-17 against the maxperf binary in a simulated install layout, dev tree
hidden). `--init` writes to `./resources/dumps/` (`ini_tex.rs` L240) and the loader's
step 3 is `<exe_dir>/../resources/dumps` = `~/.cargo/resources/dumps` for a
`cargo install`. So:

```bash
cd ~/.cargo && latexml_oxide --init=plain.tex && latexml_oxide --init=latex.ltx
```

writes `~/.cargo/resources/dumps/{plain,latex}.YYYY.dump.txt` and every later run
finds them. (`LATEXML_DUMP_DIR` overrides the location; the engine's own
no-dump error already says *"run `latexml_oxide --init=latex.ltx` to generate"*.)
Documented in the README's crates.io section.

**Why not generate dumps from `build.rs` at `cargo install` time?** (Asked
2026-07-17.) A dump is produced by *running the fully linked binary* (`make_formats.sh`
builds `latexml_oxide`, *then* runs `--init`), but a build script runs before its own
crate compiles and the binary is the graph's last node — an ordering cycle. The escape
(`latexml_oxide/build.rs` + `[build-dependencies] latexml_engine`) compiles the engine
a second time for the host, doubling exactly the crate split out to cut compile RAM,
and inverts the embed wiring. It would also demand TeX Live at build time, turning
today's *slower startup* into a hard `cargo install` **failure** (docs.rs has no TeX);
and a dump baked at install goes stale on the next TL upgrade, whereas `--init` is
simply re-run.

**Follow-up:** auto-generate on first run into a persistent per-user dir, slotted
into the existing fallback chain. Changes first-run behaviour (a surprise
multi-second conversion + a disk write), so it wants its own design + test cycle.

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

**One command *starts* it** — `cargo publish --workspace` (cargo 1.99) topo-sorts and
publishes bottom-up, and handles the not-yet-published intra-workspace deps that make
a per-crate `cargo publish -p <sibling>` fail ("no matching package named
`latexml_codegen` found"). The order it picks matches §1 exactly: core → codegen →
math_parser → engine → package → post → contrib → latexml.

```bash
cargo publish --workspace --dry-run     # all 8 package + verify; RC=0
cargo publish --workspace               # the real thing
```

> **A green `--dry-run` does not mean the publish will succeed.** It aborts before
> uploading, so it exercises neither the **10 MiB size cap** (B7) nor the **new-crate
> rate limit** (B8). On 2026-07-17 the dry-run passed all 8 while the real publish
> stopped at 5/8 on a 429, and the top crate could not have been accepted at any
> point — it was 13.6 MiB against a 10 MiB cap. Both are now fixed/known, but treat
> a green dry-run as "the *contents* are sound", not "this will go through".

Expect to finish the first-ever publish with per-crate `cargo publish -p <crate>`
calls as rate-limit tokens refill (B8). That is a one-time cost: all 8 names now
exist, so subsequent releases are version bumps under a much looser limit.

0. [x] **Publish a STABLE version, not an `-rc`.** *(Honoured at 0.7.4; the rule
       stands for every future release.)* When this was written crates.io held only
       the placeholders `latexml` **0.0.1 / 0.0.2** (`max_stable_version: 0.0.2`);
       today it is 0.7.4.
       `cargo install` and `latexml = "0.7"` both **ignore pre-releases**, so
       publishing `0.7.4-rc4` would leave `cargo install latexml` resolving to the
       **0.0.2 placeholder** — while the README shipped inside that very crate tells
       people to run exactly that command. Publishing an rc is worse than not
       publishing. Tag `0.7.4-rc4` for the GitHub **draft prerelease** (what
       `release.yml` does for `-*` tags: cross-OS testing, not a public release) and
       do the crates.io publish on the **stable `0.7.4`** tag.
1. [x] **Repo is public.** Done 2026-07-17.
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
