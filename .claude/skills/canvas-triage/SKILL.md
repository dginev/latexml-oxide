---
name: canvas-triage
description: >
  Triage a failing/suspicious arXiv paper and decide whether its errors are a
  GENUINE Rust regression or parity with Perl LaTeXML. Use when investigating a
  conversion error/fatal, a "Rust-only" candidate from cortex, a sandbox/canvas
  failure, or any "is this a real bug vs. parity?" question. Pairs with
  tools/parity_check.sh, triage_failure.sh, first_error.sh and the cortex
  document/<id> API. Invoke for "triage this paper", "is 1234.5678 a regression",
  "classify this failure vs Perl", "/canvas-triage".
---

The output of this skill is a **classification**, not a fix. Only one verdict —
GENUINE-RUST-ONLY, confirmed same-host — justifies code changes. Reach for the
`perl-port` skill once you get there.

## Golden rules (non-negotiable — violating these has burned us repeatedly)

1. **Never downgrade errors to cheat the task.** A change that makes Rust emit
   *fewer* `Error:`/`Fatal:` than Perl on the same paper is a **divergence**, not
   a fix — it needs explicit proof Perl emits the same severity, or user
   authorization to surpass Perl. We nearly committed unfaithful fixes off false
   signals (`1308.2655` `\lefteqn`, `0710.5692` equationgroup-in-`<p>`) — both
   were parity (Perl errored too). Reverted after checking cortex.
2. **Fail-safe toward flagging failure.** A *failure to parse the log* must NEVER
   be treated as success. False positives (flagging a clean run) are acceptable;
   false negatives (missing real errors) are forbidden. When in doubt, count it
   as a failure to investigate.
3. **Classify Perl with VERBOSE, never `--quiet`.** `/usr/local/bin/latexml
   --quiet` prints "0 errors" / exits 0 *even when the conversion has errors* — it
   suppresses both the `Error:` lines and the final count. This trap has bitten us
   ≥3 times (babel-russian, collcell, francais). Run `latexml <paper>.tex` plain
   and read the `Conversion complete: … N errors; … undefined macros[…]` summary.
4. **ANSI-strip before grepping.** Logs may carry color codes
   (`\x1b[31mError:`), so a naive `grep '^Error:'` silently matches **zero** on a
   paper with hundreds. Always: `sed 's/\x1b\[[0-9;]*m//g' | grep -acE
   '^(Error|Fatal):'`. Better signals when available: cortex
   `Status:conversion:N` (**3=fatal, 2=error**, lower=ok) or the ANSI-free
   on-disk `.latexml.log`.
5. **Same-host Perl only.** A cortex `Perl=clean / Rust=error` delta is often a
   pure **host TeX-package-availability artifact** — the two services ran on hosts
   with different texmf trees. Confirmed phantoms: inputenc `isolatin` (needs
   `umlaute` pkg), `cp1251`/Cyrillic (needs `cyrillic`/`t2`), babel `russian`
   (older babel ≤3.8). In all three, Rust == Perl *on the same host*. Reproduce
   **both** engines on the **same** tree before trusting any delta. Host packages
   are out of scope (CLAUDE.md) — we don't ship `.def`/`.ldf`.
6. **The cortex DB is a SCREEN, not ground truth — it lags HEAD.** The
   `sandbox-arxiv-10k-shuffle` Rust column predates recent fixes (e.g. `1805.00875`
   shows errors in cortex but is **0 on the current binary**). Use the DB only for
   *candidate discovery*; re-run every flagged paper on the **current** binary
   before chasing it. Never trust a bespoke cross-join's Perl column — confirm
   against the live `GET /api/corpus/<corpus>/tex_to_html/document/<id>` API (it
   carries per-severity `message_counts` for both services).

## Workflow

**1 — Reproduce Rust on the current binary.** For a quick count:

```bash
cargo run --bin latexml_oxide -- --format=html5 --log=cortex.log --dest=/tmp/out.html paper.tex
sed 's/\x1b\[[0-9;]*m//g' cortex.log | grep -acE '^(Error|Fatal):'
```

For a crash/backtrace under the diagnosable test profile:
`tools/triage_failure.sh <arxiv_id>` (`RUST_BACKTRACE=full`; `KEEP_TMP=1`
preserves the extracted dir). To see the *first* non-cascade error class with
source context: `tools/first_error.sh <paper.log>`.

**2 — Reproduce Perl on the SAME host** (verbose, never `--quiet`):
`/usr/local/bin/latexml paper.tex` → read the `Conversion complete: … N errors`
line. This is the parity ground truth.

**3 — Or run the automated verdict:** `tools/parity_check.sh <arxiv_id>` emits
BOTH-CLEAN / OUT-OF-SCOPE / REAL_REGRESSION / PERL_REGRESSION / Perl-capped /
Perl-timeout (reads `SANDBOX_DIR`, `TIMEOUT_SECS=180`).

**4 — Classify:**

| Verdict | Meaning | Action |
|---|---|---|
| BOTH-CLEAN | 0/0 | nothing |
| PARITY | same severity both engines (same host) | nothing — document if surprising |
| ENV-ARTIFACT | delta is a host TeX-package/version diff | nothing — note in memory; do NOT "fix" |
| GENUINE-RUST-ONLY | Perl clean, Rust errors, **same host** | → fix candidate |
| DEFERRED | content-MathML / `expected:id` / known-large | leave; tracked in SYNC_STATUS |

**5 — Only GENUINE-RUST-ONLY (same-host-confirmed) is a fix candidate.** Read the
Perl source for the construct and port faithfully via the `perl-port` skill;
reduce to a fixture via `min-repro`; validate with `cargo test --tests
--no-fail-fast` + clippy + Perl parity on the witness.

## cortex API (candidate discovery only — open reads, no token)

`http://127.0.0.1:8000/api`; Rust svc `oxidized-tex-to-html`, Perl svc
`tex_to_html`, corpus `sandbox-arxiv-10k-shuffle`.
- `GET /api/reports/<corpus>/<svc>/<severity>` → categories
- `…/<severity>/<category>` → per-`what` → `…/<what>` → papers
- `GET /api/corpus/<corpus>/tex_to_html/document/<id>` → Perl status + counts
URL-encode `\`→`%5C`, `^`→`%5E`. A Rust-only win = Perl `no_problem`/`warning`
but Rust `error`/`fatal` — then re-confirm same-host (rule 5/6).

## Known phantom signals (do not re-mine / re-fix)

inputenc `isolatin` · `cp1251`/Cyrillic · babel `russian`/`ukrainian` · the
"shared Node" cluster (fixed in HEAD) · `1805.00875` (stale, 0 on current) ·
`1308.2655` / `0710.5692` (parity, not Rust-only). The full `error`-severity
sweep against the stale 10k DB is **exhausted** — do not re-mine it per-`what`; a
genuinely new correctness bug requires a **fresh** cortex Rust rerun on current
HEAD.
