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

## Confirmed defects the pipeline must fix

1. **Phantom empty creators** — comma-split author lists with interspersed
   `\IEEEmembership`/`\thanks` emit empty `<personname/>` creators (ieee_membership).
   Stage 2/4: never emit a creator with no name text.
2. **Shared trailing affiliation not shared** — attaches only to the last author.
3. **Leading-`\\` line-2 first author** — lost to affiliation (Ruiqi Li). Free under
   line-first splitting.
4. **Marker/ordinal/`*` leak into the name** (`1st …`, `Name*`) — candidates; decide
   per-idiom whether the mark is semantic (drop to a note) or presentational (strip).

## Open rows

- **R1** Land the two in-flight patches (#615 bold; #52(d) leading-break) so the
  corpus baseline includes them, then re-capture.
- **R2** Build the green-parallel harness (new pipeline behind a cfg/flag; diff emitted
  calls vs old on the corpus).
- **R3** Implement stages 1-3 to match the current clean witnesses byte-for-byte.
- **R4** Stage 4 shared-affiliation (own goldens + divergence entry).
- **R5** Fix the confirmed defects (phantom empties first — lowest risk, clear win).
