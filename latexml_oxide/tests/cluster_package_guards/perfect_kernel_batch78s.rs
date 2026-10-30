//! Red/green guards for batch 78s (quality batch 16, fidelity audit 4's binding Errors): a robust `\genfrac`,
//! `\@ifdefinable` on a group, epsf's `\epsfsize`, wrapfig's `{wrapfloat}`, equations.sty's eqnarray, ntheorem's
//! `\newtheorem*` stepping its counter, iopart's `\submitted`, espcrc's `\addressmark[…]`. Each guard converts its
//! repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and whole elements, against pdflatex.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// Every `<equation>` in document order as `labels:refnum`, `labels:-` for one without tags.
fn equation_refnums(xml: &str) -> String {
  let re = regex::Regex::new(r#"(?s)<equation ([^>]*)>\s*(<tags>.*?</tags>)?"#).unwrap();
  let labels = regex::Regex::new(r#"labels="([^"]*)""#).unwrap();
  let refnum = regex::Regex::new(r#"<tag role="refnum">([^<]*)</tag>"#).unwrap();
  let printed = regex::Regex::new(r#"<tag>([^<]*)</tag>"#).unwrap();
  re.captures_iter(xml)
    .map(|c| {
      let label = labels
        .captures(&c[1])
        .map_or("", |l| l.get(1).unwrap().as_str());
      let tags = c.get(2).map_or("-".to_string(), |t| {
        let refnum = refnum
          .captures(t.as_str())
          .map_or("", |r| r.get(1).unwrap().as_str());
        let printed = printed
          .captures(t.as_str())
          .map_or("", |p| p.get(1).unwrap().as_str());
        format!("{refnum}{printed}")
      });
      format!("{label}:{tags}")
    })
    .collect::<Vec<_>>()
    .join("|")
}

/// amsmath.sty:244 `\DeclareRobustCommand{\genfrac}`: in xy's matrix-entry scanner the fraction is whole (pdflatex
/// clean). Red (int57a): its body's bare `\if.#1.\else…\fi` leaked, 8 "Didn't expect a \fi" among 15 Errors
/// (KNOWN_PERL_ERRORS #1115). Witness 0704.1039 (101 Errors, then TooManyErrors). Repro
/// ams-internals/genfrac_robust_in_xymatrix.
#[test]
fn genfrac_in_an_xymatrix_entry() {
  let xml = convert_clean(repro!("ams-internals/genfrac_robust_in_xymatrix.tex"));
  assert_element(
    &xml,
    "XMApp",
    &[],
    "<XMApp xml:id=\"p1.m1.pic1.m1.1.1\"><XMTok mathstyle=\"text\" role=\"FRACOP\" thickness=\"0.0pt\"/>\
     <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">a</XMTok>\
     <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">b</XMTok></XMApp>",
  );
}

/// latex.ltx:1296-1305: `\@ifdefinable{\@nameuse{olA}}{…}` tests the `\string` of the whole group, a name not defined,
/// so the definition runs (pdflatex prints Ā). Red (int57a): "undefined \olA", the DefToken read `\@nameuse`
/// (KNOWN_PERL_ERRORS #1116). Witness 1703.00683. Repro macro-state/ifdefinable_group_argument.
#[test]
fn ifdefinable_of_a_group_tests_its_string() {
  let xml = convert_clean(repro!("macro-state/ifdefinable_group_argument.tex"));
  assert_element(
    &xml,
    "XMApp",
    &[],
    "<XMApp xml:id=\"p1.m1.1.1\"><XMTok name=\"overline\" role=\"OVERACCENT\" stretchy=\"true\">\u{af}</XMTok>\
     <XMTok font=\"italic\" role=\"UNKNOWN\">A</XMTok></XMApp>",
  );
}

/// epsf.sty:648 `\def\epsfsize#1#2{\epsfxsize}`: `\epsfsize=0.6\textwidth` sets the width (pdflatex: no "=0.6").
/// Red (int57a): "undefined \epsfsize" and "=0.6" printed (KNOWN_PERL_ERRORS #1117). Witness nucl-th0009090. Repro
/// package-bindings/epsfsize_assigns_epsfxsize.
#[test]
fn epsfsize_assigns_epsfxsize() {
  let xml = convert_clean(repro!("package-bindings/epsfsize_assigns_epsfxsize.tex"));
  assert_element(
    &xml,
    "graphics",
    &[],
    "<graphics graphic=\"x.eps\" options=\"width=207.0pt\" xml:id=\"S0.F1.g1\"/>",
  );
  assert!(!xml.contains("=0.6"), "{xml}");
}

/// A 1991 epsf.sty resets the size locally (hep-ph9404329's :161), so inside `\subfigure{\epsfbox{…}}`
/// (subfigure.sty:74 boxes it) the size outlives the box and serves the next sub-figure too (pdflatex "a.eps:
/// width=100.0pt", "b.eps: width=100.0pt"). Red (78s r1): b.eps at its natural size, the reset global (KNOWN_PERL_ERRORS
/// #1117). Witness hep-ph9404329 (latex: Fig1b.ps width=198.45662pt). Repro
/// package-bindings/epsfbox_local_reset_of_a_1991_copy.
#[test]
fn epsfbox_local_reset_of_a_1991_copy() {
  let xml = convert_clean(repro!(
    "package-bindings/epsfbox_local_reset_of_a_1991_copy.tex"
  ));
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"b.eps\""],
    "<graphics graphic=\"b.eps\" options=\"width=100.0pt\" xml:id=\"S0.F1.sf2.g1\"/>",
  );
  assert!(
    xml.contains("graphic=\"a.eps\" options=\"width=100.0pt\""),
    "{xml}"
  );
}

/// TL's epsf.sty:505-506 resets the size `\global`ly, so it does not outlive the first `\subfigure` (pdflatex "b.eps:
/// width=0.0pt"). Witness nucl-th0009090. Repro package-bindings/epsfbox_global_reset_of_a_tl_copy.
#[test]
fn epsfbox_global_reset_of_a_tl_copy() {
  let xml = convert_clean(repro!(
    "package-bindings/epsfbox_global_reset_of_a_tl_copy.tex"
  ));
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"b.eps\""],
    "<graphics graphic=\"b.eps\" xml:id=\"S0.F1.sf2.g1\"/>",
  );
  assert!(
    xml.contains("graphic=\"a.eps\" options=\"width=100.0pt\""),
    "{xml}"
  );
}

