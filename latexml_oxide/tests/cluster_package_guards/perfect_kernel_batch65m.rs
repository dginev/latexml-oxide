//! Red/green guards for perfect-kernel batch 65m, "store, don't typeset": text that TeX only writes to an auxiliary
//! file or a list, or gobbles, or reads as a name, is not digested as typeset content — the run-340 cluster
//! `Error:unexpected:_ Script _ can only appear in math mode` (6,357 first-error papers), which pdflatex never gives:
//! - natbib's `\bibitem` label (natbib.sty:822 writes it to the .aux) and a caption's or heading's `[short]` form
//!   (latex.ltx:17403-17404, the .lof/.toc) are written copies, whose `_`/`^` are characters (`tex_math::WRITTEN_COPY`,
//!   OXIDIZED_DESIGN #550);
//! - a counter's name is spliced into its `\the<ctr>@ID` as characters, not re-read from a string;
//! - imsart's `\bdoi` gobbles its DOI, the file and address arguments of feynmp's `{fmffile}`, cas's `\bio`, aastex's
//!   `{interactive}` and achemso's `\email` are read as strings, and a raw class's stored `\doi` is set as one
//!   (OXIDIZED_DESIGN #551);
//! - cleveref's `\label[type]{key}` after `\caption` gives its type, not its key, to the optional (OXIDIZED_DESIGN #552).
use latexml::util::test::{
  assert_element, convert_files_with, convert_with, rng_error_count, warning_count,
};

use super::perfect_kernel_batch46::error_count;

/// The XML of a converted repro (ar5iv), with exactly `errors` errors and `warnings` warnings, no Fatal, and
/// schema-valid (when jing is installed).
fn checked((log, xml): (String, String), errors: usize, warnings: usize) -> String {
  assert_eq!(error_count(&log), errors, "{log}");
  assert_eq!(warning_count(&log), warnings, "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  if let Some(invalid) = rng_error_count(&xml) {
    assert_eq!(invalid, 0, "schema errors:\n{xml}");
  }
  xml
}

fn convert_counted(tex: &str, errors: usize, warnings: usize) -> String {
  checked(convert_with(tex, Some("ar5iv.sty")), errors, warnings)
}

/// 65m: a natbib label is what natbib.sty:822 `\NAT@wrout` writes to the .aux, and the numeric list prints `[1]`: its
/// `_` is a character of the authors tag, not "Script _ can only appear in math mode" (1904.06560, 2205.04819,
/// 2109.10085; Perl the same, KNOWN_PERL_ERRORS #620). In author-year mode the year, authors, fullauthors and refnum
/// tags keep the label's text as plain text. Repros index-bib/natbib_bibitem_label_is_aux_only,
/// index-bib/natbib_authoryear_label_is_aux_only.
#[test]
fn natbib_label_is_tag_text() {
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/natbib_bibitem_label_is_aux_only.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "bibitem",
    &["key=\"MS_QDK\""],
    r#"<bibitem key="MS_QDK" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="authors">MS_</tag><tag role="refnum">[1]</tag><tag role="key">MS_QDK</tag></tags><bibblock> The Microsoft Quantum Development Kit.
</bibblock></bibitem>"#,
  );
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/natbib_authoryear_label_is_aux_only.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "bibitem",
    &["key=\"ds\""],
    r#"<bibitem key="ds" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2020</tag><tag role="authors">Data_Set</tag><tag role="refnum">Data_Set (2020)</tag><tag role="key">ds</tag></tags><bibblock> A data set.
</bibblock></bibitem>"#,
  );
  assert_element(
    &xml,
    "bibitem",
    &["key=\"eu\""],
    r#"<bibitem key="eu" xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="year">2019</tag><tag role="authors">Eu_B</tag><tag role="fullauthors">Eu_A</tag><tag role="refnum">Eu_B (2019)</tag><tag role="key">eu</tag></tags><bibblock> Another.
</bibblock></bibitem>"#,
  );
}

