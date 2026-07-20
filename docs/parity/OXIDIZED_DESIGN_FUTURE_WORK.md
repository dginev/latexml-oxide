# Oxidized Design — Future Work (Beyond Perl Parity)

[← OXIDIZED_DESIGN.md](OXIDIZED_DESIGN.md) · Directions where we know what "better than Perl" looks like but have not built it yet.

---


The Rust port aims first for behavioral parity with Perl LaTeXML
(see "Faithfulness first" above). But the project also positions us
to **go beyond parity** in places where Perl LaTeXML's grammar or
output choices are themselves limited. This section records
deliberate "future work" directions where we know what better looks
like; their resolution is not a parity regression to fix but an
extension of the project's value.

### Rich math-grammar parsing for kerned-stack norm idioms

**Status:** Future work — extends beyond Perl LaTeXML.

**Background.** Papers routinely fake double-bar and triple-bar
norms by stacking `\left|\right|` pairs with small negative kerns:

```latex
\newcommand{\vertii}[1]{{\left\vert\kern-0.25ex\left\vert
                          #1 \right\vert\kern-0.25ex\right\vert}}     % ‖x‖
\newcommand{\vertiii}[1]{{\left\vert\kern-0.25ex\left\vert\kern-0.25ex
                          \left\vert #1
                          \right\vert\kern-0.25ex\right\vert\kern-0.25ex
                          \right\vert}}                                % |||x|||
```

Visually the bars touch and render as `‖x‖` / `|||x|||`. Semantically
both Perl LaTeXML and the Rust port currently parse each
`\left|`/`\right|` pair as an *independent fence delimiter*,
producing nested `|·|` inside `|·|` rather than a single
norm-delimiter pair. For a juxtaposed expression like
`|||M||| · |||Σ||| · ‖M−M'‖_F + ‖M−M'‖_F · |||Σ||| · |||M'|||`
this yields ~25-level nesting in MathML (witness
`tests/math/norm_kerned_delims.tex`, originally from arXiv:2211.13044
§S4.Ex17).

**Why this is "beyond parity" not a regression.** Perl LaTeXML
focuses on fence-pairing rules that mirror TeX's `\left`/`\right`
matching and does not attempt to detect kerned-stack idioms. The
Rust port's math layer is built on a more expressive Marpa-based
grammar (see [`MATH_GRAMMAR_FIRST_PRINCIPLES.md`](../math/MATH_GRAMMAR_FIRST_PRINCIPLES.md)
and [`MATH_PARSER_AND_ASF.md`](../math/MATH_PARSER_AND_ASF.md)), giving us
the option to produce **well-structured MathML Core** that follows
the XMath taxonomy: a single `<mrow intent=":Frobenius-norm">` or a
proper U+2016 `‖` / U+2AF4 `⫴` delimiter, instead of token-level
fence soup.

**Approach (sketch, three layers — pick any):**

1. **Gullet-level rewrite.** Detect the kerned-stack pattern in the
   gullet (the kern argument has a known small negative value
   between two adjacent `\left|` or `\right|` tokens) and merge into
   a synthesized macro like `\lx@doublebar` / `\lx@triplebar`. The
   math parser then sees clean delimiters and the existing fence
   rules produce well-typed MathML directly. Smallest blast radius.

2. **Math-grammar level.** Add explicit NORM / OPERATORNORM
   nonterminals to the Marpa grammar that accept balanced `|`/`‖`/
   `|||` openings, with their own action closures that emit a
   semantic `intent=":operator-norm"` mrow. This is the
   "richer-grammar" path the Rust port was designed to enable.

3. **Both, with role tagging.** Pre-process at the gullet AND keep
   the grammar prepared for U+2016 / U+2AF4 delimiters arriving on
   the token stream. Belt-and-suspenders for varied paper inputs.