/// A paper shipping only a 1991 epsf.tex and loading `\usepackage{epsf}` gets TL's epsf.sty (its `\def \epsfbox #1{`,
/// :294, read with TL's spaces), whose reset is global (:505-506): the second sub-figure takes its natural size, as
/// pdflatex. Red on 78s r2 (review rev78s2 finding 1). Repro package-bindings/epsfbox_tl_sty_over_a_shipped_1991_tex.
#[test]
fn epsfbox_tl_sty_over_a_shipped_1991_tex() {
  let xml = convert_clean(repro!(
    "package-bindings/epsfbox_tl_sty_over_a_shipped_1991_tex.tex"
  ));
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"b.eps\""],
    "<graphics graphic=\"b.eps\" xml:id=\"S0.F1.sf2.g1\"/>",
  );
  assert!(
    xml.contains("graphic=\"a.eps\" options=\"width=100.0pt\""),
    "{xml}"
  );
}

/// epsf.sty:505-506: each `\epsfbox` resets `\epsfxsize`/`\epsfysize`, so the size is the next graphic's only.
#[test]
fn epsfbox_resets_its_size() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage{epsf}\\begin{document}\n\\epsfxsize=100pt\\epsfbox{a.eps}\n\
     \\epsfbox{b.eps}\\end{document}",
  );
  assert!(
    xml.contains("graphic=\"a.eps\" options=\"width=100.0pt\""),
    "{xml}"
  );
  assert!(
    xml.contains("<graphics graphic=\"b.eps\" xml:id=\"p1.g2\"/>"),
    "{xml}"
  );
}

