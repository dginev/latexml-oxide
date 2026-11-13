//! Red/green guards for perfect-kernel batch 64zb: a locked binding owns its calls (`state::install_definition`,
//! `state::owns_calls`). A definition the document makes of a locked binding that takes a braced argument becomes the
//! meaning for every use without an argument, as in pdflatex, while a call (`{`, or a `[`/`*` the binding's signature
//! opens with) still runs the binding, kept as `\lx@locked@<cs>`. Perl refuses every TeX-source redefinition of a
//! locked name (State.pm:502-517). Repros `tools/perfect_kernel/repros/macro-state/m5_*.tex`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{
  convert_files_with, convert_with, convert_with_then, error_count, warning_count,
};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// The log and XML of `tex`, which must give no error and no warning.
fn convert_clean_log(tex: &str) -> (String, String) {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  (log, xml)
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/macro-state/",
      $name,
      ".tex"
    ))
  };
}

/// a1: a pgffor loop variable named `\text` (`\foreach \i/\text in {1/{$v_1$},2/{$v_2$}}`; the M5 cluster of run
/// 336, 2405.18512) is the node's text, as in pdflatex — not amsmath's `\text` reading the node's `}` (15 errors).
#[test]
fn foreach_text_variable_is_the_node_text() {
  let xml = convert_clean(repro!("m5_a1_foreach_text"));
  assert_eq!(xml.matches("<Math ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "Math",
    &["tex=\"v_{1}\""],
    r#"<Math mode="inline" tex="v_{1}" text="v _ 1" xml:id="p1.pic1.m1"><XMath xml:id="p1.pic1.m1.1"><XMApp xml:id="p1.pic1.m1.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &["tex=\"v_{2}\""],
    r#"<Math mode="inline" tex="v_{2}" text="v _ 2" xml:id="p1.pic1.m2"><XMath xml:id="p1.pic1.m2.1"><XMApp xml:id="p1.pic1.m2.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math>"#,
  );
}

/// a2: `\@for\text:=alpha,beta\do{[\text]}` prints the values (base: Fatal).
#[test]
fn atfor_text_variable_is_the_value() {
  let xml = convert_clean(repro!("m5_a2_atfor_text"));
  assert_element(&xml, "p", &[], "<p xml:id=\"p1.1\">[alpha][beta]</p>");
}

/// a3: a local `\def\text{local}` is the value inside its group; amsmath's `\text{…}` call still works in the same
/// group's math and after it.
#[test]
fn local_text_value_and_later_call() {
  let xml = convert_clean(repro!("m5_a3_def_text_group"));
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">Value: local. After: <Math mode="inline" tex="a\text{ and }b" text="a * [ and ] * b" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMText xml:id="p1.m1.1.1.3"> and </XMText><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>.</p>"#,
  );
}

/// a4: a body-level `\def\title{On Widgets}` is the value `\section*{\title}` prints (base: the locked `\title` read
/// `}` as its argument, 1 error).
#[test]
fn body_title_value_is_the_section_title() {
  let xml = convert_clean(repro!("m5_a4_def_title_body"));
  assert_element(&xml, "title", &[], "<title>On Widgets</title>");
}

/// a5: a pgffor loop variable named `\title`.
#[test]
fn foreach_title_variable_is_the_value() {
  let xml = convert_clean(repro!("m5_a5_foreach_title"));
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">(1: Alpha) (2: Beta)</p>",
  );
}

/// d1: a preamble `\gdef\text{TXT}` is the value of `\text` (pdflatex: "Value: TXT.").
#[test]
fn preamble_gdef_text_is_the_value() {
  let xml = convert_clean(repro!("m5_d1_text_gdef_preamble"));
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">Value: TXT. <Math mode="inline" tex="a" text="a" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></p>"#,
  );
}

/// A value use, then a call, in one scope: the call still makes amsmath's `\text` element.
#[test]
fn value_use_then_call() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage{amsmath}\\begin{document}\n\\def\\text{plain}[\\text] $x\\text{ if }y$\n\\end{document}",
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">[plain] <Math mode="inline" tex="x\text{ if }y" text="x * [ if ] * y" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMText xml:id="p1.m1.1.1.3"> if </XMText><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math></p>"#,
  );
}

