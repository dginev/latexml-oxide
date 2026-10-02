# Perfect Kernel — improvement-plans ledger (living)

Execution-ready plans with file:line root causes. Ranking is not done here: `docs/PERFECT_KERNEL.md` "Goal bar and
ranked path" and the sweep scoreboard (S3 missing words → schema-invalid → timeouts, user 2026-10-01) pick the next
batch; a plan here is taken when an in-scope manual needs it. A row leaves for the archive when it lands or its
witnesses are clean or out of scope (closed rows: [`archive/PLANS_DONE_PHASE56_2026-09-27.md`](archive/PLANS_DONE_PHASE56_2026-09-27.md),
[`archive/PLANS_CLOSED_2026-10-02.md`](archive/PLANS_CLOSED_2026-10-02.md) — P-numbers are cited from code, keep them).

## Open plans

| # | Target (mass) | Status | Plan summary |
|---|---|---|---|
| P72 | skak `\mainline`/xskak `\xskakloop` move parser under beamer `\only<…>` (chessboard_and_beamer: in scope, Fatal `Mouth:EoF Until:.` after `\errmessage mainline: black, not white, to move (e4)`; xskak_and_beamer recall 18.2 %, s134) | RULED | beamer evaluates every `\only<n>` overlay body in one pass (beamer_cls.rs:543 filters mode specs only; Perl's overlay machinery is unported), so skak's game state leaks between overlays and `\EatNumberA`'s `Until:.` runs off EOF. Repros `expl3/s41msg_chessboard_beamer_only_gamestate.tex` (RED), `singletons/chessboard_beamer_overlay.tex` (KNOWN_PARITY_GAP); DIFFICULT_CASES §D14. RULED 2026-09-10 (user: keep Perl's single pass) — an accepted residual (`accepted_residuals.tsv`); not scheduled. |
| P35 | w3J-4 `\Block{i-j}<fmt>{content}` angle optional (nicematrix.tex:555, 821; 0 errors, garbled content) | FINAL → b48 | RUST-ONLY. Add `D<>{}` slot via `gullet.rs:2298 read_optional_angled`, discard (formatting). LOW, content-only gain. |
| P30 | verify only | OPEN | A RED repro (`\newenvironment{example}{A}{B}` under an unbound class) to check that 59g's yielding OmniBus (DIVERGENCES #408) also beats the combined `\begin{example}` stub (omnibus_cls.rs:554-573); no in-scope witness. |
| P16 | SHARED-failure surpass residue (no in-scope witness) | DRAFT | (i) `\verb` force-opens `ltx:p` inside `svg:g` — gate the p-open on `is_openable("ltx:p")` (Perl Document.pm:833); (ix) raw-class `\fontsize`/`\@setfontsize` model (KOMA `scrsize11pt.clo`; typearea "Bad type area settings!"; 54v/58c may cover it — verify); (x) #175's `malformed` warning for unknown section types; (xi) `\markright`/`\markboth`/`\pagenumbering` as macros if a raw-class witness appears; upstream filings for #175 / KPE #118-#121 not done. Landed parts in the archive. |

**Settled, do not re-attempt** (STOP rows, archived with the landed ones):
- P55 yquant-doc after unmasking (200 errors) — STOP: SHARED/out-of-scope
- P71 pagelayout's page canvas (`\page{…}` wraps the whole page in a `picture`; pagelayout.cls:3116/3154) — STOP: SHARED, HIGH risk
- P75 physics2 ×2 (lualatex: `\the0` from unicode-math's `\g__um_main_font_defined_bool` and a 95-line `\egroup` cascade) — STOP: needs unicode-math math support
- kblocks/kblocks-doc 0→97 after unmasking — STOP: SHARED
- P61 — do NOT revert pgfmath `divideround`: the error storms it removed were what made liftarm, wheelchart and
  twoxtwogame "finish" early.
- P62 (RUST-ONLY, CJK) — an `\if` operand read as charcode 256 after ltjsarticle's leaked frame (gullet.rs
  `read_x_token` → tex_logic.rs:30-43); only luatexja triggers it — reopen on a pdfTeX witness.