/// 65m: `\caption[fig_a]` and `\section[sec_a]` write their short forms to the .lof/.toc (latex.ltx:17403-17404,
/// `\@sect`), which no list here reads: the `toccaption` and `toctitle` keep "fig_a"/"sec_a" as text, with no Error
/// (1303.1693, 1206.2108, astro-ph0412052, hep-lat9812022; Perl pool:3189/3370 the same). Repro
/// captions-floats/caption_short_form_is_lof_only.
#[test]
fn short_forms_are_list_text() {
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/caption_short_form_is_lof_only.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S1.F1\""],
    r#"<figure inlist="lof" xml:id="S1.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p align="center" xml:id="S1.F1.1">A</p><toccaption class="ltx_centering"><tag close=" ">1</tag>fig_a</toccaption><caption class="ltx_centering"><tag close=": ">Figure 1</tag>A figure.</caption></figure>"#,
  );
  assert_element(
    &xml,
    "toctitle",
    &[],
    r#"<toctitle><tag close=" ">1</tag>sec_a</toctitle>"#,
  );
}

/// 65m: a counter named `c_n` or `claim_nb` (2606.16543, 1912.04210): its `\the<ctr>@ID` holds the name as characters,
/// so stepping it raises nothing, and the claim's id carries the name (Perl Package.pm:695-702 re-reads the name from
/// a string, KNOWN_PERL_ERRORS #621). Repro macro-state/counter_name_with_underscore.
#[test]
fn counter_name_with_underscore_is_its_id() {
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/counter_name_with_underscore.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "theorem",
    &["class=\"ltx_theorem_claim\""],
    r#"<theorem class="ltx_theorem_claim" inlist="thm theorem:claim" labels="LABEL:c" xml:id="Thmthm1.claim_nb1"><tags><tag>Claim 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Claim 1</tag></tags><title class="ltx_runin"><tag><text font="bold" xml:id="Thmthm1.claim_nb1.1">Claim 1</text></tag></title><para xml:id="Thmthm1.claim_nb1.p1"><p xml:id="Thmthm1.claim_nb1.p1.1"><text font="italic" xml:id="Thmthm1.claim_nb1.p1.1.1">B</text></p></para></theorem>"#,
  );
}

/// 65m: imsart.sty:2583 `\def\bdoi#1{\ignorespaces}` gobbles the DOI of a .bbl entry (1405.0671, 1511.07613,
/// 2110.07639); the binding printed it. The one Warning is imsart.sty, which is not in TeX Live. Repro
/// class-bindings/imsart_bdoi_is_gobbled.
#[test]
fn imsart_bdoi_is_gobbled() {
  let xml = convert_counted(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/imsart_bdoi_is_gobbled.tex"),
    0,
    1,
  );
  assert_element(
    &xml,
    "bibitem",
    &["key=\"a\""],
    r#"<bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Author. Title.
</bibblock></bibitem>"#,
  );
}

/// 65m: a file or address argument is a name, never typeset: feynmp's `{fmffile}{sde_feynman}` (1906.06341,
/// 2512.17695), cas's `\bio{Figures/chamara_sandeepa.png}`
/// (2203.04264), aastex's `{interactive}{animation}{rolling_ns3.mp4}` (2303.02355), achemso's
/// `\email{ranojoy_bose@apple.com}` — the author's contact, as the other classes' addresses (1803.03591; the Warning
/// is the mciteplus stub). Repros graphics-tikz/feynmp_fmffile_name_is_not_typeset,
/// class-bindings/cas_bio_photo_file_is_not_typeset, class-bindings/aastex_interactive_arguments_are_not_typeset,
/// class-bindings/achemso_email_is_an_address.
#[test]
fn file_arguments_are_not_typeset() {
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/feynmp_fmffile_name_is_not_typeset.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "document",
    &[],
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><note role="feynman-diagram" xml:id="id1">(Feynman diagram, 40x25)</note></document>"#,
  );
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/cas_bio_photo_file_is_not_typeset.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "note",
    &["role=\"biography\""],
    r#"<note role="biography" xml:id="p1.2"><text font="bold" xml:id="p1.2.1">C. S.</text> is a student.
</note>"#,
  );
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aastex_interactive_arguments_are_not_typeset.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "figure",
    &[],
    &[
      r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="autoref">Figure"#,
      "\u{a0}",
      r#"1<text xml:id="S0.F1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p align="center" xml:id="S0.F1.2">A</p><toccaption><tag close=" ">1</tag>C</toccaption><caption><tag close=": ">Figure 1</tag>C</caption></figure>"#,
    ]
    .concat(),
  );
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/achemso_email_is_an_address.tex"
    ),
    0,
    1,
  );
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Ranojoy Bose</personname><contact name=\"Email:\u{a0}\" role=\"email\">ranojoy_bose@apple.com</contact></creator>",
  );
  assert!(!xml.contains("<note role=\"email\""), "{xml}");
}

