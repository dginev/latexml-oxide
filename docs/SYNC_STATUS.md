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
> record; lift anything worth re-reading into
> `docs/archive/SYNC_SESSIONS_YYYY-MM.md`). Last compaction: **2026-07-25**
> (1979 → 1179 lines; 23 completed sections lifted to `SYNC_SESSIONS_2026-07.md`).
>
> **Verify a status label before acting on it — and before deleting it.** Two
> entries here pointed at work that did not exist: a `⏸️ 13 commits NOT PUSHED`
> banner for commits already merged as PR #323, and a "#312 → next step: render
> under MathJax 4" bullet for an issue closed 2026-07-21 whose MathJax screenshot
> `ISSUE_AUDIT.md` records as out of scope. Check the **named guard test** in the
> tree, or `gh issue view <N>`. SHA-ancestry does **not** work as a check — the
> repo squash-merges, so a branch SHA quoted here is never an ancestor of `main`.

## Current status

- **2026-07-25 — session: siunitx v3 + split-fence math, and a worklist freshening.**
  Merged `0f7711c0b5` (PR #372) — faithful `six_format_complexnumber`
  (imaginary-unit semantics, `complex-root-position`, mantissa brackets; 0→17 of
  Perl's 20 golden signatures), the five undefined siunitx v3 commands,
  `\qtyproduct` off `\SIlist`, and Perl's `\sisetup` defaults mirrored 57→107 keys.
  Merged `0dda6ca833` (PR #373) — fences split by TeX's null delimiter now parse
  (divergence #67); measured over 24 arXiv 2606 papers carrying the pattern,
  `unparsed_math` **177 → 102**, 19 improved, 0 regressed, 0 new errors. Witness
  2606.13010 (arXiv/html_feedback#6624) now converts at 0 errors / 0 warnings /
  0 unparsed math. This file was compacted the same day — see the header.

- `cargo test --tests`: **1684 passing / 91 targets, 0 failed, 0 ignored**
  (2026-07-25, on `main` @ `0dda6ca833`, dev box with ImageMagick + ghostscript +
  poppler installed, `mutool` absent). +6 against the 2026-07-24 count of 1678;
  two of those are this session's `siunitx_v3_test` and `split_fence_test`.
  **Caveat that keeps mattering:** the two vector-SVG tests
  (`test_vector_svg_graphics_path`, `test_vector_svg_pathological_convert_case`)
  do NOT go red on a bare host — `svg_converter_available()`
  (`tests/integration.rs`) returns early and reports **ok** when neither `mutool`
  nor `pdftocairo` is on PATH, and the skip `eprintln!` is swallowed without
  `--nocapture`. So a green local run does not prove that branch ran; CI installs
  poppler/mupdf. (An earlier "one `latexml_post` graphics failure needs a host
  image tool" caveat was carried forward for weeks before being shown not to
  reproduce — no `latexml_post` test can produce it as written.)

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

## HANDOFF — session of 2026-07-20 (branch `more-minisprint-ar5iv`) — **MERGED as PR #323**

The 13 commits below shipped in `55c5ee5ead` ("ar5iv mini-sprint: alignment +
runaway fixes, parity triage"); the branch is gone and nothing here is at risk.
This section survives only for its **open threads** (below) — the "Landed" list
is history. *(It read `⏸️ … NOT PUSHED / Resume here` until 2026-07-25, which
sent a later session looking for lost work that had in fact merged. Verify a
"not pushed" banner against `git log main` before acting on it.)*

### Open threads, in the order I'd pick them up

- **#312 — CLOSED 2026-07-21, nothing outstanding.** The reporter's actual bug
  was ours and is fixed: a stale/empty `LaTeXML.css` in the destination was
  truncated rather than overwritten (`fs::copy` onto itself past a string-compare
  guard), so the browser rendered flush-left. Fixed by the canonicalize-based
  `same_file` check at all three copy sites — `a9a4c3c355` (PR #339), guard
  `004_stale_css_overwrite`. Their *other* complaint, "equations are not
  centered", is **parity**: the `ltx_eqn_table`/`ltx_eqn_center_pad*` markup and
  equation CSS are byte-identical to Perl's on their file.
  *(This bullet used to read "#312 is NOT demonstrated fixed … next step: render
  under MathJax 4". That contradicted `ISSUE_AUDIT.md`, which records the
  MathJax-4 screenshot as **out of scope**, and the operand-slot half renders
  identically in Chrome with or without the slot. Do not re-open a MathJax
  measurement without a fresh reason.)*
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

## CLI options — the option-C policy (issue #191 CLOSED 2026-07-09) + `validate()`

Issue #191 "support the original latexmlc/latexmlpost options" is **closed**;
what survives here is the standing **option-C policy** it established, plus the
one feature deliberately left undone (`validate()`, below). The policy: wire only
options whose engine feature genuinely works end-to-end; keep the clap parser
**strict** (no accept-and-warn stubs); deferred/missing features stay hard parse
errors. Consult it before adding any CLI flag.

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

### Beyond-Perl performance levers — from the 2026-07-10 60k-doc telemetry (POST-RELEASE — deferred out of release week; that stabilization review is in `docs/archive/SYNC_SESSIONS_2026-07.md`)

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
the release-week stabilization review (archived in `SYNC_SESSIONS_2026-07.md`);
first work after the tag ships:**
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

### `\gls`/`\acrshort` in MATH mode (1705.10306) — RE-CLASSIFIED 2026-06-27: almost certainly PARITY (source-confirmed), blocked on unrunnable Perl
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
