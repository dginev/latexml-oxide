//! Red/green guards for batch 78h: "a copy supplies only the text names its binding declares" (user ruling
//! 2026-10-21; OXIDIZED_DESIGN_DIVERGENCES #731 rules 13 and 14). The declared text names are the kernel's default,
//! the standard `\…name` macros (`state::COPY_TEXT_NAMES`), and the names a binding adds (`state::declare_copy_text_names`:
//! the aas_macros binding's journal abbreviations). A copy its `\Provides` line routed to a binding defines them as the
//! real class does, when its definition reads as text (the safety check, which fails closed); everything else stays the
//! binding's — its caption formats (`\fnum@<type>`) and separator, `\@thmcountersep`, list labels, structure. A copy
//! under the binding's own name supplies none (rule 1). Rule 13's standard-class derivatives `\newcommand` the declared
//! names only. Witnesses 1402.7103 (proto.cls, `\ProvidesClass{aastex}`), 1106.2565 (the doubled caption separator of
//! r1), 1803.06644 (r1's "Theorem 51"). Repros `tools/perfect_kernel/repros/stand-in-copies/provides_route/`.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/provides_route/",
      $path
    ))
  };
}

/// 1402.7103's shape: a renamed AASTeX 5.2 class (`\ProvidesClass{aastex}`, routed to the aastex binding). Its
/// `\figurename` "Fig." stands by rule 1's text default — a name article assigned, which the aastex binding does not
/// own —, not by its declaration, in the binding's caption label "Fig. 1" with the binding's ": " — not its own
/// `\fnum@figure` "Fig. 1.—" before the binding's separator, which r1 printed ("Fig. 1.—: ", 1106.2565's "Fig.
/// 1.—.— "); its `\abstractname` "ABSTRACT" and its journal names (declared by the aas_macros binding) stand; its
/// `\aj`, a paragraph, fails the safety check and keeps the binding's "AJ"; its `\fnum@table`, `\@thmcountersep` (r1:
/// "Theorem 11"), `\thefigure@ID` (the figure's id `S1.F1`, not "ZZ1"), `\labelitemi` (r1: the star), `\appendix` and
/// `\acknowledgments` stay the binding's.
#[test]
fn routed_copy_supplies_only_declared_text_names() {
  let (log, xml) = convert_files_with(
    repro!("routed_copy_text_names.tex"),
    &[("aascopy.cls", repro!("aascopy.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:fallback:aascopy The paper's aascopy.cls declares itself aastex: loading the aastex.cls binding, then the \
       copy as its stand-in's"
    ),
    "{log}"
  );
  assert!(
    log.contains(
      "Info:provide:\\aj \\aj keeps LaTeXML's definition: the paper's aascopy.cls defines this text name as more than \
       text (\\par)"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    "<abstract inlist=\"toc\" name=\"ABSTRACT\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">An \
     abstract.</p></abstract>",
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S1.F1\""],
    "<figure inlist=\"lof\" placement=\"h\" xml:id=\"S1.F1\"><tags><tag>Fig. 1</tag><tag role=\"autoref\">Figure\u{a0}1<text \
     xml:id=\"S1.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Fig. 1</tag></tags><toccaption><tag \
     close=\" \">1</tag>A figure.</toccaption><caption><tag close=\": \">Fig. 1</tag>A figure.</caption></figure>",
  );
  assert!(
    xml.contains("<caption><tag close=\": \">Table 1</tag>A table.</caption>"),
    "{xml}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S1.p2.1\""],
    "<p xml:id=\"S1.p2.1\">Journals: <text font=\"italic\" xml:id=\"S1.p2.1.1\">Astrophys. J.</text>; \
     Applied\u{a0}Optics.</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S1.p3.1\""],
    "<p xml:id=\"S1.p3.1\">Also: AJ.</p>",
  );
  // (the binding's: the theorem's number, the item label, the structure)
  assert_element(
    &xml,
    "theorem",
    &["xml:id=\"S1.Thmthm1\""],
    "<theorem class=\"ltx_theorem_thm\" inlist=\"thm theorem:thm\" xml:id=\"S1.Thmthm1\"><tags><tag>Theorem \
     1.1</tag><tag role=\"autoref\">1.1<text xml:id=\"S1.Thmthm1.1\"/></tag><tag role=\"refnum\">1.1</tag><tag \
     role=\"typerefnum\">Theorem 1.1</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" \
     xml:id=\"S1.Thmthm1.2\">Theorem 1.1</text></tag></title><para xml:id=\"S1.Thmthm1.p1\"><p \
     xml:id=\"S1.Thmthm1.p1.1\"><text font=\"italic\" xml:id=\"S1.Thmthm1.p1.1.1\">A theorem.</text></p></para></theorem>",
  );
  assert_element(
    &xml,
    "itemize",
    &["xml:id=\"S1.I1\""],
    "<itemize xml:id=\"S1.I1\"><item xml:id=\"S1.I1.i1\"><tags><tag>\u{2022}</tag><tag role=\"autoref\">item\u{a0}<text \
     xml:id=\"S1.I1.i1.1\"/></tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S1.I1.i1.p1\"><p \
     xml:id=\"S1.I1.i1.p1.1\">An item.</p></para></item></itemize>",
  );
  assert_element(
    &xml,
    "acknowledgements",
    &[],
    "<acknowledgements inlist=\"toc\" xml:id=\"S1.acknowledgements1\">Thanks.\n</acknowledgements>",
  );
  assert_element(
    &xml,
    "appendix",
    &["inlist=\"toc\"", "xml:id=\"A1\""],
    "<appendix inlist=\"toc\" xml:id=\"A1\"><tags><tag>Appendix A</tag><tag \
     role=\"autoref\">Appendix\u{a0}A<text xml:id=\"A1.1\"/></tag><tag role=\"refnum\">A</tag><tag \
     role=\"typerefnum\">Appendix A</tag></tags><title><tag close=\" \">Appendix A</tag>Proofs</title><toctitle><tag \
     close=\" \">A</tag>Proofs</toctitle><para xml:id=\"A1.p1\"><p xml:id=\"A1.p1.1\">Appendix \
     text.</p></para></appendix>",
  );
  assert!(xml.contains("Text EXTRA."), "{xml}");
}

