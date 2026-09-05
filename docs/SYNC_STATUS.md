# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

## How to read this file

**Start at "Ranked worklist" below and take the top unblocked row.** That is the
whole intent of this file; everything after it is supporting detail.

| section | what it is | when you read it |
|---|---|---|
| **Ranked worklist** | every open item, ordered, with size + where the detail lives | **first, always** |
| Current status | suite count, the last session, release state | to orient |
| Open items | the detail behind the ranked rows | when you pick that row |
| Standing policies | rules that constrain *how* you fix things | before adding a CLI flag, a stub, or a divergence |
| Parked families | pointers to extracted docs | only when starting that family |
| Reference | stable facts, not active work | when something surprises you |

Three rules that keep this file honest:

1. **Verify a status label before acting on it — and before deleting it.** Check the
   **named guard test** in the tree, or `gh issue view <N>` / `gh pr view <N>`.
   **SHA-ancestry does not work** as a check — the repo squash-merges, so a branch
   SHA quoted here is never an ancestor of `main`.
2. **This is the BRIEF ACTIONABLE LIST.** Day-by-day logs live in `git log` and
   `docs/archive/`. When you close an item, delete it here and lift anything
   worth re-reading into `docs/archive/SYNC_SESSIONS_YYYY-MM.md`.
3. **Keep it under ~500 lines.** When a section outgrows ~100 lines it has become
   its own subject — give it a doc under `docs/` and leave a one-line pointer.

*Last compaction: 2026-09-03 — 949 → ~320 lines. Lifted non-actionable upstream review R1, R9-BST historical narrative, ltx_env design notes, and m:menclose/glossaryref parity analyses to `archive/SYNC_SESSIONS_2026-09.md` and dedicated docs. Elevated high-impact fatal seeds (2605.22927/2606.11121 P0/R101 flood) and unblocking class fixes (sn-jnl.cls) to ranked active list. Prior: 2026-09-03 (1276 → 953 lines); 2026-08-18 (1462 → ~890 lines); 2026-07-25 (1979 → ~500 lines).*

---

## Ranked worklist — start here

Ordered by: **does it reproduce today** → **is a real user affected** → **is it unblocked** → **effort**.
High-impact fatal seeds and major publisher class fixes take priority.

| # | item | state | size | detail |
|---|---|---|---|---|
| **R1** | **Fatal-Seed: Perl-0 vs Rust-101 Error Floods** (`2605.22927`, `2606.11121`) | **OPEN**, fresh seed from rc4 60k run. Hits `TooManyErrors:MaxLimit(100)` fatal abort in Rust | medium | Open items §R1 |
| **R2** | **Springer Nature `sn-jnl.cls` Dependency Drop** (witness `2606.00121`) | **OPEN**; raw-load drops `\usepackage{booktabs}` and `\usepackage[title]{appendix}` in `content.rs:2429` | small-medium | Open items §R2 |
| **R3** | **Bibliography-absence campaign** (PR #444) — **291 recovered / 20 338 entries**. Remaining unblocked: **R3d tab-mark parameter scan vs cell read** | **R3d next** (12 papers left, unblocks alignment macro `&` splits) | medium | Open items §R3, [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md) |
| **R4** | `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`) | **OPEN**, re-verified (1 error with `--preload=article.cls`, 0 without). Pool load reordering | medium | Open items §R4 |
| **R5** | **Physical In-Place Image Cropping (`trim`/`clip`)** | **OPEN**, witness `2510.17772` Fig 7. Image metadata scaled but raster uncropped | small | Open items §R5 |
| **R6** | **`Collector::rescan` Refactor for Generated Backmatter** | **OPEN**; `Scan` owns `ObjectDB` by value; generated backmatter lacks full relations/labels | medium | Open items §R6 |
| **R7** | Presentation-MathML **F5** Linebreaker | **OPEN**, full linebreaker feature gap needing a port-or-drop scope decision | family | Open items §R7 |
| **R8** | **Generalized kernel-capability program** (branch `perfect_kernel`; user-approved 2026-09-05) — K1 definition provenance + overlay bindings, K3 lthooks store, K4 templates/sockets, K5 raw-line reader, K2 nest vs save stack (= R9), K6 font model, K7 file model, K8 runaway cap | **OPEN**, seeds landed in batch 56i | program | [`perfect_kernel/KERNEL_CAPABILITIES.md`](perfect_kernel/KERNEL_CAPABILITIES.md) |
| **R8b** | **forest.sty full support** (side goal, user 2026-09-05; heavily used on arXiv) — discard stub today; overlay-binding vs native-tree shape to be decided | **OPEN**, recorded | large | [`perfect_kernel/DIFFICULT_CASES.md` §D10](perfect_kernel/DIFFICULT_CASES.md) |

