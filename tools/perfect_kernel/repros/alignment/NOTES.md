# alignment topic — Checkpoint 1 notes (2026-09-03, binary b54l / d1dd27af3c)

Judge: ANSI-stripped `^Error:|^Fatal:`. All repros verified RED on b54l with
`--preload=[rawstyles,rawclasses]latexml.sty`.

## Roots found in the residue (ranked by docs × error mass)

- **Root A — hidden-$ in a MATH-ALIGNMENT cell left open.** Symptom: at cell/row
  close, `egroup`/`endgroup`/`\lx@begin@alignment`/`\org@halign` hits a mode-switch
  math frame opened by the amsmath column template's `\lx@begin@inline@math` (the
  template `$`). Docs: numerica(83, SEED), tablists-rus(101, `\org@halign` variant),
  mhchem(14, SEED). SHARED (Perl 6-7), pdflatex 0 → in scope.
- **Root B — `\noalign` inside a display-math `\halign`.** `\egroup ... display_math`
  + `\noalign cannot be used here`. Docs: polynom/polydemo(101).
- **Root C — `\begingroup` spanning an alignment.** `\lx@begin@alignment Attempt to
  close boxing group`, frame `\begingroup`. Docs: t-angles/t-manual(101).
- **Root D — package owns its own alignment-body parse; the kernel over-cell error
  leaks ("Extra alignment tab '&'").** Rust has BINDINGS (tabularray_sty.rs,
  nicematrix_sty.rs) that delegate to a fixed-column kernel `\tabular`; Perl has NO
  binding for either — it RAW-LOADS the real package, which parses its own body and
  tolerates ragged/nested cells. Docs: nicematrix(39), circularglyphs-doc(1). RUST-ONLY.
- Downstream/secondary `\noalign` docs (first error is NOT alignment; deprioritise):
  aguplus(24 figcaps gate), objectz(29 math-version), harmony(9), tex-font-cheatsheet(24),
  shipunov(4 tabular boldline), pfdicons(8), uwthesis(1).
- PARKED (not this topic): gckanbun, kksymbols (`\epTeXinputencoding`, pTeX).

## SEED hypothesis — VERIFIED with frame trace, and CORRECTED

Seed claimed the mechanism is `MATH_ALIGN_$_BEGUN` boxing-level pairing and proposed
resetting it in `alignment::end_column` / `tex_tables::digest_alignment_body`.
**That is the WRONG mechanism for align*/gather*.** The amsmath rearrangeable
templates (amsmath_sty.rs:210-236, faithful to amsmath.sty.ltxml:458-462) use
`T_MATH!()` = `\lx@dollar@default`, and amsmath_sty.rs:123-131 DELIBERATELY does NOT
`Let(T_MATH,'\lx@dollar@in@mathmode')`. So `\lx@dollar@default` (tex_math.rs:469)
dispatches purely on MODE ("math" → `\lx@end@inline@math`), and never consults
`MATH_ALIGN_$_BEGUN`. Resetting `MATH_ALIGN_$_BEGUN` would be a no-op here.

