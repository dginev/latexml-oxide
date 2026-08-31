# Perfect Kernel — failure-cluster worklist (living)

Rebuilt after each sweep from `~/data/perfect_kernel/sweep_verdicts.tsv` +
first-error extraction. Discipline: cluster by first-error signature, sample
2–3 representatives before believing a cluster, count *clusters* fixed, not
documents.

## Current clusters — sweep #2 (2026-08-31, fresh dump + #161)

Ranked by **distinct bundles** among oracle-clean docs with errors (first-error
signature). 770 oracle-clean docs still carry errors; 8 fatal; 1 timeout.

| Rank | Signature (bundles) | Representative | Verdict / plan |
|---|---|---|---|
| 1 | `\minisec` (17) + `{labeling}` (11) + `\addsec`-family | beamerposter | KOMA surface missing from the OmniBus-backed scr* bindings. Extend the existing contrib scr* bindings (no new files) or improve OmniBus's KOMA vocabulary. |
| 2 | `\@indexfile` (14) | arydshln-man | index machinery (`\makeindex`/`theindex` raw path writes `\@indexfile`). Kernel gap — root-cause next. |
| 3 | `\ltd@title@title` (12) | abraces-doc | **PARKED by user decision 2026-08-31 ("Keep locks")**: locked kernel CSes stay protected from raw-file redefinitions everywhere (Perl-identical). ltxdockit.cls's `\renewrobustcmd*{\titlepage}` stays refused; the 12 bundles remain SHARED-FAILURE. Do not re-attempt without a new user decision. |
| 4 | `\CJKaddEncHook` (12) + `luatexja-core` (5) + `\dhucs@hu` | CJK ecosystem | CJK raw interpretation — catalog under DIFFICULT_CASES (encoding-hook machinery). |
| 5 | `\PkgInfo` (10) | oberdiek family | hobsub/pdftexcmds generation-era `\PkgInfo` — kernel-adjacent, likely small. |
| 6 | `Error:latex:(etoolbox)` (9) | hep-acronym | etoolbox binding raises GenericError on patch failure classes. |
| 7 | `\subtitle` (8) | beamerposter themes | beamer `\subtitle` in non-beamer classes / theme frontmatter. |
| 8 | `\setmathfont` (8) + `\fontspec_if_language:nT` (5) + `\defaultfontfeatures` (4) | unicode-math / fontspec | LuaLaTeX font-selection surface; extend existing fontspec binding noops (D6 policy). |
| 9 | `\Hy@MakeCurrentHref` (8) | hyperref internals | raw packages poking hyperref internals our binding doesn't expose. |
| 10 | `malformed:ltx:glossaryphrase` (7) | abntex2 | glossaries schema-shape bug — S2-relevant. |

Error-mass hotspots (oracle-clean, errors per doc): gckanbun 12,566×2 (vertical
kanbun), panda-doc 3,600, xcolor2 2,280, kksymbols 1,003, atableau/pmdraw 1,001.
NOTE: several exceed the nominal error cap — cap behavior itself worth a look.

### Post-sweep-4 top clusters (2026-08-31, later session)

| Signature (bundles) | Status |
|---|---|
| `Error:expected:\fi` (13; hep-* family ≈10 of them, + pythontex=D7, misc) | OPEN — 5-line repro: `\usepackage{hep-font}` under the raw preload → `Missing \fi or \else, conditional fell off end` attributed to hep-font EOF (line 193). NOT the xpatch `{\else#1}` block (removing it doesn't help; standalone xpatch repros are clean). Error appears right after xparse/expl3 mid-preamble raw loading; INTERMITTENT w.r.t. load path (local-searchpath copy was clean in one run, errored in another) — suspect conditional-stack bookkeeping across file/mouth boundaries during nested raw loads. Needs a focused session with conditional-stack tracing. |
| `Error:unexpected:&` (12) | mode/alignment family — overlaps parked R9 mode-frame; sample before attempting. |
| `\ltd@title@title` (12) | expected to clear in sweep 5 (ltxdockit_cls.rs landed after the sweep-4 binary). |
| `malformed:ltx:glossaryphrase` (9, abntex2 family) | OPEN — glossary entries emitted at #Document root; simple glossaries flow is clean, abntex2's path differs. |
| `\BreakableUnderscore` (6) | l3doc/underscore interplay, surfaced once earlier errors cleared. |
| CJK/Japanese (`\CJKaddEncHook` 12, luatexja, pTeX prims `\kanjiskip`/`\prebreakpenalty`/`\西`) | DIFFICULT_CASES — pTeX/upTeX engine primitives out of pdfTeX-model scope; catalog, don't chase. |
| fontspec surface (`\setmathfont` 8, `\fontspec_if_language:nT` 5, `\defaultfontfeatures`/`\addfontfeature` 4+4) | D6 LuaLaTeX-only docs (oracle passes them via lualatex fallback); policy decision needed on modeling fontspec under pdfTeX-model engine. |

### nicematrix exemplar residual (2026-08-31, session 2)

One error left: `\cmidrule(rl){2-4}` inside `{NiceTabular}{lSSSS}` with a
`\Block{2-1}` in the same row → `\noalign cannot be used here`
(tex_tables.rs:224) — the Block/multicolumn row leaves the alignment
mid-cell when the rule's `\noalign` arrives. Alignment-family (overlaps the
parked R9 mode/alignment work); single instance in the manual.

