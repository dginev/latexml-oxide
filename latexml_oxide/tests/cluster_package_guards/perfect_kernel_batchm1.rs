//! Red/green guards for the TeX Live manuals' regressions (docs/perfect_kernel/MANUAL_REGRESSIONS.md).
//!
//! Cluster A: under raw classes a binding's name prefix took a published class that is a class of its own (`bookest` ->
//! `book`), and its commands were undefined. A prefix marks an author's renamed copy beside the document, or a sibling
//! shipped with the binding's class; `content.rs` `is_unrelated_distribution_class`, OD #444.
//!
//! The batch's other causes each have one guard, on its one code path: the arity of the `\multicolumn` lock
//! (`state.rs`), the tblr read-ahead's tab marks (`tabularray_sty.rs`), the block instances of the latex-lab binding, the
//! test of a known xcolor model (`xcolor_sty.rs`), a `<titlepage>` for a later late abstract's cover line
//! (`cover_order.rs`), `\selectfont` out of `\nullfont` (`sect13.rs`), a template's own text in `typesets_content`
//! (`util_text.rs`), multibib's style (`multibib_sty.rs`), and ijsra's slots and abstract (`ijsra_cls.rs`).
use latexml::util::test::{convert_with, rng_error_count};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch51::kpsewhich_has,
  perfect_kernel_batch78i::html_of,
  perfect_kernel_batch86g::root_order,
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $name,
      ".tex"
    ))
  };
}

/// The arXiv profile reads classes raw (`rawclasses`), as the manuals' sweep does.
fn log_of(tex: &str) -> String { cli_conversion(tex, false).0 }

#[test]
fn a_published_class_named_after_a_binding_is_read_as_itself() {
  let log = log_of(repro!("distribution_class_named_after_a_binding"));
  assert!(!log.contains("Can't find binding"), "{log}");
  assert_eq!(error_count(&log), 0, "{log}");
}

#[test]
fn a_class_beside_the_bindings_class_keeps_the_binding() {
  let log = log_of(repro!(
    "distribution_class_beside_its_binding_is_its_family"
  ));
  assert!(
    log.contains("Can't find binding for class jmlrbook (using jmlr)"),
    "{log}"
  );
  assert_eq!(error_count(&log), 0, "{log}");
}

#[test]
fn a_renamed_copy_beside_the_document_keeps_the_binding() {
  let log = log_of(repro!(
    "renamed_class_copy_beside_the_document_keeps_the_binding"
  ));
  assert!(
    log.contains("Can't find binding for class bookofmine (using book)"),
    "{log}"
  );
}

macro_rules! manual_repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

fn converted(tex: &str) -> (String, String) { convert_with(tex, Some("ar5iv.sty")) }

/// `tex`'s output with its ids and the indentation between elements taken out, to read whole elements at a glance.
fn plain(xml: &str) -> String {
  let at = xml.find("<document").unwrap_or(0);
  let mut plain = String::new();
  let mut rest = &xml[at..];
  while let Some(id) = rest.find(" xml:id=\"") {
    plain.push_str(&rest[..id]);
    let value = id + " xml:id=\"".len();
    rest = &rest[value + rest[value..].find('"').expect("a closed xml:id") + 1..];
  }
  plain.push_str(rest);
  plain
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .replace("> <", "><")
}

