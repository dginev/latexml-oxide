//! Red/green guards for batch 68x: raw-class frontmatter rows from the int16 A/B. Item 1 — K11 for every class read
//! raw: `input_definitions` fires the title-page store reroutes at the end of a class load for the class a binding
//! read raw (apa6, apa, beilstein, lni, …), as for one the rawclasses lever reads, the binding's own setters staying its
//! own (68u round 3 called it from four bindings by hand; 2608.21389, 2102.04789). Item 2 — an aguplus binding over
//! the raw class: `\affil`, `\authoraddr`, `\altaffilmark`/`\altaffiltext` and the dates as the AAS binding makes them,
//! and natbib's binding for the class's copy of natbib 7.0 (1107.5305, 1203.5042, physics9702009, 1010.3998). Item 3 —
//! apa6's `\note` and `\authornote` as apa7's binding makes them, the mask honoured (2406.08022). Item 4 — cjs-rcs-article's
//! environ-collected abstracts (in their languages) and keywords, and babel's `\iflanguage` expandable (2608.13762).
//! Item 5 — smfart's `{altabstract}` a second abstract (2506.20120); projlib-author's `\address` lines the author's
//! (2609.02711). Item 6 — mathtools.sty's internals papers and witharrows call after `\MHInternalSyntaxOn`.
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{assert_element, normalize_markup};

use super::perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with};

/// class-bindings/lni_subtitle_through_k11: lni.cls's `\subtitle` (a store only its refused `\maketitle` reads) is the
/// frontmatter's subtitle though the class is read by the lni binding, not the lever (pdflatex: "The Title" over "A
/// Subtitle"). Before 68u r3 a bold paragraph; 68u r3 by an explicit call in the binding, 68x by the one rule.
#[test]
fn a_class_a_binding_reads_raw_reroutes_its_stores() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/lni_subtitle_through_k11.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(&xml, "subtitle", &[], "<subtitle>A Subtitle</subtitle>");
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Ann Able</personname><contact name="Email: " role="email">ann@example.org</contact><contact name="Affiliation: " role="affiliation">First University</contact></creator>"#,
  );
}

/// `expected` with its invisible characters spelled out: `{nbsp}` a no-break space (a contact's label ends in one,
/// `name="Affiliation:{nbsp}"`; `~`), `{em}` an em space (the separator `before` a creator).
fn spelled(expected: &str) -> String {
  expected
    .replace("{nbsp}", "\u{a0}")
    .replace("{em}", "\u{2003}")
}

