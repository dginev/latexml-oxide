# Perfect Kernel — improvement-plans ledger (living)

Detailed, execution-ready plans distilled from root-cause investigations
(subagent fan-out 2026-09-01, user directive: up to 8 read-only agents,
plans recorded here, fixes executed from the main session as each plan
finalizes). A plan graduates DRAFT → FINAL when its root cause carries
file:line evidence and the fix shape has a named risk assessment; FINAL
plans are executed in batch order and the row moves to DONE with the
batch number. Keep the conclusion, not the play-by-play
(CLAUDE.md doc rules).

| # | Target (mass) | Status | Plan summary |
|---|---|---|---|
| P1 | aomart display-math break | **DONE b33** | Root: locked `\newtheorem` grabbed the class's leading style-optional `[` as the theorem NAME; its csname form clobbered `\[`. Fix: signature absorbs+discards the optional (aomart.cls's own semantics). aomsample 101→10. KPE #82. |
| P2 | biblatex/droit-fr expandafter cascade | **DONE b34** | Root: an active csquotes quote char inside `\verb` (semiverbatim ignored `\dospecials`) fired `\csq@fixkern`'s \expandafter 7-chain at an argument-mouth end; our \expandafter loop then spun per-lap errors. Fixes: (F1) loop terminates on EOF with ONE error; (F2) verb applies `\dospecials` (models the real kernel registration mechanism). biblatex 101+F→6. |
| P3 | xint roman-csname recursion | **DONE b34** | Root: `\scantokens`/`writable_tokens` hardcoded `\` instead of honoring `\escapechar` (tex.web §1594 print_esc; Perl shares) — xint's two-stage capture executed live primitives inside its edef. Fix: escapechar-aware writable_tokens + §262 space rule. tkz-grapheur fatals→0; esc3 twin pdflatex-identical. |
| P4 | packdoc index shredding | **DONE b33** | Root: index-phrase splitter cut through brace groups (Perl flat scan). Fix: separators act at depth 0 only. algxpar 315→1. KPE #83. |
| P5 | Session refactor pass | **DONE b33 (SAFE-NOW set)** | with_meaning swaps, unread_mut deleted, retract_scanned_brace helper, dead expl3 locals, log hygiene. Deferred (measure-first): vsplit split-index-then-materialize, kpse negative-memo. Open: VerbatimOut dedup → superseded by the VFS abstraction below. |
| P6 | elsdoc "regression" | **DONE b34** | Verdict: sweep-21 "0" was fatal-masked (Status:3 with zero Error lines — the CLAUDE.md signal-integrity trap). Root: `\verbatim@start` baked the element-open into the line pump; raw write-pumps (moreverb) never close it. Fix: env macros own the element, `\verbatim@start` is a pure pump (real verbatim.sty division of labor). elsdoc→0. |
| P7 | dijkstra template gap | **DONE b33** | Root: template reader never expanded macro-valued column ops; real `\@mkpream` edef-expands the whole preamble. Fix: expand unprotected expandables in the fallback arm (protected filter = `\@unexpandable@protect` analogue). |
| P8 | Post-undefined digestion loops | **DONE b34 (family a + memoir ticket)** | Family (a): `read_until` returns distinguishable EOF (`Option`, Perl's undef) and `Until:` params raise the per-iteration Missing-argument Error — the TooManyErrors latch now ends zero-progress delimited scans naming the loop (willowtreebook 511+F→8 with the memoir line-capture). Families (b)/(c) correctly stay on the cycle guard (a surpass — Perl hangs). Open ticket: ulem internals for xeCJKfntef (fixdif). |

| P9 | atableau storm (795+206, 1 doc) | **DONE b40** | Two roots: (1) NewTCBListing/DeclareTCBListing lacked the real leading [init-options] optional (tcblistingscore.code.tex:329) — the `{}` args grabbed `[`,`u`,`s` and the options body digested raw (misdefined:# storm); same class as the plain \newtcblisting fix ten lines above it. (2) atableau warn-stub blocked raw interpretation of a package that raw-loads CLEAN. 1001→0. Residual: `s` star specifier not expressible via \lstnewenvironment. |
| P10 | celestia orphan-\fi (30/14) | **DONE b40** | Root: undefined \newif conditionals inside SKIPPED branches are invisible to the meaning-counting body skipper (tex.web §366; skipper is CORRECT — don't touch it), so their \fi closes the outer frame early. Fix: beamer's full 66-name \newif surface with real initial states (blocks/ams/amssymb/keywords/inpresentation/inlecture/notesnormals/suppressreplacements/theme@subsection true). Demos 2→0. jlreq sub-family (9 docs, \iftombow) NOT fixed: pLaTeX kernel newif = engine-probe risk, D9-parked. |
| P11 | amsldoc `unexpected:_` family (124/13 → 3 docs this root) | **DONE b41** | Root: amsldoc.cls makes \arg doc-markup; real amsopn.sty L56-89 unconditionally re-asserts all 34 log-like operators; our binding (and Perl's — SHARED) relied on kernel defs. Fix: binding restates the math_common operator table (kept in sync by comment). amsldoc 101→0, it/vn 101→2 (residual: \bslchar chardef rejected by relational-token reader — open). Other 10 docs = distinct Anonymous-String family, untriaged. |
| P12 | beamer-sectioning malformed:ltx (185/40) | **DONE b41** | Root: DefEnvironment-installed bare \frame opens the _noautoclose subsection and waits for an \end{frame} that never comes; `\frame{content}` command form then swallows all later sectioning. Fix: `\frame OptionalAngled [][] {}` macro routes through the env. Min-repro `\section{S}\frame{f}\section{T}` 3→0; beamerauxtheme 16→0. Faithful ltx:slide/slidesequence model remains the tracked follow-up. |
| P13 | curve2e/dsptricks Pair/Match (15/2) | **DONE b40** | (A) curve2e stub lacked \ver@curve2e.sty → \GetFileInfo delimited scan ran away, poisoning the doc (88+F→41; rule: ANY stub replacing a raw .sty must register ver@<file> WITH the ` v.` pattern). (B) \pscircle pair is optional in pstricks (Perl ZeroPSCoord) — OptionalPair. Residual 41 = stub's 34 missing exports (full-fidelity path: fix the \the\edef raw-load gap, delete stub). |
| P14 | grab-bag singles (agF) | PARTIAL b40 (assoccnt deps, biblatex datamodel) | Remaining FINAL: incgraph \tcbusetemp/\dispListing temp write + `listing file=` via the VFS (13); colorblind pgfmath `array(list,i)` real implementation (17; Perl silently no-ops); couleurs-fr color-name key symmetric normalization (\lx@applyaccent leak); albi svg CaptureBlock→svg wrap (10). Recorded: abntexto \csstring = Lua-absorb tier (probe caveat); askmaps/iwonamath \mathversion = PERL-PARITY, surpass needs approval. |
| P15 | everyeof wiring design (agG) | **FINAL — engine batch** | Loop mechanism SOLVED: 5-token window = l3tl \__tl_replace_next:w recursion whose search pattern was POISONED by our alignment column-after program — the marked-mouth cross in read_token is unbounded, so a marker miss escapes into the live cell stream (l3doc typesets function names inside tabular). Design: keep once-only latch; move the cross into read_until/read_balanced bounded by the inserted payload token count; miss at payload end → today's quiet None; gate insertion on !forced. Dead-end (3× tried): unbounded read_token-level crossing, both Until policies. Test plan in agent report (exactness, alignment non-poisoning, spath3/litetable/zref loop regression, l3doc sweep bar). |
| P16 | SHARED-failure surpass candidates (agB) — NEEDS USER APPROVAL | DRAFT | (i) svg-verb (68/2): \verb's before_construct force-opens ltx:p inside svg:g where foreignObject would be legal — skip the p-open when it cannot be contained in an SVG context (Perl identical, latex_constructs.pool.ltxml:1844). (ii) margin-caption (148/7): tufte marginfigure caption in ltx:text → insert_block ltx:block fallback rejects caption; wrap as figure instead (Perl identical). Both pdflatex-clean. |

## Architectural queue — principled abstractions over per-package scanners

User directive (2026-09-01): no stopgap guards / one-off defensive logic;
model the kernel's underlying mechanics generally. Assessment of the
session's landed shapes against that bar, and the generalizations owed:

1. **TeX file I/O as a virtual file store (HIGH).** The `{name}_contents`
   cache is already a de-facto VFS, but it has FOUR ad-hoc writers
   (filecontents env, fancyvrb VerbatimOut, fancybox VerbatimOut, memoir
   writeverbatim line-capture) and ad-hoc readers (verbatiminput cache
   check, find_file slurp path). The kernel mechanics being modeled are
   exactly `\openout`/`\write`/`\closeout` → `\input`/`\openin`/
   `\IfFileExists` round-trips. The general abstraction: one virtual
   file-store module (latexml_core) that ALL \write-to-stream output lands
   in and ALL file reads consult first; verbatim WRITING environments
   become one shared "raw-line capture until end-marker (with
   `\VerbatimEnvironment`-style env redirect)" facility parameterized by
   terminator + sink. Retires the three duplicated scanners and makes
   every future write-out/read-back package (dry.sty, answers.sty,
   exercisebank, tutodoc "examples") work without per-package code.
2. **Beamer template/option execution (MEDIUM).** The color model now
   mirrors beamerbasecolor's mechanics; templates and `\DeclareOptionBeamer`
   remain absorbing no-ops. General model = actually storing template
   bodies + executing the beamer option processor against declared keys.
3. **`\iffontchar` truth (LOW).** Currently always-true (args faithfully
   consumed). General model needs per-font glyph coverage (TFM bc/ec +
   existence) in the metrics layer.
4. **`\everyeof` + artificial mouth ends (MEDIUM-HIGH).** The eTeX
   mechanism is implemented (MouthRuntime::insert_everyeof, close-time
   token-identity insertion, read_token transparency for marked mouths)
   but UNWIRED: enabling it makes `\tl_set_rescan` exact (stop quarks
   delivered) yet loops spath3/litetable/zref in 5-token expansion cycles
   — an unresolved interaction to root-cause (suspect: repeated insertion
   or crossing order vs l3tl's grouped everyeof discipline). Related
   insight: our isolated argument mouths create ARTIFICIAL EOFs that real
   TeX doesn't have — the deep model is fewer isolated mouths, not more
   EOF patches.
5. **expl3 file-boundary state (MEDIUM-HIGH).** batch 31 narrowed the
   ad-hoc exit-Off to kernel-managed frames; the principled model is
   routing ALL load boundaries through the dump's real `\@pushfilename`/
   `\@popfilename` (their expl3 hooks) and deleting the flag machinery
   (CLUSTERS ctex part-2 row).

## Standing execution queue (main session)

1. Execute FINAL plans in ascending risk order; batch 3-5 per suite run
   (feedback_batch_fixes_parallel_rootcause).
2. Every executed plan: guard test or witness re-run + LEDGER batch row.
3. Sweep after each 2-3 batches banks the corpus effect.

## DONE

(moves here with batch number)