/// wrapfig.sty:40 `\wrapfloat{<type>}`, here newfloat's `listing` (as minted's), and :265-301 `wrap<type>` for a
/// float.sty `\newfloat` after wrapfig (pdflatex "Listing 1: Code.", "Program 1: More."). Red (int57a): "undefined
/// {wrapfloat}", "\caption outside any known float" (KNOWN_PERL_ERRORS #1118). Witness 2411.16612. Repro
/// captions-floats/wrapfloat_of_a_newfloat_type.
#[test]
fn wrapfloat_of_a_newfloat_type() {
  let xml = convert_clean(repro!("captions-floats/wrapfloat_of_a_newfloat_type.tex"));
  assert_element(
    &xml,
    "float",
    &["xml:id=\"listing1\""],
    "<float class=\"ltx_float_listing\" float=\"right\" inlist=\"lolisting\" labels=\"LABEL:l\" width=\"30%\" \
     xml:id=\"listing1\"><tags><tag>Listing 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Listing \
     1</tag></tags><p xml:id=\"listing1.1\">int x;</p><toccaption><tag close=\" \">1</tag>Code.</toccaption>\
     <caption><tag close=\": \">Listing 1</tag>Code.</caption></float>",
  );
  assert_element(
    &xml,
    "float",
    &["xml:id=\"program1\""],
    "<float class=\"ltx_float_program\" float=\"right\" inlist=\"lop\" labels=\"LABEL:m\" width=\"30%\" \
     xml:id=\"program1\"><tags><tag>Program 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Program \
     1</tag></tags><p xml:id=\"program1.1\">int y;</p><toccaption><tag close=\" \">1</tag>More.</toccaption>\
     <caption><tag close=\": \">Program 1</tag> More.</caption></float>",
  );
}

/// `{wrapfloat}{figure}` is the `ltx:figure` `{wrapfigure}` makes.
#[test]
fn wrapfloat_of_figure_is_a_figure() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage{wrapfig}\\begin{document}\n\\begin{wrapfloat}{figure}{l}{0.5\\textwidth}\
     \nX\\caption{C.}\\label{f}\\end{wrapfloat}\n\\end{document}",
  );
  assert_element(
    &xml,
    "figure",
    &[],
    "<figure float=\"left\" inlist=\"lof\" labels=\"LABEL:f\" width=\"50%\" \
     xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure \
     1</tag></tags><p xml:id=\"S0.F1.1\">X</p><toccaption><tag close=\" \">1</tag>C.</toccaption><caption>\
     <tag close=\": \">Figure 1</tag>C.</caption></figure>",
  );
}

/// equations.sty: the kernel's eqnarray numbered by its `\@@eqncr` (pdflatex "See 1 and 2."). Red (int57a): the
/// binding's raw `$$\halign` begin against the kernel's end, "\end@eqnarray Attempt to close a group…" and "Missing $",
/// label y "3" (KNOWN_PERL_ERRORS #1119). Witness hep-ph9904268. Repro package-bindings/equations_sty_eqnarray.
#[test]
fn equations_sty_eqnarray_numbers_as_latex() {
  let xml = convert_clean(repro!("package-bindings/equations_sty_eqnarray.tex"));
  assert_eq!(
    equation_refnums(&xml),
    ":-|LABEL:x:1(1)|LABEL:y:2(2)",
    "{xml}"
  );
}

/// equations.sty:128-152: eqnarray*'s rows start unnumbered (`\@defeqnswfalse`) and `\yesnumber` numbers one;
/// `\yesnumber[**]` prints `(**)` and steps nothing (pdflatex "(1)", "(2)", "(3)", the next equation "(4)").
#[test]
fn equations_sty_yesnumber_in_eqnarray_star() {
  let xml = convert_clean(
    "\\documentstyle[equations]{article}\\begin{document}\n\\begin{eqnarray}\na&=&b\\label{p}\\\\\nc&=&d\\nonumber\
     \\\\\ne&=&f\\label{q}\n\\end{eqnarray}\n\\begin{eqnarray*}\ng&=&h\\\\\ni&=&j\\yesnumber\\label{r}\\\\\n\
     k&=&l\\yesnumber[**]\\\\\nm&=&n\n\\end{eqnarray*}\n\\begin{equation}\\label{t} o \\end{equation}\n\
     \\end{document}",
  );
  assert_eq!(
    equation_refnums(&xml),
    "LABEL:p:1(1)|:-|LABEL:q:2(2)|:-|LABEL:r:3(3)|:**(**)|:-|LABEL:t:4(4)",
    "{xml}"
  );
}

