//! Red/green guards for batch 78v (open review rows): a marked affiliation list's address run after a trailing comma
//! (a panic), `[\numexpr…\relax]` and `[ \numexpr…]` footnote marks, a `\protect\thanks` and a note macro that prints
//! text inside an author's mark, caption's float.sty styles and typed names, newfloat's `\SetupFloatingEnvironment`,
//! epsf's reset scope through `\input epsf.sty` and psfrag's epsf.sty, an `.eps` shipped only as `.pdf`. Each guard converts its repro under
//! `tools/perfect_kernel/repros/` (pdflatex-checked) and asserts the diagnostics by count and whole elements.
use latexml::util::test::{
  assert_element, convert_files_with, convert_with, normalize_markup, xml_element,
};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors and fatals, 0 warnings.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// Every `<tag>` element of `xml` in document order, whole.
fn all_elements(xml: &str, tag: &str) -> Vec<String> {
  let mut found = Vec::new();
  let mut rest = xml;
  let open = format!("<{tag}");
  while let Some(element) = xml_element(rest, tag, &[]) {
    found.push(element);
    let at = rest.find(&open).unwrap();
    rest = &rest[at + 1..];
  }
  found
}

/// Every `<creator …>` element in document order, whole, against `expected`, after 0 errors and 0 warnings.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&str]) {
  let xml = convert_clean(tex);
  let expected: Vec<String> = expected.iter().map(|e| normalize_markup(e)).collect();
  assert_eq!(all_elements(&xml, "creator"), expected, "{xml}");
}

/// Every `<caption>` element in document order, whole, against `expected`, after 0 errors and 0 warnings.
#[track_caller]
fn assert_captions(tex: &str, expected: &[&str]) {
  let xml = convert_clean(tex);
  let expected: Vec<String> = expected.iter().map(|e| normalize_markup(e)).collect();
  assert_eq!(all_elements(&xml, "caption"), expected, "{xml}");
}

/// An address run after an affiliation that ends in a comma (`\IEEEauthorrefmark{1}Univ A,\\ \{ann, bob\}@a.edu;\\`)
/// continues it, as it prints: "Univ A," then the addresses. Red (int61a): a panic, "range end index 11 out of range
/// for slice of length 10" (the entry trimmed before the run was spliced at its former length). Witness 2509.13772.
/// Repro sectioning-frontmatter/affiliation_address_run_after_a_trailing_comma.
#[test]
fn affiliation_address_run_after_a_trailing_comma() {
  assert_creators(
    repro!("sectioning-frontmatter/affiliation_address_run_after_a_trailing_comma.tex"),
    &[
      r#"<creator role="author"><personname>Ann Alpha<sup xml:id="id1">1</sup></personname><contact name="Affiliation: " role="affiliation">Univ A,<break/>{ann, bob}@a.edu</contact></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Beta<sup xml:id="id2">1</sup></personname><contact name="Affiliation: " role="affiliation">Univ A,<break/>{ann, bob}@a.edu</contact></creator>"#,
      r#"<creator before="  " role="author"><personname>Cy Gamma<sup xml:id="id3">2</sup></personname><contact name="Affiliation: " role="affiliation">Univ B, cy@b.edu</contact></creator>"#,
    ],
  );
}

/// latex.ltx:17678 `\c@footnote #1\relax` reads `[\numexpr\value{footnote}-1\relax]` and `[ \numexpr…]` as TeX's
/// scan_int does (tex.web §440-§445): the numbers 1 and 2, each mark paired with its text. Red (int61a): 4 "Missing
/// number" Warnings, the marks lost and the texts apart. KNOWN_PERL_ERRORS #1135; witness 2210.14698. Repro
/// package-bindings/footnote_mark_numexpr_relax_and_space_are_numbers.
#[test]
fn footnote_mark_numexpr_relax_and_space_are_numbers() {
  let xml = convert_clean(repro!(
    "package-bindings/footnote_mark_numexpr_relax_and_space_are_numbers.tex"
  ));
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">A<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>One.</note> B<note mark="2" role="footnote" xml:id="footnote2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags>Two.</note><note mark="1" role="footnote" xml:id="footnotex1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Again one.</note> C<note mark="2" role="footnote" xml:id="footnotex2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags>Again two.</note>.</p>"#,
  );
}

