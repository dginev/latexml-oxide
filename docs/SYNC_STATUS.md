# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

> **This file is the BRIEF ACTIONABLE LIST.** The day-by-day fix log and
> completed-task records are NOT kept here — they live in `git log` and
> `docs/archive/`. **When you close an item, delete it here** (git keeps the
> record). Last compaction: 2026-06-21.

## Current status

- `cargo test --tests`: **1577 / 0 / 0** (on `public-release-prep-week` after
  merging `origin/main`'s Windows release hardening; the completed 2026-07
  session logs are archived — see the pointer below).

- **2026-07-17 — crates.io: all code blockers cleared; tagged `0.7.4-rc4`.**
  `#[derive(LoadModel)]` reads `latexml_core`'s **embedded** RelaxNG table instead of
  resolving `LaTeXML.model` cwd-relative, so `resources/RelaxNG` could move into
  `latexml_core/` (108 files) where `cargo package` sees it. Also B6 (`readme`
  outside the crate dir → symlink) and the dead `script-bindings` alias, dropped
  pre-publish. Detail: [`release/CRATES_IO_PUBLISH.md`](release/CRATES_IO_PUBLISH.md)
  B3b/B6.
  **The class worth remembering: a resource move silently breaks path-referencing
  gates.** `audit_vendored_natives.py` scanned only the root `resources/`, so B3a had
  *already* dropped XSLT/CSS/js out of the license audit unnoticed, and B3b would have
  printed `ok resources/RelaxNG/svg/ (0 file(s))` and exited 0 — its own header's
  scenario. Fixed, plus a prefix-must-exist guard (verified to fire). Same for
  `THIRD-PARTY-NOTICES` §2.2/§2.3, `LICENSE_INVENTORY` §B, `compileschema.sh`, the XML
  catalog, and ar5iv-editor's deploy script.
  **Tags are bare-numeric, no `v`** (`release.yml` matches `[0-9]+.[0-9]+.[0-9]+-*`;
  `v0.7.4-rc4` runs nothing), and `make_release.sh` refuses a tag ≠ Cargo.toml version.

- **2026-07-09 — `\AtBeginDocument` #2754/#2846 re-done via context-aware `\par`
  (Direction B retired; ported to Perl too).** The earlier `inBeginDocumentHook`
  guard-decouple is reverted: `\begin{document}` restores the pre-#2846
  `inPreamble=0`-after-hooks placement and `only_preamble` is a plain `inPreamble`
  check again (no second flag). `\lx@normal@par` is a no-op **only in the raw
  preamble** — `inPreamble` set AND `document` NOT on the `current_environment`
  stack; everywhere else it closes the paragraph being built. So a blank line in
  `\AtBeginDocument` (runs in the document env) splits paragraphs (#2754) while a
  deferred `\RequirePackage`/`\usepackage` stays legal (inPreamble still 1). NOT the
  note's literal "no-op in vertical mode": LaTeXML's mode tracking isn't faithful
  (stays vertical after display math — would also mis-merge `\AtBeginDocument{\[x\]…}`;
  raw-preamble text is horizontal yet must merge — expl3 case fixtures), so CONTEXT
  (are we in the document env) is the stable signal; the env-**stack** check also
  handles nested envs inside hooks. Applied identically in Perl
  (`LaTeXML/lib/.../latex_constructs.pool.ltxml` + `TeX_Paragraph.pool.ltxml`,
  `lookupStackedValues`). New regression tests: `tests/structure/atbegindocument_*`.
  See `KNOWN_PERL_ERRORS.md` #43. Candidate to upstream as the #2846 follow-up.

### Session logs (2026-06-22 … 2026-07-08) — ARCHIVED

Completed "Landed this session" entries, the slowest-100 batch triage, the
finished upstream-sync U1–U11 mission log, and the mined-out methodology
history now live in the dated session archives:

- [`archive/SYNC_SESSIONS_2026-07.md`](archive/SYNC_SESSIONS_2026-07.md) —
  the 2026-07-02 … 07-08 window: upstream PR #2829 "Framing", the MathML-post
  exhaustive line audit (waves 1+2), live-run fatal/error mining rounds,
  author/affiliation frontmatter split, width-based figure-panel arrangement,
  and the `\AtBeginDocument`/`\RequirePackage` #2846-port regression fix.
- [`archive/SYNC_SESSIONS_2026-06.md`](archive/SYNC_SESSIONS_2026-06.md) —
  the 2026-06-22 … 07-01 window plus the slowest-100 batch triage and the
  2026-06 cortex-cross-join methodology history.

(Upstream-sync catalog also at
[`archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md`](archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md).)

### apxproof bibliography + option-value catcode (LANDED 2026-07-10)
Rust Error Fix. `gdsm.tex` (biblatex + `\usepackage[bibliography=common]{apxproof}`)
now converts error-free in every config (bare / `--includestyles` / ar5iv): 24
linked bibitems, 6 `ltx_proof` (amsthm markup, correctly inline — apxproof defers
only its own `apxproof`/`proofatend` envs). Two parts:
1. **`latexml_contrib/src/apxproof_sty.rs`** — force-raw-loads `apxproof.sty` in
   all configs (no Perl binding exists; Perl aborts the bib on kvoptions
   `\ProcessLocalKeyvalOptions*`). Surpass-Perl; see KNOWN_PERL_ERRORS #44.
2. **Core catcode fix** (`binding/content.rs`): `\opt@<name>.<ext>` now built with
   `ExplodeText!` (LETTER catcode) not `Explode!` (OTHER), so kvoptions/keyval
   `\setkeys` values pass catcode-sensitive `\equal`/`\ifx` validation. Broad
   reach (every `\DeclareStringOption` validator). See WISDOM #61; regression
   fixture `tests/keyval_options/optcatcode*`. Full suite 1538/0.

### Figure panels of unmeasurable images wrapped by filename length (LANDED 2026-07-10)
Rust Error Fix (fidelity). A float of bare `\includegraphics` with no explicit
`\\` is partitioned into rows by `arrange_panels_and_breaks`, using each panel's
MEASURED box width. `read_image_dimensions` reads PNG/JPEG/EPS only (like Perl's
`imgsize`), so for **PDF/SVG** it early-returned with no `cached_width`;
`compute_size` then summed the whatsit's argument boxes — including the
Semiverbatim **path string** — so panels wrapped by *filename length*.
arXiv:2409.16471 fig 2 (12 uniform `0.245\textwidth` PDF panels) split 3/3/2/3/1
instead of 3 rows of 4. **Fix** (`latexml_core/src/util/image.rs`) emulates
pdfTeX, not Perl: on a raster-reader miss, read the natural size from the file
itself — a PDF's CropBox→MediaBox (pdfTeX's default, shared with
`LaTeXML::Post::Graphics::read_pdf_page_box`) or an SVG's viewBox — and apply the
graphicx transform in points. Only when the page box is hidden in a compressed
object stream do we fall back to the requested `width=` (else 0); `cached_width`
is always set so the filename is never summed. No ImageMagick dep (that is a
Perl-only workaround for Image::Size's lack of PDF support; even it forces
`use-cropbox` to match pdfTeX). Verified against `\the\wd` under pdflatex:
`width=` → the request outright; bare/`scale=`/`height=` → the natural box.
Corpus-wide reach but NARROW — `width=` figures get an identical box width either
way, so only no-explicit-width PDF/SVG figures change; a 260-paper before/after
sample (142 with PDF figures) showed 0 error/fatal/exit-code regressions, and the
2 layout changes were previously-merged multi-panel figures now wrapping into
rows (e.g. 8 panels → 2 rows of 4). Golden suite untouched (all-PNG/JPEG).
Regression tests `figure_panel_native` + `figure_panel_unmeasured`. See WISDOM
#62. Fig 2 → uniform 84.52pt → 3 rows of 4, 0 errors.

### `\halign`-in-math runaway (Cluster H #2 / kbordermatrix) — ✅ LANDED 2026-07-20

Rust Error Fix, **surpass-Perl**. The long-standing "HIGH difficulty, post-release"
`\lx@begin@alignment`/`\halign`-in-math crash turned out to be a one-line
**inherited-kernel-macro leak**, not deep frame surgery.

Rust raw-loads `latex.ltx` into the kernel dump, so it has the real
`\@arraycr`/`\@xarraycr` (L16583-16585); **Perl LaTeXML has neither**. That body
balances TeX's `align_state` with ``${\ifnum0=`}\fi … \ifnum0=`{\fi}${}\cr``,
valid only under a real `\halign` — digested by LaTeXML it re-opens an inline-math
frame the alignment's column-after template cannot balance. Any macro using the
documented `\bordermatrix` idiom `\let\\\@arraycr` inside its own `\ialign`
therefore leaked → `Attempt to close a group that switched to mode math` → runaway.
Fix: `Let!("\\@arraycr", "\\lx@alignment@newline")` in `latex_constructs.rs`,
beside the `\@tabularcr` retraction Perl already performs
(`latex_constructs.pool.ltxml:3612`).

- **arXiv:2605.23849** (the Cluster H witness): ~149 s runaway → token-limit Fatal,
  **0 formulae** ⇒ **1.9 s, 0 errors, 985 formulae / 8 XMArray, 1.34 MB**. Same-host
  Perl: 52.7 s, 3 errors, identical 985/8 counts — so Rust is now faster AND
  error-free at equal structure.
- **arXiv:2605.05194** (found by corpus scan): 125 errors + `Fatal:TooManyErrors`
  and a **39-byte** (empty) document ⇒ **0 errors, 422 KB**.
- Breadth **6 / 6,000** 2605 papers (0.1%); the other hits are byte-unchanged.
  Neutral by construction — no Rust binding and no `.ltxml` names `\@arraycr`.
- Suite **1614/0**, clippy clean. Guard `tests/alignment/arraycr_halign`.

**Two prior hypotheses were wrong; do not retry them.** (a) "Make `egroup`'s
mode-switch recovery degrade like Perl" — Perl was *skipping* the matrix (its `\\`
was undefined), not recovering, so matching its error count would have meant
matching a content loss. (b) The `\lastbox`/`\unhbox` box-peel repro is a
different, SHARED loop. See WISDOM #64 for the reusable bisection method
(hand-expand the suspect macro) and `docs/known_crashes/kbordermatrix_halign_math/`.

### Stale-autoload-trigger runaway (Cluster H #1 + #3) — ✅ LANDED 2026-07-20

Rust Error Fix. The remaining two Cluster H runaways — long framed as "Rust
error-recovery *loops* where Perl keeps *advancing*" and expected to need
separate per-mechanism gullet surgery — were **one bug**, in `def_autoload`
(`latexml_engine/src/tex.rs`).

The autoload closure's "package already loaded → just re-emit the trigger CS"
branch is correct only when a **different** CS was `\let` to the trigger (the
`\varmathbb` case it was written for, arXiv:2310.13684). But `<pkg>.sty_loaded`
is assigned **globally** while the package's macros install at the current
frame, so loading a package or class **inside a group** pops the macros and
keeps the flag — leaving the globally installed trigger as the CS's only
definition. It then re-emits *itself* forever, and because it emits **no
`Error:`**, the `too_many_errors` cap is never reached; the run grinds ~42 s to
the token limit and writes a 39-byte document. Fix: when the CS that fired the
closure IS the trigger itself, clear the stale trigger globally so the CS takes
the ordinary bounded undefined path.

- **2606.21610** (Overleaf/Springer `\IfFileExists{sn-jnl.cls}{\documentclass…}`
  template): 42.9 s `Fatal:Timeout:TokenLimit`, empty output ⇒ **0.203 s**,
  bounded `Fatal:TooManyErrors:MaxLimit(100)`. Perl: 1.1 s / 102 errors /
  `too_many_errors:100` — same verdict, **5× faster**.
- **2605.21013** (undefined-macro cascade, was `Fatal:Timeout:IfLimit` at 107 s):
  43.1 s ⇒ **0.203 s**, same bounded verdict. Perl 1.9 s — **~10× faster**.
- Both papers are genuinely broken LaTeX (pdflatex fatals too), so the win is
  *failing like Perl instead of grinding*, not converting them.
- Known `def_autoload` regression traps re-verified clean: 2310.13684 (0 err),
  1403.6801 (0 err), 1711.11576 (1 err, 3.5 MB).
- Suite **1615/0**, clippy clean. Guard `tests/100_stale_autoload_no_runaway.rs`
  (6-line self-contained repro; verified red at 54.7 s without the fix).