## Retired clusters (this mission)

| Signature | Resolution | Guard |
|---|---|---|
| `\prop_gput_if_not_in:*`(124 docs) / `\UseTaggingSocket` (74) / `\sys_if_*` (66+28) / `\prop_new_linked:*` (50) / `\IfPackageLoaded*`+`\IfFormatAtLeast*` | Stale dual-TL dump — `make_formats.sh` TEXMF* pin + regenerated TL2025 dump; engine-side backend agreement check (`choose_kpaths` `same_tree_as_ambient`) | `ambient_tree_mismatch_falls_back_to_subprocess`; zero-error `--init` gate |
| `Error:expected:{` from `\lstnewenvironment` bodies on next line (~148 docs) | OXIDIZED_DESIGN #161 `DefPlain` skips blanks (surpass-Perl, approved) | `defplain_skips_blanks_before_brace` |
| First listing body line silently lost in `\lstnewenvironment[1][]` envs | OXIDIZED_DESIGN #162 pushback-aware raw-line capture | same guard, data-attr assertion |
| `Error:undefined:\newunit` (a4wide etc.) | DOCUMENT-STALE — oracle pass excludes (801 stale docs) | `oracle_verdicts.tsv` |
| babel Lua layer dead under luatex profile: EVERY profiled doc logged `attempt to index a nil value (field 'locale_props')`; chunks silently branched on `tex.count` stub zeros; `require` of texmf Lua modules failed (witnesses: derivative, abntexto, abntexto-uece, newpax — [LUA_REBINDING.md](LUA_REBINDING.md)) | Rebind-as-we-emulate landing: `\lx@directlua` double-`Expand!` removed + `\par` filtered (real-luatex probes as oracle), live register mirror, absorb shims, kpse+lualibs require, `\bbl@luapatterns` format parity, direction-keyword eaters | `luatex_babel_api::babel_lua_api_layer_initializes`, `lua_state_mirror::directlua_reads_and_writes_live_registers`, `rebound_engine_intents_absorb_and_resolve` |

### Post-batch residual clusters (2026-08-31, from witness re-runs)

| Signature | Note |
|---|---|
| `undefined:\@openrightfalse` + `\if@openright` (6+ docs, toptesi residual) | class-context newif; toptesi now passes its version check (was aborting) and exposes this. |
| `undefined:\BeforeClosingMainAux` (toptesi) | atveryend surface. |
| math-symbol CS as tikzmath variable (`\angle`, sunpath 1001 errs) | same meaning-shape family as #170 but for math chars — separate fix. |
| pgf-spectra manuals TIMEOUT under debug 120s | legitimately heavy spectral compute now runs (witness call solo: 9s clean, 0 err); re-check at release-profile sweep. |
| `\AlegreyaSans`/`\Alegreya` (parnotes 2 errs) | font-package family macros behind further engine branches. |

### Sweep-11 sampled, still open (2026-08-31)

| Signature (bundles) | Sample verdict |
|---|---|
| `undefined:\bool` (10, create-theorem/einfart family) | expl3 catcode regime lost mid-raw-load: einfart.cls L155 `\bool_if:NT` tokenized as `\bool`+`_if:NT` — the `[[project_explsyntax_midload]]` family, not a missing macro. |
| `Error:latex:(doc) Character table corrupted` (11, frankenstein) | doc.sty's catcode-table self-check fails under our engine — catcode introspection parity, needs its own min-repro session. |
| `misdefined:#` (17, adtrees) | PARAM token reaching Stomach from an Anonymous String after microtype — engine-level, min-repro session needed. |
| `undefined:\luaPST` (12, bardiag) | PSTricks-Lua surface; sample against D6 tiers before deciding. |
| fontspec surface (`\fontspec_if_language:nT` 17, `\setmonofont` 16) | D6 clean-lua slice — the declared next worklist. |

### Clean-lua slice — remaining (2026-08-31, post-rebinding)

The 216-doc/50k-error clean-lualatex slice now has its Lua substrate working
(babel Lua API up, mirrors live). Remaining mass is expected to be dominated
by the fontspec/unicode-math surface and ordinary (non-Lua) clusters — re-rank
after sweep 11 banks the rebinding fixes corpus-wide. Residual Lua witnesses:
newpax write-side (`.newpax` into the root-owned TL tree — real lualatex
fails there too; degrades to `Info:lua`).

Settled protocol point (user directive 2026-08-31): compiled `.rs` bindings
keep precedence under rawclasses/rawstyles; an experiment demoting the contrib
tier to raw was reverted the same day. Corpus focus = bindingless packages.
The `rawclasses` no-OmniBus requirement was verified already-true for
bindingless classes and is guarded by
`cluster_package_guards::rawclasses_binding_precedence_and_no_omnibus`.

## Retired clusters

| Signature | Resolution | Guard |
|---|---|---|
| | | |
