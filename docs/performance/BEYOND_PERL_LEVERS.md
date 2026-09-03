# Beyond-Perl performance levers (BP-1 … BP-6)

> Lifted out of `docs/SYNC_STATUS.md` on 2026-07-25. Derived from the
> 2026-07-10 60k-doc telemetry; companion to `ARXIV_PERFORMANCE.md` (the
> measurement campaign) and `PERFORMANCE.md` (timeless principles).
> **POST-RELEASE** — deferred out of release week by the stabilization review.
> Immediate static findings and implementation handoff are tracked separately
> in [`PERFORMANCE_AUDIT_2026-09-03.md`](PERFORMANCE_AUDIT_2026-09-03.md).

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
not libxml). *Lever Perl lacks:* Parse::RecDescent + single thread.

**Enabling blocker:** the current IR is not portable between worker threads.
`Token.text` contains a `SymStr` handle into a thread-local arena; the same
numeric handle has no stable meaning in another thread's independently
populated arena. Box/state/libxml structures also carry `Rc`, `RefCell`, or
thread-affine nodes. Do not send the existing IR to Rayon as-is.

*Required approach:* define an owned/shared-string formula snapshot, re-intern
it on the worker, return a portable parse result, and graft DOM nodes
sequentially in original order. Measure the O(tokens) boundary conversion and
audit every parser context input before claiming independence. *Feasibility:*
medium after this prerequisite; output neutrality remains mandatory.

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

> **⚠️ MEASURED 2026-08-18 — Step 1 is REFUTED; the "re-parsed per process"
> premise is false.** Flat CPU profile of witness 1910.01256 (AMD Threadripper,
> `perf` cycles self-time, 65k samples/20 iters): every `xsltParseStylesheet*`
> symbol is **≤0.01%**, the whole `libxslt` DSO is **0.6%**, `libexslt` 0.02%. The
> stylesheet **compile is below sampling noise** — precompiling/embedding a parsed
> stylesheet buys **<0.1%**. The visible libxslt time is template *application*
> (`xsltApplyTemplates`/`xsltGetTemplate`), not parsing, so **only Step 2
> (transpile hot apply-templates) remains live**. Caveat: 1910.01256 is
> small/text-heavy (XSLT is only **0.9%** here); the 13.2% figure is a corpus
> aggregate dominated by large docs, so **re-measure the XSLT *apply* share on a
> big multi-section paper before committing to Step 2** — don't invest on the 13.2%
> aggregate alone.

> **Median-path micro-wins found in the same profile (2026-08-18)** — cheap,
> divergence-neutral, off the BP roadmap but worth landing opportunistically:
> **(1) UTF-8 SIMD fast-path on the `.cls`/`.sty` dep-scan slurp** (`binding/content.rs`
> — `from_utf8_lossy` walked a 197 KB `ieeeconf.cls` grapheme-aware; ~3% CPU) —
> **LANDED** on `perf-utf8-slurp-fastpath`, byte-identical. **(2) `state.rs:789`
> `format!("{:?}") != format!("{:?}")` value compare — SPIKED + REFUTED 2026-08-18,
> do not chase.** `diff_from_snapshot` runs ONLY at dump *generation*
> (`ini_tex.rs:214`, `make_formats.sh`/release), NEVER during paper conversion — the
> LBR-less AMD profile misattributed the 4.3% fmt cluster to it (no call tree). That
> fmt cluster is actually **eager logging** (`Info!`/`Warn!`/`Debug!` format their
> args before the level gate + token `{:?}` in messages), wasted only in bare-CLI
> suppressed runs — a bound log buffer (`--log`/fleet) consumes them, so it's not a
> production win either. Leave state.rs:789 alone: it feeds the format-dump parity
> oracle, so a structural-`eq` swap risks changing which entries serialize for zero
> conversion benefit. **Method lesson:** on the AMD box (no LBR), verify a profile's
> caller attribution against the actual call graph before acting. The **25.5% gullet
> loop** and **12.7% alloc/memcpy** token churn are architectural, not free wins.

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

**BP-6 — Streaming Fragmented Core DOM — IMPLEMENTED.** The fragmented
digest/build, disk segment store, pass-2 driver, label/id index, assembly splice,
CLI flag, and auto-activation are live; see
[`STREAMING_CORE_DESIGN_2026-07-29.md`](STREAMING_CORE_DESIGN_2026-07-29.md).
The two-pass large split front-end is also implemented in
[`STREAMING_POST_DESIGN_2026-07-06.md`](STREAMING_POST_DESIGN_2026-07-06.md).

**Residual, not BP-6 reimplementation:** core XML is still collected into a
document-sized `String` before post, and pass 2 repeats global map/rule work per
segment. Complete the writer/file handoff and shared pass-2 ownership described
by audit F3/F4. Retain libxml2: dynamic `DefRewrite` XPath remains the hard
constraint that invalidates a wholesale pure-Rust tree substitution.

**Digest (19.7%) note:** sequential TeX engine — **no** parallelism lever; the win
is algorithmic:
- Profile hot macros with sampled `EXP_TRACE` histogram.
- Negative `SymHashMap` probes already use `arena::get`; preserve the invariant.
- Duty-cycled macro-cycle detection is already implemented; do not add a
  second guard without a new failing witness.
- Fast-path internal TeX counters (`if_count`, `if_limit`) via typed `State` fields.

---

## Ranked Roadmap (Updated 2026-09-03)

Refined after the 2026-08-18 profile, the 2026-09-03 source reconciliation,
and the landing of BP-6. This is the longer-horizon roadmap; execute the
smaller ranked work in `PERFORMANCE_AUDIT_2026-09-03.md` first.

1. **BP-5 (content-addressed formula memoization):** the lower-risk math lever;
   use a bounded cache and include every parse-affecting context in the key.
2. **BP-1 enabling work, then parallel math:** define and benchmark the portable
   formula/result boundary before introducing Rayon.
3. **BP-3 (bounded graphics concurrency):** first re-profile process scheduling
   and existing worker limits; concurrency must obey `THERMALS.md` and the
   external-process discipline in `PERFORMANCE.md`.
4. **BP-2 Step 2 (native hot XSLT transforms):** proceed only after a large,
   representative document identifies apply-time templates. Stylesheet
   precompilation is already refuted.

BP-6 is no longer a roadmap item. Its implementation residuals are ordinary
performance backlog work in the dated audit.
