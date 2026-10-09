//! Stub for fcs.cls (Frontiers of Computer Science).
//!
//! fcs.cls is an expl3/xparse-heavy Springer-style class. The raw load
//! trips on the `\NewDocumentCommand \fcssetup { m }` definition body.
//! Provide a minimal stub: route most user-facing macros through
//! \\@add@frontmatter so author content (title, authors, abstract,
//! keywords, etc.) reaches the XML output.
//!
//! Witness: 2503.12978 (\\fcssetup, {acknowledgement} env).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");
  RequirePackage!("booktabs");
  RequirePackage!("array");
  RequirePackage!("multirow");
  RequirePackage!("caption");

  // \fcssetup{key=value, ...} — main metadata block. The keys
  // (title, author, address, abstract, keywords) are user-content;
  // routing the whole arg as a ltx:note is the simplest preservation.
  DefMacro!(
    "\\fcssetup{}",
    "\\@add@frontmatter{ltx:note}[role=fcssetup]{#1}"
  );

  // {acknowledgement} env — render as acknowledgements with
  // internal_vertical mode so multi-paragraph body is accepted.
  DefEnvironment!("{acknowledgement}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");

  // {compactenum} env — like enumerate but compact.
  DefEnvironment!("{compactenum}", "<ltx:enumerate>#body</ltx:enumerate>");
  DefEnvironment!("{compactitem}", "<ltx:itemize>#body</ltx:itemize>");

  // Chinese-typesetting helpers — gobble (visual only).
  def_macro_noop("\\zihang[]{}")?;

  // `\begin{biography}{photo/a.jpg}` (fcs.cls:323/337, laid out at :637-660): the author's photo, 2.5cm wide (:684),
  // beside the text; shaped as IEEEtran's biography (ieeetran_cls.rs). Undefined before, its photo path read as text
  // (2504.14891: `photo/Aoran_Gan.jpg`, its `_` an error).
  DefMacro!(
    T_CS!("\\begin{biography}"),
    "{}",
    "\\begin{lx@fcs@biography}{\\includegraphics[width=2.5cm]{#1}}"
  );
  DefMacro!(T_CS!("\\end{biography}"), None, "\\end{lx@fcs@biography}");
  DefEnvironment!(
    "{lx@fcs@biography}{}",
    "<ltx:float class='biography'>\
      <ltx:tabular>\
        <ltx:tr>\
          <ltx:td>#1</ltx:td>\
          <ltx:td><ltx:inline-block>#body</ltx:inline-block></ltx:td>\
        </ltx:tr>\
      </ltx:tabular>\
    </ltx:float>"
  );
  // `{competinginterest}` (fcs.cls:570-588): a paragraph led by a bold "Competing interests".
  DefMacro!(
    T_CS!("\\begin{competinginterest}"),
    None,
    "\\par\\noindent{\\bfseries Competing interests}\\quad"
  );
  DefMacro!(T_CS!("\\end{competinginterest}"), None, "\\par");
});
