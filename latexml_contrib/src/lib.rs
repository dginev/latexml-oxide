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
pub mod apackage_sty;
pub mod discard_env;
pub mod filelistclass_cls;
pub mod myclass_cls;
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

// ar5iv-bindings ports
pub mod aliascnt_sty;
pub mod ar5iv_sty;
pub mod arxbj_cls;
pub mod arydshln_sty;
pub mod atableau_sty;
pub mod axessibility_sty;
pub mod biblatex_sty;
pub mod breqn_sty;
pub mod bussproofs_sty;
pub mod capt_of_sty;
pub mod catchfile_sty;
pub mod changepage_sty;
pub mod changes_sty;
pub mod chngpage_sty;
pub mod cjk_sty;
pub mod cjkutf8_sty;
pub mod cmcal_sty;
pub mod commath_sty;
pub mod crckapb_sty;
pub mod currfile_sty;
pub mod czjphys_cls;
pub mod datetime2_sty;
pub mod datetime_sty;
pub mod dblfloatfix_sty;
pub mod deluxe_sty;
pub mod diagrams_sty;
pub mod diagrams_tex;
pub mod ed_sty;
pub mod emlines_sty;
pub mod equations_sty;
pub mod eso_pic_sty;
pub mod fontawesome5_sty;
pub mod fontawesome_sty;
pub mod forest_sty;
pub mod fp_sty;
pub mod fullname_sty;
pub mod glyphtounicode_tex;
pub mod harvmac_tex;
pub mod hepnames_sty;
pub mod hepparticles_sty;
pub mod hobby_code_tex;
pub mod hyphenat_sty;
pub mod ifdraft_sty;
pub mod jpc_sty;
pub mod kotexutf_sty;
pub mod l3draw_sty;
pub mod lanlmac_tex;
pub mod letltxmacro_sty;
pub mod lettrine_sty;
pub mod libertine_sty;
pub mod ltablex_sty;
pub mod ltluatex_tex;
pub mod luatexbase_sty;
pub mod mciteplus_sty;
pub mod mdframed_sty;
pub mod memoir_cls;
pub mod minted_sty;
pub mod mnsymbol_sty;
pub mod mssymb_tex;
pub mod needspace_sty;
pub mod nicematrix_sty;
pub mod oldgerm_sty;
pub mod pb_diagram_sty;
pub mod phyzzx_plus;
pub mod phyzzx_tex;
pub mod pinlabel_sty;
pub mod program_sty;
pub mod pst_plot_sty;
pub mod savetrees_sty;
pub mod scrbook_cls;
pub mod scrpage2_sty;
pub mod scrpage_sty;
pub mod siamltex_cls;
pub mod stix2_sty;
pub mod stix_sty;
pub mod svg_extract_sty;
pub mod svn_multi_sty;
pub mod svninfo_sty;
pub mod tabu_sty;
pub mod tabularray_sty;
pub mod tipa_sty;
pub mod tlp_cls;
pub mod ucs_sty;
pub mod ut_thesis_cls;
pub mod ws_p8_50x6_00_cls;
pub mod xltabular_sty;
pub mod xr_sty;

/// Type of a binding loader fn — matches the return type of every
/// `*::load_definitions` in this crate.
pub type BindingLoader = fn() -> Result<()>;

