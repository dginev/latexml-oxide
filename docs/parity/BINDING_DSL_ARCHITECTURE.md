# Binding DSL Architecture — Decision Record

> **Status:** accepted (2026-06-09). Implementation in progress.
>
> This records *how* latexml-oxide models its binding-definition DSL
> (`DefMacro`/`DefConstructor`/`DefPrimitive`/`DefEnvironment`/`DeclareOption`/…)
> across a **compile-time** front-end (native `.pool` ports) and a **runtime**
> front-end (toolchain-free contributed bindings), without the two drifting.
> It also resolves how issues **#93** (declarative binding dialect) and **#171**
> (dedicated parser for XML replacements) relate to this architecture.

## Context

We need binding definitions reachable two ways:

1. **Compile-time, native** — the thousands of `Def*` calls that port LaTeXML's
   `.pool.ltxml` files, authored in Rust and compiled in. Terseness matters:
   these are compared line-by-line against the Perl originals.
2. **Runtime, contributed** — bindings authored next to a paper's `.tex`, loaded
   with no Rust toolchain and no recompile (the Rhai `script-bindings` feature;
   see [`script_bindings_plan.md`](script_bindings_plan.md)).

The risk: two front-ends implementing the *same* construct semantics
independently and drifting apart. Historically the lowering logic lived inside
`macro_rules!` arms in `setup_binding_language.rs` (option-bag building, closure
wiring), which is brittle, hard to share, and gives poor errors.

## Decision

**One shared lowering spine in `latexml_core`; separate *syntax* from
*semantics*; let any number of front-ends converge on the spine.**

```
  compile-time macro_rules!  ─┐
  runtime Rhai (#{ … })       ├─►  Builder (latexml_core)  ─►  def_constructor / def_macro / …
  (future) declarative YAML   ─┘     ConstructorBuilder, …       (the one place semantics live)
            #93
```

- **Semantics → builders** (`latexml_core::binding::def::builder::ConstructorBuilder`,
  and the same shape for macro/primitive/environment). Rhai-agnostic; take native
  values and closures, so they live in core and pull in neither the macro
  machinery nor Rhai. Scalar options route through one generic
  `set_option(key, OptionValue)` (the key→field `match` is single-source);
  closure options use typed setters (`after_digest`, …); `install()` calls the
  `def_*` function. This is the source of truth — testable plain Rust.
- **Syntax → thin front-ends.** The `macro_rules!` keeps the terse TeX-like
  surface (essential for the `.pool` ports) but its body becomes
  `Builder::new(proto)?.replacement(…).set_option(…)?.after_digest(…).install()`
  — no semantics. The Rhai layer feeds the same builder from an object map.
