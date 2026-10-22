//! Red/green guards for batch 78y (user ruling 2026-10-22): acmart's per-format `\captionsetup` (acmart.cls:952-984) and
//! IEEEtran's `\@makecaption` (IEEEtran.cls:2731-2797) in their bindings; a class's `\@makecaption` reaching every caption
//! LaTeX makes with it (a newfloat type's, listings'), caption's default separator for the class (an AMS class's ". ")
//! on every type caption formats, longtable's own "Table 1: ". The class formats are names and separators only, the
//! class's caption fonts behind the class caption fonts knob (user ruling 2026-10-22; caption_sty.rs
//! `CLASS_CAPTION_FONTS`, off by default; the `*_with_class_caption_fonts` guards set it). Round 2: an AMS class's format
//! the class's, not amsaddr's; a `\restylefloat`ed table keeps float.sty's; ieeeaccess/ieeeoj/ieeetj names; algorithm2e's
//! ": ". Each guard converts its pdflatex-checked repro under `tools/perfect_kernel/repros/class-bindings/` and asserts
//! every `<caption>` whole, after 0 errors and 0 warnings.
use latexml::util::test::{convert_files_with, normalize_markup, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

/// The class caption fonts knob set (caption_sty.rs `CLASS_CAPTION_FONTS`): latexml.sty's `classcaptionfonts` option as
/// the preload, ahead of the class (with `rawstyles`, ar5iv.sty's, for the packages a shipped copy loads raw).
const KNOB: &str = "[classcaptionfonts,rawstyles]latexml.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// Every `<caption>` element of `tex`'s conversion in document order, whole, against `expected`, after 0 errors and 0
/// warnings.
#[track_caller]
fn assert_captions(tex: &str, expected: &[&str]) {
  assert_captions_with(tex, &[], PRELOAD, expected)
}

/// [`assert_captions`] with `files` beside `tex` and `preload`.
#[track_caller]
fn assert_captions_with(tex: &str, files: &[(&str, &str)], preload: &str, expected: &[&str]) {
  let (log, xml) = convert_files_with(tex, files, Some(preload));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let mut found = Vec::new();
  let mut rest = xml.as_str();
  while let Some(element) = xml_element(rest, "caption", &[]) {
    found.push(element);
    let at = rest.find("<caption").unwrap();
    rest = &rest[at + 1..];
  }
  let expected: Vec<String> = expected.iter().map(|e| normalize_markup(e)).collect();
  assert_eq!(found, expected, "{xml}");
}

/// acmart's `manuscript` format is a journal one (acmart.cls:214-254): caption's AMS default ". " for every type,
/// the figure named "Fig." (acmart.cls:952-958; its small sans serif under the knob alone). pdflatex "Fig. 1. Fig one.",
/// "Table 1. Tab one.", "Listing 1. Code one.", "Program 1. Prog one.". Red (int65a): "Figure 1", "Listing 1: ",
/// "Program 1: ". Repro class-bindings/acmart_manuscript_caption_format.
#[test]
fn acmart_manuscript_takes_the_journal_caption_format() {
  assert_captions(repro!("acmart_manuscript_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=". ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `acmsmall`: as `manuscript`, the journal caption settings (acmart.cls:952-958). pdflatex "Fig. 1. Fig
/// one." … "Program 1. Prog one.". Red (int65a): "Figure 1", "Listing 1: ", "Program 1: ". Repro class-bindings/acmart_acmsmall_caption_format.
#[test]
fn acmart_acmsmall_takes_the_journal_caption_format() {
  assert_captions(repro!("acmart_acmsmall_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=". ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `acmlarge`: the journal caption settings (acmart.cls:952-958). pdflatex "Fig. 1. Fig one." … "Program
/// 1. Prog one.". Red (int65a): "Figure 1", "Listing 1: ", "Program 1: ". Repro class-bindings/acmart_acmlarge_caption_format.
#[test]
fn acmart_acmlarge_takes_the_journal_caption_format() {
  assert_captions(repro!("acmart_acmlarge_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=". ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `acmtog`: the journal caption settings (acmart.cls:952-958). pdflatex "Fig. 1. Fig one." … "Program 1.
/// Prog one.". Red (int65a): "Figure 1", "Listing 1: ", "Program 1: ". Repro class-bindings/acmart_acmtog_caption_format.
#[test]
fn acmart_acmtog_takes_the_journal_caption_format() {
  assert_captions(repro!("acmart_acmtog_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=". ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `sigconf`: `labelsep=colon` (acmart.cls:960-961) for every type. pdflatex "Figure 1: Fig one." … "Program
/// 1: Prog one." (bold, under the knob alone). Red (int65a): figure and table ". ". Repro class-bindings/acmart_sigconf_caption_format.
#[test]
fn acmart_sigconf_captions_close_with_a_colon() {
  assert_captions(repro!("acmart_sigconf_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=": ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=": ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=": ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `siggraph`: sigconf's settings (acmart.cls:967-968; the italic text under the knob alone). pdflatex "Figure
/// 1: Fig one." …
/// Red (int65a): figure and table ". ", no fonts. Repro class-bindings/acmart_siggraph_caption_format.
#[test]
fn acmart_siggraph_captions_close_with_a_colon() {
  assert_captions(repro!("acmart_siggraph_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=": ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=": ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=": ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `sigplan`: `labelsep=period` (acmart.cls:969-971), a listing's caption as a figure's (1807.03757's minted
/// `listing`, PDF "Listing 1."). pdflatex "Figure 1. Fig one." … "Program 1. Prog one." (bold labels, under the knob
/// alone). Red (int65a): "Listing 1: ", "Program 1: ". Repro class-bindings/acmart_sigplan_caption_format.
#[test]
fn acmart_sigplan_captions_close_with_a_period() {
  assert_captions(repro!("acmart_sigplan_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=". ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `sigchi`: sigconf's settings (acmart.cls:972-976; the figure's and table's small and bold under the knob
/// alone). pdflatex
/// "Figure 1: Fig one." … Red (int65a): figure and table ". ", no fonts. Repro class-bindings/acmart_sigchi_caption_format.
#[test]
fn acmart_sigchi_captions_close_with_a_colon() {
  assert_captions(repro!("acmart_sigchi_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=": ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=": ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=": ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `sigchi-a`: as `sigchi` (acmart.cls:977-981), in the format's sans body, the float.sty type's label too
/// (caption makes it, not float.sty's `\rmfamily`; review rev78y finding 5). pdflatex "Figure 1: Fig one." …
/// Red (int65a): figure and table ". ", no fonts. Repro class-bindings/acmart_sigchi_a_caption_format.
#[test]
fn acmart_sigchi_a_captions_close_with_a_colon() {
  assert_captions(repro!("acmart_sigchi_a_caption_format.tex"), &[
    r#"<caption class="ltx_centering" font="sansserif"><tag close=": ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption font="sansserif"><tag close=": ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption font="sansserif"><tag close=": ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption font="sansserif"><tag close=": ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// acmart's `acmengage`: the conference settings (acmart.cls:960-961). pdflatex "Figure 1: Fig one." …. Red
/// (int65a): figure and table ". ". Repro class-bindings/acmart_acmengage_caption_format.
#[test]
fn acmart_acmengage_captions_close_with_a_colon() {
  assert_captions(repro!("acmart_acmengage_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=": ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=": ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=": ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// IEEEtran's traditional `\@makecaption` (IEEEtran.cls:2774-2797): "Fig. 1. Fig one.", a newfloat type as a figure
/// ("Listing 1. Code one."), the table's label over its text ("TABLE I" / "TAB ONE.", the small caps under the knob
/// alone), float.sty's own
/// "Program 1: Prog one." (pdflatex). Red (int65a): every label closed by ": ". Repro class-bindings/ieeetran_journal_caption_format.
#[test]
fn ieeetran_journal_caption_format() {
  assert_captions(repro!("ieeetran_journal_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag>TABLE I</tag><break/>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
  ]);
}

/// IEEEtran `conference`: the traditional `\@makecaption` (IEEEtran.cls:2774-2797), as the journal's. Red (int65a):
/// every label closed by ": ". Repro class-bindings/ieeetran_conference_caption_format.
#[test]
fn ieeetran_conference_caption_format() {
  assert_captions(repro!("ieeetran_conference_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag>TABLE I</tag><break/>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
  ]);
}

/// IEEEtran `transmag`: the traditional `\@makecaption` (IEEEtran.cls:2774-2797). Red (int65a): every label closed by
/// ": ". Repro class-bindings/ieeetran_transmag_caption_format.
#[test]
fn ieeetran_transmag_caption_format() {
  assert_captions(repro!("ieeetran_transmag_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag>TABLE I</tag><break/>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
  ]);
}

/// IEEEtran `comsoc`: the traditional `\@makecaption` (IEEEtran.cls:2774-2797). Red (int65a): every label closed by
/// ": ". Repro class-bindings/ieeetran_comsoc_caption_format.
#[test]
fn ieeetran_comsoc_caption_format() {
  assert_captions(repro!("ieeetran_comsoc_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag>TABLE I</tag><break/>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
  ]);
}

/// IEEEtran `compsoc` journals (IEEEtran.cls:2753-2770): "TABLE 1" (arabic, :2826-2831) over its text (every caption
/// in sans serif under the knob alone). pdflatex "Fig. 1. Fig one.", "TABLE 1" / "Tab one.", "Listing 1. Code one.", "Program 1: Prog one.".
/// Red (int65a): "TABLE I", every label closed by ": ". Repro class-bindings/ieeetran_compsoc_caption_format.
#[test]
fn ieeetran_compsoc_caption_format() {
  assert_captions(repro!("ieeetran_compsoc_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag>TABLE 1</tag><break/>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
  ]);
}

/// IEEEtran `compsoc,conference` (IEEEtran.cls:2733-2750): "Figure 1. Fig one." (:2609), "TABLE 1. TAB ONE." on
/// one line (the small caps under the knob alone), "Listing 1. Code one.", "Program 1: Prog one." (pdflatex). Red (int65a): "Fig. 1", "TABLE I", ": ". Repro class-bindings/ieeetran_compsoc_conference_caption_format.
#[test]
fn ieeetran_compsoc_conference_caption_format() {
  assert_captions(repro!("ieeetran_compsoc_conference_caption_format.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">TABLE 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
  ]);
}

/// Caption (loaded by subcaption) replaces IEEEtran's `\@makecaption`: every label closed by ": " (pdflatex "Fig. 1:
/// Fig one.", "TABLE I: Tab one."), the class format not installed (caption_sty.rs `class_makecaption`). Red (int65a)
/// only by the float.sty caption's stray leading space. Repro class-bindings/ieeetran_caption_yields_to_caption.
#[test]
fn ieeetran_caption_yields_to_caption() {
  assert_captions(repro!("ieeetran_caption_yields_to_caption.tex"), &[
    r#"<caption class="ltx_centering"><tag close=": ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=": ">TABLE I</tag>Tab one.</caption>"#,
    r#"<caption><tag close=": ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag>Prog one.</caption>"#,
  ]);
}

/// listings' caption is the class's `\@makecaption` (listings.sty:1675): IEEEtran's "Listing 1. Code one."; longtable's
/// own `\LT@makecaption` keeps "TABLE I: Long one." (pdflatex). Red (int65a): "Listing 1: ". Repro class-bindings/ieeetran_listings_and_longtable_captions.
#[test]
fn ieeetran_listings_caption_takes_the_figure_format() {
  assert_captions(repro!("ieeetran_listings_and_longtable_captions.tex"), &[
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">TABLE I</tag>Long one.</caption>"#,
  ]);
}

/// With caption loaded, listings' and longtable's captions are caption's ("Listing 1: Code one.", "TABLE I: Long
/// one.", pdflatex), IEEEtran's format not installed. Green on int65a (a guard for the class format). Repro class-bindings/ieeetran_listings_and_longtable_captions_under_caption.
#[test]
fn caption_settings_reach_a_listing() {
  assert_captions(
    repro!("ieeetran_listings_and_longtable_captions_under_caption.tex"),
    &[
      r#"<caption><tag close=": ">Listing 1</tag>Code one.</caption>"#,
      r#"<caption><tag close=": ">TABLE I</tag>Long one.</caption>"#,
    ],
  );
}

/// amsart's `\@makecaption` (amsart.cls:1386-1410) makes a newfloat type's caption "Listing 1. Code one."; float.sty's
/// plain style and longtable keep their own ": " (pdflatex). Red (int65a): "Listing 1: ", "Table 2. Long one.". Repro class-bindings/amsart_newfloat_caption_takes_the_class_format.
#[test]
fn amsart_newfloat_caption_takes_the_class_format() {
  assert_captions(
    repro!("amsart_newfloat_caption_takes_the_class_format.tex"),
    &[
      r#"<caption><tag close=". ">Figure 1</tag>Fig one.</caption>"#,
      r#"<caption><tag close=". ">Table 1</tag>Tab one.</caption>"#,
      r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
      r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
      r#"<caption><tag close=": ">Table 2</tag>Long one.</caption>"#,
    ],
  );
}

/// Caption under an AMS class takes caption-ams-smf.sto's default separator for every type (caption3.sty:1138): pdflatex
/// "Program 1. Prog one.", "Table 2. Long one.". Red (int65a): "Listing 1: ", "Program 1: ". Repro class-bindings/amsart_caption_package_formats_every_type.
#[test]
fn caption_under_an_ams_class_closes_every_type_with_a_period() {
  assert_captions(repro!("amsart_caption_package_formats_every_type.tex"), &[
    r#"<caption><tag close=". ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">Table 1</tag>Tab one.</caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=". ">Program 1</tag>Prog one.</caption>"#,
    r#"<caption><tag close=". ">Table 2</tag>Long one.</caption>"#,
  ]);
}

/// algorithm2e makes its captions itself (algorithm2e.sty:2287-2318), so acmart's caption settings reach the figure
/// ("Fig. 1. Fig one.") and not the algorithm, which keeps algorithm2e's format: "Algorithm 1: Alg one." (pdflatex;
/// :2285-2289 `\algocf@capseparator`), its bold label `\fnum@font@algorithm` the package's, not behind the class caption
/// fonts knob. Red at 78y before the guard: the algorithm's label and text in small sans serif; at 78y r1 closed by " "
/// (Perl's too). Witnesses 1908.06049, 2205.14076, 2205.01906, 2004.00833. Repro
/// class-bindings/acmart_algorithm2e_caption_is_not_caption_s.
#[test]
fn algorithm2e_caption_is_not_caption_s() {
  assert_captions(
    repro!("acmart_algorithm2e_caption_is_not_caption_s.tex"),
    &[
      "<caption><tag close=\": \"><text font=\"bold\" xml:id=\"algorithm1.3\">Algorithm\u{a0}1</text></tag>Alg one.</caption>",
      r#"<caption><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    ],
  );
}

/// amsaddr under article loads ams_support, which no longer carries the AMS classes' caption format
/// (ams_support_sty.rs `ams_class_caption_format`): the kernel's "Figure 1: ", caption's colon default for the newfloat
/// and float.sty types (pdflatex "Figure 1: Fig one.", "Listing 1: Code one.", "Program 1: Prog one."). Red (78y r1):
/// every label closed by ". " (int65a and Perl the figure's). Repro class-bindings/amsaddr_article_keeps_the_kernel_caption_format.
#[test]
fn amsaddr_article_keeps_the_kernel_caption_format() {
  assert_captions(
    repro!("amsaddr_article_keeps_the_kernel_caption_format.tex"),
    &[
      r#"<caption><tag close=": ">Figure 1</tag>Fig one.</caption>"#,
      r#"<caption><tag close=": ">Listing 1</tag>Code one.</caption>"#,
      r#"<caption><tag close=": ">Program 1</tag>Prog one.</caption>"#,
    ],
  );
}

/// As [`amsaddr_article_keeps_the_kernel_caption_format`] without caption: the kernel's ": " for the figure and the
/// newfloat type, float.sty's own for its type. Red (78y r1): "Figure 1. ", "Listing 1. ". Repro
/// class-bindings/amsaddr_article_keeps_the_kernel_caption_format_without_caption.
#[test]
fn amsaddr_article_keeps_the_kernel_caption_format_without_caption() {
  assert_captions(
    repro!("amsaddr_article_keeps_the_kernel_caption_format_without_caption.tex"),
    &[
      r#"<caption><tag close=": ">Figure 1</tag>Fig one.</caption>"#,
      r#"<caption><tag close=": ">Listing 1</tag>Code one.</caption>"#,
      r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
    ],
  );
}

/// A `\restylefloat{table}` under IEEEtran makes the table's captions float.sty's (float.sty:37-52): "TABLE I: Tab one."
/// on one line, the figure the class's "Fig. 1. Fig one." (pdflatex; caption_sty.rs `\lx@class@floatsty@kept`). Red
/// (78y r1): "TABLE I" over "TAB ONE.". A `\restylefloat{figure}` figure is float.sty's too, "Fig. 1: Fig one.", and a
/// `\restylefloat*{table}` table keeps LaTeX's `\@caption` (float.sty:38-39), so the class's "TABLE I" over its text
/// (pdflatex; review rev78y2 finding 2, repro class-bindings/ieeetran_restylefloat_figure_and_starred_table). Witness
/// 2006.08364. Repro class-bindings/ieeetran_restylefloat_table_keeps_float_sty_format.
#[test]
fn ieeetran_restylefloat_table_keeps_float_sty_format() {
  assert_captions(
    repro!("ieeetran_restylefloat_table_keeps_float_sty_format.tex"),
    &[
      r#"<caption><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
      r#"<caption><tag close=": ">TABLE I</tag>Tab one.</caption>"#,
    ],
  );
  assert_captions(
    repro!("ieeetran_restylefloat_figure_and_starred_table.tex"),
    &[
      r#"<caption><tag close=": ">Fig. 1</tag>Fig one.</caption>"#,
      r#"<caption><tag>TABLE I</tag><break/>Tab one.</caption>"#,
    ],
  );
}

/// ieeeaccess names its figures "FIGURE" and numbers its tables arabic (ieeeaccess.cls:244-262; Roman under `jtehm`):
/// pdflatex with 1910.07402's copy "FIGURE 1. Fig one.", "TABLE 1. Tab one.". Red (78y r1): "Fig. 1", "TABLE I". Repro
/// class-bindings/ieeeaccess_caption_names.
#[test]
fn ieeeaccess_caption_names() {
  assert_captions(repro!("ieeeaccess_caption_names.tex"), &[
    r#"<caption class="ltx_centering"><tag close=". ">FIGURE 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">TABLE 1</tag>Tab one.</caption>"#,
  ]);
}

/// ieeeoj: "FIGURE", arabic tables (ieeeoj.cls:1667, :1867-1869); pdflatex with 2608.19980's copy "FIGURE 1. Fig one.",
/// "TABLE 1. Tab one.". Red (78y r1): "Fig. 1", "TABLE I". Repro class-bindings/ieeeoj_caption_names.
#[test]
fn ieeeoj_caption_names() {
  assert_captions(repro!("ieeeoj_caption_names.tex"), &[
    r#"<caption><tag close=". ">FIGURE 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">TABLE 1</tag>Tab one.</caption>"#,
  ]);
}

/// ieeetj: "FIGURE", arabic tables (ieeetj.cls:1650, :1850-1852); pdflatex with 2510.05542's copy "FIGURE 1. Fig one.",
/// "TABLE 1. Tab one.". Red (78y r1): "Fig. 1", "TABLE I". Repro class-bindings/ieeetj_caption_names.
#[test]
fn ieeetj_caption_names() {
  assert_captions(repro!("ieeetj_caption_names.tex"), &[
    r#"<caption><tag close=". ">FIGURE 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=". ">TABLE 1</tag>Tab one.</caption>"#,
  ]);
}

/// algorithm2e closes its label with `\algocf@capseparator` (algorithm2e.sty:2285-2289; ":", `\SetAlgoCaptionSeparator`):
/// pdflatex "Algorithm 1: Alg one.", "Algorithm 2. Alg two.". Red (78y r1, int65a, Perl): both closed by " ". Repro
/// class-bindings/algorithm2e_caption_closes_with_its_separator.
#[test]
fn algorithm2e_caption_closes_with_its_separator() {
  assert_captions(
    repro!("algorithm2e_caption_closes_with_its_separator.tex"),
    &[
      "<caption><tag close=\": \"><text font=\"bold\" xml:id=\"algorithm1.2\">Algorithm\u{a0}1</text></tag>Alg one.</caption>",
      "<caption><tag close=\". \"><text font=\"bold\" xml:id=\"algorithm2.2\">Algorithm\u{a0}2</text></tag>Alg two.</caption>",
    ],
  );
}

/// algorithm.sty's float.sty `{algorithm}` beside `[algo2e]{algorithm2e}`: caption makes its captions in its `ruled`
/// style (caption.sty:904-908), "Algorithm 1 Leakage Removal" (pdflatex), though algorithm2e marks the type its own.
/// Red (78y r1): the text after a stray space (float.sty's format, caption kept off). Witnesses 2204.02306, 1812.03871,
/// 2408.08621. Repro class-bindings/algorithm_sty_beside_algo2e_caption_is_caption_s.
#[test]
fn algorithm_sty_beside_algo2e_caption_is_caption_s() {
  assert_captions(
    repro!("algorithm_sty_beside_algo2e_caption_is_caption_s.tex"),
    &[
      "<caption><tag close=\" \"><text font=\"bold\" xml:id=\"algorithm1.2\">Algorithm\u{a0}1</text></tag>Leakage Removal</caption>",
    ],
  );
}

// The class caption fonts knob on (caption_sty.rs `CLASS_CAPTION_FONTS`): each class's caption fonts as 78y r1 set them.

/// acmart's journal formats under the knob: label and text in small sans serif (acmart.cls:952-958). Repro
/// class-bindings/acmart_acmsmall_caption_format.
#[test]
fn acmart_acmsmall_with_class_caption_fonts() {
  assert_captions_with(repro!("acmart_acmsmall_caption_format.tex"), &[], KNOB, &[
    r#"<caption class="ltx_centering"><tag close=". "><text font="sansserif" fontsize="90%">Fig. 1</text></tag><text font="sansserif" fontsize="90%">Fig one.</text></caption>"#,
    r#"<caption><tag close=". "><text font="sansserif" fontsize="90%">Table 1</text></tag><text font="sansserif" fontsize="90%">Tab one.</text></caption>"#,
    r#"<caption><tag close=". "><text font="sansserif" fontsize="90%">Listing 1</text></tag><text font="sansserif" fontsize="90%">Code one.</text></caption>"#,
    r#"<caption><tag close=". "><text font="sansserif" fontsize="90%">Program 1</text></tag><text font="sansserif" fontsize="90%">Prog one.</text></caption>"#,
  ]);
}

/// acmart's `sigchi-a` under the knob: the figure's and table's labels and text bold and small, the listing's and the
/// float.sty type's bold (acmart.cls:960-961, :977-981), all in the format's sans body. Repro
/// class-bindings/acmart_sigchi_a_caption_format.
#[test]
fn acmart_sigchi_a_with_class_caption_fonts() {
  assert_captions_with(repro!("acmart_sigchi_a_caption_format.tex"), &[], KNOB, &[
    r#"<caption class="ltx_centering" font="sansserif"><tag close=": "><text font="bold" fontsize="90%">Figure 1</text></tag><text font="bold" fontsize="90%">Fig one.</text></caption>"#,
    r#"<caption font="sansserif"><tag close=": "><text font="bold" fontsize="90%">Table 1</text></tag><text font="bold" fontsize="90%">Tab one.</text></caption>"#,
    r#"<caption font="sansserif"><tag close=": "><text font="bold">Listing 1</text></tag><text font="bold">Code one.</text></caption>"#,
    r#"<caption font="sansserif"><tag close=": "><text font="bold">Program 1</text></tag><text font="bold">Prog one.</text></caption>"#,
  ]);
}

/// IEEEtran's traditional format under the knob: the table text in small caps (IEEEtran.cls:2774-2797). Repro
/// class-bindings/ieeetran_journal_caption_format.
#[test]
fn ieeetran_journal_with_class_caption_fonts() {
  assert_captions_with(repro!("ieeetran_journal_caption_format.tex"), &[], KNOB, &[
    r#"<caption class="ltx_centering"><tag close=". ">Fig. 1</tag>Fig one.</caption>"#,
    r#"<caption><tag>TABLE I</tag><break/><text font="smallcaps">Tab one.</text></caption>"#,
    r#"<caption><tag close=". ">Listing 1</tag>Code one.</caption>"#,
    r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
  ]);
}

/// IEEEtran `compsoc` journals under the knob: every caption in sans serif (IEEEtran.cls:2753-2770). Repro
/// class-bindings/ieeetran_compsoc_caption_format.
#[test]
fn ieeetran_compsoc_with_class_caption_fonts() {
  assert_captions_with(repro!("ieeetran_compsoc_caption_format.tex"), &[], KNOB, &[
    r#"<caption class="ltx_centering"><tag close=". "><text font="sansserif">Fig. 1</text></tag><text font="sansserif">Fig one.</text></caption>"#,
    r#"<caption><tag><text font="sansserif">TABLE 1</text></tag><break/><text font="sansserif">Tab one.</text></caption>"#,
    r#"<caption><tag close=". "><text font="sansserif">Listing 1</text></tag><text font="sansserif">Code one.</text></caption>"#,
    r#"<caption><tag close=": ">Program 1</tag> Prog one.</caption>"#,
  ]);
}

/// A jfm copy's `\@makecaption` under the knob: the label in small caps (0712.2739's jfm.cls:953-958). Repro
/// class-bindings/jfm_caption/jfm_copy_caption_ends_with_a_period.
#[test]
fn jfm_with_class_caption_fonts() {
  assert_captions_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/jfm_caption/jfm_copy_caption_ends_with_a_period.tex"
    ),
    &[(
      "jfm.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/jfm_caption/jfm.cls"),
    )],
    KNOB,
    &[
      r#"<caption><tag close=". "><text font="smallcaps">Figure 1</text></tag>Schematic.</caption>"#,
      r#"<caption><tag close=" "><text font="smallcaps">Table 1</text></tag>Values.</caption>"#,
    ],
  );
}

/// webofc's `\@makecaption` under the knob: the label and its period bold (webofc.cls:788-812). Repro
/// class-bindings/webofc/webofc_sidecaption_keeps_its_caption.
#[test]
fn webofc_with_class_caption_fonts() {
  assert_captions_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc_sidecaption_keeps_its_caption.tex"
    ),
    &[(
      "webofc.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc.cls"),
    )],
    KNOB,
    &[
      r#"<caption><tag close=" "><text font="bold">Figure 1.</text></tag>Side caption text.</caption>"#,
      r#"<caption><tag close=" "><text font="bold">Table 1.</text></tag>Values.</caption>"#,
    ],
  );
}

/// jinstpub's `\@makecaption` under the knob: the label bold, the period after it (jinstpub.sty:362-378). Repro
/// package-bindings/jinstpub/jinstpub_frontmatter_is_jheppub_s.
#[test]
fn jinstpub_with_class_caption_fonts() {
  assert_captions_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/jinstpub/jinstpub_frontmatter_is_jheppub_s.tex"
    ),
    &[(
      "jinstpub.sty",
      include_str!("../../../tools/perfect_kernel/repros/package-bindings/jinstpub/jinstpub.sty"),
    )],
    KNOB,
    &[r#"<caption><tag close=". "><text font="bold">Figure 1</text></tag>A box.</caption>"#],
  );
}
