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
/// as ints (`0 < n` was `false` silently), `roman(Number)` (Function not found), Perl's `int()` of a string (OD #1035).
const LXRH: &str = r#"DefMacro("\\foo", "OLD");
DefMacro("\\swap", || { let d = LookupDefinition(T_CS("\\foo")); DefMacro("\\foo", "NEW"); AssignMeaning(T_CS("\\bar"), d); });
DefMacro("\\saveit", || { AssignValue("saved", LookupDefinition(T_CS("\\foo"))); });
DefMacro("\\restoreit", || { AssignMeaning(T_CS("\\baz"), LookupValue("saved")); });
DefPrimitive("\\fR", || MergeFont(#{color: "red", background: "yellow"}));
DefConstructor("\\fC{}", "<ltx:text class='c'>#1</ltx:text>", #{font: #{color: "blue"}});
DefMacro("\\rows", || {
  let n = CounterValue("cc");
  let r = [0 < n, 3 <= n, 4 > n, 3 >= n, 3 == n, 4 != n, n < 3.5, 3.5 > n, n == 3.0, 3.0 == n, n + 0.5, 1.5 * n];
  r.push(ToString(roman(n)) + ToString(Roman(n)));
  for spec in ["1e3", ".5", "1.5", " -7 ", "abc"] { r.push(Number(spec).valueOf()); }
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
      r##"<para xml:id="p3"><p><text color="#0000FF">true true true true true true true true true true 3.5 4.5 iiiIII 1000 0 1 -7 0</text></p></para>"##,
    ),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
}
