//! Red/green guards for batch 79a. Item 1 (user ruling 2026-10-22, OXIDIZED_DESIGN_DIVERGENCES #662): K11's line
//! rule — a raw class's title-page store printed in one line with a value the frontmatter does not carry stays the
//! class's line (tlp.cls:627, 1209.2617) — counts only printed text as a value, and the line ends at `\par`, `\\` and
//! `\newline`: a font macro before the address (`{\@addrfont\@address\par}`, review 68x r2 probe ka) and a class line
//! the title page ends before it (`\vbox{…\@jinfo\par\@author\par\@address\par}`, probe kb) leave `\address` the
//! frontmatter's. Item 2 (side item B, review rev78z finding 1): the natural `cssstyle` of a graphic reads the
//! candidate the box is sized from, so a raster first on the candidate list sizes no `em`, and the candidates follow the
//! document's `\DeclareGraphicsExtensions` (OD #886, #883). Round 3 (review rev79a2): a head or foot definition in
//! the title page is page furniture as a running head is, a declared extension finds a name in another case as
//! kpathsea does, epstopdf's `prepend` puts `.eps` first, and only a placeholder defers a store to `\maketitle`.
//!
//! Each guard converts its pdflatex-checked repro under `tools/perfect_kernel/repros/` (its header holds the oracle)
//! and asserts the elements whole, after 0 errors and 0 warnings.
use latexml::util::test::{assert_element, convert_bytes_files_then, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/title_page_typesetting/",
      $path
    ))
  };
}