/// C1: sltables.sty:193's one-argument `\multicolumn` is its table's own command, kept (Perl pool:3702 does not lock it);
/// the kernel's is back after the table's group. sltables-doc-en, sweep #150's 11 errors. A three-argument copy stays
/// refused: `perfect_kernel_batch56::raw_multicolumn_redefinition_is_dropped`; one with no parameters too:
/// [`a_multicolumn_of_no_parameters_stays_refused`].
#[test]
fn a_package_multicolumn_of_another_arity_is_its_own() {
  let (log, xml) = converted(manual_repro!(
    "alignment-bindings/multicolumn_of_another_arity_is_the_packages_own.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = plain(&xml);
  assert!(
    xml.contains(
      r#"<td align="center" border="b" class="ltx_nopad_r" colspan="3"><inline-block vattach="top"><p>At the end</p></inline-block></td>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<td align="center" colspan="2">Y</td>"#),
    "{xml}"
  );
}

/// C1: a `\multicolumn` declaring no parameters may read any number, so the arity lock refuses it (`another_interface`'s
/// `count > 0`): here latex.ltx's command behind one more name, whose `\@mkpream` has no preamble to run against. Its shape
/// that prints something else is the limit `multicolumn_of_no_declared_parameters_is_refused_red`.
#[test]
fn a_multicolumn_of_no_parameters_stays_refused() {
  let (log, xml) = converted(manual_repro!(
    "alignment-bindings/multicolumn_of_no_parameters_passes_to_the_binding.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    plain(&xml).contains(r#"<td align="center" colspan="2">Span</td>"#),
    "{xml}"
  );
}

/// C2: every read of a tblr body in a `p` cell or a `p` span (the width read of `tblr`, a `talltblr`, a nested one) leaves
/// the inner `&`s alone. ProfSio-doc-fr's 10 errors (awesomebox's `{tipblock}`).
#[test]
fn a_tblr_in_a_p_cell_reads_its_tab_marks_safely() {
  let (log, xml) = converted(manual_repro!(
    "alignment-bindings/tblr_in_a_p_cell_reads_its_tab_marks_safely.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = plain(&xml);
  for row in [["A", "B"], ["C", "D"], ["E", "F"], ["G", "H"]] {
    assert!(
      xml.contains(&format!(
        r#"<tr><td align="left">{}</td><td align="left">{}</td></tr>"#,
        row[0], row[1]
      )),
      "{xml}"
    );
  }
  assert!(
    xml.contains(r#"</tabular></td><td align="left">I</td></tr>"#),
    "{xml}"
  );
  assert!(xml.contains("<p>After.</p>"), "{xml}");
}

/// C3: the latex-lab block binding declares the file's `block` instances: keytheorems edits `theoremblock-0` and
/// `listblock-1` for a theorem with margins (keytheorems-doc's 4 errors).
#[test]
fn the_block_instances_of_the_file_are_declared() {
  let (log, xml) = converted(manual_repro!(
    "package-bindings/keytheorems_block_instances_are_the_files.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    plain(&xml).contains(
      r#"<para><p><text font="italic">There are infinitely many primes.</text></p></para></theorem>"#
    ),
    "{xml}"
  );
}

/// C4: a model raw code defines (`\XC@mod@<model>`, xcolor.sty:565's test) is known (the colorspace manual's 8 errors;
/// Perl errs, Color.pm:59-61), and its colour is lost: one counted Warning for the model, in `\definecolor`, `\color[…]`,
/// `\textcolor[…]` and `\colorbox[…]` alike, the words kept; a spot colour by name keeps its alternate, no Warning. A
/// model nothing defines stays the Error it is, also with its name left `\relax` by a test.
#[test]
fn a_color_model_is_known_by_its_xcolor_definition() {
  let (log, xml) = converted(manual_repro!(
    "package-bindings/xcolor_model_raw_code_defines_is_known.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log
      .contains("Warning:color_model:fooshaded The color model 'fooshaded' is defined by raw code"),
    "{log}"
  );
  assert!(
    plain(&xml).contains(
      r##"<p>Before shaded and after. Third. <text backgroundcolor="#000000">Boxed.</text><text color="#008033">Spot.</text></p>"##
    ),
    "{xml}"
  );
  let (log, xml) = converted(manual_repro!(
    "package-bindings/xcolor_model_nobody_defines_is_an_error.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains("Unrecognized color model 'nosuchmodel'"),
    "{log}"
  );
  assert!(plain(&xml).contains("<p>word</p>"), "{xml}");
}

/// C4: the Warning is per model, whatever repeats it (a title, an author, a heading and its toc copy, a note, a cell), and
/// the `&spot` model of colorspace.sty:78's raw storage, read after a `\definespotcolor` in a group, is one too.
#[test]
fn a_raw_color_model_warns_once_per_model() {
  let (log, xml) = converted(manual_repro!(
    "package-bindings/xcolor_raw_models_warn_once_each.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  for model in ["fooshaded", "&spot"] {
    assert_eq!(
      log
        .matches(&format!("Warning:color_model:{model} "))
        .count(),
      1,
      "{log}"
    );
  }
  let xml = plain(&xml);
  for words in [
    "T title",
    "A Author",
    "S head",
    "Note inner.",
    ">cell<",
    "spot and again.",
  ] {
    assert!(xml.contains(words), "{words}: {xml}");
  }
}

/// C4 under color alone: its model is the driver macro `\color@<model>` (color.sty:118), in `\definecolor` and in the
/// optional arguments alike (these were Perl's Error), and with no alternate from a binding the colour is lost, as under
/// xcolor. The alternate's route: `perfect_kernel_batch72a::spot_color_defined_first_takes_the_alternate`.
#[test]
fn a_driver_color_model_without_an_alternate_warns() {
  let (log, xml) = converted(manual_repro!(
    "package-bindings/color_driver_model_without_alternate_warns.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("Warning:color_model:mymodel "), "{log}");
  assert!(
    plain(&xml)
      .contains(r##"<p>A one two three <text backgroundcolor="#000000">four</text>.</p>"##),
    "{xml}"
  );
}

/// xcolor.sty.ltxml:131-142's `tHsb` converts (black in silence before; pdflatex paints 0.5 1 0, then 0.49998 0 0.5).
#[test]
fn the_thsb_color_model_converts() {
  let (log, xml) = converted(manual_repro!(
    "package-bindings/xcolor_thsb_model_converts.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    plain(&xml).contains(
      r##"<p>A <text color="#80FF00">one</text> B <text color="#800080">two</text>.</p>"##
    ),
    "{xml}"
  );
}

/// C6: the cover line between two late abstracts is a continued `<titlepage>`; as a `<para>` it made the abstract and
/// pubnote after it invalid (simples-matrices, annee-scolaire, paresse, 2 jing lines each).
#[test]
fn a_cover_line_between_two_abstracts_is_a_titlepage() {
  let (log, xml) = converted(manual_repro!(
    "sectioning-frontmatter/cover_line_between_two_abstracts_is_a_titlepage.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(root_order(&xml), [
    "title",
    "creator",
    "titlepage[block: ]",
    "abstract",
    "titlepage[block: ]",
    "abstract",
    "pubnote",
    "para",
    "section"
  ]);
  assert_eq!(
    plain(&xml)
      .matches(r#"<titlepage class="ltx_titlepage_continued">"#)
      .count(),
    2,
    "{xml}"
  );
  assert_eq!(rng_error_count(&xml).unwrap_or(0), 0, "{xml}");
}

/// C7: `\selectfont` of a family the font table does not map leaves `\nullfont` (prerex/chart: every cell, silently);
/// pgf's next `\nullfont` keeps what is written at picture level dropped.
#[test]
fn selectfont_of_an_unmapped_family_leaves_nullfont() {
  let (log, xml) = converted(manual_repro!(
    "fonts-nfss/selectfont_of_an_unmapped_family_leaves_nullfont.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for word in ["<text>Hello</text>", "<text>Direct</text>", ">Dings</text>"] {
    assert!(
      xml.contains(word) || plain(&xml).contains(word),
      "{word}: {xml}"
    );
  }
  assert!(!xml.contains("stray"), "{xml}");
}

/// C8: a block whose only text is a constructor's own template (`\LaTeX`, `\TeX`) shows something (niepraschk-eso-pic);
/// a strut and a rule alone do not.
#[test]
fn a_template_of_its_own_text_is_content() {
  let (log, xml) = converted(manual_repro!(
    "package-bindings/textpos_block_of_a_logo_alone.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches(r#"class="ltx_LaTeX_logo""#).count(), 2, "{xml}");
  assert_eq!(xml.matches(r#"class="ltx_TeX_logo""#).count(), 1, "{xml}");
  assert!(xml.contains(">Word</text>"), "{xml}");
  assert_eq!(xml.matches("<para ").count(), 4, "{xml}");
}

/// C9: `\bibliographystyle<suffix>` reaches `BIBSTYLE`, so the fields its `.bst` declares print: quantum.bst's `urldate`
/// as `accessed` (quantum-bibliographystyle-demo).
#[test]
fn a_multibib_style_reaches_the_bst_fields() {
  let html = html_of(
    manual_repro!("index-bib/multibib_style_reaches_the_bst_fields.tex"),
    &[(
      "multibib_style_reaches_the_bst_fields.bib",
      manual_repro!("index-bib/multibib_style_reaches_the_bst_fields.bib"),
    )],
  );
  assert!(
    html.contains(r#"ltx_bib_accessed">12.07.2019</span>"#),
    "{html}"
  );
}

/// C9: each list reads its own style, keyed by the suffix `\newcites` declares: `\bibliographystyle{alpha}
/// \bibliographystyleweb{plain}` before both lists, as pdflatex's `[1]` and `[Zet01]` (the alpha label prints `[ZET01]` here;
/// that casing is apart). The shape of opticajnl.cls:288's full-reference list: 2607.03975, 2308.15957. The web list comes first
/// in this document, so swapped styles go red.
#[test]
fn each_multibib_list_reads_its_own_style() {
  let html = html_of(
    manual_repro!("index-bib/multibib_each_list_reads_its_own_style.tex"),
    &[(
      "multibib_each_list_reads_its_own_style.bib",
      manual_repro!("index-bib/multibib_each_list_reads_its_own_style.bib"),
    )],
  );
  let label = |kind: &str, text: &str| {
    format!(r#"<span class="ltx_tag {kind} ltx_role_refnum ltx_tag_bibitem">{text}</span>"#)
  };
  let numbered = html.find(&label("ltx_bib_key", "[1]")).expect(&html);
  let alpha = html.find(&label("ltx_bib_abbrv", "[ZET01]")).expect(&html);
  assert!(numbered < alpha, "{html}");
}

/// C9: a suffix's style never takes `BIBSTYLE` from the main list, even when the main list's has the same name as an earlier
/// suffix's. The main list prints first here, numbered, then `web`'s, then `alt`'s under alpha.
#[test]
fn the_main_list_keeps_its_style_beside_a_same_named_suffix() {
  let html = html_of(
    manual_repro!("index-bib/multibib_main_style_beside_a_same_named_suffix.tex"),
    &[(
      "multibib_each_list_reads_its_own_style.bib",
      manual_repro!("index-bib/multibib_each_list_reads_its_own_style.bib"),
    )],
  );
  let numbered = html.find("ltx_bib_key").expect(&html);
  let alpha = html.find("ltx_bib_abbrv").expect(&html);
  assert!(numbered < alpha, "{html}");
}

/// C4: raw storage naming any model is read safely: `named` doesn't come back to the same lookup (the stack overflow on M2 r2),
/// a model no code defined has no Error, and a stored `HTML` keeps its colour where `\XC@mod@HTML` exists.
#[test]
fn raw_color_storage_of_any_model_is_read_safely() {
  let (log, xml) = converted(manual_repro!(
    "package-bindings/xcolor_raw_storage_of_any_model_is_read_safely.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for word in ["word", "zed", "ex"] {
    assert!(xml.contains(word), "{word}: {xml}");
  }
}

/// C4: xcolor.sty's own `natural` is no model of raw code: pdflatex errs on it, and so do we, with no Warning for it.
#[test]
fn an_internal_xcolor_model_stays_an_error() {
  let (log, xml) = converted(manual_repro!(
    "package-bindings/xcolor_internal_model_stays_an_error.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(xml.contains("word"), "{xml}");
}

/// C5: ijsra.cls's frontmatter is `\def` slots: under its binding (ijsra_cls.rs) the document's `\def\abstract{…}` stands, and
/// `{IJSRAabstract}`, the opening's and a document's own later one, is `ltx:abstract` with its `ltx:keywords`. No `\def\abstract`,
/// no abstract. The whole manual, in a directory of its own, with its printed text whole. Under any other class `\abstract` is
/// locked as it was: the value refused, the capture's error (`abstract_value_under_another_class_is_refused`).
#[test]
fn the_ijsra_abstract_is_semantic() {
  let (log, xml) = converted(manual_repro!(
    "macro-state/abstract_value_under_another_class_is_refused.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(log.contains(r"\endlx@list"), "{log}");
  assert!(!xml.contains("the abstract text"), "{xml}");
  if !kpsewhich_has("ijsra.cls") {
    return;
  }
  let (log, xml) = cli_conversion(
    manual_repro!("class-bindings/ijsra_abstract_slot_is_the_abstract.tex"),
    false,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  let xml = plain(&xml);
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Abstract"><p>Printed abstract words.</p></abstract><abstract inlist="toc" name="Abstract"><p><text xml:lang="fr">Mots en francais.</text></p></abstract><keywords name="Keywords: ">one, two</keywords><keywords name="Mots cles: "><text xml:lang="fr">trois, quatre</text></keywords><chapter"#
    ),
    "{xml}"
  );
  let (log, xml) = cli_conversion(
    manual_repro!("class-bindings/ijsra_no_abstract_prints_none.tex"),
    false,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(!xml.contains("<abstract"), "{xml}");
  assert!(xml.contains("Body text."), "{xml}");
  // (the manual, if this TeX Live has its documentation: read as a file, converted in cli_conversion's own tempdir)
  let dist = std::process::Command::new("kpsewhich")
    .arg("--var-value=TEXMFDIST")
    .output()
    .expect("run kpsewhich");
  let dist = String::from_utf8_lossy(&dist.stdout).trim().to_string();
  let Ok(manual) = std::fs::read_to_string(format!("{dist}/doc/latex/ijsra/ijsra.tex")) else {
    return;
  };
  let (log, xml) = cli_conversion(&manual, false);
  assert_eq!(error_count(&log), 0, "{log}");
  let xml = plain(&xml);
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Abstract"><p>This is a documentation of the class <text font="typewriter">ijsra</text> which is used for the <emph font="italic">International Journal of Student Research in Archaeology</emph>. Read this documentation carefully. When you start editing do it according to the structure shown in <ref labelref="LABEL:document" show="creftype~creflabel"/>.</p></abstract><abstract inlist="toc" name="Abstract"><p><text xml:lang="fr">Abstract<note"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<keywords name="Mots clés: "><text xml:lang="fr">keywords in French, "#),
    "{xml}"
  );
}
