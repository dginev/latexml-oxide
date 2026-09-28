//! Red/green guards for perfect-kernel phase-57 batches from 57ah on: the K13 stage-2 binding
//! audit's residue (bindings that read differently from their real macro).
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// 57ah: amsart's `\authors`, `\shortauthors` and `\addresses` are parameterless storage macros
/// that `\author[#1]{#2}` accumulates (amscls/amsart.cls:460-477; the short form defaults to the
/// full name, `\@dblarg`, and an explicit `[]` adds none). The binding read an argument (Perl
/// ams_support.sty.ltxml:82-84, KPE #366), so `\authors` swallowed the next token — the text after
/// it, or a tabular's `\end` (2605.03453). amsart's `\maketitle` keeps `\and`, which the kernel title
/// code clears, so the names still read "A and B" after the title and a section.
/// Repro sectioning-frontmatter/ams_authors_are_storage_macros.
#[test]
fn ams_authors_are_storage_macros() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ams_authors_are_storage_macros.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let names = "Ann Author and Bob Writer and Cy Empty";
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    &format!(
      "<para xml:id=\"p1\"><p>Written by {names} (short: A.\u{a0}Author and B.\u{a0}Writer).\n\
       <tabular vattach=\"middle\"><tbody><tr><td align=\"center\">{names}</td></tr></tbody></tabular></p></para>"
    ),
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.p1""#],
    &format!("<para xml:id=\"S1.p1\"><p>Again: {names}.</p></para>"),
  );
}

/// 57ah: varioref's macros read optionals — `\vref*[text]{l}`, `\vpageref*[here][far]{l}`,
/// `\vrefrange[here]{a}{b}`, `\vpagerefrange*[here]{a}{b}` — and `\fullref`/`\reftextfaraway`
/// read a label (tools/varioref.sty:803-966, :123-125). The binding read none (Perl
/// varioref.sty.ltxml:24-27, KPE #367): `[here]` became the label `[`, the label printed as text,
/// and the two-parameter range texts used `#2`/`#3`. The page text stays dropped (no pages).
/// Repro singletons/varioref_reads_its_optionals.
#[test]
fn varioref_reads_its_optionals() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/varioref_reads_its_optionals.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let r = r#"<ref labelref="LABEL:sec:a"/>"#;
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.p1""#],
    &format!(
      "<para xml:id=\"S1.p1\"><p>See {r}, {r},\n{r}\u{2013}{r}, {r} and .\n\
       Also {r}, {r}, {r}\u{2013}{r} and {r}\u{2013}{r}.</p></para>"
    ),
  );
}

/// 57ah: attachfile loads hyperref (`\RequirePackageWithOptions{hyperref}`, attachfile.sty:40);
/// the binding did not (Perl attachfile.sty.ltxml:19-22, KPE #368), so a document loading only
/// attachfile had no `\href` or `\autoref`. Repro singletons/attachfile_loads_hyperref.
#[test]
fn attachfile_loads_hyperref() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/singletons/attachfile_loads_hyperref.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.p1""#],
    concat!(
      r#"<para xml:id="S1.p1"><p>See <ref class="ltx_refmacro_autoref" labelref="LABEL:sec:a" show="autoref"/>"#,
      r#" and <ref class="ltx_href" href="https://example.org">the site</ref>.</p></para>"#
    ),
  );
}

/// 57ah: svn-multi loads raw. The contrib stub made `\ifsvnmodified` a TeX conditional (real: a
/// two-argument chooser, svn-multi.sty:256), gave the storage macro `\svnurl` an argument (:269)
/// and made `\svnidlong` a no-op, so the keyword groups' `$…$` became math and the revision line
/// took `Error:expected:\fi`. Repro singletons/svn_multi_loads_raw.
#[test]
fn svn_multi_loads_raw() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/singletons/svn_multi_loads_raw.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<Math"), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Revision -2 by  (clean); url [].</p></para>"#,
  );
  // The document keywords are pdflatex's first pass (pass 2 reads them from the .aux); a file's
  // `\svnid` sets its keywords at once.
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>File 7 by bob.</p></para>"#,
  );
}

/// 57ah: savetrees processes its options. Under `bibnotes=tight` (the default; `subtle` clears
/// it) `\savetreesbibnote` drops the note and the token after it (savetrees.sty:342-346); the
/// binding read no option and always printed the note. Repro
/// singletons/savetrees_reads_its_options, default and `[subtle]`.
#[test]
fn savetrees_reads_its_options() {
  let source =
    include_str!("../../../tools/perfect_kernel/repros/singletons/savetrees_reads_its_options.tex");
  for (options, text) in [("", "2020."), ("[subtle]", "2020. The note.")] {
    let (stderr, xml) = convert_with(
      &source.replace(r"\usepackage{", &format!(r"\usepackage{options}{{")),
      None,
    );
    assert_eq!(error_count(&stderr), 0, "{options}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{options}: {stderr}");
    latexml::util::test::assert_element(
      &xml,
      "bibblock",
      &[],
      &format!(r#"<bibblock> A. Author, <emph font="italic">Title</emph>, {text}</bibblock>"#),
    );
  }
}

/// 57ah: `\FloatBarrier` begins with `\par` (placeins.sty:30); the binding made it a no-op (Perl
/// placeins.sty.ltxml:24, KPE #369), so the text around it ran together. Repro
/// singletons/floatbarrier_ends_the_paragraph.
#[test]
fn floatbarrier_ends_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/floatbarrier_ends_the_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>First paragraph text.</p></para>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>Second paragraph text.</p></para>"#,
  );
}

/// 57ah: the kernel's case changers read `O{} +m` (latex.ltx:22367-22378; the optional is the
/// locale keys). The engine's read `[1]` (Perl latex_constructs.pool.ltxml:5914-5935, KPE #370),
/// so `[lang=en]` was the argument. Repro singletons/case_changers_read_their_locale.
#[test]
fn case_changers_read_their_locale() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/case_changers_read_their_locale.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>A WORD B word C Word\nD WORD.</p></para>",
  );
  // The fronts re-brace what they read: `\MakeUppercase\foo` takes `\foo` whole, also inside
  // another case changer and a `\protected@edef` (latex.ltx:22367-22378).
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>E x ABC y F abcZ G ABC.</p></para>"#,
  );
}

/// 57ah: `\listoftodos[1][…]` reads its heading (todonotes.sty:323); the binding read nothing
/// (Perl todonotes.sty.ltxml:38, KPE #372), so the heading was printed as "[My Notes]". Repro
/// singletons/listoftodos_reads_its_heading.
#[test]
fn listoftodos_reads_its_heading() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/listoftodos_reads_its_heading.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>C.</p></para>"#,
  );
}

/// 57ah: every supertabular variant reads a `[pos]` it does not use (supertabular.sty:352-400);
/// the binding read none (Perl supertabular.sty.ltxml:24, :40, :60, :70; KPE #371), so `[t]`
/// became the column template and each row took two `Extra alignment tab` errors. Repro
/// alignment-bindings/supertabular_reads_its_position.
#[test]
fn supertabular_reads_its_position() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/supertabular_reads_its_position.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for id in ["tab1", "tab2"] {
    latexml::util::test::assert_element(
      &xml,
      "table",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        r#"<table xml:id="{id}"><tabular><tr><td align="left">a</td><td align="left">b</td></tr></tabular></table>"#
      ),
    );
  }
}
