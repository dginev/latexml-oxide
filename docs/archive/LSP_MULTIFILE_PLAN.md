# LSP multi-file: project-root + overlay model

> Status: **LANDED 2026-06-04** (all three phases, same day as planned —
> see "Implementation deltas" at the end for where reality improved on
> the plan). Live-verified end-to-end by `tools/lsp_smoke.py multifile`:
> chapter-buffer convert resolves to the detected root, the preview uses
> the UNSAVED buffer over the disk file, diagnostics attribute to
> `sections/ch2.tex`. Companion docs: `docs/SOURCE_PROVENANCE.md`
> (product track), [`LSP_SERVER.md`](LSP_SERVER.md) (the server's
> design/status/known-gaps doc). Both archived 2026-06-05 — landed
> work, moved out of top-level `docs/` to keep focus on parity.

## 1. Problem

The `latexml_oxide --server` v1 model converts the text of the edited
buffer as if it were a complete document. Real papers are
`main.tex` + `\input`/`\include` parts + local `.sty`/`.bib`. Today:

1. **Editing `chapter2.tex` converts chapter2 standalone** — no
   `\documentclass`, no preamble: guaranteed garbage output and
   diagnostics.
2. **Unsaved sibling edits are invisible.** The engine reads included
   files from disk; only the edited buffer's text travels over LSP.
3. **Diagnostics are misattributed.** `Diag` carries no file; an error
   inside an `\input`-ed file is published against the edited uri with
   the *other* file's line numbers.
4. **The warm cache is keyed to one uri** and its same-directory mtimes;
   two alternating documents thrash it, and preamble-time `\input`s in
   subdirectories are not tracked.

## 2. Design principles

* **The project unit is a directory, and the entrypoint is *detected*,
  not assumed.** latexml-oxide already owns an arXiv-grade detector:
  `--whatsin=directory` / `latexml::main_tex::find_main_tex` —
  00README.json (`usage == "toplevel"`), legacy `00README.XXX`
  (`toplevelfile` / `ignore`), then Pack.pm-derived likelihood scoring
  with `\input`-reference vetoes. ar5iv-editor and cortex_worker
  already call it. **The server should resolve every request uri to a
  project root and convert the root** — the same method, one level up.
* **The overlay is an engine seam, not a server hack.**
  `Mouth::create` already has a cached-content branch
  (`MouthOptions.content`) used by the server's named in-memory mouths.
  An overlay table consulted at that seam gives every `\input`/package
  load the editor's unsaved text with no per-call-site changes.
* **Keep the fork model.** A thread-local overlay set in the parent
  before warm-up is inherited by every body child via COW — zero extra
  plumbing in the fork path.
* **TeX Live stays out of scope.** The overlay applies only to paths
  under the project root. kpathsea-resolved system files are never
  overlaid (mirrors the self-contained-binary rule: the host texmf tree
  is the host's business).

## 3. Components

### A. Root resolution (server-side)

`resolve_root(uri) -> ProjectRoot { dir, main: PathBuf }`, resolution
order:

1. **Client override**: `latexml.rootDocument` initialization option /
   per-request param (the VSCode setting already exists for the engine
   path; add the sibling setting).
2. **Magic comment**: `% !TEX root = ../main.tex` in the edited buffer
   (TeX-ecosystem convention, texlab-compatible). New, cheap phase —
   worth adding to `find_main_tex` itself (Phase I.0) so the CLI
   benefits too.
3. **`find_main_tex(dir)`** on the buffer's directory; on failure, walk
   up at most N=2 parents (a chapter often lives in `sections/`).
4. **Fallback**: the buffer itself (today's behavior — correct for
   single-file documents, which remain the common ar5iv-editor case).

Cache `dir -> root` per directory; invalidate on directory mtime or on
`didOpen` of a file that vetoes the previous choice. Diagnostics if
detection is ambiguous: log + carry `"root": <path>` in the convert
response so clients can display what was converted.

### B. Overlay table (engine-side, `latexml_core`)

```rust
// latexml_core/src/overlay.rs (new, ~80 lines)
thread_local! { static OVERLAY: RefCell<FxHashMap<PathBuf, Arc<str>>> ... }
pub fn overlay_set(path, text)   // server: didOpen/didChange
pub fn overlay_remove(path)      // server: didClose (reverts to disk)
pub fn overlay_clear()
pub fn overlay_get(path) -> Option<Arc<str>>   // engine probes
```

Hook sites (the complete funnel, by construction):

1. **`Mouth::create`** (`latexml_core/src/mouth.rs:184`): before the
   `FoodType::File` branch, canonicalize `source` and probe
   `overlay_get` — on hit, take the existing `content` branch
   (locators keep the real path; `--source-map` provenance unchanged).
2. **`find_file` / `find_file_fallback`**
   (`latexml_core/src/binding/content.rs:2285/2407`): treat overlay
   paths as existing, so an unsaved *new* file (never on disk) still
   resolves for `\input`/`\usepackage`. Probe only for candidates under
   the project root — kpathsea results bypass the overlay.
3. *(v2)* BibTeX reads and `\openin`, same probe. Binary assets
   (graphics) are explicitly **disk-only**: clients must save figures;
   document this in the extension.

Thread-locality note: children inherit the table via fork COW. The
in-process fallback path uses the same table on the server thread. The
table must be **cleared and repopulated per conversion** from the
server's buffer map (don't mutate incrementally mid-conversion).

