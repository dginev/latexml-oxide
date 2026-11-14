# Writing a Rhai binding: a guide for newcomers

[← docs/README.md](../README.md) · Reference: [`API.md`](../../latexml_contrib/src/script_bindings/API.md) (every call) ·
[`CHEATSHEET.md`](../../latexml_contrib/src/script_bindings/CHEATSHEET.md) (Perl | Rust macro | Rhai) · Design:
[`script_bindings_plan.md`](script_bindings_plan.md)

A **binding** tells latexml-oxide what a LaTeX package or class means: which macros expand to what, which commands
build which XML. Perl LaTeXML writes them in Perl (`foo.sty.ltxml`); latexml-oxide compiles its own in Rust, and you
can add yours at runtime in **Rhai**, a small embedded scripting language, with no Rust toolchain and no recompile. The
dialect keeps Perl's names and argument forms, so a Perl binding translates almost line by line.

## Load a binding

Name the file after the package or class it binds, plus `.rhai`: `mypkg.sty.rhai`, `myclass.cls.rhai`. With
`mypkg.sty.rhai` beside `doc.tex`, which says `\usepackage{mypkg}`, run `latexml_oxide doc.tex --dest=doc.html` (or
`--dest=doc.xml` for the XML); every released binary reads `.rhai` bindings.

- Put it beside the document (or in a directory given with `--path=DIR`): `\usepackage{mypkg}` loads it. The `.rhai`
  file is searched **first**, so `article.cls.rhai` overrides the built-in `article` binding.
- Or load it for every document: `--preload=mypkg.sty` finds `mypkg.sty.rhai` the same way.
- A `.rhai` file is code: treat one that ships with a source tree as you would any untrusted program.

## The shape of a file

A binding file is a sequence of calls, read top to bottom when the package loads, as Perl reads a `.ltxml` file:

```rhai
DefMacro("\\hello{}", |gullet, name| Tokens([T_OTHER("("), name, T_OTHER(")")]));
DefConstructor("\\term{}", "<ltx:text class=\"term\">#1</ltx:text>");
```

- **Backslashes double** in Rhai strings: `"\\hello"`, as in Perl's double-quoted strings (Perl's `'\hello'` single
  quotes have no Rhai equivalent).
- **A string names a control sequence** wherever Perl takes a CS (`Let("\\a", "\\b")`), as Perl's `T_CS('\a')` would.
- **Option hashes are maps:** Perl's `DefMath('\sum', "∑", role => 'SUMOP')` is
  `DefMath("\\sum", "∑", #{ role: "SUMOP" })`. A key Perl does not accept is an `Error:misdefined`.
- **Scopes** are strings, as Perl's: `AssignValue("k", v, "global")`; no scope means local.

## Values

| Rhai | Perl | build it with |
|---|---|---|
| `Token` | `LaTeXML::Core::Token` | `T_CS("\\relax")`, `T_LETTER("a")`, `T_OTHER("(")`, `T_SPACE()`, `T_BEGIN()`, `T_END()` |
| `Tokens` | `LaTeXML::Core::Tokens` | `Tokens(t1, t2)` (up to four) or `Tokens([..])`; `Tokenize("TeX source")`; `Explode("text")` |
| `Number`, `Dimension`, `Glue`, `MuGlue` | the `Common::` value classes | `Number(5)`, `Dimension("3pt")`, `Glue("1pt plus 2fil")`, `MuGlue("3mu")` |
| string, int, bool, array, map | Perl's scalars, lists and hashes | Rhai literals |
| `()` | `undef` | a missing value: an absent optional argument, an unset `LookupValue` |

