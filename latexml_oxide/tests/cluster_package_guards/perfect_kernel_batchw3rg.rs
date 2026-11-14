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

/// Each row red on 72b3971e4ca (round F), Perl's output beside it. p1 (row 1): `MergeFont`'s Font.pm merge keys
/// `color`/`background`/`size` by name/`forcebold` (all dropped; Perl `color="red"`, a name, where ours holds
/// LookupColor's Color); an unknown key Warns (Perl ignores it). p2: ExplodeText's `é` is OTHER (row 3; 0 before);
/// CounterValue a Number (row 5; i64); SetCounter/AddToCounter of a Number, StepCounter's `$noreset` (rows 6-7;
/// "Function not found"); variadic Push/UnshiftValue (row 8); `Number("12abc")` 12 with Perl's `perl:warn` (row 10;
/// an Error and 0) and Revert of a string; ptValue's precision clamped (row 11; 1.6); LookupDimension of a Token and
/// of a Dimension (row 14). p3: `getAttribute` absent is `()` (row 2; ""), `document.setAttribute(node, k, v)` (row 9).
/// p4 (AUDMAC, silent loss): a `DefPrimitive` body's returned box, or array of boxes, is the primitive's (`B`, `E`
/// alone before), as Perl's `invoke` (Primitive.pm:78) and the `DefPrimitive!` macro.
/// Diagnostics in Perl's `(cat, obj, $where, msg, @details)` form (row 4). Perl: 3 W, plus the Error; ours 1 W more.
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
