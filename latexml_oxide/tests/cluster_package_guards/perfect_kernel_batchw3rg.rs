//! W3RG (PR #829 Rhai round G): a Rhai name that is a Perl API name behaves as Perl's (DG 2026-11-14; AUDNAME rows).
//! Pinned against Perl `latexml` with the equivalent `.ltxml` binding (`RequirePackage('color')` and the same calls).
#![cfg(feature = "runtime-bindings")]

use latexml::util::test::{assert_element, convert_files_with};

use crate::perfect_kernel_batch46::{error_count, warning_count};

const LXRG: &str = r#"RequirePackage("color");
DefPrimitive("\\fA", || MergeFont(#{color: "red", background: "blue"}));
DefPrimitive("\\fS", || MergeFont(#{size: "large"}));
DefPrimitive("\\fB", || MergeFont(#{forcebold: true}));
DefPrimitive("\\fU", || MergeFont(#{colour: "red"}));
DefMacro("\\rows", || {
  let r = [];
  r.push(if Equals(ExplodeText("éa"), Tokens(T_OTHER("é"), T_LETTER("a"))) { 1 } else { 0 });
  r.push(type_of(CounterValue("foo")));
  SetCounter("foo", Number(3)); r.push(CounterValue("foo").valueOf());
  AddToCounter("foo", Number(2)); r.push(CounterValue("foo").valueOf());
  StepCounter("foo", true); r.push(CounterValue("foo").valueOf());
  PushValue("plist", "a", "b"); r.push(LookupValue("plist").len());
  UnshiftValue("ul", "c"); UnshiftValue("ul", "a", "b"); r.push(LookupValue("ul").reduce(|s, v| s + v, ""));
  r.push(Number("12abc").valueOf());
  r.push(ToString(Revert("ab")));
  r.push(ptValue(Dimension("1.6pt"), -1).to_int());
  r.push(if LookupDimension(T_CS("\\parindent")) == LookupDimension("\\parindent") { "same" } else { "diff" });
  r.push(ToString(LookupDimension(Dimension("2pt"))));
  Warn("probe", "w4", (), "MSG4", "DETAIL");
  Warn("probe", "w4s", (), "MSG4S");
  Error("probe", "e4", (), "MSGE", "EDETAIL", "EDETAIL2");
  r.reduce(|s, v| if s == "" { `${v}` } else { s + " " + v }, "")
});
DefConstructor("\\fooA{}", "<ltx:text class='#1'>#1</ltx:text>", #{
  afterConstruct: |document, whatsit| {
    let el = document.getElement();
    document.setAttribute(el, "class", "set3" + if defined(el.getAttribute("nosuch")) { "D" } else { "U" });
  }
});
DefPrimitive("\\digtest", || Digest(T_LETTER("A")));
DefPrimitive("\\digarr", || [Digest(T_LETTER("C")), Digest(T_LETTER("D"))]);
"#;

const LXRG_TEX: &str = r"\documentclass{article}
\usepackage{lxrg}
\newcounter{foo}
\begin{document}
A {\fA B} {\fS C} {\fB D} {\fU E}

\rows

Text \fooA{x}.

\digtest B \digarr E
\end{document}
";

/// Rows red on 72b3971e4ca (round F), Perl's output beside each (OD #1035). Perl: 3 W plus the Error; ours 1 W more.
#[test]
fn rhai_perl_names_behave_as_perl() {
  let (log, xml) = convert_files_with(LXRG_TEX, &[("lxrg.sty.rhai", LXRG)], None);
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(warning_count(&log), 4, "{log}");
  for line in [
    "Warning:perl:warn Argument \"12abc\" isn't numeric in int",
    "Warning:probe:w4 MSG4\n",
    "\tDETAIL",
    "Warning:probe:w4s MSG4S\n",
    "Error:probe:e4 MSGE\n",
    "\tEDETAIL\n\tEDETAIL2",
    "Warning:unexpected:colour MergeFont: unknown font key 'colour' ignored",
  ] {
    assert!(log.contains(line), "{line}\n{log}");
  }
  for (id, para) in [
    (
      "p1",
      r##"<para xml:id="p1"><p>A <text backgroundcolor="#0000FF" color="#FF0000">B</text> <text fontsize="120%">C</text> <text font="bold">D</text> E</p></para>"##,
    ),
    (
      "p2",
      r#"<para xml:id="p2"><p>1 Number 3 5 6 2 abc 12 ab 2 same 2.0pt</p></para>"#,
    ),
    (
      "p3",
      r#"<para xml:id="p3"><p class="set3U">Text <text class="x">x</text>.</p></para>"#,
    ),
    ("p4", r#"<para xml:id="p4"><p>AB CDE</p></para>"#),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
}

/// W3RH, no colour package loaded: rows red on 02ce42ad5dc (revW3RFG). p1: a `LookupDefinition` handle keeps the
/// definition it found, also stored as a value (`A:NEW:NEW`, `B:` an undefined `\baz`, before; Perl `A:OLD:NEW`). p2: a base colour name in `MergeFont` and a `font:` option
/// without color.sty (dvipsnam.def read first: 4 Errors, Black; Perl 0 E; the `font:` colour outlives `\fC`, as Perl's). p3: `int ⊕ Number` and float pairs compare
/// as ints (`0 < n` was `false` silently), `roman(Number)` (Function not found), Perl's `int()` of a string (OD #1035);
/// W3RI (revW3RH B1, red on 914dc00e41f): `n * 1.5` gave 4 silently (Perl's truncating `multiply`), `%`/`**`/bitwise/`i / n`/`N * n` Function not found.
/// revW3RI: `n / Number(0)` and `n / 0` returned 58720255 silently (the core `divide`); now Rhai's division-by-zero error (`ERR`).
const LXRH: &str = r#"DefMacro("\\foo", "OLD");
DefMacro("\\swap", || { let d = LookupDefinition(T_CS("\\foo")); DefMacro("\\foo", "NEW"); AssignMeaning(T_CS("\\bar"), d); });
DefMacro("\\saveit", || { AssignValue("saved", LookupDefinition(T_CS("\\foo"))); });
DefMacro("\\restoreit", || { AssignMeaning(T_CS("\\baz"), LookupValue("saved")); });
DefPrimitive("\\fR", || MergeFont(#{color: "red", background: "yellow"}));
DefConstructor("\\fC{}", "<ltx:text class='c'>#1</ltx:text>", #{font: #{color: "blue"}});
DefMacro("\\rows", || {
  let n = CounterValue("cc");
  let r = [0 < n, 3 <= n, 4 > n, 3 >= n, 3 == n, 4 != n, n < 3.5, 3.5 > n, n == 3.0, 3.0 == n, n + 0.5, 1.5 * n,
    n * 1.5, n * 1.5 == 4.5, 7 / n, n % 2, 7 % n, n ** 2, 2 ** n, n % 2.0, n & 1, 1 << n, Number(2) * n];
  r.push(ToString(roman(n)) + ToString(Roman(n)));
  for spec in ["1e3", ".5", "1.5", " -7 ", "abc"] { r.push(Number(spec).valueOf()); }
  for z in [Number(0), 0] { let q = "ERR"; try { q = `${n / z}`; } catch { } r.push(q); }
  r.reduce(|s, v| if s == "" { `${v}` } else { s + " " + v }, "")
});
"#;

const LXRH_TEX: &str = r"\documentclass{article}
\usepackage{lxrh}
\newcounter{cc}\setcounter{cc}{3}
\begin{document}
\swap A:\bar:\foo. \saveit\def\foo{NEWER}\restoreit B:\baz:\foo.

{\fR D} \fC{E}

\rows
\end{document}
";

#[test]
fn rhai_perl_names_review_rows() {
  let (log, xml) = convert_files_with(LXRH_TEX, &[("lxrh.sty.rhai", LXRH)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  for line in [
    "Warning:deprecated:Number as an int",
    "Warning:perl:warn Argument \"abc\" isn't numeric in int",
  ] {
    assert!(log.contains(line), "{line}\n{log}");
  }
  for (id, para) in [
    (
      "p1",
      r#"<para xml:id="p1"><p>A:OLD:NEW. B:NEW:NEWER.</p></para>"#,
    ),
    (
      "p2",
      r##"<para xml:id="p2"><p><text backgroundcolor="#FFFF00" color="#FF0000">D</text> <text class="c" color="#0000FF">E</text></p></para>"##,
    ),
    (
      "p3",
      r##"<para xml:id="p3"><p><text color="#0000FF">true true true true true true true true true true 3.5 4.5 4.5 true 2 1 1 9 8 1.0 1 8 6 iiiIII 1000 0 1 -7 0 ERR ERR</text></p></para>"##,
    ),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
}

/// W3RH part 1 (AUDNAME "Perl API binding authors use that Rhai lacks"): the I-forms (`coerceCS` names, a Token, a
/// `\csname` name, an active character; a parameter spec or `()`; each body kind), `IfCondition`, `Box`, `List`,
/// `unlist`, `getCatcode`, `getString`, `getCSName`, `whatsit.getFont`. Each row was Function not found on 17a64c0415a;
/// Perl's `.ltxml` twin gives these paragraphs and the one Error (`IfCondition('\relax')`).
const LXI: &str = r#"DefMacroI("\\mA", (), "MA");
DefMacroI(T_CS("\\mB"), "{}", |gullet, x| Tokens(x, x));
DefMacroI("\\mC", "[]{}", Tokens(T_LETTER("c")), #{locked: false});
DefMacroI("\\csname m D\\endcsname", (), "MD");
DefMacroI("~", (), "TILDE");
DefPrimitiveI("\\pA", "{}", |stomach, x| Box("P" + ToString(x)));
DefPrimitiveI("\\pB", (), ());
DefPrimitiveI("\\pL", (), |stomach| List([Box("x"), Box("y")]));
DefConstructorI("\\cA", "{}", "<ltx:text class='ca'>#1</ltx:text>");
DefConstructorI(T_CS("\\cB"), (), |document| { document.absorbString("CB"); });
DefConstructorI("\\cC", "{}", "<ltx:text class='cc'>#1</ltx:text>", #{
  afterDigest: |whatsit| { AssignValue("lx:shape", whatsit.getFont().getShape(), "global"); }
});
DefMacro("\\rows", || {
  let r = [];
  r.push(Tokenize("ab\\c").unlist().len());
  r.push(T_LETTER("a").getCatcode());
  r.push(if T_CS("\\foo").getString() == "\\foo" { "S" } else { "N" });
  r.push(T_BEGIN().getCSName());
  r.push(if T_CS("\\foo").getCSName() == "\\foo" { "C" } else { "N" });
  r.push(if IfCondition("\\iffoo") { "T" } else { "F" });
  r.push(if IfCondition(T_CS("\\iftrue")) { "T" } else { "F" });
  r.push(if defined(IfCondition("\\relax")) { "D" } else { "U" });
  r.push(LookupValue("lx:shape"));
  r.reduce(|s, v| if s == "" { `${v}` } else { s + " " + v }, "")
});
"#;

const LXI_TEX: &str = r"\documentclass{article}
\usepackage{lxi}
\newif\iffoo\footrue
\begin{document}
A~B \mA \mB{b} \mC[o]{z} \csname m D\endcsname

\pA{q} \pB \pL \cA{a} \cB \textit{\cC{s}}

\rows
\end{document}
";

#[test]
fn rhai_perl_i_forms_and_accessors() {
  let (log, xml) = convert_files_with(LXI_TEX, &[("lxi.sty.rhai", LXI)], None);
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    log.contains("Error:expected:conditional Expected a conditional, got '\\relax'"),
    "{log}"
  );
  for (id, para) in [
    ("p1", r#"<para xml:id="p1"><p>ATILDEB MAbb c MD</p></para>"#),
    (
      "p2",
      r#"<para xml:id="p2"><p>Pqxy<text class="ca">a</text>CB<text class="cc" font="italic">s</text></p></para>"#,
    ),
    (
      "p3",
      r#"<para xml:id="p3"><p>3 11 S Begin C T T U italic</p></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
}

/// W3RH part 2, the REDs `rhai-bindings/lookupmeaning_is_a_string`, `tostring_array_of_handles` and
/// `lookupstring_of_tokens` (red on 17a64c0415a: `N`, Rust type paths with 3 Errors, `Stored::Tokens[…]`).
/// `LookupMeaning` is the definition or Token (`()` undefined, `== ""` Warned, ruling 9); a `Definition` prints as
/// Perl's `ToString` (`\foo []` here, as Perl; `\` as `!` to keep the row literal); an array prints each handle as TeX.
const LXV: &str = r#"fn bang(s) { s.replace("\\", "!"); s }
DefMacro("\\foo{}[]", "F");
DefPrimitive("\\lxlm", || AssignMeaning(T_CS("\\lxr"), LookupMeaning("\\relax")));
DefPrimitive("\\keep{}", |stomach, v| { AssignValue("lx:k", v, "global"); AssignValue("lx:t", T_LETTER("t"), "global"); });
DefMacro("\\rows", || {
  let r = [];
  r.push(bang(ToString(LookupDefinition("\\foo"))));
  r.push(bang(ToString(LookupMeaning("\\foo"))));
  r.push(if defined(LookupMeaning("\\nosuch")) { "D" } else { "U" });
  r.push(ToString(LookupMeaning(T_LETTER("q"))));
  r.push(type_of(LookupMeaning("\\foo")));
  r.push(if LookupMeaning("\\nosuch") == "" { "E" } else { "N" });
  r.push(LookupString("lx:k") + LookupString("lx:t"));
  r.push(ToString([Number(3), Dimension("1pt"), T_LETTER("a"), Tokenize("bc"), Glue("1pt plus 1fil")]));
  r.push(`${[T_LETTER("z"), Dimension("2pt")]}`);
  r.reduce(|s, v| if s == "" { `${v}` } else { s + " ; " + v }, "")
});
"#;

const LXV_TEX: &str = r"\documentclass{article}
\usepackage{lxv}
\begin{document}
\lxlm\ifx\lxr\relax R\else N\fi

\keep{k}\rows
\end{document}
";

#[test]
fn rhai_values_print_as_perl() {
  let (log, xml) = convert_files_with(LXV_TEX, &[("lxv.sty.rhai", LXV)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(log.contains("Warning:deprecated:() == string"), "{log}");
  for (id, para) in [
    ("p1", r#"<para xml:id="p1"><p>R</p></para>"#),
    (
      "p2",
      r#"<para xml:id="p2"><p>!foo [] ; !foo [] ; U ; q ; Definition ; E ; kt ; [3, 1.0pt, a, bc, 1.0pt plus 1.0fil] ; [z, 2.0pt]</p></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
}

/// revRHAIH's rows (repro `rhai-bindings/ifcondition_args_and_coerce_cs`), red on 396ffa641ea (the binding load failed
/// on `DefMacroI(Tokenize(…))`, 4 Errors, `IfCondition("\\ifx")` read the document). Perl's twin: these paragraphs and
/// the two SetCondition Errors; ours adds ruling 9's two deprecation Warnings.
const LXH: &str = include_str!("../../../tools/perfect_kernel/repros/rhai-bindings/lxh.sty.rhai");

const LXH_TEX: &str = include_str!(
  "../../../tools/perfect_kernel/repros/rhai-bindings/ifcondition_args_and_coerce_cs.tex"
);

#[test]
fn rhai_review_rows_round_h() {
  let (log, xml) = convert_files_with(LXH_TEX, &[("lxh.sty.rhai", LXH)], None);
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  for line in [
    "Error:expected:conditional Expected a conditional defined by \\newif, got '\\relax'",
    "Error:expected:conditional Expected a conditional defined by \\newif, got '\\nosuch'",
    "Warning:deprecated:value.is_empty()",
    "Warning:deprecated:value.len()",
  ] {
    assert!(log.contains(line), "{line}\n{log}");
  }
  for (id, para) in [
    ("p1", r#"<para xml:id="p1"><p>ME</p></para>"#),
    (
      "p2",
      r#"<para xml:id="p2"><p>T T F T T F D U E 6 1</p></para>"#,
    ),
    ("p3", r#"<para xml:id="p3"><p>X</p></para>"#),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
}