/// ntheorem.sty:824-839, :870: `\newtheorem*{rmk}[thm]` steps `thm` and prints no number (pdflatex "Lemma 1",
/// "Remark", "Lemma 3"). Red (int57a): "Lemma 2" (KNOWN_PERL_ERRORS #1120). Witness 1606.01585. Repro
/// cross-references/ntheorem_starred_steps_counter.
#[test]
fn ntheorem_starred_steps_its_counter() {
  let xml = convert_clean(repro!(
    "cross-references/ntheorem_starred_steps_counter.tex"
  ));
  assert_element(
    &xml,
    "tags",
    &[],
    "<tags><tag>Lemma 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Lemma 1</tag></tags>",
  );
  let third = xml.split("<theorem ").nth(3).expect("three theorems");
  assert!(
    third.starts_with(
      "class=\"ltx_theorem_lem\" inlist=\"thm theorem:lem\" labels=\"LABEL:c\" xml:id=\"Thmthm3\">"
    ),
    "{xml}"
  );
  assert!(
    third.contains(
      "<tags>\n      <tag>Lemma 3</tag>\n      <tag role=\"refnum\">3</tag>\n      <tag role=\"typerefnum\">Lemma \
       3</tag>\n    </tags>"
    ),
    "{xml}"
  );
}

/// iopart.cls:277-278 `\submitted` prints "Submitted to: \journal" (pdflatex "Institute of Physics Publishing"). Red
/// (int57a): "Error:recursion:\journal" (KNOWN_PERL_ERRORS #1121). Witness hep-th9910179. Repro
/// sectioning-frontmatter/iopart_submitted_journal.
#[test]
fn iopart_submitted_names_the_journal() {
  let xml = convert_clean(repro!(
    "sectioning-frontmatter/iopart_submitted_journal.tex"
  ));
  assert_element(
    &xml,
    "pubnote",
    &[],
    // (the class's label, iopart.cls:268-269 "Submitted to:": batch 92e, OD #915)
    "<pubnote name=\"Submitted to:\u{a0}\" role=\"journal\">Institute of Physics Publishing</pubnote>",
  );
}

/// espcrc1.sty:159-162 `\addressmark[<key>]`: the key is read, not printed (pdflatex "Philippe de Forcrand" and its
/// mark). Red (int57a): `<personname>Philippe de Forcrand[ETHZ]</personname>` (KNOWN_PERL_ERRORS #1122). Witness
/// hep-lat0409072. Repro sectioning-frontmatter/espcrc_addressmark_key.
#[test]
fn espcrc_addressmark_key_not_printed() {
  let xml = convert_clean(repro!("sectioning-frontmatter/espcrc_addressmark_key.tex"));
  assert_element(
    &xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Philippe de Forcrand</personname>\
     <contact name=\"Address:\u{a0}\" role=\"address\">CERN</contact></creator>",
  );
  assert!(!xml.contains("[ETHZ]"), "{xml}");
}

/// minted.sty:1931-1936: with `newfloat`, `listing` is newfloat's float type, "Listing 1: Code." numbered apart from
/// the figures, which `{wrapfloat}{listing}` wraps (2411.16612). Red (int57a): a `{figure}`, "Figure 2"
/// (KNOWN_PERL_ERRORS #1123).
#[test]
fn minted_listing_is_a_float_of_its_own() {
  let xml = convert_clean(
    "\\documentclass{article}\\usepackage[newfloat]{minted}\\usepackage{wrapfig}\\begin{document}\n\
     \\begin{figure}X\\caption{F.}\\label{f}\\end{figure}\n\
     \\begin{listing}[h]\nY\\caption{Code.}\\label{l}\n\\end{listing}\n\
     \\begin{wrapfloat}{listing}{r}{0.3\\textwidth}\nZ\\caption{More.}\\label{m}\n\\end{wrapfloat}\n\
     \\end{document}",
  );
  assert_element(
    &xml,
    "float",
    &["xml:id=\"listing1\""],
    "<float class=\"ltx_float_listing\" inlist=\"lol\" labels=\"LABEL:l\" placement=\"h\" xml:id=\"listing1\"><tags>\
     <tag>Listing 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Listing 1</tag></tags>\
     <p xml:id=\"listing1.1\">Y</p><toccaption><tag close=\" \">1</tag>Code.</toccaption><caption><tag close=\": \">\
     Listing 1</tag>Code.</caption></float>",
  );
  assert!(
    xml.contains("labels=\"LABEL:m\" width=\"30%\" xml:id=\"listing2\""),
    "{xml}"
  );
  assert!(
    xml.contains("<tag role=\"typerefnum\">Figure 1</tag>"),
    "{xml}"
  );
}