`LXML_TRACE_BOUND_MODE=1` trace of the numerica seed: exactly **2 `begin_mode math`
(the two cells' before-template `$`) and 0 `end_mode`** before the first error → the
cell's AFTER-template `$` never ran `\lx@end@inline@math`, so the template math frame
is still on top when the cell box is closed → the egroup mode-switch error.

Trigger isolated empirically:
- `\eval{1+1}` (arith path) in align* → CLEAN. Only the SLASH path breaks.
- `$\eval{1/8}$` (plain inline math, no alignment) → CLEAN.
- `\eval{1/8}` in gather* (single col) → SAME 6 errors → it is any amsmath math
  alignment, not align-specific.
- Hand-written balanced `\begingroup/\endgroup` AND expl3 `\group_begin:/\group_end:`
  + `\exp_after:wN \group_end: <tl> \group_begin:` dances in an align cell → CLEAN.
  So numerica's raw group dance (numerica.sty:1580) is NOT sufficient on its own; the
  `\l__nmc_show_tl` content / `\eval` epilogue must interact with how the alignment
  RE-TOKENISES + re-digests the cell (locator during the dance is at `\end{align*}`,
  i.e. the cell was collected as tokens then replayed). Pinning exactly which token in
  the replayed cell prevents the after-`$` from firing is Checkpoint-N work.
- tex.web-faithful direction: the align template supplies BOTH `$`; TeX re-inserts
  v_j (with its `$`) at `&`/`\cr` regardless of the cell body's groups, so the cell
  math is balanced by construction. The fix must guarantee the after-template `$`
  runs at the cell boundary even when the cell body did an inner close+reopen — NOT
  a MATH_ALIGN_$_BEGUN reset.

Classification: **SHARED** (Perl fails identically, 6-7 errors), pdflatex 0 → surpass
in scope.

## Repros written (scratch repros/)

- cell_dollar_numerica_eval_slash.tex / cell_dollar_mhchem_ce.tex — SEEDS (Root A),
  re-verified RED 6 errors each on b54l.
- tblr_ragged_row_circularglyphs.tex — Root D. Rust=1, Perl=0 (RUST-ONLY),
  pdflatex=0. tabularray tblr, 13-col colspec, a 14-cell row.
- block_ampersand_nicematrix.tex — Root D. Rust=3, pdflatex=0; Perl blocked earlier
  by pgfsys driver (UNCLASSIFIED-Perl, in scope). `\Block{}{a & b}` +
  `[ampersand-in-blocks]`.
- CONTROL boundary: plain `\begin{tabular}{ccc}` with an extra `&` errors in ALL of
  Rust(2)/Perl(2)/pdflatex(1) — proves the kernel over-cell error is CORRECT; only
  tabularray/nicematrix (which own the body) must tolerate it.

## Fix sites (Checkpoint-N)

- Root A: kernel — guarantee the amsmath column template's after-`$`
  (`\lx@end@inline@math`) runs at cell boundary. tex_tables.rs alignment
  digest / latexml_core alignment.rs cell close; NOT a MATH_ALIGN_$_BEGUN reset.
- Root D (tabularray): latexml_contrib/src/tabularray_sty.rs — `translate_tblr_colspec`
  / tblr body handling must grow columns to the row max (real tabularray parses its
  own body; Perl raw-loads it and is clean).
- Root D (nicematrix): latexml_contrib/src/nicematrix_sty.rs — `\Block{}{…}` body with
  `ampersand-in-blocks` must keep the inner `&` as literal content, not a column tab.

# =====================================================================
# Checkpoint N #1 — ROOT A: the ACTUAL mechanism (align_state drift)
# =====================================================================

CLASSIFICATION: SHARED (Perl 6-7 errors identically), pdflatex 0 → surpass in
scope. Witnesses: numerica/numerica (83), mhchem/mhchem (14), tablists/tablists-rus
(101, `\org@halign` variant).

## Mechanism (proven end to end)

The amsmath align/gather column template wraps each cell in `$…$`
(`amsmath_sty.rs:210-236`; the `$` = `T_MATH!` = `\lx@dollar@default`,
tex_math.rs:469, dispatching on MODE). The BEFORE-template `$` opens inline math
(`\lx@begin@inline@math`, a mode-switch frame). The AFTER-template `$` (v-part)
is NOT in the cell token stream: it is inserted by `handle_template`
(latexml_core/src/gullet.rs:650-687) — but ONLY when `read_x_token` sees the
cell-ending `&`/`\cr`/`\\` WITH `align_group_count()==0`
(gullet.rs:1250-1267). `align_group_count` is the tex.web `align_state` analog
(one ledger per alignment, pushed at tex_tables.rs:832; braces inc/dec it;
`start_column` sets 1000000; the "before-column" marker resets to 0).

Q1 (where after-tokens go): the after-template/v-part is inserted into the GULLET
as tokens by `handle_template` at the `&`/`\cr`, gated on `align_group_count()==0`.
It is NOT appended to a digested list.

Q2 (does numerica read past the cell `$`/`&`): NO. `\__nmc_delim_arg:Nnn` scans a
STORED tl var (`\l__nmcA_tl`), never the live cell stream. The bug is a ledger
DRIFT, not a read-past.

ROOT: numerica's `\eval` SLASH path leaves `align_group_count` != 0, so the cell's
closing `&`/`\cr` fails the gullet.rs:1251 gate, `handle_template` is never called,
the after-`$` is never inserted, the before-`$`'s `\lx@begin@inline@math` frame
stays open, and the cell/row-close egroup hits it (stomach.rs:733 "Attempt to close
a group that switched to mode math").

## Proof chain (repros in repros/, all on b54l)
- `\eval{1+1}` (arith) in align* → CLEAN;  `\eval{1/8}` (slash) → 6 errors.
- `$\eval{1/8}$` (no alignment) → CLEAN;  `\eval{1/8}` in gather* → 6 errors
  (any math alignment, not align-specific).
- `\eval{1/8} & b` → 8 errors: the `&` AFTER the eval is NOT recognized (proves the
  ledger is left non-zero); `\eval{1.5} & b` → CLEAN.
- Bisect on a scratch copy of numerica.sty (TEXINPUTS-preferred): the culprit is
  `\__nmc_delim_arg:Nnn` (numerica.sty:2569). D0 (body→noop)=CLEAN, baseline=8;
  the head-is-group test alone=CLEAN; the `\exp_args:NNNV \group_end:` split is
  NOT it. Wrapping ITS else-branch loop in expl3's own `\group_align_safe_begin:
  … \group_align_safe_end:` → 0 errors and correct XML (equationgroup /
  ltx_eqn_align, "0.125" present). Wrapping only `\__nmc_next:` did NOT fix →
  the drift accumulates across the loop's MANY macro-arg re-grabs (e.g.
  `\bool_until_do:nn` re-reads `{cond}{body}` each iteration).

## tex.web (why real TeX never drifts)
macro_call makes parameter scanning align_state-NEUTRAL: undelimited args freeze
`align_state:=1000000` (tex.web.p §394, background/tex.web:15512 restart pattern)
and delimited args self-correct `align_state:=align_state-unbalance`
(background/tex.web:8095, §400). read_toks/scan-def likewise save+1000000+restore
(tex.web:9459). So braces consumed inside a macro's arguments leave the SURROUNDING
`align_state` untouched. Rust does the correct thing for `read_balanced` (live
ledger, gullet.rs:1511) but does NOT neutralize `align_group_count` around
macro-PARAMETER scanning — so expl3 brace-tricks re-grabbed across a loop's macro
calls drift the live ledger permanently.

## FIX (kernel, faithful to tex.web macro_call §394/§400)
Site: `latexml_core/src/parameter.rs::read_arguments` (654) and
`read_arguments_and_digest` (673) — wrap the parameter-reading loop in
`SuppressedTabMarks::for_argument_scan()` (latexml_core/src/common/
local_assignments.rs:159-179), which already implements save→set 1000000→restore
and self-disarms outside alignments (`has_reading_alignment()`). It is currently
DEAD except for a one-off package patch (physics_sty.rs:118, `\mqty`, witness
2605.05903) — generalise that to the kernel. Each in-cell macro call then leaves
`align_group_count` at its pre-call value; the cumulative loop drift vanishes
(= the W2 result), and the cell-ending `&`/`\cr`, read at cell top level OUTSIDE
any arg scan, is recognised so the after-`$` closes the math.
- Guard (cluster_package_guards / perfect_kernel batch): `cell_dollar_numerica_
  eval_slash.tex` → 0 errors AND output contains `<ltx:equationgroup` with the
  cell Math XMTok "0.125"; add `cell_dollar_mhchem_ce.tex` → 0 errors + one
  `<ltx:equation>` with the reaction. Keep the CONTROL (`{$b$}`) still erroring.
- Risk: MED. Hot-ish (every in-alignment macro call), but guarded by
  `has_reading_alignment()` (off outside alignments) and cheap (Option<i32>).
  Must NOT touch `read_balanced` (gullet.rs:1511 design boundary). Main session to
  build + run the full alignment tests to confirm mhchem/tablists also clear and no
  alignment regression (e.g. xymatrix, physics `\mqty`, split/eqnarray).
- Expected corpus gain: numerica(83) + mhchem(14) + tablists-rus(101) share this
  root; other math-alignment expl3 truncations likely fold in too.

## Dead ends (one line each)
- Seed's `MATH_ALIGN_$_BEGUN` boxing-level reset: wrong mechanism — align*/gather*
  use `\lx@dollar@default` (MODE-based), never consult `MATH_ALIGN_$_BEGUN`.
- Neutralising numerica's `\exp_args:NNNV \group_end:` split (parenth+delim_arg):
  no effect (still 8 errors).
- Hand-built expl3 idioms (exp_args:NNNV+group_end; exp_last_unbraced:NV+q_stop
  split; implicit `\c_group_begin_token` in a delimited arg; group dance): none
  reproduce the drift in isolation — it is the loop's cumulative residue.
