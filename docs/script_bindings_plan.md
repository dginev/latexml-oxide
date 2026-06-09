# Runtime Script Bindings (Rhai) — Plan

> **Status:** proposal / planning (2026-06-08). Spike in progress.
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
  module is `latexml_contrib/src/script_bindings.rs`, every line behind the
  feature. It is the only place Rhai enters the workspace.
- **Feature: `script-bindings`** on `latexml_contrib` (`= ["dep:rhai"]`, off by
  default); `rhai = { version = "1", optional = true }`. Propagated up via
  `latexml_oxide/script-bindings = ["latexml_contrib/script-bindings"]`. Core,
  engine, package untouched. `--no-default-features`/lean builds drop it.
- **Packaging:** because the value is end-user extensibility, the official
  GitHub-release binary should ship with `script-bindings` **on** (it's pure
  Rust, modest size); minimal/embedded builds drop it. (Open: confirm default.)
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

## 9. Graduation path

Because a script `DefConstructor(proto, |..| {..})` and a native
`def_constructor(...)` register the *same* `Definition`, a proven script binding
ports to native Rust largely mechanically: same prototype string, same API
calls, body rewritten from Rhai to Rust. The script tier is thus a staging lane
for the native `BINDINGS` table.

## 10. Milestones

- **M0 — spike (in progress).** Prove the Rhai mechanics behind `script-bindings`:
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
- **Binary size / default.** Confirm whether the official release ships with the
  feature on.

---

# Post-M0 critical re-evaluation (2026-06-08)

The M0 spike (`latexml_contrib/src/script_bindings.rs`, `m0_self_test`) is
**green**. What it actually settles, and what it leaves wide open:

## What the spike decisively validated

- **The reliability objection is gone.** Rhai compiled cleanly through the entire
  workspace (3 min cold, ~27 s warm), pure Rust, zero native deps, no bindgen, no
  ABI, no `_Generic`. The single failure class that killed the libperl attempt
  **cannot occur here.** This was the one genuinely uncertain question after the
  Perl debacle, and it is answered: yes, decisively.
- **The seam works end to end** — Rust→Rhai eval, Rhai→Rust registered fn
  (`DefMacro`), and the load-bearing one: a **stored Rhai `FnPtr` called back
  later from Rust** (deferred-expansion seam). Errors are ordinary `Result`s.
- **Isolation is clean** — off by default, `rhai` absent from the dep graph when
  off, core/engine/package untouched.

## What the spike revealed (new, concrete)