/// Single source of truth for contrib bindings. Pairs a filename (`name`,
/// `ext`) with its `load_definitions` fn. Used by `dispatch` (runtime
/// loader) and by `class_binding_names` (consumed by
/// `latexml_core::binding::content::load_class` for the prefix-match
/// fallback alongside `latexml_package::BINDINGS`).
///
/// II. Connect the filename to the `load_definitions` function of your
///     `.rs` binding by adding a new row here.
pub const BINDINGS: &[(&str, &str, BindingLoader)] = &[
  ("apackage", "sty", apackage_sty::load_definitions),
  ("filelistclass", "cls", filelistclass_cls::load_definitions),
  ("myclass", "cls", myclass_cls::load_definitions),
  ("mykeyval", "sty", mykeyval_sty::load_definitions),
  ("mytemplate", "sty", mytemplate_sty::load_definitions),
  ("myxkeyval", "sty", myxkeyval_sty::load_definitions),
  // xkeyval test packages — passthrough to raw TeX (noltxml)
  ("xkvdop1", "sty", xkvdop1_sty::load_definitions),
  ("xkvdop2", "sty", xkvdop2_sty::load_definitions),
  ("xkvdop3", "sty", xkvdop3_sty::load_definitions),
  ("xkvdop4", "sty", xkvdop4_sty::load_definitions),
  ("xkvdop5", "cls", xkvdop5_cls::load_definitions),
  ("xkvdop6", "cls", xkvdop6_cls::load_definitions),
  ("xkvview", "sty", xkvview_sty::load_definitions),
  // ar5iv-bindings ports
  ("aliascnt", "sty", aliascnt_sty::load_definitions),
  ("atableau", "sty", atableau_sty::load_definitions),
  ("bussproofs", "sty", bussproofs_sty::load_definitions),
  ("capt-of", "sty", capt_of_sty::load_definitions),
  ("chngpage", "sty", chngpage_sty::load_definitions),
  ("commath", "sty", commath_sty::load_definitions),
  ("crckapb", "sty", crckapb_sty::load_definitions),
  ("czjphys", "cls", czjphys_cls::load_definitions),
  ("dblfloatfix", "sty", dblfloatfix_sty::load_definitions),
  ("deluxe", "sty", deluxe_sty::load_definitions),
  ("diagrams", "sty", diagrams_sty::load_definitions),
  ("fontawesome", "sty", fontawesome_sty::load_definitions),
  ("fontawesome5", "sty", fontawesome5_sty::load_definitions),
  ("fp", "sty", fp_sty::load_definitions),
  ("fullname", "sty", fullname_sty::load_definitions),
  (
    "glyphtounicode",
    "tex",
    glyphtounicode_tex::load_definitions,
  ),
  ("hepnames", "sty", hepnames_sty::load_definitions),
  ("hepparticles", "sty", hepparticles_sty::load_definitions),
  ("jpc", "sty", jpc_sty::load_definitions),
  ("kotexutf", "sty", kotexutf_sty::load_definitions),
  ("lanlmac", "tex", lanlmac_tex::load_definitions),
  ("letltxmacro", "sty", letltxmacro_sty::load_definitions),
  ("ltluatex", "tex", ltluatex_tex::load_definitions),
  ("luatexbase", "sty", luatexbase_sty::load_definitions),
  ("needspace", "sty", needspace_sty::load_definitions),
  ("phyzzx", "plus", phyzzx_plus::load_definitions),
  ("phyzzx", "tex", phyzzx_tex::load_definitions),
  ("pinlabel", "sty", pinlabel_sty::load_definitions),
  ("program", "sty", program_sty::load_definitions),
  ("scrpage", "sty", scrpage_sty::load_definitions),
  ("scrpage2", "sty", scrpage2_sty::load_definitions),
  ("stix2", "sty", stix2_sty::load_definitions),
  ("stix", "sty", stix_sty::load_definitions),
  ("svg-extract", "sty", svg_extract_sty::load_definitions),
  ("tipa", "sty", tipa_sty::load_definitions),
  ("tlp", "cls", tlp_cls::load_definitions),
  ("axessibility", "sty", axessibility_sty::load_definitions),
  ("biblatex", "sty", biblatex_sty::load_definitions),
  ("breqn", "sty", breqn_sty::load_definitions),
  ("catchfile", "sty", catchfile_sty::load_definitions),
  ("changepage", "sty", changepage_sty::load_definitions),
  ("CJK", "sty", cjk_sty::load_definitions),
  ("CJKutf8", "sty", cjkutf8_sty::load_definitions),
  ("cmcal", "sty", cmcal_sty::load_definitions),
  ("datetime2", "sty", datetime2_sty::load_definitions),
  ("datetime", "sty", datetime_sty::load_definitions),
  ("ed", "sty", ed_sty::load_definitions),
  ("emlines", "sty", emlines_sty::load_definitions),
  ("hobby", "code.tex", hobby_code_tex::load_definitions),
  ("hyphenat", "sty", hyphenat_sty::load_definitions),
  ("ifdraft", "sty", ifdraft_sty::load_definitions),
  ("l3draw", "sty", l3draw_sty::load_definitions),
  ("lettrine", "sty", lettrine_sty::load_definitions),
  ("libertine", "sty", libertine_sty::load_definitions),
  ("ltablex", "sty", ltablex_sty::load_definitions),
  ("MnSymbol", "sty", mnsymbol_sty::load_definitions),
  ("mssymb", "tex", mssymb_tex::load_definitions),
  ("oldgerm", "sty", oldgerm_sty::load_definitions),
  ("pst-plot", "sty", pst_plot_sty::load_definitions),
  ("savetrees", "sty", savetrees_sty::load_definitions),
  ("scrbook", "cls", scrbook_cls::load_definitions),
  ("tabularray", "sty", tabularray_sty::load_definitions),
  ("xltabular", "sty", xltabular_sty::load_definitions),
  ("xr", "sty", xr_sty::load_definitions),
  ("ar5iv", "sty", ar5iv_sty::load_definitions),
  ("arxbj", "cls", arxbj_cls::load_definitions),
  ("arydshln", "sty", arydshln_sty::load_definitions),
  ("changes", "sty", changes_sty::load_definitions),
  ("currfile", "sty", currfile_sty::load_definitions),
  ("diagrams", "tex", diagrams_tex::load_definitions),
  ("equations", "sty", equations_sty::load_definitions),
  ("eso-pic", "sty", eso_pic_sty::load_definitions),
  ("forest", "sty", forest_sty::load_definitions),
  ("harvmac", "tex", harvmac_tex::load_definitions),
  ("mciteplus", "sty", mciteplus_sty::load_definitions),
  ("mdframed", "sty", mdframed_sty::load_definitions),
  ("memoir", "cls", memoir_cls::load_definitions),
  ("minted", "sty", minted_sty::load_definitions),
  ("nicematrix", "sty", nicematrix_sty::load_definitions),
  ("pb-diagram", "sty", pb_diagram_sty::load_definitions),
  ("siamltex", "cls", siamltex_cls::load_definitions),
  ("svn-multi", "sty", svn_multi_sty::load_definitions),
  ("svninfo", "sty", svninfo_sty::load_definitions),
  ("tabu", "sty", tabu_sty::load_definitions),
  ("ucs", "sty", ucs_sty::load_definitions),
  ("ut-thesis", "cls", ut_thesis_cls::load_definitions),
  ("ws-p8-50x6-00", "cls", ws_p8_50x6_00_cls::load_definitions),
];