/// A redefinition with parameters defines a command, not a value (user ruling 2026-10-11): it stays refused, and
/// every use is the binding's, a single-token argument too — `\renewcommand\mathbf[1]{\boldsymbol{#1}}` keeps both
/// `\mathbf x` and `\mathbf{y}` amsmath-bold (2006.13053 `\mathbf j`, 2006.14895 `\mathbf 0`, 1806.00338 `\mb a_0`).
#[test]
fn command_redefinition_keeps_every_use_with_the_binding() {
  let (log, xml) = convert_clean_log(
    "\\documentclass{article}\\usepackage{amsmath,amsbsy}\n\\renewcommand\\mathbf[1]{\\boldsymbol{#1}}\n\\begin{document}\n$\\mathbf x\\,\\mathbf{y}$\n\\end{document}",
  );
  assert_element(
    &xml,
    "Math",
    &[],
    r#"<Math mode="inline" tex="\mathbf{x}\,\mathbf{y}" text="x * y" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="bold" role="UNKNOWN" rpadding="1.7pt">x</XMTok><XMTok font="bold" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"#,
  );
  assert!(!log.contains("Info:lock"), "{log}");
  // …and so do the robust and ltcmd forms, whose outer macro is parameterless (`\protect \mathit␣`,
  // `\__cmd_start_optimized: \mathbf code`): `state::defines_value` reads what they wrap (2306.08511, 2306.10719,
  // 2306.05287; review probe p2).
  let (log, xml) = convert_clean_log(
    "\\documentclass{article}\\usepackage{amsmath,amsbsy,mathrsfs}\n\\RenewDocumentCommand\\mathbf{m}{\\boldsymbol{#1}}\n\\RenewDocumentCommand\\mathcal{o m}{\\mathscr{#2}}\n\\DeclareRobustCommand\\mathit[1]{\\boldsymbol{#1}}\n\\begin{document}\n$\\mathbf x\\,\\mathbf{y}\\,\\mathcal D\\,\\mathcal{C}\\,\\mathit u\\,\\mathit{w}$\n\\end{document}",
  );
  assert_element(
    &xml,
    "Math",
    &[],
    r#"<Math mode="inline" tex="\mathbf{x}\,\mathbf{y}\,\mathcal{D}\,\mathcal{C}\,\mathit{u}\,\mathit{w}" text="x * y * D * C * u * w" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="bold" role="UNKNOWN" rpadding="1.7pt">x</XMTok><XMTok font="bold" role="UNKNOWN" rpadding="1.7pt">y</XMTok><XMTok font="caligraphic" role="UNKNOWN" rpadding="1.7pt">D</XMTok><XMTok font="caligraphic" role="UNKNOWN" rpadding="1.7pt">C</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">u</XMTok><XMTok font="italic" role="UNKNOWN">w</XMTok></XMApp></XMath></Math>"#,
  );
  assert!(!log.contains("Info:lock"), "{log}");
}

/// A parameterless forwarder is a value: `\def\mathcal{\mathscr}` leaves the braced call `\mathcal{C}` to the
/// binding (calligraphic). The accepted residual (user ruling 2026-10-11): the unbraced `\mathcal D` runs the
/// document's definition, script, as pdflatex does — author code is not analysed.
#[test]
fn value_forwarder_keeps_braced_calls_with_the_binding() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage{amsmath,mathrsfs}\n\\def\\mathcal{\\mathscr}\n\\begin{document}\n$\\mathcal{C}\\,\\mathcal D$\n\\end{document}",
  );
  assert_element(
    &xml,
    "Math",
    &[],
    r#"<Math mode="inline" tex="\mathcal{C}\,\mathscr{D}" text="C * D" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="caligraphic" role="UNKNOWN" rpadding="1.7pt">C</XMTok><XMTok font="script" role="UNKNOWN">D</XMTok></XMApp></XMath></Math>"#,
  );
}

/// Scope: a local override ends with its group (the binding is the meaning again, `\ifx`-equal to its kept copy); a
/// global one outlives it.
#[test]
fn override_follows_its_scope() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage{amsmath}\\begin{document}\n\\makeatletter\n{\\def\\text{in}[\\text]}\\ifx\\text\\lx@locked@text BINDING\\else OVERRIDE\\fi\n{\\gdef\\text{out}}[\\text]\\ifx\\text\\lx@locked@text BINDING\\else OVERRIDE\\fi\n\\makeatother\n\\end{document}",
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">[in]BINDING[out]OVERRIDE</p>"#,
  );
}

/// The `\@for` leftover: after the loop `\text` is still the document's (latex.ltx's `\@forloop` leaves it `\@nil`),
/// and a later `$\text{b}$` still calls amsmath's `\text`.
#[test]
fn atfor_leftover_then_call() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage{amsmath}\\begin{document}\n\\makeatletter\\@for\\text:=alpha,beta\\do{[\\text]}\\makeatother\n$a\\text{b}$\n\\end{document}",
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">[alpha][beta]<Math mode="inline" tex="a\text{b}" text="a * [b]" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMText xml:id="p1.m1.1.1.3">b</XMText></XMApp></XMath></Math></p>"#,
  );
}

