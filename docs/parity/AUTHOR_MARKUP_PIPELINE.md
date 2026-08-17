# Author-markup pipeline — unification worklist

> **Status: in progress (design + witness collection).** Living worklist for
> replacing the two-branch `\lx@add@authors` author/affiliation parser with a
> single line-first pipeline. Not a frozen diagnostic — take the top open row.
> Owner-directed 2026-08-16. Related: `OXIDIZED_DESIGN_DIVERGENCES.md` #52.

## Why

`latexml_engine/src/base_utilities.rs::\lx@add@authors` (the `DefMacro!("\\lx@add@authors{}", …)`
at ~:883) recovers structured creators from the abused `\author{…}` idiom. It runs
**two different algorithms**, picked by whether the block contains a superscript
(`position_of(stuff, authorsup_markers())`, markers = `^` / `\textsuperscript`):

- **Marker branch** (~:901-960): flatten-split on `\\`+`\quad`+`\and` together
  (`author_affil_splits`), classify each fragment by **superscript position**
  (`name_precedes_marker` → author, else affiliation).
- **No-marker branch** (~:971-1003): split on `\quad`+`\and` **only** into groups
  (`author_group_splits`, *no* `\\`), then split each group on `\\`; **first piece =
  names, rest = affiliations** (order-based).

