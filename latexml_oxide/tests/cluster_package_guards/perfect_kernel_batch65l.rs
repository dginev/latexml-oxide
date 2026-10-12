//! Red/green guards for batch 65l, the MnSymbol binding's symbol coverage: run 340's 13,470 MnSymbol papers met a
//! binding of four names (3,724 ended in Error, their first errors `\nin` 175, `\udots` 173, `\sumint` 165, `\cupdot`
//! 130, `\strokedint` 113, `\lsem` 108). The binding now declares every name of MnSymbol.sty
//! (OXIDIZED_DESIGN_DIVERGENCES #545). Repros under `tools/perfect_kernel/repros/class-bindings/mnsymbol_*`.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// 65l: the names of run 340's first errors are math symbols of their class — a name sharing a defined name's slot
/// is that name (`\nin` is `\notin`), and the roles are the sibling bindings' (RELOP, ARROW, ADDOP, MULOP, ID, INTOP,
/// SUMOP). Witnesses 1012.3905, 2012.10201 (`\nin`), 2608.11129, 2408.06194 (`\sumint`), 1505.03695 (`\cupdot`),
/// 2502.09381 (`\udots`), 2010.00736 (`\medstar`), 1704.05595 (`\leftmapsto`), 2311.12218 (`\bigcupdot`), 2604.25507
/// (`\upmodels`, ⫫ DOUBLE UP TACK). Repro class-bindings/mnsymbol_symbols_are_defined.
#[test]
fn mnsymbol_symbols_are_defined() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/mnsymbol_symbols_are_defined.tex"
  ));
  for (id, element) in [
    (
      "p1.m1.1.1",
      "<XMApp xml:id=\"p1.m1.1.1\"><XMTok meaning=\"not-element-of\" name=\"notin\" role=\"RELOP\">\u{2209}</XMTok><XMTok \
       font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">B</XMTok></XMApp>",
    ),
    (
      "p1.m2.1.1",
      "<XMApp xml:id=\"p1.m2.1.1\"><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">x</XMTok><XMTok name=\"udots\" role=\"ID\">\u{22F0}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">y</XMTok></XMApp>",
    ),
    (
      "p1.m3.1.1",
      "<XMApp xml:id=\"p1.m3.1.1\"><XMApp xml:id=\"p1.m3.1.1.1\"><XMTok role=\"SUBSCRIPTOP\" \
       scriptpos=\"post1\"/><XMTok mathstyle=\"text\" name=\"sumint\" role=\"INTOP\">\u{2A0B}</XMTok><XMTok \
       font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">i</XMTok></XMApp><XMTok font=\"italic\" \
       role=\"UNKNOWN\">f</XMTok></XMApp>",
    ),
    (
      "p1.m4.1.1",
      "<XMApp xml:id=\"p1.m4.1.1\"><XMTok name=\"cupdot\" role=\"ADDOP\">\u{228D}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">A</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">B</XMTok></XMApp>",
    ),
    (
      "p1.m9.1.1.1",
      "<XMApp xml:id=\"p1.m9.1.1.1\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/><XMTok mathstyle=\"text\" \
       name=\"bigcupdot\" role=\"SUMOP\" scriptpos=\"post\">\u{2A03}</XMTok><XMTok font=\"italic\" fontsize=\"70%\" \
       role=\"UNKNOWN\">i</XMTok></XMApp>",
    ),
    (
      "p1.m10.1.1",
      "<XMApp xml:id=\"p1.m10.1.1\"><XMTok name=\"leftmapsto\" role=\"ARROW\">\u{21A4}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp>",
    ),
    (
      "p1.m12.1.1",
      "<XMApp xml:id=\"p1.m12.1.1\"><XMTok name=\"upmodels\" role=\"RELOP\">\u{2AEB}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp>",
    ),
    (
      "p1.m13.3.1.2",
      "<XMWrap xml:id=\"p1.m13.3.1.2\"><XMTok name=\"medstar\" role=\"MULOP\" \
       xml:id=\"p1.m13.1\">\u{2606}</XMTok><XMTok role=\"PUNCT\">\u{2063}</XMTok><XMTok name=\"filledstar\" \
       role=\"MULOP\" xml:id=\"p1.m13.2\">\u{2605}</XMTok></XMWrap>",
    ),
  ] {
    let tag = element[1..].split([' ', '>']).next().unwrap();
    assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
}

