# Bibliography-absence case lists (2026-07-29)

Complete flagged-document lists behind
[`../BIB_ABSENCE_AUDIT_2026-07-29.md`](../BIB_ABSENCE_AUDIT_2026-07-29.md).
One row per flagged paper (= HTML lacks any `class="ltx_bibitem"`).

| file | scope |
|---|---|
| `2605.flagged.tsv` | `/data/arxiv/2605` sandbox-13 rerun (2026-07-27 binary), all 625 flagged |
| `2606.flagged.tsv` | `/data/arxiv/2606` sandbox-14 rerun, all 623 flagged |
| `corpus.flagged.full.tsv.gz` | full `/data/arxiv` 2026-07 rerun (pre-R5-re-port binary), all 153 296 flagged |
| `VERIFICATION.md` + `verification_2026-07-29.tsv.gz` | re-conversion of all 533 known articles with the PR build: 189 recovered / 14 715 entries, duplication audit, content spot-checks |
| `corpus.wrongly_missing.tsv.gz` | the actionable subset: HTML present + source wants a bibliography (52 299 rows) |

Columns (tab-separated):

1. `id` — arXiv id (= dir under `/data/arxiv/<yymm>/`)
2. `verdict` — `no_bib` | `empty_bib` | `no_html` | `no_result`
3. `cortex_status` — `Status:conversion:N` int (3 fatal, 2 error, 1 warn, 0 ok; `-` none)
4. `expect` — source bibliography intent: `yes` | `yes_legacy` (only via amsrefs/aastex/harvmac-era signals) | `no` | `auto_ignore` (withdrawal stub) | `no_tex` | `no_src`
5. `srcsig` — matched pass-2 source signals (`bbl,bib,stub,thebib,bibcmd,printbib,bibitem`)
6. `category` — telemetry category (`ok`/`conversion_error`/`conversion_fatal`/`-`)
7. `first_error_class` — first `Error:`/`Fatal:` log line, truncated at first space (`-` = silent)
8. `sig2` — pass-2b legacy signals (`biblist,bibcs,reference,referencesenv,listrefs,Refs`; `-` = none/not rechecked)
9. *(corpus full list only)* `markers` — `;`-joined bibliography log markers (`Missing Entry for citation`, `bibentries, 0 cited`, `bibliography:missing_keys`, `Couldn't find usable bibliography`)

`repros/` preserves the minimal repro inputs from the deep-dives:
`f1_bib_cascade/` — one-entry `.bib` files distilling each `$$`-poisoning
flavor (`a.bib`–`d.bib`, `t.bib`, plus the 2605.01115 `mini.bib`) with driver
`.tex` files and `bibbisect.py` (bisects a full `.bib` to its offending
entry); `f3_empty_arg_bbl/` — the empty-arg `\bibliography{}` + jobname-`.bbl`
repro (`min5.tex`/`min5.bbl`, silent in Rust AND Perl).

`pass2c.sh` is the tightened re-check of the legacy signals: `pass2b.sh`
matched `\references` inside longer control sequences (0704.0420's
`\def\referencesz{…}`), and re-running it over the 17,055 `yes_legacy` rows
drops 1,522 false positives.

Regeneration: `scan_bib.sh` / `pass2.sh` / `pass2b.sh` / `pass2c.sh` / `marksweep.sh` — the
actual sweep scripts, checked in beside this README (~20 min for pass 1 over
the 2.79 M-doc corpus at 64-way parallelism; pass 2 ~35 min over the flagged
set).