### C. Convert-the-root (server-side)

`latexml/convert` / `didChange` for ANY uri in a project:

1. `resolve_root(uri)`; populate the overlay with **all open buffers**
   of that project (not just the edited one).
2. Read the root's text — from the overlay if the root itself is open,
   else from disk.
3. Run the existing warm-fork pipeline on the **root's** text; preamble
   split and warm cache operate on the root exactly as today.

The response gains `"root"`, and `sources` already provides the
file-tag decoder ring for `data-sourcepos`.

### D. Per-file diagnostics

* `Diag` gains `file: Option<String>` — `parse_line_col` currently
  *discards* the `at <path>; line N` prefix; keep it.
* `publishDiagnostics` groups by file uri (LSP explicitly allows
  publishing for non-open documents). Diagnostics in files outside the
  project (texmf) attach to the root with a prefixed message, as today.
* The custom convert response's `diagnostics` array carries `file` so
  ar5iv-editor can route lint markers to the right buffer in its
  multi-buffer panel.

### E. Warm-cache key, honestly multi-file

Cache validity today: `(uri, preamble-text, same-dir mtimes)`. New key:

* `(root, preamble-text, dep-snapshot)` where the dep snapshot is the
  **set of files actually opened during warm-up** (record paths at the
  `Mouth::create` hook — a warm-up read-log) with, per file:
  `Overlay(version)` or `Disk(mtime)`. This fixes both blind spots:
  subdirectory `\input`s and overlay edits of preamble-time files.
* Editing a preamble-consumed file (`macros.sty`) is then a cache miss
  per keystroke — *correct*, slow, and tolerable given queue
  coalescing; note it in the extension docs (debounce harder for
  `.sty`).

### F. Out of scope here (separate plans)

* **Pool of warm roots** (multi-project LRU; also the Windows worker
  story — no fork there). Single-slot cache remains in this plan.
* **Push-model protocol unification** (one conversion per edit feeding
  both diagnostics and a version-tagged `latexml/preview`
  notification) — the PR #243 review's other structural item. This
  plan is compatible with either pull or push.

## 4. Phasing

| Phase | Scope | Acceptance |
|---|---|---|
| **1. Root detection** | `resolve_root` (override + `% !TEX root` + `find_main_tex` + walk-up), convert-the-root with disk-state siblings, `"root"` in response | Editing `sections/ch2.tex` of a fixture project produces the full-document preview; magic-comment and 00README projects resolve; single-file behavior byte-identical |
| **2. Overlay** | `latexml_core::overlay`, `Mouth::create` + `find_file` probes, server buffer map, warm-up read-log dep snapshot | Unsaved edit in `ch2.tex` appears in the preview without saving; unsaved *new* `mystyle.sty` resolves; editing `macros.sty` invalidates the warm cache; suite green (overlay empty ⇒ zero behavior change) |
| **3. Per-file diagnostics** | `Diag.file`, grouped publish, response routing | Error injected in `ch2.tex` squiggles in `ch2.tex` at the right line, not in `main.tex` |

Phase order is deliberate: each lands alone, each is testable alone,
and Phase 1 already kills failure (1) — the worst user-facing lie.

## 5. Testing

* Fixture project under `latexml_oxide/tests/lsp_project/`:
  `main.tex` + `sections/ch2.tex` + `macros.sty` + a decoy
  `notes/draft.tex` containing `\documentclass` (exercises the
  `\input`-veto scoring).