---

## Current status

- **2026-08-02 — rc4-recut full rerun of sandbox-arxiv-2605+2606 (60,505 docs):**
  - **Overall:** no_problem 6,078/6,359 · warning 19,744/19,724 · error 3,991/4,102 · fatal 266/241.
  - **Fatal clusters:**
    | cluster | size | verdict |
    |---|---|---|
    | `panic:caught` | 3 | **FIXED (PR #491)** — pooled-worker math parser `PENDING_DISCARDS` stale handle sweep on abort. |
    | `TooManyErrors:MaxLimit(100)` | 117 | **REAL seed** — 4/8 sampled REAL, led by **2605.22927 & 2606.11121** (Perl 0 vs Rust 101-flood). |
    | `Stomach:Recursion` | 55 | **MIXED** — 3/8 REAL-by-count (`2605.17696` R144/P56, `2606.05321` R35/P15, `2606.08524` R94/P50). |
    | `Timeout:PushbackLimit` | 120 | Environmental/budget caps, not conversion bugs. |
    | `Timeout:TokenLimit` | 88 | Performance ceiling; legitimate heavy papers. |

---

## Open items — detail for the ranked rows

### R1 — Fatal-Seed: Perl-0 vs Rust-101 Error Floods (`2605.22927`, `2606.11121`)
- **Symptom:** In the 60,505-paper rerun, 117 papers hit `Fatal:TooManyErrors:MaxLimit(100)`. On `2605.22927` and `2606.11121`, Perl converts cleanly with **0 errors**, while Rust cascades past 100 errors and fatally aborts (also `2606.01136` P63/R101, `2605.10685` P7/R101).
- **Action:** Bisect each paper with `latexml --verbose` to identify the initial diverging token/macro. Fixing these primary triggers will recover multiple papers from fatal abortion.

### R2 — Springer Nature `sn-jnl.cls` Dependency Drop (witness `2606.00121`)
- **Symptom:** Springer Nature's standard class `sn-jnl.cls` (1765 lines) raw-loads but drops `\usepackage{booktabs}` (:307) and `\usepackage[title]{appendix}` (:303), causing undefined `\toprule`/`\midrule`/`\bottomrule` cascades.
- **Root Cause:** `maybe_require_dependencies` in [`latexml_core/src/binding/content.rs:2429`](latexml_core/src/binding/content.rs#L2429) fails to extract or load dependencies declared mid-class during raw interpretation.
- **Action:** Trace dependency extraction in `content.rs` and ensure required packages are loaded.

### R3 — Bibliography-Absence Campaign (PR #444 Residuals)
- **R3d: Alignment Parameter Scan vs Cell Read Distinction (`suppressed_tab_marks`):**
  - *Symptom:* An unescaped `&` inside a delimiter-fenced macro argument splits the alignment row and truncates the document (and bibliography). 12 papers remain affected.
  - *Mechanism:* `tex.web` §394 `macro_call` suppresses tab marks while scanning parameters. `SuppressedTabMarks` in [`latexml_core/src/common/local_assignments.rs:194`](latexml_core/src/common/local_assignments.rs#L194) fixed `physics.sty`'s `\mqty` (14 papers), but applying it globally to `Parameters::read_arguments` regressed 5 tests (`cells_test`, `numprints_test`, `xytest_test`, `consort_flowchart_test`, `unit_tests_by_silviu_test`) because that path also reads alignment cell content.
  - *Action:* Distinguish macro parameter scanning from alignment cell reading in `Parameters::read_arguments` so tab marks inside `{...}` do not split outer cells.
- **R3b: No-Diagnostic Chase Candidates (~6 left):**
  - Remaining papers with real `\cite` calls but empty bibliography: `2605.14990`, `2606.05629` (math-in-body silent drop), `2606.10056`, `2606.17491`, `2606.00231`, `2605.29754`.
- **R3g: amsrefs Bare `\begin{biblist}` (4 papers):**
  - `\begin{biblist}` without `{bibdiv}` wrapper $\to$ `malformed:ltx:biblist`. Requires `BACKMATTER_ELEMENT` route.

### R4 — `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`)
- **Symptom:** `--preload=article.cls` prints `LaTeX hooks Error: Extra \PopDefaultHookLabel` (clean without preload or with `LATEXML_NODUMP=1`).
- **Mechanism:** `\@pushfilename` changes meaning mid-load: `article` is pushed before `LaTeX.pool` loads, using a pre-pool `\@pushfilename` that does not touch `\g__hook_name_stack_seq`. The pool installs expl3's `\@popfilename`, which pops an empty seq and errors.
- **Resolution:** A TeX-side repair or re-synchronizing the sequence at the point `LoadPool('LaTeX')` executes.

### R5 — Physical In-Place Image Cropping (`trim` / `clip`)
- **Symptom:** In `\includegraphics[trim=..., clip]`, [`latexml_core/src/util/image.rs:433`](latexml_core/src/util/image.rs#L433) adjusts metadata dimensions but keeps the original uncropped image, causing browsers to squish the entire raster into the sub-box (witness `2510.17772` Fig 7).
- **Resolution:** Implement `crop_image_inplace` in [`latexml_post/src/graphics.rs`](latexml_post/src/graphics.rs) alongside `rotate_image_inplace` (:771) using `convert -crop`.

### R6 — `Collector::rescan` Refactor for Generated Backmatter
- **Symptom:** Generated subtrees (Bibliographies, Indexes, Glossaries) lose ObjectDB relations, labels, and fragids because `Scan` owns `ObjectDB` by value (`latexml_post/src/scan.rs:49`) and cannot rescan generated nodes.
- **Resolution:** Refactor `Scan` to borrow or take/restore `ObjectDB`, enabling clean rescanning of generated nodes and removing fragile ad-hoc registrations.

### R7 — Presentation-MathML F5 Linebreaker
- **Status:** The only remaining pMML gap from the MathML line audit (F17 is closed). A full linebreaker algorithm gap requiring a port-or-drop scope decision before coding.

---

## Secondary residuals & unranked active items

### Font-Selection Chain Residuals
1. **`\cal ABC` collapses to one `<mi>`**: Drops `class="ltx_font_mathcaligraphic"` (Perl emits three `<mi>` elements, Rust one containing `𝒜ℬ𝒞`). Token grouping and class diverge.
2. **`\DeclareTextCommand`/`\ProvideTextCommand` lack encoding-dispatch chain**: `latex_constructs.rs:6525/6544` bind bare `\cs` to first-encoding expansion permanently.
3. **Minor font registration gaps**: `\DeclareTextSymbol` decodes eagerly instead of installing deferred `CharDef`; `\DeclareErrorFont` is a bare no-op where Perl defines argument as `\relax`.

### Fragid Parity Open Items
1. **`associateNode` (Post.pm L508-585) unported**: Generated MathML/OpenMath nodes carry no `xml:id`, so `convertedIDs` and pmml↔cmml parallel cross-linking do not exist.
2. **`in_page_id` lacks `labelids` and `split_from_id` branches**: Affects `--splitnaming=label*`.
3. **`strip_ref_display_fragids`** (crossref.rs:131): Matches `//ltx:ref//*[@fragid]` wholesale; narrow to IDs absent from the ObjectDB.
4. **`make_sub_collection_documents` returns `vec![]`** (collector.rs:141): `--splitindex`/`--splitbibliography` drop entries past first initial.

### Corpus Triage Quick Wins
- **`newunicodechar` four-hex `^^^^` caret support** (~119 docs, e.g. `2606.00241`): Adding 4-hex/6-hex caret parsing to `mouth.rs:get_next_char` allows `newunicodechar` to take its Unicode branch cleanly.
- **`floatrow` raw-load (witness `2606.10047`)**: Floatrow reroutes subcaption placement, causing 18 `malformed` errors in Rust vs 0 in Perl.

---

## Standing policies & method

### Methodology & the cortex cross-join
- Working method: **re-triage LARGE-error papers** (the single-error tail is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose` → fix the divergence.
- Cortex API: `http://127.0.0.1:8000/api`. Endpoints: `/api/reports/<corpus>/oxidized-tex-to-html/<severity>` and `/api/corpus/<corpus>/tex_to_html/document/<id>`.

### CSS themes — `ar5iv.css` vs `LaTeXML.css`
- `ar5iv.css` is actively developed (`~/git/ar5iv-css`). Base `LaTeXML.css` is a faithful copy of upstream Perl LaTeXML's default theme; bugs in base CSS route upstream to `brucemiller/LaTeXML`.
- Sizing intent gap (witness #721, ar5iv#83): Preserve absolute (`7in`) vs relative (`\textwidth`) intent in `image.rs:186` instead of flattening both to `pt`.

### Algorithm Markup + CSS Unification (0.7.7 Target)
- Part 2 landed (line numbers, captions, CSS mirrors).
- Shared markup class: Assign every algorithm listing a shared marker class (regardless of surrounding env) so one generic CSS rule targets it in both `LaTeXML.css` and `ar5iv.css`. Details in [`parity/ALGORITHM_RENDERING.md`](parity/ALGORITHM_RENDERING.md).

### CLI options policy (Option-C) + `validate()`
- Wire only options whose engine feature works end-to-end; strict clap parser (no accept-and-warn stubs).
- `--validate`: STUB today (`latexml_post/src/document.rs:1717`). Requires safe RelaxNG bindings in `rust-libxml` published to crates.io before wiring.

---

## Parked families — pointers to dedicated docs

| family | doc |
|---|---|
| Environment markup class (`ltx_env_<name>`) | [`parity/ENV_MARKUP_DESIGN.md`](parity/ENV_MARKUP_DESIGN.md) (Phase 2 / Post-Release) |
| Beyond-Perl performance levers (BP-1…BP-6) | [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) |
| Content-MathML & math parser gaps | [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) |
| Deep deferred families (`.bst`, xy-pic, etc.) | [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) |
| Stage 4 WASM bring-up plan | [`release/WASM_COMPATIBILITY_PLAN.md`](release/WASM_COMPATIBILITY_PLAN.md) |
| Streaming core DOM design | [`performance/STREAMING_CORE_DESIGN_2026-07-29.md`](performance/STREAMING_CORE_DESIGN_2026-07-29.md) |
| Two-pass streaming split | [`performance/STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) |
| Multi-document streaming post-join | [`performance/MULTIDOC_JOIN.md`](performance/MULTIDOC_JOIN.md) |

---

## Reference & stable notes

- **SVG picture path:** Post SVG handling for `<ltx:picture>` in `post.rs` splices SVG strings into placeholders after XSLT. A DOM-based port (`latexml_post::svg::SVG`) exists and will replace the string splice once `rust-libxml`'s `PostDocument` subtree insertion cleanup is stabilized.
- **Picture `\unitlength` sizing:** Inkscape `.pdf_tex` pictures ignore `\unitlength` during core `{picture}` sizing, producing degenerate outer SVGs (`DEGENERATE_SVG_PX = 4.0`).
- **Primitive layer:** Audited faithful (2026-06-20); core arithmetic, glue, conditionals, and token tables match Perl byte-for-byte.
