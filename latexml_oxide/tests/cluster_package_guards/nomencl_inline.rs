//! Batch 56cl: nomencl's list is a makeindex product (`\nomenclature`
//! writes `\jobname.nlo`, `\printnomenclature` inputs `\jobname.nls`,
//! nomencl.sty:227-245/:277-282), lost silently by the raw load in both
//! engines (the five shipped samples at 17-37 % recall, 0 errors). The
//! binding turns each entry into a `<glossarydefinition>` and the list
//! into a `<glossary role="nomenclature">` that MakeGlossary fills with
//! every definition (OXIDIZED_DESIGN_DIVERGENCES #234).
use crate::cluster::convert_and_post_clean;

const TEX: &str = "\\documentclass{article}\n\\usepackage[nocfg]{nomencl}\n\\makenomenclature\n\\begin{document}\n\\section*{Main equations}\n\\begin{equation}\n  a=\\frac{N}{A}\n\\end{equation}%\n\\nomenclature{$a$}{The number of angels per unit area\\nomrefeq}%\n\\nomenclature{$N$}{The number of angels per needle point}%\n\\nomenclature[z]{$A$}{The area of the needle point}%\n\\printnomenclature\n\\end{document}\n";

/// The core XML: each entry is a definition with its sort/name/description
/// phrases (the `\nomrefeq` entry carries ", see equation (1)"), and the
/// list is an empty titled glossary for the post stage.
#[test]
fn nomenclature_entries_become_glossary_definitions() {
  let (stderr, xml) = super::convert(TEX, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "glossarydefinition",
    &[r#"key="nomencl.1""#],
    r##"<glossarydefinition inlist="nomenclature" key="nomencl.1"><glossaryphrase key="nomencl.1" role="sort">a<Math mode="inline" tex="a" text="a" xml:id="Sx1.p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></glossaryphrase><glossaryphrase key="nomencl.1" role="name"><Math mode="inline" tex="a" text="a" xml:id="Sx1.p1.m2"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></glossaryphrase><glossaryphrase key="nomencl.1" role="description">The number of angels per unit area, see equation (1)</glossaryphrase></glossarydefinition>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "glossarydefinition",
    &[r#"key="nomencl.3""#],
    r##"<glossarydefinition inlist="nomenclature" key="nomencl.3"><glossaryphrase key="nomencl.3" role="sort">z<Math mode="inline" tex="A" text="A" xml:id="Sx1.p1.m5"><XMath><XMTok font="italic" role="UNKNOWN">A</XMTok></XMath></Math></glossaryphrase><glossaryphrase key="nomencl.3" role="name"><Math mode="inline" tex="A" text="A" xml:id="Sx1.p1.m6"><XMath><XMTok font="italic" role="UNKNOWN">A</XMTok></XMath></Math></glossaryphrase><glossaryphrase key="nomencl.3" role="description">The area of the needle point</glossaryphrase></glossarydefinition>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "glossary",
    &[],
    r##"<glossary lists="nomenclature" role="nomenclature" xml:id="glo.nomenclature"><title>Nomenclature</title></glossary>"##,
  );
}

/// After MakeGlossary: every entry is listed, sorted by prefix + symbol
/// (the `z`-prefixed `A` last), though nothing references them; each label
/// is the symbol's `Math`, cloned (batch 56hi, Perl MakeIndex.pm:477-478). The
/// fixture file is `TEX` verbatim (the post helper takes a path).
#[test]
fn printnomenclature_lists_every_entry() {
  let xml = convert_and_post_clean("tests/cluster_regressions/nomencl_printnomenclature.tex");
  latexml::util::test::assert_element(
    &xml,
    "glossary",
    &[],
    r##"<glossary fragid="glo.nomenclature" lists="nomenclature" role="nomenclature" xml:id="glo.nomenclature"><title>Nomenclature</title><glossarylist><glossaryentry fragid="glo.nomenclature.nomencl.1" key="nomencl.1" lists="nomenclature" xml:id="glo.nomenclature.nomencl.1"><glossaryphrase key="nomencl.1" role="label"><Math mode="inline" tex="a" text="a" xml:id="Sx1.p1.m2a"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></glossaryphrase><glossaryphrase role="definition">The number of angels per unit area, see equation (1)</glossaryphrase></glossaryentry><glossaryentry fragid="glo.nomenclature.nomencl.2" key="nomencl.2" lists="nomenclature" xml:id="glo.nomenclature.nomencl.2"><glossaryphrase key="nomencl.2" role="label"><Math mode="inline" tex="N" text="N" xml:id="Sx1.p1.m4a"><XMath><XMTok font="italic" role="UNKNOWN">N</XMTok></XMath></Math></glossaryphrase><glossaryphrase role="definition">The number of angels per needle point</glossaryphrase></glossaryentry><glossaryentry fragid="glo.nomenclature.nomencl.3" key="nomencl.3" lists="nomenclature" xml:id="glo.nomenclature.nomencl.3"><glossaryphrase key="nomencl.3" role="label"><Math mode="inline" tex="A" text="A" xml:id="Sx1.p1.m6a"><XMath><XMTok font="italic" role="UNKNOWN">A</XMTok></XMath></Math></glossaryphrase><glossaryphrase role="definition">The area of the needle point</glossaryphrase></glossaryentry></glossarylist></glossary>"##,
  );
}