/// subfig.sty:239-263 `\newsubfloat{<type>}` for a float.sty type: the type's `\subfloat`s are its sub-floats, "1a",
/// "1b" (pdflatex "See 1, 1a, 1b."). Red (int57a): `\newsubfloat` a no-op, the sub-floats lost (Perl
/// subfig.sty.ltxml:30-62 makes them). Witness 2411.16612. Repro captions-floats/newsubfloat_listing.
#[test]
fn newsubfloat_of_a_float_type() {
  let xml = convert_clean(repro!("captions-floats/newsubfloat_listing.tex"));
  assert_element(
    &xml,
    "float",
    &["xml:id=\"listing1.sf2\""],
    "<float class=\"ltx_figure_panel\" labels=\"LABEL:b\" xml:id=\"listing1.sf2\"><tags><tag>(b)</tag>\
     <tag role=\"refnum\">1b</tag></tags><p xml:id=\"listing1.sf2.1\">B</p><toccaption><tag close=\" \">b</tag>Two.\
     </toccaption><caption><tag close=\" \">(b)</tag>Two.</caption></float>",
  );
  assert!(
    xml.contains("labels=\"LABEL:a\" xml:id=\"listing1.sf1\""),
    "{xml}"
  );
}

/// minted.sty:284, :1937-1944: without `newfloat`, `listing` is float.sty's `\newfloat{listing}{tbp}{lol}` named
/// `\listingscaption`, which the document renames (pdflatex "Code 1: Code one.", "Code 2: Code two."). Red (78s r1):
/// "Listing 1" closed by a space (KNOWN_PERL_ERRORS #1123). Witnesses 2102.00527, 1902.08498, 2609.21330. Repro
/// captions-floats/minted_listing_float_sty_branch.
#[test]
fn minted_listing_is_float_sty_s_by_default() {
  let xml = convert_clean(repro!(
    "captions-floats/minted_listing_float_sty_branch.tex"
  ));
  assert_element(
    &xml,
    "float",
    &["xml:id=\"listing1\""],
    "<float class=\"ltx_float_listing\" inlist=\"lol\" labels=\"LABEL:l\" placement=\"H\" xml:id=\"listing1\"><tags>\
     <tag>Code 1</tag><tag role=\"autoref\">1<text xml:id=\"listing1.1\"/></tag><tag role=\"creftype\">listing</tag>\
     <tag role=\"creftypecap\">Listing</tag><tag role=\"creftypeplural\">listings</tag><tag \
     role=\"creftypepluralcap\">Listings</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Code 1</tag></tags>\
     <p xml:id=\"listing1.2\">Y</p><toccaption><tag close=\" \">1</tag>Code one.</toccaption><caption>\
     <tag close=\": \">Code 1</tag> Code one.</caption></float>",
  );
  assert!(
    xml.contains("<tag close=\": \">Code 2</tag> Code two."),
    "{xml}"
  );
  assert!(
    xml.contains("<tag role=\"typerefnum\">Figure 1</tag>"),
    "{xml}"
  );
}

/// minted.sty:157-159, :1931-1936: `[newfloat,section]` numbers the listings within the section (pdflatex "Listing
/// 1.1: One.", "Listing 2.1: Two."). Red (78s r1): "Listing 1", "Listing 2" (the option dropped). Witness 2411.16612.
/// Repro captions-floats/minted_newfloat_option_within_section.
#[test]
fn minted_newfloat_option_within_section() {
  let xml = convert_clean(repro!(
    "captions-floats/minted_newfloat_option_within_section.tex"
  ));
  assert_element(
    &xml,
    "float",
    &["labels=\"LABEL:b\""],
    "<float class=\"ltx_float_listing\" inlist=\"lol\" labels=\"LABEL:b\" placement=\"h\" xml:id=\"S2.listing1\">\
     <tags><tag>Listing 2.1</tag><tag role=\"refnum\">2.1</tag><tag role=\"typerefnum\">Listing 2.1</tag></tags>\
     <p xml:id=\"S2.listing1.1\">Z</p><toccaption><tag close=\" \">2.1</tag>Two.</toccaption><caption>\
     <tag close=\": \">Listing 2.1</tag>Two.</caption></float>",
  );
}

