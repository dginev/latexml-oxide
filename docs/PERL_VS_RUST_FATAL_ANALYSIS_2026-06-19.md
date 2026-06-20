# Perl-vs-Rust fatal analysis — tikz-cd / xy / tcolorbox sandbox (2026-06-19)

Point-in-time diagnostic from the 3-sandbox reconversion (corpora 7
`sandbox-arxiv-tikz-cd`, 8 `sandbox-arxiv-xy`, 12 `sandbox-arxiv-tcolorbox`;
29,621 docs). Pairs with `HANDOFF.md` and memory
`sandbox-3corpus-run-2026-06-19`. Both engines' results live in the cortex DB:
**service 3 = Perl `tex_to_html`**, **service 4 = Rust `oxidized-tex-to-html`**.

## 1. Headline: Rust already substantially outperforms Perl here

`tasks.status` per service (−1 ok, −2 warn, −3 error, −4 fatal, −5 invalid):

| status | Perl (svc 3) | Rust (svc 4) |
|---|---|---|
| ok (−1)      | 1,383  | **9,379** |
| warn (−2)    | 15,385 | 13,526 |
| error (−3)   | 9,809  | 5,853 |
| **fatal (−4)** | **3,011** | **861** |
| invalid (−5) | 23 | 2 |

Rust has **~3.5× fewer fatals** (861 vs 3,011) and far more clean-OK docs.

## 2. Most Rust fatals are *shared* failures, not Rust bugs

Cross-join svc4-fatal × svc3-outcome on `(corpus_id, entry)`:

| Perl outcome on the Rust-fatal doc | papers |
|---|---|
| **Perl also fatal** | **743 (86 %)** |
| Perl error (completes w/ errors) | 101 |
| Perl warn | 13 |
| Perl ok | 2 |
| Perl todo (not run) | 2 |

So **743/861 Rust fatals are also fatal in Perl** — pathological tikz-cd/xy
inputs both engines reject. Only **15 papers are Rust-fatal while Perl
succeeds (≤ warn)** = genuine Rust-worse divergences; +101 where Perl limps to
output and Rust gives up.

## 3. The 15 genuine divergences (Rust fatal, Perl ≤ warn)

| paper | Perl | Rust fatal class | root cause |
|---|---|---|---|
| 1806.07508 | warn | `caught` (panic) | **FIXED** this session (P1, math-parser; now converts) |
| 1610.00974 | ok | MaxLimit(500) | pgf matrix `&` catcode → `\GenericError` ×501 |
| 1709.07916 | ok | never_completed | timeout / runaway |
| 0710.3853 | warn | never_completed | timeout / runaway |
| 1105.4857 | warn | never_completed | timeout / runaway |
| 1903.02279 | warn | never_completed | timeout / runaway |
| 1112.5148 | warn | (timeout) | timeout / runaway |
| 1312.6499 | warn | TokenLimit→MemoryBudget | pgf/tikz runaway |
| 1605.08297 | warn | IfLimit | (not on disk) |
| 1912.13052 | warn | IfLimit→MemoryBudget | pgf/tikz runaway |
| 2004.14791 | warn | IfLimit→MemoryBudget | pgf/tikz runaway |
| 1703.02996 | warn | MaxLimit(100) | (not on disk) |
| 1906.03240 | warn | MaxLimit(100) | `mijnpackages.sty` aborts at `\usepackage{tikz-cd}`/`\usetikzlibrary{babel}` → all its macros (`\GL`,`\bP`,…) undefined → group/error cascade |
| math0111244 | warn | MaxLimit(100) | (not on disk) |
| math0110249 | warn | (timeout) | (not on disk) |

**Every investigated divergence traces to deep tikz-cd / pgf / babel / xy
binding gaps**, not to isolated, easily-bound macros:
- **1610.00974** — pgf matrix interpretation emits `Package pgf Error: Single
  ampersand used with wrong catcode` per cell (502×); the consecutive-error
  runaway guard then fatals. Perl's pgf handles matrix `&` and degrades to warn.
- **1906.03240** — `mijnpackages.sty` loads xy + babel + tikz-cd +
  `\usetikzlibrary{babel}`, then defines the paper's macros. Rust silently stops
  processing the wrapper around the babel/tikz-cd interaction (tikz-cd.sty never
  loads), so `\GL`/`\bP`/`\Aut`/… and `{thm}` stay undefined → ~100 cascade
  errors → MaxLimit(100) fatal. Perl loads the wrapper fully (Status ok).