/// 65l: every name the binding declares from MnSymbol.sty (the 1,307 `\Decl@Mn@Char`/`\Decl@Mn@Alias` names but
/// the internal `\@cdots`/`\@vdots`, 136 of them the `symbols` font's sized operators, and the 55
/// `\Decl@Mn@Op`/`\Decl@Mn@Int` operators) converts in math, without an error.
#[test]
fn mnsymbol_declares_every_name() {
  let names: Vec<&str> = latexml_contrib::mnsymbol_sty::declared_names().collect();
  assert_eq!(names.len(), 1362);
  let body: String = names.iter().map(|name| format!("${name}$\n")).collect();
  let tex = format!(
    "\\documentclass{{article}}\n\\usepackage{{MnSymbol}}\n\\begin{{document}}\n{body}\\end{{document}}\n"
  );
  let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert_eq!(xml.matches("<Math ").count(), names.len(), "{xml}");
}

/// 65l: MnSymbol's delimiters fence a formula after `\left`/`\right` (a math symbol is a delimiter to the delimiter
/// scan, OXIDIZED_DESIGN_DIVERGENCES #530). Witnesses 2112.14729, 2410.20575. Repro
/// class-bindings/mnsymbol_delimiters_after_left.
#[test]
fn mnsymbol_delimiters_fence_after_left() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/mnsymbol_delimiters_after_left.tex"
  ));
  assert_element(
    &xml,
    "XMWrap",
    &["xml:id=\"p1.m1.2.1.2\""],
    "<XMWrap xml:id=\"p1.m1.2.1.2\"><XMTok name=\"lsem\" role=\"OPEN\" stretchy=\"true\">\u{27E6}</XMTok><XMApp \
     xml:id=\"p1.m1.1\"><XMTok mathstyle=\"text\" meaning=\"divide\" role=\"FRACOP\"/><XMTok font=\"italic\" \
     fontsize=\"70%\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" fontsize=\"70%\" \
     role=\"UNKNOWN\">b</XMTok></XMApp><XMTok name=\"rsem\" role=\"CLOSE\" stretchy=\"true\">\u{27E7}</XMTok></XMWrap>",
  );
  assert_element(
    &xml,
    "XMDual",
    &["xml:id=\"p1.m2.2.1\""],
    "<XMDual xml:id=\"p1.m2.2.1\"><XMApp xml:id=\"p1.m2.2.1.1\"><XMTok meaning=\"delimited-\u{2989}\u{298A}\"/><XMRef \
     idref=\"p1.m2.1\" xml:id=\"p1.m2.2.1.1.2\"/></XMApp><XMWrap xml:id=\"p1.m2.2.1.2\"><XMTok name=\"langlebar\" \
     role=\"OPEN\" stretchy=\"true\">\u{2989}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" \
     xml:id=\"p1.m2.1\">x</XMTok><XMTok name=\"ranglebar\" role=\"CLOSE\" \
     stretchy=\"true\">\u{298A}</XMTok></XMWrap></XMDual>",
  );
}

/// 65l: a document that undefines `\vec` so that MnSymbol may declare it (MnSymbol.sty:1848 `\DeclareMathAccent`)
/// gets the accent back. Witness 2210.15345. Repro class-bindings/mnsymbol_vec_after_undefining.
#[test]
fn mnsymbol_declares_an_undefined_vec() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/mnsymbol_vec_after_undefining.tex"
  ));
  assert_element(
    &xml,
    "XMApp",
    &["xml:id=\"p1.m1.1.1.3\""],
    "<XMApp xml:id=\"p1.m1.1.1.3\"><XMTok name=\"vec\" role=\"OVERACCENT\" stretchy=\"false\">\u{2192}</XMTok><XMTok \
     meaning=\"0\" role=\"NUMBER\">0</XMTok></XMApp>",
  );
}

/// 65l: MnSymbol's own declarators, used by a document after loading the package, keep the binding's definition of
/// a name it already declares (`\Decl@Mn@Open{\lsem}{MNlargesymbols}{'102}`). Witness 2410.20575. Repro
/// class-bindings/mnsymbol_own_declarator_after_load.
#[test]
fn mnsymbol_own_declarators_keep_the_binding() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/mnsymbol_own_declarator_after_load.tex"
  ));
  assert_element(
    &xml,
    "XMWrap",
    &["xml:id=\"p1.m1.2.1.2\""],
    "<XMWrap xml:id=\"p1.m1.2.1.2\"><XMTok name=\"lsem\" role=\"OPEN\" stretchy=\"false\">\u{27E6}</XMTok><XMTok \
     font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.1\">m</XMTok><XMTok name=\"rsem\" role=\"CLOSE\" \
     stretchy=\"false\">\u{27E7}</XMTok></XMWrap>",
  );
}

