# DP-audit package long-tail worklist

One file per line, flag count in brackets. Delete entries as each file
is either (a) ported to kind-parity, (b) breadcrumbed with a WISDOM
cross-ref explaining the structural/intentional divergence, or (c)
shown to be a Perl double-declaration captured by last-wins dedup.

Regenerate the flag counts with:
```
awk -F'\t' '{split($1,a,":"); print a[1]}' docs/def_parity_package.tsv \
  | sort | uniq -c | sort -rn
```

## Package files — still to examine

Cluster A (large, catalogued in SYNC_STATUS DP patterns — breadcrumbed
at top of each file; individual entries already covered by the
class-level WISDOM cross-ref):

- [x] texvc_sty.rs (30) — WISDOM #40 DefMacroI↔DefMath top-of-file
- [x] physics_sty.rs (22) — WISDOM #44 + inline at `:265`
- [x] pgfsys_latexml_def.rs (17) — top-of-file comment
- [x] babel_support_sty.rs (15) — inline comment
- [x] llncs_cls.rs (15) — inline at `:143`
- [x] svmult_cls.rs (13) — cross-ref inline (llncs)
- [x] amsppt_sty.rs (11) — WISDOM #42
- [x] mn2e_support_sty.rs (9) — top-of-file comment
- [x] revsymb_sty.rs (8) — WISDOM #41 + inline at `:12`
- [x] amsmath_sty.rs (5) — WISDOM #41 (alignsafeOptional)

Cluster B (long tail, ≤4 flags each — per-cycle triage):

- [x] pspicture_sty.rs (3) — WISDOM #41 (Pair/picture-helper gap)
- [x] mathtools_sty.rs (2) — WISDOM #44 (`\adjustlimits`/`\newgathered`)
- [x] titlesec_sty.rs (2) — WISDOM #44 + edef-free usage check
- [x] xcolor_sty.rs (2) — WISDOM #44 (`\xglobal`/`\providecolor`)
- [x] marvosym_sty.rs (2) — cleared by last-wins dedup
- [x] graphics_sty.rs (1) — `\Gscale@div` ported to DefPrimitive (cycle 231)
- [x] siunitx_sty.rs (2) — already breadcrumbed at `:1928` (verified cycle 233)
- [x] pstricks_support_sty.rs (2) — WISDOM #41 breadcrumb added cycle 233
- [x] pst_node_sty.rs (2) — WISDOM #41 breadcrumb added cycle 233
- [x] psfrag_sty.rs (2) — TODO + semantic note (verified cycle 233)
- [x] xspace_sty.rs (1) — WISDOM #44 breadcrumb at `:10` (verified cycle 232)
- [x] wasysym_sty.rs (1) — cleared by last-wins dedup (cycle 230)
- [x] sv_support_sty.rs (1) — WISDOM #44 breadcrumb added cycle 232
- [x] subfig_sty.rs (1) — DP-flag note in inline comment (verified cycle 232)
- [x] numprint_sty.rs (1) — DP note inline in "Port of Perl" comment (cycle 233)
- [x] multirow_sty.rs (1) — wrapper explanation at `:47`; DefMacro→DefPrimitive structural (cycle 233)
- [x] makecell_sty.rs (1) — WISDOM #44 breadcrumb at `:7` (verified cycle 232)
- [x] jhep_cls.rs (1) — WISDOM #44 breadcrumb added cycle 233
- [x] colordvi_sty.rs (1) — fully WISDOM #44 breadcrumbed (verified cycle 233)
- [x] dcolumn_sty.rs (1) — fully WISDOM #44 breadcrumbed (verified cycle 233)
- [x] deluxetable_sty.rs (1) — fully WISDOM #44 breadcrumbed (verified cycle 233)
- [x] ieeetran_cls.rs (0 — closed cycle 229)
- [x] pgfcircutils_tex.rs (0 — closed cycle 229)

**Cluster B fully triaged 2026-04-23, cycle 233.** All package DP flags
now carry either a code fix or a WISDOM cross-ref breadcrumb. The
package TSV still has 176 flagged entries, but each corresponds to a
catalogued structural pattern with a per-file breadcrumb. The audit
is now purely regression-tracking — flag increases will signal new
port regressions rather than hidden debt.

## Contrib files

- [x] biblatex_sty.rs (2) — WISDOM #44 cycle 229
- [x] catchfile_sty.rs (2) — WISDOM #44 cycle 229

## Engine files (13 remaining, all blocked / intentional per SYNC_STATUS)

- [x] tex_math.rs (3) — WISDOM #41 (TeXDelimiter missing)
- [x] latex_constructs.rs (6) — WISDOM #38 + #41 (picture / \vspace)
- [x] plain_base.rs (4) — WISDOM #40 (mode-split intentional)