/// Runtime lookup: route `filename` (e.g. `"MnSymbol.sty"`,
/// `"hobby.code.tex"`) through its compiled `load_definitions` fn, or return
/// `None` when the filename has no registered binding. Splits on the *first*
/// `.` so `("hobby", "code.tex", …)` matches correctly — mirrors
/// `latexml_package::dispatch`.
pub fn dispatch(filename: &str) -> Option<Result<()>> {
  let (base, ext) = filename.split_once('.')?;
  BINDINGS
    .iter()
    .find(|(name, extension, _)| *name == base && *extension == ext)
    .map(|(_, _, loader)| loader())
}

/// All registered (name, extension) pairs for this crate's BINDINGS.
/// Mirror of `latexml_package::binding_names`. Consumed by
/// `find_file(notex=true)` to detect compiled-binding existence across
/// all registered extensions (cls/sty/def/pool/code.tex/...). The class
/// names (entries with `ext == "cls"`) flow into `load_class`'s
/// Perl-parity prefix-match fallback via the
/// `state::get_class_binding_names()` filtered view.
pub fn binding_names() -> &'static [(&'static str, &'static str)] {
  use std::sync::OnceLock;
  static NAMES: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
  NAMES
    .get_or_init(|| {
      BINDINGS
        .iter()
        .map(|(name, ext, _)| (*name, *ext))
        .collect()
    })
    .as_slice()
}

// III. That's all! Run "cargo test" in the latexml_oxide/ root and your binding will be compiled
// and made visible to the main latexml-oxide executable