/// class-bindings/aguplus/aguplus_author_block_and_citations: each `\author` carries the `\affil` after it, the
/// `\authoraddr` given before the authors is the author's it names, and the citations are natbib's in square brackets
/// (pdflatex: "Ann Able / First Institute, Town / Bob Baker / Second Laboratory, City"; "As Able [1990] show, and
/// others [Baker et al., 1991]"; the address and "Received 1 May 1999" after the references). Before 68x the
/// affiliations were body text, the address lost and 6 "Missing number" warnings (1107.5305, 1203.5042).
#[test]
fn aguplus_authors_carry_affiliations_and_addresses() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aguplus/aguplus_author_block_and_citations.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "para",
      "Sx1.p1",
      &spelled(
        r#"<para xml:id="Sx1.p1"><p>As <cite class="ltx_citemacro_citet"><bibref bibrefs="key1" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","><bibrefphrase>[</bibrefphrase><bibrefphrase>]</bibrefphrase></bibref></cite> show, and others <cite class="ltx_citemacro_citep">[<bibref bibrefs="key2" separator=";" show="AuthorsPhrase1Year" yyseparator=","><bibrefphrase>, </bibrefphrase></bibref>]</cite>, see <cite class="ltx_citemacro_cite"><bibref bibrefs="key1" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","><bibrefphrase>[</bibrefphrase><bibrefphrase>]</bibrefphrase></bibref></cite> and <cite class="ltx_citemacro_cite">[see <bibref bibrefs="key2" separator=";" show="AuthorsPhrase1Year" yyseparator=","><bibrefphrase>, </bibrefphrase></bibref>, p.{nbsp}2]</cite>.</p></para>"#,
      ),
    )],
  );
  assert_eq!(xml.matches("<creator").count(), 2, "{xml}");
  assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    &spelled(
      r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation:{nbsp}" role="affiliation">First Institute, Town</contact><contact name="Address:{nbsp}" role="address">A. Able, First Institute, Main Street 1, Town (able@example.org)</contact></creator>"#,
    ),
  );
  assert_element(
    &xml,
    "creator",
    &[spelled(r#"before="{em}{em}""#).as_str()],
    &spelled(
      r#"<creator before="{em}{em}" role="author"><personname>Bob Baker</personname><contact name="Affiliation:{nbsp}" role="affiliation">Second Laboratory, City</contact></creator>"#,
    ),
  );
  assert_element(
    &xml,
    "date",
    &[],
    r#"<date name="Received" role="received">1 May 1999</date>"#,
  );
  assert!(!xml.contains("First Institute, Town</p>"), "{xml}");
}

/// class-bindings/aguplus/aguplus_altaffil_and_spelled_addresses: `\altaffilmark`/`\altaffiltext` give the marked
/// authors their affiliations, and each `\authoraddr` goes to the author it names (pdflatex: "A. Able,1 B. Baker,1,2
/// and C. Cole2", the footnotes "1 First Institute, Town." "2 Second Laboratory, City.", the addresses after the
/// references). Before 68x no affiliation or address (1010.3998).
#[test]
fn aguplus_altaffiliations_and_addresses_go_to_their_authors() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aguplus/aguplus_altaffil_and_spelled_addresses.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(xml.matches("<creator").count(), 3, "{xml}");
  for creator in [
    r#"<creator role="author"><personname>A. Able</personname><contact name="Alternate Affiliation:{nbsp}" role="altaffiliation">First Institute, Town.</contact><contact name="Address:{nbsp}" role="address">A. Able, First Institute, Town; email: able@example.org</contact></creator>"#,
    r#"<creator before="{em}{em}" role="author"><personname>B. Baker</personname><contact name="Alternate Affiliation:{nbsp}" role="altaffiliation">First Institute, Town.</contact><contact name="Alternate Affiliation:{nbsp}" role="altaffiliation">Second Laboratory, City.</contact></creator>"#,
    r#"<creator before="{em}{em}" role="author"><personname>C. Cole</personname><contact name="Alternate Affiliation:{nbsp}" role="altaffiliation">Second Laboratory, City.</contact><contact name="Address:{nbsp}" role="address">C. Cole, Second Laboratory, City; email: cole@example.org</contact></creator>"#,
  ] {
    let creator = spelled(creator);
    assert!(
      normalize_markup(&xml).contains(&creator),
      "{creator}\n{xml}"
    );
  }
}

/// class-bindings/apa6_note_and_authornote: apa6.cls:776 `\note` and :779 `\authornote` are the frontmatter's notes
/// (pdflatex: "First University / Author Note / Correspondence …", then "Author Note / Supported by a grant."); under
/// `mask` neither (the class prints neither). Before 68x the `\note` was lost (2406.08022).
#[test]
fn apa6_note_and_authornote_are_frontmatter_notes() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/apa6_note_and_authornote.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "note",
    &[r#"role="note""#],
    r#"<note role="note">Author Note <break/>Correspondence concerning this article: ann@example.org.</note>"#,
  );
  assert_element(
    &xml,
    "note",
    &[r#"role="authornote""#],
    r#"<note role="authornote">Supported by a grant.</note>"#,
  );
  let masked = assert_elements(
    &tex.replace("[man]{apa6}", "[man,mask]{apa6}"),
    RAW,
    (0, 0),
    &[],
  );
  assert!(!masked.contains("<note"), "{masked}");
}

/// babel-lang/iflanguage_expands_in_edef: `\iflanguage` of a language the format knows expands inside an `\edef`
/// (pdflatex: "Ann and Bob. Ann et Bob."). Before 68x (and in Perl, KNOWN_PERL_ERRORS #861) `\@@@@lang` was undefined
/// there, an Error (cjs-rcs-article.cls:368, 2608.13762).
#[test]
fn babel_iflanguage_expands_in_an_edef() {
  let xml = assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/babel-lang/iflanguage_expands_in_edef.tex"),
    RAW,
    (0, 0),
    &[],
  );
  assert_element(
    &xml,
    "p",
    &[],
    "<p>Ann and Bob.\n<text xml:lang=\"fr\">Ann et Bob.</text></p>",
  );
}

