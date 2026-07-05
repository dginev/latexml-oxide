# Commit review — July 1–5, 2026 (overconfidence audit)

Adversarial review of the 79 code commits on `ar5iv-2606-prep` dated
2026-07-01 … 2026-07-05, run 2026-07-05 (user request: "some may have been
overconfident"). Six parallel skeptical passes, each cross-checking every
"exact parity / faithful / verbatim / byte-parity" claim against the Perl
ground truth (`LaTeXML/lib/LaTeXML/…`), tex.web, or a real pdflatex/pgfmath
run. **Snapshot — date+archive when the branch merges.**

Verdict: the "verbatim/byte-parity" *table* claims (MathML opdict, vpack
`prevdepth`, parbox sizer, atom-pair spacing) held up under machine-diff. The
genuine overconfidence was concentrated and, for the MathML cluster, mostly
**self-corrected later on the same branch**. Two live wrong/under-report bugs
were found; one is fixed here, the rest are re-documented or tracked.

## Disposition table

| Commit | Finding | Sev | Disposition (2026-07-05) |
|---|---|---|---|
| `f4a6420b15` pgfmath | `int_result` was a stream-**global** flag, not top-level as claimed → `2*(1<2)` printed `"2"` where pgfmath prints `"2.0"` (verified pdflatex). Wrong answer, tested witness-only. | MED (live) | **FIXED** — arithmetic combinators (`expr`/`term`/`factor` `^`) now clear `int_result`; ground-truth regression test `comparison_int_result_is_scoped_not_global`. |
| `ede2bdcc2c` bibliography | "mirrors Perl, only rendered fields digest" is **inverted** — Perl `BibTeX.pool.ltxml` digests ~28 fields incl. abstract (L708)/keywords (L732)/annote (L680); Rust digests 13 → under-reports errors vs same-host Perl on raw-`.bib` ADS/Zotero. | HIGH (live, narrow) | **RE-DOCUMENTED** as a first-stage-toward-Perl partial port (user decision) — corrected the in-code comment; open parity task in SYNC_STATUS. Bounded to the raw-`.bib`-without-`.bbl` path. |
| `ff87a841e5` F8b MathML | Message claims ENCLOSE/FRACOP color fallbacks the diff never contained (only sqrt/mroot landed); witness had no fraction/enclose to expose it. | HIGH (claim only) | **DOC CORRECTED** — the arms actually landed in `cb1ad27a61`; code is correct, `MATHML_POST_LINE_AUDIT.md` F8b entry annotated. |
| `bff1a2550f` bibliography | "0 errors / suite green" on one witness; the next commit records a **714-paper +599-error regression** it caused. | HIGH (historical) | Mitigated by the `50a48f33de`→`aac7537bcc`→`ede2bdcc2c` churn. Process lesson only. |
| `786d9ed89d` lxDeclare | "matching `font_match_xpaths`" collapses Perl's family+series+**shape** match to 3 booleans (`bold`/`caligraphic`/`typewriter`) — drops shape + sansserif/fraktur/blackboard; a plain-italic `\lxDeclare{$x$}` wrongly annotates `\mathrm{x}`. | MED (latent) | **OPEN** — tracked below. Niche (DLMF-style) feature, witness-clean. |
| `dd226d1973` lxDeclare | `neutralize_font()` uses concrete serif/medium/upright where Perl `compile_replacement` uses an **empty inheriting** font — survives on bold/sansserif matches Perl would inherit. | MED (latent) | **OPEN** — tracked below. |
| `2da23e6154` gullet | `read_balanced` crossing now gates on mouth KIND — a subtle hot-path behavioral change shipped with **no locking test** (witness only in the message). Logic assessed sound. | MED (test debt) | **OPEN** — add a fixture. Witness: `Before. \input{missing}` must error like Perl AND keep the parent paragraph. |
| `2b1ebe2492` #46 fo font-size | "exact for every font" is true for box geometry only; the same `font-size` is inherited by visible content → text renders at the TFM quad (cmtt10 ~+5%). Contradicts `WISDOM §47.3`. | MED (live tension) | **DOC CORRECTED** — WISDOM §47.3 caveat added; splitting geometry-vs-text anchor is the open follow-up. |
| `3dcc6fdc27` #47 verbatim | "never ignorable" rides `font==typewriter` (not "is verbatim"); surpass-Perl with no oracle — digested tt spaces outside verbatim (`\texttt{ x}`, tabbing) can now survive p-edge trim. | MED (surpass, watch) | **NOTED** — documented divergence #47; monitor for over-preservation. |
| `f0a8847c07` panics | Graphics arm catches ALL worker panic payloads generically → one Error + dropped figure; lacks the first-principles pdflatex-avoidance reasoning its sibling `d83685c69d` shows. | MED | **NOTED** — emits an Error (passes the literal rule); the parser/alloc half is sound. Tighten to name the OOM class. |
| `da2515e3a8` kpsewhich memo | "stable for a fixed texmf tree" false — kpathsea path includes cwd (`.`); broke cross-paper. | MED | Mitigated by `8f5c077bce` (per-conversion clear); within-conversion miss-then-appears edge remains. |
| `24def068cd` F14 share-id | per-formula `SH_COUNTER` could mint duplicate `xml:id`s (invalid XML); "byte-identical incl. share hrefs" witness-only. | MED | Fixed later by `2e6517b19d` (monotonic global counter). |
| `dd433e560d`/`83ffcf9dfc` | fvextra/tabularray lean on `\let…\relax` suppression / column caps — masking, not translating; honestly labeled but bail to the same failure mode out of range. | LOW | As-is (labeled). |
| `295f6603f9` | "1.81x faster" subject from a single load-noisy A/B run; doc body is honest and scoped. | LOW | Subject overreach only. |
| `20adb952e7` | "Complete the #2835 port" contradicted by its own disclosed `\widthof`-returns-0 gap. | LOW | As-is (disclosed). |
| `856de84a10`, `189b50d4d8`, `6ee4f7bcfc`, `e4a0313629`, `e46f38a2d3`, `5d2ed92988`, `c08e47fee4`, `c38d159205`, `f5ab8aa2da`, `aac7537bcc`, `50a48f33de`, `0dabea6758` | subject-line overreach / honest incompleteness / cosmetic / under-covered test surface. | LOW | Noted; no action. |

## Held up as sound (spot-verified against ground truth)
#2829 Framing (`179d445955`/`6d82d2eb06`, incl. the genuine double-digest crash
fix), amsmath multline/`\shove*` (`83b05eea5e`/`e5b77edd91`), paralist typo,
vpack `prevdepth` (faithful tex.web `append_to_vlist`), parbox sizer `#5`,
opdict tables (0 diffs / 788 codepoints), the re-blessed `9a679469e1` golden
(verified vs reference Perl — not circular), F18 mroot (really spec-backwards),
F2 dead-code (genuinely dead), `\dabar@`/`\+`/`find_file`/`\string`→`\special_relax`,
graphics casefold, aa.cls lineno, the unresponsive-worker watchdog (triggers on
CPU-time freeze, won't kill a progressing worker), and every OXIDIZED_DESIGN
#42–#47 citation (all resolve, none dangling).

## Open follow-ups (post-review)
1. **lxDeclare font fidelity** (`786d9ed89d`, `dd226d1973`) — widen the fast-path
   font match to Perl's family+series+shape discrimination, and make the
   `replace=` font empty/inheriting (`Font::new()`), not a concrete default.
   See memory `lxdeclare-two-paths-font-rewrite-2026-07-01` ("@font unreliable
   at rewrite time").
2. **Gullet `read_balanced` locking test** (`2da23e6154`) — add a fixture for the
   File-boundary-crossing behavior.
3. **foreignObject geometry-vs-text anchor split** (`2b1ebe2492`) — decouple the
   em-divisor quad (geometry) from the inherited text `font-size` (design size).
4. **Bibliography field-interpretation parity** — widen the 13-field whitelist to
   Perl's rendering-field set (folds into the full bib re-port; SYNC_STATUS).
