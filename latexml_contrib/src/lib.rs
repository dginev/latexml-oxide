// `latexml_engine` carries the macro layer (`DefMacro!`, `LoadDefinitions!`,
// `compile_*!`) since the engine extraction; latexml_package keeps the
// prelude that adds `package::*` etc. (no #[macro_use] needed on
// latexml_package — it forwards macros transparently via the
// `pub extern crate latexml_engine` re-export, but contrib gets them
// directly here.)
#[macro_use]
extern crate latexml_engine;
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
pub mod aistats2026_sty;
pub mod aliascnt_sty;
pub mod ar5iv_sty;
pub mod arxbj_cls;
pub mod arydshln_sty;
pub mod ascmac_sty;
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
pub mod daj_cls;
pub mod fundam_cls;
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
pub mod latexmlman_sty;
pub mod letltxmacro_sty;
pub mod lettrine_sty;
pub mod libertine_sty;
pub mod ltablex_sty;
pub mod ltluatex_tex;
pub mod luatexbase_sty;
pub mod mciteplus_sty;
pub mod mdframed_sty;
pub mod memoir_cls;
pub mod mhchem_sty;
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
pub mod scicite_sty;
pub mod extarticle_cls;
pub mod scrartcl_cls;
pub mod scrbook_cls;
pub mod scrpage2_sty;
pub mod scrpage_sty;
pub mod aamas_cls;
pub mod achemso_cls;
pub mod agujournal2019_cls;
pub mod aomart_cls;
pub mod apacite_sty;
pub mod asme2ej_cls;
pub mod autart_cls;
pub mod birkjour_cls;
pub mod bmvc2k_cls;
pub mod bytedance_seed_cls;
pub mod cas_dc_cls;
pub mod ceurart_cls;
pub mod chemformula_sty;
pub mod doclicense_sty;
pub mod cimart_cls;
pub mod colm2025_conference_sty;
pub mod cvpr_sty;
pub mod ecai_cls;
pub mod egpubl_cls;
pub mod ejpecp_cls;
pub mod fcs_cls;
pub mod gretsi_cls;
pub mod iccv_sty;
pub mod ieeeaccess_cls;
pub mod ieeeaerospace_cls;
pub mod interspeech_cls;
pub mod ieeecolor_cls;
pub mod ieeeojcsys_cls;
pub mod ifacconf_cls;
pub mod ieeetaes_cls;
pub mod imsart_cls;
pub mod informs_cls;
pub mod interact_cls;
pub mod jmlr2e_sty;
pub mod jmlr_cls;
pub mod lipics_cls;
pub mod lmcs_cls;
pub mod mcom_l_cls;
pub mod mdpi_cls;
pub mod nature_pre_cls;
pub mod newpxmath_sty;
pub mod optica_article_cls;
pub mod oup_authoring_template_cls;
pub mod ptephy_cls;
pub mod sagej_cls;
pub mod scipost_cls;
pub mod scis2024_cls;
pub mod siamart_cls;
pub mod sigma_cls;
pub mod smc_ieeeconf_cls;
pub mod sn_jnl_cls;
pub mod spie_cls;
pub mod svproc_cls;
pub mod uai2025_cls;
pub mod wlscirep_cls;
pub mod wileymsp_template_cls;
pub mod wileynjd_cls;
pub mod wlpeerj_cls;
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
pub mod widetext_sty;
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
  ("aistats2026", "sty", aistats2026_sty::load_definitions),
  ("aliascnt", "sty", aliascnt_sty::load_definitions),
  ("ascmac", "sty", ascmac_sty::load_definitions),
  ("atableau", "sty", atableau_sty::load_definitions),
  ("bussproofs", "sty", bussproofs_sty::load_definitions),
  ("capt-of", "sty", capt_of_sty::load_definitions),
  ("chngpage", "sty", chngpage_sty::load_definitions),
  ("commath", "sty", commath_sty::load_definitions),
  ("crckapb", "sty", crckapb_sty::load_definitions),
  ("czjphys", "cls", czjphys_cls::load_definitions),
  ("daj", "cls", daj_cls::load_definitions),
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
  ("latexmlman", "sty", latexmlman_sty::load_definitions),
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
  ("scicite", "sty", scicite_sty::load_definitions),
  ("extarticle", "cls", extarticle_cls::load_definitions),
  // extbook / extreport / extletter / extproc share the same idea —
  // route them all to plain article for our XML/HTML purposes.
  ("extbook",   "cls", extarticle_cls::load_definitions),
  ("extreport", "cls", extarticle_cls::load_definitions),
  ("extletter", "cls", extarticle_cls::load_definitions),
  ("extproc",   "cls", extarticle_cls::load_definitions),
  ("scrartcl", "cls", scrartcl_cls::load_definitions),
  ("scrbook", "cls", scrbook_cls::load_definitions),
  ("tabularray", "sty", tabularray_sty::load_definitions),
  ("widetext", "sty", widetext_sty::load_definitions),
  ("xltabular", "sty", xltabular_sty::load_definitions),
  ("xr", "sty", xr_sty::load_definitions),
  ("xr-hyper", "sty", xr_sty::load_definitions),
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
  ("mhchem", "sty", mhchem_sty::load_definitions),
  ("minted", "sty", minted_sty::load_definitions),
  ("nicematrix", "sty", nicematrix_sty::load_definitions),
  ("pb-diagram", "sty", pb_diagram_sty::load_definitions),
  ("aamas", "cls", aamas_cls::load_definitions),
  ("achemso", "cls", achemso_cls::load_definitions),
  ("agujournal2019", "cls", agujournal2019_cls::load_definitions),
  ("agutexSI2019", "cls", agujournal2019_cls::load_definitions),
  ("aomart", "cls", aomart_cls::load_definitions),
  ("apacite", "sty", apacite_sty::load_definitions),
  ("asme2ej", "cls", asme2ej_cls::load_definitions),
  ("autart", "cls", autart_cls::load_definitions),
  ("birkjour", "cls", birkjour_cls::load_definitions),
  ("bmvc2k", "cls", bmvc2k_cls::load_definitions),
  ("bytedance_seed", "cls", bytedance_seed_cls::load_definitions),
  ("cas-dc", "cls", cas_dc_cls::load_definitions),
  ("cas-sc", "cls", cas_dc_cls::load_definitions),
  ("ceurart", "cls", ceurart_cls::load_definitions),
  ("chemformula", "sty", chemformula_sty::load_definitions),
  ("doclicense", "sty", doclicense_sty::load_definitions),
  ("cimart", "cls", cimart_cls::load_definitions),
  ("colm2025_conference", "sty", colm2025_conference_sty::load_definitions),
  ("cvpr", "sty", cvpr_sty::load_definitions),
  ("cvpr2023", "sty", cvpr_sty::load_definitions),
  ("cvpr2024", "sty", cvpr_sty::load_definitions),
  ("cvpr2025", "sty", cvpr_sty::load_definitions),
  ("ecai", "cls", ecai_cls::load_definitions),
  ("egpubl", "cls", egpubl_cls::load_definitions),
  ("ejpecp", "cls", ejpecp_cls::load_definitions),
  ("fcs", "cls", fcs_cls::load_definitions),
  ("fundam", "cls", fundam_cls::load_definitions),
  ("gretsi", "cls", gretsi_cls::load_definitions),
  ("IEEEapm", "cls", ieeeaerospace_cls::load_definitions),
  ("IEEEoj", "cls", ieeeaerospace_cls::load_definitions),
  ("IEEEtai", "cls", ieeeaerospace_cls::load_definitions),
  ("IEEEojcsys", "cls", ieeeojcsys_cls::load_definitions),
  ("ifacconf", "cls", ifacconf_cls::load_definitions),
  ("IEEEtaes", "cls", ieeetaes_cls::load_definitions),
  ("iccv", "sty", iccv_sty::load_definitions),
  ("iccvw", "sty", iccv_sty::load_definitions),
  ("ieeeaccess", "cls", ieeeaccess_cls::load_definitions),
  ("IEEEAerospaceCLS", "cls", ieeeaerospace_cls::load_definitions),
  ("ieeecolor", "cls", ieeecolor_cls::load_definitions),
  ("imsart", "cls", imsart_cls::load_definitions),
  ("informs", "cls", informs_cls::load_definitions),
  ("interact", "cls", interact_cls::load_definitions),
  ("Interspeech", "cls", interspeech_cls::load_definitions),
  ("clear2025", "cls", jmlr_cls::load_definitions),
  ("jmlr", "cls", jmlr_cls::load_definitions),
  ("jmlr2e", "sty", jmlr2e_sty::load_definitions),
  ("jmlr2e_preprint", "sty", jmlr2e_sty::load_definitions),
  ("lipics", "cls", lipics_cls::load_definitions),
  ("lipics-v2019", "cls", lipics_cls::load_definitions),
  ("lipics-v2021", "cls", lipics_cls::load_definitions),
  ("lipics-v2024", "cls", lipics_cls::load_definitions),
  ("lmcs", "cls", lmcs_cls::load_definitions),
  ("mcom-l", "cls", mcom_l_cls::load_definitions),
  ("mdpi", "cls", mdpi_cls::load_definitions),
  ("Definitions/mdpi", "cls", mdpi_cls::load_definitions),
  ("proc-l", "cls", mcom_l_cls::load_definitions),
  ("tran-l", "cls", mcom_l_cls::load_definitions),
  ("nature-pre", "cls", nature_pre_cls::load_definitions),
  ("newpxmath", "sty", newpxmath_sty::load_definitions),
  ("optica-article", "cls", optica_article_cls::load_definitions),
  ("oup-authoring-template", "cls",
   oup_authoring_template_cls::load_definitions),
  ("ptephy_v1", "cls", ptephy_cls::load_definitions),
  ("ptephy_v2", "cls", ptephy_cls::load_definitions),
  ("ptephy", "cls", ptephy_cls::load_definitions),
  ("siamart", "cls", siamart_cls::load_definitions),
  ("siamonline", "cls", siamart_cls::load_definitions),
  ("sagej", "cls", sagej_cls::load_definitions),
  ("SciPost", "cls", scipost_cls::load_definitions),
  ("SCIS2024", "cls", scis2024_cls::load_definitions),
  ("siamltex", "cls", siamltex_cls::load_definitions),
  ("sigma", "cls", sigma_cls::load_definitions),
  ("smc_ieeeconf", "cls", smc_ieeeconf_cls::load_definitions),
  ("sn-jnl", "cls", sn_jnl_cls::load_definitions),
  ("spie", "cls", spie_cls::load_definitions),
  ("svproc", "cls", svproc_cls::load_definitions),
  ("uai2025", "cls", uai2025_cls::load_definitions),
  ("WileyASNA-v1", "cls", wileynjd_cls::load_definitions),
  ("WileyMSP-template", "cls", wileymsp_template_cls::load_definitions),
  ("WileyNJD-v1", "cls", wileynjd_cls::load_definitions),
  ("WileyNJD-v2", "cls", wileynjd_cls::load_definitions),
  ("wileyNJDv5", "cls", wileynjd_cls::load_definitions),
  ("wlpeerj", "cls", wlpeerj_cls::load_definitions),
  ("wlscirep", "cls", wlscirep_cls::load_definitions),
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
  // Perl pathname_find L383-389: strict-case first, case-insensitive fallback
  // (matches `\documentclass{jhep}` against `JHEP.cls.ltxml`-style entries).
  // See latexml_package::dispatch for the parallel comment.
  BINDINGS
    .iter()
    .find(|(name, extension, _)| *name == base && *extension == ext)
    .or_else(|| {
      BINDINGS.iter().find(|(name, extension, _)| {
        name.eq_ignore_ascii_case(base) && extension.eq_ignore_ascii_case(ext)
      })
    })
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