/// An `\edef` capture of a call keeps the binding (`\lx@locked@text`, global) past the override's group, and its
/// `tex` names `\text`.
#[test]
fn edef_capture_of_a_call_outlives_the_group() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage{amsmath}\\begin{document}\n{\\def\\text{v}\\xdef\\mycap{\\text{x}}}$a\\mycap b$\n\\end{document}",
  );
  assert_element(
    &xml,
    "Math",
    &[],
    r#"<Math mode="inline" tex="a\text{x}b" text="a * [x] * b" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMText xml:id="p1.m1.1.1.3">x</XMText><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"#,
  );
}

/// A package (File origin) is still refused, as in Perl: its `\def\text{STY}` does not become the meaning.
#[test]
fn package_redefinition_stays_refused() {
  let (log, xml) = convert_files_with(
    "\\documentclass{article}\\usepackage{amsmath}\\usepackage{mystyle}\\begin{document}\n\\def\\stytext{STY}\\ifx\\text\\stytext OVERRIDDEN\\else REFUSED\\fi{} $a\\text{b}$\n\\end{document}",
    &[("mystyle.sty", "\\def\\text{STY}\n")],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(xml.contains("REFUSED"), "{xml}");
  assert!(!log.contains("Info:lock"), "{log}");
}

/// One `Info:lock` per control sequence and document, however many iterations override it.
#[test]
fn one_lock_info_per_cs() {
  let (log, xml) = convert_clean_log(
    "\\documentclass{article}\\usepackage{amsmath}\\begin{document}\n\\makeatletter\\@for\\text:=a,b,c\\do{[\\text]}\\makeatother\n\\end{document}",
  );
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">[a][b][c]</p>"#);
  assert_eq!(log.matches("Info:lock:\\text").count(), 1, "{log}");
  assert!(!log.contains("Ignoring redefinition of \\text"), "{log}");
}

/// The derived class of every locked name under a common preamble (`state::owns_calls`, from each binding's own
/// parameter list): which own their calls, and which stay absolutely locked.
#[test]
fn locked_names_derived_classes() {
  let (log, _xml, (owners, absolute)) = convert_with_then(
    "\\documentclass{article}\\usepackage{amsmath,amssymb,natbib,url,hyperref}\\begin{document}x\\end{document}",
    Some("ar5iv.sty"),
    |_| {
      use latexml_core::state::{locked_control_sequences, lookup_meaning, owns_calls};
      let mut owners = Vec::new();
      let mut absolute = Vec::new();
      for name in locked_control_sequences() {
        let cs = latexml_core::T_CS!(name.as_str());
        let owns = lookup_meaning(&cs).is_some_and(|b| owns_calls(&cs, &b));
        if owns {
          owners.push(name)
        } else {
          absolute.push(name)
        }
      }
      (owners, absolute)
    },
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  // Own their calls: a braced first mandatory argument, outside LaTeX's `@` namespace.
  assert_eq!(owners, [
    r"\DeclareLowercaseMapping",
    r"\DeclareTextAccentDefault",
    r"\DeclareTitlecaseMapping",
    r"\DeclareUppercaseMapping",
    r"\author",
    r"\bibliography",
    r"\cite",
    r"\citep",
    r"\citet",
    r"\mathbf",
    r"\mathcal",
    r"\mathit",
    r"\mathnormal",
    r"\mathrm",
    r"\mathscr",
    r"\mathsf",
    r"\mathtt",
    r"\multicolumn",
    r"\newcounter",
    r"\overline",
    r"\sqrt",
    r"\tag",
    r"\text",
    r"\title",
    r"\underline",
  ]);
  // Absolutely locked: no braced interface (sectioning reads downstream, `\footnote`, `\url`, `\maketitle`,
  // environments, font defaults, `\def`), or machinery (`@`).
  assert_eq!(absolute, [
    r"\@@tabular",
    r"\@caption",
    r"\@definecounter",
    r"\@eqnnum",
    r"\@footnotetext",
    r"\@lbibitem",
    r"\@onecolumna",
    r"\@part",
    r"\@spart",
    r"\@startsection",
    r"\@tabular",
    r"\@textsuperscript",
    r"\@trivlist",
    r"\@twocolumna",
    r"\DeclareTextAccent",
    r"\DeclareTextCommand",
    r"\DeclareTextComposite",
    r"\DeclareTextCompositeCommand",
    r"\DeclareTextSymbol",
    r"\DeclareTextSymbolDefault",
    r"\LaTeX",
    r"\LaTeXe",
    r"\NAT@wrout",
    r"\ProvideTextCommand",
    r"\TeX",
    r"\abstract",
    r"\active@math@prime",
    r"\align",
    r"\align*",
    r"\aligned",
    r"\alignedat",
    r"\alloc@",
    r"\begin{description}",
    r"\begin{displaymath}",
    r"\begin{enumerate}",
    r"\begin{equation*}",
    r"\begin{equation}",
    r"\begin{itemize}",
    r"\bfdefault",
    r"\bibitem",
    r"\ch@ck",
    r"\def",
    r"\define@newfont",
    r"\description",
    r"\displaymath",
    r"\e@alloc",
    r"\e@ch@ck",
    r"\edef",
    r"\endalign",
    r"\endalign*",
    r"\endaligned",
    r"\endalignedat",
    r"\enddescription",
    r"\enddisplaymath",
    r"\endenumerate",
    r"\endeqnarray",
    r"\endeqnarray*",
    r"\endequation",
    r"\endequation*",
    r"\enditemize",
    r"\endsubequations",
    r"\endtabular",
    r"\endthebibliography",
    r"\end{description}",
    r"\end{displaymath}",
    r"\end{enumerate}",
    r"\end{equation*}",
    r"\end{equation}",
    r"\end{itemize}",
    r"\enumerate",
    r"\eqnarray",
    r"\eqnarray*",
    r"\equation",
    r"\equation*",
    r"\footnote",
    r"\footnotemark",
    r"\footnotetext",
    r"\gdef",
    r"\itdefault",
    r"\itemize",
    r"\lx@@tabular@args",
    r"\lx@NBSP",
    r"\lx@VerbatimFootnotes",
    r"\lx@ifundefined",
    r"\lx@maketitle@body",
    r"\lx@nat@bibitem",
    r"\maketitle",
    r"\mddefault",
    r"\newif",
    r"\paragraph",
    r"\prime",
    r"\rmdefault",
    r"\scdefault",
    r"\section",
    r"\sfdefault",
    r"\sldefault",
    r"\subequations",
    r"\subparagraph",
    r"\subsection",
    r"\subsubsection",
    r"\tabular",
    r"\thebibliography",
    r"\today",
    r"\try@load@fontshape",
    r"\ttdefault",
    r"\updefault",
    r"\url",
    r"\xdef",
  ]);
}

/// b2 (unchanged by D1): `\renewcommand\cite[1]{[\textbf{#1}]}` takes a parameter, so it stays refused and the
/// citation keeps its `bibref`.
#[test]
fn restyled_cite_keeps_its_bibref() {
  let xml = convert_clean(repro!("m5_b2_cite_restyle"));
  assert_element(
    &xml,
    "cite",
    &[],
    r#"<cite class="ltx_citemacro_cite">[<bibref bibrefs="knuth" separator="," yyseparator=","/>]</cite>"#,
  );
}

/// e1 (unchanged by D1): `\abstract` reads its argument downstream (a parameterless closure), so its lock stays
/// absolute and `\def\abstract#1{…}` leaves the `<abstract>` element.
#[test]
fn redefined_abstract_keeps_its_element() {
  let xml = convert_clean(repro!("m5_e1_abstract_def1"));
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">We study widgets.</p></abstract>"#,
  );
}

/// The value side of robust and ltcmd redefinitions (review probes b2, b6, b12, b11): a parameterless
/// `\DeclareRobustCommand`/`\RenewDocumentCommand` — also after an earlier `[1]`/`{m}` declaration, whose `\text␣` or
/// `\text code` it replaces — is a value: `[\text]` prints it, with one `Info:lock`, and `\text{c}` is the binding's.
/// ltcmd's optimized wrapper is judged by `\l__cmd_current_arg_int`, not by the `\text code` left from the previous
/// declaration (latex.ltx:1963-1983).
#[test]
fn robust_and_ltcmd_values_take_uses_without_an_argument() {
  for (name, preamble) in [
    ("b2", "\\DeclareRobustCommand\\text{V}"),
    ("b6", "\\RenewDocumentCommand\\text{}{V}"),
    (
      "b12",
      "\\DeclareRobustCommand\\text[1]{X#1}\n\\DeclareRobustCommand\\text{V}",
    ),
    (
      "b11",
      "\\RenewDocumentCommand\\text{m}{X#1}\n\\RenewDocumentCommand\\text{}{V}",
    ),
  ] {
    let (log, xml) = convert_clean_log(&format!(
      "\\documentclass{{article}}\\usepackage{{amsmath}}\n{preamble}\n\\begin{{document}}\n[\\text] $a\\text{{c}}b$\n\\end{{document}}"
    ));
    assert_eq!(log.matches("Info:lock:\\text").count(), 1, "{name}: {log}");
    assert_element(
      &xml,
      "p",
      &[],
      r#"<p xml:id="p1.1">[V] <Math mode="inline" tex="a\text{c}b" text="a * [c] * b" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMText xml:id="p1.m1.1.1.3">c</XMText><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></p>"#,
    );
  }
}
