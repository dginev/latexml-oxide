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
pub mod axessibility_sty;
pub mod breqn_sty;
pub mod catchfile_sty;
pub mod changepage_sty;
pub mod cjk_sty;
pub mod cjkutf8_sty;
pub mod cmcal_sty;
pub mod datetime2_sty;
pub mod datetime_sty;
pub mod emlines_sty;
pub mod hobby_code_tex;
pub mod hyphenat_sty;
pub mod l3draw_sty;
pub mod lettrine_sty;
pub mod libertine_sty;
pub mod ltablex_sty;
pub mod mnsymbol_sty;
pub mod mssymb_tex;
pub mod oldgerm_sty;
pub mod pst_plot_sty;
pub mod savetrees_sty;
pub mod scrbook_cls;
pub mod tabularray_sty;
pub mod xltabular_sty;
pub mod xr_sty;

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
    "axessibility.sty" => Some(axessibility_sty::load_definitions()),
    "breqn.sty" => Some(breqn_sty::load_definitions()),
    "catchfile.sty" => Some(catchfile_sty::load_definitions()),
    "changepage.sty" => Some(changepage_sty::load_definitions()),
    "CJK.sty" => Some(cjk_sty::load_definitions()),
    "CJKutf8.sty" => Some(cjkutf8_sty::load_definitions()),
    "cmcal.sty" => Some(cmcal_sty::load_definitions()),
    "datetime2.sty" => Some(datetime2_sty::load_definitions()),
    "datetime.sty" => Some(datetime_sty::load_definitions()),
    "emlines.sty" => Some(emlines_sty::load_definitions()),
    "hobby.code.tex" => Some(hobby_code_tex::load_definitions()),
    "hyphenat.sty" => Some(hyphenat_sty::load_definitions()),
    "l3draw.sty" => Some(l3draw_sty::load_definitions()),
    "lettrine.sty" => Some(lettrine_sty::load_definitions()),
    "libertine.sty" => Some(libertine_sty::load_definitions()),
    "ltablex.sty" => Some(ltablex_sty::load_definitions()),
    "MnSymbol.sty" => Some(mnsymbol_sty::load_definitions()),
    "mssymb.tex" => Some(mssymb_tex::load_definitions()),
    "oldgerm.sty" => Some(oldgerm_sty::load_definitions()),
    "pst-plot.sty" => Some(pst_plot_sty::load_definitions()),
    "savetrees.sty" => Some(savetrees_sty::load_definitions()),
    "scrbook.cls" => Some(scrbook_cls::load_definitions()),
    "tabularray.sty" => Some(tabularray_sty::load_definitions()),
    "xltabular.sty" => Some(xltabular_sty::load_definitions()),
    "xr.sty" => Some(xr_sty::load_definitions()),
    _ => None,
  }
}

// III. That's all! Run "cargo test" in the latexml_oxide/ root and your binding will be compiled
// and made visible to the main latexml-oxide executable