- **Template compilation is its own shared concern** (see #171 below): a
  constructor's `"<ltx:…>"` replacement is compiled to a `ReplacementClosure`,
  either at compile time (proc-macro, fast native path) or at runtime
  (`apply_template`, for Rhai). The builder accepts either — it is agnostic.
- **Anti-drift → conformance tests.** A test defines the *same* construct via two
  front-ends and asserts identical behaviour (landed:
  `builder_conformance_macro_style_vs_rhai_afterdigest`). Evolve
  `setup_binding_language.rs` freely; the test fails the moment a front-end falls
  behind.

## How #93 and #171 relate (today)

When filed, #93 and #171 were independent backlog items. The builder work has
made them **two components of one architecture**:

### #93 — *Declarative Def/binding dialect* → the umbrella, **closed 2026-06-09 in favor of #247** (Interpreted runtime bindings)
#93 asked for a declarative `Def*` variant that works **both at compile-time and
at runtime**. That is precisely this architecture, and the live work now lives in
**#247**. The Rhai `script-bindings` layer + the builder spine **are** the runtime
half; the thin macro-over-builder is the compile-time half. #93 sketched a
*YAML/TOML* surface;
we chose **Rhai** because #93's own examples need expression evaluation
(`properties: href: CleanURL(ToString(#1))`, `condition: LookupValue(...)`) —
i.e. a constrained scripting language, which Rhai provides generally. Crucially,
a **YAML front-end is not excluded**: on the builder spine it would be a *third*
front-end converging on the same lowering, ideal for the pure-data simple cases
#93 wanted. So #93 is being realized, with the syntax question reframed:
Rhai now, optional declarative-YAML later, both on one spine.

### #171 — *Dedicated parser for XML replacements* → a component, now more urgent
#171 is the **template-compilation** piece of one construct: turning
`"<ltx:…>"` into Document operations. It is now *more* pressing than when filed,
because the architecture introduced a **second** template implementation:
- compile-time: the ported `Constructor::Compiler` proc-macro
  (`latexml_codegen`, the "fragile" code #171 calls out);
- runtime: `apply_template` (`latexml_contrib::script_bindings`), for Rhai.

Two implementations of one template language is exactly the drift this doc exists
to prevent. The right resolution in the two-front-end world is a **single XML-
replacement parser producing a shared AST/op-list**, consumed by *both* the
compile-time codegen and the runtime interpreter. Note the libraries #171 lists
(RSTML/leptos, typed-html/axum, RSX/dioxus) are **compile-time-only** code
generators — they could sharpen the proc-macro path but cannot serve the runtime
interpreter, so adopting one wholesale would *deepen* the split. The fit is to
borrow their parsing/error-reporting ideas for a shared parser, not to switch the
whole template path to a compile-time HTML macro.

**Summary:** #93 is the dialect umbrella (front-ends on the builder spine); #171
is the shared template-compilation component within it. The builder spine and the
runtime template interpreter are the connective tissue that now relates them.

## Rejected alternatives (for the semantics layer)

- **`macro_rules!` as the lowering site (status quo).** Terse surface, but
  semantics-in-macro-arms is unmaintainable and unshareable. Keep the macro as
  *syntax only*.
- **Full proc-macro for the whole DSL.** More idiomatic for complex syntax, but
  the project already tracks rust-analyzer instability from `latexml_codegen`
  proc-macros (see CLAUDE.md), and the builder already owns semantics — so a
  heavy proc-macro front-end buys little and worsens RA. Reserve proc-macros for
  where compile-time work pays (template compilation, #171).
- **Builder-only, no macro.** Most maintainable in isolation, but too verbose at
  the thousands of `.pool` call sites and harder to diff against Perl.
- **Pure data tables / `ConstructorOptions { …, ..Default::default() }`.** The
  most idiomatic for *pure-data* options (and worth exposing for power users),
  but closures-as-options (`afterDigest`) are awkward as data; the builder's
  typed setters handle them better.

## Migration plan (lowest-risk first)

1. **Done:** `ConstructorBuilder` is the shared lowering; the Rhai path routes
   through it; conformance test green.
2. **Migrate `DefConstructor!` arms to emit builder calls**, arm-by-arm, guarded
   by the conformance test + the native test suite. Macro keeps its syntax.
3. **Done (#171):** the proc-macro and the runtime interpreter share one
   `winnow` parser + `ReplacementOp` AST in `latexml_core`
   (`binding/def/replacement.rs`). See "Implementation landed" below.
4. **Repeat** for `MacroBuilder` / `PrimitiveBuilder` / `EnvironmentBuilder`.
5. **Optional:** add a declarative-YAML front-end on the spine (closes #93's
   original data-format idea for simple bindings).

## Status / pointers

- Landed: `latexml_core/src/binding/def/builder.rs` (`ConstructorBuilder`);
  Rhai constructor path on the builder; conformance test.
- Landed (#171): `latexml_core/src/binding/def/replacement.rs` (shared
  `ReplacementOp` AST + `winnow` parser + runtime interpreter); both consumers
  retargeted onto it. See "Implementation landed (2026-06-09)".
- Runtime layer detail + dialect reference: [`script_bindings_plan.md`](script_bindings_plan.md).
- Tracking issue: **#247** (Interpreted runtime bindings). Related: #93 (the
  declarative-dialect umbrella, closed → #247), #171 (the shared template-parser
  component) — see [`ISSUE_AUDIT.md`](../release/ISSUE_AUDIT.md).

---

# Resolved decisions (2026-06-09)

From the DRY audit of the compile-time `DefConstructor!` macro vs the runtime
Rhai DSL:

1. **Macro ↔ builder unification: incremental.** Keep the `DefConstructor!`
   macro working; make `ConstructorBuilder` the canonical lowering and migrate
   macro arms onto it opportunistically, each guarded by the conformance test +
   the native test suite. (Avoids a risky big-bang touch of every native
   binding.)

2. **`&func(...)` in templates at runtime: a whitelisted function registry.**
   Compile-time templates resolve `&func(...)` to Rust calls; the runtime
   interpreter resolves them through a curated allow-list of registered helpers
   (e.g. `CleanURL`, `ToString`) — safe for untrusted contributed bindings.

3. **Unknown / misspelled option keys: fail-fast (error).** Both front-ends
   reject unknown keys, catching typos and making the conformance guarantee
   strict. (Supersedes the macro's current silent generic fallback and the
   Rhai layer's silent-ignore.)

4. **XML-replacement parser (#171): adopt `winnow`; AST + parser live in
   `latexml_core`.** Rationale:
   * winnow gives a **correct RD implementation** (cursor/backtracking/lookahead/
     char-boundary/EOF) and a **structured error model** (positions, context
     stacks, `cut_err`) — directly serving the fail-fast, contributor-facing
     errors. The current `constructable.rs` is a 639-line *regex-strip state
     machine fused to codegen*, not a reusable parser; both a hand-written RD and
     winnow are rewrites to *parse → `ReplacementOp` AST → two thin consumers*.
   * winnow is **already lock-pinned** (0.7.15, via `toml_edit`) and authored by
     the maintainer of the `clap` we already use — no new ecosystem dependency
     risk; modest binary cost (leanest/fastest combinator lib).
   * **No new crate:** `latexml_codegen` already depends on `latexml_core`, so the
     shared `ReplacementOp` AST + winnow parser live in core. Compile-time codegen
     (in `latexml_codegen`) consumes the AST for `quote!`; core's runtime
     interpreter consumes it directly. The AST is the stable interface, so the
     parser stays swappable.
   * Hand-written RD was considered (zero-dep, frozen small grammar) and is a
     reasonable fallback, but winnow's correctness + error guarantees were judged
     worth the (already-present) dependency.

**Build sequence for #171** (de-risked): (a) add `ReplacementOp` AST + winnow
parser in core with golden tests + a conformance test against the *existing*
`constructable.rs` compiler output; (b) retarget the `constructable.rs` proc-macro
to parse → AST → `quote!`; (c) retarget runtime `apply_template` to parse → AST →
interpret (gaining `#prop`/conditionals/PIs/floats + the `&func` registry);
(d) the native test suite (1334) + the macro↔Rhai conformance test are the
regression guards.

---

# Testing strategy for #171 (decision, 2026-06-09)

**Care = tests.** The shared `ReplacementOp` parser/AST and both consumer
retargets (compile-time codegen, runtime Rhai interpreter) are gated by a
conformance corpus drawn from the **most complex *real* templates in the
supported binding files** — not toy cases — asserted through **both** downstream
consumers. We do not remove the existing `constructable.rs` regex-strip codegen
until this corpus passes in both consumers and the full native suite stays green.

## The corpus (real, active specimens)

1. **`\footnote{}{}`** — `latexml_engine/src/plain_constructs.rs:293`
   ```
   ^<ltx:note role='footnote' ?#mark(mark='#mark')()>?#prenote(#prenote )()#2</ltx:note>
   ```
   The single richest active specimen: leading **float** (`^`), a **conditional
   attribute** (`?#mark(mark='#mark')()`), **conditional content**
   (`?#prenote(#prenote )()`), **`#prop`** holes (`#mark`, `#prenote`), an **`#n`**
   arg (`#2`), and a nested element.

2. **Class/package PI** — `latexml_engine/src/latex_constructs.rs:2702`, `:4088`
   ```
   <?latexml class='#2' ?#1(options='#1')?>
   ```
   A **processing instruction** with `#n` attribute interpolation and an **inline
   conditional inside the PI body**.

3. **Plain template baselines** (regression floor): a simple
   `"<ltx:emph>#1</ltx:emph>"`-class constructor and an attribute-bearing
   `"<ltx:text class='…'>#1</ltx:text>"` — to keep the common path exact.

(Provenance is pinned by file:line so the corpus tracks the live bindings.)

## What each corpus entry must assert — in BOTH consumers

For every corpus template:

* **(a) Compile-time path** — the native `DefConstructor!` (as it ships in the
  `.pool`) drives a tiny conversion that exercises the construct; assert the
  produced XML.
* **(b) Runtime path** — the *same* template loaded via the Rhai
  `DefConstructor("\\cs{}", "<…>")` form drives the *same* conversion; assert
  **identical** XML to (a).
* **(c) Golden AST** — `parse(template)` equals the expected `Vec<ReplacementOp>`
  (unit-level, no consumers).

(a)+(b) together are the anti-drift guarantee for the template layer; (c) pins
the parser itself.

## Gate

The `constructable.rs` regex-strip codegen is replaced by the AST→`quote!` codegen
**only after** (a)+(b)+(c) pass for the full corpus **and** the 1334 native tests
remain green. Until then, the new parser lands alongside the old codegen and is
exercised by the runtime path + golden/corpus tests.

## Scoping note

`&func(...)` and `%&hash` template features currently appear only in *commented*
(Perl-only / not-yet-ported) bindings, so the active corpus centers on
float / conditional / `#prop` / PI. The parser must still implement `&func`
(with the whitelisted runtime registry) and escapes for parity, but those get
their own focused tests rather than gating the active-binding corpus.

## Performance check (folded in)

The corpus conversions double as the no-regression probe: the **compile-time path
must stay byte-identical** in emitted ops (so native runtime timing is within
1–2%, ideally faster), and the **runtime path parses each template once** into a
cached AST (eliminating today's per-invocation byte-scan). Binary-size delta of
`winnow`-in-core is measured and recorded.

---

# Implementation landed (2026-06-09)

The shared template parser/AST and both consumer retargets are implemented.

## What landed

- **`latexml_core/src/binding/def/replacement.rs`** — the single source of truth:
  the `ReplacementOp` AST, a `winnow` parser (`parse_replacement`), the runtime
  interpreter (`apply_ops`), and `unquote`/`slashify` helpers. Faithful to Perl
  `Compiler.pm` / the old `constructable.rs`, including quirks (e.g. `unquote`
  deletes `\X` escapes; `^\s*<` whitespace-eating asymmetry between open and close
  tags; `font=` attribute dropped). `winnow = "0.7"` added to `latexml_core`
  (was already lock-pinned via `toml_edit`).
- **Runtime consumer** — `latexml_contrib::script_bindings::template_replacement`
  now parses each template **once** at wire time into the cached AST and runs
  `apply_ops` per invocation. The old per-invocation byte-scanner
  (`apply_template`/`parse_attrs`/`interpolate_attr`, ~140 lines) is **deleted**.
  This also fixes a fidelity gap: attributes now render via `to_attribute()`
  (matching codegen), not the byte-scanner's `untex()`.
- **Compile-time consumer** — `latexml_codegen/src/constructable.rs` is rewritten
  (783→~330 lines) to `parse_replacement(template)` and walk `&[ReplacementOp]`
  emitting the same `quote!` `Document` calls. The regex-strip state machine is
  gone; the emitted code is unchanged.

## How the dual-consumer guarantee is established

1. **Golden AST tests** (`replacement.rs`) — the shared parser produces the
   expected op-list for both corpus specimens (`\footnote` float+conditional-attr+
   conditional-content+`#prop`+`#n`; the class/package PI with an inline
   conditional) plus baselines.
2. **Evaluation-semantics conformance** (`replacement.rs`, Document-free) — the
   interpreter's attribute/condition/`#prop`/`#n` evaluation computes exactly what
   the codegen derives (pins `to_attribute()` rendering, the conditional truth
   test, the `font=` drop).
3. **Runtime e2e** (`30_script_bindings.rs`) — corpus-shaped templates (incl. a
   top-level conditional, both branches) drive a real conversion through
   `apply_ops`.
4. **The codegen retarget is behavior-neutral** — verified by restoring the
   original `constructable.rs`, rebuilding, and observing **identical** behavior
   on the native suite. Because both consumers now share `parse_replacement`, and
   the native suite proves the AST→codegen rendering is faithful, the runtime
   interpreter (same AST) is conformant by construction.

## Documented residual

`slashify` is retained in the **codegen** literal path for byte-identical
emission; the **runtime** path does not slashify (correct — there is no Rust
source to embed into). The two therefore differ only for *literal backslash text*,
which does not occur in any active template (escapes appear only in commented
Perl-only bindings). Tracked as a benign, untriggered divergence.

## Performance gate — PASSED (2026-06-09)

Measured against the pre-#171 commit (`d70282644b`), release profile, same
embedded TL2025 dumps in both binaries:

- **Binary size**: 45,794,032 → 45,819,888 bytes = **+25.9 KB (+0.056%)** —
  the entire monomorphized winnow cost; far inside the 1–2% budget.
- **Wall clock** (`tests/complex/si.tex`, 5 interleaved release runs each):
  base ≈ 1.780 s, new ≈ 1.792 s — **±0.7%, within run-to-run noise** on the
  measurement host. Peak RSS unchanged (~331 MB both).
- **Output**: converted XML is **byte-identical** between the two binaries.
- **Runtime template path is strictly faster by construction**: parse-once
  cached AST at wire time vs the old per-invocation byte-scan.
- Full native suite after the retarget: **1409 passed / 0 failed** (55 test
  binaries; the earlier "failures" were a dumpless-checkout artifact, fixed by
  generating dumps + the `build.rs` dumps-dir tracking fix).
