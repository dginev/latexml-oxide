#[macro_use]
extern crate latexml_package;
#[macro_use]
extern crate latexml_codegen;
use latexml_core::common::error::*;

// =======================
// Adding custom bindings:
// =======================
//
// I. Add your custom binding definition as a module delcaration here
pub mod mykeyval_sty;
pub mod mytemplate_sty;
pub mod myxkeyval_sty;
pub mod xkvdop1_sty;
pub mod xkvdop2_sty;
pub mod xkvdop3_sty;
pub mod xkvdop4_sty;
pub mod xkvdop5_cls;
pub mod xkvdop6_cls;
pub mod xkvview_sty;
pub mod apackage_sty;
pub mod filelistclass_cls;
pub mod myclass_cls;

// ar5iv-bindings ports
pub mod aliascnt_sty;
pub mod atableau_sty;
pub mod bussproofs_sty;
pub mod capt_of_sty;
pub mod chngpage_sty;
pub mod commath_sty;
pub mod czjphys_cls;
pub mod dblfloatfix_sty;
pub mod deluxe_sty;
pub mod diagrams_sty;
pub mod fp_sty;
pub mod fullname_sty;
pub mod glyphtounicode_tex;
pub mod hepnames_sty;
pub mod hepparticles_sty;
pub mod jpc_sty;
pub mod kotexutf_sty;
pub mod lanlmac_tex;
pub mod letltxmacro_sty;
pub mod ltluatex_tex;
pub mod luatexbase_sty;
pub mod needspace_sty;
pub mod pinlabel_sty;
pub mod program_sty;
pub mod scrpage2_sty;
pub mod stix2_sty;
pub mod stix_sty;
pub mod svg_extract_sty;
pub mod tipa_sty;
pub mod tlp_cls;

pub fn dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    // II. Connect the filename to the `load_definitions` function of your .rs binding:
    "apackage.sty" => Some(apackage_sty::load_definitions()),
    "filelistclass.cls" => Some(filelistclass_cls::load_definitions()),
    "myclass.cls" => Some(myclass_cls::load_definitions()),
    "mykeyval.sty" => Some(mykeyval_sty::load_definitions()),
    "mytemplate.sty" => Some(mytemplate_sty::load_definitions()),
    "myxkeyval.sty" => Some(myxkeyval_sty::load_definitions()),
    // xkeyval test packages — passthrough to raw TeX (noltxml)
    "xkvdop1.sty" => Some(xkvdop1_sty::load_definitions()),
    "xkvdop2.sty" => Some(xkvdop2_sty::load_definitions()),
    "xkvdop3.sty" => Some(xkvdop3_sty::load_definitions()),
    "xkvdop4.sty" => Some(xkvdop4_sty::load_definitions()),
    "xkvdop5.cls" => Some(xkvdop5_cls::load_definitions()),
    "xkvdop6.cls" => Some(xkvdop6_cls::load_definitions()),
    "xkvview.sty" => Some(xkvview_sty::load_definitions()),
    // ar5iv-bindings ports
    "aliascnt.sty" => Some(aliascnt_sty::load_definitions()),
    "atableau.sty" => Some(atableau_sty::load_definitions()),
    "bussproofs.sty" => Some(bussproofs_sty::load_definitions()),
    "capt-of.sty" => Some(capt_of_sty::load_definitions()),
    "chngpage.sty" => Some(chngpage_sty::load_definitions()),
    "commath.sty" => Some(commath_sty::load_definitions()),
    "czjphys.cls" => Some(czjphys_cls::load_definitions()),
    "dblfloatfix.sty" => Some(dblfloatfix_sty::load_definitions()),
    "deluxe.sty" => Some(deluxe_sty::load_definitions()),
    "diagrams.sty" => Some(diagrams_sty::load_definitions()),
    "fp.sty" => Some(fp_sty::load_definitions()),
    "fullname.sty" => Some(fullname_sty::load_definitions()),
    "glyphtounicode.tex" => Some(glyphtounicode_tex::load_definitions()),
    "hepnames.sty" => Some(hepnames_sty::load_definitions()),
    "hepparticles.sty" => Some(hepparticles_sty::load_definitions()),
    "jpc.sty" => Some(jpc_sty::load_definitions()),
    "kotexutf.sty" => Some(kotexutf_sty::load_definitions()),
    "lanlmac.tex" => Some(lanlmac_tex::load_definitions()),
    "letltxmacro.sty" => Some(letltxmacro_sty::load_definitions()),
    "ltluatex.tex" => Some(ltluatex_tex::load_definitions()),
    "luatexbase.sty" => Some(luatexbase_sty::load_definitions()),
    "needspace.sty" => Some(needspace_sty::load_definitions()),
    "pinlabel.sty" => Some(pinlabel_sty::load_definitions()),
    "program.sty" => Some(program_sty::load_definitions()),
    "scrpage2.sty" => Some(scrpage2_sty::load_definitions()),
    "stix2.sty" => Some(stix2_sty::load_definitions()),
    "stix.sty" => Some(stix_sty::load_definitions()),
    "svg-extract.sty" => Some(svg_extract_sty::load_definitions()),
    "tipa.sty" => Some(tipa_sty::load_definitions()),
    "tlp.cls" => Some(tlp_cls::load_definitions()),
    _ => None,
  }
}

// III. That's all! Run "cargo test" in the latexml_oxide/ root and your binding will be compiled
// and made visible to the main latexml-oxide executable