/// `\textsuperscript{1\protect\thanks{…}}` keeps its label "1" and the note; `\newcommand\fnA{2\footnotemark}` in a mark
/// prints "2" and the note's mark, as TeX reads its body. Red (int61a): Ann's "1" lost; Bob's "2" in his name text.
/// A macro whose body does more than print a label beside its note is taken out whole, as int62a read every note
/// macro (review rev78v finding 1; red at 78v r1: Di's "3" lost, Ed's "4" lost with an invented footnotemark, Fa's "6"
/// and mark lost): `\fnD{\thanks{D note}\unskip}` "Di Delta3" and its note, `\fnE{\ifshow\footnotemark\fi}` under
/// `\showfalse` "Ed Eps4" with no mark, `\fnF[1][x]{6\footnotemark}` (optional arguments). So is one with a `\relax` or
/// a `\protect` beside its label and one after a `\protect` (review rev78v2 finding 1; red at 78v r1-r2: the label kept
/// the `\relax`/`\protect`, which nothing answers, so Gu's, Hal's and Io's "5", Jo's "1" and "5", Ka's "2" were lost):
/// `\fnU{\relax 5\footnotemark}`, `\fnV{\protect 5\footnotemark}`, `\DeclareRobustCommand\fnS{5\footnotemark}` (its
/// body `\protect\fnS␣`) alone and as `1\fnS`, `\protect\fnA`. pdflatex "Ann Alpha1∗ , Bob Beta2† , Cy Gamma3, Di
/// Delta3‡ , Ed Eps4 , Fa Phi6§ , Gu Upsilon5¶ , Hal Vau5‖ , Io Sigma5∗∗ , Jo Rho15†† , Ka Kappa2‡‡".
/// Known residuals pinned here: Bob's footnotemark stands before his "2" (78u's order residual); Fa's "6" is in his
/// name text, not a `<sup>` (int62a's, the optional-argument macro read as one note); Gu's, Hal's, Io's, Jo's "5" and
/// Ka's "2" are in the name text, Jo's "1" after the mark (int63a's, each macro taken out whole).
/// Review rev78u findings 3-4. Repro sectioning-frontmatter/author_protect_thanks_and_note_macro_inside_a_mark.
#[test]
fn author_protect_thanks_and_note_macro_inside_a_mark() {
  assert_creators(
    repro!("sectioning-frontmatter/author_protect_thanks_and_note_macro_inside_a_mark.tex"),
    &[
      r#"<creator role="author"><personname>Ann Alpha<sup xml:id="id1">1</sup></personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Prot note.</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Beta<note mark="†" role="footnotemark" xml:id="footnote2"><tags><tag>†</tag><tag role="refnum">†</tag><tag role="typerefnum">footnote †</tag></tags></note><sup xml:id="id3">2</sup></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Cy Gamma<sup xml:id="id4">3</sup></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Di Delta<sup xml:id="id5">3</sup></personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id6">D note</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Ed Eps<sup xml:id="id7">4</sup></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Fa Phi6<note mark="§" role="footnotemark" xml:id="footnote4"><tags><tag>§</tag><tag role="refnum">§</tag><tag role="typerefnum">footnote §</tag></tags></note></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Gu Upsilon5<note mark="¶" role="footnotemark" xml:id="footnote5"><tags><tag>¶</tag><tag role="refnum">¶</tag><tag role="typerefnum">footnote ¶</tag></tags></note></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Hal Vau5<note mark="‖" role="footnotemark" xml:id="footnote6"><tags><tag>‖</tag><tag role="refnum">‖</tag><tag role="typerefnum">footnote ‖</tag></tags></note></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Io Sigma5<note mark="**" role="footnotemark" xml:id="footnote7"><tags><tag>**</tag><tag role="refnum">**</tag><tag role="typerefnum">footnote **</tag></tags></note></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Jo Rho5<note mark="††" role="footnotemark" xml:id="footnote8"><tags><tag>††</tag><tag role="refnum">††</tag><tag role="typerefnum">footnote ††</tag></tags></note><sup xml:id="id8">1</sup></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Ka Kappa2<note mark="‡‡" role="footnotemark" xml:id="footnote9"><tags><tag>‡‡</tag><tag role="refnum">‡‡</tag><tag role="typerefnum">footnote ‡‡</tag></tags></note></personname></creator>"#,
    ],
  );
}