/// The safety check fails closed (78h r3; review probe 6 of r2): a declared text name the routed copy defines through a
/// robust command whose inner macro makes a paragraph (`\DeclareRobustCommand\myhead{\par\noindent HEADX}`,
/// `\def\abstractname{\myhead}`: r2 wrote `name="\par\noindentHEADX"`), through a macro whose expansion is LaTeXML's
/// code (`\begin{center}`: r2 split the paragraph), through `\csname` or with a `\par` keeps the binding's — "Abstract",
/// "Phys. Rev. A", "Phys. Rev. B", "Phys. Rev. D" —, one Info each naming the refused token, though the class-end hooks
/// run the check again (r2 logged `\prd`'s three times: review probe 9); a text one (`\prc`) stands.
#[test]
fn routed_copy_text_name_check_fails_closed() {
  let (log, xml) = convert_files_with(
    repro!("routed_copy_text_names_fail_closed.tex"),
    &[("aasrobust.cls", repro!("aasrobust.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (name, refused) in [
    ("abstractname", "\\par"),
    ("pra", "\\begin"),
    ("prb", "\\csname"),
    ("prd", "\\par"),
  ] {
    let info = format!(
      "Info:provide:\\{name} \\{name} keeps LaTeXML's definition: the paper's aasrobust.cls defines this text name as \
       more than text ({refused})\n"
    );
    assert_eq!(log.matches(&info).count(), 1, "{info}\n{log}");
  }
  assert!(
    log.contains(
      "Info:provide:\\prc \\prc takes the paper's aasrobust.cls definition: a text name its binding declares"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">Abs.</p></abstract>",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"S1.p1\""],
    "<para xml:id=\"S1.p1\"><p xml:id=\"S1.p1.1\">J: Phys.\u{a0}Rev.\u{a0}A; Phys.\u{a0}Rev.\u{a0}B; PRCX; \
     Phys.\u{a0}Rev.\u{a0}D.</p></para>",
  );
}

/// A copy under the binding's own name (a paper's aas_macros.sty, rule 0) supplies no declared text name: its `\apj`
/// keeps the aas_macros binding's "ApJ" (rule 1), though the binding declares it; its `\figurename` is article's text
/// default, which the binding does not own, and stands as before (rule 1).
#[test]
fn self_named_copy_supplies_no_text_names() {
  let (log, xml) = convert_files_with(
    repro!("self_named_copy_text_names.tex"),
    &[("aas_macros.sty", repro!("aas_macros.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("a text name its binding declares"), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Journal: ApJ.</p>",
  );
  assert!(
    xml.contains("<caption><tag close=\": \">FIG. 1</tag>A figure.</caption>"),
    "{xml}"
  );
}

/// Rule 13 with the declared list: a standard-class derivative's `\newcommand\figurename{FIGURE}` defines the declared
/// name, "FIGURE 1"; its `\newcommand\fnum@table{TABLE~\thetable}` (stdderiv.cls) is a caption format, the binding's
/// "Table 1" ("Ignoring redefinition", as at int47a; r1 let it yield); its `\newcommand\appendix` stays refused (the
/// binding's `<appendix>`).
#[test]
fn standard_class_derivative_newcommands_only_declared_text_names() {
  let (log, xml) = convert_files_with(
    repro!("standard_class_derivative_text_names.tex"),
    &[("stdderiv.cls", repro!("stdderiv.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("Ignoring redefinition (\\newcommand) of T_CS[\\fnum@table]"),
    "{log}"
  );
  assert!(
    xml.contains("<caption><tag close=\": \">FIGURE\u{a0}1</tag>A figure.</caption>"),
    "{xml}"
  );
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\": \">FIGURE\u{a0}1</tag>A figure.</caption>",
  );
  assert!(
    xml.contains("<caption><tag close=\": \">Table 1</tag>A table.</caption>"),
    "{xml}"
  );
  assert_element(
    &xml,
    "appendix",
    &["inlist=\"toc\"", "xml:id=\"A1\""],
    "<appendix inlist=\"toc\" xml:id=\"A1\"><tags><tag>Appendix A</tag><tag role=\"refnum\">A</tag><tag \
     role=\"typerefnum\">Appendix A</tag></tags><title><tag close=\" \">Appendix A</tag>Proofs</title><toctitle><tag \
     close=\" \">A</tag>Proofs</toctitle><para xml:id=\"A1.p1\"><p xml:id=\"A1.p1.1\">Appendix \
     text.</p></para></appendix>",
  );
}