/// Convert a title-page repro with its class, 0 errors and 0 warnings.
fn convert_with_class(tex: &str, cls_name: &str, cls: &str) -> String {
  let (log, xml) = convert_files_with(tex, &[(cls_name, cls)], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// (the contact's name ends in a no-break space)
const AUTHOR_WITH_ADDRESS: &str = "<creator role=\"author\"><personname>An Author</personname><contact name=\"Address:\u{a0}\" role=\"address\">Dept of Probes, Somewhere</contact></creator>";

/// title_page_typesetting/address_after_a_font_macro: zzfma.cls's `{\@addrfont\@address\par}`, `\@addrfont` an
/// `\itshape` — a font switch prints no value, so `\address` is the frontmatter's (pdflatex: "Dept of Probes, Somewhere"
/// in italic under the author). Red (a01d8b410f, 68x r2-r3): no address contact, an italic body paragraph.
#[test]
fn address_after_a_font_macro_is_the_frontmatters() {
  let xml = convert_with_class(
    repro!("address_after_a_font_macro.tex"),
    "zzfma.cls",
    repro!("zzfma.cls"),
  );
  assert_element(&xml, "creator", &[r#"role="author""#], AUTHOR_WITH_ADDRESS);
  assert_eq!(xml.matches("Dept of Probes").count(), 1, "{xml}");
}

/// title_page_typesetting/address_in_a_vbox_with_a_class_line: zzfmb.cls's `\vbox{\centering{\LARGE\@title\par}
/// \@jinfo\par\@author\par\@address\par}` — `\par` ends the class's line "Vol. 1, 2026" before the address, which is the
/// frontmatter's; the class's line stays in the body (pdflatex: "The Title" / "Vol. 1, 2026" / "An Author" / "Dept of
/// Probes, Somewhere"). Red (a01d8b410f): no address contact, "Dept of Probes, Somewhere" a body paragraph.
#[test]
fn address_after_a_class_line_ended_by_par_is_the_frontmatters() {
  let xml = convert_with_class(
    repro!("address_in_a_vbox_with_a_class_line.tex"),
    "zzfmb.cls",
    repro!("zzfmb.cls"),
  );
  assert_element(&xml, "creator", &[r#"role="author""#], AUTHOR_WITH_ADDRESS);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para vattach=\"bottom\" xml:id=\"p1\"><p align=\"center\" xml:id=\"p1.1\">Vol.\u{a0}1, 2026</p></para>",
  );
  assert_eq!(xml.matches("Dept of Probes").count(), 1, "{xml}");
}

/// title_page_typesetting/address_after_a_line_break: zzfmc.cls's `{\@jinfo\\[2pt]\@addrfont\@address}` — `\\` (with
/// its `[<skip>]`) ends the class's line, and `\@addrfont`, the store of the class's `\addressfont` setter, holds
/// `\small\itshape`: no value, so `\address` is the frontmatter's (pdflatex: "Vol. 1, 2026" / "Dept of Probes,
/// Somewhere"). Red (a01d8b410f): no address contact.
#[test]
fn address_after_a_class_line_ended_by_a_line_break_is_the_frontmatters() {
  let xml = convert_with_class(
    repro!("address_after_a_line_break.tex"),
    "zzfmc.cls",
    repro!("zzfmc.cls"),
  );
  assert_element(&xml, "creator", &[r#"role="author""#], AUTHOR_WITH_ADDRESS);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p align=\"center\" xml:id=\"p1.1\">Vol.\u{a0}1, 2026</p></para>",
  );
  assert_eq!(xml.matches("Dept of Probes").count(), 1, "{xml}");
}

/// title_page_typesetting/value_in_the_line_stays_the_class_line, the control: zzfmd.cls's `{\@jinfo, \@address\par}`
/// prints the address in one line with the class's text "Vol. 1, 2026", so the line stays whole, as pdflatex prints it,
/// and the author has no address (a01d8b410f the same). Its `{\itshape submitted \@submitted; e-mail \@email\par}`, with
/// `\@submitted` a store the class leaves empty for a setter no table names, is judged again at `\maketitle`, where the
/// document's `\submitted{1 May 2026}` is printed text: the line stays whole too, and the e-mail is no contact (pdflatex
/// "submitted 1 May 2026; e-mail ann@probes.org"). Red (int68a, 79a r1): an e-mail contact and the body line "submitted
/// 1 May 2026; e-mail" (review rev79a finding 2, Q1).
#[test]
fn a_store_in_one_line_with_class_text_stays_the_class_line() {
  let xml = convert_with_class(
    repro!("value_in_the_line_stays_the_class_line.tex"),
    "zzfmd.cls",
    repro!("zzfmd.cls"),
  );
  assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    "<creator role=\"author\"><personname>An Author</personname></creator>",
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p align=\"center\" xml:id=\"p1.1\">Vol.\u{a0}1, 2026, Dept of Probes, Somewhere</p><p align=\"center\" xml:id=\"p1.2\"><text font=\"italic\" xml:id=\"p1.2.1\">submitted 1 May 2026; e-mail ann@probes.org</text></p></para>",
  );
  assert!(!xml.contains("<contact"), "{xml}");
}

/// title_page_typesetting/doi_glued_to_class_text: zzfme.cls's `\if\@doi\relax\else{DOI: 10.4279/PIP.\@doi}\fi`
/// (piparticle-final.cls:272) glues the class's "PIP." to the store: one printed value the frontmatter does not carry,
/// so the line stays the class's (pdflatex "DOI: 10.4279/PIP.020001"). Red (79a r1): `<pubnote name="DOI: "
/// role="doi">020001</pubnote>`, a DOI that does not exist, and the line gone (1002.3829, 1012.4507, 1503.03445,
/// 1503.07722, 1510.06950; review rev79a finding 1, Q2). Its `{\itshape\hskip.5em\@address\par}`: the letters before
/// the address are the skip's dimension (jps-cp.cls's `\hskip.5em\@kword`), so the address is the frontmatter's.
#[test]
fn a_store_glued_to_class_text_stays_the_class_line() {
  let xml = convert_with_class(
    repro!("doi_glued_to_class_text.tex"),
    "zzfme.cls",
    repro!("zzfme.cls"),
  );
  assert_element(&xml, "creator", &[r#"role="author""#], AUTHOR_WITH_ADDRESS);
  assert_eq!(xml.matches("Dept of Probes").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p align=\"center\" xml:id=\"p1.1\"><text fontsize=\"90%\" xml:id=\"p1.1.1\">DOI: 10.4279/PIP.020001</text></p></para>",
  );
  assert!(!xml.contains("<pubnote"), "{xml}");
}

/// title_page_typesetting/store_in_a_line_with_a_document_value: zzfmf.cls's `{\itshape \@issueinfo; DOI \@doi\par}`,
/// `\@issueinfo` empty when the class ends: the document's `\issueinfo{Issue 4}` is printed text when `\maketitle`
/// prints the line, so `\doi` — handed on as it is set, deferred to `\maketitle` for it — stays in the class's line
/// (pdflatex "Issue 4; DOI 10.1000/xyz"); without `\issueinfo` the DOI is the frontmatter's pubnote, the control.
/// `{\@jinfo\linebreak\@address\par}`: `\linebreak` ends the class's line, so the address is the frontmatter's. Red
/// (int68a, 79a r1): the doi pubnote beside the body line "Issue 4; DOI", and no address contact.
#[test]
fn a_store_in_one_line_with_a_document_value_stays_the_class_line() {
  let tex = repro!("store_in_a_line_with_a_document_value.tex");
  let xml = convert_with_class(tex, "zzfmf.cls", repro!("zzfmf.cls"));
  assert_element(&xml, "creator", &[r#"role="author""#], AUTHOR_WITH_ADDRESS);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p align=\"center\" xml:id=\"p1.1\">Vol.\u{a0}1, 2026</p><p align=\"center\" xml:id=\"p1.2\"><text font=\"italic\" xml:id=\"p1.2.1\">Issue 4; DOI 10.1000/xyz</text></p></para>",
  );
  assert!(!xml.contains("<pubnote"), "{xml}");
  assert_eq!(xml.matches("Dept of Probes").count(), 1, "{xml}");
  // (the control: `\@issueinfo` left empty prints no value, and the DOI is handed on at `\maketitle`)
  let control = tex.replace("\\issueinfo{Issue 4}\n", "");
  assert_ne!(control, tex);
  let xml = convert_with_class(&control, "zzfmf.cls", repro!("zzfmf.cls"));
  assert_element(
    &xml,
    "pubnote",
    &[r#"role="doi""#],
    "<pubnote name=\"DOI:\u{a0}\" role=\"doi\">10.1000/xyz</pubnote>",
  );
  assert_element(&xml, "creator", &[r#"role="author""#], AUTHOR_WITH_ADDRESS);
}

/// package-bindings/natural_cssstyle_reads_the_box_candidate: an extensionless `\includegraphics{q}` beside `q.png`
/// (200 x 100 px) and `q.eps` (40 x 20 bp) — candidates `q.png,q.eps` — is sized from the png, box and natural
/// `cssstyle` alike: a raster has no `em` size, so no `cssstyle` (pdflatex sets the png 200.7495pt; the box keeps Perl's
/// 100 DPI, OD #337). The control `m.pdf` (1cm square) before `m.png` keeps its `em` size. Red (a01d8b410f):
/// `cssstyle="width:4.015em; height:2.007em"`, the eps's size for the png post renders (review rev78z finding 1).
#[test]
fn natural_cssstyle_reads_the_candidate_the_box_is_sized_from() {
  let png = include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/trimquad.png");
  let (log, xml, ()) = convert_bytes_files_then(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/natural_cssstyle_reads_the_box_candidate.tex"
    )
    .as_bytes(),
    &[
      ("q.png", png),
      (
        "q.eps",
        include_bytes!("../../../tools/perfect_kernel/repros/package-bindings/eps_as_pdf_two.eps"),
      ),
      (
        "m.pdf",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/ext/fig.pdf"),
      ),
      ("m.png", png),
    ],
    Some(PRELOAD),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">q 144.54pt.
<graphics candidates="q.png,q.eps" graphic="q" xml:id="p1.g1"/>
m 28.4523pt.
<graphics candidates="m.pdf,m.png" cssstyle="width:2.845em; height:2.845em" graphic="m" xml:id="p1.g2"/></p>"#,
  );
}

/// package-bindings/declared_graphics_extensions_order_the_candidates: after `\DeclareGraphicsExtensions{.pdf,.eps,.png,
/// .jpg}` an extensionless `\includegraphics{q}` beside `q.png` and `q.eps` is the eps, as TeX tries `\Gin@extensions`
/// in turn (graphics.sty:206-224): candidates `q.eps,q.png`, box 40.15pt (pdflatex 40.14989pt) and the eps's natural
/// `cssstyle`. Red (a01d8b410f): `q.png,q.eps` and the png's 144.54pt box. Witness 2107.02597.
#[test]
fn declared_graphics_extensions_order_the_candidates() {
  let (log, xml, ()) = convert_bytes_files_then(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/declared_graphics_extensions_order_the_candidates.tex"
    )
    .as_bytes(),
    &[
      (
        "q.png",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/trimquad.png"),
      ),
      (
        "q.eps",
        include_bytes!("../../../tools/perfect_kernel/repros/package-bindings/eps_as_pdf_two.eps"),
      ),
    ],
    Some(PRELOAD),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">q 40.15pt.
<graphics candidates="q.eps,q.png" cssstyle="width:4.015em; height:2.007em" graphic="q" xml:id="p1.g1"/></p>"#,
  );
}

/// title_page_typesetting/short_title_in_a_running_head: zzfmg.cls's `\markboth{\@shortauthor: \@shorttitle}{…}` in its
/// `\@maketitle` prints the short title beside the document's short author in the running head, page furniture on no
/// line of the title page (`running_head_arguments`), so `\shorttitle` is the frontmatter's `<toctitle>` (preprint.cls's
/// `\ihead{\@shortauthor:~\@shorttitle}`, 2512.04983). Red in 79a r2's first build, which judged the head's line again
/// at `\maketitle` and kept the short title the class's (int68a and 79a r1 the toctitle).
#[test]
fn a_store_in_a_running_head_is_the_frontmatters() {
  let xml = convert_with_class(
    repro!("short_title_in_a_running_head.tex"),
    "zzfmg.cls",
    repro!("zzfmg.cls"),
  );
  assert_element(&xml, "toctitle", &[], "<toctitle>A Short Title</toctitle>");
}

/// title_page_typesetting/short_title_in_a_head_definition: zzfmh.cls's `\@maketitle` defines the first page's heads,
/// `{\def\@oddhead{\@shortauthor: \@shorttitle\hfil}\gdef\@evenhead{Vol.\@volume\hfil}}` (ptptex.cls's `\maketitle`
/// head) — page furniture as a running head is (`running_head_arguments`, `head_definition_body`), so the short title
/// is no value of a class line beside the short author and the volume is glued to no printed text: `\shorttitle` is
/// the frontmatter's `<toctitle>`, `\volume` its pubnote; the same with `\renewcommand{\@oddhead}{…}` and titleps's
/// `\sethead{…}{…}{…}`/`\setfoot*{…}{…}{…}`. pdflatex prints no head line on the title page. Red (79a r2, review
/// rev79a2 finding 1, probe zqk): no toctitle (`\maketitle The raw class's \shorttitle stays the class's`), no volume.
#[test]
fn a_store_in_a_head_definition_is_the_frontmatters() {
  const FRONTMATTER: &str = "<toctitle>A Short Title</toctitle>";
  // (the note's name ends in a no-break space)
  const VOLUME: &str = "<pubnote name=\"Volume:\u{a0}\" role=\"volume\">12</pubnote>";
  let cls = repro!("zzfmh.cls");
  let tex = repro!("short_title_in_a_head_definition.tex");
  let def_heads =
    "{\\def\\@oddhead{\\@shortauthor: \\@shorttitle\\hfil}\\gdef\\@evenhead{Vol.\\@volume\\hfil}}%";
  assert!(cls.contains(def_heads));
  let renewed = cls.replace(
    def_heads,
    "{\\renewcommand{\\@oddhead}{\\@shortauthor: \\@shorttitle\\hfil}\\renewcommand*\\@evenfoot{Vol.\\@volume}}%",
  );
  let titleps = cls
    .replace(
      "\\LoadClass{article}",
      "\\LoadClass{article}\n\\RequirePackage{titleps}",
    )
    .replace(
      def_heads,
      "\\sethead{\\@shortauthor}{}{\\@shorttitle}\\setfoot*{Vol.\\@volume}{}{}%",
    );
  for cls in [cls, renewed.as_str(), titleps.as_str()] {
    let xml = convert_with_class(tex, "zzfmh.cls", cls);
    assert_element(&xml, "toctitle", &[], FRONTMATTER);
    assert_element(&xml, "pubnote", &[r#"role="volume""#], VOLUME);
  }
}

/// title_page_typesetting/store_beside_a_font_macro_keeps_its_order: zzfmi.cls's `{\@journalfont\@journal\par}`,
/// `\@journalfont` a font switch, is the line's typesetting and no placeholder the document fills
/// (`may_become_a_value`), so `\journal` is handed on as the document sets it, before `\volume` (USG.cls:1590,
/// 2605.00042). Red (79a r2, review rev79a2 finding 5): `\journal` deferred to `\maketitle`, after the volume.
#[test]
fn a_store_beside_a_font_macro_keeps_its_order() {
  let xml = convert_with_class(
    repro!("store_beside_a_font_macro_keeps_its_order.tex"),
    "zzfmi.cls",
    repro!("zzfmi.cls"),
  );
  let journal = xml
    .find("<pubnote name=\"Journal:\u{a0}\" role=\"journal\">Journal of Probes</pubnote>")
    .expect("the journal pubnote");
  let volume = xml
    .find("<pubnote name=\"Volume:\u{a0}\" role=\"volume\">3</pubnote>")
    .expect("the volume pubnote");
  assert!(journal < volume, "{xml}");
}

/// Convert a graphics repro beside `q.png` (200 x 100 px) and `q.eps` (BoundingBox 40 x 20 bp), 0 errors and 0
/// warnings, and return its first paragraph's `<p>`.
fn convert_beside_png_and_eps(tex: &str) -> String { convert_beside_raster_and_eps(tex, "q.png") }

/// [`convert_beside_png_and_eps`], the png named `png`.
fn convert_beside_raster_and_eps(tex: &str, png: &str) -> String {
  let (log, xml, ()) = convert_bytes_files_then(
    tex.as_bytes(),
    &[
      (
        png,
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/trimquad.png"),
      ),
      (
        "q.eps",
        include_bytes!("../../../tools/perfect_kernel/repros/package-bindings/eps_as_pdf_two.eps"),
      ),
    ],
    Some(PRELOAD),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

macro_rules! graphics_repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $path
    ))
  };
}

/// The png's box at Perl's 100 DPI (OD #337), no natural `cssstyle`: the candidates `q.png,q.eps`.
const PNG_FIRST: &str = r#"<p xml:id="p1.1">q 144.54pt.
<graphics candidates="q.png,q.eps" graphic="q" xml:id="p1.g1"/></p>"#;

/// The eps's box (pdflatex 40.14989pt) and natural `cssstyle`: the candidates `q.eps,q.png`.
const EPS_FIRST: &str = r#"<p xml:id="p1.1">q 40.15pt.
<graphics candidates="q.eps,q.png" cssstyle="width:4.015em; height:2.007em" graphic="q" xml:id="p1.g1"/></p>"#;

/// package-bindings/append_graphics_extensions_keeps_the_driver_order: grfext's `\AppendGraphicsExtensions{.eps}`
/// appends to the driver's `\Gin@extensions` (pdftex.def:597-600, which graphics_sty.rs seeds), so `q` is the png
/// pdflatex prints (200.7495pt). Red (79a r1): the list started `\@empty`, `.eps` alone ranked first, the eps's box
/// (review rev79a finding 3; 1711.06128's `\AppendGraphicsExtensions{.gif}`).
#[test]
fn append_graphics_extensions_keeps_the_driver_order() {
  let xml = convert_beside_png_and_eps(graphics_repro!(
    "append_graphics_extensions_keeps_the_driver_order.tex"
  ));
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], PNG_FIRST);
}

/// package-bindings/prepend_graphics_extensions_puts_the_type_first: after `\usepackage{epstopdf}`,
/// `\PrependGraphicsExtensions{.eps}` puts `.eps` before the driver's list, so `q` is the eps (pdflatex 40.14989pt).
/// Red (int68a): the png's box, the candidates in pdftex.def's order.
#[test]
fn prepend_graphics_extensions_puts_the_type_first() {
  let xml = convert_beside_png_and_eps(graphics_repro!(
    "prepend_graphics_extensions_puts_the_type_first.tex"
  ));
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], EPS_FIRST);
}

/// package-bindings/epstopdf_appends_eps_last: `\DeclareGraphicsExtensions{.pdf,.eps,.png}` before
/// `\usepackage{epstopdf}`, whose epstopdf-base appends `.eps` (epstopdf-base.sty:498-505: every `.eps` removed, one
/// appended), tries `.png` before `.eps` — and so does a `\PrependGraphicsExtensions{.eps}` in the preamble with grfext
/// alone, which the driver's begin-document load of epstopdf-base undoes (pdftex.def:681-701): the png, as pdflatex
/// prints it (200.7495pt). Red (79a r1): the eps.
#[test]
fn epstopdf_appends_eps_last() {
  let tex = graphics_repro!("epstopdf_appends_eps_last.tex");
  let xml = convert_beside_png_and_eps(tex);
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], PNG_FIRST);
  let grfext = tex.replace(
    "\\DeclareGraphicsExtensions{.pdf,.eps,.png}\n\\usepackage{epstopdf}",
    "\\usepackage{grfext}\n\\RemoveGraphicsExtensions{.eps}\\PrependGraphicsExtensions{.eps}",
  );
  assert_ne!(grfext, tex);
  let xml = convert_beside_png_and_eps(&grfext);
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], PNG_FIRST);
}

/// package-bindings/declared_graphics_extensions_fold_case: after `\DeclareGraphicsExtensions{.pdf,.jpeg,.png}` an
/// extensionless `\includegraphics{q}` beside `q.PNG` and `q.eps` is `q.PNG`, which kpathsea finds for the `.png` entry
/// in another case (TL's `texmf_casefold_search=1`) before the `.eps` pdftex.def appends at `\begin{document}`; and with
/// `\RemoveGraphicsExtensions{.png}` beside `q.png`, pdftex.def's `.PNG` entry finds the png (pdflatex 200.7495pt both;
/// `declared_rank`). Red (79a r2, review rev79a2 finding 2, probe gc): the eps (1608.05172's `img/nsfnet.eps`, where
/// its pdflatex log reads `<./img/nsfnet.PNG>`).
#[test]
fn declared_graphics_extensions_fold_case() {
  let tex = graphics_repro!("declared_graphics_extensions_fold_case.tex");
  let xml = convert_beside_raster_and_eps(tex, "q.PNG");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">q 144.54pt.
<graphics candidates="q.PNG,q.eps" graphic="q" xml:id="p1.g1"/></p>"#,
  );
  let removed = tex.replace(
    "\\usepackage{graphicx}\n\\DeclareGraphicsExtensions{.pdf,.jpeg,.png}",
    "\\usepackage{graphicx}\\usepackage{grfext}\\RemoveGraphicsExtensions{.png}",
  );
  assert_ne!(removed, tex);
  let xml = convert_beside_png_and_eps(&removed);
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], PNG_FIRST);
}

/// package-bindings/epstopdf_prepend_puts_eps_first: `\usepackage[prepend]{epstopdf}` makes epstopdf-base's load
/// `\PrependGraphicsExtensions{.eps}` (epstopdf-base.sty:498-505), and `\epstopdfsetup{prepend}` after the load is the
/// same (:509-515), so `q` beside `q.png` and `q.eps` is the eps (pdflatex 40.14989pt both). Red (79a r2, review
/// rev79a2 finding 4, probes ga, gd): the option and the key not read, the png.
#[test]
fn epstopdf_prepend_puts_eps_first() {
  let tex = graphics_repro!("epstopdf_prepend_puts_eps_first.tex");
  let xml = convert_beside_png_and_eps(tex);
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], EPS_FIRST);
  let setup = tex.replace(
    "\\usepackage[prepend]{epstopdf}",
    "\\usepackage{epstopdf}\\epstopdfsetup{update,prepend}",
  );
  assert_ne!(setup, tex);
  let xml = convert_beside_png_and_eps(&setup);
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], EPS_FIRST);
}