/// 65m: a raw class's `\doi` stores its DOI (sig-alternate-05-2015.cls:1639-1641, printed by `\url`); rerouted to the
/// frontmatter (K11), the DOI is set as a string, so its `_` is a character (1610.03579, 1710.01977). Repro
/// sectioning-frontmatter/raw_class_doi_store_is_a_string, its .cls a reduced model of the class.
#[test]
fn raw_class_doi_is_a_string() {
  let xml = checked(
    convert_files_with(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_class_doi_store_is_a_string.tex"
      ),
      &[(
        "raw_class_doi_store_is_a_string.cls",
        include_str!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_class_doi_store_is_a_string.cls"
        ),
      )],
      Some("ar5iv.sty"),
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "pubnote",
    &["role=\"doi\""],
    "<pubnote name=\"DOI:\u{a0}\" role=\"doi\">10.475/123_4</pubnote>",
  );
  assert_eq!(xml.matches("<pubnote ").count(), 1, "{xml}");
  // An escaped `\_` stays its source text, as the class's `\url` prints it (pdflatex: "DOI: 10.1002/a\_b").
  let xml = checked(
    convert_files_with(
      &include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_class_doi_store_is_a_string.tex"
      )
      .replace(r"\doi{10.475/123_4}", r"\doi{10.1002/a\_b}"),
      &[(
        "raw_class_doi_store_is_a_string.cls",
        include_str!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_class_doi_store_is_a_string.cls"
        ),
      )],
      Some("ar5iv.sty"),
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "pubnote",
    &["role=\"doi\""],
    "<pubnote name=\"DOI:\u{a0}\" role=\"doi\">10.1002/a\\_b</pubnote>",
  );
}

/// 65m: cleveref's `\label[appendixtable]{tab:a_b}` right after `\caption` (2404.11739): the type is the optional, the
/// key the label — not `labels="LABEL:["` and "appendixtable]tab:a_b" typeset (Perl pool:3172 the same). Repro
/// captions-floats/cleveref_label_type_after_caption.
#[test]
fn cleveref_label_type_after_caption() {
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/cleveref_label_type_after_caption.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "table",
    &[],
    r#"<table inlist="lot" labels="LABEL:tab:a_b" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="creftype">table</tag><tag role="creftypecap">Table</tag><tag role="creftypeplural">tables</tag><tag role="creftypepluralcap">Tables</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>T</toccaption><caption><tag close=": ">Table 1</tag>T</caption></table>"#,
  );
  // The same move after algorithm2e's procedure caption and after caption.sty's `\captionof`. Repros
  // captions-floats/cleveref_label_type_after_procedure_caption, captions-floats/cleveref_label_type_after_captionof.
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/cleveref_label_type_after_procedure_caption.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "float",
    &[],
    "<float class=\"ltx_algorithm\" framed=\"top\" labels=\"LABEL:alg:a_b\" xml:id=\"algorithmx1\"><tags><tag role=\"refnum\">Proc</tag></tags><toccaption>Procedure\u{a0}Proc</toccaption><caption><text font=\"bold\" xml:id=\"algorithmx1.1\">Procedure</text>\u{a0}Proc</caption><listing class=\"ltx_lst_numbers_left\" framed=\"topbottom\" xml:id=\"algorithmx1.2\"><listingline xml:id=\"algorithmx1.2.1\">x;</listingline></listing></float>",
  );
  assert!(!xml.contains("procedure]"), "{xml}");
  let xml = convert_counted(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/cleveref_label_type_after_captionof.tex"
    ),
    0,
    0,
  );
  assert_element(
    &xml,
    "table",
    &[],
    r#"<table align="center" inlist="lot" labels="LABEL:tab:a_b" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="creftype">table</tag><tag role="creftypecap">Table</tag><tag role="creftypeplural">tables</tag><tag role="creftypepluralcap">Tables</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>T</toccaption><caption><tag close=": ">Table 1</tag>T</caption></table>"#,
  );
  assert!(!xml.contains("appendixtable]"), "{xml}");
}
