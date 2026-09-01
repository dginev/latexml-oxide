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
| 5 | `\PkgInfo` (10) | ~~oberdiek family~~ **CLEARED** (agent-verified 2026-08-31) | Misattributed: it was Frigeri's codedescribe `[infograb]` option (pkginfograb.sty L301-314 aliases, gated on `\ProcessKeyOptions`) — already fixed by OXIDIZED_DESIGN #164 (`\@raw@opt@<name>.<ext>`). All 13 bundles re-verified: 10 at 0 errors; residue = tikzfxgraph shell-escape gnuplot (out of scope), ufrgscca titleps (`\renewpagestyle`/`\sethead`, small binding would clear 4/6) + datetime `\monthname`, tikzquads group-frame family (parked). |
| 6 | `Error:latex:(etoolbox)` (8 docs/7 bundles; hep-acronym was a mis-attribution — it PASSES) | windycity, biblatex-ext/-fiwi/-sbl | Agent-root-caused 2026-08-31, 3 causes: **(A)** 77/81 errs = biblatex binding never loads `.bbx`/`.cbx` style files (Rust-only regression — Perl has no biblatex binding and raw-loads the chain; biblatex.sty L2256 `\RequireBibliographyStyle`); two-step recipe: first port declaration-gobblers (`\DeclareBiblatexOption` etc. — probed: raw .bbx input without them = 100 NEW errors), then wire the raw `.bbx`/`.cbx` load at biblatex_sty.rs L757. Must land with `\citereset`/`\AtNextCitekey` family or no visible change. **(B)** `\robustify\bmod` — FIXED batch 6 (Stored[ sentinel). **(C1)** fullwidth = upstream-broken doc (pdflatex fails too, ltxmdf.cls L46 `\pdftex_if_engine:TF` undefined in TL2025) — closed, shared failure. **(C2)** beamerswitch = `\mode<presentation>` gobble bug + class-mode dispatch (`\IfEndWith*\JobName`), separate ticket. |
| 7 | `\subtitle` (was "8") | ~~beamerposter~~ **CLEARED/CLOSED** (agent-verified 2026-08-31) | Actually 1 fixable doc (schooldocs — `\subtitle` defined inside a `\fancypagestyle` body both engines discard; FIXED batch 6, schooldocs_sty.rs) + 6 docs on `amltxdoc.cls` which TeX Live does not ship (unfixable, real LaTeX can't compile them either). Residual schooldocs-examples: `\correct`/`\makesmalltitle` state divergence (our locked `\maketitle` refuses schooldocs' renew — 3× `Info:ignore`) — separate item. |
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

| xkeyval-internals cluster E (`\XKV@testopta`/`\XKV@s@tkeys`/… undefined — chessboard ×3, xskak ×2, xkeymask) + `\Xskakthe package…` 1000-error csname cascades + `\define@cmdkey` `misdefined:#` | Batch-5 xkeyval deep port: verbatim front-end scaffolding + `\XKV@s@tkeys` shim onto private `\lx@xkv@setkeys`; real pointer system (`\savekeys`/`\savevalue`/`\usevalue` + `\XKV@<header><key>@value` store); KNOWN_PERL_ERRORS #80 cmdkey fix; token-form key defaults (the cascade's root: `[\xskak@val@defaultid]` split by string round-trip); xkeymask_sty.rs binding. xkeymask 7→0/0/0, xskak 1001+fatal→5, chessboard 1001-truncated→1 | `xkeyval_internals::*` (2 tests) |

### Post-batch residual clusters (2026-08-31, from witness re-runs)

| Signature | Note |
|---|---|
| chessboard.tex `\chessboard[boardfontencoding=LSBC4]` → 400M-token churn (doc L1627; LSBC4 falls back to OT1 first) | Doc now renders to L1627/2049 with 1 error after the batch-5 xkeyval port; a single board render under the missing-encoding fallback loops. Font-encoding-dependent raw drawing — needs its own min-repro session. |
| `misdefined:#` (adtrees 17→**2** after KNOWN_PERL_ERRORS #80) | cmdkey fix cleared 15/17; residual pair fires at an Anonymous String near begin-document (l3backend load) and does NOT reproduce with the full preamble + frontmatter alone (m4/m5 probes clean) — body-driven, needs its own bisect session. |
| xskak residual 5 errs: `\usepackage` post-preamble (doc-driven filecontents flow), `\csq@hook@{multilang,hyperref}`, `\board` | small distinct items, re-rank in sweep 13. |
| `undefined:\@openrightfalse` + `\if@openright` (6+ docs, toptesi residual) | class-context newif; toptesi now passes its version check (was aborting) and exposes this. |
| `undefined:\BeforeClosingMainAux` (toptesi) | atveryend surface. |
| math-symbol CS as tikzmath variable (`\angle`, sunpath 1001 errs) | same meaning-shape family as #170 but for math chars — separate fix. |
| pgf-spectra manuals TIMEOUT under debug 120s | legitimately heavy spectral compute now runs (witness call solo: 9s clean, 0 err); re-check at release-profile sweep. |
| `\AlegreyaSans`/`\Alegreya` (parnotes 2 errs) | font-package family macros behind further engine branches. |

### Sweep-11 sampled, still open (2026-08-31)

| Signature (bundles) | Sample verdict |
|---|---|
| `undefined:\bool` (10, create-theorem/einfart family) | expl3 catcode regime lost mid-raw-load: einfart.cls L155 `\bool_if:NT` tokenized as `\bool`+`_if:NT` — the `[[project_explsyntax_midload]]` family, not a missing macro. |
| `Error:latex:(doc) Character table corrupted` (11: frankenstein ×10 + pkgloader) | **Agent-root-caused 2026-08-31, recipe ready (batch 7)**: `\DocInput{<name>.sty}` routes through `input_definitions` (content.rs L1384-1399) whose mouth forces `@`=letter (`at_letter: true`, mouth.rs L488), so the re-read `\CharacterTable`'s `\@` tokenizes as a control WORD (skips following spaces) vs the stored control-symbol+space — `\ifx` fails (doc.sty L818-849; the check is at L827). Perl silently skips the re-read entirely (`reloadable => false`, Package.pm L2265) — passes by doing nothing. Natural A/B in pkgloader's own log: definitions-path load = corrupted, content-path load of the SAME file = correct. Recipe (recommended B): gate the `is_binding_extension` probe on INTERPRETING_DEFINITIONS/preamble; document-body `\input{x.sty}` falls through to `load_tex_content`. Also removes pkgloader's duplicated Implementation section. |
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
