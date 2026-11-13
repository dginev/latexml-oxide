# latexml-oxide agent instructions

These are the durable instructions for automated contributors. Current source,
tests, and checked-in documentation override remembered or imported context.

## Start from current state

- `CLAUDE.md` is the original and primary instruction document for this
  repository; its invariants, directives, and build/test contracts apply across
  all contributors and tools.
- Read `docs/README.md` to find the owning documentation, then read the relevant
  subsystem docs before proposing or editing. For current priorities, test
  counts, and corpus status, read `docs/SYNC_STATUS.md`; never quote a remembered
  snapshot as current.
- Inspect `git status --short` before editing. This is a shared, frequently dirty
  checkout: preserve unrelated changes, never discard work you did not create,
  and avoid destructive Git commands. Do not use the repository-wide stash;
  worktrees share its stack and can consume one another's entries.
- Read `docs/THERMALS.md` before any suite, sweep, oracle, validation, or other
  parallel workload. Do not overlap a sweep with the full test suite. Respect
  the documented `JOBS` and memory ceilings. Keep bulk corpus extraction, logs,
  and sweep output under `/home/data` or another persistent filesystem, not the
  RAM-backed `/tmp`.
- Preserve agent-specific material already present under `.agents/`. Shared
  project workflows belong in `.agents/skills/latexml-*`; local rules and
  third-party managed skills remain untracked unless the user requests otherwise.

## Correctness contract

- This is a faithful Perl-to-Rust translation. Read the corresponding source in
  `LaTeXML/` first and preserve its semantics, control flow, edge cases, naming,
  and definition kind. `LaTeXML/` is a read-only oracle, not an edit target.
- Perl LaTeXML is the translation and behavior baseline. Compare both engines
  verbosely on the same host and TeX tree. When Perl's binding is a simplified
  model, inspect the real kernel or package source with `kpsewhich`, plus
  `background/tex.web` where TeX mechanism matters.
- Never make a failure disappear by lowering diagnostic severity, suppressing a
  log, weakening a guard, loosening an assertion, or installing a semantic no-op
  stub. Fix the responsible mechanism. A log parse failure is not success.
- A deliberate beyond-Perl behavior requires explicit authorization and a
  numbered entry in the `docs/parity/OXIDIZED_DESIGN*` family, referenced at the
  code site. Check `docs/parity/KNOWN_PERL_ERRORS.md` and
  `docs/parity/WISDOM.md` before deciding that parity is wrong.
- Rust package bindings take precedence over raw `.sty`/`.cls` loading. Prefer
  complete semantic support; document the narrow reason for any cosmetic or
  explicitly out-of-scope stub.
- Do not delete witness arXiv identifiers from code comments. Re-run any witness
  named beside a construct you change.

## Dump and architecture invariants

- Preserve strict `LoadFormat` exclusivity: either
  `bootstrap -> dump -> constructs` or `bootstrap -> base -> constructs`, never
  both. Dump replay is unconditional and last-writer behavior is intentional.
- Keep engine definitions in the Rust file corresponding to the Perl pool. Port
  Perl `RawTeX` definitions as serializable token bodies, not opaque closures.
  Dump-generation init runs must finish with zero errors.
- `SymStr`, the global `State`, and libxml-backed nodes are thread-affine. Do not
  add `Send`/`Sync` implementations or move them across threads. Keep libxml2 for
  dynamic XPath behavior unless an approved design replaces that contract.
- Do not add raw C FFI to `latexml_oxide`; native bindings belong in their
  dedicated wrapper or `-sys` crates.
- Follow the repository's interner and hash-map conventions instead of adding
  owned-string churn or default `HashMap` use in hot paths. Verify conventions
  in the owning crate before changing representations.
- The distributed binary is self-contained for project-owned dumps, schemas,
  XSLT, CSS, and JavaScript. Embed owned resources; runtime reads from the host
  TeX tree remain expected.

## Performance work

- Read `docs/performance/PERFORMANCE.md` and the latest dated audit before
  optimizing. Do not reopen a measured, closed lever without new evidence or a
  changed architectural premise.
- Performance changes are output-neutral unless the user explicitly authorizes
  a beyond-Perl change. Measure one lever at a time with same-host, back-to-back
  baseline/candidate runs on representative production inputs.
- Use `--profile bench` for profiling and wall-time benchmarks, `--release` for
  production-like corpus/parity runs, and `maxperf` only for distribution work.
- A performance claim needs exact output, status, diagnostic, and phase parity,
  plus wall time, CPU time, and peak RSS. Report regressions and variance, not
  only the favorable aggregate.

## Tests, docs, and handoff

- The full suite is `cargo nextest run --workspace`. Use focused tests while
  iterating. A new `.tex`/`.xml` pair needs `cargo clean` once so compile-time
  discovery sees it. Judge suites by exit status and result summaries, not by
  grepping expected diagnostic fixtures.
- Before publication, run the applicable full suite, nightly clippy with
  `-D warnings`, and formatting. Do not claim a check that was not run.
- Put durable state in its owning doc, reusable procedure in a skill, and only
  slow-changing preferences or recall hints in generated local memory. Update
  `docs/README.md` when adding, moving, or archiving documentation.
- Handoffs must name the current branch/worktree state, exact files changed,
  commands actually run, remaining risks, witnesses, and the single best next
  step. Do not copy volatile counts or priorities into durable instruction files.

## Project skills

Use the matching workflow under `.agents/skills/` for performance measurement,
Perl ports, paper triage, reproducer reduction, dump debugging, corpus clustering,
intentional divergence, issue resolution, session startup, and releases. Their
descriptions are activation gates; do not load every workflow for unrelated work.