/// 65l round 2: the glyphs MnSymbol.pdf draws — the footlines' bar where the name points (`\rightfootline` ⟞,
/// `\upfootline` ⟙), `\upmodels` ⫫ and `\downmodels` ⫪ (2604.25507) — and a name a document declares with the
/// package's own declarators takes the glyph the binding's table gives its slot: `\Decl@Mn@Char` at MnSyD slot 65 is
/// `\gtr`'s > (TeX4ht's `&gt;` decoded), `\Decl@Mn@Open` at `largesymbols` '102 `\lsem`'s ⟦; a slot the table does not
/// map is declared by the kernel, with an Info (OXIDIZED_DESIGN_DIVERGENCES #545). `\Decl@Mn@Op` keeps amsmath's
/// `\slimits@` (`\displaylimits`): limits under the operator in display. Repro
/// class-bindings/mnsymbol_declarators_take_slot_glyphs.
#[test]
fn mnsymbol_declarators_take_slot_glyphs() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/mnsymbol_declarators_take_slot_glyphs.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    log.matches("Info:unmapped:MnSymbol.sty").count(),
    1,
    "{log}"
  );
  assert!(
    log.contains(
      "Info:unmapped:MnSymbol.sty \\myord: MnSymbol has no Unicode glyph for slot 12 of the symbol font MnSyQ; \
       declared as a generic symbol."
    ),
    "{log}"
  );
  for (id, element) in [
    (
      "p1.m1.1.1",
      "<XMApp xml:id=\"p1.m1.1.1\"><XMTok meaning=\"multirelation\"/><XMTok font=\"italic\" \
       role=\"UNKNOWN\">a</XMTok><XMTok name=\"rightfootline\" role=\"RELOP\">\u{27DE}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">b</XMTok><XMTok name=\"leftfootline\" role=\"RELOP\">\u{27DD}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">c</XMTok><XMTok name=\"upfootline\" role=\"RELOP\">\u{27D9}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">d</XMTok><XMTok name=\"downfootline\" role=\"RELOP\">\u{27D8}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">e</XMTok></XMApp>",
    ),
    (
      "p2.m1.1.1",
      "<XMApp xml:id=\"p2.m1.1.1\"><XMTok meaning=\"multirelation\"/><XMTok font=\"italic\" \
       role=\"UNKNOWN\">a</XMTok><XMTok name=\"upmodels\" role=\"RELOP\">\u{2AEB}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">b</XMTok><XMTok name=\"downmodels\" role=\"RELOP\">\u{2AEA}</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">c</XMTok></XMApp>",
    ),
    (
      "p3.m1.1.1",
      "<XMApp xml:id=\"p3.m1.1.1\"><XMTok name=\"mygtr\" role=\"RELOP\">&gt;</XMTok><XMTok font=\"italic\" \
       role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp>",
    ),
    (
      "p3.m2.2.1.2",
      "<XMWrap xml:id=\"p3.m2.2.1.2\"><XMTok name=\"mylsem\" role=\"OPEN\" stretchy=\"true\">\u{27E6}</XMTok><XMTok \
       font=\"italic\" role=\"UNKNOWN\" xml:id=\"p3.m2.1\">x</XMTok><XMTok name=\"myrsem\" role=\"CLOSE\" \
       stretchy=\"true\">\u{27E7}</XMTok></XMWrap>",
    ),
    (
      "S0.Ex1.m1.1.1.1",
      "<XMApp xml:id=\"S0.Ex1.m1.1.1.1\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"mid1\"/><XMTok mathstyle=\"text\" \
       name=\"tsum\" role=\"SUMOP\" scriptpos=\"mid1\">\u{2211}</XMTok><XMTok font=\"italic\" fontsize=\"70%\" \
       role=\"UNKNOWN\">i</XMTok></XMApp>",
    ),
  ] {
    let tag = element[1..].split([' ', '>']).next().unwrap();
    assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
}

/// 65l round-2 review: the negated stacks keep their negation (⪋̸/⪌̸ via U+0338, not the plain ⪋/⪌ of
/// `\lesseqqgtr`), and a document-declared `\mathord` takes the kernel `\DeclareMathSymbol`'s `ID` role.
#[test]
fn mnsymbol_negated_stacks_and_declared_mathord() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage{MnSymbol}\\makeatletter\\Set@Mn@Sym{65}\
     \\Decl@Mn@Char\\mydec{MnSyD}{\\mathord}\\makeatother\\begin{document}\
     $\\lessneqqgtr$ $\\gtrneqqless$ $\\mydec$\\end{document}",
  );
  assert_element(
    &xml,
    "XMTok",
    &["name=\"lessneqqgtr\""],
    "<XMTok name=\"lessneqqgtr\" role=\"RELOP\">\u{2A8B}\u{338}</XMTok>",
  );
  assert_element(
    &xml,
    "XMTok",
    &["name=\"gtrneqqless\""],
    "<XMTok name=\"gtrneqqless\" role=\"RELOP\">\u{2A8C}\u{338}</XMTok>",
  );
  assert_element(
    &xml,
    "XMTok",
    &["name=\"mydec\""],
    "<XMTok name=\"mydec\" role=\"ID\">&gt;</XMTok>",
  );
}