## Architectural queue

1. **Virtual file store and expl3 file boundaries** — owned by KERNEL_CAPABILITIES K7 and ARCHITECTURE_THEMES §6.
2. **Beamer templates/options (MEDIUM).** Theme options run the real `\ProcessOptionsBeamer` (55c); template and
   overlay hooks partly (55s); overlays are still not acted on (Perl's `ltx_covered`, #270). beamertheme-mirage-doc
   is out of scope (CJK clause); find an in-scope witness before scheduling.
3. **`\iffontchar`** — LANDED 56x (per-font coverage, etex.rs:35/:565).
4. **`\everyeof`** — wired for file and `\scantokens` mouths (b51, gullet.rs `insert_everyeof`). Open: the artificial
   EOFs of isolated argument mouths (ARCHITECTURE_THEMES §4b); spath3/litetable/zref 5-token loops were the
   crossing-order bug.

## Standing items (numbers are cited from code)

5. **Guard strength (user 2026-09-17).** Assertions capture the whole element or a line golden, never a substring.
   Census 2026-09-17 (`~/data/pk_agents/w23/test_audit/` REPORT.md + inventory.tsv): 2,867 tests, A 421, B 1,508,
   C 938 (520 on XML output, 371 of them in `cluster_package_guards`; 418 typed-value unit predicates). B1's
   `assert_element` helper LANDED 2026-09-17 (8d6606ded2, `latexml_oxide/src/util/test.rs:782`); remaining batches
   B5 (40 error-count-only guards), B2-B4, unit predicates last; re-count before the next batch. No test is deleted.
7. **pgf/TikZ throughput (user 2026-09-18).** Landed levers 56da-56du (pgfkeys native slices 56dk/56dl/56dp, datatool
   native 56dt/56du: tikz-network manual 190.7 → 25.0 s vs pdflatex 34.5 s) and 59a/59c (pgf-interference-en/-de
   220/212 → 166/166 s; wheelchart 210 s Fatal → 118 s with 59b's pgfmath fixes, KPE #420/#421); history in the
   archive. Rule: algorithmic and strategic levers only; levers B and G reverted as measured losses; `.try` < 0.5 % of
   wall (declined). Next: the audit's L1 (`macro_call` pstack path) and L6 (`\expandafter`)
   (`~/data/pk_agents/main/agent_reports/2026-10-01_tikz_perf_audit.md`); pgf-interference-en is the in-scope case
   nearest the 180 s ceiling.
9. **`\\` under a corrupted alignment template (align-state guard)** — PARKED: the witness plextdelarray (Rust 27 vs
   Perl 10 `Extra alignment tab`) is a pLaTeX manual, out of scope. Rust `Let!`s `\@arraycr` to
   `\lx@alignment@newline` whose `inside_cell_group()` heuristic misfires (tex_tables.rs; latex.ltx:16583-16594 is the
   real guard); fix = model the align-state guard, HIGH risk (1610.00974, tabularray/ProfSio in-cell `\\`,
   kbordermatrix). Side find to verify: plain `delarray` Rust 0 / Perl 3 errors
   (`~/data/pk_agents/w23/regr96/aligntab/da.tex`).
10. **tcolorbox manual RSS — SETTLED (2026-09-19, `~/data/pk_agents/w23/perf_pgf/tcb_rss/`):** a heavy outlier, not a
    leak — 37.8 GB uncapped (349 s, 198,801 `svg:g`), the whole digested box forest pinned by `node_boxes` (SHARED
    with Perl's per-node box); the manual is an out-of-scope canary (timeout at 180 s). Levers if it ever matters:
    yield seams inside `breakable`/poster digestion (MED-HIGH, with item 11), eager `node_boxes` release (HIGH).
11. **Memory: the whole-document box tree is the peak.** LANDED: picture-end seam 56dv (guard
    `112_fragment_yield::a_picture_end_is_a_seam_request`), streaming restart of a fused eager run 56dw (CLI) and 56ea
    (`cortex_worker`, `streaming_restart.rs`), `remove_node` frees 56dx, pre-fuse watermark 56dz (nine tenths of the
    fuse). Open (fleet, not manuals): wrap each paper's convert in the daemon frame (`state.rs` `push_daemon_frame`,
    Perl State.pm:607-627; MED-HIGH); eager-path `node_boxes` pruning (Document.pm:1667-1669; ~340 MB on
    pgf-PeriodicTableManual; LOW). Dead ends: fewer `svg:g` (already 0.34× pdflatex), draining during Build, a leaner
    `Whatsit` (#361 M4).
11a. **Sandbox reruns 2605/2606 as the regression oracle** — DONE 2026-09-19 (baselines 2605: 30,079 docs, fatal 258;
    2606: 30,426, fatal 237; its regressions became item 13); recurring as gate L6 (PERFECT_KERNEL.md).
12. **High-performance raw interpretation (user 2026-09-19).** pdfTeX's advantage is tex.web's data model (packed
    tokens, in-place input levels, `eqtb` + save stack). Candidate costs here: interned-symbol lookups, expansion that
    clones token vectors, ~2.4 KB per `Rc<DigestedData>` box (Perl 0.7 KB). 59a (number scanning per tex.web
    §440/§448, typed `if_stack`) and 59c (parameter name-kind cache) are the first such levers. Method: `--profile
    bench` profile on a token-churning witness, one lever per run with pre-registered bars.
    **Interface constraint (user 2026-09-19):** this codebase will be maintained by
    LaTeXML developers who know the Perl Gullet/Mouth/Stomach/State abstractions and
    ergonomics — any representation change (packed tokens, in-place expansion, hot
    state) must stay behind the Perl-shaped interfaces (`readToken`/`readXToken`,
    `unread`, `Tokens`, `lookupMeaning`/`assignValue`, mouth push/pop) so the code still
    reads like the Perl to them; a faster core is not allowed to turn the gullet into
    something a Perl LaTeXML developer would not recognize. This is the only remaining
    throughput lever with corpus reach (P5's successor).
13. **Sandbox-regression lessons as kernel capabilities (2026-09-19; notes
    `~/data/pk_agents/w23/regress_2605/{CLUSTERS,KERNEL_LESSONS}.md`).** LANDED: (e) `try_spawn_or_degrade` (56ec,
    `latexml_core::util::thread`); (g) `\g@addto@macro` as `DefPrimitive`+`AddToMacro!`, restoring Perl's guard
    (56ed; ~6 papers' `recursion:\normalsize`). OPEN (arXiv fleet): (a) driver-probe sentinels define their whole
    family (hyperref `\hyper@makecurrent`); (b) defining primitives define a stub target (`\newbibmacro` is still a
    no-op, biblatex_sty.rs; biblatex-ieee, 53 papers); (c) beyond-Perl `Until:` captures need an EOF-recovery reader
    (the jmlr `\addr` `\let` attempt collapsed colt 2→1, midl 7→1 — dead end); (d) OmniBus recovers `\Xauthor`
    wrappers (l4dc2026 2605.22207, neus2025 2605.05795); (f) raw-loader bindings keep a `\providecommand` fallback
    (tipa; colt2024_cls.rs). Lesson (g): a raw-TeX shortcut must keep the Perl guard the native binding carried.

## DONE

The landed, executed and stopped plans of phase 56 are in [`archive/PLANS_DONE_PHASE56_2026-09-27.md`](archive/PLANS_DONE_PHASE56_2026-09-27.md),
the rows and items closed on 2026-10-02 in [`archive/PLANS_CLOSED_2026-10-02.md`](archive/PLANS_CLOSED_2026-10-02.md),
with their P-numbers.
