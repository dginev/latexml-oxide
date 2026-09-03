# Oxidized Design: Rust Port Design Decisions

Design decisions, architecture, and **intentional divergences** from Perl
[LaTeXML](https://github.com/brucemiller/latexml) in the Rust port
**latexml-oxide** — for public readers evaluating the project and contributors
resuming work.

This file is the **index + overview**. The detail lives in a themed family:

| Theme file | What it holds |
|---|---|
| [OXIDIZED_DESIGN_DIVERGENCES.md](OXIDIZED_DESIGN_DIVERGENCES.md) | The numbered **Intentional Divergences from Perl** (`#1–#15`, `#17–#18`, `#19–#190`) + a brief Known-Upstream-Perl-Issues list. Code comments cite these as `OXIDIZED_DESIGN #N`. |
| [OXIDIZED_DESIGN_MATH.md](../math/OXIDIZED_DESIGN_MATH.md) | Marpa math-parser design: `#16` design rules + the grammar-rule cluster `#7–#18`. |
| [OXIDIZED_DESIGN_TYPES.md](OXIDIZED_DESIGN_TYPES.md) | Type-system improvements (behavior-neutral) + tactical internal pitfalls. |
| [OXIDIZED_DESIGN_FUTURE_WORK.md](OXIDIZED_DESIGN_FUTURE_WORK.md) | Beyond-parity directions not yet built. |

> **Finding a divergence by number.** The `### N` numbers are load-bearing —
> `.rs` comments reference them — so they are kept verbatim, which means three
> quirks: (1) the numbers are **not globally unique** (the math cluster `#7–#18`
> collides by value with divergences `#7–#18`); (2) a code-referenced number
> resolves to the file above that owns that *topic* — most (`#1–#15`, `#19–#190`)
> are in DIVERGENCES, the math ones (incl. the code-referenced **`#18` = f(x)
> "Speculative function application"**) are in MATH; (3) **`#76` is a retired
> number** — its entry was consolidated into `#74` and the number was not
> reused, so a gap in the sequence is expected, not a missing file. Next free
> number: **#191**. When in doubt,
> `grep '### N\.' docs/parity/OXIDIZED_DESIGN_*.md docs/math/OXIDIZED_DESIGN_MATH.md`.

---

## Guiding Principles

- **Faithfulness first.** Aim for behavioral parity with Perl; follow its
  organization, abstractions, and naming. Diverge only when Rust's type system
  enables a meaningfully better representation, or the Perl behavior is a known
  bug — and record it in [OXIDIZED_DESIGN_DIVERGENCES.md](OXIDIZED_DESIGN_DIVERGENCES.md).
- **Meaningful types for untyped Perl.** Replace strings / arrayrefs / blessed
  hashrefs with enums, structs, and newtypes that make invalid states
  unrepresentable — without changing observable behavior.
- **Test parity as the north star.** The Perl test suite (`.t` with `.tex`/`.xml`
  pairs) is ground truth; every passing Perl test is a target.
- **Curated binding layer.** The `DefMacro!`/`DefPrimitive!`/`DefConstructor!`/
  `DefEnvironment!` macro system and the Rust types are deliberately curated —
  follow their patterns and abstraction levels; extend traits (or add new ones)
  when the existing shapes don't fit.
- **Self-contained, portable binary.** A conversion must not *read* latexml_oxide's
  *own* auxiliary resources from disk during its run: engine dumps, the compiled
  RelaxNG model, XSLT (+ `xsl:import` chains), and post-processor CSS/JS are all
  embedded at compile time (`include_bytes!`/`include_str!`) and served from
  memory. *Writing* outputs into the destination directory is fine. **Out of
  scope: the host TeX ecosystem** — reading `.sty`/`.cls`/`.tfm` from the user's
  texmf tree via `kpathsea` is allowed and expected (those are the user's files,
  as for `pdflatex`). Litmus test: a release binary in an empty dir on a
  TeX-Live machine that has never seen the LaTeXML tree must convert using only
  the input + the TeX ecosystem. **Status (2026-05-23): met for all owned
  assets, verified end-to-end** — XSLT/CSS/JS via `embed:///` input callbacks
  (`libxml` ≥ 0.3.12, zero `.xsl` disk reads under `strace`); dumps via
  `include_str!` (byte-identical output with the on-disk `resources/dumps/`
  renamed away). An on-disk dump is still *preferred* when present, as a dev
  override — see [DUMP_DESIGN.md](DUMP_DESIGN.md).

---

## Architecture

**Pipeline.** Perl LaTeXML is two programs, `latexml` (TeX→XML) and `latexmlpost`
(XML→HTML/MathML). The Rust port covers `latexml` in five stages: **Digestion**
(Mouth chars→tokens, Gullet expansion, Stomach → boxes/whatsits) → **Construction**
(→ XML DOM via Constructors, auto-open/close from the Model) → **Rewriting** (DOM
rules: ligatures, math-token declarations) → **Math Parsing** (grammar parse of
flat XMath) → **Serialization**. (The post-processing pipeline lives in
`latexml_post`.)

**Workspace** — 8 crates mirror and specialize the Perl hierarchy:

| Crate | Perl equivalent | Role |
|-------|----------------|------|
| `latexml_core` | `LaTeXML::Core::*` | Mouth, Gullet, Stomach, document builder, state |
| `latexml_engine` | `LaTeXML::Engine::*` | Pool ports (`tex.rs`/`latex.rs`, primitives, constructs) |
| `latexml_package` | `LaTeXML::Package` | Package/class definitions, binding macro infrastructure |
| `latexml_math_parser` | `LaTeXML::MathParser` | Marpa-style math parser |
| `latexml_post` | `LaTeXML::Post::*` | XML→HTML/MathML/ePub/JATS post-processing |
| `latexml_oxide` | `latexml` CLI | Binary targets (`latexml_oxide`, `latexmlmath_oxide`) + integration suite |
| `latexml_codegen` | *(none)* | Proc macros for compile-time macro expansion |
| `latexml_contrib` | *(none)* | Contributed/runtime script bindings (Rhai) |

(Full engine map: [ORGANIZATION.md](ORGANIZATION.md).)

- **State** — a thread-local, global, mutable singleton (CHANGELOG 0.3.2),
  preserving TeX's inherently stateful/sequential model without threading a state
  parameter everywhere.
- **String interning** — CS names, attribute keys, font names go through an
  `arena` interner (O(1) equality, less allocation than Perl's copy-on-read).
- **Compile-time macros** — `DefMacro!`/`DefConstructor!`/`DefPrimitive!` expand at
  build time (proc macros), eliminating Perl's per-`\usepackage` parse cost.
- **Engine file layout** — the pools are split along the same seams Perl uses,
  one Rust file per pool: `latex_bootstrap.rs` / `latex_base.rs` /
  `latex_constructs.rs` (plus the Rust-only `latex_constructs_rust_only.rs` and
  the dump loader `latex_dump.rs`), and `plain_bootstrap.rs` / `plain_base.rs` /
  `plain_constructs.rs` / `plain_dump.rs`. Full map:
  [ORGANIZATION.md](ORGANIZATION.md).
- **`latexml_contrib`** — dispatches package names to Rust binding loaders
  (`Rc<dyn Fn(&str) -> Option<Result<()>>>`); raw-TeX-only packages use
  `InputDefinitions!(name, noltxml => true)`.

---

For the detailed catalogue of divergences, math-grammar rules, type improvements,
and future work, follow the theme files in the table above.
