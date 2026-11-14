# Runtime Script Bindings (Rhai) — Plan

> **Status:** ✅ **IMPLEMENTED and SHIPPED BY DEFAULT** (updated 2026-07-20).
> `runtime-bindings` is in `latexml_oxide`'s `default` features and in the
> distribution build (`tools/make_release.sh`); M0–M5 are all landed and the
> live contract is the generated `API.md` (see "The script API" below).
> *(Original header: "proposal / planning (2026-06-08). Spike in progress.")*
>
> **What:** an **optional, secondary, feature-gated add-on** that lets users
> contribute LaTeXML package/class bindings **at runtime** — written in an
> embedded scripting language (Rhai), loaded from data files, with **no Rust
> toolchain and no recompile** — at the cost of interpreted-execution
> performance. The native, compiled-in bindings remain the primary, full-speed
> path; this changes nothing about them.
>
> **Supersedes** the abandoned Perl-emulator idea (embedding libperl), which was
> dropped as unreliable — see the `perl-embedding-crate-eval` memory. The
> lesson driving this design: **no native FFI, no unstable ABI, no pre-1.0
> single-maintainer C dependency.**

## 1. Scope and non-goals

- **Primary path unchanged.** Compiled-in Rust bindings (the `BINDINGS` table,
  `DefMacro!`/`DefConstructor!` macros, the prelude) stay the default and the
  fast path. This feature does not touch them.
- **Add-on, off the hot path.** Only bindings a user *loads as a script* pay the
  interpretation cost. A conversion that uses no script bindings is byte-for-byte
  unaffected.
- **Toolchain-free authoring.** The whole point: contribute a binding without
  `cargo`, without a compile, without matching our toolchain. Edit a text file,
  run a conversion.
- **Two tiers, one API, a graduation path.** Script bindings and native bindings
  drive the *same* registration seam, so a script binding that proves popular or
  hot can be **mechanically ported to native Rust** and compiled in. The add-on
  doubles as a low-friction staging area for future first-class bindings.
- **Non-goal:** running *existing Rust* contrib source as-is (that needs
  `dlopen`/ABI-locked plugins — the fragility we are deliberately avoiding).
  Script bindings are authored in Rhai.
- **Non-goal:** performance parity with native bindings. Interpreted is slower,
  consciously accepted, scoped to the user's own bindings.

## 2. Why Rhai (and not the alternatives)

Decision rationale, through the reliability lens that killed the Perl approach:

| Option | Verdict |
|---|---|
| Embed libperl (Perl closures) | **Abandoned** — pre-1.0 FFI crate, unstable C ABI, broke on host glibc. Unreliable. |
| Interpret Rust source | **Doesn't exist** — no mature embeddable Rust interpreter; Rust is AOT/monomorphized/borrow-checked. |
| `dlopen`/`abi_stable` native plugin | **Rejected** — Rust ABI is unstable across compiler versions *and runs*; toolchain-locked; breaks the self-contained binary. Same fragility class as Perl. |
| WASM (`wasmi`) | Viable but heavier — needs a wasm build step + a marshaling boundary; defers the "no toolchain" goal. |
| **Rhai** (embedded scripting) | **Chosen** — pure Rust, zero native deps, no FFI, no ABI, sandboxed, self-contained, mature. |

Rhai's concrete wins for *this* use case:
- **Pure Rust, compiles wherever Rust compiles.** The exact wall we hit with
  libperl-sys (host header/ABI parsing) cannot occur.
- **No FFI boundary.** Errors are ordinary Rust `Result`/`EvalAltResult`. No
  `croak`/`longjmp`/`catch_unwind` UB tightrope.
- **Sandboxed by default + resource limits** (max operations, call depth, string
  size; no file/network/process access unless registered). Makes running
  **untrusted third-party contrib bindings safe** — which the Perl path (raw
  arbitrary code execution) never could.
- **Self-contained binary preserved** — Rhai is a crate; scripts are data.

Cost, stated plainly: bindings are authored in **Rhai** (a dynamically-typed,
JS+Rust-ish language), not Rust. That is the price of toolchain-free runtime
loading.

## 3. The keystone seam (reused from the Perl plan)

latexml-oxide already stores every definition body as a boxed Rust closure
(`latexml_core/src/definition.rs`): macro = `Rc<dyn Fn(Vec<ArgWrap>) ->
Result<Tokens>>`, primitive = `Rc<dyn Fn(Vec<ArgWrap>) -> Result<Vec<Digested>>>`,
constructor replacement = `Rc<dyn Fn(&mut Document, &args, &props) -> Result<()>>`,
etc. State is reached via a `#[thread_local]` singleton, not passed in.

