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
3. **Scope the proc-macro to template compilation only**; have it and
   `apply_template` share one parser/AST (this is #171's concrete deliverable).
4. **Repeat** for `MacroBuilder` / `PrimitiveBuilder` / `EnvironmentBuilder`.
5. **Optional:** add a declarative-YAML front-end on the spine (closes #93's
   original data-format idea for simple bindings).

## Status / pointers

- Landed: `latexml_core/src/binding/def/builder.rs` (`ConstructorBuilder`);
  Rhai constructor path on the builder; conformance test.
- Runtime layer detail + dialect reference: [`script_bindings_plan.md`](script_bindings_plan.md).
- Tracking issue: **#247** (Interpreted runtime bindings). Related: #93 (the
  declarative-dialect umbrella, closed → #247), #171 (the shared template-parser
  component) — see [`ISSUE_AUDIT.md`](ISSUE_AUDIT.md).

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
