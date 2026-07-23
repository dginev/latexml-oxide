# Runtime Script Bindings (Rhai) — Plan

> **Status:** ✅ **IMPLEMENTED and SHIPPED BY DEFAULT** (updated 2026-07-20).
> `runtime-bindings` is in `latexml_oxide`'s `default` features and in the
> distribution build (`tools/make_release.sh`); M0–M5 are all landed and the
> "Implemented script API (v0 reference)" section below is the live contract.
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
- `document.absorbString(s)` — insert literal text (ESCAPED — `<b>` becomes `&lt;b&gt;`).
- `document.insertXML(markup)` — **(#350)** parse an XML/(X)HTML string and splice
  the parsed SUBTREE in at the current point, as structured element/attribute/text
  nodes. The runtime half of Perl BookML's `\bmlRawHTML` idiom, which composes
  `XML::LibXML->parse_string` (`Common/XML/Parser.pm` `parseChunk`) with
  `$document->appendTree` (`Document.pm:2093`); backed by
  `common::xml::parse_fragment` (over `parse_chunk`, the `parseChunk` port) →
  `Document::insert_xml` → the existing `append_tree`. Namespaces resolve through
  the registry — see below.

  Overloads: `insertXML(markup)`, `insertXML(node)`, `insertXML(nodes)` — so a
  script may parse-and-insert in one call, or insert nodes it already holds.

- `ParseXML(markup) -> [Node]` — **(#350)** the parser as its own primitive, so a
  script can inspect or edit markup *before* it reaches the document. Returns an
  array because a chunk may be a fragment of several siblings (OXIDIZED_DESIGN
  #66). Each node OWNS the throwaway document it was parsed into, so unlike an
  in-tree node from a rewrite callback it is safe to hold, walk and mutate for as
  long as the script keeps it.

  **The markup must be WELL-FORMED, and parser recovery is OFF** (matching Perl's
  `parse_string`, which defaults to `recover => 0`). It need *not* be a single
  root — several siblings, or bare text, parse as a fragment (OXIDIZED_DESIGN
  #66) — but nothing is salvaged. That strictness is deliberate: libxml's recovery
  mode *silently destroys* author content — measured, `<b>a</b> <i>b</i>` salvaged
  to just `<b>a</b>`, `a&nbsp;b` to `ab`, `a & b` to `a  b`. Practical consequence
  for a binding author: write HTML entities in their numeric form (`&#160;`, not
  `&nbsp;`) unless the chunk declares them — `&nbsp;` is undefined in XML without
  a DTD.

  **On malformed input both degrade, they do not abort.** `insertXML` inserts
  nothing, `ParseXML` returns an empty array, and each raises a clean `Error:`
  naming the likely cause (and, for `insertXML`, quoting the snippet). Neither
  raises a *Rhai* error, which would propagate out of the constructor and kill the
  whole conversion — see the failure-isolation caveat below. Guard:
  `30_script_bindings.rs::malformed_insert_xml_degrades_the_binding_not_the_conversion`.

  > **Known gap (pre-existing, not #350).** That containment is hand-rolled in
  > these two functions; it is *not* the general behaviour. A Rhai error raised
  > anywhere in a script `DefConstructor` body — a bare `throw`, a limit breach,
  > a failing `setAttribute` — is mapped to a hard `Error::from` by
  > `wire.rs:90` and **aborts the conversion**, contradicting the
  > failure-isolation promise stated above (§"Failure isolation") and again in
  > §Sandbox. Verified by making a constructor `throw`: `convert_file` returns
  > `Err`. Closing it means containing the error at the `wire_*` seam for every
  > binding kind, which is its own change.
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

**`Node` — the XML-manipulation foundation.** One Rhai type serves every node,
whatever its provenance, so a capability added once works everywhere:

* a node *inside the conversion's tree*, handed to a `DefRewrite`/`DefMathLigature`
  callback — valid only for that body (it does not own the tree);
* a node *returned by `ParseXML`*, which **owns** its parsed document (libxml's
  `Document` is refcounted), and is therefore safe to keep, walk and mutate across
  statements. This is what makes handing parsed XML to an untrusted script sound
  at all: a bare libxml node is a pointer into its document, and one that outlives
  its owner is a dangling FFI pointer — a class of bug that has already cost this
  project SIGSEGVs, and which bypasses `catch_unwind`.

Nodes reached *from* a node (`parent`, `children`, `firstChild`, `nextSibling`,
`prevSibling`) inherit that provenance, so ownership propagates however deep a
script walks a parsed tree.

Current methods — read: `qname`, `name`, `content`, `getAttribute`,
`hasAttribute`, `toString` (serialize back to markup); write: `setAttribute`,
`removeAttribute`, `setContent`, `unlink`. **Extending is one `register_fn` over
`NodeProxy`** in `engine.rs` (see the "node traversal / editing" block) — there is
no parallel API to keep in step, which is the point of the single type.

`parent()` on a TOP-LEVEL `ParseXML` node is `()`. Everything above such a node is
an artifact of how the chunk was parsed — the throwaway `_lxfragment` wrapper a
multi-root chunk is parsed inside, or the parsed document node — never markup the
script wrote, and handing the wrapper back would let `insertXML(n.parent())`
splice `<_lxfragment>` into the page. In-tree nodes are unaffected. Guard:
`30_script_bindings.rs` `data-top="detached"` assertion.

**The Document XML surface (#350).** The runtime layer reaches the same
`Core/Document.pm` operations a compile-time `_sty.rs` binding calls, under their
Perl names, so the two do not diverge:

| Rhai | Perl / `Document` | notes |
|---|---|---|
| `findnodes(xpath[, node])` | `findnodes` | XPath query; array of Node |
| `findnode(xpath[, node])` | `findnode` | first match, or `()` |
| `getNode()` / `setNode(n)` / `getElement()` | `getNode`/`setNode`/`getElement` | the insertion point |
| `insertElement(qname[, content][, attrs])` | `insertElement` | open+absorb+close; returns the node |
| `openElementAt(n, qname[, attrs])` / `closeElementAt(n)` | `openElementAt`/`closeElementAt` | build at an explicit node |
| `addClass(n, c)` / `generateID(n, prefix)` | `addClass`/`generateID` | |
| `removeNode(n)` / `replaceNode(n, [n])` / `renameNode(n, name[, reinsert])` | `removeNode`/`replaceNode`/`renameNode` | |
| `unwrapNodes(n)` / `wrapNodes(qname, [n])` / `appendClone(n, [n])` | `unwrapNodes`/`wrapNodes`/`appendClone` | `wrapNodes` returns `()` when it cannot wrap |

Two rules hold across all of them, because a script is untrusted:

* **No panic reaches the conversion.** `Document::rename_node` `expect`s a parent,
  so `renameNode` on an ORPHAN — a node the script just removed — would abort the
  conversion, and the process under `panic = "abort"`. It is refused with a clean
  script error instead. Guard (red-checked): `xq:orphanmsg`.
* **A `ParseXML` node may only enter the document through `insertXML`.** Every
  node-splicing method refuses one. A parsed node still belongs to its throwaway
  parse document, whose libxml *dictionary* may own the node's name and content
  strings; moving it into our tree leaves those dangling when the script drops the
  handle. `insertXML` is safe because it RE-CREATES the subtree via `append_tree`.
  Guard (red-checked): `xq:foreignmsg`.

`getAttribute`/`hasAttribute`/`removeAttribute` resolve a PREFIXED name through
the namespace registry (`model::get_node_attribute`). libxml stores `xml:id` under
the built-in xml namespace and matches `xmlGetProp` on the plain name, so a
qualified lookup found nothing — a script could not read back the id `generateID`
had just written.

All of the above are pinned by
`30_script_bindings.rs::document_xml_api_matches_the_compile_time_surface`.

**Namespaces (URIs vs prefixes).** A binding declares its prefix↔URI mappings with
the same two helpers the Perl bindings use — `RegisterNamespace(prefix, uri)` (code
prefix) and `RegisterDocumentNamespace(prefix, uri)` (output-document prefix),
Perl `Package.pm:2049-2057` — both registered on the Rhai engine. Node re-creation
resolves a node's namespace **URI through that registry**, faithful to Perl
`Model::getNodeQName` → `getNamespacePrefix`; it does NOT read the raw libxml
prefix. (Rust splits that resolution into `model::get_foreign_node_qname`, called
only from `append_tree`, from the hot `get_node_qname` — a pure performance
factoring, since only a tree parsed *elsewhere* can present a foreign default
namespace, and reading the URI on the shared path cost a measured 3.4%.) That
distinction is load-bearing for `insertXML`: a snippet written the
natural way, `<p xmlns="http://www.w3.org/1999/xhtml">`, carries an *empty* libxml
prefix, and treating "empty prefix" as "the ltx namespace" would relabel it
`ltx:p` — stripping exactly what the XHTML post-processor keys on (`copy-foreign`
matches `xhtml:*`, `LaTeXML-common.xsl:358`) and silently dropping the raw HTML
from the final page. Wildcard schema entries participate too: `ltx:rawhtml`'s
content model is `xhtml:*`, so a concrete `xhtml:p` resolves its permissions
against that wildcard (Perl `canContain`/`canHaveAttribute`'s `$1.':*'` fallback),
which is what lets attributes such as `class` survive on absorbed markup.

**State API:** `assign_value(key, val)` (group-local), `assign_global(key, val)`,
`lookup_value(key) -> string`.

**Sandbox:** `max_operations`, `max_call_levels`, `max_string_size` are bounded;
no file/network/process access is exposed. Errors (compile, `throw`, limit
breach, document op failure) surface as clean latexml `Error`s and degrade only
the offending binding.

**Not yet covered** (truth as of 2026-06-09, post-residual pass):
deeper gullet access (ReadArg/ReadUntil/ReadOptional/SkipSpaces ARE
covered; DefRewrite's replace-closure — replace-by-reinsertion with
document context — and Node proxy read/write are covered too); structural `Token`/`Whatsit` marshaling (handles
cover `Tokens`/`Digested`); per-script key namespacing; sandboxed file-I/O
policy. Everything else in `setup_binding_language.rs`/`content.rs` is
covered — incl. (this pass) `sizer`, closure-form `reversion`, `DefAccent`,
read-only whatsit contexts in construction hooks, and default `.rhai` file
discovery.

## Binding-language surface — the 2026-06-09 "feature-complete" expansion

The Rhai surface now covers the working majority of
`setup_binding_language.rs` + `content.rs` under the **same names** (each
registration lowers to the same native function its macro does):

- **State**: `AssignValue(k,v[,scope])`, `LookupString/Number/Bool`,
  `lookup_value`, `assign_value`/`assign_global` (legacy snake_case kept).
- **Definitions**: `Let`, `XEquals`, `IsDefined`, `RawTeX`, `TeX`,
  `DefRegister` (int → count, "5pt" → dimen), `DefKeyVal` (3/4-arg),
  `DefLigature(regex, replacement)`, `DefMath(proto, presentation[, #{opts}])`
  with the full scalar option set, `DefConditional(proto, |args|->bool)`.
- **Tokens/boxes**: `Tokenize`, `TokenizeInternal`, `Expand`,
  `ExpandPartially`, `UnTeX`, `Digest`, `DigestText` → `Digested` handle,
  `ToString`/`ToAttribute`/`Revert` on handles, `Today`.
- **Counters**: `NewCounter(c[,within])`, `StepCounter`, `ResetCounter`,
  `AddToCounter`, `CounterValue`, `RefStepCounter` → map with live `Digested`
  values (returnable directly from a `properties` closure — the amsmath idiom).
- **Package/class**: `RequirePackage(name[,opts])`, `LoadClass(name[,opts])`,
  `DeclareOption`, `ProcessOptions([inorder])`, `ExecuteOptions`,
  `PassOptions`, `RequireResource`, `Tag(name, #{autoOpen, autoClose})`,
  `MergeFont(#{family,…})`, `Warn`/`Error` (with MAX_ERRORS escalation).
- **Option bags everywhere**: `DefMacro`/`DefPrimitive`/`DefMath` take a
  trailing `#{…}` (scope/locked/protected/robust/… via per-struct mappers);
  constructors/environments add `afterDigestBegin`, `beforeDigestEnd`,
  `afterDigestBody`, `before/afterConstruct` (document context published; whatsit
  TBD), string-form `reversion`, and `font: #{family: …}` directives.
- **Digest-hook option parity (2026-07-21):** the `beforeDigest`/`afterDigest`
  **closures** now work in the `DefPrimitive`/`DefMath` option bag (they were
  scalar-only, silently dropping the keys), and `afterDigestBody` was added to the
  constructor/environment bag — closing the gaps against the compile-time
  `Def*!` macros. These are structural, not just tested: the primitive/math
  mappers populate the SAME `PrimitiveOptions`/`MathPrimitiveOptions`
  `before_digest`/`after_digest` fields the macro's `defi_opts!` sets, and
  `afterDigestBody` routes through a new shared `ConstructorBuilder` setter — so
  the two front-ends cannot drift. Guards: `primitive_option_bag_runs_before_and_after_digest`,
  `math_option_bag_runs_digest_hooks`, `30_script_bindings::option_bag_digest_hooks_end_to_end`.

**Load semantics fixed (load-bearing):** `load_script` now caches only the
COMPILATION; the script RUNS on every load and each `Def…`/side-effect call
installs immediately, in script order — exactly Perl `.ltxml` semantics. (The
old run-once-then-rewire model both broke `DeclareOption` → `ProcessOptions()`
ordering and silently dropped `RawTeX`/`Let`/`NewCounter` effects on every
conversion after the first.)

**`LookupDefinition` + hook-push (#321, 2026-07-21):** `LookupDefinition(cs)`
returns a `Definition` proxy (or `()` for an undefined CS, mirroring Perl
`undef`); the proxy exposes `push<H>`/`unshift<H>` for **every** before/after
hook family — the Rhai analog of Perl BookML's `push(@{ $$def{afterConstruct}
}, sub{…})`. `push*` appends (Perl `push`), `unshift*` prepends (Perl
`unshift`). `before/afterDigest` apply to any DefConstructor/DefPrimitive/DefMath;
`before/afterConstruct` and `afterDigestBody` are DefConstructor-only (a
construct/body push onto a Primitive/MathPrimitive/Macro is a clear script
error — MathPrimitive's construct fields are never executed). Installed defs
have no interior mutability, so each push clones the current front definition,
splices the trampolined hook, and re-installs at **`Scope::InPlace`** (Perl
`State.pm:175` `'inplace'`, "same level"): the front binding is replaced without
touching the save stack, so the patch is the active meaning immediately
(sequential pushes accumulate by re-looking-up) and rides exactly as long as the
definition's existing binding — faithful to Perl mutating the shared def-hash in
place. (This corrects an earlier `Scope::Global` re-install that promoted a
locally-bound def to global — harmless for BookML, which only patches
already-global defs, but a real divergence flagged in PR #333 r3623947537
by @xworld21; the `state::reentrancy_tests::inplace_scope_keeps_the_bindings_level`
test pins the same-level semantics.) Hook arities match
the option-bag form (digest = parameterless via `whatsit()`; construct =
`|document|`). Guards: `script_bindings::tests::lookup_definition_*` and the
end-to-end `30_script_bindings::lookup_definition_pushes_construct_hooks_end_to_end`.

**Challenging-specimen e2e corpus** (all green through real conversions,
`30_script_bindings.rs`): plain `\footnote{}{}` (full hook set), ieeetran
`{IEEEproof}`-style (properties closure that DIGESTS its title + `#font` from
a `Digested`), amsmath-style `\numbered` (RefStepCounter properties + `#tags`
+ string reversion), natbib-style `\rcite OptionalMatch:* [][] Semiverbatim`,
graphics `\Gscale@box`-style `{Float}{Float}` → Transformable attributes,
listings-style `OptionalKeyVals:RH`, cas-dc `{bio}{}`, `{quote}`, plus
`\usepackage[draft]` exercising DeclareOption+ProcessOptions end-to-end.

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

## Default-on cost (measured 2026-06-09)

`runtime-bindings` default-on adds **+3.36 MB (+7.3%)** to the release
binary (45.82 → 49.18 MB; rhai built with `no_module`/`no_time`, which
trimmed ~0.2 MB — the remainder is the core interpreter). Accepted as the
price of downstream customize-without-recompiling (user decision); the
maxperf/fat-LTO distribution build compresses the delta further. Opting out:
`--no-default-features` (the old distribution recipe) drops it.