1. **Backslash escaping is a real authoring wrinkle.** Rhai string literals
   process escapes, so a TeX control sequence is written `"\\dbl"`, not `"\dbl"`
   (the spike's first run failed exactly here). TeX bindings are backslash-dense
   (`\def`, `\section`, …) — this will bite every author. **Mitigations to
   evaluate in M1:** Rhai literal/raw string syntax if available; and/or API
   helpers that take the *name* without the backslash (`T_CS("section")`), so
   authors rarely type a literal `\`.
2. **Deferred `FnPtr::call` requires retaining `(Engine, AST)` for the whole
   conversion.** `fnptr.call(&engine, &ast, …)` means every native definition
   that wraps a Rhai body must hold a shared handle (`Rc<Engine>`, `Rc<AST>`,
   `FnPtr`) alive until the conversion ends. The `FnPtr` is bound to the `AST`
   it was compiled in, so multiple contrib files = multiple `AST`s (one shared
   `Engine` is fine). This is more lifecycle state than the plan implied — M1
   must nail the per-conversion ownership model (and AST caching across the
   thousands of canvas papers, without cross-conversion state leakage given the
   `#[thread_local]` State).

## What remains unproven — i.e. the hard 80% is untouched

The spike validated the **language embedding** (the easy 20%). It proves
**nothing** about the part that was always the real cost:

- **The API shim.** M0 used *stub* `DefMacro`/`Tokens`. Real marshaling of
  `Token`/`Tokens`/`ArgWrap`/`Whatsit`/`Digested` between `rhai::Dynamic` (custom
  types) and the runtime, the active-context for `$document` ops — all ahead and
  unvalidated. This is identical in size to the Perl plan's shim; only the
  *boundary safety* got easier (safe Rust, not FFI).
- **Re-entrancy (the GATE-1 equivalent).** A script `DefConstructor` body calling
  `$document.absorb(...)` that triggers nested digestion of *another* script
  binding, while a `&mut Document`/`state!()` borrow is live. Pure Rust makes the
  failure *clean* (panic/`Result`, not UB) — a real improvement — but a
  double-borrow still aborts the conversion. Needs the same adversarial test
  before any constructor breadth.
- **Real-binding ergonomics.** Whether a `\lx@superscript`-class definition (four
  closures, whatsit introspection, reversion) is *pleasant* to author in
  dynamically-typed Rhai is unknown. An M2/M3 specimen decides it.
- **ROI / adoption.** Even reliable and isolated, the value depends on a
  population that wants to add bindings, won't use a Rust toolchain, and will
  learn Rhai. That intersection is narrower than "all contributors." This is a
  lightly-evidenced product bet — honest framing, not a blocker for an optional
  add-on, but not validated either.

## Verdict & revised next step

M0 did its job: it **retired the reliability risk** that sank the previous
attempt, cheaply and conclusively. But it validated the easy part. **Do not jump
to breadth.** The right next gate is a **single thin vertical slice (M1):** take
*one real `DefMacro`* end-to-end through the actual expander — real `Token`/
`Tokens` custom-type marshaling, the retained-`(Engine,AST)` lifecycle, real
error mapping — and judge the shim's ergonomics and the lifecycle model on
something real. Pair it with the **re-entrancy adversarial test** before any
`DefConstructor` work. If M1's authoring experience is good and the lifecycle
holds, proceed; if the shim proves awkward, that's the signal to reconsider scope
*before* sinking effort into breadth.

The mechanism is green-lit. The open questions are now **shim ergonomics** and
**adoption ROI** — both illuminated cheaply by M1, neither by more M0-style work.

---

# Progress log

## M1 — macro seam: VALIDATED (2026-06-08)

`latexml_contrib/src/script_bindings.rs` — `load_script(src)` compiles a Rhai
binding, runs it to collect registrations, wraps engine+AST in `Rc`, and installs
a native definition per registration. Three unit tests green
(`cargo test -p latexml_contrib --features script-bindings`):

- a script `DefMacro` expands end-to-end through the **real gullet**
  (`\twice{ab}`→`abab`, `\greet{World}`→`Hello, World!`);
- expansion to a control sequence is faithful (`\emphx{hi}`→`\textit{hi}`, a real
  CS token — via `mouth::tokenize_internal` re-tokenization, not letters);
- compile errors and a body `throw` both surface as clean latexml `Error`s.

Findings confirmed: the retained-`(Rc<Engine>, Rc<AST>)` lifecycle works; bodies
receive args as TeX-source strings (`ArgWrap::to_string`); `parse_prototype(.., true)`
needs the base parameter-type registry (bootstrap `latexml_engine::base::load_definitions`
in tests; present in any real conversion).

## M2/M4 — constructor seam + dispatch: in validation

`DefConstructor` wired to native `def_constructor`. Bodies build XML imperatively
via `el_open`/`el_close`/`arg`, reached through a thread-local **active-context**
stack that publishes the live `&mut Document` + digested args for the call (raw
pointers copied out before each Document op, so the `CTOR_CTX` borrow is never
held across a re-entrant call). Compiles; macro tests still green. End-to-end
conversion test (`latexml_oxide/tests/30_script_bindings.rs`) loads a sample
binding via the extra dispatcher on `\usepackage{lxrhaitest}` and asserts the XML
— running.

## M2–M4 + maturation — FULL MECHANISM VALIDATED (2026-06-08)

All four binding dialects now work **end-to-end through a real conversion**
(`latexml_oxide/tests/30_script_bindings.rs`, green): a sample Rhai binding is
loaded at runtime via the *extra* dispatcher on `\usepackage{lxrhaitest}` and the
produced XML is asserted.

- **DefMacro** (expandable) — `\twicex{ab}`→`abab`.
- **DefConstructor, imperative** — `\myemph{hi}`→`<emph>hi</emph>`, body builds XML
  via the active-context document API (`el_open`/`el_close`/`arg`).
- **DefConstructor, template** — `\mytext{zz}`→`<text class="rhai">zz</text>`, run by
  a pure-Rust runtime template interpreter (`apply_template`) mirroring the
  compile-time compiler's Document calls. No Rhai per invocation.
- **DefPrimitive** — `\setx{hello}` performs a digestion-time `assign_value`
  side-effect, verified by reading State after the conversion.

Maturation landed:
- **Re-entrancy (GATE-1) validated**: `\wrap{\myemph{deep}}` makes one script
  constructor's body trigger another's construction while the first's
  active-context is live — `<emph>deep</emph>` is produced, no borrow panic. The
  active-context **stack** + "copy raw ptrs out before each Document op" borrow
  discipline hold.
- **AST cache**: compile+run happens once per unique script source
  (`SCRIPT_CACHE`); re-wiring into each conversion's State is cheap. Matters for
  canvas (same contrib package across many papers).
- **State API**: `assign_value`/`lookup_value` exposed to scripts.
- **Boundary safety**: every body call maps `EvalAltResult`→latexml `Error`; every
  document XSUB copies the active-context out before the call (no borrow held
  across re-entry). Compile/throw both surface as clean `Error`s.
- Feature isolation re-confirmed: `rhai` absent with the feature off; core/engine
  /package untouched.

Test status: `latexml_contrib` unit suite 4/4 (macro ×2, cache, errors);
integration 1/1 (all four dialects + re-entrancy + primitive side-effect).

## Complex-binding surface — `\footnote` port GREEN (2026-06-09)

With the shared `ReplacementOp` AST (#171) as the runtime template engine, the
richest real binding shape now runs from Rhai end-to-end. New surface, each
mirroring its Perl idiom 1:1:

- **`properties` option**, both Perl shapes: static map and closure (digested
  args in as TeX-source strings, property map out). Routed through the new
  `ConstructorBuilder::properties` typed setter (same anti-drift spine as
  `after_digest`).
- **`whatsit().setProperty(key, val)` / `propertyString(key)`** for hook bodies.
- **`beforeDigest` option** (parameterless closure trampoline) and the
  `neutralize_font()` pool helper registered under its native name — completing
  the `\footnote` option set.
- e2e specimens in `30_script_bindings.rs`: a **fully 1:1** port of plain TeX's
  `\footnote{}{}` (its `^` float prefix, `?#mark(mark="#mark")()` conditional
  attribute, `?#prenote(…)()` content conditional, `mode`,
  `beforeDigest: || neutralize_font()`, and the afterDigest mark routing), a
  `properties`-closure constructor, a static-map constructor, and a `<?pi…?>`
  template — all asserted on the produced XML, including the negative case
  (empty mark ⇒ no `mark=` attribute).

## DefEnvironment runtime front-end — GREEN (2026-06-09)

`DefEnvironment` joins the script surface, same four shapes as `DefConstructor`
(template/closure × bare/option-bag), via a new core `EnvironmentBuilder`
(prototype parsed exactly as `DefEnvironmentWO!`: braced name + parameters
against a synthetic `\name`). The scalar-option key→field map is now a shared
free function (`apply_scalar_option`) used by both builders, and the contrib
option-bag loop is generic over a local `BindingBuilder` trait — one
`apply_opts` serves constructors and environments. New proxy:
`document.absorbProperty("body")`, the imperative analog of a template's
`#body` hole (mirrors natives like `{center}`'s `sub[document, _args, props]`).

e2e specimens (all green through a real conversion):
- `{rquote}` — 1:1 port of latex_base's `{quote}` (`#body` + `mode`).
- `{bio}{}` — 1:1 port of the cas-dc contrib class's biography environment.
- `{biop}{}` — env arg → `properties` closure → `#prop` hole at attribute
  position (the Perl-idiomatic route, asserted `class="Ada"`).
- `{rbox}` — imperative body using `absorbProperty("body")`.

Two **faithful-semantics findings** pinned by these specimens, each verified
identical across Perl LaTeXML (direct `.sty.ltxml` probes), Rust native, and
the Rhai runtime:
1. An environment's `#n` at **attribute** position renders **empty** (the
   begin's args don't interpolate into attributes; Perl consumes the arg and
   emits no attribute). The cas-dc `name='#1'` is dead weight in the original
   too. The working idiom is `properties` + `#prop` (the `{biop}` specimen).
2. **Schema-disallowed attributes are silently dropped** by both Perl and Rust
   `Document` (e.g. `ltx:note` has no `@name`; a literal `name='LIT'` probe is
   dropped by both). Pick schema-allowed attributes (`class` is universal).

---

# Post-PoC critical re-evaluation (2026-06-08)

The mechanism is proven and reliable, and the four-dialect skeleton works
end-to-end. But a working skeleton is not full coverage. Honest assessment:

## Proven (strong)
- **Reliability** — pure-Rust, no FFI/ABI, builds clean, feature-isolated. The
  failure class that killed the Perl attempt is structurally absent.
- **All four seams** (macro, imperative + template constructor, primitive) run
  through the *real* conversion, including the dispatch/`\usepackage` path.
- **Re-entrancy (GATE-1)** holds; **AST caching** gives a per-source compile.

## Gaps that bound real-world coverage (the honest 80%)
1. **Marshaling is string-based and lossy.** Macro/primitive args arrive as
   TeX-source strings and macro bodies return strings (re-tokenized). Structure
   (catcodes, digested boxes, token identity) is flattened. Faithful for simple
   text/CS expansions; inadequate for bindings that inspect token/box structure.
   *Highest-value fix:* `Token`/`Tokens`/`Number` as Rhai custom types (Tier-1),
   opaque handles for `Whatsit`/`Box` (Tier-2).
2. **Template interpreter is a subset.** No conditionals `?t(..)(..)`, no `#prop`
   / `#body` interpolation, no PIs, no float/font/`^`-float attributes. The
   compile-time compiler (`constructable.rs`) is far richer. A real fraction of
   constructors won't run yet.
3. **Thin body API.** Bodies cannot reach the gullet (`readToken`/`readArg`) or
   most document ops; constructors have only `el_open`/`el_close`/`arg` (no
   imperative attributes, `insert_text`, whatsit introspection,
   `properties`/`afterDigest`/`reversion`/`sizer`). The complex-constructor
   class (e.g. `\lx@superscript`) is out of reach. This is the "API shim is 80%"
   reality — perhaps ~15% built.
4. **Missing dialects.** `DefEnvironment`, `DefMath`, `DefRegister`,
   `DefConditional`, `DefKeyVal` are absent. Environments and math are common.
5. **State API is blunt.** `assign_value` is hard-coded `Global` (real
   assignments are group-local by default) and accepts *any* key — an untrusted
   script could clobber internal State. *Fix:* expose scope; namespace-guard or
   restrict keys for the untrusted-script promise to hold.
6. **Performance unmeasured.** Per-invocation Rhai-call + marshaling overhead is
   not benchmarked; "graduate hot bindings to native" is policy, not data.
7. **Unbounded source cache** (minor; contrib scripts are few).
8. **Adoption/ROI** unchanged — value hinges on contributors willing to author
   Rhai.

## Verdict
PoC: **success**. Production coverage: a real, *bounded* build-out, dominated (as
predicted) by the API shim. Priority order for maturation by value:
(a) richer marshaling (custom types) → unlocks fidelity;
(b) template completeness (conditionals/`#prop`/`#body`);
(c) `DefEnvironment`;
(d) scope-correct + namespace-guarded State;
(e) benchmarking, then graduate any hot path.
None of these is a research risk; they are incremental shim growth on a validated
spine.

---

# Implemented script API (v0 reference)

The surface a `.rhai` binding may call today (see `docs/examples/sample.sty.rhai`).
Backslashes in TeX control sequences must be doubled in Rhai strings (`"\\foo"`).

**Loading (Rust side):** `latexml_contrib::script_bindings::load_script(&str)` and
`load_file(path)`; both return the number of bindings installed. Compilation is
cached by source (`SCRIPT_CACHE`).

**Registration (script side):**
- `DefMacro("\\cs{}", |args…| -> string)` — expandable; body returns TeX source
  (faithfully re-tokenized). Args arrive as TeX-source strings.
- `DefPrimitive("\\cs{}", |args…| { … })` — digestion-time side-effects.
- `DefConstructor("\\cs{}", "<ltx:tag a=\"#1\">#2</ltx:tag>")` — template form.
  Since #171 landed, the template is parsed once into the shared `ReplacementOp`
  AST (`latexml_core::binding::def::replacement`) — the *same* parser the
  compile-time `DefConstructor!` macro uses — so the **full dialect** is
  supported at runtime: elements, nesting, self-close, `#1`..`#9` and `#prop`
  holes at content + attribute position, `?test(then)(else)` conditionals (top
  level, attribute-pair, and attribute-value), `^`/`^^` float prefixes, `<?pi…?>`
  processing instructions, literal text.
- `DefConstructor("\\cs{}", |document, arg1, …| { … })` — imperative form. The
  body gets a **`document` proxy** as its first argument (Perl's `$_[0]`) and each
  digested argument as an opaque handle — so it reads like the Perl original.
- `DefConstructor("\\cs{}", replacement, #{ mode: …, afterDigest: |…| {…} })` —
  **option-bag form**. A trailing Rhai object map is the analog of Perl's
  `%options` / the `DefConstructor!` macro's `key => value`: named, any order,
  omittable; values may be strings *or* closures. `parse_ctor_options` maps each
  key onto native `ConstructorOptions` — a *value* option sets a field, a
  *closure* option pushes a trampoline. Wired so far: the scalar options routed
  through `ConstructorBuilder::set_option` (`mode`, `bounded`, `requireMath`,
  `forbidMath`, `enterHorizontal`, `leaveHorizontal`, `captureBody`, `alias`),
  plus the closure options `afterDigest`, `beforeDigest` (parameterless, for
  state/font side-effects like `neutralize_font()`), and `properties` — the
  latter in **both** Perl shapes: a static map (`properties: #{ k: "v" }`) and a
  closure (`properties: |arg1, …| #{ k: … }`, receiving each digested arg as its
  TeX-source string, returning the whatsit's property map). The rest
  (`reversion`, `sizer`, `before/afterConstruct`, …) are one-line additions of
  the same two shapes.

**`whatsit` proxy (inside `afterDigest`-style hook bodies):**
- `whatsit().argString(n)` — the n-th (1-based) digested argument's TeX source.
- `whatsit().setProperty(key, val)` — set a string property (Perl
  `$whatsit->setProperty`); read by the template's `#key` holes, e.g. the
  plain-`\footnote` port's afterDigest routing its mark arg to `mark`.
- `whatsit().propertyString(key)` — read a property back ("" when absent).

- `DefEnvironment("{name}{}…", replacement[, #{ options }])` — environments,
  same four shapes as `DefConstructor`; the template typically references
  `#body`. Prototype is the `DefEnvironment!` form: braced name, then the
  parameter list. Routed through the core `EnvironmentBuilder` (the environment
  analog of `ConstructorBuilder`, sharing the same option machinery).

**`document` proxy methods (inside an imperative constructor body):**
- `document.openElement(tag)`, `document.closeElement(tag)`,
  `document.maybeCloseElement(tag)`.
- `document.setAttribute(key, val)` — attribute on the current node.
- `document.absorbString(s)` — insert literal text.
- `document.absorb(arg)` — absorb a digested argument handle (`arg1`, …).
- `document.absorbProperty(name)` — absorb a whatsit property at the current
  point (the imperative analog of a template's `#name` hole; `"body"` inside an
  imperative `DefEnvironment`).

This proxy is the **extension point for the full prelude**: each additional
`$document->method` is a one-line registration (the `doc_qname_method!` mini-DSL
covers the common `(qname)` side-effect shape); `gullet`/`stomach`/`whatsit`
proxies follow the same mold. The doc example translates verbatim:
`DefConstructor('\endreferences', sub { $_[0]->maybeCloseElement('ltx:biblist');
$_[0]->maybeCloseElement('ltx:bibliography'); })` →
`DefConstructor("\\endreferences", |document| {
document.maybeCloseElement("ltx:biblist");
document.maybeCloseElement("ltx:bibliography"); })`.

**State API:** `assign_value(key, val)` (group-local), `assign_global(key, val)`,
`lookup_value(key) -> string`.

**Sandbox:** `max_operations`, `max_call_levels`, `max_string_size` are bounded;
no file/network/process access is exposed. Errors (compile, `throw`, limit
breach, document op failure) surface as clean latexml `Error`s and degrade only
the offending binding.

**Not yet covered** (see the critical re-eval): structural arg/return marshaling
(`Token`/`Tokens`/`Whatsit` as types rather than strings); gullet access from
bodies; constructor `reversion`/`sizer`; `DefMath`/`DefRegister`/
`DefConditional`; configurable assignment scope per-call + key namespacing for
untrusted scripts.
(Template conditionals/`#prop`/floats/PIs, constructor `properties`/
`afterDigest`/`beforeDigest` + the `neutralize_font()` pool helper, and
`DefEnvironment` with `#body` — template and imperative — are covered as of
2026-06-09; see above.)

---

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
