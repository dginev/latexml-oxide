# Persistent server (`latexml_oxide --server`) — design & status

> **Living doc** for the editor/preview server (the `--server` LSP):
> architecture, review records, and the known-gaps worklist. Moved out
> of `SYNC_STATUS.md` 2026-06-05 — the server is *beyond-Perl* product
> work (issues #47/#92, `docs/SOURCE_PROVENANCE.md`), not Perl parity,
> so it does not belong in the parity log. Archived 2026-06-05 (with
> its companion plan) to keep top-level `docs/` on the parity mission —
> the content describes the SHIPPED `--server` code, it is not stale.
> Companions: [`LSP_MULTIFILE_PLAN.md`](LSP_MULTIFILE_PLAN.md) (the
> landed multi-file root/overlay design),
> [`SOURCE_PROVENANCE.md`](../SOURCE_PROVENANCE.md) (product track).
> Live smoke: `tools/lsp_smoke.py <binary>` (basic/preempt/multifile/
> staledep).

A JSON-RPC-over-stdio server (LSP framing) in `latexml_oxide/src/lsp_server/`
for editor/preview integration. Speaks a subset of LSP (`initialize`,
`didOpen`/`didChange` → `publishDiagnostics`, `shutdown`, `exit`) plus a custom
`latexml/convert` request returning `{html, log, diagnostics, sources, status,
statusCode}` — the shape the `ar5iv-editor` client consumes.

## Architecture
* **Warm-preamble + fork-body.** The preamble (through `\begin{document}`) is
  digested once in the parent and cached; each body conversion `fork()`s a
  child that inherits the warm post-preamble state via copy-on-write, digests
  only the body, builds + post-processes the DOM, and writes the result over a
  pipe before exiting. The child is throwaway, so a body conversion never
  pollutes the cache and a panicking/looping body cannot kill the server. The
  child uses a bare `latexml_core::Core` over the inherited state — **never**
  `Converter::from_config`/`Core::new`, which would `set_state` and wipe the
  inherited definitions.
* **Single-threaded `poll(2)` loop.** While a child runs the parent multiplexes
  `{stdin, child-pipe}`; a newer same-document `latexml/convert` `SIGKILL`s the
  in-flight child (a pid still owned/un-reaped here → no PID-recycle race) and
  supersedes it. Single-threaded is also what makes the `fork()` safe (no other
  thread can hold the allocator lock at fork time).
* **Unified pipeline + cache coherence.** `latexml/convert` and
  `didOpen`/`didChange` both go through one warm-fork path, so the cache stays
  coherent (an in-process fallback always invalidates it). Preamble (warmup)
  log is captured and merged with the body log so preamble diagnostics survive
  across cache hits.
* **Source-map.** Preamble and body are opened as *named* in-memory mouths
  (the document's `.tex` path), not anonymous `literal:` — required so
  `--source-map` stamps locators (it only stamps `.tex/.ltx/.bbl/.bib`). Body
  line numbers are made file-relative by prepending the preamble's newline
  count. Output is post-processed to HTML5 (`run_post_processing`,
  `nodefaultresources=true` so no files are written), yielding the
  `data-sourcepos` (dash) attributes + `sources` decoder ring the client uses.

## Verified (2026-06-01)
* End-to-end: warm convert, cache-hit convert, `didChange`→diagnostics, and
  same-document preemption all work; output is full HTML5 with MathML,
  `data-sourcepos`, and `sources:['<file>.tex']`; fork-path locators match the
  full-document baseline; no cwd pollution.
* Unit tests in `lsp_server::tests` (JSON round-trip + control-char escaping,
  diagnostic parsing/0-basing, basename, cancelled-shape); daemon-frame
  round-trip in `tests/00_unit_state.rs`.

## Code review 2026-06-04 (PR #243) — fixes landed
A line-by-line audit of the server (assumptions + bug hunt) landed fixes for:
unanswered `latexml/convert` requests on missing params (-32602 now);
didChange-flurry snowball (same-uri didChange/didOpen now PREEMPTS the
in-flight child, and the pending queue COALESCES — only the newest trigger
per doc runs); orphaned `setsid` graphics converters when a body child is
killed mid-post (`PR_SET_PDEATHSIG` in `run_with_timeout` `pre_exec`,
SIGTERM-grace-SIGKILL preemption); mojibake percent-decoding of non-ASCII
URIs (byte-level decode); `--max-memory` being a silent no-op outside
`--server` (standalone CLI now `Watchdog::with_limits`); preamble split at a
COMMENTED-OUT `\begin{document}` (comment-aware `find_begin_document`);
partial-frame stdin deadlock window in `wait_for_child` (poll-driven fill,
never blocks mid-frame); child stdout corrupting the protocol stream
(`dup2(/dev/null, 1)` insurance); fork-safety invariant now `debug_assert`ed
(single thread at fork, `/proc/self/task`); signal deaths reported as
`128+signo` (OS OOM-kill maps to the same 137 as the Watchdog ceiling);
`push_daemon_frame` comment corrected (Perl's `daemon_copy` deep-copy is NOT
ported — required before any thread-reusing daemon mode relies on it).

## Performance review 2026-06-05 (PR #243) — stale-preamble bug found & fixed
A per-edit-location cost review ("what does one keystroke cost, where?")
found the multi-file landing's dep snapshot was **vacuously empty**: it
reused the locator `source_table`, which is populated at
*document-construction* time (in the forked body child, AFTER the parent
snapshots) and filters out `.sty`/`.cls`. Consequence (live-confirmed):
an **unsaved edit of a preamble-consumed file** (`\input{defs}` in the
preamble, a local `.sty`) never invalidated the warm cache — the preview
silently kept the stale macro bodies until the file was saved to disk.
Fixed by a dedicated `Mouth::create`-level read-log
(`state::opened_sources`, recorded for named file/cached-content mouths,
`\openin` included; ~15 engine lines), with the ROOT excluded from the
snapshot (its preamble half is keyed by string equality; pinning its
buffer version would kill the cache for every body keystroke). The
same-dir dependency scan was simultaneously narrowed from mtime-equality
to file-SET equality: mtime comparison forced a **full preamble re-warm
on every save of a same-dir body file** (`.tex` chapters, `.bib`) even
though body files are re-read each conversion anyway; content staleness
of warm-up-opened files is precisely the read-log's job. Guards:
`lsp_server::overlay` unit tests + `tools/lsp_smoke.py staledep`
(overlay edit of a preamble dep re-warms; body-file save converts warm).
Verified per-edit-location costs after the fix: body edits (footnotes,
captions, `thebibliography`, body frontmatter) = fork + body redigest
(~0.05s on the probe document vs ~0.7s cold); preamble-region or
preamble-dep edits = full re-warm (correct, by design); same-dir body
saves = warm (was: re-warm).

## Known gaps / follow-ups
* **In-process fallback is unguarded against native hangs.** The fallback
  (no `\begin{document}`, fork failure) runs on the server thread with only
  the cooperative deadline — a Marpa/libxslt tight loop wedges the whole
  server (a Watchdog would kill the server itself). Mitigation candidate:
  fork the fallback too.
* ~~Dependency mtime scan is non-recursive~~ FIXED 2026-06-04 by the
  warm-up read-log snapshot (`overlay::warmup_dep_snapshot`, pinned
  Overlay(version)/Disk(mtime) per opened source); the same-dir scan is
  kept alongside to catch files *appearing/disappearing*.
* ~~No multi-file project model~~ **LANDED 2026-06-04**: project-root
  detection (override > `% !TEX root` > `find_main_tex` with
  reference-guarded walk-up), unsaved-buffer overlay via the engine's
  `{file}_contents` channel (zero engine diff, COW-inherited by forks),
  read-log dep snapshot keying the warm cache, per-file diagnostics
  (record-format log parser + attribution + stale-clear publishes).
  Design + implementation deltas:
  [`LSP_MULTIFILE_PLAN.md`](LSP_MULTIFILE_PLAN.md). Live-verified:
  `tools/lsp_smoke.py` (basic/preempt/multifile/staledep, 19/19).
* **Warm-up is synchronous and unpreemptible** (2026-06-05 review). The
  preamble digest runs on the event-loop thread BEFORE the fork; stdin is
  only drained while waiting on the body child. Typing in the PREAMBLE
  region (e.g. `\title`/`\author`/`\newcommand` in article-style classes,
  or an open local `.sty`) therefore serializes one full re-warm per
  coalesced burst, during which the server is deaf (even `exit` queues).
  Preemption only saves the body phase. Mitigation candidates: client-side
  debounce keyed on edit region (preamble vs body), or a zygote chain — a
  second checkpoint process after the package block so volatile preamble
  tails (`\title`/`\author`/macros) re-digest in milliseconds. (revtex-style
  documents put frontmatter AFTER `\begin{document}`, so those edits are
  already on the fast body path.)
* **Graphics conversion output lands in the server's CWD** (2026-06-05
  review). `post_process_html` passes `destination: None`, and the Graphics
  phase defaults `dest_dir` to `"."` — a figure-bearing paper hardlinks/
  copies converted images into wherever the server was started, per
  conversion. (Reconversion itself is cheap: the content-hash XDG
  `graphics_cache` serves repeats without subprocesses.) Fix candidate: a
  per-project temp dir + absolute `imagesrc`, or data-URI inlining for the
  preview.
* **`.bib` files bypass the unsaved-buffer overlay** (plan §3B item 3, v2):
  `pre_bibtex.rs` reads bibliographies via `std::fs::read_to_string` — no
  `{file}_contents` consult, so unsaved `.bib` edits are invisible until
  saved. Body-time read, so a *saved* `.bib` is picked up warm (and no
  longer triggers a spurious re-warm since the file-set narrowing).
* Body content following `\begin{document}` on the same line gets correct
  line numbers but wrong *columns* for that first line (the body mouth
  starts at column 1).
* Interner growth: long-lived sessions keep interning new symbols (the daemon
  uses `reset_thread_state`, not `reset_thread_engine`); a periodic full reset
  is a future hardening.
* Earlier prototype benchmark figures (cold-vs-warm speedups) are **not**
  re-verified post-rewrite and were measured before post-processing was wired
  in; treat as stale until re-measured.

