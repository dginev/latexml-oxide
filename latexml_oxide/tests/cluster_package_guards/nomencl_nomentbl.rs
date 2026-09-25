//! Batch 56hn: a `nomentbl` entry's `unit` and `note` columns (nomencl.sty:228-235)
//! follow its description in the list as classed text instead of being dropped
//! (OXIDIZED_DESIGN #234).
use crate::cluster::convert_and_post_clean;

#[test]
fn nomentbl_units_and_notes_reach_the_list() {
  let xml = convert_and_post_clean("tests/cluster_regressions/nomencl_nomentbl_units.tex");
  latexml::util::test::assert_element(
    &xml,
    "glossaryentry",
    &[],
    r##"<glossaryentry fragid="glo.nomenclature.nomencl.1" key="nomencl.1" lists="nomenclature" xml:id="glo.nomenclature.nomencl.1"><glossaryphrase key="nomencl.1" role="label"><Math mode="inline" tex="c" text="c" xml:id="p1.m3a"><XMath><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math></glossaryphrase><glossaryphrase role="definition">Speed of light <text class="ltx_glossary_unit">m/s</text> <text class="ltx_glossary_note">constant</text></glossaryphrase></glossaryentry>"##,
  );
}