**Model A (fine seam):** a script's `DefMacro`/`DefConstructor` call registers a
**native Rust definition** via the existing `def_macro`/`def_primitive`/
`def_constructor` functions (`latexml_core::binding::def::dialect`, all `pub`).
The body closure trampolines into the Rhai engine to invoke the stored Rhai
function. Prototype parsing, argument reading, Whatsit/digestion plumbing, the
document build — all stay native. Rhai runs *only* the user's binding bodies.

Unlike Perl, the trampoline is **safe pure-Rust**: marshal args → call the Rhai
`FnPtr` (`fnptr.call(&engine, &ast, args)`) → map any `EvalAltResult` to a clean
`Error:` → marshal the result. No unsafe boundary.

## 4. Architecture

```
 contrib script (foo.sty.rhai)  ──►  Rhai Engine (per conversion, retained)
                                       │  runs the script verbatim
                                       │  calls registered API fns: DefMacro, Token, ...
                                       ▼
   Def* shim fns (registered on the Engine)
                                       │  parse prototype (reuse Rust Parameters parser)
                                       │  wrap each Rhai FnPtr in a native Rust body closure
                                       ▼
   native Rust Definition installed into State (install_definition)
                                       │
   ... later, during conversion ...
                                       ▼
   gullet/stomach/document invoke the Rust body closure
                                       │  marshal args Rust→Dynamic
                                       │  fnptr.call(&engine, &ast, args)   [safe, Result]
                                       │  marshal result Dynamic→Rust
                                       ▼
   Rhai body runs; calls API fns ($document.open_element, Tokens(), ...)
                                       │  each resolves the active context (thread-local)
                                       ▼
   latexml-oxide Rust runtime (State, Gullet, Stomach, Document)
```

Components:
1. **Engine ownership** — one `rhai::Engine` + compiled `AST` retained for the
   conversion (they must outlive the stored `FnPtr`s that the definitions call).
2. **Marshaling** — tiered, like the Perl plan but over `rhai::Dynamic` instead
   of SVs: Tier-1 by value (string/int/bool, `Token`, `Tokens` — registered as
   Rhai custom types); Tier-2 opaque handles (`Whatsit`, `Box`, `Document`,
   `Gullet`, …) passed through as registered custom types, resolved via the
   active context, never deconstructed in script.
3. **API shim** — the binding API registered as Rhai functions/custom-types.
4. **Active-context** — a `#[thread_local]` holding the current
   document/gullet/whatsit pointers for the duration of a trampolined call, so
   `$document.*` script calls reach the live Rust objects. **Borrow discipline:**
   never hold a `state!()`/`&mut Document` borrow across a `fnptr.call` (same
   RefCell rule as the rest of digestion). Re-entrancy is tractable here because
   it's pure Rust — worst case a clean panic/`Result`, never UB.
5. **Loader + dispatch hook** — when package-load dispatch finds no native
   binding, and the feature is on, it consults the script loader (see §5).

## 5. Crate placement, feature gating, dispatch hook

- **Home: `latexml_contrib`** (this is literally the user-contributed-bindings
  crate; it sits atop core/engine/package and can reach the whole API). The new
  module is `latexml_contrib/src/script_bindings/` (now a directory: `{engine,wire,tests}.rs`), every line behind the
  feature. It is the only place Rhai enters the workspace.
- **Feature: `runtime-bindings`** on `latexml_contrib` (`= ["dep:rhai", "dep:log"]`);
  `rhai = { version = "1", optional = true }`. Propagated up via
  `latexml/runtime-bindings = ["latexml_contrib/runtime-bindings"]`. Core, engine,
  package untouched. *(Renamed from `script-bindings` — this plan's original name;
  the dead alias was dropped 2026-07-17, pre-publish. The Rust module keeps the
  `script_bindings` name.)*
- **Packaging: RESOLVED — on by default**, for end-user extensibility.
  `make_release.sh` builds `--no-default-features --features runtime-bindings`
  (drops `test-utils`, keeps this); `latexml`'s `default` includes it too. The
  cortex-worker image omits it — see `SAFETY.md` §H on untrusted input.
- **Dispatch hook (dependency-clean):** package/core dispatch cannot call *up*
  into `latexml_contrib`. So the script loader installs itself at startup via a
  registered function-pointer hook (`Option<fn(&str, &str) -> Option<...>>`)
  that dispatch consults on a binding miss. Contrib (or `latexml_oxide` main)
  installs the hook when the feature is on.

