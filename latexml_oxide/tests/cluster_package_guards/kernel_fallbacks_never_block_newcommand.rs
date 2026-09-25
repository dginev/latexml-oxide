//! Batch 56di: the beyond-Perl `\inst` author-superscript fallback
//! (KNOWN_PERL_ERRORS #201) is provided only inside a group around author
//! content (`\lx@author@withinst`, Perl's `\lx@author@withsup` shape), never
//! as a kernel-global definition — so a class's own `\newcommand\inst` is
//! never blocked, as in Perl, which has no `\inst`. ptptex.cls:616's
//! `\newcommand\inst[1]{\gdef\@inst{#1}}` (the affiliation STORE, never
//! typeset) was silently refused, the kernel superscript typeset the
//! affiliation into the body before `\maketitle`, and the stored
//! `<title>`/`<creator>` frontmatter landed after that paragraph — manptp's
//! three jing lines (RUST-ONLY; Perl 0 errors, frontmatter first).

/// The frontmatter is the document's first content: no `<para>` precedes
/// the `<title>`, and the title element is whole.
#[test]
fn ptptex_inst_store_keeps_the_frontmatter_first() {
  if !latexml::util::test::kpse_has("ptptex.cls") {
    return;
  }
  let tex = std::fs::read_to_string("tests/cluster_regressions/ptptex_inst_store_frontmatter.tex")
    .expect("fixture");
  let (stderr, xml) = super::convert_with(&tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "title", &[], r##"<title>Instruction Title</title>"##);
  let title_at = xml.find("<title>").expect("title");
  let first_para = xml.find("<para").unwrap_or(usize::MAX);
  let first_creator = xml.find("<creator").expect("creator");
  assert!(
    title_at < first_creator && title_at < first_para,
    "frontmatter must precede every body paragraph:\n{xml}"
  );
  // What the class's `\@maketitle` still typesets beyond the captured stores
  // (its journal banner, a "(Received )" label) is deposited AFTER the
  // frontmatter by `\lx@deposit@maketitle`, never before it.
  assert!(
    xml.find("<pubnote>").expect("pubnote") < first_para,
    "the frontmatter is complete before any deposited paragraph:\n{xml}"
  );
  // The affiliations are frontmatter contacts linked from the authors'
  // `$^{n,}$` superscripts, and the abstract, received date and
  // publication note carry their kinds.
  latexml::util::test::assert_element(
    &xml,
    "creator",
    &[],
    r##"<creator role="author"><personname>Shin-Ichiro <text font="smallcaps">Tomonaga</text></personname><contact name="Note: " role="note">Note A.</contact><contact name="Affiliation: " role="affiliation">Physics Dept, Tokyo</contact></creator>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "creator",
    &["before="],
    r##"<creator before="  " role="author"><personname>Hideki <text font="smallcaps">Yukawa</text></personname><contact name="Note: " role="note">Note B.</contact><contact name="Affiliation: " role="affiliation">Yukawa Institute, Kyoto</contact></creator>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "abstract",
    &[],
    r##"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>This is the abstract text.</p></abstract>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "date",
    &[],
    r##"<date name="Received " role="received">April 1, 2004</date>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "subtitle",
    &[],
    r##"<subtitle>Sub Version</subtitle>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "pubnote",
    &[],
    r##"<pubnote>Vol. 120, No. 5, November 2008</pubnote>"##,
  );
}