/// caption.sty:856-868 formats a ruled float.sty float in caption's `ruled` style (`labelfont=bf,labelsep=space`), the
/// global `labelformat=allcaps,labelsep=quad` reset, a typed `\captionsetup[program]{labelsep=colon}` applied after it.
/// pdflatex "Algorithm 1 Two.", "FIGURE 1 Three.", "Program 1: Four.". Red (int61a): "ALGORITHM 1", "PROGRAM 1".
/// Witnesses 2106.08363, 2109.00138. Repro captions-floats/caption_ruled_float_sty_float_takes_its_style.
#[test]
fn caption_ruled_float_sty_float_takes_its_style() {
  assert_captions(
    repro!("captions-floats/caption_ruled_float_sty_float_takes_its_style.tex"),
    &[
      r#"<caption><tag close=" "><text font="bold" xml:id="algorithm1.3">Algorithm 1</text></tag>Two.</caption>"#,
      r#"<caption><tag close=" ">FIGURE 1</tag>Three.</caption>"#,
      r#"<caption><tag close=": "><text font="bold" xml:id="program1.3">Program 1</text></tag>Four.</caption>"#,
    ],
  );
}

/// caption.sty:882-884: `\captionsetup[<type>]{name=…}` of a float.sty type is its `\floatname` too, which its caption
/// prints: "Snip 1", "Proc 1". Red (int61a): "Listing 1", "Algorithm 1". Review rev78s2 finding 3. Repro
/// captions-floats/caption_typed_name_of_a_float_sty_type.
/// The plain float.sty float's caption had a double space after its tag (`</tag> Code one.`), removed in 78v for ruled
/// floats only (review rev78v finding 4) and in 78y for every float.sty float under caption, which formats it.
#[test]
fn caption_typed_name_of_a_float_sty_type() {
  assert_captions(
    repro!("captions-floats/caption_typed_name_of_a_float_sty_type.tex"),
    &[
      r#"<caption><tag close=": ">Snip 1</tag>Code one.</caption>"#,
      r#"<caption><tag close=" "><text font="bold" xml:id="algorithm1.3">Proc 1</text></tag>A one.</caption>"#,
    ],
  );
}

/// newfloat.sty:304 `\SetupFloatingEnvironment{listing}{name=Prog,within=section,listname=…}` renames the type and numbers
/// it within sections: "Prog 1.1: Code one.". Red (int61a): "List of listings 1" (Perl's signature read the type from
/// the options' place). KNOWN_PERL_ERRORS #1138; witnesses 2306.12224, 2411.16612, 2510.09447. Repro
/// captions-floats/newfloat_setup_floating_environment_sets_its_options.
#[test]
fn newfloat_setup_floating_environment_sets_its_options() {
  let xml = convert_clean(repro!(
    "captions-floats/newfloat_setup_floating_environment_sets_its_options.tex"
  ));
  assert_element(
    &xml,
    "float",
    &["xml:id=\"listing1\""],
    r#"<float class="ltx_float_listing" inlist="lolisting" placement="h" xml:id="listing1"><tags><tag>Prog 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">Prog 1.1</tag></tags><p xml:id="listing1.1">Y</p><toccaption><tag close=" ">1.1</tag>Code one.</toccaption><caption><tag close=": ">Prog 1.1</tag>Code one.</caption></float>"#,
  );
}

/// `\input epsf.sty` reads the paper's 1991 epsf.sty, whose reset `\subfigure`'s box confines: both panels 100pt, as
/// latex logs. Red (int61a): b.eps at its natural size (`\@currname` unset, so only epsf.tex was read). KNOWN_PERL_ERRORS
/// #1117; witnesses 0906.1361, astro-ph0006390, astro-ph0109126, astro-ph0410051, astro-ph9701125. Repro
/// package-bindings/epsfbox_input_sty_reads_the_shipped_copy.
#[test]
fn epsfbox_input_sty_reads_the_shipped_copy() {
  let xml = convert_clean(repro!(
    "package-bindings/epsfbox_input_sty_reads_the_shipped_copy.tex"
  ));
  assert_eq!(
    all_elements(&xml, "graphics"),
    [
      r#"<graphics graphic="a.eps" options="width=100.0pt" xml:id="S0.F1.sf1.g1"/>"#,
      r#"<graphics graphic="b.eps" options="width=100.0pt" xml:id="S0.F1.sf2.g1"/>"#,
    ],
    "{xml}"
  );
}