The author↔affiliation decision uses **two different signals** (superscript position
vs `\\`-piece order) over **two different split orders**. Every recent frontmatter
fix patches one branch: #6614 `name_precedes_marker` (marker), the trailing-`\quad\\`
empty-piece drop (no-marker, #52(d)), #52(a)/(b) `\thanks`→affil / comma-not-a-split
(no-marker). Three heuristics drifting apart.

## Target: one line-first pipeline

Same entry point, same emitted `\lx@add@author` / `\lx@add@affiliation` /
`\lx@add@email` calls (blast radius = one function):

1. **Segment into lines** — split the whole block on `\\` **first** (the true row
   separator; `\\[len]` optionals already consumed at :892).
2. **Classify each line** with one classifier over all signals: email-list → email;
   marker-led / institution-shaped → affiliation; else → author-names. Superscript
   position becomes one input, not a separate algorithm.
3. **Split author-name lines into creators** (`split_author_line`: comma / " and " /
   `\quad` / `\and`); fold the whole-line `\textbf{…}` unwrap in here.
4. **Attach contacts** — affiliation/email lines to the right authors; a lone trailing
   affiliation after a multi-line author list attaches to **all** authors (shared),
   not just the last.

## De-risking

Green-parallel: run the new pipeline beside the old two-branch, diff the **emitted
calls** over the whole witness corpus (below) + a real-arXiv frontmatter sweep, cut
over only when every diff is understood witness-by-witness. Land **shared-affiliation**
as its own step (behaviour change → own goldens, likely a divergence entry). This is a
surpass of an already-surpass heuristic (#52) tuned against 30 years of arXiv abuse →
surpass protocol + sign-off before cutover.

## Witness corpus (current baseline)

22 curated fixtures in `latexml_oxide/tests/cluster_regressions/frontmatter_*.tex`,
all convert with 0 errors. `✓` = clean markup today; `✗` = defect the pipeline must
fix; `?` = judgement call. Baseline re-captured by `tools/author_markup_char.py
<core.xml>` (run over each fixture's `--nopost` output).

| Witness | Idiom | Current creators | Verdict |
|---|---|---|---|
| acl_quad_authors (2606.08234) | `Name\quad… \\ ⁿAffil`, superscripts | 4 authors, affils by number | ✓ (#6614) |
| acmart_author_optarg (2405.08372) | `\author[short]{Full}` | 1, orcid+affil+email | ✓ |
| czipreprint | `\author` + `*`/orcid | 2, affils+orcid+email | ✓ |
| ieee_authorblock (2602.05517) | `\IEEEauthorblockN{1ˢᵗ Name}` | `1st Alice Smith`, `2nd Bob Jones` | ? ordinal in name |
| ieee_membership (2508.00603) | `Name, \IEEEmembership{…}, and Name…\thanks` | Alice, **EMPTY, EMPTY**, Bob, **EMPTY**::thanks | ✗ 3 phantom empty creators |
| ieee_linebreak_optarg (2605.23553) | lazy `\\[1em]` block | 3 authors; shared emails land on last author | ? email distribution |
| inst_affiliation_dedup | shared institute | 1, deduped affils | ✓ |
| jmlr_name | structured `\author` | 2, thanks+email+affil | ✓ |
| llncs_and_institutes (2605.00347) | `\institute{A \and B}`, "…and…" names | 2, affils+email (institution "and" intact) | ✓ #52 |
| llncs_lazy_superscripts | numeric superscripts | 2, affils+shared email | ✓ |
| llncs_shared_email | `{a,b}@u` shared | 2, email shared to both | ✓ (shared works here) |
| mrm | `Name*`, equal-contrib | `Jakob Asslander*` | ✗ `*` leaks into name (#52 limit) |
| sn_jnl_affil / _numbered | springer nature | 2-3, email+affil | ✓ |
| spconf_* (2606.00315 kin) | `\name`, comma addresses | 2, affils not comma-shredded | ✓ #52 |
| interspeech2024 | `\name[affiliation=…]{}` | 2 clean | ✓ |
| atlasdoc / abstract_centering / acmart_pubnotes | title/abstract, not authors | n/a | ✓ |

**Not yet in-corpus (on unmerged branches / to add):**
- multi-line author block, trailing `\quad\\` → line-2 first author lost as empty
  personname+affil (2507.06670 "Ruiqi Li"). Fix in flight (#52(d)); pipeline makes it
  free. Add `frontmatter_multiline_author_leading_break.tex` once merged.
- whole-name `\textbf{}` per-line (2308.06262, 2507.06670) → incoherent bold. Fix in
  flight (#61 / PR #615). Pipeline folds the unwrap into stage 3.
- **shared affiliation across a multi-line list** (2507.06670: "Zhejiang University"
  attaches only to the last author, not all 9) — no fix yet; pipeline stage 4 target.

## Reader-reported evidence — 200 open `front matter` issues

Triaged 2026-08-16 (all ~200 open issues under the `front matter` label,
newest→oldest). Sources fetched where the report carries a public arXiv id; ~15%
link only a private `services.arxiv.org/html/submission/…` preview (no fetchable
source). The reports converge on **eight failure families**, each mapping to a
pipeline stage. Counts are approximate (many issues span families).

| # | Family | Stage | ~N | Representative witnesses (issue → arXiv) |
|---|---|---|---|---|
| F1 | **Multi-line `\author{}` cells joined by `\\` + `\and`/`\And`/`\AND`/`\quad`** — separator leaks literally / eats the next space; names stack one-per-line or one-word-per-line; email/affil lines render inline or as fake authors. The dominant cluster and the redesign's core target. | 1–3 | ~30 | 6687→2406.07811 (arxiv.sty, 9× `\And`), 6298→2409.19467 (acl), 4841→2403.00393 (acl `\And` literal), 5851→2512.24601 (neurips), 6242→2510.02340, 5786→2601.06574, 5262→2505.07453 |
| F2 | **Superscript/numeric affiliation markers not linked, rendered unraised, or doubled** (`\inst{n}`, `$^{n}$`, `\textsuperscript{\rm n}`) — show as plain "11 22 33", never anchor to the affiliation. | 2,4 | ~12 | 6209→2407.09826 (llncs `\inst`), 4697→2507.01800 (llncs), 5159→2502.21106 (llncs), 4644→2508.14765 (neurips_2025), 5315→2309.15463 (revtex4-2), 6314→2601.07136 |
| F3 | **`\thanks`/`\footnotemark`/`\IEEEmembership` mis-segmentation** — internal `\\` in a `\thanks` fabricates phantom authors; equal/corresponding markers lost or jammed; membership between commas leaves stray `, ,` + empty creators. | 2,3,4 | ~15 | 4539→2508.00603 (IEEEtran `, ,`), 6295→2512.12923 (IEEE affil-as-author), 5881→2601.17760 (`\thanks` `\\`→phantom), 5874→2511.04594 (wrong corresp.), 6547→2405.09426 |
| F4 | **Structured/keyed author↔affiliation schemes** (class macros) — key→author mapping lost, so affils go missing / mis-attach / appear as authors. | 2,4 | ~12 | 6366→2506.08134 (ICML `\icmlauthor`+`\icmlaffiliation`), 5761→2601.03547 (elsarticle `\author[n]`+`\address[n]`+`\ead`), 6285→2604.01119 (elsarticle `\affiliation[KEY]organization=`), 6522→2603.01467 (Interspeech `\author[affiliation={n}]{first}{last}`), 5315→2309.15463 (revtex) |
| F5 | **Contact mis-attachment** — all affiliations → first author; shared affiliation shown only once; emails split into fake authors; addresses on the wrong author. | 4 | ~14 | 4877→2509.22519 (shared affil only under 1st), 5495→2512.07995 (all addrs→1st), 6291→2604.01735 (email→wrong author), 6255→2503.02656 (email line→3 authors), 5761→2601.03547 (addr swapped), 6590→2606.04947 |
| F6 | **Duplicated frontmatter (SEG)** — title/author block emitted twice (top + after abstract), disproportionately acl.sty. Block-boundary, not name parsing. | 1 | ~13 | 4807→2509.10377 (acl), 4820→2406.14673 (acl), 4932→2509.11625, 4521→2507.23776, 5332→2511.16470 (acl), 6588→2606.01317, 6493→2605.10734 |
| F7 | **Raw macro / key=value leak** — unsupported class macros surface as visible text: `\And`, `\fnm`/`\sur`, `\name`/`\email`/`\addr`, `\affiliation[KEY]organization=`, `\authormark`, `\WarningFilter`, `nation=…`, acmart journalyear. Mostly a *binding-coverage* gap, adjacent to (not solved by) the pipeline. | — | ~15 | 6231, 5762, 5802, 6200, 4522, 4323, 6010, 6169 |
| F8 | **Title contamination** — journal/DOI/CCS/"Submitted to…"/"Accepted at…" absorbed into the title. Line-classification at the title boundary. | 2 | ~7 | 6885→2608.07766, 6542→2604.24199, 6333→2604.12543, 6140→2603.04284, 5107→2510.20036 |

**Reading:** F1–F5 are the parser's job and the pipeline's core. F6 (duplicate
frontmatter) is a separate block-segmentation bug worth its own investigation. F7 is
binding coverage (per-class macro support), not the parser. F8 is a title-boundary
classification case. **Reproduce each on HEAD before fixing** — the deployed arxiv.org
binary lags, and several (e.g. 6870, 6614) are already fixed.

## Reproduction library

Verbatim author blocks from the highest-value witnesses, one+ per family, ready to
shrink into `frontmatter_*.tex` fixtures as each stage lands. `class` is the real
`\documentclass`/style.

- **F1 · 6687 · 2406.07811 · arxiv.sty** — 9 authors, each `Name \\ Affil \\ \texttt{email} \\`, `\And`-separated. Desired: 9 creators, each with its affil + email; `\And` never printed.
- **F1 · 4841 · 2403.00393 · acl** — `\author{A \And B \And C \\ \AND D \And E \\ \AND Microsoft Research India \\ \texttt{…} \\}`: `\And`/`\AND` separate names; the last two `\\` lines are a shared affiliation + shared email.
- **F3 · 4539 · 2508.00603 · IEEEtran** — `\author{Liang, Mak, \IEEEmembership{Senior Member, IEEE}, and Lee, \IEEEmembership{…} \thanks{…}\thanks{…}}`: 3 authors, membership dropped/noted, `\thanks` → affil+email contacts. **No empty creators, no `, ,`.** (Matches the local `ieee_membership` phantom-empty defect.)
- **F4 · 6366 · 2506.08134 · ICML** — `\icmlauthor{Name}{key,…}` + `\icmlaffiliation{key}{Inst}` + `\icmlcorrespondingauthor`. Desired: 5 creators, affils resolved by key, one corresponding email.
- **F4 · 5761 · 2601.03547 · elsarticle** — `\author[1]{K Wang\corref} \ead{…} \author[2]{J Hu} \ead{…} \address[1]{…} \address[2]{…}`: address[n] attaches to author[n] (currently swapped).
- **F2 · 6209 · 2407.09826 · llncs** — `\author{X\inst{1} \and Y\inst{2}\orcidlink{…} …}` + `\institute{A \and B …}`: `\inst{n}` becomes a raised link to institute n, not literal "1".
- **F5 · 4877 · 2509.22519 · article** — `\author{A\thanks{Aff-A} \and B\thanks{Aff-B} \and C\thanks{Aff-C} \and D\footnotemark[3]}`: D shares C's affiliation via `\footnotemark[3]`; affil must appear for D too.
- **F1/F5 · 6255 · 2503.02656 · googledeepmind** — `\author{A*, B, …, Z* \\ \{paulgc, zhedong\}@google.com, Google Inc.}`: the trailing `\\` line is a shared email + org, NOT three authors.
- **F6 · 4807 · 2509.10377 · acl** — title + author block rendered twice (top + after abstract); one canonical frontmatter only.
- **F2/F5 · 4644 · 2508.14765 · neurips_2025** — `\textsuperscript{\rm n}` markers, `\textbf{…}` wrapping half the block, `\thanks` with `\&`, affil lines "Merck \& Co."; the `\&`/superscript must not fragment the affiliation.

(Full fetched sources for the deep-dived issues are cached in scratchpad
`e<N>/`; the census above lists every issue by number for re-fetch.)

## R2 — reproduced on HEAD (2026-08-16)

Each representative re-converted on HEAD (`--nopost`, `tools/author_markup_char.py`)
to separate LIVE parser defects from deployed-lag (already fixed). **The core idioms
are largely fixed; the live defects are the *mixed* variants + a few class-specifics.**

| Rep | Class | HEAD result | Verdict |
|---|---|---|---|
| 4539 (F3) | IEEEtran | 6 creators, **3 empty `<personname/>`**; `\thanks` affil+email land on an empty creator | **LIVE** — R5 target |
| 6242 (F2/F5) | article | names all clean; superscript affiliations MERGE onto one author + `\texttt` email bleeds into the affiliation text | **LIVE** (affiliation attachment) |
| 4877 (F5) | article | names clean; `D\footnotemark[3]` gets only the marker, not C's shared affiliation | **FAITHFUL** — `\footnotemark` is a reference, not a copy (matches the PDF) |
| 6255 (F5) | googledeepmind | all authors merged into **one** personname (comma-list not split) | **LIVE** (class-specific) |
| 6209 (F2) | llncs | `\inst{n}`→affiliation mapping **correct**; the report is the *visible* superscript render (display layer, not parser) | not-parser |
| 5761 (F4) | elsarticle | `\author[n]`+`\address[n]` maps **correctly** | already-fixed |
| f1-core (F1) | article | 3× `Name\\Affil\\email\\ \And` → clean creators | already-fixed |
| 6687 (F1) | arxiv.sty | 9× `\And` → 10 clean creators w/ affil+email | already-fixed |

**Retracted (tool artifact):** an earlier pass flagged `\footnotemark[n]` as "leaking N
footnote N into the name" (6242, 4877). That was a bug in `tools/author_markup_char.py`
which flattened a `<note>`'s `<tags>` refnum/autoref metadata into the displayed name.
The real `<personname>` is clean — `<personname>Name<note role="footnotemark" mark="n">…`
— so `\footnotemark` handling is correct; the tool now strips `<note>` subtrees before
extracting name text.

**The genuine remaining live defects (root-caused on HEAD):**
- **6242 (F2) — multiple `\textsuperscript{n}Affil` on one space-separated line.** The
  affiliation line `\textsuperscript{1}UC San Diego \textsuperscript{2}SUNY Buffalo` has
  no `\\`/`\quad` between the two affils, so the marker-branch classifies the whole line
  as ONE affiliation (merging both) and the trailing `\texttt{…}` email bleeds into it.
  Fix would split an affiliation line at each interior `\textsuperscript` — heuristic,
  regression-risky (an affiliation may legitimately contain a superscript).
- **6255 (F1) — authblk single `\author{A, B, C, …}`.** googledeepmind loads `authblk`;
  the whole comma list is authblk's single-author-arg form, so LaTeXML keeps it as one
  `<personname>A, B, C, …</personname>` (faithful to authblk, but should split into
  creators). Fix is in the authblk `\author` binding — package-specific.

Both are heuristic/package-specific with regression risk; neither is a clean low-risk
win like R5. They need their own witness-guarded branches (R6), and a steer on which
first given the frontmatter-heuristic caution.

## Confirmed defects the pipeline must fix

1. **Phantom empty creators** (F3) — comma-split author lists with interspersed
   `\IEEEmembership`/`\thanks` emit empty `<personname/>` creators + stray `, ,`
   (local `ieee_membership`; witness 4539→2508.00603). Never emit a nameless creator.
2. **Shared trailing affiliation not shared** (F5) — attaches only to the last author
   (2507.06670; 4877→2509.22519; 5495→2512.07995).
3. **Leading-`\\` line-2 first author** (F1) — lost to affiliation (Ruiqi Li,
   2507.06670). Free under line-first splitting.
4. **Marker/ordinal/`*` leak into the name** (`1st …`, `Name*`) — decision is
   **per-idiom and evidence-driven** (F2/F3): whether a mark is semantic (→ note) or
   presentational (→ strip) depends on class + article. Do not decide abstractly;
   each idiom gets its own witness + red/green fixture.

## Open rows

Evidence-driven, lowest-risk-first. Every row: minimal repro from a real witness →
RED test → fix → full-corpus + real-arXiv green.

- **R1** Land the two in-flight patches (#615 bold; #52(d) leading-break) so the
  baseline reflects HEAD; re-capture with `tools/author_markup_char.py`.
- **R2** Reproduce each F1–F8 representative **on HEAD** (deployed binary lags); drop
  the already-fixed, keep the live ones as fixtures. This is the red/green seed set.
- **R3** Green-parallel harness — new pipeline behind a flag; diff emitted calls vs the
  old two-branch over the whole corpus + a real-arXiv frontmatter sweep. No cutover.
- **R4** Implement stages 1–3 to match every currently-`✓` witness byte-for-byte
  (esp. the #52-tuned comma-address / "…and…"-institution cases).
- **R5** Fix confirmed defects, **phantom empties (F3) first** — lowest risk, clear win,
  already witnessed (4539); then shared-affiliation (F5, own goldens + divergence entry).
- **R6** Per-family passes (F2 marker linking, F4 keyed schemes, F6 duplicate
  frontmatter) — each its own branch + witness set; F7 (raw-macro leak) is binding
  coverage, tracked separately.