**Related item (same paper, same equation) — ⚠️ PARTLY DONE AND PARTLY
RETRACTED, 2026-07-20.** Equation rows whose first non-whitespace token is a
binary relation (`\leq`, `=`, `\subseteq`) carry an empty placeholder as the
left operand. Two directions were proposed here: suppress it, or tag the row
`intent=":continuation"`.

**Do NOT suppress it.** That half was implemented and had to be reverted: in
MathML the operand SLOT is what makes the operator infix (form is inferred from
position — first child of its `<mrow>` ⇒ prefix — and the form selects the
operator-dictionary spacing). Dropping it made `<mo>=</mo>` the first child of
every continuation row, diverging from Perl, which keeps the slot
(`MathML.pm:1474` renders `absent` as an empty `<m:mi/>`). Reported as issue
**#312**. Guard: `90_latexmlpost.rs::alignrows_operand_slot_keeps_relop_infix`.

What DID land from this item: the placeholder is an empty `<m:mphantom/>`
rather than Perl's `<m:mi/>` — an empty `<mi>` asserts "here is an identifier"
about content that has none, and `document.rs` carries a `debug_assert!`
refusing to materialize one. `mphantom` is chosen over a bare `mrow` because
its definition is precisely "occupies space, renders invisibly".

**Still open:** the `intent=":continuation"` tagging, which is the part that
actually expresses "the LHS is the prior row" without touching layout. That is
the direction to take if this is revisited. Task #264.

**Pinned-baseline test.** The current (over-nested) output is
captured as `tests/math/norm_kerned_delims.{tex,xml}` so we can
detect when a future grammar/preprocess change *improves* it
without it silently regressing. The test file's leading
`% comments` annotate each section with the expected shape.

### TOML profiles instead of Perl `.opt` (issue #191, `--profile`)

**Status:** Planned — not yet implemented. Deliberate divergence from
Perl's profile file format.

**Perl behavior.** `--profile=NAME` (and its `--mode` alias) loads
`<NAME>.opt` — a flat `key = value` file (`Config.pm::_obey_profile`).
We already ship the set under `resources/Profiles/*.opt` (`fragment`,
`math`, `standard`, `modern`, `stex*`, …). The format has three warts: an
empty value means "boolean true" (`pmml =`), lists are repeated keys
(`preload = …` ×N), and everything is stringly-typed.

**Planned Rust shape.** Express profiles as **TOML**, deserialized via
serde into the same option struct `clap` already populates — so a profile
is just a *defaults layer*: `built-in/embedded profile < user CLI flags`
(CLI wins, matching Perl's precedence). TOML fixes all three warts
natively (`pmml = true`, `preload = ["a","b"]`, `timeout = 120`) and adds
`extends = "fragment"` profile inheritance that `.opt` can't express
cleanly.

```toml
# fragment.toml
extends   = "math"          # optional inheritance
format    = "xhtml"
whatsin   = "fragment"
whatsout  = "fragment"
pmml = true; cmml = true; mathtex = true
nodefaultresources = true
preload = ["LaTeX.pool", "article.cls", "amsmath.sty", "[ids]latexml.sty"]
path    = ["$LATEXMLINPUTS"]
```

**Decision (2026-05-24): TOML-native, convert-and-drop.** Convert the
shipped `resources/Profiles/*.opt` to `*.toml` and remove the `.opt`
files; **no legacy `.opt` reader** — `--profile` consumes only TOML. (A
Perl `.opt` is trivially hand-portable, and we control the shipped set, so
the compat reader isn't worth the surface area.)

**Constraints to preserve:** built-in profiles stay **embedded**
(`include_str!`/`include_dir!`) per the self-contained-binary principle,
with a disk override (`<NAME>.toml`); keep `$LATEXMLINPUTS` expansion in
`path`; keep `--mode` as an alias for `--profile`.
Tracked under issue #191 in [`ISSUE_AUDIT.md`](../release/ISSUE_AUDIT.md).
