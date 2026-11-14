# Critical review — status/memory/diagnostics/persistence campaign (2026-08-03)

Multi-lens review of the work landed 2026-08-01..03 (#480, #482, #484-#487):
three delegated deep-reads (diagnostics semantics, ObjectDB persistence,
hot-path cost) + an approach-level pass. Findings ranked; "fix-on-#487" items
block that PR, "follow-up" items get their own branch.

## Confirmed sound
- Hot-path cost of the single-vehicle rework: ~0.1-0.2 s of 5065 s (0.003%).
  Nothing ungated emits in the digestion loop (telemetry_due is power-of-two
  gated). The dominant per-record cost is the PRE-EXISTING stderr flush()
  (logger.rs), 50-100x the new overhead — that is the lever if render logging
  ever needs to be cheaper.
- Target renames are consumer-safe: no tool/doc greps the old module-path targets.
- serde_json without preserve_order = BTreeMap = deterministic encoding
  (landmine note: any dep unifying `preserve_order` breaks finish()'s
  changed-only compare silently).
- getKeys/register/UTF-8/readonly semantics match ObjectDB.pm.

## Defects to fix on #487 (ObjectDB persistence, unmerged)
1. `unregister` must DELETE the stored row — ObjectDB.pm:183 does, and says
   why ("else it'll get pulled back in!"); ours resurrects on re-attach.
2. XML idempotence unproven: add attach→finish==0 over an XML-bearing entry
   (round-trip bytes must be stable or every finish rewrites all titles).
3. One bad row poisons attach (`?` propagation) — skip+Warn per row instead.
4. No WAL / busy_timeout: the layer exists for concurrent workers; two
   readers + a writer must not SQLITE_BUSY-fail mid-transaction.
5. finish() encodes ALL entries before filtering and holds `baseline` (a full
   JSON copy of the db) resident attach→finish — restructure to
   encode/compare/insert per entry inside the transaction; measure at witness
   scale before the worker-mode PR.
6. Minors: `clean+readonly` deletes then fails to open; Int via
   `as_i64().unwrap_or_default()` silently zeroes; foreign sqlite file with
   user_version==0 gets tables injected.

## Defects to fix in diagnostics (follow-up branch)
1. `emit_error`'s exact-crossing cap latch (== maxerrors+1) is permanently
   defeatable (demoted-scope increments, MAX_ERRORS scope drops, STATE
   contention skip). Fix: latch on `>` guarded by the sticky fatal
   (fires once, robust to skipped checks).
2. Consecutive-cap message is dead code (over_total wins the ternary) and the
   latch can double-fire — pick message by which cap crossed, latch once.
3. Document the deliberate divergence: post errors now CAP at 100 where Perl's
   Post neither counts nor caps ($STATE-gated, Common/Error.pm:372), and our
   cap latches-and-continues where Perl dies. Fix latexml_post/diag.rs's now-
   false "no error-cap" doc; add OXIDIZED_DESIGN entry.
4. lint_raw_log_diag.sh misses `use log::warn; warn!(…)` and `log::log!` —
   tighten the pattern.
5. Noted, deliberate (Perl parity): tallies count records the verbosity
   filter drops — Perl counts independent of verbosity. --quiet pays eager
   format!+count (~0.1 s/800k records): acceptable.
6. Pre-existing, unchanged: note_status borrow_mut panic risk inside
   report_mut! scopes; Fatal!'s count-without-emit phantom shape.

## Approach-level assessment
- v1 eager-load per worker contradicts §6's page-cache-sharing rationale —
  stated now in §6: v1 accepts N x db RAM; lazy read-through is the
  escalation if it binds. finish()-cost fix (above) is the same work.
- Release hygiene: rc4 was re-tagged three times; same-name artifacts differ
  by download date. Recommendation: NEXT cut is rc5; never move a published
  tag again.
- Test pyramid gap: witness-scale truths (tally, memory, split) are proven
  only by the 84-min manual run. Recommendation: a generated mid-scale
  fixture (~2-5 MB, math-dense, sectioned) exercising streaming + split +
  status + tally in CI.
- Dashboard coordination: tally + taxonomy changes shift cortex
  classification; corpus-diff the first fleet run against a pre-#484 baseline
  and expect warning-count jumps + new category buckets.