**Ground truth, recorded but deliberately NOT ported:** real LaTeX rejects the
premise outright — `\@fileswithoptions` (latex.ltx L18700) errors *"Loading a
class or package in a group"* when `\currentgrouplevel > 0`. Porting that guard
would give a better message, but `standalone_sty.rs` **deliberately** wraps its
`\@standalone@documentclass` in `bgroup()` + `RequirePackage`, so the guard
would need an internal-load exemption. Not worth the risk now that the runaway
is gone; noted here if the diagnostic is ever wanted. **Updated 2026-07-23
(#311):** the exemption is still required — that wrapping is unchanged, and
faithful to Perl — but the *harm* it caused is gone: `require_package` now
hoists a load's definitions past the enclosing group, so a package loaded in a
subfile preamble no longer loses its `\newif`s while the hooks reading them
survive (OXIDIZED_DESIGN #65, KNOWN_PERL_ERRORS #55). We reproduce latex.ltx's
invariant instead of its enforcement.

**Diagnostic gap closed alongside:** the `TokenLimit` fatal previously printed
only "infinite loop?" with no window — the cycle guard dumps its repeating
tokens, but a run that reaches the *token limit* is by definition one the cycle
guard did not recognise, i.e. exactly the case with no other clue. It now dumps
the same recent-token ring under `LATEXML_DEBUG_FATAL` (and the ring fills
before the guard activates, so a lowered `LATEXML_TOKEN_LIMIT` still captures
it). That dump is what identified this bug in one run.

### Reproducer re-verification + 400-paper output-neutrality sweep (2026-07-20)

Validation pass for the two fixes above, which also **re-dated every committed
reproducer**. Both halves changed the worklist more than the fixes did.

**A. 400-paper corpus sweep, baseline (`381efaf81b`) vs fixed, same sample.**
`0` error-count changes, `0` fatal-class changes, total wall 575.9 s → 578.9 s
(+0.5%, noise). 26 papers differed by 1–21 bytes — **re-running those solo gave
byte-identical output from BOTH binaries**, so that is run-to-run
nondeterminism under parallel load, not a behaviour change. Neutrality of the
`\@arraycr` retraction is anyway structural: nothing else in the tree names it.

*Sweep-harness caveat worth reusing:* a naive `grep -rl '\begin{document}'`
main-file pick manufactured 2 of the 4 apparent "fatals" (it chose
`figures-pgf/tinylora_preamble.tex` and a fragment instead of the real main) —
the trap `SYNC_STATUS` already records for the bibliography sweeps. With the
right main, **all four are fine**: `2605.30585` Rust 0.2 s/102 err vs Perl
2.0 s/102 err (exact parity, 10× faster); `2605.12207` Rust 0.3 s/39 err vs
Perl **3 m 57 s**/47 err; `2605.14493` and `2605.25400` fail in BOTH engines,
Rust in 15 s / 8.5 s vs Perl timing out at 200 s. **Zero Rust-only regressions
in the sample.**

**B. Every committed reproducer re-run against same-host Perl.** Several
long-standing "OPEN, GENUINE-RUST-ONLY" entries are **already fixed** — they
were stale, and left in place they mis-rank the whole worklist:

| reproducer | recorded | measured 2026-07-20 |
|---|---|---|
| `1610.00974_multicolumn_pcell_newline` | OPEN, Rust-only, 502 err + Fatal | **0 err**, and the full paper `Nikbakht.tex` **0 err** |
| `array_pcolumn/B_prefix_alignment_td_align` | OPEN (`align="justify"` vs Perl `left`) | **byte-identical to Perl** |
| `array_pcolumn/C_m_column_vbox_rendering` | OPEN, deferred (2 structural diffs) | **byte-identical to Perl** |
| `pcolumn_block_content_in_p` | OPEN, **BLOCKED** on the `\hsize`-invariant box model | **byte-identical to Perl** — that blocker no longer gates it |
| `ieeeeqnarray_leading_empty_cell` | SHARED (both engines fail) | Rust **0** / Perl **5** — the surpass-Perl half is done |
| `tabbing_math_code_env_2311.06609` (ar5iv #472) | Rust-worse | **11 = 11**, parity on the repro |

`1610.00974` keeps one structural difference from Perl, and **pdflatex says Perl
is the wrong one**: for `\multicolumn{2}{|p{1cm}|}{\centering A\\ B}` Rust makes
`B` a line break *inside* the merged cell while Perl opens a new `<tr>`;
`pdftotext -layout` stacks A/B in the single merged cell with `y z` as the next
row. Do NOT "fix" Rust toward Perl there.

The only reproducer still genuinely Rust-worse is `glossaryref_math_xmtok`
(Rust 12 / Perl 1) — and that Perl `1` is a **timeout kill**, not a clean run
(`rc=124`), confirming the recorded "blocked on an unrunnable Perl reference"
verdict is still current. **Method note:** the first pass of this table was
wrong for exactly that reason — always capture the exit code, or a
timeout-killed Perl reads as a 1-error success and flips the verdict.

### A recoverable Fatal no longer throws the whole document away — LANDED 2026-07-20

Rust reliability fix (**beyond-Perl**). `digest_internal` is written to keep
partial output after a recoverable Fatal ("Perl `finishDigestion` L219-220: loop
consuming input even after errors"), but the intent only worked when the failure
landed in a **later** body: `digest_next_body` accumulates into the stomach's
`box_list` and hands it back only on the success path, so a Fatal inside the
FIRST body left the caller's `boxes` empty and the run wrote a **39-byte empty
document**. One pathological `\tikz` picture cost an entire paper.

New `stomach::salvage_pending_box_lists` unwinds the stranded levels in document
order. Results on ar5iv user-report papers, all previously **0 bytes**:

| paper | issue | now |
|---|---|---|
| `2405.19920` | #522 | **1.82 MB** — 6 sections + **80 bibitems**, ~the complete paper. Same-host Perl: **5 min, 0 bytes**. |
| `2508.07407` | #556 | **31 KB** — title/authors/abstract recovered |

**Scope was narrowed by measurement, twice — both narrowings matter:**
1. For the stomach box-cycle guard the innermost level IS the pathology (a
   repeating window past 50k boxes), so it is dropped and only the suspended
   outer levels kept — "drop the offending construct, keep the document".
   Grafting the window in would produce a vast garbage document.
2. **Salvage fires ONLY for `ErrorTarget::Stomach`.** Extending it to the
   gullet's `Timeout:Recursion` looked reasonable (the token stream vs the box
   list) and was actively harmful: on `2605.25400` it revived a poisoned state
   that re-entered the same loop during build, turning an 8.7 s fatal into a
   **2 m 12 s wall-clock timeout writing a ZERO-byte file** — strictly worse
   than the 39-byte stub it replaced, for a 1.7 KB gain on the single paper it
   helped. The same reasoning bars `TooManyErrors`. Widening to either needs its
   own measurement; do not assume more salvage is better.

Validation: suite **1617/0**, clippy clean, and the 400-paper sweep vs
`381efaf81b` shows **0 error-count and 0 fatal-class changes**, wall 575.9 s →
579.5 s. Guard `tests/101_fatal_salvages_partial_document.rs` (verified red —
39 bytes, prose gone — without the fix). It asserts the Fatal is still reported:
salvaging partial output is not a licence to downgrade the diagnostic.

**Corrects a stale claim:** `docs/reproducers/tikz_calc_node_recursion_2508.07407.tex`
and the AR5IV notes said this fatal was "caught gracefully — conversion
COMPLETES, only the one tikz table is dropped". Re-measured, the full paper
produced a **0-byte** file. It is graceful *now*.

## ⏸️ HANDOFF — session of 2026-07-20 (branch `more-minisprint-ar5iv`, 13 commits, **NOT PUSHED**)

**Resume here.** Working tree clean; `frontmatter_bug_ids.txt` is a pre-existing
untracked scratch file, not mine. Suite **1618/0**, clippy clean.

### Landed (each with a red/green guard, full suite + clippy green)
1. **`\@arraycr` retraction** — ended the `\halign`-in-math runaway. 2605.23849
   ~149 s→Fatal ⇒ **1.9 s / 0 errors / 985 formulae**; 2605.05194 ⇒ 0 errors /
   422 KB. Now surpass-Perl.
2. **Stale-`def_autoload` guard** — Cluster H #1 and #3 were ONE bug.
   2606.21610 42.9 s ⇒ 0.203 s; 2605.21013 43.1 s ⇒ 0.203 s, both landing on
   Perl's own verdict 5–10× faster.
3. **`salvage_pending_box_lists`** — a Stomach Fatal no longer discards the
   document. 2405.19920 (ar5iv #522) 0 bytes ⇒ **1.82 MB**; 2508.07407 (#556)
   ⇒ 31 KB.
4. **Issue #312 operand slot** — see the caveat below.
5. **Docs vetting** — three commits; see "what changed" in git log.

### Open threads, in the order I'd pick them up

- **#312 is NOT demonstrated fixed.** The structural divergence is repaired
  (we match Perl's continuation-row shape again), but Chrome renders identically
  with or without the slot, and I could not get a working MathJax measurement
  (only v2 installed, it did not typeset headless). **Next step:** render the
  reporter's document under MathJax 4 and compare, before replying on the issue.
  Note their *other* complaint — "equations are not centered" — is **parity**:
  the `ltx_eqn_table`/`ltx_eqn_center_pad*` markup and equation CSS are
  byte-identical to Perl's on their file.
- **expl3 catcode gap closed; the "regressed" witness was a different bug — now
  fixed.** 2112.11932 1003⇒0, 2110.10227 102⇒0, 2204.05282 86⇒0, 2110.12034
  45⇒8. **2203.05327 78 ⇒ 411 ⇒ 0**: the 411 was NOT the catcode gap — it was one
  amsmath `align` breaking (`\lx@begin@alignment` group/mode) because
  `aligned-overset.sty` was raw-loaded under ar5iv; the `unexpected:_` flood was
  downstream. Fixed with a near-no-op `aligned_overset_sty.rs` contrib binding
  (411⇒0, 831 KB⇒5.1 MB whole paper; Perl still dies `token_limit` → beyond-Perl).
  Guarded by `102_aligned_overset_includestyles.rs`.
  The **TL2026 dump-gate blocker may still be closer than recorded** —
  re-run the init gate on a TL2026 host.
- **ar5iv residuals — DONE (2026-07-20 second pass).** All three now have
  same-host Perl baselines; all resolve parity-or-Rust-better, none Rust-only.
  2405.19920 = Rust-better (salvage 1.82 MB, Perl 0 B); 2501.10235 (#551) and
  1802.01134 (#599) = **parity** — both engines hang in shared deep machinery
  (pgfplots pgfmath coord processing at `river_cps.tex:117`; the paper's own
  `imgresize` `\wd0` box-convergence 2-cycle) and emit 0 B, Perl killed at the
  6-min cap while Rust self-terminates via its guards. No faithful fix without a
  box-measurement divergence. See the AR5IV_DIAGNOSTICS re-measurement block.
- **`latexmlmath_oxide` single-structure formula** and **`--preload=<cls>` hook
  stack** — both re-verified as still reproducing exactly as documented above.

### Cross-repo state (both pushed, both mine to finish)
- **PR #310** (`fix-309-standalone-class-options`) — reviewed, then improved:
  the option allowlist was hand-split on `,` and missed every valued form
  (`[varwidth=5cm]` → `Error:undefined:{varwidth}`, pdflatex clean). Now read as
  `OptionalKeyVals`, matched on the key. **CI fully green.** Ready to merge.
- **Upstream Perl PR brucemiller/LaTeXML#2852** — same bug, same fix ported
  (`OptionalKeyVals` + `getPairs`), plus a `t/structure` case that actually
  guards it. Pushed to `dginev/LaTeXML`; CI was 11 pass / 4 pending at handoff —
  **check it before asking for review.**

### Two traps that cost me time — worth keeping
- A **fresh git worktree has no `resources/dumps/`**, and the suite then fails
  26 expl3/dump-dependent tests (`glossary_test`, `regex_*`, `str_*case_*`,
  `xparse`, mhchem, si). Copy the dumps in before suspecting code.
- **Capture Perl's exit code.** A timeout-killed `latexml` prints one line that
  a naive `grep -c '^Error:'` reads as "1 error", which flips a verdict from
  "Perl times out" to "Perl is better". It did exactly that to me once.

## Methodology & the cortex cross-join

Working method (2026-06): **re-triage LARGE-error papers** (the single-error tail
is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose`
→ fix the divergence. Random sweeps are low-yield.

**Cortex agentic API (reads open, no token):** `http://127.0.0.1:8000/api`.
Recipe: `GET /api/reports/<corpus>/oxidized-tex-to-html/<severity>` → categories;
`…/<severity>/<category>` → per-`what`; `…/<category>/<what>` → paper list. Then
`GET /api/corpus/<corpus>/tex_to_html/document/<id>` for Perl status — a Rust-only
win is **Perl=no_problem/warning but Rust=error/fatal**. Corpus
`sandbox-arxiv-10k-shuffle`. URL-encode `\`→`%5C`, `^`→`%5E`.

## CLI options (#191) + `validate()` implementation — ACTIVE (public-release-prep-week)

Completing issue #191 "support the original latexmlc/latexmlpost options" under
the **option-C policy**: wire only options whose engine feature genuinely works
end-to-end; keep the clap parser **strict** (no accept-and-warn stubs); deferred/
missing features stay hard parse errors.

### Landed this session (real features, verified + committed)
- `--timestamp=STR` (`--timestamp=0` omits) → XSLT `TIMESTAMP` footer param;
  deterministic no-timestamp default (divergence from Perl's localtime).
- `--icon=FILE` → XSLT `ICON` param + favicon resource copy.
- `--nographicimages` / `--graphicimages` → gate the Graphics post-phase.
- `--numbersections`, `--mathparse`, `--invisibletimes`, `--defaultresources`
  → positive complements of existing negative-only flags (verbatim Perl-CLI
  parity; the negative wins if both are given).

### Deferred — feature genuinely NOT supported (do NOT stub)
- `--parse=STRATEGY` — grammar selection unsupported (one Marpa grammar);
  `--nomathparse` / `--mathparse` is the real interface. (Attempted + removed.)
- `--svg` / `--nosvg` — **deferred (verified 2026-07-09):** the HTML5 XSLT
  already renders `<ltx:picture>` as inline `<svg>` by default, so the standalone
  `svg.rs` post-processor (`impl Processor for SVG`, unwired) is redundant and
  produces divergent, unverified output (25 vs 27 `<svg>` on `tests/graphics/
  picture.tex`). Wiring it was built + reverted.
- `--pictureimages` / `--nopictureimages` — `picture_images.rs` delegates to the
  **unwired LaTeXImages latex+dvipng pipeline** (`latex_images.rs`); same
  category/effort as `--mathimages`.
- `--openmath|om` — no OpenMath serializer. (User: defer.)
- daemon net (`--port` / `--address` / `--expire` / `--autoflush` / `--cache_key`)
  — socket-daemon model; we ship `--server` (stdio LSP). (User: defer.)
- `--mode` (= alias for `--profile`); `--profile=NAME` — needs a preset registry.
- `--mathimages` / `--mathsvg` / `--mathimagemagnification` — needs a
  latex+dvipng math-render pipeline.
- `--unicodemath` / `--plane1` / `--hackplane1` / `--linelength` — plain/unicode
  math output modes.
- crossref cluster (`--crossref` / `--scan` / `--noscan` / `--urlstyle` /
  `--prescan` / `--dbfile` / `--bibliography` / `--splitbibliography`) + index
  cluster (`--index` / `--permutedindex` / `--splitindex`) — multi-doc site-DB
  features. (Scan IS wired as post Phase 2, so `--noscan` is a real-but-risky
  off-switch; parked with the cluster.)
- `--tex` / `--box` — intermediate box/tex serializers absent.
- `--omitdoctype` — DTD-only in Perl; Rust has no DTD (moot).

### `validate()` / `--validate` — POSTPONED to the NEXT release (decided 2026-07-09)
Today `Post::Document::validate()` (`latexml_post/src/document.rs:1717`) is a
STUB: it logs "Would validate against RelaxNG schema" and returns `Ok(())`.
Real RelaxNG validation is wanted, but is **deferred to the next release** because
it is gated on a `rust-libxml` crates.io publish (see below). Reference: Perl
`LaTeXML/lib/LaTeXML/Common/XML/RelaxNG.pm` + `LaTeXML/lib/LaTeXML/Post.pm`.

**Architecture decision (owner, 2026-07-09): `rust-libxml` provides the public,
safe Rust RelaxNG interface; `latexml-oxide` is a pure consumer.** All libxml2
`unsafe`/FFI stays in the fork — the alternative (raw `xmlRelaxNG*` FFI inline in
`latexml_post`, which would compile against the shipped crates.io `libxml 0.3.15`
with no publish) was **rejected**. So this feature cannot fully land until the
fork's RelaxNG module is published as `libxml 0.3.16`.

Constraint: the schema is **modular** (`LaTeXML.rng` `<include>`s
`LaTeXML-common.rng`, `-structure`, `-math`, …) and the binary is
**self-contained** — no on-disk schema. Includes MUST resolve through the
embedded table (`latexml_core::common::relaxng::embedded::lookup`), served via
the fork's existing `libxml::io::register_input_callback` (built for exactly this
— "bundles RNG schemas via include_bytes! … RelaxNG `<include>` via
`xmlRelaxNGParse`"), NOT disk.

Steps (next-release session):
1. **rust-libxml fork — add a safe `relaxng` module.** The fork's `schemas`
   module is **XSD-only** (`xmlSchema*`). Mirror it: `relaxng/{parser,schema,
   validation}.rs` wrapping `xmlRelaxNGNewParserCtxt`(URL — so relative includes
   resolve through the callback) / `xmlRelaxNGNewMemParserCtxt` + `xmlRelaxNGParse`
   (→ `RelaxNGSchema`) and `xmlRelaxNGNewValidCtxt` + `xmlRelaxNGValidateDoc`
   (→ `RelaxNGValidationContext`), with `xmlRelaxNGSetValidStructuredErrors`
   capture. Fork unit test (valid + invalid doc). **Publish `libxml 0.3.16`.**
2. **Embedded-include resolution** via `libxml::io::register_input_callback`
   (`embed:///RelaxNG/LaTeXML-*.rng` → `embedded::lookup`); verify with the
   renamed-`resources/` smoke that no schema is read from disk.
3. **Consume in workspace** — bump the `libxml` dep `0.3.15` → `0.3.16`; `cargo test`.
4. **Flesh out `validate()`** — parse+cache the schema once; run `validate_doc`;
   map each captured `StructuredError` to a `Warn!` / `post_error` in the project
   logging convention (Perl reports schema violations).
5. **Wire `--validate` / `--novalidate`** — CLI flags + `PostOptions.validate`;
   call `validate()` in `run_post_processing_impl` when enabled. DEFAULT
   decision: Perl defaults ON; propose **opt-in** in Rust (validation cost +
   corpus warning noise) as a documented divergence — confirm with owner before
   flipping the default on.
6. **Tests** — a valid fixture validates clean; an intentionally schema-invalid
   doc reports the expected violation; `--novalidate` skips.

## Math-parser / content-MathML gaps — DEFERRED to a dedicated session

> **User directive 2026-06-20: defer ALL content-MathML items to a dedicated
> session** (the math parser is a full Marpa-vs-RecDescent rewrite; these touch
> the parse-tree / content-MathML structure and want a focused regression
> budget). Notes kept here; do NOT pick at them piecemeal.

- **`f(a,b)` multi-arg flattening — FIXED 2026-06-22.** A KNOWN function applied
  to a paren comma-list now flattens: `\max(a,b)`→`maximum@(a,b)` (was
  `maximum@(vector@(a,b))`), matching Perl `ApplyDelimited`/`extract_separators`.
  Implementation was simpler than the planned grammar-rule approach: a post-parse
  spread in the `prefix_apply` ACTION (`semantics.rs`, helper `vector_tuple_items`)
  — when a function-role op (FUNCTION/OPFUNCTION/TRIGFUNCTION) applies to a
  `Dual` whose content is `Apply(vector, [refs])`, spread the items as direct
  operands instead of wrapping. No grammar/pruning change → NOT pruning-sensitive,
  zero fixture regressions. Scoped to known function roles, so unknown-`f` apply
  (`f(a,b)`→`f@(vector@(a,b))`) is untouched — the intentional divergence #18.
  Verified Perl-identical: `\max(a,b)`/`\gcd(a,b)`/`\min(x,y,z)`/`g(a,b,c)` +
  nesting/`\frac`/trailing-ops; suite 1466/0; regression test in
  `parse/functions`. (Known pre-existing aside: juxtaposed `\max(a,b)\min(c,d)`
  greedily reads `\max` over the product — a separate function-juxtaposition
  pruning issue, not this flatten.)
- **`f(x)` single-arg apply-vs-multiply** (most PERVASIVE divergence): for an
  UNKNOWN/undeclared symbol + paren arg, Rust reads *application*, Perl reads
  *multiplication* — `\Gamma(s)`→Rust `Gamma@(s)` vs Perl `Gamma * s` (likewise
  `\zeta(s)`, `\Phi(x)`, `f(x)`). A real fix must respect Perl's "only declared
  FUNCTION/known-operator names apply; bare letters multiply" rule; heavily
  pruning-sensitive.
  > **SURVEY 2026-06-22 (current-state + blast radius — groundwork, NOT yet
  > changed):** confirmed the split cleanly — KNOWN functions ALREADY match Perl
  > (`\sin(x)`/`\log(x)` → `sine@(x)`/`logarithm@(x)` in both); only UNKNOWN
  > symbols diverge (`f(x)`/`g(x)`/`P(x)`/`\Gamma(s)`/`\zeta(s)`/`\phi(x)` →
  > Rust `X@(x)` vs Perl `X * x`; `f(x+1)` → Rust `f@(x+1)` vs Perl `f * (x+1)`).
  > LEXER ROLE: unknown `f` = `role="UNKNOWN"`, `\max` = `role="OPFUNCTION"` — so
  > the apply-of-UNKNOWN (A) is separable from the known-fn flatten (B). BLAST
  > RADIUS of A is corpus-wide: 25 test fixtures, ~150 single-letter applies
  > (`f@(`×57, `d@(`×51, `g@(`×13, …) would flip to multiply — a sweeping change
  > that reshapes all math output. Because A is corpus-wide (even though
  > toward-Perl), it needed explicit scope sign-off; B (below) was the
  > contained first step (~5 fixtures).
  > **DECISION FINAL 2026-07-02: divergence #18 STANDS — `f(x)` leans toward
  > function application.** The toward-Perl flip was green-lit earlier the
  > same day, fully implemented (12/12 witness parity with Perl, ~22 fixtures
  > verified toward-Perl), and then **REVERTED on user review**: "f(x) is
  > almost always an application in common STEM use." The apply-of-UNKNOWN
  > reading is the settled intentional divergence (OXIDIZED_DESIGN #18,
  > re-affirmed). The reverted implementation is preserved on branch
  > `archive/fx-perl-parity-attempt-2026-07-02` (local) for reference — do
  > NOT re-attempt the flip without a fresh explicit user decision.
- **`[a|b]` / `[a \mid b]` bracket-conditional — FIXED 2026-06-22.** Was unparsed
  in Rust; now `delimited-[]@(conditional@(a,b))` matching Perl (`E[X|Y]` etc.).
  Root: the bare `a|b` conditional reduces only at statement level (not as an
  `expression`), so `[a|b]` had no fence rule — though `[(a|b)]` already worked.
  Fix: a surgical grammar rule `lbracket formula singlevertbar formula rbracket =>
  bracket_conditional` (`singlevertbar` also covers `\mid`) + a `bracket_conditional`
  action (semantics.rs) that builds the inner `conditional@(a,b)` (delimiter-less
  presentation) and wraps it in `delimited-[]` via the same `fenced` path
  `[(a|b)]` uses (ctxt reborrow for the two ref levels). Suite 1466/0, clippy
  clean, zero other-fixture changes; regression test in `parse/vertbars`. (The
  `E` in `E[X|Y]` stays `E@(…)` apply vs Perl `E * …` — divergence #18, preserved.)
- **`⁡` DecorateOperator over-insertion — FIXED 2026-06-22.** Presentation MathML
  emitted `⁡` (U+2061 FUNCTION APPLICATION) after operators that render as
  `<m:mo>` — `\nabla \phi`→`∇⁡ϕ`, `\partial f`→`∂⁡f`, and (pre-existing) `\sum_i
  a_i`→`∑⁡a_i`, `\int f`→`∫⁡f` — where Perl juxtaposes (∇ϕ/∂f/∑a/∫f). Perl's rule
  (MathML.pm `Apply:?:?`): insert `⁡` only when the op base is NOT an `<m:mo>` (a
  function identifier `f`/`\sin`/`\max` IS `<m:mi>` → keeps `⁡`). FIX
  (`latexml_post/.../presentation.rs`): new `op_base_is_mo` helper (descends
  msub/msup/munder/mover to the base); applied at the generic-apply site AND in
  `pmml_summation`; and removed `DIFFOP` from the big-op→`pmml_summation` route
  (Perl MathML.pm:702 `# Not DIFFOP`). Suite 1466/0, clippy clean; verified
  Perl-identical for ∇/∂/∑/∫/∏/⋃/lim + `\sin`/`\max`/scripted forms; only residual
  diff is the `f(x)` apply-vs-multiply (`f⁡(` vs `f⁢(`) — divergence #18,
  preserved. Regression test in `tests/post/opdecoration`.
- **wide-space PUNCT XMDual content-arm XMRef ordering**: `x^2\quad y` — the
  `\quad` (≥10pt) becomes a virtual PUNCT through `formulae_apply`, producing an
  XMDual whose content-arm XMRef siblings emit one slot off from Perl. Same
  MathFork/split content-arm xml:id family as the `expected:id` tail
  (`EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`). NOT the rpadding path (thin spaces
  `\,` are Perl-faithful incl. NewScript transfer, `005716ff66`).
- **`\DeclareMathOperator` cluster — INVESTIGATED 2026-06-22, LOW-VALUE metadata,
  deprioritized** (`text=` and cMML already match): (a) Perl splits Math attrs
  `tex="\operatorname{Tr}…"` vs `content-tex="\Tr…"` (Perl defines `\Tr` *via*
  `Invocation(\operatorname,…)` + `revert_as=>'context'`); Rust defines it
  directly so `tex` keeps the user macro `\Tr` (arguably MORE source-faithful) and
  emits no `content-tex`. Matching Perl needs the deep `revert_as=>context`
  content-tex mechanism — high effort, metadata-only value. (b) The `name="Tr"`
  "gap" is NOT a bug: `def_math` (dialect.rs:1567) DOES infer `name` from the CS
  but DROPS it when `name == presentation` (line ~33) — a deliberate
  redundant-attr cleanup. `\Tr` (name "Tr" == content "Tr") drops it; `\argmax`
  (name ≠ "arg max") keeps it. Perl always emits it. Changing this touches the
  GENERAL def_math path (every math token) for cosmetic value → not worth it.
  (c) `\DeclareMathOperator*` `scriptpos` in display mode — the remaining
  candidate if revisited, but mode-dependent and niche. Whole cluster parked.
- **N-ary bare-operator listing — ✅ NOW WORKS (verified 2026-06-27); note was
  STALE.** `+,-,\times,\div` → `list@(+,-,*,/)` (Perl-exact); `+,-`, `+,+`, `a,+,b`,
  `++`, `+-` all parse and match Perl. An intervening fix (likely the comma-list /
  marpa-drain work) closed this. NOT an open gap anymore. The truly-remaining
  operator-script cases are narrower and finicky/context-dependent: `\Omega_{+,+-}`
  (a comma-list-of-operators in a SUBSCRIPT — Perl's subscript grammar parses it as
  `list@(+, absent + -)`, Rust's doesn't; note `+,+-` STANDALONE is PARITY-unparsed
  in BOTH), and operator-scripts where both parse but DIVERGE structurally
  (`a^{++}`: Rust `a^(list@(+,+))` vs Perl `a^(absent + +)`). These are the deferred
  math-fork session (subscript-content grammar + scripted-operator structure).
- **comma-list LEFT of a relation `a,b \in A` — FIXED 2026-06-22 (2-item path).**
  Was the wrong `formulae@(a, b∈A)` (∈ binding only `b`). Now the user-specified
  surpass-Perl **XMDual**: content **DISTRIBUTES** — `formulae@(∈(a,A), ∈(b,A))`,
  sharing XMRefs to the relop and RHS — presentation wraps the list as the
  relation's LHS — `Apply(∈, XMWrap(a,',',b), A)`. Implemented as a scoped
  transform at the end of `formulae_apply` (semantics.rs): when `left` is a bare
  (non-relational, non-Dual) item and `right` is a binary RELOP relation
  `Apply(R,[lhs,rhs])` under a comma, `distribute_list_relation` builds the dual.
  `x,y \le z`→`formulae@(x≤z, y≤z)`. The list-RIGHT `0<x,y`→`list@(0<x,y)`,
  all-relational `a=b,c=d`→`formulae@`, and bare `a,b`→`list@` all stay. Full suite
  1466/0, clippy clean, zero other-fixture changes; regression test in
  `parse/relations`. **Remaining (follow-up):** the 3+-item `a,b,c \in S` goes
  through `list_apply` (not `formulae_apply`) → still `list@(a,b,c∈S)`; the same
  distribution needs porting to that path.
- **relation with a list-RHS that itself contains a scripted relop**:
  `a \le b \quad \stackrel{?}{\ge} \quad c` → Perl `a <= list@(b, >=^?, c)`.
  **UPDATED 2026-06-27: no longer `ltx_math_unparsed` (stale)** — Rust now PARSES
  it as `fragments@(a <= b, >= ^ ?, c)` (the `\quad`-WIDE_PUNCT routes it through
  `formulae_apply`→`fragments@` rather than the relation-with-list-RHS shape). So
  it's now a STRUCTURAL divergence (fragments@ vs `a <= list@(…)`), not a parse
  failure. Lower-severity (renders) cMML-structure item; the scripted-relop atomic
  fix (`4a5ebf29f7`) cleared standalone list items.
- **`\underset`/`\overset` over an ARROW with a multi-token script**:
  `x \underset{n\to\infty}{\to} y` — the under-script reads `n@to@infinity`
  (apply) where Perl groups `(n to infinity)`. Same ARROW-as-applied-function
  family as `f(a,b)`.

CAUTION: new VERTBAR/fence grammar rules can collide with package-built
structures — always cross-check the affected fixture against Perl before
assuming a regression (the norm rule "regressed" physics_test, but Perl matched
the new output, so it was a parity *fix*).

### Archived-audit residuals (2026-07-09 docs compaction)

Two completed diagnostic snapshots were dated + archived; their still-open
residuals stay here so the live worklist keeps them visible:

- **MathML-post line audit** (sweep complete; →
  `archive/MATHML_POST_LINE_AUDIT_2026-07-05.md`). Open feature-gaps: **F5**
  Linebreaker (full feature gap — the sketch used the wrong strategy), **F11**
  Hint width normalization, **F14** multirelation + lt-or-approx cMML, **F15**
  continued-fraction, **F16** OperatorDictionary Cat A/B data holes + U+2A50
  misclassification + fence U+0331, **F17** formulae pMML arm, plus PARTIAL
  inherited-context bindings on `pmml_top`/`pmml_parenthesize`/`stylizeContent`.
  (Content-MathML items obey the defer-to-a-dedicated-session directive above.)
- **arXiv velocity-fork audit** (items 1–4 landed 2026-07-03; →
  `archive/ARXIV_FORK_AUDIT_2026-07-03.md`). Sole residual: **item G** —
  `readBalanced` drops comment tokens (fork `4e1578d1`); Rust `read_balanced`
  still flushes `pending_comments` (gullet.rs ~L1170). Low urgency
  (`INCLUDE_COMMENTS=false` default); port at the next gullet-seam session.

## Open tasks (actionable)

### Rhai `LookupDefinition(cs).push*` hook-splice re-installs at same-level, not global — ✅ LANDED 2026-07-21

Follow-up to the BookML/@xworld21 cluster: PR #333 review comment r3623947537 flagged that the
`LookupDefinition(cs).push*/unshift*` hook-splice (#321) re-installed the patched def at
`Scope::Global`, which **promotes a locally-bound def to global** and makes the patch survive
group exit — a divergence from Perl, which mutates the shared def-hash *in place* (never touching
the save stack). Harmless in practice (BookML only patches already-global `\hrule`/`\vrule`/`\rule`),
but a real gap. Fix: ported Perl `State.pm:175`'s fourth scope, `'inplace'` ("Special case for
`\box` & friends"), as first-class **`Scope::InPlace`** (`latexml_core/src/state.rs` enum +
`assign_internal` arm; `\globaldefs` deliberately does NOT re-scope it, matching Perl's
`$scope ne 'global' && ne 'local'` guard). The 9 `install_definition(d, Some(Scope::Global))` sites
in `script_bindings/wire.rs::push_definition_hook` now pass `Scope::InPlace`. The Value-table
`assign_value_inplace` fast path (WISDOM #19) was the pre-existing witness that this scope existed;
this lifts it to the Meaning table too. Guards: `state::reentrancy_tests::inplace_scope_keeps_the_bindings_level`
(proves neither-Global-nor-Local across a `push_frame`/`pop_frame` boundary) + the existing
`script_bindings::tests::lookup_definition_*` (unchanged — top-level pushes, where in-place ≡ global).
See WISDOM #48.

### XSLT `LATEXML_VERSION` param — generator-stamp parity gap + BookML `utils.xsl` — ✅ LANDED 2026-07-21 (branch `xslt-latexml-version-param`)

Completes the BookML/@xworld21 cluster follow-up. `latexml_oxide/src/post.rs` now injects
`LATEXML_VERSION` (= OUR Cargo `X.Y.Z`, `core_interface::LATEXML_VERSION`, #320) into the XSLT
params — mirrors Perl `LaTeXML.pm:562`, restores `LaTeXML-common.xsl`'s `LaTeXML_identifier`
generator stamp (`<!--Generated by LaTeXML (version X)…-->`) that oxide had been silently
omitting (empty param → `<xsl:if>` false), and gives BookML's `utils.xsl`
`b:version-leq($LATEXML_VERSION,…)` a non-empty value. Inserted before the user-override loop,
so `--xsltparameter LATEXML_VERSION=…` still wins (Perl's `LATEXML_VERSION:TEST` idiom).
Verified empirically: oxide's serialization keeps XSLT-emitted comments; no active test
full-compares an HTML golden, so **no re-bless was needed** (the `07/08/09_xslt_*`, `001`/`002`
tests are structural/`.contains`; `hello_new.html` + `daemon/formats/*.xml` are orphaned
Perl-copied artifacts). Guard: `tests/10_xslt_generator_version.rs` (default stamp carries the
const version, read dynamically → no version-bump churn; + explicit-param override wins).

### `--preload=<cls>` alone trips the hook stack; class-name divergence (2026-07-17) — OPEN

**Symptom.** `--preload=<any>.cls` prints `LaTeX hooks Error: Extra \PopDefaultHookLabel`
(article/book/report; `.sty` clean; `\documentclass` clean; `LATEXML_NODUMP=1` clean).
Perl is silent for the same preload.

**Mechanism (traced, 2026-07-17).** Push/pop are perfectly balanced and nested — the trace
is `PUSH article → (LaTeX.pool loads) → PUSH textcomp → POP textcomp → POP article → error`.
The bug is that **`\@pushfilename` changes MEANING mid-load**: `article` is pushed *before*
`LaTeX.pool` (and the kernel dump behind it) loads, so it uses a pre-pool `\@pushfilename`
that never touches `\g__hook_name_stack_seq`; the pool then installs the real expl3
`\@popfilename`, so `article`'s pop hits a seq holding only the *inner* packages' pushes,
finds it empty, and errors. `\documentclass` escapes because the pool is already loaded, so
both sites use the same meaning.

**A definedness check cannot see this** — the CS is defined at both sites; only its meaning
changes. Perl's `$pushpop` (Package.pm L2595, computed once and reused at L2637) is a
definedness check too, so Perl has the same hole; it is silent only because its dump omits
`\g__hook_name_stack_seq`, and `\seq_gpop:NNTF` on an *undefined* seq does not complain.
Ours dumps it as `\c_empty_seq`, so the real expl3 code correctly notices.

**Mitigated, not fixed (2026-07-17):** `util::preset::new_test_engine` now preloads
`LaTeX.pool` first (the order ar5iv's list already used), so `latexmlmath_oxide` stops
provoking it. `--preload=article.cls` on its own STILL errors.

**Dead ends — measured, do not retry:**
* Filtering the L3-hook stubs + filename stack from the dump (write+read): symptom gone,
  preloads clean — but `cluster_mhchem_cf_author_macro` 0 → **1003 errors** (suite 1581/0 →
  1572/9). The dump REPLACES base (DUMP_DESIGN rule 1), so filtering leaves a HOLE, not a
  fallback to `latex_base.rs`.
* Filtering ONLY `\g__hook_name_stack_seq` to match Perl's dump exactly: symptom gone,
  mhchem still fails — that record is load-bearing for our expl3 emulation.
* Threading Perl's `$pushpop` from push to pop instead of re-deciding (more Perl-faithful,
  worth doing anyway): does NOT fix it — the flag is `true` at both sites; the *meaning*
  moved underneath.
* Filtering `\PopDefaultHookLabel` alone: inert. The erroring caller is the internal
  `\__hook_curr_name_pop:`.

**Candidate fixes.** (a) Ensure a class/package preload cannot be the thing that drags in
the pool — auto-prepend `LaTeX.pool` when any `.sty`/`.cls` is preloaded. Rejected for the
release: Perl prepends only `TeX.pool` (LaTeXML.pm L710) and never auto-loads `LaTeX.pool`,
so this is a Rust-only divergence, and it would drag the LaTeX kernel into a `.sty` preload
on a plain-TeX document (the LaTeX-2.09 class `graphicx_sty.rs` already guards against). If
adopted, make it conditional on the pool being unloaded and LOG it. (b) Pair the pop to the
push's actual *meaning* rather than to definedness. (c) Make the pool load before any
handleoptions push. (b)/(c) address the cause.

**Second divergence, same area.** `\documentclass{article}` → `<?latexml class="article"?>`
but `--preload=article.cls` → `<?latexml class="article.cls"?>`; **Perl emits
`class="article"` for both.** Otherwise the two paths' output is byte-identical, so the
preload does load `article_cls.rs` correctly; `parse_preload_spec` splits correctly to
`("article","cls")`, so the extension is re-attached further in.

### `latexmlmath_oxide` empties a single-structure formula (2026-07-17) — OPEN

`latexmlmath_oxide '\frac{1}{2}'` and `'\sqrt{2}'` emit `<mrow/>` — an empty math
element. Perl `latexmlmath` renders both. Add anything around it (`\frac{a}{b}+c`) and
it works, so the trigger is a formula whose ENTIRE body is one top-level structure.

**Localized: NOT the engine or the math parser.** `latexml_oxide` converts the same
`\(\frac{1}{2}\)` correctly (mfrac present), while `latexmlmath_oxide` does not — so it
is that binary's preset path, `latexml::util::preset::lex_single_tex_formula` /
`new_test_engine`, probably in the `xmath.get_child_nodes() → unlink → into_xmath`
sequence in `bin/latexmlmath_oxide.rs`.

Pre-existing (reproduced on `66808398c4`), found 2026-07-17 while aligning the binary's
output with Perl. Not a regression from that work — which is verified byte-identical to
Perl modulo whitespace on formulas that do convert.


### TL2026 `latex.ltx` dump init is NOT release-gate-clean — expl3 catcode gap (2026-07-12) — ✅ CLOSED 2026-07-23

> **✅ RE-MEASURED 2026-07-23 — BOTH TL2026 blockers are clear; 2026 is IN the
> release dump window.** Measured the way the release actually runs it, rather
> than on a local install: a kpathsea-UNLINKED dumper built exactly as
> `release-dumps.yml`'s `build-dumper` job does (`KPATHSEA_NO_LINK=1
> KPATHSEA_SKIP_TOOLCHAIN_CHECK=1 cargo build --release`, `ldd` asserted
> kpathsea-free), run inside the real `ghcr.io/tkw1536/texlive-docker:2026`
> under the verbatim gate (`LATEXML_INIT_DEBUG=1`, ANSI-strip, `grep -acE
> '^(Error|Fatal):'`):
>
> | init | 2026-07-12 | 2026-07-23 |
> |---|---|---|
> | `--init=plain.tex` | exit 0, 0 errors | exit 0, **0 errors** |
> | `--init=latex.ltx` | exit 0, **137 errors** | exit 0, **0 errors** |
>
> The **likely** closers (not bisected — the 0/0 result is measured, the
> attribution is inferred) are the two expl3 fixes that landed **2026-07-20**,
> after the measurement below: force `\ExplSyntaxOff` when `_` is still LETTER
> (`latex_constructs.rs`) and the global `:`/`_`/`~` restore (`expl3_sty.rs`).
> They are the same pair credited with closing
> [`EXPL3_CATCODE_GAP_2026-06-08.md`](parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md),
> and all three error families listed below are now gone (the 90 ×
> `unexpected:_` was the bulk). Dump is sound, not degenerate: **24,221** latex entries vs
> 2025's 21,997 — the delta IS TL2026's expanded l3 `text-case` module — and
> the plain dump is byte-identical to 2025's.
>
> The **container** blocker (TODO(#217), independent) cleared too:
> tkw1536/historic-texlive-docker#1 merged 2026-06-08 and `:2026` publishes on
> the SAME `none-5.42.0`/debian:trixie base as `:2025` — libxml2 apt candidate
> 2.9.14 → `libxml2.so.2`, glibc 2.41 ≥ the ubuntu-22.04 build host's 2.35 — so
> the one-binary-serves-all-containers design holds unchanged.
>
> Landed: `release-dumps.yml` matrix + **all five** of `release.yml`'s
> `verify dump window completeness` gates (they are duplicated per build leg —
> `build-macos`, `build-macos-intel`, `build-linux-arm64`, `build-windows`,
> `release`; update them together or the legs disagree about the window) now
> span **2022–2026**. Validated end-to-end, not just at the gate: a binary embedding
> only TL2025 warns `latex_dump:mismatch loaded the TL2025 kernel dump, but the
> ambient TeX Live is 2026` on a TL2026 host; rebuilt with the 2026 dump the
> warning is gone and the conversion is clean. Cost: 48.1 → 49.0 MB (`release`
> profile), i.e. ~876 KB gzipped, against the 64 MB RELEASE_CRITERIA §2 cap.
> `release-dumps.yml` was also dispatched for real (run 30014067643): all five
> legs green.
>
> **Runner-disk flake on the 2026 leg — `generate` no longer uses a job-level
> `container:`.** The first CI run went 5/5 green; an identical re-run failed
> *only* the 2026 leg with `failed to register layer: … no space left on
> device` (3 pull retries, all out of space). 2026 is the fattest image —
> compressed 2022=4.6, 2023=5.0, 2024=5.4, 2025=5.7, **2026=6.2 GB** (9.8 GB
> extracted), monotonically worse each year — so it is the first to fall over.
>
> **Careful with the diagnosis** (an earlier draft of this note got it wrong):
> this is *not* a steady-state capacity ceiling. Instrumenting the job shows
> `/` at **145 GB total, 88 GB free before any cleanup** — roughly 5× the ~16 GB
> the pull peaks at. The pass and the fail landed on *different runner image
> versions* (`ubuntu-24.04` `20260714.240.1` passed, `20260720.247.2` failed),
> so the fleet is heterogeneous and some images come up with far less headroom.
> There is no way to pin a runner image *version* on GitHub-hosted runners, so
> the fix has to be tolerance, not capacity.
>
> Fix, in two layers of *tolerance* (not capacity): (1) drop `container:` (it
> pulls during "Initialize containers", *before* any step can run, so nothing
> can free space first) and `docker run` the image explicitly after a
> `free runner disk space` step that drops the preinstalled Android SDK / ghc /
> dotnet / CodeQL trees — measured 58 GB used → 30 GB, ~28 GB of margin ahead of
> the pull; and (2) a `pull TL image (retry-tolerant)` step that does an
> explicit `docker pull` with reclaim + linear backoff (10/20/30/40 s) across 4
> attempts, so a transient network / marginal-disk first failure clears on
> retry (`docker run`'s implicit pull is a single, non-retried attempt).
> Verified by extracting each step's script straight out of the YAML and running
> it verbatim — the gate against the real `:2026` image (byte-identical
> artifacts), the retry loop's control flow via a failing-`docker` stub (4 tries
> then exit 1) — plus a 5/5 green CI run. **Do not "simplify" back to
> `container:`.** This matters more than it used to, because every `release.yml`
> build leg now hard-requires the full window, so one flaky leg blocks a whole
> release.
>
> **Observed in passing — dumps are not bit-reproducible (pre-existing, ALL
> years, not introduced here).** The CI-produced `latex.2026.dump.txt` and the
> local one are identical in size and entry count but differ in exactly ONE
> byte-range: `V\ttexsys.aux_contents` embeds a wall-clock stamp
> (`2026/07/23:13:54` local vs `:14:13` in CI). Harmless today (cosmetic
> `texsys.cfg` capture), but it means two dumps of the same TL year + same
> revision never compare equal, so any future "did the dump change?" check must
> normalize that field rather than `cmp`.
>
> History below kept for the root-cause trail.

Blocked adding **2026** to the release dump window (`release-dumps.yml`,
then 2022–2025; see also the container blocker TODO(#217) — the two are
independent). Measured on a full local TL2026 install (`x86_64-pc-windows-msvc`,
but Linux-equivalent — this is the raw-load path, not a platform issue), using
the exact release gate (`LATEXML_INIT_DEBUG=1 ./latexml_oxide --init=<init>`,
ANSI-strip, `grep -acE '^(Error|Fatal):'`):

- `--init=plain.tex` → exit 0, **0 errors** (release-clean). ✓
- `--init=latex.ltx` → exit 0, **137 errors** → would FAIL the gate:
  - 90 × `Error:unexpected:_ Script _ can only appear in math mode`
  - 29 × `Error:misdefined:# … catcode PARAM … should never reach Stomach`
  - ~18 × undefined l3 **case-change** internals (`\DeclareUppercaseExclusions`,
    `\DeclareCaseChangeEquivalent`, `\CaseSwitch`, `\@@text@case@aux`,
    `\NoCaseChange`, `\AddToNoCaseChangeList`, `\keys`/`\tl`/`\str`/`\clist`…).

Root cause: the **known deep raw-load expl3 catcode gap**
(`EXPL3_CATCODE_GAP_2026-06-08.md`; "four attempted fixes all regressed and
were reverted"), newly triggered by TL2026's expanded l3 `text-case` module,
which older TL (2022–2025, all gate-clean) did not exercise during init.
Distinct from the `\Declare*caseMapping` no-ops landed above (those are
different macros; that fix does not touch this). **NOT introduced by the
Windows branch** — pre-existing, surfaced only because that branch is the
first to run a full TL2026.

Note the two-bars distinction that hid this: `tools/make_formats*` write the
dump *despite* init errors (24,333 valid latex entries still land), and the
test fixtures don't exercise the affected macros — so `cargo test` is 1531/0
and everyday TL2026 conversion works, while the strict release gate would
still reject a TL2026 latex dump. "Usable dump" ≠ "gate-clean dump."

### TL2026 ambient-drift fixes (2026-07-12, windows-compatibility branch) — LANDED

Two suite failures surfaced by running against TL 2026 (bleeding-edge; CI's
ubuntu TL is older) — both root-caused and fixed TL-independently:

1. **`\Declare{Upper,Lower,Title}caseMapping` native no-op handlers**
   (`latex_constructs.rs`, next to the `DeclareText*` family). The TL2023+
   kernel case-mapping declarations ARE captured in the latex dump, so
   `\ifdefined` guards (greek-fontenc `lgrenc.def`) passed and the dumped
   expl3 kernel bodies executed — hitting the raw-load expl3 catcode gap
   (`EXPL3_CATCODE_GAP_2026-06-08.md`) and spraying `Script _` + undefined
   `\acc*` errors (81_babel `greek_test`: 87 errors → 0; real pdflatex is
   error-clean on the same fixture). LaTeXML cases via Unicode internally,
   so ignoring these matches the `ignoredDefinition` policy. **Perl has no
   handler either — same cascade expected there on TL2023+; candidate
   upstream.**
2. **tikz `ac_drive_components`: SKIPPED ON WINDOWS ONLY, kept live on
   Linux/macOS.** circuitikz 1.8.0 rewrote its path logic, lengthening drawn
   capacitor plates (12.4 → 12.68 in our SVG space). This coordinate tracks
   the exact circuitikz version, which is NOT pinnable (both Perl
   `FindFile_fallback` `[vV]?[-_.\d]+` and Rust version-strip a
   `circuitikz-X.Y.Z` request to the current binding) and **differs by the
   platform's TeX distribution**: Linux (apt texlive) and macOS (Homebrew
   texlive) ship an OLDER circuitikz → 12.4 = the committed golden; the
   Windows CI's `setup-texlive` net-install and any fresh `install-tl` get
   the NEWEST → 12.68. So the fixture is **compared on Linux/macOS (where the
   golden is deterministic) and skipped on Windows** via a `#[cfg(windows)]`
   `WINDOWS_GOLDEN_SKIP` guard in `latexml_test_single` — a Linux↔Windows
   portability difference, NOT a code divergence (the engine faithfully
   renders whatever circuitikz emits — it was testing circuitikz's version,
   not our code). Not TL-year-keyable (macOS and Windows are both TL2026 with
   different circuitikz — a `tl2026.xml` variant attempt regressed macOS CI
   and was reverted). `INTENTIONALLY_FAILING`/`ERROR_DEBT` don't apply (they
   gate error *counts*, not golden *diffs*). Discovered via Windows CI run
   `29219528633` (which fail-fast-stopped at 86_tikz; the complete tail came
   from a local `--no-fail-fast` run on a newest-circuitikz box — a faithful
   Windows-CI proxy — confirming 1530/0 with this one fixture out).

### Release-week stabilization (2026-07-10, user-directed) — THE LENS FOR THIS WEEK

**Public release is ~1 week out (branch `public-release-prep-week`). The bias is
STABILIZE, not add capability.** A regression introduced in release week is far
costlier than a feature deferred. So the actionable list below is re-ordered by
*risk*, not by *ambition*: the safe, landed-or-verification work leads; every
hot-path / broad-diff / deep-engine item is explicitly demoted to POST-RELEASE.

**SAFE — do in release week (low risk, high stabilization value):**

1. **Verify the already-landed >500 MB `index.xml` path on the release binary**
   (see the investigation note directly below). The foundation is **already in the
   release** — PR **#274** (`b0cc70f319`, squash-merged 2026-07-07): limit-safe
   DOM-walk queries so split fires + loud XPath errors, stream-the-file/skip engine
   init, CrossRef O(n²)→O(n) (42m50s→2m18s). So there is **nothing to land** (an
   earlier "not in the release branch" read was an ancestry-check error — #274 was
   squash-merged, so the branch SHAs aren't ancestors even though the content is).
   It fixes a **silent-failure class** (any doc large enough to cross libxml2's 10M-
   nodeset ceiling → NULL nodeset → swallowed → `[not split]`) and converts a
   document **Perl LaTeXML cannot** (Perl `latexmlpost` fatals at the nodeset
   ceiling in 8.67s). The release-week action is a **confidence check**: run the
   614 MB witness on the `maxperf`/release binary and confirm `Split into 40201
   pages` + byte-identical HTML (design-doc baseline 2m18s, ~21.6 GB peak; a 32 GB
   box handles it — watch RAM contention). *Excludes* the deferred two-pass
   streaming split (task #44 / `STREAMING_POST_DESIGN`) — that risky memory-only
   half is NOT needed for release.
2. **Full regression + smoke gate on the release binary** — the release
   discipline, pure risk reduction. `cargo test --tests --no-fail-fast` (expect
   ~1534/0), `cargo clippy --workspace --all-targets -- -D warnings`, then a
   `tools/benchmark_canvas.sh` smoke of a few hundred mixed papers on the
   `maxperf` binary, checking fatal classes against the known list + spot-checking
   HTML with the shipped CSS. (Mirrors the July-5 prep item 6.)
3. **Confirm the graceful-abort safety floors still fire** — these, NOT the deep
   loop fixes, are the release's real stability guarantee: the 4500 MB RSS fuse
   (Cluster A/D/E), IfLimit 16M / TokenLimit 1B (Cluster H), the 12k expand-depth
   guard + stack guard (Cluster F). All landed; this is verification only (a
   pathological paper must Fatal cleanly, never hang/segfault/OOM the process).

**DEFER to POST-RELEASE — do NOT start in release week (risk > reward now):**

- **All BP-1…BP-6 beyond-Perl perf levers** (below) — hot-path, output-neutrality
  gated, ambitious (rayon math parse, XSLT transpile, document-builder rewrite).
  A regression here is a release-killer; the 60k telemetry that motivates them
  keeps. **First post-release work, not release-week work.**
- **Cluster H deep runaway-loop fixes** (`STABILITY_WITNESSES.md`: `\kbordermatrix`
  box-peel, `\IfFileExists`-before-`\documentclass` readBalanced-past-EOF,
  undefined-cascade IfLimit). Genuine Rust bugs, but the fixes are deep
  gullet/box-register surgery with broad blast radius — AND current behavior is
  already SAFE (graceful Fatal via an existing limit ~100s in, bounded, no
  crash/corruption), so they are fidelity/perf gaps, NOT release-blocking
  stability risks. ~~The one clean regression (`2605.23849`, Perl completes) is a
  real fidelity loss whose fix is still deep.~~ **FIXED 2026-07-20** — and the
  premise was doubly wrong: Perl does not "complete" it (it skips the matrix),
  and the fix was one `Let!` retracting the inherited kernel `\@arraycr`, not deep
  surgery. All of Cluster H is now resolved.
- **`ltx_env_<name>` class enhancement** (below) — churns nearly every golden
  XML; running it in release week would swamp the regression baseline and mask
  real regressions. Isolated branch, post-release (as already noted).
- **MakeBibliography full re-port** (below) — already marked post-release.
- **`validate()` / `--validate`** (above) — already postponed to the next release
  (gated on the `rust-libxml` RelaxNG publish).
- **Verbatim-in-box items 4–6, biblatex `.bbl` `2605.17646`** (below) — low-value
  fidelity / graceful-fatal; not blockers.

*(Deliberately conservative: no contained "quick-win" bug fix in the current list
clears the risk/reward bar for release week — the parity long-tail is graceful
already. If a NEW same-host-confirmed GENUINE-RUST-ONLY regression surfaces from
the smoke sweep, that jumps the queue; nothing currently open does.)*

### Frontmatter-fidelity pass over the arXiv `html_feedback` reports — LANDED 2026-07-12

Drove the ~280 arXiv "front matter" `html_feedback` reports to clean, structured
frontmatter (branch `public-release-prep-week`). Method: convert each reported
paper to standalone HTML on the ar5iv config, then **Playwright red/green** DOM
checks (`.ltx_personname`/`.ltx_authors`/`.ltx_bibitem` counts + raw-macro-leak
regex). Two commits landed the class bindings: `12ccebefc1`/`537aac9e50` (20
classes), `3bc8a3342d` (JMLR structured author blocks + Wiley `MRM.cls`). See
[[frontmatter-class-bindings-2026-07-12]] memory for the binding patterns.

- **JMLR** (`jmlr_cls.rs`): `\Name`/`\Email`/`\addr` now digest **directly** into
  structured creators (name → personname, email/affiliation → contacts) instead
  of the generic `\and`/comma splitter, which crammed every author into one
  `<personname>` and split the affiliation's commas into phantom authors; `\nametag`
  no longer leaks. (Answers a user question on maximizing structured markup —
  beyond-Perl, Perl ships no jmlr binding.)
- **MRM.cls** (Wiley "Magnetic Resonance in Medicine", new `mrm_cls.rs`):
  `\author[idx]{name}{orcid}`, `\address`, `\corres`, `\finfo`, `\authormark`,
  `\state` (deliberately absent from OmniBus), plus own dep loads for ORCID/math/cites.

**Harness note (signal integrity):** the arXiv-source main-`.tex` detector must
skip `*-backup.tex` / `template/*` / `Rebuttal.tex` / `*_preprint.tex` /
versioned-subdir mains and *bonus* the file that carries the bibliography — an
early detector picked wrong mains and produced ~5 false "no authors / no
bibliography" reds (e.g. `2511.04594` = `Rebuttal.tex`). Corrected detector +
re-convert cleared them.

**Residual reds — all PARITY or already-beyond-Perl (NOT release-blocking):**
`2402.09505` (aa `\href`-in-name, parity/cosmetic), `2601.05137` (author `\def\name`
in a redefined `\@maketitle`, KPE #47 parity), `2403.07832` (minor `\footnotesize`
in a `\thanks`, no minimal repro); `2306.06628`/`2512.16391`/`2605.23904` (no
`\author` in source); `2508.20929` (atlasdoc author list `\input` in the body);
`2405.13705` (iidtp `\makeiidtp` `titlepage` suppresses the document title block —
**shared Perl XSLT rule**, and Perl *times out* entirely — authors show via the
titlepage ORCID links); `2505.13921` (neurips: Perl *times out*; Rust produces the
full doc with authors preserved in `<ltx:creator>` metadata, but the visible title
block doesn't render — a `\maketitle`-expandability interaction). The last two are
**beyond-Perl already** (Perl produces nothing).

### Bibliography "missing references" — NEXT-TARGET list (surveyed 2026-07-12)

Per the user follow-up ("detect docs where References are entirely missing … the
next target for beyond-Perl bibliography work"). Playwright scan over all 297
reported papers (correct mains) → only **4** genuinely lack a rendered
bibliography, and the dominant root cause is **NOT** bibliography markup — it is a
**mid-body digestion error that truncates the document** before the (end-of-doc)
bibliography, which is then collateral damage:

- `2507.21938` (ICML): document tree **truncates** mid-section-2 (empty table
  cells + empty figure); `\bibliography{example_paper}` + co-located
  `example_paper.bbl` never reached. Body-truncation bug.
- `2508.13557` (IEEEtran): undefined `\node` (tikz outside a picture) corrupts
  `display_math` mode → `\lx@end@display@math` cascade → **truncation** before the
  bibliography. `main.bbl` *is* input (the `\jobname.bbl` fallback works). Body-error bug.
- `2510.25135`: **source path-doubling** — main is `submissio-v0/main.tex` and
  `\bibliography{submissio-v0/mypub,submissio-v0/ref}` resolves relative to that
  dir → `submissio-v0/submissio-v0/…`. Source quirk (assumes top-level compile).
- `2606.25280`: **source filename case/extension quirk** —
  `\bibliography{EvoFlock.bib}` vs the shipped `Evoflock.bib` (fails only on a
  case-sensitive FS; parity with Perl on Linux).

So the real beyond-Perl lever is **body-error resilience** (2 papers where a
mid-body digestion error truncates the tail); the other 2 are source quirks/parity.
Post-release (release-week bias is stabilize, and these are deep digestion work).

#### Corpus-scale confirmation (swept 2026-07-14, sandbox-2605, 30,058 result ZIPs)

The 297-paper scan above is confirmed at corpus scale, and the split is now
measured. Detection is a **rendering** property, not a cortex category (there is
no "empty References" category): read the produced HTML out of each result ZIP
and count `ltx_bibitem`. Baseline (pre-fix run): **ok 29,308 (97.5%) / EMPTY 324
(1.1%) / no-bib 359 / no-html 67**.

The EMPTY bucket is NOT one defect — decomposing it is the whole point, and a
first pass that lumps them together mis-ranks the work:

| class | n | what it is |
|---|---|---|
| **TRUNCATED** | 169 | citations rendered but **no bibliography element at all** — the document died before reaching `\bibliography`. NOT a bibliography bug. |
| **NO-CITES** | 92 | a literal "References" heading but no citations/bibliography markup — mostly author-hand-rolled lists; largely parity. |
| **EMPTY-SECTION** | 63 | bibliography element present, **zero entries** — the genuine bibliography defect. |

So **truncation, not bibliography code, is the dominant cause of a missing
References section** (169 vs 63) — the 2026-07-12 hypothesis, now quantified.
Body-error resilience remains the top lever.

**TRUNCATED (169) is REAL, not a stale-ZIP artifact** — spot-checked 4 witnesses
of the largest sub-cluster on the current binary: 3 still truncate
(`2605.00025` 455 errors, `2605.09913` 91, `2605.12696` 14; only `2605.09761`
recovered). Contrast the EMPTY-SECTION side, where stale ZIPs DO dominate
(`2605.02024` shows 38 bibitems / 0 dangling on the current binary) — so
**re-convert before chasing any EMPTY-SECTION paper**.

Dominant TRUNCATED trigger (first error, not the cascade):

| trigger | n | note |
|---|---|---|
| `unexpected:\lx@end@inline@math` | 25 | math-mode desync |
| `unexpected:\lx@begin@alignment` | 19 | alignment opened inside inline math |
| *no errors at all* | 17 | **silent** content loss — worst kind |
| `unexpected:_` / `^` | ~37 | sub/superscript outside math (same family) |
| `unexpected:\lx@tag@intags` | 4 | the `\fnum@figure` cascade above |

The math-desync + alignment families together are ~66/169 (39%) and look like one
root family: a group/mode nesting break around inline math. **11 of the 169 are
the known mhchem `\ce`-in-`align` parity limit** (`2605.12696`: `\ce{CO2(aq) +
H2O &<=> H2CO3}` inside `align` — identical in same-host Perl, investigated
2026-06-27, NOT a Rust gap). The remaining ~147 are the concrete next target for
body-error resilience.

**One TRUNCATED sub-cluster is now FIXED — inline `\end{lstlisting}`** (7 of the
169, 3 of them in the silent subset). See OXIDIZED_DESIGN #61 /
KNOWN_PERL_ERRORS #51: Perl anchors the terminator regex at the line start, so
`</body></html> \end{lstlisting}` never terminates and the reader eats the rest
of the file, `\end{document}` included — **zero `Error:`**. pdflatex accepts the
same input and renders the leading text as the listing's last line, so both
LaTeXML engines were wrong vs the PDF (same-host Perl: "No obvious problems", tail
gone). Fix = match `\end{<env>}` anywhere in the line. 5 of the 7 witnesses
recover: `2605.11619` 0 → 32 refs (Conclusion + appendix restored), `2605.29675`
107, `2605.21677` 66, `2605.29786` 42, `2605.07451` 28 — **275 references**. The
other 2 (`2605.08378`, `2605.08915`) have unrelated causes.

**This is why the "17 silent" subset is the highest-value slice**: no `Error:`
means no cortex signal, so these never surface in any severity report — the only
way to find them is a rendering sweep like this one.

Within EMPTY-SECTION the one clean, landed win was the **non-UTF-8 `.bib`**
cluster (below). Remaining EMPTY-SECTION sub-clusters, not yet triaged:
`undefined:\affiliations/\emails` (7), `post:convert` (8), a revtex4-2 +
bibunits `bu1.bbl`/`bu2.bbl` group (7).

**Trap (hit and corrected):** the classifier's citation needle must be real
citation markup (`ltx_cite`/`ltx_bibref`/`ltx_missing_citation`) — `ltx_ref`
also matches `\ref` to a figure, which over-counted TRUNCATED 231 → 169.
Second trap: **pick the main file cortex picked** (`Processing content …` in
the log). A `grep -rl '\begin{document}'` harness picks the first match, which
for `2605.30360` was the decoy `proof.tex`, not `polyhist.tex` — that alone
manufactured a false "still broken" verdict.

#### `\renewcommand*{\fnum@figure}[1]` truncates the document — ANALYSED 2026-07-14, NOT fixed (needs a decision)

Witness `2605.01731` (cas-sc): 18 figures × 3 errors
(`\lx@tag@intags`/`\lx@tag`/`\end{figure}` "Attempt to end mode
restricted_horizontal") → body collapses to ONE section, 19 `<bibref>` survive
but **no `<bibliography>` element at all**. Breadth: **18 papers corpus-wide**
(`grep 'lx@tag@intags'`), 5 of them in the EMPTY set.

Root cause is a real-world author hack that pdflatex tolerates:

```tex
% Change Fig. 1: to Fig. 1.
\makeatletter
\renewcommand*{\fnum@figure}[1]{\figurename~\thefigure.}
\makeatother
```

Real `\fnum@figure` takes **no** argument. In LaTeX, `\@caption` passes it to
`\@makecaption{\csname fnum@\@captype\endcsname}{…}`, whose body is
`\sbox\@tempboxa{#1: #2}` — so the author's 1-arg version **eats the `:`**,
which is exactly their stated intent. It works in pdflatex.

LaTeXML has no `:` token to eat: `\format@title@figure` is
`\lx@tag[][: ]{\lx@fnum@@{figure}}#1` — the separator is a **tag attribute**, not
a token. So `\csname fnum@figure\endcsname` (Base_Utility L1041-1043) grabs the
group's closing `}` instead, wrecking the caption and cascading.

This is **PARITY** — Perl's `\lx@fnum@@` is identical — so fixing it is a
surpass-Perl divergence, and both engines are wrong vs the PDF. Candidate fix:
expand as `\csname fnum@#1\endcsname{}` so an arg-taking `\fnum@<type>` eats a
harmless empty group (reproducing pdflatex's result) while a normal 0-arg one
just gains an empty group. **Not done**: `\lx@fnum@@` formats every figure/table
caption in every document — blast radius far out of proportion to 18 papers, and
release-week bias is stabilize. Needs a user decision + a full-suite diff.

Minimal repro (article + subfigure + the `\renewcommand*` above) reproduces the
exact 3-error signature; `cas-sc` is NOT implicated (it was the first
hypothesis and it was wrong — plain `article` reproduces).

#### Non-UTF-8 `.bib` silently dropped the whole bibliography — LANDED 2026-07-14

`std::fs::read_to_string` hard-errors on the first non-UTF-8 byte, so a legacy
`.bib` lost **every** entry and rendered an empty References section with **no
`Error:`** — a silent, total loss. Witness `2605.00490`: a JabRef file
self-declaring `% Encoding: Cp1252`. Real `bibtex` 0.99d is 8-bit clean, and
Perl never fails here (`Mouth.pm` L75-80: decode with `Encode::FB_DEFAULT`, or
pass the raw bytes through when `PERL_INPUT_ENCODING` is undef) — so this was
**GENUINE-RUST-ONLY**, not parity.

Fix: both `.bib` read sites (`pre_bibtex::new_from_file` engine-side,
`make_bibliography::convert_bib_file_to_xml` post-side) now decode via the new
shared `latexml_core::mouth::decode_input_bytes` — UTF-8, else a **Latin-1
passthrough** (lossless byte → char, so accented names survive intact instead of
collapsing to U+FFFD; legacy `.bib` files are overwhelmingly Latin-1/Cp1252).
The Mouth's own no-encoding branch now calls the same helper, so there is one
implementation rather than three (the "bespoke duplicate shadowing a faithful
port" anti-pattern has already bitten twice here).

Breadth: 17 papers corpus-wide, 10 of them EMPTY. All 10 recovered: **0 → 336
references** (7/15/5/22/57/25/39/48/108/10), 0 dangling.

Red/green tests: `pre_bibtex::tests::non_utf8_bib_file_is_read_not_rejected`
(engine reader) **and** `06_cluster_regressions::non_utf8_bib_file_still_yields_a_bibliography`
(post path — where the production failure actually was; the engine-side test
alone would NOT have guarded it). Fixture `cluster_regressions/cp1252_refs.bib`
carries a raw `0xe9`; it is asserted non-UTF-8 so the test cannot go vacuous.
Note when asserting on rendered author names: `author = {Café, André}` is
BibTeX's `Last, First`, so the style abbreviates the given name — the entry
renders `A. Café`, and only the SURNAME is a safe needle.

Third test: `one_bad_byte_does_not_mojibake_the_rest_of_the_file` pins the
per-line granularity (a whole-buffer fallback turns a valid-UTF-8 `Ü` into
`Ã\u{9c}` — verified by reverting).

### >500 MB `index.xml` (Nasser) — INVESTIGATED 2026-07-10

Witness `~/scratch/nasser/index.xml`: 614 MB, ~7M nodes, **40 000 one-equation
sections** (`solving_ODE` auto-generated notes), `--splitat=section`. Findings:

- **Perl LaTeXML cannot convert it.** The reporter's own `index.latexmlpost.log`:
  `latexmlpost` (0.8.8) dies `Fatal:perl:die … growing nodeset hit limit`
  (`XPath.pm:36`) in **8.67s** — libxml2's `XPATH_MAX_NODESET_LENGTH`. Perl's
  *core* also took **52m 7s** just to emit the XML (40000 formulae / 1577s math).
- **latexml-oxide CAN, and the fix is ALREADY in the release** (PR **#274**,
  `b0cc70f319`, squash-merged 2026-07-07 → ancestor of `public-release-prep-week`).
  With the foundation it converts fully: `Split into 40201 pages`, ~2m18s, peak
  ~21.6 GB, byte-identical across all pages (measured;
  `STREAMING_POST_DESIGN_2026-07-06`). A genuine **beyond-Perl** win (Perl outright
  fatals). Without the fix, `//*[@xml:id]` would overflow the 10M-nodeset ceiling →
  NULL → swallowed → `[not split]`, silently reproducing Perl's failure class — but
  that landed in #274, so the release-week action is only the confidence check in
  SAFE step #1, not a merge.
- **The lean-RSS half stays deferred (task #44).** Two-pass streaming split
  (21.6 GB → <1 GB) is unneeded for release (reporter has >64 GB RAM; eager path
  is correct + fast). Revisit only if a <64 GB target appears. Design preserved in
  `STREAMING_POST_DESIGN_2026-07-06.md`.

### Beyond-Perl performance levers — from the 2026-07-10 60k-doc telemetry (POST-RELEASE — deferred out of release week per the stabilization review above)

The 2605+2606 reruns (60,469 docs, containerized worker, per-job `telemetry.json`
mined in `docs/performance/ARXIV_PERFORMANCE.md` "Corpus-wide phase budget 2026-07-10")
re-point the perf campaign. **Wall time is broad, not math-dominated:** digest
19.7% · math_parse 19.2% · build 18.1% · **xslt 13.2%** · graphics 8.9% ·
mathml_pres 4.5%. Concentration is only moderate (slowest 1% = 10% of wall), so
median-path wins pay off as broadly as tail-chasing. These are **Target-2
beyond-Perl** tasks: Perl LaTeXML is single-threaded (thread-local State
singleton) and libxslt/`XML::LibXML`-bound; Rust affords levers it cannot.

**Architectural constraints that shape feasibility (respect these):**
- State is a thread-local global singleton → the **digest phase is sequential**;
  no parallelism lever there, only algorithmic.
- rust-libxml nodes are **not `Send`/`Sync`** (libxml2 FFI) → cannot naively
  parallelize DOM mutation. The tractable pattern is **parallelize the pure,
  `Send`-able computation (Marpa parse, MathML *structure*), keep the DOM graft
  sequential.**
- one-conversion-per-process harness (memory isolation) → amortize *within* a
  conversion (fork/threads), not across docs.
- **Output-neutrality gate is non-negotiable** (`ARXIV_PERFORMANCE.md`): every
  lever must be byte-identical on the isolated before/after harness + keep Perl
  parity. A perf change that alters output is a separate, authorized decision.

**BP-1 — Parallel per-formula math parsing** (attacks math_parse 19.2%; the
math-dense slow tail — `2605.16382` 4136 formulae/116s, `2605.20736`, `2605.14423`).
Each `<XMath>` Marpa parse is independent and operates on a token/box IR (data,
not libxml). *Lever Perl lacks:* Parse::RecDescent + single thread. *Approach:*
collect formula IRs during digest; parse them in a rayon pool (each thread gets
its own thread-local SymStr arena — verify the parser is arena-isolatable and
free of cross-formula shared mutable state); graft XMDual/parse results into the
DOM sequentially in original order. *Feasibility:* medium (arena-per-thread +
parser-purity audit). Output-neutral by construction (same parses, same order).

**BP-2 — XSLT amortization → native transpilation** (attacks xslt 13.2%, the
single most under-exploited phase — only the 3 `O(n²)` template fixes touched it).
13% is libxslt *interpreting our own fixed, embedded stylesheets*, re-parsed per
one-doc process. *Step 1 (cheap, do first):* `xsltproc --profile` split of xslt
into stylesheet-COMPILE (fixed/doc) vs APPLY (scales with doc); if compile-heavy,
embed a **pre-parsed/precompiled stylesheet** (the XSLT analog of the kernel-dump
precompilation we already ship). *Step 2 (ambitious, beyond-Perl):* transpile the
hottest templates the profile flags into **native Rust DOM transforms**, bypassing
libxslt entirely for them (Perl is libxslt-bound and cannot). *Feasibility:* Step1
low-risk/moderate win; Step2 high-effort/high-win.

**BP-3 — Concurrent graphics + parallel MathML structure** (graphics 8.9% +
mathml_pres 4.5% ≈ 13%). Graphics conversions are independent *subprocesses*
(gs/dvisvgm/inkscape) run **serially** today — fork them in a bounded concurrent
pool (no `Send` barrier; the tractable, high-feasibility half). MathML
presentation per formula is independent pure computation → parallelize on BP-1's
enabling work. Perl runs both serially.

**BP-4 — Live digest-progress watchdog — RETIRED 2026-07-10 (triage overturned the
premise).** The Cluster H "digest-runaway fatals" were triaged against same-host
Perl (`STABILITY_WITNESSES.md` Cluster H): they are **not** a clean beyond-Perl
watchdog opportunity but a heterogeneous set of **genuine Rust runaway-loop bugs**,
and a no-progress abort would have **aborted `2605.23849`** (note the old premise
"which Perl converts cleanly" is wrong — Perl skips the construct)
(46s, 0 fatal). Reclassified as Target-1 parity work — **all three FIXED
2026-07-20**, and the "three distinct root causes" reading was itself wrong: (a)
and (c) turned out to be ONE bug (a stale `def_autoload` trigger), and (b)'s
recorded root was a red herring. Superseded diagnoses, kept so they are not
retried: ~~(a) `\IfFileExists`-before-`\documentclass` → expansion spins past EOF
→ TokenLimit (2606.21610)~~ — nothing reads past EOF; the `\IfFileExists` group
makes `\documentclass` load inside a group, stranding the autoload trigger.
~~(b) `\kbordermatrix` `\lastbox`/`\ifhbox` box-peel loop → IfLimit (2605.23849;
the clean must-fix regression)~~ — that box-peel loop **also loops in Perl**
(SHARED); the real root was the inherited kernel `\@arraycr`. ~~(c)
undefined-macro cascade → IfLimit (2605.21013)~~ — same bug as (a), it merely
tripped a different limit. Note the still-OPEN *read_balanced unbalanced-group
leak* family was never this witness's problem.
Each trips an *existing* high limit ~100s in (safety net present but late) and needs
a faithful per-mechanism fix, NOT a blunt early-abort. The unifying theme in (a)+(c):
Rust error-recovery *loops* where Perl keeps *advancing* (emitting bounded errors →
`too_many_errors` cap, which Rust also has but never reaches because the loop emits
none). Do not build the watchdog.

**BP-5 — Content-addressed formula memoization** (math_parse 19% + mathml 4.5% on
matrix/table/aligned-system-heavy papers, which repeat identical sub-formulae).
Hash the normalized formula token-stream (FxHashMap + interner — cheap in Rust)
and memoize parse→XMDual→MathML. *Lever Perl lacks.* **Correctness crux:** the key
must capture every parse-affecting context (font, mode, catcodes, math-style);
mis-keying silently corrupts output, so gate hard on the output-neutrality diff.
*Feasibility:* medium; large win on table/matrix-dense papers.

**BP-6 (stretch/experiment) — Native construction tree, defer libxml FFI**
(attacks build 18.1% = per-node rust-libxml FFI during construction). Build a
native arena tree during construction, convert to libxml once at the end (or emit
HTML directly on the non-`--validate` path). Perl is also `XML::LibXML`-FFI-bound,
so this is a structural beyond-Perl bet. *Feasibility:* low-medium, HIGH effort
(rewrites the document builder core) — park as an experiment, measure the FFI
share first.

**Digest (19.7%) note:** sequential TeX engine — **no** parallelism lever; the win
is algorithmic (profile the hot macros with the sampled `EXP_TRACE` histogram, cut
redundant re-tokenization / re-expansion). Track separately from the parallelism
BPs above.

Suggested order (revised 2026-07-10 after BP-4 was retired) — **all POST-RELEASE per
the release-week stabilization review above; first work after the tag ships:**
**BP-2 Step 1** (cheap XSLT profile+amortize — the cleanest, divergence-free win) →
**BP-3 graphics batch** → **BP-1** (parallel parse) → BP-5 → BP-2 Step 2 / BP-6. Each
lands on a feature branch, gated by the isolated before/after output-neutrality
harness + Perl parity + `cargo test`. ~~Separately, the Cluster H runaway-loop bugs
(ex-BP-4) are Target-1 parity work tracked in `STABILITY_WITNESSES.md` (also
post-release — deep engine surgery, not release-week work).~~ **Cluster H is
fully resolved as of 2026-07-20** — and none of it needed deep engine surgery.

### MakeBibliography full parity re-port (user directive 2026-07-04: reuse TeX interpretation, no special-case parser)

Audit 2026-07-04 (agent, both files read end-to-end): `make_bibliography.rs`
(3,545 lines) vs Perl `MakeBibliography.pm` (818 lines) is a **faithful port
with one large divergent subsystem**: ~11 of 18 Perl subs are structural
ports (FMT_SPEC stays table-driven; getBibEntries referrer/suffix logic,
formatBibEntry, all do_* formatters track Perl), BUT the .bib->XML route
replaces Perl's 63-line recursive-core-session `convertBibliography` with
~770 lines (~22% of the file) of hand-rolled string parsing
(`parse_bibtex`, `read_bib_value`, `parse_bib_authors`, `strip_braces`,
`is_braced_group`, `convert_bib_file_to_xml`, plus the whole
metadata-fallback path that exists only because no real bibentry XML is
produced).

INTERIM (landed 2026-07-04): field VALUES now go through the real engine —
`interpret_tex_text` = `digest(mouth::tokenize(v)).to_string()` against the
LIVE in-process state (Perl's `ToString(Digest(Tokenize($x)))`; article-
class macros like `\aap` expand because aa.cls is loaded); the ~150-line
`decode_tex_accents` transliterator is DELETED. DOI identifiers emit
absolute `https://doi.org/` hrefs (percent-encoded, Perl BibTeX.pool
L750-756) and scheme-less bib URLs are forced absolute — normalized both at
.bib conversion AND in `format_links` (covers .bbl-borne/pre-compiled XML).

FULL RE-PORT remaining (post-release):
1. Replace `convert_bib_file_to_xml` with the recursive core conversion
   (`DigestionMode::BibTeX` + `PreBibTeX` + bibtex.rs already exist):
   inject from latexml_oxide's post-orchestration (latexml_post cannot
   depend on the converter); recover class+packages(+options) preloads from
   the document PIs; isolate/accumulate REPORT counters + log around the
   recursive session; single combined pass for multiple raw bibs
   (cross-bib @string sharing); prefer `<name>.bib.xml`; kpsewhich +
   literaldata inputs. Deletes the string parser + metadata fallback
   (~770 lines).
2. Secondary parity gaps from the audit: `unisort` (Unicode collation) vs
   `Vec::sort()`; citestyle semantics swapped (`AY` should be the
   abbreviated `[AA+yy]` label, not full author-year); `Formatter::Year`
   drops the disambiguation `@SUFFIX`; document-global NUMBER across split
   documents.
3. **Field-interpretation whitelist (first stage, not yet Perl-faithful)** —
   flagged by the 2026-07-05 commit review of `ede2bdcc2c`. The `.bib`→XML
   path (`make_bibliography.rs`) only digests 13 fields
   (author/editor/title/year/journal/journaltitle/booktitle/volume/number/
   issue/pages/publisher/note). Perl's `BibTeX.pool.ltxml` has ~28
   `\bib@field@default@*` constructors that DO digest — incl. `abstract`
   (L708), `keywords` (L732), `annote` (L680), `series`, `institution`,
   `organization`, `school`, `edition`, `chapter`, `howpublished`,
   `translator`, `subtitle`, `type` — so Perl raises (and MergeStatus'es) the
   undefined-macro errors those fields carry, while Rust currently does NOT.
   The commit's original "mirrors Perl" comment was factually inverted
   (corrected in-code 2026-07-05). Decision (user, 2026-07-05): keep the
   narrow set FOR NOW as a first stage — it suppresses the junk-field error
   floods of ADS/Zotero exports — but the eventual target is Perl's full
   rendering-field set. Bounded blast radius: this path only fires for raw
   `.bib` inputs WITHOUT a `.bbl`. Widen when the full re-port (item 1) lands
   the recursive core session, which digests fields the Perl way by
   construction.

Witness: 2605.00223 (ADS .bib: `{\'\i}`, `~` ties, `\aap`, bare DOIs).

### Verbatim-in-box completeness (2026-07-04; breaklines LANDED same day)

Engine gaps behind the last ~1% of the 2605.00468 tcolorbox fidelity
arc (the class fixes — prevdepth glue transparency OXIDIZED #44, NFSS
family vocabulary #45, and the glowup verbatim contract — are landed):

1. ✅ **fvextra `breaklines` — DONE 2026-07-04**: the blanket
   `@Break→@NoBreak` line-processor neutralization in `fvextra_sty.rs`
   was an over-reach; only the `\FV@Break` char-scanner (the
   PushbackLimit/TokenLimit fatal source) needs relaxing. With the real
   `\FV@ListProcessLine@Break` running, every line is re-typeset as
   fvextra's `\parbox` (BOTH branches parbox — the over-wide one wraps),
   so the height budget counts the same wrapped lines pdflatex produces.
   Witness 2605.01024 (breaklines+breakanywhere fatal cluster):
   unchanged 4 errors, 0 fatals.
2. ✅ **Whitespace-river / 2× height budget — DONE 2026-07-04**: the
   `\lx@parbox` sizer was a pre-#2798 hand-rolled estimate
   (unwrapped-width/width, ceil, × baselineskip) that measured a
   one-line parbox at 2 baselineskips, inflating every breaklines
   prompt-box budget ~2×. Replaced with the faithful Perl delegation
   (sizer '#5' + Box::computeSizeStore: body through computeBoxesSize
   with the whatsit's width/vattach/totalheight; requested width wins).
   Also ported Perl's `\parindent\z@\parskip\z@skip` into the `\parbox`
   macro and the dropped `totalheight` property. 2605.00468 prompt-box
   fill 55–81% → **86% avg** (budget now line-exact on repro matrix).
3. ✅ **Leading spaces of verbatim lines — DONE 2026-07-04**: verbatim
   spaces are `\FV@Space` → `\FV@SpaceCatTen` (a braced ordinary space),
   eaten by TWO whitespace gates in the document builder (`open_text`'s
   initial-whitespace guard + `open_text_internal`'s Perl-L1146 gate)
   when the line's paragraph isn't open yet, plus the `ltx:p` afterClose
   trim. Fix: typewriter-font whitespace is never ignorable (guard
   bypass + `verbatim_space_pending` handoff + typewriter skip in
   `trim_node_whitespace`). JSON-schema indentation now preserved as
   REAL spaces (copy-paste-safe). Perl parity note: same-host Perl
   cannot convert these files at all (raw fvextra+breaklines exceeded
   7 min on a 6-line repro) — surpass-Perl scope.
4. **Prompt 1/6 budget undercounts wraps — paper-preamble-specific**
   (the remaining 2 spills on 2605.00468, 15/33px on 2/24 boxes,
   user-flagged 2026-07-05). CORRECTED diagnosis after bisection: NOT a
   `\small` attribution gap — in the paper the declared font at the fo
   AND its content block is serif-10 (traced), no size deltas exist to
   lose, and the budget counts NO wrapped lines for these boxes
   (~15 blocks × 12pt) while the browser wraps 6 borderline lines
   (383pt natural vs 345pt parbox width) → 19 rendered lines. The
   isolated repro chain does NOT reproduce (plain / breakable /
   breakable+title+colors all budget wraps correctly and emit `\small`
   deltas) — the trigger needs the paper's fuller preamble, prime
   suspect the colm class's inconsolata (`\ttdefault`=zi4) metrics vs
   cmtt in the line-width estimate (zi4 advance ≠ 0.525em → sub-list
   width/measure disagreement). Needs a preamble-bisection session with
   `LXML_SIZE_TRACE`; the speculative "anchor = declared fo font"
   change was built, traced, and REVERTED (no measurable effect — fo
   declared font equals the whatsit font in every observed case).
5. **Space-only verbatim lines still prune to empty** (blank-gap
   fidelity vs the PDF; render 0px + budget 0 = consistent, no
   overflow). Their spaces don't reach absorb (unlike line-leading
   ones); low priority.
6. **Non-verbatim `\ttfamily` lines in measured boxes don't wrap**
   (witness 2605.02240 `innercode`: `fontupper=\ttfamily\small` prose
   with `\\` breaks; pdflatex wraps each segment at the inner box
   width, our estimator emits one line-box per `\\` segment → 9–31px
   right pokes, ~2.7%). Same class as breaklines but general: paragraph
   wrap measurement inside measured boxes. Pre-existing (run-232-era
   binaries identical); not a July-5 blocker.

CSS side note: verbatim mono capacity is now token-derived
(`--code-font-advance` beside `--code-font-family`, `--tex-tt-advance`
constant) with `font-size-adjust: ch-width` upgrade where supported —
the browser font stays user-configurable; the conversion emits only TeX
facts (budgets + font-size anchor + abstract family). The breaklines
parbox shape has dedicated glowup rules (leaf-only `pre`/`pre-wrap`,
flex hbox rows, nested-picture fill-width exclusion).

### biblatex .bbl TokenLimit loop — 2605.17646 (pre-existing, NOT a PR regression)

A biblatex (apa style) paper whose `.bbl` ends in `\missing{Cowen2021}` hits
`Fatal:Timeout:TokenLimit` (999M tokens) during .bbl processing under the
ar5iv profile. Bisect 2026-07-04: **9a679469e1 (run-230 binary) fatals
identically** under equal local conditions (release, `LATEXML_TOKEN_LIMIT`
=50M, `--preload=ar5iv.sty`) — run 230's "error" status for this paper was
fleet nondeterminism, so the July PR branch did not introduce it. Repro:
`scratchpad fatal5/17646src` (arXiv 2605.17646). Suspect area: biblatex
runtime binding's refsection/datalist handling with `\missing`. Not a
July-5 blocker; needs a dedicated session.


### July-5 arXiv run — prep checklist (drafted 2026-07-02, user-approved sequence)

**Status 2026-07-05:** items 1, 3, 3b, 5 ✅ DONE — ar5iv-css **v0.9.0** released (on
jsDelivr); PR #273 merged → tag **`0.7.2`** "First public use of latexml-oxide in
ar5iv 2606" published (6 assets); `cortex_worker` rebuilt from tagged `main` +
fleet restarted; **ar5iv-editor redeployed to `latexml.rs`** (image
`20260705-9aafba841f`, public `/api/version` = `9aafba841f`, all services
healthy). Cross-repo required set is COMPLETE; items 6–8 are the run itself
(item 2 cortex/ar5iv CSS re-vendor: confirm).

Ordered; items 1–3 are cross-repo and REQUIRED (user, 2026-07-02):

1. **ar5iv-css `glowup`** — ✅ DONE 2026-07-05 (**v0.9.0** released, on jsDelivr):
   merged the `glowup` branch and **released a new ar5iv-css version**.
2. **Propagate ar5iv-css** to **ar5iv** (`~/git/ar5iv`) and **cortex**
   (`~/git/cortex`) — bump/vendor the released CSS in both (user, 2026-07-04:
   both should track the latest ar5iv-css whenever a release is available;
   cortex currently serves the glowup RC from `public/css/` — after the
   release, refresh those files from the released build, or point the
   preview template back at the released CDN tag).
3. **PR `ar5iv-2606-prep` → `main`** — ✅ DONE 2026-07-05: merged as **#273**
   (`8d9189f7e4`, squash) — parity fixes, perf audit + pin! sweep, fatal-mining
   fixes, docs consolidation. **Tagged + released** as `0.7.2` (`bdda7d4a33`),
   and **cortex** now runs a `cortex_worker` rebuilt from the tagged `main`
   (fleet restarted).
3b. **ar5iv-editor redeploy** — ✅ DONE 2026-07-05: rebuilt against
   latexml-oxide `main` @`9aafba841f` + ar5iv-css v0.9.0, pushed
   `ghcr.io/dginev/ar5iv-editor/{ar5iv-editor,ar5iv-validator}:20260705-9aafba841f`,
   cut over on `latexml.rs` (`/opt/ar5iv-editor/deploy`, `.env` repin + compose
   pull/up); public `https://latexml.rs/api/version` reports `9aafba841f`, all
   services healthy. Procedure + the `JAVA_HOME`=Java-21 vnu.jar gotcha captured
   in memory `ar5iv-editor-deploy-latexml-rs`.
   Mechanics (retained for reference): the editor path-deps on the sibling checkout and
   `deploy/Dockerfile` COPYs `~/git/latexml-oxide` into the build context —
   put the checkout on the tagged main, run `deploy/build-and-push.sh` +
   `deploy/release.sh`, and verify `/api/version` reports the tagged sha
   ("powered by latexml-oxide @<sha>").
   **CSS vendoring gotcha:** the editor EMBEDS ar5iv-css
   (`include_bytes!` of `frontend/public/css/ar5iv{,-fonts}.css`, plus the
   VS Code extension's `build:assets` copies the same files) and currently
   holds a PRE-glowup single-file copy. Glowup's `css/ar5iv.css` is modular
   (`@import "./ar5iv/*.css"`), so a raw copy silently drops the imports —
   re-vendor from the BUNDLED release build (`dist/ar5iv.min.css` /
   `dist/ar5iv-fonts.min.css`, lightningcss inlines the imports) and rebuild
   both the server crate and the extension.
4. ~~`f(x)` apply-vs-multiply dedicated session~~ — **CANCELLED 2026-07-02**:
   built, verified vs Perl, then reverted on user review; divergence #18
   (f(x) → function application) re-affirmed and stands. No math-output
   change ships in the July-5 binary from this item.
5. **After the current full-arXiv run finishes (~2026-07-04)**: rebuild
   `target/maxperf-cortex/cortex_worker` from merged `main` (fleet binary was
   deliberately NOT swapped mid-run). — ✅ DONE (folded into item 3's fleet
   rebuild from tagged `main`).
6. **Smoke canvas** on the new binary (a few hundred mixed papers via
   `tools/benchmark_canvas.sh`; verify fatal classes vs the known list, spot
   HTML with the new CSS).
7. **Corpus/service setup** for the July-5 (2606) run; verify the harness
   watchdog + memory-governor settings match `CORTEX_WORKER_HARNESS.md`.
8. Post-run: idle standing-corpus perf re-baseline (PERFORMANCE.md audit-log
   follow-up) — still OPEN — then ~~tag 0.7.0~~ **✅ tagged `0.7.2`** 2026-07-05
   (the release was cut now for the ar5iv 2606 first-public-use run rather than
   post-run; `0.7.0` rolled forward into `0.7.2`).

### Large arXiv corpus troubleshooting (2026-06-30, user-requested) — IN PROGRESS
**User directive 2026-06-30:** after the 2605 (10k/sandbox) troubleshooting, also troubleshoot
the **full arXiv corpus** at
<https://corpora.latexml.rs/corpus/arXiv/oxidized_tex_to_html>. **First pass done 2026-07-02**
(see the session entry above): live-run fatal mining at ~32% corpus produced 4 landed fixes
(2 panic sites, `\dabar@`, plain-`\+`) + PARITY verdicts for `\tikzcdmatrixname`/tikz-cd.
**Remaining threads for the next pass** (fresh fatals accrue as the run completes, ~2026-07-04;
fleet binary intentionally NOT swapped mid-run — rebuild only for the July-5 run):
- the residual `\lx@begin@alignment`/group-leak TooManyErrors family (516 papers; `\+` covered
  one driver, scalebox `\Gscale@@box` (~129, 2605 numbers) still open, others unidentified);
- the generic `_`/`^` math-mode cascade families (1.7k/1.4k papers — need sub-clustering by
  first-error);
- `never_completed_with_retries` (1,069) — sample for OOM/hang/crash witnesses
  (STABILITY_WITNESSES overlap);
- plain-layer leakage decision (55-name audit in the 2026-07-02 session entry): retract
  remaining tabbing entry points vs keep (user call pending).
Method: DB signature-clustering + `cortex_worker --standalone` (exact fleet binary) +
same-host Perl verbose; the canvas-triage skill encodes the rules.

### TokenLimit `tblr` colspec binding — ✅ DONE 2026-06-30 (`226d3bfa51`)
The cleanest fixable thread from the TokenLimit root-cause: `\tblr` now parses its inner spec,
extracts `colspec`, and translates the column mini-language to a classic `\tabular` template
(see the 2026-06-30 "Landed this session" TokenLimit note). **Remaining tabularray follow-ups
(not done):** the `colspec` translation drops X-column stretch (maps `X→l`) and ignores the
non-`colspec` keys (cell/row coloring, spans via `\SetCell`, `hlines`/`vlines` are no-ops) —
those are fidelity polish, not the alignment-leak/runaway bug (which is fixed). The babel-`.ini`
and expl3 TokenLimit hot loops (witnesses 2605.29738 / 2605.05840) remain deep open efforts.

### mhchem-manual fidelity mission (2026-06-27, on `followups-2026-06-27`) — LANDED
Driven by a manual review of `~/Downloads/mhchem.tex` (the mhchem package manual)
rendered with `--preload=ar5iv.sty --css=ar5iv.css --nodefaultresources
--path=~/git/ar5iv-css/css` (glowup branch), examined via playwright + Chrome.

1. **7 new `latexml_contrib` package bindings** for the manual's missing packages
   (errors 10→0): `fancyvrb-ex`, `rsphrase`, `hpstatement`, `tgpagella`,
   `sourcecodepro`, `AlegreyaSans` (raw-load real `.sty` where installed, per the
   user directive that raw-loading `.sty` is encouraged; fonts no-op where absent),
   and `scrreprt` (OmniBus `.cls` stub like `scrbook_cls`, + `\minisec`/`addmargin`/
   `\addtokomafont`). Perl ships no binding for any of these, so they are surpass-Perl
   contrib additions. `pstricks` already bound (its warning is a transitive
   fancyvrb-ex dep-scan artifact when the raw `pstricks.sty` is absent — benign).
2. **`\marginpar` font-leak fix** (`latex_constructs.rs`, `bounded => true`) — the
   manual's `\marginpar{\Large !}` leaked `\Large` document-wide (1388 `144%` nodes →
   4). PARITY bug (Perl 0.8.8 leaks identically); fixed surpass-Perl. OXIDIZED_DESIGN
   #39, KNOWN_PERL_ERRORS #38. Output-neutral (suite 1487/0).
3. **mhchem stub RETIRED → raw-load real `mhchem.sty`.** The engine's expl3/xparse/
   chemgreek support is now mature enough that `\usepackage{mhchem}` raw-loads the
   genuine package: chemistry renders with proper digit subscripts (`\ce{H2O}`→H₂O),
   charge superscripts, reaction arrows (`->`/`<=>`/`->[..]`), bonds, states,
   `\cesplit`. Simple `\ce` is 0 errors + correctly formatted (the old stub rendered
   formulae FLAT). chemformula stub updated to require mhchem with `version=4` (the
   real package warns without it; the old stub was silent). **Residual = SHARED Perl
   limitation, NOT a Rust gap (re-classified 2026-06-27):** the full manual still
   emits ~69 edge-case errors under raw-load (`\ce` inside `align*` →
   `\lx@begin@alignment`/`\end@amsalign`; ~56 `\lx@end@inline@math`). The minimal
   reduction `\begingroup$a$\endgroup` inside `align*` errors **IDENTICALLY in Rust
   AND same-host Perl** — deferred-alignment can't clean the cell `$`-frame across an
   intervening `\begingroup`. Nothing to fix for parity; a fix would be a deliberate
   deep surpass-Perl core divergence (not autonomous work). Basic
   `SideBySideExample`+`\ce` is clean. See memory `mhchem-ce-amsmath-alignment-2026-06-27`.

### `ltx_env_<name>` env-markup class — PLANNED, separate branch (churns every test XML)
**User-requested generic enhancement** (2026-06-27): tag environment wrapper markup
with `class="ltx_env_<name>"` so custom/minipage-like envs (e.g. `SideBySideExample`)
become responsively styleable in CSS instead of fixed-width minipages. **MUST be on a
dedicated branch** — it changes nearly every test XML (additive class on every env
element), so the golden-suite update is large and must be done in isolation.
Two implementations, same markup outcome:
- **Binding side (`DefEnvironment!`):** the constructor guarantees exactly one element,
  so unconditionally add `ltx_env_<name>` (via an `@ADDCLASS`/`add_class` after the
  begin constructor opens). Applies to ALL DefEnvironments (`figure`, `table`,
  `theorem`, `minipage`, …) — user chose full scope.
- **Raw side (`\newenvironment`/`\renewenvironment`):** arm at env start; at `\begin`
  construction record `{name, anchor = globally-unique gid of current node, mark}`; at
  `\end` afterConstruct, if EXACTLY ONE element was deposited under the anchor since
  the mark → tag it; zero (font/text-only) or >1 (siblings, e.g. SideBySideExample's
  parboxes) → nothing. **Needs a globally-unique monotonic node gid** (verify/ add;
  `record_node_ids` exists but is xml:id-oriented).
- **SideBySideExample:** keep the working `fancyvrb-ex` raw-load (correct source+result)
  + drive responsive layout from the resulting `ltx_minipage`/`ltx_env_*` hooks in
  `ar5iv.css`; do NOT re-implement the verbatim+render dual capture.

### 1. `\gls`/`\acrshort` in MATH mode (1705.10306) — RE-CLASSIFIED 2026-06-27: almost certainly PARITY (source-confirmed), blocked on unrunnable Perl
293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>`: a glossary command in
math mode digests the link display text (#3, the literal acronym term) as math →
bare per-letter `<XMTok>`, which the `glossaryref` content model rejects.
**Source-confirmed 2026-06-27 that this is most likely PARITY (NOT a Rust-only
gap — the cortex "Perl 1" is stale/unreliable, per `use-cortex-for-parity-work`):**
- Perl `Stomach.pm::enterHorizontal` (L422-434) is a **no-op in math** (`$mode
  =~ /math$/ => {}`) — Rust's `enter_horizontal` matches faithfully. So the
  `enterHorizontal => 1` on the shared `\lx@glossaries@gls@link` constructor does
  NOT switch #3 to text in math in EITHER engine.
- BOTH engines raw-load the SAME `glossaries.sty` (`InputDefinitions(noltxml=>1)`)
  with the SAME override constructor → both digest #3 in the ambient math mode →
  both produce `glossaryref > XMTok` → both hit the same schema rejection.
- `\ref`/`\cite` in math do NOT error (verified) — their content is STRUCTURED
  (bibref / ref-number), not a literal term; only `\gls`/`\acrshort` emit raw
  letter-XMToks. So glossaryref is specific, but the mechanism is shared with Perl.
- **The earlier "Perl raw-loads glossaries.sty and typesets as TEXT" hypothesis is
  weakened:** Rust raw-loads the identical `.sty`, so if it typeset the term as
  text, Rust would too. It doesn't (output: italic letter-XMToks) → so the `.sty`
  display chain does NOT force text in math.
**Perl confirmed UNRUNNABLE here (2026-06-27):** `latexml glx.tex` → `Fatal:terminate`
in `expl3-code.tex` (l3kernel) at 150 s — glossaries pulls in expl3 which is
pathologically slow in Perl 0.8.8 on this host; cannot capture ground truth.
**Fixing is therefore deferred as a likely non-bug.** If pursued, it parallels the
figure_mixed_content surpass-Perl pattern (a monotonic schema expansion to accept
the math content the builder already produces) — BUT the correct structure is
genuinely uncertain without Perl (XMTok directly? XMText-wrapped? operator-token
for the `\DeclareMathOperator` case? text PCDATA?), and there is **no precedent**
for `XMTok` in any inline element's model, so a speculative change risks an
unfaithful divergence. Repro + full notes:
`docs/reproducers/glossaryref_math_xmtok.tex`.

### 2. Release — ✅ `0.7.2` RELEASED 2026-07-05 (superseded the planned `0.7.0`)
Version bumped, `runtime-bindings` in the artifact, `.deb` deps, CHANGELOG/README
done. **Shipped:** tag **`0.7.2`** on `main` (`bdda7d4a33`, "First public use of
latexml-oxide in ar5iv 2606") → `release.yml` ran the TL-window `dumps` + macOS
arm64 leg + publish (each first-exercised on that tag); **6 assets live** —
Linux + macOS-arm64 tarballs and the `.deb`, each with a `.sha256`. The planned
`0.7.0` was rolled forward into `0.7.2` to fold the July-1–5 parity/perf/stability
fixes.

### 3. Speed: residual XSLT cost on large math books — ✅ FIXED 2026-06-29 (3rd O(n²) found)
After the seclev (`1172569034`) and head-keywords (`da74f6ecfe`) O(n²) XSLT fixes, the
slowest 2605 papers were multi-chapter math books where XSLT still dominated. Profiled
witness **2605.01585** ("From Qubit to Qubit", 2000+ formulae, 512 titles): `xsltproc
--profile` pinned **`maketitle` at 22.7 s of 24.9 s self-time (95 %)** — the inline
`not(//ltx:navigation/ltx:ref[@rel='up'])` full-tree scan, re-run **per title** =
O(titles × tree). Fixed by memoizing the document-global check into the global
`$maketitle_has_up_nav` (`LaTeXML-structure-xhtml.xsl`), same shape as the seclev fix.
**XSLT 24.94 s → 2.15 s (11.6×); maketitle self 22.7 s → 0.004 s; output byte-identical**
(`cmp` clean, 25 MB Core XML). Suite **1502/0** + guard `09_xslt_maketitle_navscan.rs`.
OXIDIZED_DESIGN #41, ARXIV_PERFORMANCE Hotspot #4. The three XSLT O(n²) templates on
large arXiv docs (seclev / head-keywords / maketitle) are now all O(n).

---

## Deep deferred families (parked — large or shared; dedicated sessions)

- **Native `.bst` interpretation — DEFERRED (pending plan, ~a few months out; do NOT
  start work that requires reading `.bst`).** arXiv's bibliography convention is codified
  in `ar5iv.sty`: LaTeXML prefers a ready-made `.bbl` and, only if none is present,
  interprets the `.bib` itself into XML internally (its own `MakeBibliography` conventions).
  In production this is a non-issue — arXiv's AutoTeX runs `bibtex`, so a `.bbl` is present
  and the conversion reproduces the PDF. The gap only appears when a conversion sees
  `.bib` + `.bst` but **no** `.bbl` (e.g. a standalone/manual run that skips `bibtex`):
  the `.bib`-direct fallback cannot reproduce the document's `.bst` output, because we do
  not read `.bst` yet. **Witness: arXiv:2605.16562** (LNCS, `splncs04.bst`). With a
  `bibtex`-generated `main.bbl` present, the bibliography matches the PDF exactly — PDF sort
  order, inline `\url`/`\doi` links, no "External Links:" label, corporate author rendered
  "W3C Math Working Group". Without the `.bbl`, the `.bib`-direct path still diverges from
  the PDF in ways that genuinely require the `.bst` (DEFERRED): LaTeXML's own alphabetical
  sort (different order from splncs04), "External Links:" prefixes instead of inline links,
  and DOI shown as bare text (`10.48550/...`) rather than a `https://doi.org/...` link.
  These are inherent to synthesising a bibliography from `.bib` without the `.bst`, not
  formatting bugs. **Resolution:** until native `.bst` interpretation lands, rely on
  `bibtex`/AutoTeX producing the `.bbl` (production already does); no latexml-oxide change.
  To reproduce: `latex main && bibtex main`, add `main.bbl` to the source, re-convert →
  matches PDF; remove it → diverges as above.
  NOTE: two *native-pipeline* bib bugs surfaced by the same witness were genuine and have
  been FIXED (they did NOT need `.bst`): (1) the duplicate Note/External-Links bibblock
  (`8ffca54713`); (2) brace-protected corporate authors mis-split into initials
  ("{W3C Math Working Group}" → "W. M. W. Group") and the `@inproceedings` `booktitle`
  dropped to a "See ," artifact — both from the simplified `.bib` parser
  (`convert_bib_file_to_xml`) and the lightweight XPath matcher in `document.rs`
  (value-less `[@attr]` predicate treated as always-true; `split('/')` fragmenting a
  predicate's `../`). Fixed: corporate-author detection in `parse_bib_authors`, and a
  bracket-aware / existence-checking `findnodes_by_traversal`.

- **`Fatal:Stomach:Recursion` (43 cortex Rust-service fatals) — TRIAGED 2026-06-28,
  mostly SHARED / Rust-better; ~1 Rust-only over-fatal DEFERRED (deep core).** Two
  guards in `stomach.rs`: the box-cycle "Infinite digestion loop" (9 papers,
  stomach.rs:1040) and the token-stack-depth "Excessive recursion(?)" (28 pkg-loading
  + 6 box/thm, stomach.rs:1343, `MAXSTACK=200`). **Same-host Perl parity on an 11-paper
  sample: ~10/11 SHARED** — the box-cycle/digloop papers (1906.06902, 1810.02304,
  1911.00254, 1911.11563, 2605.27339) **HANG in Perl 50–94 s** while Rust fail-fasts in
  <1 s via the guard (**Rust strictly better**); others (1809.00641, 2103.12717,
  1409.4048, 2011.08422) fail in BOTH. **1804.01117 (svjour3) was thought Rust-only but
  is actually SHARED — see the corrected deep-dive below (Perl `--includestyles` hits the
  identical readBalanced failure).** Crucially the limit
  **matches Perl exactly** (`Stomach.pm:159 $MAXSTACK=200`, identical guard at L175) —
  so it is NOT a mis-set cap; do NOT raise `MAXSTACK` (diverges from Perl and lets genuine
  infinite recursion run). The guard is doing its job — this category is a Rust **stability
  win**, not a bug cluster.
  **DEEP-DIVE of the lone Rust-only case 1804.01117 (2026-06-28): it is NOT a
  stomach-accounting bug — it is a tikz/pgf cascade.** Full stack capture: the top ~170
  frames are `{ \bgroup { \bgroup …` piled up by **`\pgffor@expand@list`** (pgffor's
  `\foreach`), immediately after `Error:pushback_limit:Timeout … loading binding for
  'tikz.sty'`. Rust fails to load the `tikz.sty` binding (pushback-limit), leaving
  `\foreach` in a broken state that floods the digestion stack → `Stomach:Recursion`;
  Perl loads tikz fine and never gets there. (The earlier "Rust digests packages deeper"
  hypothesis was WRONG.) Minimal `\usepackage{tikz}`, the full preamble package set, and
  `tikz`+`\foreach` in the body all load CLEANLY — the binding-load pushback only triggers
  under the paper's specific complex state. **FULLY ROOT-CAUSED 2026-06-28 (a 2nd deep
  dive) — it is NOT tikz/pgf either; it is a Rust `read_balanced` bug in xint.** The
  trigger is **`--preload=ar5iv.sty` + `xintexpr` (loaded before pgfmath/tikz)**. ar5iv
  (INCLUDE_STYLES) RAW-loads xint; `xintexpr`'s load of its built-in float functions
  (`\xintdeffloatfunc`, e.g. xinttrig's `@sind`) runs `\xintexprSafeCatcodes` (a
  `\begingroup`) then `\XINT_NewFloatFunc`/`\XINT_NewExpr` (xintexpr.sty:4721) whose
  body-compilation does a balanced read that goes UNBALANCED ("readBalanced ran out of
  input in an unbalanced state" + "Attempt to close boxing group").
  **✅ SURPASS-PERL LANDED 2026-06-28: 1804.01117 now converts FULLY under
  `--preload=ar5iv.sty` (0 Error/Fatal, 423 KB HTML, renders cleanly with `--css=ar5iv.css
  --nodefaultresources --path=~/git/ar5iv-css/css`; 463 native MathML nodes, 0 degraded
  body nodes). Perl LaTeXML still DEGRADES to a 459-byte error stub here** (`latexml
  --includestyles` → 26 errors, the IDENTICAL `readBalanced ran out` at xinttrig.sty:350),
  so this is a genuine beyond-Perl win. The chain: ar5iv (INCLUDE_STYLES) raw-loads xint;
  `xintexpr` does `\edef\X{\scantokens{...}}` where `\scantokens` opens an autoclose
  "Anonymous String" mouth MID-`\edef`-body and the `\edef`'s closing `}` is in the PARENT
  file. The fix is two-part, both faithful to tex.web `get_next`/`get_x_token` §362-365:
  (1) **`read_balanced` now CROSSES autoclose mouths** (gullet.rs `None =>` arm: close the
  exhausted autoclose mouth and resume the parent instead of `break`-ing unbalanced — the
  same crossing `read_x_token` already does; dump-neutral, suite 1491/0). This kills the
  `\xintexprSafeCatcodes` `\begingroup` leak → no "Attempt to close boxing group" → no
  TokenLimit cascade. DELIBERATE divergence from Perl (Gullet.pm:466 `last`s here and so
  also fails this input). (2) the prior-committed transient-`\noexpand` arg-capture decode +
  per-token `\special_relax` family + native `\Ucharcat` (see
  [[ucharcat-char-generate-noexpand-2026-06-28]]) which eliminated the `\XINT_expr_var_!`
  expr-compiler cascade.
  **Residual (HARMLESS, package-load-time only): 112 `Warning:expected:<number>` during
  xinttrig's `\xintdeffloatfunc` compilation** (56× `\the` seeing `$`, 56× `\romannumeral`
  seeing the f-stop `\special_relax\XINTusefunc`, all inside the "Anonymous String"
  scantokens mouth). xint's compiled expression token-stream is slightly MISALIGNED vs real
  xint, so a number scan lands on the f-stop. **Zero body impact** — this paper only
  `\usepackage{xintexpr}` and never evaluates an expression in the body. Full xint
  expression *evaluation* fidelity (so a real `\xintthefloatexpr sind(30)` computes the
  correct value, not just "doesn't crash") is a deeper, separate surpass layer — **parked**.
  **LONG-TERM FIDELITY FOLLOW-UP (user-flagged 2026-06-28):** the ar5iv rendering is a fair,
  successful conversion but not yet pixel-perfect — improve the *fidelity* of **subfigures
  and listings (reflow)**. Tracked here as a long-term task (not a correctness bug; the page
  is far better than the prior broken/Fatal state). Repro + full bisection history in
  `docs/reproducers/xintexpr_pgfmath_ar5iv_pushback.tex`. The Stomach:Recursion category
  itself still has **zero genuine stomach bugs**.

- **1610.00974 step-3 (global p{}→VBox) + cluster-B — ✅ LANDED 2026-06-22, NO
  LONGER DEFERRED.** See "Landed this session" above. p{}/m{}/b{} columns now build
  the cell as Perl's `\lx@tabular@p` inline-block (VBoxContents); p/m/b `<td>`
  `align="left"`; **cluster-B FULLY RESOLVED**; fixes 1510.07685. Commits
  `f65b80c1c2` / `eb978df5a9` / `1867f17da9` (+ box-model `7545e07fd6`). NOTE: the
  `collcell`/`\collectcell` undefined seen in some table papers is PARITY (both
  engines default `notex=1`/`INCLUDE_STYLES=false`, so neither raw-loads
  `collcell.sty`; the `--quiet` Perl "0 errors" was a display-suppression artifact —
  use verbose Perl).
- **`expected:id` cmml dangling-XMRef tail** — MathFork/split content-arm xml:id
  duplication; the last live `expected:id` class. See
  `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`. **★ CANONICAL WITNESS FIXED AT THE ROOT
  (2026-06-26q, LANDED on `class-b-xmref`):** the grammar rule `statements punct
  statement vertbar statements => vertbar_modifier_listlhs` makes a comma-list left
  of a conditional bar parse (`a,b|c` → `list@(a, conditional@(b,c))`, Perl-exact),
  so the witness's aligned `\Pr(s_A,s_B|\Omega)` arg parses → refs RESOLVE, dual
  PRESERVED. cb_repro & full witness `2311.01600` → 0 danglers; suite 1470/0; also
  fixes the standalone `a,b|c` aside. **RESIDUAL CHARACTERIZED (2026-06-26r):** the
  fix closed the "No node found"/DANGLING sub-case (canonical witness). The
  DOMINANT remaining `warning/expected/id` cortex cluster (**370 tasks**) is a
  DISTINCT class — `Missing idref on ltx:XMRef … _xmkey is `` ` (keyless XMRef, no
  idref, document.rs:3238), NOT a dangling idref — Rust-only (0704.2334 Rust 2 /
  Perl 0), from `\quad`/`\;`-separated **formulae/lists** with function-fence
  applies; context-dependent; root = `formulae_apply` content ref whose key never
  reaches the presentation item's top node (structure captured 2026-06-26t: a
  `formulae@` dual with a trailing bare `XMRef _xmkey=XM291` and no presentation
  top carrying XM291; the extend path doesn't clone `right`, so it's a subtler
  nested-relation/`\lx@dual` interaction). **SEVERITY: content-MathML QUALITY gap,
  NOT corruption** — the keyless ref has no idref so the prune sweep skips it; it
  survives with the faithful `Missing idref` Warn, schema-valid, no content dropped.
  Lower-priority cMML-polish item for the deferred math-fork session; the two
  higher-severity sub-classes (Class-B dangling + content-corruption) are FIXED.
  **★ COMMON SUB-CAUSE FIXED (2026-06-26v):** the keyless bare ref is a
  distribute-dual extend interaction — `distribute_list_relation` makes a
  `formulae`-content dual with a relation-`Apply` (non-Wrap) presentation; the
  formulae/list extend paths then push a content ref but silently skip the non-Wrap
  presentation → bare ref. Fix = gate the extend on a Wrap presentation (fall
  through to a fresh dual otherwise). Witnesses 0704.2334/0705.0790/0707.1173 →
  0 Missing-idref; suite 1471/0; regression `cluster_formulae_distribute_no_bare_ref`.
  PARTIAL: 0707.1339 still emits 2 (a different sub-cause). **QUANTIFIED 2026-06-22 (pre-fix): this WAS the
  #1 remaining Rust-only divergence** — `warning/expected/id` is **1005 cortex
  tasks** ("Cannot find a node with xml:id='S…E…m1.N'" from
  `latexml_math_parser/src/parser.rs:2840`; math-node ids, so genuinely the
  content-arm/MathFork XMRef cluster). It's a large Rust-only WARNING excess vs
  Perl (e.g. 0704.3530 Rust 152 vs Perl 9 warnings) — NOT parity. The prime
  candidate for the deferred content-MathML dedicated session; do NOT pick at it
  piecemeal (user directive). **FULLY DIAGNOSED + DE-RISKED 2026-06-26** (branch
  `class-b-xmref`, research-only, no code): same-host confirmed (0803.3810 Rust 51
  vs Perl 0), exact 6-dangler witness `2311.01600` (now `/data/arxiv/2311/`),
  Perl's target tree captured, a ~15s repro, and ALL peripheral fixes (clone/move/
  `.mf`/combos) empirically RULED OUT — the sole fix is the core post-parse
  preserving the structural XMArg ids (it rebuilds a fresh result tree → fresh
  per-row `{group}X.m1.*` ids, stranding the build-time `{group}.m1.*` refs). The
  re-id is in a distributed parse/install path (the `parser.rs:1354` reinstall is
  NOT it). **PIN SHARPENED 2026-06-26 (notes 2026-06-26i/j) — full end-to-end
  runtime trace; exact unrecord site identified by backtrace.** The danglers are
  the `\Pr` (physics-pkg `I_dual`) CONTENT-arm arg refs; the arg material is still
  present (ref merely dangles → any prune/drop is content loss, RULED OUT as a
  cheat). The arg XMArg (`_xmkey="1"`, `xml:id`) is **swallowed by the
  `parse_single` reparse of its ancestor presentation XMWrap** (`unrecord_node_ids`
  ← `parser.rs:1501`), NOT parse_rec'd standalone — so the working `parse_rec`
  id-transfer (`:1136-1196`, which heals the sibling dual args keys 2,3,5,6,7,8)
  never applies. RULED OUT (all empirically): prune/drop, `XProps` xml:id capture
  (dual not ingested via `From<&Node>`), `_xmkey` re-resolution + remap (parser
  REGENERATES keys; `XM::Arg` drops the build key). LANDMINE: the reparse
  orphan-detection (`:1502-1528`) is dead-code via the `@xml:id` namespace footgun;
  naively fixing it ACTIVATES a content-losing `__LOSTNODE__` drop. Two viable fix
  designs (key-carrying `XM::Arg` + re-point handler; OR cross-recursion old↔new
  `_xmkey` snapshot) with failure modes in the design doc. **DEFINITIVE ROOT
  (2026-06-26k, proven vs Perl source):** the ASF-vs-RecDescent node-identity
  divergence — Perl `parse_rec` returns an array-tree EMBEDDING the real parsed
  child nodes, so `appendTree` preserves their `xml:id`; Rust's ASF `into_xmath`
  REBUILDS fresh nodes (XM::Apply), so a re-materialized (non-`XM::Lexeme`)
  referenced target loses its id and the content XMRef strands. Faithful fix =
  identity-preserving `into_xmath` for non-leaf referenced nodes (reuse the input
  DOM node, like the leaf `XM::Lexeme` arm); LOSTNODES re-point is the pragmatic
  alternative. **TRIGGER ISOLATED (2026-06-26l):** the dangler is a downstream
  symptom of a CONTEXT-DEPENDENT **parse FAILURE** of the `\Pr` argument
  (`s_A,s_B|Ω_{len=k}` → `parse_single` returns `None`), so the `parse_rec` id-transfer
  (which heals the args that DO parse) never runs and the ancestor reparse strands the
  ref. Confirmed: the SAME arg parses standalone (0 danglers) — only the paper's
  preamble makes it fail in-context. Two fix axes (both dedicated-session): (A)
  parse-coverage (make the in-context arg parse; relates to the open VERTBAR/comma-list
  asides); (B) failure-robust id preservation via reused-leaf correspondence
  (`record_replacement(oldXMArgId, newTopId)` re-point, content-preserving). Precise
  repro + ruled-out approaches in `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`
  (2026-06-26a–o). The dedicated session = fix axis A or B + full math-fixture/corpus
  validation. **PARTIAL FIX LANDED (2026-06-26o, `class-b-xmref`):** an
  operand-protection guard in `prune_dangling_split_xmrefs` stops the broad `^S\d+`
  sweep from DROPPING `\Pr` content-arm arg refs (which emitted a malformed
  `apply(probability)` = silent content loss for section-numbered aligned `\Pr`);
  it now PRESERVES the arg (dangling, closer to Perl). 1469/0, clippy clean, does
  NOT re-flood wp3, regression test `cluster_xmref_pr_arg_not_dropped`. Does NOT
  make refs resolve — that is still the dedicated session (the leaf-LCA re-point,
  design B, works mechanically but collapses the dual; the faithful fix needs a
  CONTENT-branch arg copy, Perl's `.mf` scheme, via `rearrange_lone_ams_aligned`).
  **ROOT CAUSE + EXACT FIX FOUND (2026-06-26p) — AXIS A now recommended.** Bisected:
  only `\Pr(a,b|c)` (comma-list-LHS conditional) dangles; `\Pr(x)/\Pr(a|b)/\Pr(a,b)`
  resolve. The grammar's lone VERTBAR-modifier rule is `statement vertbar statements`
  (single LHS, `builder.rs:447`), so `a,b|c` doesn't parse → arg fails → ref strands.
  ONE-LINE fix `statements vertbar statements` TESTED: standalone `a,b|c` parses
  (fixes the open VERTBAR aside), witness → 0 danglers, refs **RESOLVE**, dual
  PRESERVED (faithful, = Perl's path). BUT regresses abs-value (`a|a|` →
  `conditional@(a,a)` not `a*|a|`; abs-value-vs-conditional ambiguity defeats
  `prefer_fewer_conditionals`). Reverted. Targeted fix = a `comma_statements`
  nonterminal (≥1 comma, not subsumed by `statements`) so the rule fires only on
  genuine lists, OR a pruning tweak — dedicated math-parser session. Axis A produces
  the genuinely-correct tree; preferred over the deep rearrange materialization.
- **xy-pic `svg:path` / curve cluster** (1501.03690) — shifted-arrows `svg:path`
  in `ltx:text`; mode-frame cascade root.

**SHARED (both engines fail — match Perl; do NOT "fix" by downgrading):**
- **1804.01117 xint raw-load** — both raw-load xint and fail (plain: both stub,
  byte-identical). The Rust stack-overflow crash is FIXED (gullet `stack_guard`,
  configurable via `latexml_core::stack_guard`). Deep xint emulation parked.
- **mode-frame auto-close cluster** (1611.04940, 2009.05630, 1702.06692,
  1702.02037) — a theorem env opened via its bare begin-command with no matching
  `\end…` leaks the mode-switch frame; Perl `Stomach.pm:343-376` errors
  identically. A graceful auto-close would *surpass* Perl (beyond-parity R&D).

---

## Reference (stable — not active work)

### Engine file open gaps (MINOR, demand-driven)
- `tex_box.rs` box-dimension edges; `tex_fonts.rs` `\fontdimen` array + per-font
  `\hyphenchar`; `tex_tables.rs` padding CSS (XSLT concern).
- **Document-builder block/paragraph auto-wrap of inline content** (core,
  broad/risky family — two witnesses):
  - **`\fcolorbox` inline paragraph-grouping**: an inline `\fcolorbox`
    mid-paragraph — Perl breaks the `<p>` (its `internal_vertical` block ends
    it), Rust keeps it inline. SAME flags on both; Rust's inline reading
    arguably matches real LaTeX's `\mbox`-based `\fcolorbox`. (`\colorbox`
    matches.)
  - **bare `\includegraphics` run in a figure** (witness 1108.0198, found
    2026-06-21 via skeleton diff — a clean, error-free reproducer): a
    `\begin{figure*}` with several consecutive `\includegraphics` (no blank
    line) — Perl wraps the inline run in a `<ltx:block>` (`figure > tags >
    block > graphics×N`), Rust emits the graphics bare (`figure > graphics×N`).
    Rust is error-clean and schema-valid. **Re-witnessed + root-confirmed
    2026-06-27** (0704.0001, 0704.0017 via the corrected structural diff): NOT
    merely cosmetic — the panel `<graphics>` WIDTHS also diverge (Rust 303.5pt vs
    Perl 241.5pt, ~1.257×), so figure sizing is visibly affected. Root: Perl's
    `arrange_panels_and_breaks` (`latex_constructs.pool.ltxml:3229-3295`) does a
    full box-metric panel layout — it inserts `<break class="ltx_break">` and wraps
    panels using `getNodeBox($child)->getWidth` vs `float_width`; Rust's
    counterpart (`latex_constructs.rs:1784-1869`) is explicitly **"Simplified: mark
    panel children with the class"** and skips the break/block arrangement. A
    faithful port DEPENDS on matching box widths → the deep box session (sibling of
    the `\resizebox` panel-width item below), not a loop-tick fix.
- **`\resizebox` panel scale-VALUE divergence**: in `complex/figure_mixed_content`
  two panels get a different computed natural width (xscale 1.13 vs 0.88). The
  construct in ISOLATION matches exactly (both xscale=1.9685); the divergence
  only appears inside the paper's `\footnotesize` + `table*` + `\subfloat` panel
  context → a font-size/box-context interaction. Scale *formatting* (%.15g) is
  already Perl-faithful (`551c5286ba`); missing-image candidates too
  (`64dd30b284`). Deep box-metric; for the focused box session.
- **~72-CS Perl-only long tail** (from the archived LoadFormat audit): misc
  atomics (`\@charlb`, point-size CSes, `\batchmode`, …) Perl defines, Rust does
  not. Investigate a CS only when a real paper witnesses it; refresh the CS-name
  diff before quoting counts (predates the BibTeX port).

### Primitive layer — AUDITED FAITHFUL (2026-06-20)
Probe-based Rust-vs-Perl audit found the core primitive layer byte-identical
(arithmetic, dimensions, glue, conditionals, string/token, case tables). Don't
re-audit without a witnessing paper. Shared-with-Perl quirks (NOT Rust bugs):
`\numexpr` divideround round-half-toward-+∞ (KNOWN_PERL_ERRORS #33); `\the\skip`
drops stretch/shrink to bare pt.

### Permanent ignores
- **Out-of-scope**: ns1–ns5 (`52_namespace`, no DTD support); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
- **Rust supersedes Perl**: `1207.6068`, `0909.3444`, + 40 more in
  `memory/project_rust_supersedes_perl.md`.
- **BibTeX**: `BibTeX.pool.ltxml` ported (Phases 1–8; remaining B1–B6 polish in
  `BIBTEX_PORT_PLAN.md`). `--nobibtex` is opt-out, not default.

### Tikz known diffs vs Perl
`foreignObject` transform; arrow-tip path data; SVG viewBox/width; matrix
`<svg:g class="ltx_tikzmatrix">` vs inline-blocks; **bare `svg:g` in `<ltx:block>`**
(tikz-cd) trips a core-XML validity error but post-processing recovers (witness
2006.12702) — Rust-only, low priority (output recovered).

### Graphics renderer chain (subprocess-only; LANDED)
PDF→PNG `mutool draw`→`pdftocairo`→`convert+gs`; PDF→SVG `mutool convert`→
`pdftocairo`→(raster PNG fallback). EPS/PS→`gs` direct→`convert+gs`. Subprocess
`exec` (no GPL linking). Apt: `poppler-utils` (req), `mupdf-tools` (rec),
`imagemagick+ghostscript`. A heavyweight inkscape third resort for PDF→SVG was
removed 2026-06-29 (GTK stack, 20–40× slower, timeout-prone, no coverage over the
raster fallback).

### Other tracks (separate docs)
- Performance: `PERFORMANCE.md` (P1 math/large-doc open; P2 allocation partial).
- Release gates: `RELEASE_CRITERIA.md`. Releasing: `RELEASING.md`.
- **BibTeX (plan archived 2026-07-02 →
  [`archive/BIBTEX_PORT_PLAN_2026-06-20.md`](archive/BIBTEX_PORT_PLAN_2026-06-20.md)):**
  Phases 1–8 shipped; live residuals = the Phase 4–5 field-handler/MR-Zbl
  long tail, divergences B1–B6 noted in `bibtex.rs`, and the deferred
  **native `.bst` interpretation** (witness 2605.16562, `f65cf7d6dc`) —
  demand-driven, pick up on corpus evidence.
- Completed missions (archived): strict-LoadFormat dump parity, Marpa ASF
  migration, distribution-readiness, the 500K/1M warning-corpus mission, the
  diagnostic-message faithfulness pass (2026-06-20), and the upstream-sync
  PR translation U1–U11 (2026-06-26) — see `docs/archive/` and `git log`.
