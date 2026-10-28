# Author-block labeling: a data-driven plan (design note, 2026-10-28)

**Status:** proposal, awaiting the user's decision. Nothing here is built.

**Scope:** author blocks with no class structure. These are `article` and raw-loaded classes whose `\author{…}`
holds names, affiliations and addresses as lines (`\\`), with no marks or `\and` saying which line is which.

Bound classes are out of scope (`\inst`/`\institute`, `\affiliation`, `\address`, `\affil`): their binding reads the
class's own structure. The pipeline itself is described in [`AUTHOR_MARKUP_PIPELINE.md`](AUTHOR_MARKUP_PIPELINE.md),
and ownership rules are in [`FIDELITY_CHARTER.md`](FIDELITY_CHARTER.md) §4.

## The problem

**The hard part is labeling the pieces, not splitting them.**
- Segmentation is given by the source: `\\`, `\and`, marks, wide spaces and spacer lines split the block, and pdflatex
  prints exactly those pieces.
- The hard part is **labeling** each piece (person, affiliation, address, e-mail, note), then **attaching** the
  contacts to their persons.

**Today's recognizer labels each line on its own**, looking one line up and one line down. It uses hand-written word
lists and rules in `latexml_engine/src/base_utilities/author_block/`: `name_count`, `reads_as_affiliation`,
`NON_NAME_WORDS`, and 85a's spacer, styling and institution-under-it signals.

**Each batch draws new counter-examples:**
- 84a needed three follow-up fixes: `\(…\)`, `\rlap,` and grouped marks.
- 85a r1 made 4 census papers worse: 1110.3939, 1404.5443, 2510.15190 and math-ph9903017.
- Its review then turned 9 ordinary affiliation lines into creators, for example:
  - "Johns Hopkins \\ University, Baltimore";
  - "Max Planck \\ Institute for …";
  - "Santa Barbara";
  - "Associate Dean".

**Size**, measured by the frontmatter correctness scorer on the int114a binary over its 2,578 scored papers:
- 84 papers have a name defect (3.3%).
- `article` is the weakest class: 74% clean, measured on int111d.

## The user's skepticism is the hypothesis to test

> "If we are only able to react to words and lines, the local approach will be nearly equivalent to the global one,
> as our limited vocabulary and patterns will yield a limited recognizer." (user, 2026-10-28)

A decoder that reads the whole block, but uses the same hand-written features, adds only two kinds of evidence:
- **parallelism within the paper:** each group is formatted like the others (punctuation, fonts, line counts);
- **corroboration:** an e-mail that spells the name, or a second author list.

Neither enlarges the vocabulary. So the plan measures before it builds, and lets data replace the hand-written
vocabulary.

## Data first

### 1. Gold: a catalogue of the hard cases

Each entry records:
- the author block's text;
- the creators pdflatex prints, labeled by hand from page 1;
- phenomenon tags.

The tags are a deliverable in themselves: the phenomena observed, each with a count.

**Sources:**
- **html_feedback `frontmatter-authors-affiliations`:** 583 issues, 547 papers, 440 still open
  (`~/data/html_feedback/TAXONOMY.md`). Re-convert on head and keep the ones still wrong. Reporters often name classes
  we bind (acl 24, elsarticle 10, sn-jnl 8), so this source spans bound classes too.
- **The scorer's defect papers on int114a:** 84 with a name defect, 18 of them with missing authors.
- **The review probes,** all pdflatex-clean: 85a's q1–q16 and r1–r15, and 85b's p01–p25.
- **The deferred witnesses** in `DEFERRED_FAMILIES.md`.
- **Every census A/B change** from 84a through 85b.

### 2. Silver: names and affiliations the authors labeled themselves

56% of the 2,987 sample papers use separate name and affiliation commands. Counts by command (a paper can use
several):

| Command | Papers |
|---|---:|
| `\affiliation` | 684 |
| `\address` | 534 |
| `\affil` | 183 |
| `\institute` | 172 |
| `\IEEEauthorblock*` | 85 |
| `\icmlauthor` | 29 |

**Why it matters:**
- Across arXiv that is on the order of a million papers' name strings and affiliation strings.
- They come from the same population of authors who write hand-formatted blocks.
- The strings are taken from our own XML for those classes (93–99% clean), not from text rules.
- **Caveat:** venues differ by field (llncs for CS, revtex for physics, aas for astronomy), so measure the distribution
  shift.

### 3. Corroboration in hand-formatted `article` blocks

Among the 464 sample papers with `\\` in `\author`:
- an e-mail address appears somewhere in the source in 69%;
- a second author list is rare: `pdfauthor` in 5%, `\markboth` in 1%.

## Recognizers to compare

The comparison runs offline and read-only, in scripts, before any Rust is written.

| | Method |
|---|---|
| **M0** | Today's rules, read from our XML. |
| **M1** | Line scores learned from data: name-versus-affiliation likelihoods (word and character n-gram statistics) from the silver set, replacing the hand-written lists. |
| **M2** | M1 plus block-level decoding: the labeling that best fits "groups of names, then their contacts", with parallelism across the groups. |
| **M3** | M2 plus corroboration: e-mail local parts, `pdfauthor`, `\markboth`. |

**Metrics**, per paper, on the gold set and on the 3,000-paper sample:
- exact creators;
- **invented creators (target 0):** they are false metadata, which is worse than a missed author, since a missed author
  keeps its text as a contact (Perl's reading);
- missed creators.

## Decision rule

**Adopt the simplest method that beats M0 on gold without adding invented creators on the sample.**
- If M2 ≈ M1, the skepticism is confirmed, and the block decoder is not built.
- If M1 ≈ M0, the vocabulary is not the bottleneck. The gold catalogue then drives per-phenomenon mechanisms instead,
  the data-driven route without a learned recognizer.

**If a method wins:**
- It ships as an embedded table (`include_bytes!`, keeping the binary self-contained), behind the same `author_block`
  interface.
- It needs an OXIDIZED_DESIGN_DIVERGENCES entry, since it goes beyond Perl.
- Segmentation stays at the TeX-call level (ruling 2026-10-25).

## Prior art

Header extractors that work from PDFs, such as GROBID and CERMINE, label author and affiliation lines with models
trained on labeled headers. That is evidence the data-driven route works, and also that it falls short of 100%.

## Cost

| Step | Cost |
|---|---|
| Gold catalogue | The main cost: labeling roughly 200–500 blocks by hand. A read-only analyst pre-fills each entry from the PDF text; the main session checks it. |
| Silver extraction and the M0–M3 comparison | About a day of scripts and compute, with no repo change. |
| Rust port | One batch, only if a method is adopted. |

## What it will not solve

- Hand-typeset title pages with no `\author` (6.9% of the sample; deferred).
- A short line with no structure, no parallelism and no corroboration ("Ann Arbor" under an institution). Its label
  stays a probability for any recognizer, so the invented-creator target keeps it a contact.
