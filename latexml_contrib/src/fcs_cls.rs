//! Stub for fcs.cls (Frontiers of Computer Science).
//!
//! fcs.cls is an expl3/xparse-heavy Springer-style class. The raw load
//! trips on the `\NewDocumentCommand \fcssetup { m }` definition body.
//! Provide a minimal stub: route most user-facing macros through
//! \\@add@frontmatter so author content (title, authors, abstract,
//! keywords, etc.) reaches the XML output.
//!
//! Witness: 2503.12978 (\\fcssetup, {acknowledgement} env); 2410.08723 (`\author*[1,2]`).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // fcs.cls loads amsmath in some versions only (1 of 6 copies in a 1/20 arXiv sample): amsmath's commands for papers
  // that use them unloaded, its "Old form" error once the paper loads it; the paper's copy decides
  // (`amsmath_sty::require_amsmath_beyond_original`, KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    &[],
  )?;
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

  // fcs.cls:268-290 `\DeclareDocumentCommand \author { s o m }`: `\author*[1,2]{Name}` is the corresponding author
  // (the star appends `\l__fcs_correspond_author_mark_tl`, an envelope `(\Letter)`, :720-723), and the option lists the
  // labels of the addresses the name requests (each printed as a `\ref` to the `\label` of `\address[1]{…}`,
  // :293-307). The kernel's `\author[]{}` read the star as the name and set `[1,2]Name` after the abstract (2410.08723:
  // a creator "*", the second author's name a paragraph). The mark is the corresponding-author contact, as acmart's
  // `\correspondingauthor` (acmart_cls.rs). Guard `perfect_kernel_batch86c::fcs_author_star_and_address_labels`.
  DefMacro!(
    "\\author OptionalMatch:* []{}",
    "\\lx@add@authors@append[annotations={#2}]{#3}\\if\\relax\\detokenize{#1}\\relax\\else\\lx@add@contact[role=corresponding]{\u{2709}}\\fi"
  );
  DefMacro!(
    "\\address[]{}",
    "\\if\\relax\\detokenize{#1}\\relax\\lx@add@address{#2}\\else\\lx@add@address[label={#1}]{#2}\\fi"
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