- **1912.13052 / 2004.14791 / 1312.6499** — pgf/tikz runaways that exhaust the
  RSS budget (MemoryBudget at the standalone 4500 MB guard) or the If/Token fuses.

## 4. Shared-bug spot-check (parity, documented, not fixed)

- **`Recursion` class** (78 papers): `\item[\refstepcounter{<itemcounter>}…]`
  with the optarg counter == the list counter → unbounded re-entry through the
  tag machinery. **Perl also fatals** (`deep_recursion`). Confirmed live on the
  freshly-installed Perl LaTeXML 0.8.8 and against the on-disk `tex_to_html.zip`
  for 2009.08640. See `KNOWN_PERL_ERRORS.md` #32.

## 5. Landed this session

- **P1 panic cluster (commit `c47d37f416`)** — 4 distinct panic sites fixed
  (`state.rs` RefCell via `try_lookup_int` in the `Error!` path + push/pop
  hygiene; `\fontdimen` empty-args guard; alignment `current_row_mut` guard);
  the 5th (math-parser) was already fixed on master. All 5 `caught`-class fatal
  papers (2001.08973, 1806.07508, 1905.02617, 1908.10358, 1910.04182) now
  convert with exit 0. Two `\fontdimen` witnesses drop to zero errors.

## 6. Recommendation (top lever for future work)

The single highest-impact remaining lever is **tikz-cd / pgf / babel binding
completeness** — it drives the non-fatal error tail (`\tikzcdmatrixname` 378
papers / 33,974 msgs; pgf-matrix `&` cascades; `\cmdGR@edge@*` tkz-graph 51
papers) *and* most of the genuine fatal divergences in §3. It is a large,
regression-prone engine effort (the pgf matrix node-naming path, the babel ×
tikz-cd × xy interaction), not a set of quick macro bindings — scope it
deliberately. The error-cap/`If`/`Token` fuses firing *fatal* where Perl limps
to output (the 101 "Perl-error, Rust-fatal" set) is a secondary
graceful-degradation lever, but the bindings are the real root.

## 7. Deeper P2 investigation (the "lever" is mostly NOT a parity gap)

Following up the §6 recommendation with the now-installed Perl reference:

- **`\tikzcdmatrixname` is NOT a Perl-parity gap.** Of the 378 papers with this
  Rust error, **377 are also Perl-FATAL** (1 Perl-error). The trigger is
  `\begin{tikzcd}[ampersand replacement=\&]` heavy diagrams, and it is a
  *cumulative document-state* effect — the same tikzcd block converts cleanly in
  isolation (even with the full witness preamble) but fails at line 3358 of the
  full 2106.16186; it does not reduce to a small construct. Basic and
  `ampersand replacement` tikzcd both work in Rust and Perl. **Conclusion:**
  improving tikzcd-matrix here is a *surpass-Perl quality* play on papers Perl
  cannot do either, not a regression fix — large, open-ended pgf-engine work.

- **One genuine babel parity gap, narrow + deep.** Witness 1906.03240
  (Perl-warn, Rust-fatal). **Minimal repro:** a custom `.sty` containing
  `\usepackage[ngerman,english]{babel}` then `\selectlanguage{english}` —
  Rust **silently truncates the rest of the `.sty`** (every macro defined after
  `\selectlanguage` is undefined → cascade → MaxLimit fatal), while Perl loads
  the `.sty` fully. Root cause: `\selectlanguage` → real-babel `\select@language`
  loads `<lang>.ldf`, and that nested `.ldf` load does not resume the enclosing
  `.sty` input stream. Nested `\usepackage` inside a `.sty` does *not* truncate
  (only the `.ldf`-via-`\selectlanguage` path does). Impact is narrow (this
  `\selectlanguage`-inside-a-custom-`.sty`-preamble pattern is rare); the fix is
  in the InputDefinitions/mouth-nesting core (regression-prone). Deferred.

**Net:** across the corpus Rust is at or above Perl; there is no high-impact,
tractable, *safe* parity fix left in tikz-cd/pgf/babel. Remaining gains are
either surpass-Perl R&D (tikzcd matrices) or narrow + risky (babel `.ldf`
nesting).

## Repro

```bash
# Perl reference (installed 2026-06-19 via cpanm . --notest in LaTeXML/):
latexml --quiet <paper>.tex
# Rust (release):
./target/release/cortex_worker --standalone --input /data/<corpus>/<id>/<id>.zip --output /tmp/<id>.zip
# DB cross-join (psql cortex): svc4-fatal × svc3-outcome on (corpus_id, entry).
```