* Unit: root resolution matrix (override > magic comment > readme >
  scoring > fallback); overlay probe precedence (overlay > disk;
  kpathsea bypass); read-log snapshot equality.
* Integration (extend the live-smoke harness): didChange on `ch2.tex` →
  preview contains the edit; diagnostics attributed per file; cache-hit
  vs `macros.sty`-edit cache-miss observable via timing or a debug
  counter.
* Regression guard: with no overlay and a single-file uri, the v1 paths
  must be byte-identical (the whole plan is additive).

## 6. Risks / cross-references

* **`Loaded`-flag path sensitivity** — package-load dedup is keyed by
  path ([[wisdom_loaded_flag_path_aware]]); overlay must canonicalize
  the same way `find_file` does or a package can load twice.
* **`find_file` caching / env hot path** ([[wisdom_env_var_hot_path_race]],
  [[wisdom_findfile_fallback]]) — probe placement must not add a
  per-token cost; the overlay probe runs only on file-open, which is
  cold.
* **Security posture unchanged**: the overlay narrows to the project
  root; `\input` of absolute paths outside the root behaves exactly as
  the CLI does today (local-tool posture, `docs/SAFETY.md`). Any
  multi-tenant web deployment still requires jailing — tracked in
  `SOURCE_PROVENANCE.md`.
* **Perl parity**: none of this touches conversion semantics; it is
  I/O-source selection. No `OXIDIZED_DESIGN` divergence entry needed
  unless the magic-comment phase is added to `find_main_tex` (then it
  is a documented Rust-side extension — Perl's Pack.pm has no
  `% !TEX root` support).

## 7. Implementation deltas (landed 2026-06-04)

Where the landed implementation deliberately differs from §3:

* **The overlay rides the engine's EXISTING `{file}_contents` channel**
  (the Perl-faithful `\begin{filecontents}` cache) instead of a new
  `Mouth::create` hook. `find_file_aux` (existence), the definitions
  loader, the `\input` open path, and the raw cls/sty dep-scan already
  consult it (`binding/content.rs:1252/1343/1820/2494`) — so the engine
  diff is ZERO lines, and fork children inherit the values via state
  COW. Buffers register under absolute / project-relative / basename
  keys, each ± `.tex` (the engine probes both literal and resolved
  names); ambiguous basenames get no bare key (disk wins).
* **The read-log is a dedicated `Mouth::create`-level log** —
  `state::opened_sources` / `record_opened_source()`, ~15 engine lines.
  (The first landing reused `source_table_snapshot()` on the belief the
  locator table "already records every named source the engine opened".
  That was WRONG twice over: `source_tag` populates at
  *document-construction* time — which happens in the forked body child,
  AFTER the parent takes the snapshot — and it filters to user sources,
  excluding `.sty`/`.cls` entirely. The snapshot was therefore always
  empty, and an unsaved edit of a preamble-consumed file never
  invalidated the warm cache: a live-confirmed **stale-preamble bug**,
  caught in the 2026-06-05 performance review and fixed same-day.
  Guarded by `tools/lsp_smoke.py staledep`.) The snapshot EXCLUDES the
  root itself — its preamble half is keyed by string equality, and
  pinning the root buffer's version would re-warm on every body
  keystroke. The same-dir scan is KEPT alongside but narrowed to
  file-SET equality (it catches files *appearing/disappearing*, which
  can flip `find_file` resolution; comparing mtimes there forced a full
  re-warm on every save of a same-dir *body* file).
* **Root detection adds a self-containment fast path**: a buffer with an
  un-commented `\documentclass`/`\documentstyle` is its own root with
  zero directory scanning (v1-identical single-file behavior), and a
  detected candidate ≠ buffer is only trusted when its text *references*
  the buffer's stem on an un-commented `\input`/`\include` line — so a
  directory of unrelated documents can never hijack a fragment.
  `% !TEX root` is handled server-side only; `find_main_tex` itself is
  untouched (Perl Pack.pm parity).
* **Preemption/coalescing went project-scoped**: any conversion trigger
  for a file of the same project supersedes the in-flight compile of
  that project (`same_project`, lexical containment in the root's dir).
* **Per-file publish includes stale-clear**: files diagnosed last round
  but clean now get an explicit empty `publishDiagnostics`
  (`Server.last_published`), so squiggles don't linger.
* Diagnostics parsing was upgraded to the record format (severity line +
  tab-indented `at <source>; line N col M - …` continuation) — the old
  line parser never saw locators at all. Same record shape
  ar5iv-editor's `parse_diagnostics` consumes.