/// newfloat's type is a kernel float captioned by the kernel's `\@makecaption` (pdflatex "Table 1: T.", "Listing 1:
/// Code."). Red (int57a, as Perl): `<tag close=" ">Listing 1</tag>` (KNOWN_PERL_ERRORS #1124). Witness 2411.16612.
/// Repro captions-floats/newfloat_caption_tag_closes_with_a_colon.
#[test]
fn newfloat_caption_tag_closes_with_a_colon() {
  let xml = convert_clean(repro!(
    "captions-floats/newfloat_caption_tag_closes_with_a_colon.tex"
  ));
  assert_element(
    &xml,
    "float",
    &[],
    "<float class=\"ltx_float_listing\" inlist=\"lol\" labels=\"LABEL:l\" placement=\"h\" xml:id=\"listing1\"><tags>\
     <tag>Listing 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Listing 1</tag></tags>\
     <p xml:id=\"listing1.1\">int x;</p><toccaption><tag close=\" \">1</tag>Code.</toccaption><caption>\
     <tag close=\": \">Listing 1</tag>Code.</caption></float>",
  );
  assert!(
    xml.contains("<caption><tag close=\": \">Table 1</tag>T.</caption>"),
    "{xml}"
  );
}

/// subfig.sty:244-266: `\newsubfloat[<keys>]{<type>}` sets `\captionsetup[sub<type>]{<keys>}` and joins
/// `\sf@counterlist`; for a type that has its sub-float, subfig's warning and nothing (pdflatex: 1 warning, "a One.",
/// "b Two.", "subfigure,subtable,sublisting", "See 1a, 1b, 1a."). Red (78s r1): "(a) One.", no warning, the list
/// without sublisting. Witness 2411.16612. Repro captions-floats/newsubfloat_keys_and_an_existing_type.
#[test]
fn newsubfloat_keys_and_an_existing_type() {
  let (log, xml) = convert_with(
    repro!("captions-floats/newsubfloat_keys_and_an_existing_type.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Package subfig Warning: The subfigure type is already defined."),
    "{log}"
  );
  assert_element(
    &xml,
    "float",
    &["xml:id=\"listing1.sf2\""],
    "<float class=\"ltx_figure_panel\" labels=\"LABEL:b\" xml:id=\"listing1.sf2\"><tags><tag>b</tag>\
     <tag role=\"refnum\">1b</tag></tags><p xml:id=\"listing1.sf2.1\">B</p><toccaption><tag close=\" \">b</tag>Two.\
     </toccaption><caption><tag close=\" \">b</tag>Two.</caption></float>",
  );
  assert!(xml.contains("subfigure,subtable,sublisting."), "{xml}");
  assert!(
    xml.contains("<figure labels=\"LABEL:p\" xml:id=\"S0.F1.sf1\">"),
    "{xml}"
  );
}

/// caption.sty:878-881: a float.sty type's `\<type>name` is its `\fname@<type>`, the name a caption label format
/// reads (pdflatex "[Listing] 1: One.", "[Figure] 1: Two.", the label format's `~` a no-break space). Red (78s r2 before the fix): "[] 1", the name empty for
/// minted's float.sty `listing`. Witness 2602.12973 (LIPIcs's boxed label format). Repro
/// captions-floats/caption_label_format_names_a_float_sty_type.
#[test]
fn caption_label_format_names_a_float_sty_type() {
  let xml = convert_clean(repro!(
    "captions-floats/caption_label_format_names_a_float_sty_type.tex"
  ));
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\": \">[Listing]\u{a0}1</tag>One.</caption>",
  );
  assert!(
    xml.contains("<caption><tag close=\": \">[Figure]\u{a0}1</tag>Two.</caption>"),
    "{xml}"
  );
}
