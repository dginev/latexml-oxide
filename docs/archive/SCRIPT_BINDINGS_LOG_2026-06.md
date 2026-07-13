# Script-bindings (Rhai) — historical progress log & re-evaluations

Archived from `docs/parity/script_bindings_plan.md` on 2026-06-09 (kept the live doc
lean). The plan's current state — surface reference, not-yet-covered list,
builder spine — lives in the main doc.

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

