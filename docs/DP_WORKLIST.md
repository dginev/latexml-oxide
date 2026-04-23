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
- [ ] siunitx_sty.rs (2) — `\lx@six@unitobject`/`\lx@six@unitobject@arg`
- [ ] pstricks_support_sty.rs (2) — `\psset`, `\@@@ackscale`
- [ ] pst_node_sty.rs (2) — `\rnode`, `\pnode`
- [ ] psfrag_sty.rs (2) — `\psfragscanon`, `\psfragscanoff`
- [ ] xspace_sty.rs (1) — `\xspace` (already has WISDOM #44 comment; verify)
- [ ] wasysym_sty.rs (1) — cleared by last-wins dedup; verify
- [ ] sv_support_sty.rs (1) — `\spnewtheorem`
- [ ] subfig_sty.rs (1) — `\newsubfloat`
- [ ] numprint_sty.rs (1) — `\ltx@text@number`
- [ ] multirow_sty.rs (1) — `\multirow`
- [ ] makecell_sty.rs (1) — `\lx@makecell@head` (already has WISDOM #44; verify)
- [ ] jhep_cls.rs (1) — `\hash`
- [ ] colordvi_sty.rs (1) — `\DefineNamedColor`
- [ ] dcolumn_sty.rs (1) — `\lx@unactivate`
- [ ] deluxetable_sty.rs (1) — `\set@deluxetable@template`
- [ ] ieeetran_cls.rs (0 — closed cycle 229)
- [ ] pgfcircutils_tex.rs (0 — closed cycle 229)

## Contrib files

- [x] biblatex_sty.rs (2) — WISDOM #44 cycle 229
- [x] catchfile_sty.rs (2) — WISDOM #44 cycle 229

## Engine files (13 remaining, all blocked / intentional per SYNC_STATUS)

- [x] tex_math.rs (3) — WISDOM #41 (TeXDelimiter missing)
- [x] latex_constructs.rs (6) — WISDOM #38 + #41 (picture / \vspace)
- [x] plain_base.rs (4) — WISDOM #40 (mode-split intentional)
