//! Batch 56hi: the post stage keeps glossary phrases as NODES (Perl
//! Scan.pm:442, MakeIndex.pm:477-480, CrossRef.pm:906-925) and fills an
//! empty `ltx:glossaryref` from its entry (CrossRef.pm:469-476).
use crate::cluster::convert_and_post_clean;

/// acronym.sty's `\ac`/`\acl`/`\acs` emit an EMPTY `<ltx:glossaryref
/// show=…>`; CrossRef fills it with the entry's `phrase:<show>`. The port
/// never called `generateGlossaryRefTitle`, so every acronym rendered as its
/// key with `ltx_missing` ("NN (NN)" for "Neural Network (NN)") and an empty
/// tooltip (it read `phrase:description`; acronym's role is `definition`).
#[test]
fn acronym_refs_show_their_phrase() {
  let xml = convert_and_post_clean("tests/cluster_regressions/acronym_glossaryref_phrase.tex");
  assert!(!xml.contains("ltx_missing"), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "glossaryref",
    &[r#"show="long""#],
    r##"<glossaryref idref="id1" inlist="acronym" key="NN" show="long" title="Neural Network"><text class="ltx_glossary_long">Neural Network</text></glossaryref>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "glossaryref",
    &[r#"show="short""#],
    r##"<glossaryref idref="id1" inlist="acronym" key="NN" show="short" title="Neural Network"><text class="ltx_glossary_short">NN</text></glossaryref>"##,
  );
}

/// glossaries' `\newglossaryentry` fields are the DIGESTED values (Perl
/// glossaries.sty.ltxml:96 inserts `$value` after KeyVals::beDigested): a
/// `$\alpha$` name is math and an `\emph` description emphasis. The binding
/// absorbed each value's string, so the definition, the glossary list and the
/// `\gls` tooltip carried the TeX source (`angle \emph{in} radians`).
#[test]
fn glossaries_fields_keep_their_markup() {
  let xml = convert_and_post_clean("tests/cluster_regressions/glossaries_field_markup.tex");
  assert!(!xml.contains("\\emph"), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "glossaryentry",
    &[],
    r##"<glossaryentry fragid="glo.main.al" key="al" lists="main" xml:id="glo.main.al"><glossaryphrase key="al" role="label"><Math mode="inline" tex="\alpha" text="alpha" xml:id="m1a"><XMath><XMTok font="italic" name="alpha" role="UNKNOWN">α</XMTok></XMath></Math></glossaryphrase><glossaryphrase role="definition">angle <emph font="italic">in</emph> radians</glossaryphrase></glossaryentry>"##,
  );
}