/// psfrag's epsf.sty is epsfig (`\RequirePackage{epsfig}`), whose reset is local (epsfig.sty:100-101): both panels
/// 100pt. Red (int61a): b.eps at its natural size. Witnesses astro-ph0012205, astro-ph9611186, astro-ph9911225. Repro
/// package-bindings/epsfbox_psfrag_wrapper_resets_locally.
#[test]
fn epsfbox_psfrag_wrapper_resets_locally() {
  let xml = convert_clean(repro!(
    "package-bindings/epsfbox_psfrag_wrapper_resets_locally.tex"
  ));
  assert_eq!(
    all_elements(&xml, "graphics"),
    [
      r#"<graphics graphic="a.eps" options="width=100.0pt" xml:id="S0.F1.sf1.g1"/>"#,
      r#"<graphics graphic="b.eps" options="width=100.0pt" xml:id="S0.F1.sf2.g1"/>"#,
    ],
    "{xml}"
  );
}

/// `\includegraphics{eps_as_pdf_one.eps}` with only `eps_as_pdf_one.pdf` shipped (the author ran epstopdf) finds the
/// `.pdf`, as post does, so the box has its size: 200bp scaled 0.5, "width 100.375pt"; an `.eps` that exists stays the
/// only candidate. Red (int61a): no candidates, "width 0.0pt". The retry's hits come in a fixed type order, the `.pdf`
/// before the `.ps` whichever the directory lists first (review rev78v finding 3, 0907.3747; red at 78v r1:
/// `eps_as_pdf_four.ps,eps_as_pdf_four.pdf`). KNOWN_PERL_ERRORS #1137, OXIDIZED_DESIGN_DIVERGENCES #882; witnesses
/// 1408.2843, 1503.04804. Repro package-bindings/includegraphics_eps_shipped_only_as_pdf_is_sized.
#[test]
fn includegraphics_eps_shipped_only_as_pdf_is_sized() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/includegraphics_eps_shipped_only_as_pdf_is_sized.tex"),
    &[
      (
        "eps_as_pdf_one.pdf",
        repro!("package-bindings/eps_as_pdf_one.pdf"),
      ),
      (
        "eps_as_pdf_two.eps",
        repro!("package-bindings/eps_as_pdf_two.eps"),
      ),
      (
        "eps_as_pdf_two.pdf",
        repro!("package-bindings/eps_as_pdf_two.pdf"),
      ),
      // (written in opposite orders: a tmpfs lists a directory newest first)
      (
        "eps_as_pdf_three.pdf",
        repro!("package-bindings/eps_as_pdf_one.pdf"),
      ),
      (
        "eps_as_pdf_three.ps",
        repro!("package-bindings/eps_as_pdf_two.eps"),
      ),
      (
        "eps_as_pdf_four.ps",
        repro!("package-bindings/eps_as_pdf_two.eps"),
      ),
      (
        "eps_as_pdf_four.pdf",
        repro!("package-bindings/eps_as_pdf_one.pdf"),
      ),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">width 100.375pt.</p>"#,
  );
  assert_eq!(
    all_elements(&xml, "graphics"),
    [
      r#"<graphics candidates="eps_as_pdf_one.pdf" class="ltx_figure_panel" graphic="eps_as_pdf_one.eps" options="scale=0.5" xml:id="S0.F1.g1"/>"#,
      r#"<graphics candidates="eps_as_pdf_two.eps" class="ltx_figure_panel" cssstyle="width:4.015em; height:2.007em" graphic="eps_as_pdf_two.eps" xml:id="S0.F1.g2"/>"#,
      r#"<graphics candidates="eps_as_pdf_three.pdf,eps_as_pdf_three.ps" class="ltx_figure_panel" cssstyle="width:20.075em; height:10.037em" graphic="eps_as_pdf_three.eps" xml:id="S0.F1.g3"/>"#,
      r#"<graphics candidates="eps_as_pdf_four.pdf,eps_as_pdf_four.ps" class="ltx_figure_panel" cssstyle="width:20.075em; height:10.037em" graphic="eps_as_pdf_four.eps" xml:id="S0.F1.g4"/>"#,
    ],
    "{xml}"
  );
}