`T_CS` and every `T_*` return one `Token`; `Tokens` only where Perl builds a list. `ToString(v)` is Perl's
`ToString`: the TeX source of Tokens, the text of a value (`"3.0pt"`). The value handles carry Perl's methods and
arithmetic (`d * 2`, `g.add(Glue("1pt"))`, `d > Dimension("5pt")`); `LookupRegister("\\parskip")` returns one. Glue
arithmetic is TeX's `\advance` (the higher fill order wins), where Perl's Glue.pm differs
(OXIDIZED_DESIGN_DIVERGENCES #1030).

## Closures: the Perl form

A body is a closure. It receives what Perl's `sub` receives in `@_`, in that order, and you **name each one**:

| definer | Perl | Rhai |
|---|---|---|
| `DefMacro` | `sub { my ($gullet, $x) = @_; … }` | `\|gullet, x\| …` (each argument is `Tokens`; return `Tokens` or a string of TeX) |
| `DefPrimitive` | `sub { my ($stomach, $x) = @_; … }` | `\|stomach, x\| …` |
| `DefConstructor` | `sub { my ($document, $x, %props) = @_; … }` | `\|document, x\| …` |
| `DefParameterType` reader | `sub { my ($gullet) = @_; … }` | `\|gullet\| gullet.readArg()` |
| `reversion`, `afterDigest`, `sizer` | `sub { my ($whatsit, …) = @_; … }` | `\|whatsit, …\| …` |

A closure is read as the Perl form when it declares the handle **and** every argument (`|gullet, x|`). With fewer
parameters it is the older form (arguments only, as strings), which still works this release and Warns
`Warning:deprecated` naming the fix; name an unused argument with a leading underscore (`|gullet, _x|`). A macro with
no parameters takes `|gullet|` and reads its own arguments (`gullet.readArg()`, `gullet.readOptional()`), as Perl's
`$gullet->readArg`. One exception today: a `properties` closure receives the arguments only.

## Truth, `()` and strings

- **Perl's truthiness is `!!v`** (and `!v` is `unless`): `()`, `""`, `"0"`, `0` and `0.0` are false; every Token,
  Tokens (even empty), value handle, array and map is true, as Perl's objects are. To test a list's text, test
  `!!ToString(x)`.
- **`defined(v)`** is Perl's `defined`: false only for `()`. An absent optional argument is `()`:
  `if defined(opt) { … }`.
- **`()` compares equal to `""`** in either order, as Perl's `undef eq ''`, with a once-per-load deprecation Warning:
  write `defined(v)` or `!v`. Rhai's `switch` matches by type, so its `"" =>` case never matches `()`.
- **Compare text with `ToString`:** `ToString(x) == "abc"`. Comparing a Tokens or a value handle with a string directly
  still works and Warns once per load.

## Diagnostics

`Warn(category, object, message)`, `Error(…)`, `Info(…)` and `Fatal(…)` are Perl's (also Perl's `(category, object, (),
message, details…)` form), with the same severities: a
`Fatal` stops the conversion, an `Error` marks the output as likely wrong, a `Warning` notes a recovered problem.
A Rhai error in a body (a `throw`, a typo'd function) is reported as an `Error:script:` naming the binding, and that
binding contributes nothing; the conversion goes on.

## Migrating from the deprecated names

Each old name still works this release and Warns `Warning:deprecated` once per binding load, naming its replacement:

| old | new |
|---|---|
| `assign_value(k, v)`, `assign_global(k, v)`, `AssignString`/`AssignNumber`/`AssignBool` | `AssignValue(k, v[, scope])` |
| `lookup_value(k)` | `LookupValue(k)` |
| `AssignMeaning(cs, "\\other")` (a string meaning) | `Let(cs, other[, scope])`, or `AssignMeaning(T_CS(cs), LookupDefinition(other))` |
| `whatsit` (the global), `whatsit.argString(n)`, `whatsit.propertyString(k)` | the hook's `whatsit` argument, `whatsit.getArg(n)`, `whatsit.getProperty(k)` |
| `document.getProperty(k)` in a constructor body | the body's trailing properties map |
| `DefAccent(cs, combining, standalone, true)` | `DefAccent(cs, combining, standalone, #{below: true})` |
| a body naming only its arguments (`\|x\|`) | the Perl form (`\|gullet, x\|`) |
| `value == "string"`, `() == ""` | `ToString(value) == "string"`, `defined(value)` / `!!value` |
| `CounterValue(c) + 1`, `CounterValue(c) == 3` (an int before) | `CounterValue(c).valueOf() + 1` — it returns a Number, as Perl's |
| `document.setAttribute(key, value)` (the current node) | `document.setAttribute(node, key, value)`, e.g. `document.getNode()` |
| a `DefPrimitive` body whose last value is not a box (`\|\| RefStepCounter("c")`) | end it with `return;`, or return the boxes it should print |

`node.getAttribute(k)` of an absent attribute is `()`, as XML::LibXML's undef: test it with `defined(…)`.

## Worked examples

Each file under [`docs/examples/`](../examples/) is a binding with a document using it; CI converts every pair with no
Error or Warning and checks the output (`30_script_bindings::docs_examples_convert_clean`):

- [`macros.sty.rhai`](../examples/macros.sty.rhai): `DefMacro` with Tokens, an optional argument, gullet reads.
- [`constructors.sty.rhai`](../examples/constructors.sty.rhai): `DefConstructor` as a template and as a closure,
  properties, an environment.
- [`registers.sty.rhai`](../examples/registers.sty.rhai): `DefRegister`, typed values, `AssignRegister`.
- [`sample.sty.rhai`](../examples/sample.sty.rhai): a tour of the wider surface (options, counters, keyvals, math,
  rewrites, definition hooks).