## 6. Sandbox & safety policy

- Register **only** the binding API on the Engine — no file/network/process
  builtins. Set Rhai limits: `max_operations`, `max_call_levels`,
  `max_string_size`, `max_array_size`, progress callback for cancellation.
- **Failure isolation:** a script parse/eval error, a limit breach, or a bad
  binding degrades **only that package** — it surfaces as a clean `Error:` and
  falls back to the normal undefined-binding path. It can never crash, hang, or
  corrupt a conversion that doesn't use it. This is the core promise of an
  optional add-on, and Rhai's pure-Rust safety makes it real.
- Consequence: running **untrusted** contrib scripts is acceptable (unlike the
  Perl ACE model). Document this in `SAFETY.md` when the feature lands.

## 7. Script discovery

- Convention: `<name>.<ext>.rhai` (e.g. `mypkg.sty.rhai`, `myclass.cls.rhai`).
- Resolution order (when a native binding is missing): `--contrib-dir <path>`
  CLI flag → `$LATEXML_CONTRIB_DIR` → a default user dir (e.g.
  `~/.latexml/contrib/`). This is reading *user-supplied* files from a
  user-named location — squarely in-scope (like reading `.sty` from texmf), not
  a read of latexml-oxide's *own* embedded resources.

> **Landed 2026-06-12 (simpler than the bespoke dirs above).** Discovery rides
> the *standard* file-search paths instead of a dedicated `--contrib-dir`/env
> var: the source document's directory (auto-prepended to `SEARCHPATHS` in
> `core_interface.rs::digest_file`) plus every `--path <dir>`. The hook is a
> single binding-resolution **priority chain** installed by
> `converter.rs::initialize_session` (one dispatcher, so call-site ordering
> can't reshuffle it):
>   1. `<request>.rhai` in the local search paths (`rhai_dispatch` →
>      `find_file(ext_type="rhai", search_paths_only=true)`, so **no kpsewhich**
>      probe per package load);
>   2. the embedder-supplied extra dispatcher (`latexml_contrib`);
>   3. `latexml_package` (core compiled bindings);
>   4. *(added 2026-07-22, #345)* `<request>.rhai` on the host **TeX tree** —
>      the same `rhai_dispatch` with `search_paths_only=false`, so kpsewhich
>      (and therefore `$TEXINPUTS`) is consulted.
>
> Because the local `.rhai` tier is checked **first**, a user-supplied
> `<name>.<ext>.rhai` *overrides any compiled binding of the same name* — e.g.
> `article.cls.rhai` shadows the built-in `article_cls`. (`latexml_package`
> and `latexml_contrib` are disjoint, so their relative order is immaterial.)
>
> The TeX-tree tier is **last** on purpose, and the two properties are
> deliberately different:
>   * *Cost.* Tier 1 runs on every package/class request (64 of them on a plain
>     acmart paper) and must stay a cheap local-path probe; a kpathsea miss is a
>     directory-tree walk, or a full fork-exec on the subprocess-`kpsewhich`
>     backend, and `pathname::kpsewhich`'s memo is keyed by candidate name, so
>     distinct package names never share a hit. Measured before the tiers were
>     split: +26 ms/conversion with linked libkpathsea, ~1.6 s with the
>     subprocess backend.
>   * *Authority.* A `.rhai` you put beside your document is an override; one
>     that merely happens to sit in a texmf tree only **fills a gap**. Left at
>     tier 1, a stray `amsmath.sty.rhai` on the TeX tree silently displaced the
>     whole compiled `amsmath` binding (`\begin{align}` → undefined).
> The most common Perl form — `DefMacro('\foo', 'bar')` with a **string** body
> (not a closure) — is now a registered Rhai overload, wiring the same native
> `ExpansionBody::Tokens` expandable as the compile-time `DefMacro!`.

## 8. API shim surface (prioritized)

Same prioritization as the Perl plan (driven by `.ltxml` corpus frequency),
registered as Rhai functions / custom types:

- **Datatype constructors:** `T_CS`, `T_OTHER`, `T_LETTER`, `Token`, `Tokens`,
  `Explode`, `Number`, `Dimension`.
- **Definition constructors:** `DefMacro`, `DefPrimitive`, `DefConstructor`
  (+`*I` forms), then `DefEnvironment`, `DefRegister`, `DefConditional`.
- **State/helpers:** `LookupValue`, `AssignValue`, `Let`, `ToString`, `Digest`,
  `Expand`, `Revert`.
- **`$document`/`$gullet`/`$stomach`/`$whatsit` methods** (Tier-2 custom types):
  `open_element`/`close_element`/`insert_element`/`absorb`; `read_token`/
  `read_arg`; `get_arg`/`get_property`/`set_property`.

Unimplemented API names should be registered to **error with telemetry**
(fail-safe + a data-driven worklist of what to add next), mirroring the Perl
plan's coverage-sweep idea.

> **Landed 2026-06-12 — test fixtures as `.rhai` (the first real coverage).**
> The local-`.ltxml` test fixtures (Perl `t/{keyval,keyval_options,structure}`)
> are now local `.rhai` next to their Rust `.tex`, replacing the compiled
> `latexml_contrib` stand-ins they had been ported to. The migrated fixtures —
> `xkvdop{1-6}`, `mykeyval`, `myxkeyval`, `apackage`, `filelistclass`,
> `lxtestclass` (the Perl `myclass.cls` fixture) — drove these surface additions:
> `InputDefinitions(name, #{type, noltxml, withoptions, handleoptions, …})`;
> `T_CS`; `DeclareOption(opt, "\tex")` (string body) and `DeclareOption(sub)` (the
> default/`undef` handler → `\default@ds`); `DefKeyVal(keyset, key, vtype,
> default, #{prefix, kind, choices, macroprefix})`; `Digest` now returns the
> digested handle (so `ToString(Digest(T_CS("\\CurrentOption")))` works); and
> `GetKeyVal` accepts a digested `KeyVals`/unit argument (not just a source
> string), plus `&GetKeyVal(#1,key)` is whitelisted in the runtime
> `replacement.rs` template path. The fixture tests live in
> `tests/{keyval_rhai,structure_rhai,daemon_rhai}` and the (whole-dir-fixture)
> `tests/keyval_options`, each `#![cfg(feature = "runtime-bindings")]`-gated so
> they are skipped when the feature is off. The harness reaches them through the
> *same* `install_binding_dispatch` chain a real conversion uses (DRY, in
> `converter.rs`).
>
> This now covers **every** Perl `t/*` local-`.ltxml` fixture. The full Perl set
> was 14 files: the 11 above, plus `testlocks`/`testlocks-b` (a daemon-mode
> `locked`-semantics test, newly ported to `tests/daemon_rhai` with
> `\usepackage{testlocks}` standing in for the Perl `.spec` preload — body
> identical, only the `class`/`package` PI order differs), and `any.sty`, which
> is **not** an executable binding at all: the `alignment/listing` test
> `\lstinputlisting{any.sty.ltxml}`s it as Perl source to typeset, so its
> `.ltxml` stays as listing *data* and needs no `.rhai`.

## 9. Graduation path

Because a script `DefConstructor(proto, |..| {..})` and a native
`def_constructor(...)` register the *same* `Definition`, a proven script binding
ports to native Rust largely mechanically: same prototype string, same API
calls, body rewritten from Rhai to Rust. The script tier is thus a staging lane
for the native `BINDINGS` table.

## 10. Milestones

- **M0 — spike — ✅ LANDED.** Prove the Rhai mechanics behind `script-bindings`:
  an `Engine` driven from Rust, a Rust fn (`DefMacro`) registered and called from
  a script, a **stored Rhai `FnPtr` called back later** from Rust (the deferred-
  expansion seam), result round-tripped. No latexml API yet. *Exit:* the seam
  works, pure-Rust, builds clean.
- **M1 — Tier-1 marshal + a real `DefMacro`.** `Token`/`Tokens` custom types;
  one script `DefMacro` expands end-to-end through the real expander.
- **M2 — `$document` + a string/`DefConstructor`** via the active-context.
- **M3 — coderef constructors + properties/afterDigest.**
- **M4 — dispatch hook + script discovery** → a `mypkg.sty.rhai` loads on a
  binding miss during a real conversion.
- **M5 — breadth + telemetry sweep + `SAFETY.md`/packaging.**

## 11. Risks & open questions

- **Authoring language.** Contrib authors write Rhai, not Rust. Mitigation:
  the graduation path (§9) + good examples; Rhai syntax is close to Rust/JS.
- **Marshaling depth.** Tier-2 opaque-handle pass-through (don't deconstruct
  Rust objects in script) keeps this bounded — same insight as the Perl plan.
- **Closure semantics.** Verify Rhai *capturing* closures (not just the
  non-capturing case the spike uses) survive deferred `FnPtr::call` with the
  retained engine+ast. (Spike validates the non-capturing case first.)
- **Re-entrancy/borrow discipline.** Active-context + "no borrow held across a
  `fnptr.call`." Pure-Rust, so failures are clean, not UB.
- **Performance.** Interpreted; scoped and accepted. If a *hot* binding lands on
  the script tier, that's the signal to graduate it (§9), not to optimize Rhai.
- **ROI reframed.** As an optional, isolated add-on the value is
  *contributor accessibility* (lower the barrier to add a binding), not fixing
  the failure tail — so it does not need the C1 demand gate that an always-on
  core dependency would.
- ~~**Binary size / default.** Confirm whether the official release ships with the
  feature on.~~ **RESOLVED** — it does: `runtime-bindings` is in `latexml_oxide`'s
  `default` features and `tools/make_release.sh` builds the distribution artifact
  with `--features runtime-bindings`. (This question was already answered earlier
  in this same file under "Packaging: RESOLVED — on by default".)

---

# Historical record (archived)

The M0 spike, the M1/M2–M4 progress log, the `\footnote`/DefEnvironment
landing notes, and the two dated critical re-evaluations are archived at
[`docs/archive/SCRIPT_BINDINGS_LOG_2026-06.md`](../archive/SCRIPT_BINDINGS_LOG_2026-06.md).
Net result carried forward: the mechanism is validated end-to-end (reliability
risk retired), re-entrancy + caching + error-boundary discipline hold, and the
surface below is what landed.

---

# The script API

The callable surface is not documented here. [`API.md`](../../latexml_contrib/src/script_bindings/API.md) lists every
call (generated from the engine), [`CHEATSHEET.md`](../../latexml_contrib/src/script_bindings/CHEATSHEET.md) sets each
beside its Perl spelling and its compile-time macro (generated, with the dialect's gaps), and
[`RHAI_BINDINGS_GUIDE.md`](RHAI_BINDINGS_GUIDE.md) teaches the dialect with the worked examples under
[`docs/examples/`](../examples/), which CI converts error-free (`30_script_bindings::docs_examples_convert_clean`).
This file keeps the design: why Rhai, the seam, discovery, the sandbox, failure containment, the shared lowering and
its cost. Every call lowers to the same core function as its macro (DG 2026-11-13); the Perl-facing rules (closure
forms, truthiness, `()` and `defined`, the deprecated aliases) are in the guide.

**Loading (Rust side):** `latexml_contrib::script_bindings::load_script(&str)` and `load_file(path)`; both return the
number of bindings installed. Compilation is cached by source (`SCRIPT_CACHE`).

**Failure containment (the `wire_*` seam).** A failure inside ANY script body is
reported as an ordinary `Error:` and the binding falls back to a neutral value —
it does not propagate an `Err` that would abort the conversion. This is what makes
the failure-isolation promise above real. All 14 trampolines in `wire.rs` route
through one helper, `contain`:

| binding kind | neutral value on failure |
|---|---|
| macro (both forms), column type, reversion | empty expansion |
| primitive, `beforeDigest`, `afterDigest` | no digested boxes |
| constructor, `afterConstruct`, rewrite-replace | no-op |
| `properties` | empty property map |
| conditional | false |
| sizer | zero-sized box |
| math-ligature matcher | no match |

In each case the binding contributes nothing and the surrounding document carries
on — the same shape as an undefined control sequence, which LaTeXML already
survives. Before this, a Rhai error anywhere in a body was mapped to a hard
`Error::from`, so ONE `throw` — or a typo'd method name, or a `max_operations`
breach — cost the whole document: measured, a single throwing `DefMacro` produced
an **empty** document.

Two things containment deliberately does NOT do:

* **It does not make a broken binding free.** `Error!` escalates to `Fatal` past
  `MAX_ERRORS`, and that `Err` still propagates — so a binding failing on every
  invocation ends the conversion, as it should.
* **It does not cover LOAD-time failures.** A script that fails to compile still
  returns `Err` from `load_script`; the package simply never installs and
  dispatch falls through to the normal undefined-binding path.

A body that fails PART-WAY may leave elements it opened unclosed; the document
builder's auto-close handles that, so the result is structurally odd rather than
malformed. Guards: `30_script_bindings.rs::a_throwing_script_body_degrades_only_its_binding`
(one throwing body of every kind, in one document) and
`script_bindings::tests::m1_errors_are_clean` (load-time `Err` vs run-time degrade).

**The full interface is GENERATED, not listed here.**
[`latexml_contrib/src/script_bindings/API.md`](../../latexml_contrib/src/script_bindings/API.md)
carries every registered function, grouped by handle, read back out of the live
engine via Rhai's reflections API (`Engine::gen_fn_signatures` — the mechanism the
Rhai book points at for documenting a host API). A test regenerates and compares
it on every run, so adding a `register_fn` without regenerating fails CI. It is
included into rustdoc as `latexml_contrib::script_bindings::interface`, so
`cargo doc` and docs.rs render it beside the Rust API. Regenerate with
`UPDATE_API_DOC=1 cargo test -p latexml_contrib --lib api_reference_is_up_to_date`.
Each row carries a one-line explanation plus a link to the Rust item that
implements the call, so the generated file answers both "what exists" and "what
does this do"; the generated `CHEATSHEET.md` beside it sets each call by its Perl spelling and its
macro; the prose in THIS document covers the design — why the surface is
shaped the way it is, and how failures are contained.

`gen_fn_signatures` needs Rhai's `metadata` feature, which pulls serde +
serde_json — so it is enabled on a **dev-dependency only**. Under
`resolver = "2"` dev-dependency features are not unified into a normal build;
verified with `cargo tree -e normal`, the shipped graph has
`rhai … no_module,no_time,std` and no serde at all.

**State API:** Perl's `AssignValue(key, val[, scope])` and `LookupValue(key)`; the snake_case shims
`assign_value`/`assign_global`/`lookup_value` and the typed `AssignString`/`AssignNumber`/`AssignBool` are deprecated
aliases (#716 §3).

**Sandbox:** `max_operations`, `max_call_levels`, `max_string_size` are bounded;
no file or network access is exposed; a process runs only through the `Command` builder. Errors (compile, `throw`, limit
breach, document op failure) surface as clean latexml `Error`s and degrade only
the offending binding.

# Shared lowering: `ConstructorBuilder` (anti-drift spine, 2026-06-09)

To keep the compile-time `DefConstructor!` macro and the runtime Rhai layer in
sync, both target one shared builder — `latexml_core::binding::def::builder::
ConstructorBuilder` (rhai-agnostic; takes native values/closures, so it lives in
core and pulls in neither the macro machinery nor Rhai).

- `ConstructorBuilder::new(proto)` parses the prototype (shared `parse_prototype`).
- **Scalar options** (`mode`, `bounded`, `requireMath`, `enterHorizontal`,
  `captureBody`, `alias`, …) go through one generic
  `set_option(key, OptionValue)` — the key→`ConstructorOptions`-field `match`
  lives in exactly **one place**, so a new scalar option updates both front-ends
  at once. Unknown keys are ignored (forgiving, like Perl `%options`).
- **Closure options** (`afterDigest`, …) use typed setters
  (`builder.after_digest(closure)`): the field + `install` are shared; the closure
  itself is produced per front-end (a macro `$body:block`, or a Rhai trampoline).
  The remaining closure options (`beforeDigest`/`properties`/`reversion`/`sizer`/
  `before+afterConstruct`) are the same shape.
- `install()` calls `def_constructor`.

The Rhai path (`wire_constructor`, `wire_constructor_template`,
`wire_constructor_opts`) now routes entirely through the builder; the macro can be
migrated arm-by-arm onto it (same shape) as a separate, low-risk change.

**Anti-drift conformance test** (`builder_conformance_macro_style_vs_rhai_afterdigest`):
the *same* `afterDigest` constructor is defined two ways — macro-style (calling
`ConstructorBuilder` directly, as `DefConstructor!` lowers) and via Rhai (which
routes through the builder) — and both produce identical behaviour. This is the
mechanical guard: evolve `setup_binding_language.rs` freely, and the test fails
the moment the Rhai layer falls behind. The same pattern extends to `MacroBuilder`/
`PrimitiveBuilder`/etc. as those front-ends are unified.

## Default-on cost (measured 2026-06-09)

`runtime-bindings` default-on adds **+3.36 MB (+7.3%)** to the release
binary (45.82 → 49.18 MB; rhai built with `no_module`/`no_time`, which
trimmed ~0.2 MB — the remainder is the core interpreter). Accepted as the
price of downstream customize-without-recompiling (user decision); the
maxperf/fat-LTO distribution build compresses the delta further. Opting out:
`--no-default-features` (the old distribution recipe) drops it.
