# Out of scope (moved from SYNC_STATUS.md 2026-05-01)

Empirically verified: Perl LaTeXML on TL2025 with --preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings does NOT produce 0 errors on this paper, so it fails the in-scope predicate ("in scope iff Perl produces 0 errors").

Original SYNC_STATUS.md task content preserved below for future reference.

### 1. math0005251 — math-parser cumulative-state OOM

Only filesystem-level hard failure left in the April29 sandbox. Rust
allocates ~28 GB digesting the paper's math while Perl finishes in
~10.5 s / 234 MB. Min repros run cleanly; the trigger needs enough
prior math-state accumulation. See
`memory/project_math_parser_state_cumulative_hangs.md`. Expected
fix is grammar-level work in `latexml_math_parser`.
Acceptance: `( ulimit -v 6291456; latexml_oxide … math0005251.zip )`
exits 0 with non-empty HTML.