/// class-bindings/cjs_rcs_article_abstracts_keywords: the class's environ-collected `{englishabstract}`,
/// `{frenchabstract}` and `{keywords}` are the frontmatter's two abstracts, each in its language and named as the class
/// names it, and its keywords joined by "; " (pdflatex: "Abstract The English abstract." "Résumé Le résumé français."
/// "Keywords Functional data; Time series."). Before 68x all three were lost (2608.13762).
#[test]
fn cjs_rcs_abstracts_and_keywords_are_frontmatter() {
  // (the three Warnings are memoir.cls's, which the class loads: :5454 `\newif \@iffirstamp`, :7811
  // `\addtodef{\endthebibliography}` — its deprecation warning pdflatex prints too — on a kernel environment end)
  let xml = assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/cjs_rcs_article_abstracts_keywords.tex"
    ),
    RAW,
    (0, 3),
    &[
      "Warning:misdefined:\\@iffirstamp",
      "Class memoir Warning: \\addtodef is marked deprecated",
      "`\\endthebibliography' is not a macro",
    ],
    &[
      (
        "abstract",
        "abstract1",
        r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1" xml:lang="en"><p>The English abstract.</p></abstract>"#,
      ),
      (
        "abstract",
        "abstract2",
        r#"<abstract inlist="toc" name="Résumé" xml:id="abstract2" xml:lang="fr"><p>Le résumé français.</p></abstract>"#,
      ),
    ],
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    r#"<keywords name="Keywords">Functional data; Time series</keywords>"#,
  );
  assert_eq!(xml.matches("<item ").count(), 2, "{xml}");
}

/// class-bindings/smfart_altabstract: smfart.cls:568-583 `{altabstract}` is the frontmatter's second abstract, in the
/// other language, named by the class's caption, the alternative title leading its text (pdflatex: "Résumé. — Le
/// résumé." "Abstract (The title). — The other abstract."); the document's abstract of the same name stays. Before
/// 68x the deposited title page's body text (2506.20120: lost).
#[test]
fn smfart_altabstract_is_a_second_abstract() {
  let xml = assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/smfart_altabstract.tex"),
    RAW,
    (0, 0),
    &[(
      "abstract",
      "abstract2",
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract2" xml:lang="en"><p>(<emph font="italic">The title</emph>) The other abstract.</p></abstract>"#,
    )],
  );
  assert_eq!(xml.matches("<abstract ").count(), 2, "{xml}");
  assert_eq!(xml.matches("The other abstract.").count(), 1, "{xml}");
}

/// class-bindings/projlib_author_address: projlib-author.sty's `\address`/`\email` lines are the author's they follow
/// (pdflatex: "Ann Able / First University, Main Street 1, Town / ann@example.org / Bob Baker / Second Institute, City").
/// Before 68x both authors without them (2609.02711).
#[test]
fn projlib_author_lines_are_the_authors_contacts() {
  // (the Warning is colorist-fancy's `\setlist`, as before)
  let xml = assert_elements_with(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/projlib_author_address.tex"),
    RAW,
    (0, 1),
    &["Missing number (Dimension)"],
    &[],
  );
  assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    &spelled(
      r#"<creator role="author"><personname>Ann Able</personname><contact name="Address:{nbsp}" role="address">First University, Main Street 1, Town</contact><contact name="Email:{nbsp}" role="email">ann@example.org</contact></creator>"#,
    ),
  );
  assert_element(
    &xml,
    "creator",
    &[spelled(r#"before="{em}{em}""#).as_str()],
    &spelled(
      r#"<creator before="{em}{em}" role="author"><personname>Bob Baker</personname><contact name="Address:{nbsp}" role="address">Second Institute, City</contact></creator>"#,
    ),
  );
}

/// package-bindings/mathtools_internals_after_internal_syntax: `\MT_start_cases:nnnn`, `\MT_matrix_begin:N`/`\MT_matrix_end:`,
/// `\MT_delim_default_inner_wrappers:n` and witharrows's `show_only_refs` test work as with mathtools.sty (pdflatex 0
/// errors). Before 68x 18 Errors (1906.10268, 1905.08664, 1704.08847, 2012.15617).
#[test]
fn mathtools_internals_after_internal_syntax_are_defined() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/mathtools_internals_after_internal_syntax.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  for tex in [
    r#"tex="f(x)=\begin{cases}1&amp;x&gt;0\\&#10;0&amp;x\leq 0\end{cases}""#,
    r#"tex="\mathopen{\langle}u,v\mathclose{\rangle}""#,
  ] {
    assert!(xml.contains(tex), "{tex}\n{xml}");
  }
  assert!(xml.contains("<XMTok") && !xml.contains("ERROR"), "{xml}");
}