/// `\graphicspath{{new/}{figs/}}` with a `gp_plot.pdf` in each: the base-name retry of `\includegraphics{gp_plot.eps}`
/// lists `new/gp_plot.pdf` first, the search-directory order LaTeX tries (latex.ltx:9698 `\@iffileonpath`), and sizes the box from
/// it, as the extensionless name does: "width 200.75pt" twice (pdflatex reads `new/gp_plot.pdf` for `gp_plot`). Red at
/// 78v r2: `figs/gp_plot.pdf,new/gp_plot.pdf` (a tie-break by full path), "width 401.5pt". Review rev78v2 finding 2;
/// OXIDIZED_DESIGN_DIVERGENCES #882. Repro package-bindings/includegraphics_eps_retry_keeps_the_graphicspath_order.
#[test]
fn includegraphics_eps_retry_keeps_the_graphicspath_order() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/includegraphics_eps_retry_keeps_the_graphicspath_order.tex"),
    &[
      // (written the later directory first)
      (
        "figs/gp_plot.pdf",
        repro!("package-bindings/eps_as_pdf_two.pdf"),
      ),
      (
        "new/gp_plot.pdf",
        repro!("package-bindings/eps_as_pdf_one.pdf"),
      ),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">width 200.75pt.
width 200.75pt.
<graphics candidates="new/gp_plot.pdf,figs/gp_plot.pdf" cssstyle="width:20.075em; height:10.037em" graphic="gp_plot.eps" xml:id="p1.g1"/>
<graphics candidates="new/gp_plot.pdf,figs/gp_plot.pdf" cssstyle="width:20.075em; height:10.037em" graphic="gp_plot" xml:id="p1.g2"/></p>"#,
  );
}

/// A document's `\DeclareCaptionStyle{ruled}{labelfont=normalfont,labelsep=colon}` is the style a ruled float.sty float
/// takes (caption.sty:864, caption3.sty:629-637): "Algorithm 1: Two." in the upright label font, the figure "Figure 1.
/// Three." under the global `labelfont=bf,labelsep=period`. Red (int61a): "Algorithm 1. Two." bold. Witness 2512.05464
/// (AAAI). Repro captions-floats/caption_declared_ruled_style_formats_a_ruled_float.
#[test]
fn caption_declared_ruled_style_formats_a_ruled_float() {
  assert_captions(
    repro!("captions-floats/caption_declared_ruled_style_formats_a_ruled_float.tex"),
    &[
      r#"<caption><tag close=": ">Algorithm 1</tag>Two.</caption>"#,
      r#"<caption><tag close=". "><text font="bold" xml:id="S0.F1.2">Figure 1</text></tag>Three.</caption>"#,
    ],
  );
}

/// LIPIcs's algorithm float is `ruled@notop` (lipics-v2021.cls:1090-1100, ported into lipics_cls.rs), a float style
/// caption declares no style for, so the class's caption settings stand: "Algorithm 1" sans-serif bold, the caption small.
/// Red (78v before the port, `ruled`'s style read): the label bold upright, the text normal size; int61a as here.
/// Witness 2008.11837. Repro class-bindings/lipics_algorithm_keeps_the_class_caption_settings.
#[test]
fn lipics_algorithm_keeps_the_class_caption_settings() {
  assert_captions(
    repro!("class-bindings/lipics_algorithm_keeps_the_class_caption_settings.tex"),
    &[
      r#"<caption><tag close=" "><text font="sansserif bold" fontsize="90%" xml:id="algorithm1.3">Algorithm 1</text></tag><text fontsize="90%" xml:id="algorithm1.4">One.</text></caption>"#,
    ],
  );
}
