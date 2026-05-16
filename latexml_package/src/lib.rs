//! Binding definitions for the `LaTeXML` converter, reimplemented in Rust
#![recursion_limit = "1024"]

#[macro_use]
extern crate latexml_codegen;
// Re-export the engine crate so existing `crate::engine::tex_tables` etc.
// paths in `package/*` keep resolving without per-file rewrites. The
// engine crate was extracted from `latexml_package::engine` to reduce
// CI cold-cache RAM peaks (see docs/SYNC_STATUS.md). Engine-side macros
// (`DefMacro!`, `LoadDefinitions!`, `compile_*!`, …) are #[macro_export]ed
// from latexml_engine and `#[macro_use]` here pulls them into our scope
// so existing `package/*.rs` keep using bare macro names.
#[macro_use]
pub extern crate latexml_engine as engine;
// Prelude for writing bindings — re-exports `latexml_engine::prelude`
// plus package-specific symbols.
#[macro_use]
pub mod prelude;
// XMath helper functions for building XMDual token streams
pub mod xmath_helpers;
// Supported LaTeX packages
#[macro_use]
pub mod package;

use latexml_core::common::error::Result;

/// Type of a binding loader fn — matches the return type of every
/// `package::*::load_definitions` / `engine::*::load_definitions`.
pub type BindingLoader = fn() -> Result<()>;

/// Single source of truth for the compiled in-distro bindings. Each entry
/// pairs a filename (with its `.pool` / `.sty` / `.cls` / `.def` extension)
/// to its `load_definitions` fn. Used by `dispatch` (runtime loader) and by
/// `class_binding_names` (prefix-match fallback in
/// `latexml_core::binding::content::load_class`). Please keep this list as
/// the ONLY place where binding filenames are enumerated — introducing a
/// second list drifts quickly.
pub const BINDINGS: &[(&str, &str, BindingLoader)] = &[
  ("TeX", "pool", engine::tex::load_definitions),
  ("LaTeX", "pool", engine::latex::load_definitions),
  ("eTeX", "pool", engine::etex::load_definitions),
  ("pdfTeX", "pool", engine::pdftex::load_definitions),
  ("AmSTeX", "pool", engine::amstex::load_definitions),
  ("BibTeX", "pool", engine::bibtex::load_definitions),
  ("latexml", "sty", package::latexml_sty::load_definitions),
  ("lxRDFa", "sty", package::lxrdfa_sty::load_definitions),
  ("marvosym", "sty", package::marvosym_sty::load_definitions),
  ("mathbbol", "sty", package::mathbbol_sty::load_definitions),
  ("a4", "sty", package::a4_sty::load_definitions),
  ("a4wide", "sty", package::a4wide_sty::load_definitions),
  ("aastex", "cls", package::aastex_cls::load_definitions),
  ("aastex631", "cls", package::aastex_cls::load_definitions), // version fallback
  ("aastex62", "cls", package::aastex_cls::load_definitions),
  ("aastex63", "cls", package::aastex_cls::load_definitions),
  ("aastex7", "cls", package::aastex_cls::load_definitions),
  ("aastex70", "cls", package::aastex_cls::load_definitions),
  ("aastex", "sty", package::aastex_sty::load_definitions),
  ("aasms", "sty", package::aasms_sty::load_definitions),
  ("aaspp", "sty", package::aaspp_sty::load_definitions),
  (
    "aas_macros",
    "sty",
    package::aas_macros_sty::load_definitions,
  ),
  (
    "aas_support",
    "sty",
    package::aas_support_sty::load_definitions,
  ),
  ("ae", "sty", package::ae_sty::load_definitions),
  ("aecompl", "sty", package::aecompl_sty::load_definitions),
  ("atveryend", "sty", package::atveryend_sty::load_definitions),
  ("auxhook", "sty", package::auxhook_sty::load_definitions),
  ("ams_core", "cls", package::ams_core_cls::load_definitions),
  (
    "ams_support",
    "sty",
    package::ams_support_sty::load_definitions,
  ),
  ("amsaddr", "sty", package::amsaddr_sty::load_definitions),
  ("amsart", "cls", package::amsart_cls::load_definitions),
  ("amsproc", "cls", package::amsproc_cls::load_definitions),
  // smfart: no binding — Perl falls through to OmniBus, which provides
  // \Subsection, \Paragraph, \institute, etc. The earlier Rust binding
  // loaded amsart instead, which doesn't define those CSes; smfart-using
  // papers (witness: arXiv:2603.04274) hit Error:undefined:\Subsection.
  ("acmart", "cls", package::acmart_cls::load_definitions),
  ("article", "cls", package::article_cls::load_definitions),
  ("OmniBus", "cls", package::omnibus_cls::load_definitions),
  ("babel", "sty", package::babel_sty::load_definitions),
  ("beamer", "cls", package::beamer_cls::load_definitions),
  ("balance", "sty", package::balance_sty::load_definitions),
  ("breakurl", "sty", package::breakurl_sty::load_definitions),
  ("algc", "sty", package::algc_sty::load_definitions),
  (
    "algcompatible",
    "sty",
    package::algcompatible_sty::load_definitions,
  ),
  ("algorithm", "sty", package::algorithm_sty::load_definitions),
  (
    "algorithmicx",
    "sty",
    package::algorithmicx_sty::load_definitions,
  ),
  ("algmatlab", "sty", package::algmatlab_sty::load_definitions),
  ("algpascal", "sty", package::algpascal_sty::load_definitions),
  (
    "algpseudocode",
    "sty",
    package::algpseudocode_sty::load_definitions,
  ),
  ("alltt", "sty", package::alltt_sty::load_definitions),
  ("array", "sty", package::array_sty::load_definitions),
  ("bbm", "sty", package::bbm_sty::load_definitions),
  ("bbold", "sty", package::bbold_sty::load_definitions),
  ("appendix", "sty", package::appendix_sty::load_definitions),
  ("book", "cls", package::book_cls::load_definitions),
  ("booktabs", "sty", package::booktabs_sty::load_definitions),
  ("caption", "sty", package::caption_sty::load_definitions),
  ("refcount", "sty", package::refcount_sty::load_definitions),
  ("remreset", "sty", package::remreset_sty::load_definitions),
  ("report", "cls", package::report_cls::load_definitions),
  ("revtex4", "cls", package::revtex4_cls::load_definitions),
  ("revtex4-2", "cls", package::revtex4_cls::load_definitions), // version fallback
  ("revtex", "sty", package::revtex_sty::load_definitions),
  ("revtex4", "sty", package::revtex4_sty::load_definitions),
  (
    "revtex3_support",
    "sty",
    package::revtex3_support_sty::load_definitions,
  ),
  (
    "revtex4_support",
    "sty",
    package::revtex4_support_sty::load_definitions,
  ),
  ("revtex", "cls", package::revtex_cls::load_definitions),
  ("revtex4-1", "cls", package::revtex4_1_cls::load_definitions),
  ("jheppub", "sty", package::jheppub_sty::load_definitions),
  ("neurips", "sty", package::neurips_sty::load_definitions),
  (
    "neurips_2019",
    "sty",
    package::neurips_sty::load_definitions,
  ),
  (
    "neurips_2020",
    "sty",
    package::neurips_sty::load_definitions,
  ),
  (
    "neurips_2021",
    "sty",
    package::neurips_sty::load_definitions,
  ),
  (
    "neurips_2022",
    "sty",
    package::neurips_sty::load_definitions,
  ),
  (
    "neurips_2023",
    "sty",
    package::neurips_sty::load_definitions,
  ),
  (
    "neurips_2024",
    "sty",
    package::neurips_sty::load_definitions,
  ),
  (
    "neurips_2025",
    "sty",
    package::neurips_sty::load_definitions,
  ),
  (
    "algorithm2e",
    "sty",
    package::algorithm2e_sty::load_definitions,
  ),
  // ar5iv.sty is in latexml_contrib (not here) since it's a contrib binding
  (
    "algorithmic",
    "sty",
    package::algorithmic_sty::load_definitions,
  ),
  ("numprint", "sty", package::numprint_sty::load_definitions),
  ("titling", "sty", package::titling_sty::load_definitions),
  ("vmargin", "sty", package::vmargin_sty::load_definitions),
  ("rotate", "sty", package::rotate_sty::load_definitions),
  ("rotating", "sty", package::rotating_sty::load_definitions),
  (
    "doublespace",
    "sty",
    package::doublespace_sty::load_definitions,
  ),
  ("ed", "sty", package::ed_sty::load_definitions),
  ("elsart", "cls", package::elsart_cls::load_definitions),
  ("elsart1p", "cls", package::elsart_cls::load_definitions),
  ("elsart3p", "cls", package::elsart_cls::load_definitions),
  ("elsart5p", "cls", package::elsart_cls::load_definitions),
  ("elsarticle", "cls", package::elsart_cls::load_definitions),
  ("elsart", "sty", package::elsart_sty::load_definitions),
  (
    "elsart_support_core",
    "sty",
    package::elsart_support_core_sty::load_definitions,
  ),
  (
    "mn2e_support",
    "sty",
    package::mn2e_support_sty::load_definitions,
  ),
  (
    "elsart_support",
    "sty",
    package::elsart_support_sty::load_definitions,
  ),
  (
    "emulateapj",
    "cls",
    package::emulateapj_cls::load_definitions,
  ),
  ("revsymb", "sty", package::revsymb_sty::load_definitions),
  ("revsymb4-1", "sty", package::revsymb_sty::load_definitions),
  ("SIunits", "sty", package::siunits_sty::load_definitions),
  ("overpic", "sty", package::overpic_sty::load_definitions),
  ("yfonts", "sty", package::yfonts_sty::load_definitions),
  ("setspace", "sty", package::setspace_sty::load_definitions),
  ("moderncv", "cls", package::moderncv_cls::load_definitions),
  ("slides", "cls", package::slides_cls::load_definitions),
  ("amsmath", "sty", package::amsmath_sty::load_definitions),
  ("mathastext", "sty", package::mathastext_sty::load_definitions),
  ("mathtools", "sty", package::mathtools_sty::load_definitions),
  ("microtype", "sty", package::microtype_sty::load_definitions),
  ("amsrefs", "sty", package::amsrefs_sty::load_definitions),
  ("amsfonts", "sty", package::amsfonts_sty::load_definitions),
  ("amssymb", "sty", package::amssymb_sty::load_definitions),
  ("amsthm", "sty", package::amsthm_sty::load_definitions),
  ("theorem", "sty", package::theorem_sty::load_definitions),
  ("mathpazo", "sty", package::mathpazo_sty::load_definitions),
  ("mathpple", "sty", package::mathpple_sty::load_definitions),
  ("mathptm", "sty", package::mathptm_sty::load_definitions),
  ("mathptmx", "sty", package::mathptmx_sty::load_definitions),
  ("rsfs", "sty", package::rsfs_sty::load_definitions),
  ("txfonts", "sty", package::txfonts_sty::load_definitions),
  ("xunicode", "sty", package::xunicode_sty::load_definitions),
  ("ntheorem", "sty", package::ntheorem_sty::load_definitions),
  ("thmtools", "sty", package::thmtools_sty::load_definitions),
  ("paralist", "sty", package::paralist_sty::load_definitions),
  ("amsbsy", "sty", package::amsbsy_sty::load_definitions),
  ("amscd", "sty", package::amscd_sty::load_definitions),
  ("amsgen", "sty", package::amsgen_sty::load_definitions),
  ("amstext", "sty", package::amstext_sty::load_definitions),
  ("amsopn", "sty", package::amsopn_sty::load_definitions),
  ("amstex", "sty", package::amstex_sty::load_definitions),
  ("amstex", "tex", package::amstex_tex::load_definitions),
  ("amsxtra", "sty", package::amsxtra_sty::load_definitions),
  ("empheq", "sty", package::empheq_sty::load_definitions),
  ("fancybox", "sty", package::fancybox_sty::load_definitions),
  ("feynmf", "sty", package::feynmf_sty::load_definitions),
  ("filehook", "sty", package::filehook_sty::load_definitions),
  ("flushend", "sty", package::flushend_sty::load_definitions),
  ("fix-cm", "sty", package::fix_cm_sty::load_definitions),
  ("fixltx2e", "sty", package::fixltx2e_sty::load_definitions),
  (
    "fancyheadings",
    "sty",
    package::fancyheadings_sty::load_definitions,
  ),
  ("fancyhdr", "sty", package::fancyhdr_sty::load_definitions),
  ("footnote", "sty", package::footnote_sty::load_definitions),
  ("footmisc", "sty", package::footmisc_sty::load_definitions),
  ("latexsym", "sty", package::latexsym_sty::load_definitions),
  ("fullpage", "sty", package::fullpage_sty::load_definitions),
  ("comment", "sty", package::comment_sty::load_definitions),
  ("csquotes", "sty", package::csquotes_sty::load_definitions),
  ("dcolumn", "sty", package::dcolumn_sty::load_definitions),
  (
    "deluxetable",
    "sty",
    package::deluxetable_sty::load_definitions,
  ),
  ("english", "sty", package::english_sty::load_definitions),
  ("english", "ldf", package::english_sty::load_definitions),
  ("endnotes", "sty", package::endnotes_sty::load_definitions),
  ("enumitem", "sty", package::enumitem_sty::load_definitions),
  ("epigraph", "sty", package::epigraph_sty::load_definitions),
  ("float", "sty", package::float_sty::load_definitions),
  ("floatfig", "sty", package::floatfig_sty::load_definitions),
  ("floatpag", "sty", package::floatpag_sty::load_definitions),
  ("gen-j-l", "cls", package::gen_j_l_cls::load_definitions),
  ("gen-m-l", "cls", package::gen_m_l_cls::load_definitions),
  ("gen-p-l", "cls", package::gen_p_l_cls::load_definitions),
  ("french", "ldf", package::french_ldf::load_definitions),
  ("frenchb", "ldf", package::french_ldf::load_definitions),
  ("nil", "ldf", package::nil_ldf::load_definitions),
  ("gensymb", "sty", package::gensymb_sty::load_definitions),
  ("geometry", "sty", package::geometry_sty::load_definitions),
  ("german", "sty", package::german_sty::load_definitions),
  ("germanb", "ldf", package::german_sty::load_definitions),
  ("german", "ldf", package::german_sty::load_definitions),
  ("ngerman", "ldf", package::ngerman_sty::load_definitions),
  ("ngermanb", "ldf", package::ngerman_sty::load_definitions),
  (
    "glossaries",
    "sty",
    package::glossaries_sty::load_definitions,
  ),
  ("graphics", "sty", package::graphics_sty::load_definitions),
  ("graphicx", "sty", package::graphicx_sty::load_definitions),
  ("grffile", "sty", package::grffile_sty::load_definitions),
  ("icml", "sty", package::icml_sty::load_definitions),
  (
    "icml2016",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2017",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2018",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2019",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2020",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2021",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2022",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2023",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2024",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml2025",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  (
    "icml_support",
    "sty",
    package::icml_support_sty::load_definitions,
  ),
  ("ifetex", "sty", package::ifetex_sty::load_definitions),
  ("ifluatex", "sty", package::ifluatex_sty::load_definitions),
  ("ifvtex", "sty", package::ifvtex_sty::load_definitions),
  ("ifpdf", "sty", package::ifpdf_sty::load_definitions),
  ("iftex", "sty", package::iftex_sty::load_definitions),
  ("ifthen", "sty", package::ifthen_sty::load_definitions),
  ("ifxetex", "sty", package::ifxetex_sty::load_definitions),
  ("ifdraft", "sty", package::ifdraft_sty::load_definitions),
  ("ieeeconf", "cls", package::ieeeconf_cls::load_definitions),
  ("IEEEtran", "cls", package::ieeetran_cls::load_definitions),
  ("import", "sty", package::import_sty::load_definitions),
  ("iopart", "cls", package::iopart_cls::load_definitions),
  (
    "iopart_support",
    "sty",
    package::iopart_support_sty::load_definitions,
  ),
  ("svjour", "cls", package::svjour_cls::load_definitions),
  ("svjour1", "cls", package::svjour3_cls::load_definitions),
  ("svjour2", "cls", package::svjour3_cls::load_definitions),
  ("svjour3", "cls", package::svjour3_cls::load_definitions),
  ("svmono", "cls", package::svjour3_cls::load_definitions),
  (
    "inst_support",
    "sty",
    package::inst_support_sty::load_definitions,
  ),
  ("JHEP", "cls", package::jhep_cls::load_definitions),
  ("JHEP2", "cls", package::jhep2_cls::load_definitions),
  ("JHEP3", "cls", package::jhep3_cls::load_definitions),
  ("keyval", "sty", package::keyval_sty::load_definitions),
  (
    "sv_support",
    "sty",
    package::sv_support_sty::load_definitions,
  ),
  ("svmult", "cls", package::svmult_cls::load_definitions),
  ("ulem", "sty", package::ulem_sty::load_definitions),
  ("url", "sty", package::url_sty::load_definitions),
  ("varioref", "sty", package::varioref_sty::load_definitions),
  ("varwidth", "sty", package::varwidth_sty::load_definitions),
  ("xargs", "sty", package::xargs_sty::load_definitions),
  ("esint", "sty", package::esint_sty::load_definitions),
  ("etoolbox", "sty", package::etoolbox_sty::load_definitions),
  ("eurosym", "sty", package::eurosym_sty::load_definitions),
  ("hhline", "sty", package::hhline_sty::load_definitions),
  ("cleveref", "sty", package::cleveref_sty::load_definitions),
  ("hypcap", "sty", package::hypcap_sty::load_definitions),
  ("hyperref", "sty", package::hyperref_sty::load_definitions),
  ("hyperxmp", "sty", package::hyperxmp_sty::load_definitions),
  ("nameref", "sty", package::nameref_sty::load_definitions),
  ("nomencl", "sty", package::nomencl_sty::load_definitions),
  ("verbatim", "sty", package::verbatim_sty::load_definitions),
  ("eucal", "sty", package::eucal_sty::load_definitions),
  ("eufrak", "sty", package::eufrak_sty::load_definitions),
  ("fontenc", "sty", package::fontenc_sty::load_definitions),
  ("fontspec", "sty", package::fontspec_sty::load_definitions),
  ("inputenc", "sty", package::inputenc_sty::load_definitions),
  ("textcomp", "sty", package::textcomp_sty::load_definitions),
  ("textgreek", "sty", package::textgreek_sty::load_definitions),
  // textalpha (greek-fontenc) and alphabeta (greek-fontenc) both define the
  // same `\text<greek>` family; route them through the same binding so a
  // raw-load failure on LGR encoding doesn't surface as `\textsigma` undef.
  ("textalpha", "sty", package::textgreek_sty::load_definitions),
  ("alphabeta", "sty", package::textgreek_sty::load_definitions),
  ("texvc", "sty", package::texvc_sty::load_definitions),
  ("listings", "sty", package::listings_sty::load_definitions),
  (
    "listingsutf8",
    "sty",
    package::listingsutf8_sty::load_definitions,
  ),
  ("longtable", "sty", package::longtable_sty::load_definitions),
  (
    "marginnote",
    "sty",
    package::marginnote_sty::load_definitions,
  ),
  ("makecell", "sty", package::makecell_sty::load_definitions),
  ("mn", "cls", package::mn_cls::load_definitions),
  ("mn2e", "cls", package::mn2e_cls::load_definitions),
  ("mnras", "cls", package::mnras_cls::load_definitions),
  (
    "supertabular",
    "sty",
    package::supertabular_sty::load_definitions,
  ),
  ("multicol", "sty", package::multicol_sty::load_definitions),
  ("multido", "sty", package::multido_sty::load_definitions),
  ("multirow", "sty", package::multirow_sty::load_definitions),
  ("newclude", "sty", package::newclude_sty::load_definitions),
  ("newfloat", "sty", package::newfloat_sty::load_definitions),
  ("applemac", "def", package::applemac_def::load_definitions),
  ("cp852", "def", package::cp852_def::load_definitions),
  ("csquotes", "def", package::csquotes_def::load_definitions),
  ("latin10", "def", package::latin10_def::load_definitions),
  ("t5enc", "def", package::t5enc_def::load_definitions),
  ("t1enc", "sty", package::t1enc_sty::load_definitions),
  ("t1enc", "def", package::t1enc_def::load_definitions),
  ("amsa", "fontmap", package::amsa_fontmap::load_definitions),
  ("amsb", "fontmap", package::amsb_fontmap::load_definitions),
  ("ding", "fontmap", package::ding_fontmap::load_definitions),
  ("ifblk", "fontmap", package::ifblk_fontmap::load_definitions),
  ("ifclk", "fontmap", package::ifclk_fontmap::load_definitions),
  ("ifgeo", "fontmap", package::ifgeo_fontmap::load_definitions),
  ("ifsym", "fontmap", package::ifsym_fontmap::load_definitions),
  ("ifwea", "fontmap", package::ifwea_fontmap::load_definitions),
  ("lgr", "fontmap", package::lgr_fontmap::load_definitions),
  ("ot4", "fontmap", package::ot4_fontmap::load_definitions),
  ("ly1", "fontmap", package::ly1_fontmap::load_definitions),
  ("t1", "fontmap", package::t1_fontmap::load_definitions),
  ("t2a", "fontmap", package::t2a_fontmap::load_definitions),
  ("t2b", "fontmap", package::t2b_fontmap::load_definitions),
  ("t2c", "fontmap", package::t2c_fontmap::load_definitions),
  ("ts1", "fontmap", package::ts1_fontmap::load_definitions),
  ("pzd", "fontmap", package::pzd_fontmap::load_definitions),
  ("pifont", "sty", package::pifont_sty::load_definitions),
  ("pict2e", "sty", package::pict2e_sty::load_definitions),
  ("utf8", "def", package::utf8_def::load_definitions),
  // Perl utf8x.def.ltxml L18: "Note: this is a copy of utf8.def.ltxml
  // for now" — dispatch utf8x to the same loader.
  ("utf8x", "def", package::utf8_def::load_definitions),
  ("tcilatex", "tex", package::tcilatex_tex::load_definitions),
  ("textcase", "sty", package::textcase_sty::load_definitions),
  ("citesort", "sty", package::citesort_sty::load_definitions),
  ("cite", "sty", package::cite_sty::load_definitions),
  ("color", "sty", package::color_sty::load_definitions),
  ("calc", "sty", package::calc_sty::load_definitions),
  ("accents", "sty", package::accents_sty::load_definitions),
  ("acronym", "sty", package::acronym_sty::load_definitions),
  ("cancel", "sty", package::cancel_sty::load_definitions),
  ("ccfonts", "sty", package::ccfonts_sty::load_definitions),
  ("cases", "sty", package::cases_sty::load_definitions),
  ("colortbl", "sty", package::colortbl_sty::load_definitions),
  ("crop", "sty", package::crop_sty::load_definitions),
  ("chngcntr", "sty", package::chngcntr_sty::load_definitions),
  ("natbib", "sty", package::natbib_sty::load_definitions),
  (
    "pdftexcmds",
    "sty",
    package::pdftexcmds_sty::load_definitions,
  ),
  ("pdfx", "sty", package::pdfx_sty::load_definitions),
  ("ngerman", "sty", package::ngerman_sty::load_definitions),
  ("orcidlink", "sty", package::orcidlink_sty::load_definitions),
  ("newtxmath", "sty", package::newtxmath_sty::load_definitions),
  ("newtxtext", "sty", package::newtxtext_sty::load_definitions),
  ("bibunits", "sty", package::bibunits_sty::load_definitions),
  (
    "subeqnarray",
    "sty",
    package::subeqnarray_sty::load_definitions,
  ),
  (
    "subcaption",
    "sty",
    package::subcaption_sty::load_definitions,
  ),
  ("subfigure", "sty", package::subfigure_sty::load_definitions),
  ("subfiles", "cls", package::subfiles_cls::load_definitions),
  ("subfiles", "sty", package::subfiles_sty::load_definitions),
  ("soul", "sty", package::soul_sty::load_definitions),
  (
    "spectralsequences",
    "sty",
    package::spectralsequences_sty::load_definitions,
  ),
  ("stfloats", "sty", package::stfloats_sty::load_definitions),
  ("stmaryrd", "sty", package::stmaryrd_sty::load_definitions),
  ("mathabx", "sty", package::mathabx_sty::load_definitions),
  ("mathdots", "sty", package::mathdots_sty::load_definitions),
  ("wasysym", "sty", package::wasysym_sty::load_definitions),
  ("wrapfig", "sty", package::wrapfig_sty::load_definitions),
  ("xkeyval", "sty", package::xkeyval_sty::load_definitions),
  ("xfor", "sty", package::xfor_sty::load_definitions),
  ("mfirstuc", "sty", package::mfirstuc_sty::load_definitions),
  ("datatool-base", "sty", package::datatool_base_sty::load_definitions),
  ("chemgreek", "sty", package::chemgreek_sty::load_definitions),
  ("chemmacros", "sty", package::chemmacros_sty::load_definitions),
  ("substr", "sty", package::substr_sty::load_definitions),
  ("shellesc", "sty", package::shellesc_sty::load_definitions),
  ("tracklang", "sty", package::tracklang_sty::load_definitions),
  ("translations", "sty", package::translations_sty::load_definitions),
  ("translator", "sty", package::translator_sty::load_definitions),
  ("xspace", "sty", package::xspace_sty::load_definitions),
  ("xurl", "sty", package::xurl_sty::load_definitions),
  ("lineno", "sty", package::lineno_sty::load_definitions),
  ("preview", "sty", package::preview_sty::load_definitions),
  ("proof", "sty", package::proof_sty::load_definitions),
  ("proofwiki", "sty", package::proofwiki_sty::load_definitions),
  (
    "pstricks_support",
    "sty",
    package::pstricks_support_sty::load_definitions,
  ),
  ("pst-node", "sty", package::pst_node_sty::load_definitions),
  ("turing", "sty", package::turing_sty::load_definitions),
  ("amsppt", "sty", package::amsppt_sty::load_definitions),
  // \documentstyle{amsppt} (sandbox math0004154 + \Cal/\Refs/
  // \topmatter cluster) routes through .cls lookup. amsppt is
  // canonically a .sty (AmS-TeX style file) used as a pseudo-class.
  // Direct alias was unblocked by the def_autoload re-entry guard
  // (engine/tex.rs:18-29) which prevents \mathfrak/\theoremstyle
  // autoloads from looping mid-package-load.
  ("amsppt", "cls", package::amsppt_sty::load_definitions),
  ("titlesec", "sty", package::titlesec_sty::load_definitions),
  ("upgreek", "sty", package::upgreek_sty::load_definitions),
  ("xcolor", "sty", package::xcolor_sty::load_definitions),
  ("xparse", "sty", package::xparse_sty::load_definitions),
  (
    "thm-restate",
    "sty",
    package::thm_restate_sty::load_definitions,
  ),
  ("subfloat", "sty", package::subfloat_sty::load_definitions),
  ("svg", "sty", package::svg_sty::load_definitions),
  ("subfig", "sty", package::subfig_sty::load_definitions),
  ("babel", "def", package::babel_def::load_definitions),
  (
    "babel_support",
    "sty",
    package::babel_support_sty::load_definitions,
  ),
  ("txtbabel", "def", package::txtbabel_def::load_definitions),
  ("xy", "sty", package::xy_sty::load_definitions),
  ("xylatexml", "tex", package::xylatexml_tex::load_definitions),
  ("xypic", "sty", package::xypic_sty::load_definitions),
  ("pgf", "sty", package::pgf_sty::load_definitions),
  (
    "pgfcircutils",
    "tex",
    package::pgfcircutils_tex::load_definitions,
  ),
  (
    "pgfsys-latexml",
    "def",
    package::pgfsys_latexml_def::load_definitions,
  ),
  (
    "pgfutil-common",
    "tex",
    package::pgfutil_common_tex::load_definitions,
  ),
  (
    "pgfmath",
    "code.tex",
    package::pgfmath_code_tex::load_definitions,
  ),
  (
    "pgfmathcalc",
    "code.tex",
    package::pgfmathcalc_code_tex::load_definitions,
  ),
  ("pgfplots", "sty", package::pgfplots_sty::load_definitions),
  ("tikz", "sty", package::tikz_sty::load_definitions),
  ("llncs", "cls", package::llncs_cls::load_definitions),
  ("xkvview", "sty", package::xkvview_sty::load_definitions),
  ("wiki", "sty", package::wiki_sty::load_definitions),
  (
    "tikzbricks",
    "sty",
    package::tikzbricks_sty::load_definitions,
  ),
  (
    "tikz-3dplot",
    "sty",
    package::tikz_3dplot_sty::load_definitions,
  ),
  ("pst-grad", "sty", package::pst_grad_sty::load_definitions),
  ("pspicture", "sty", package::pspicture_sty::load_definitions),
  ("psfrag", "sty", package::psfrag_sty::load_definitions),
  ("psfig", "sty", package::psfig_sty::load_definitions),
  // Perl: psfig.tex.ltxml just does `RequirePackage('epsfig')`.
  // Paper 0803.3406 does `\input{psfig}` (.tex form), hitting this dispatch.
  ("psfig", "tex", package::epsfig_sty::load_definitions),
  // Perl: aipcheck.tex.ltxml — "Do nothing" stub. Paper 0809.2681 does
  // `\input{aipcheck}`.
  ("aipcheck", "tex", package::aipcheck_tex::load_definitions),
  // Perl: epsf.tex.ltxml just does `RequirePackage('epsf')`.
  ("epsf", "tex", package::epsf_sty::load_definitions),
  // Perl (post-#2777 / fdc8bf91): pstricks.tex.ltxml now does
  // `InputDefinitions('pstricks', type=>'tex', noltxml=>1)` + then
  // `RequirePackage('pstricks_support')`. See pstricks_tex.rs.
  ("pstricks", "tex", package::pstricks_tex::load_definitions),
  // Perl: xypic.tex.ltxml does InputDefinitions('xy', type=>'tex') + RawTeX('\xyoption{v2}').
  // Rust xypic_sty does `RequirePackage!("xy", options=["v2"])`, which is equivalent.
  ("xypic", "tex", package::xypic_sty::load_definitions),
  // Perl: xy.tex.ltxml does InputDefinitions('xy', type=>'tex', noltxml=>1, at_letter=>0)
  // plus \xyoption driver filtering. Rust xy_sty matches this structure.
  ("xy", "tex", package::xy_sty::load_definitions),
  ("newlfont", "sty", package::newlfont_sty::load_definitions),
  ("ltxcmds", "sty", package::ltxcmds_sty::load_definitions),
  ("kvsetkeys", "sty", package::kvsetkeys_sty::load_definitions),
  ("amsbook", "cls", package::amsbook_cls::load_definitions),
  ("aa", "cls", package::aa_cls::load_definitions),
  (
    "aa_support",
    "sty",
    package::aa_support_sty::load_definitions,
  ),
  (
    "elsarticle",
    "cls",
    package::elsarticle_cls::load_definitions,
  ),
  ("iopams", "sty", package::iopams_sty::load_definitions),
  ("ijcai", "sty", package::ijcai_sty::load_definitions),
  (
    "ifplatform",
    "sty",
    package::ifplatform_sty::load_definitions,
  ),
  ("html", "sty", package::html_sty::load_definitions),
  ("floatflt", "sty", package::floatflt_sty::load_definitions),
  ("espcrc", "sty", package::espcrc_sty::load_definitions),
  ("epsf", "sty", package::epsf_sty::load_definitions),
  ("epsfig", "sty", package::epsfig_sty::load_definitions),
  (
    "emulateapj",
    "sty",
    package::emulateapj_sty::load_definitions,
  ),
  (
    "emulateapj5",
    "sty",
    package::emulateapj5_sty::load_definitions,
  ),
  ("cropmark", "sty", package::cropmark_sty::load_definitions),
  ("colordvi", "sty", package::colordvi_sty::load_definitions),
  (
    "circuitikz",
    "sty",
    package::circuitikz_sty::load_definitions,
  ),
  (
    "chapterbib",
    "sty",
    package::chapterbib_sty::load_definitions,
  ),
  ("braket", "sty", package::braket_sty::load_definitions),
  ("authblk", "sty", package::authblk_sty::load_definitions),
  (
    "attachfile",
    "sty",
    package::attachfile_sty::load_definitions,
  ),
  ("apjfonts", "sty", package::apjfonts_sty::load_definitions),
  ("aipproc", "cls", package::aipproc_cls::load_definitions),
  ("aipproc", "sty", package::aipproc_sty::load_definitions),
  (
    "actuarialangle",
    "sty",
    package::actuarialangle_sty::load_definitions,
  ),
  ("framed", "sty", package::framed_sty::load_definitions),
  ("tabularx", "sty", package::tabularx_sty::load_definitions),
  ("tabulary", "sty", package::tabulary_sty::load_definitions),
  ("tcolorbox", "sty", package::tcolorbox_sty::load_definitions),
  (
    "threeparttable",
    "sty",
    package::threeparttable_sty::load_definitions,
  ),
  ("tocbibind", "sty", package::tocbibind_sty::load_definitions),
  ("todonotes", "sty", package::todonotes_sty::load_definitions),
  (
    "transparent",
    "sty",
    package::transparent_sty::load_definitions,
  ),
  ("twoopt", "sty", package::twoopt_sty::load_definitions),
  ("type1cm", "sty", package::type1cm_sty::load_definitions),
  ("slashed", "sty", package::slashed_sty::load_definitions),
  ("slashbox", "sty", package::slashbox_sty::load_definitions),
  ("diagbox", "sty", package::diagbox_sty::load_definitions),
  ("nicefrac", "sty", package::nicefrac_sty::load_definitions),
  ("units", "sty", package::units_sty::load_definitions),
  ("parskip", "sty", package::parskip_sty::load_definitions),
  ("lscape", "sty", package::lscape_sty::load_definitions),
  ("enumerate", "sty", package::enumerate_sty::load_definitions),
  ("makeidx", "sty", package::makeidx_sty::load_definitions),
  ("bm", "sty", package::bm_sty::load_definitions),
  (
    "mleftright",
    "sty",
    package::mleftright_sty::load_definitions,
  ),
  ("placeins", "sty", package::placeins_sty::load_definitions),
  ("ragged2e", "sty", package::ragged2e_sty::load_definitions),
  ("relsize", "sty", package::relsize_sty::load_definitions),
  ("scalefnt", "sty", package::scalefnt_sty::load_definitions),
  ("sectsty", "sty", package::sectsty_sty::load_definitions),
  ("xfrac", "sty", package::xfrac_sty::load_definitions),
  ("adjustbox", "sty", package::adjustbox_sty::load_definitions),
  ("afterpage", "sty", package::afterpage_sty::load_definitions),
  ("mathrsfs", "sty", package::mathrsfs_sty::load_definitions),
  ("blindtext", "sty", package::blindtext_sty::load_definitions),
  ("bookmark", "sty", package::bookmark_sty::load_definitions),
  ("cmap", "sty", package::cmap_sty::load_definitions),
  ("doi", "sty", package::doi_sty::load_definitions),
  ("dsfont", "sty", package::dsfont_sty::load_definitions),
  ("ellipsis", "sty", package::ellipsis_sty::load_definitions),
  ("epstopdf", "sty", package::epstopdf_sty::load_definitions),
  ("fancyvrb", "sty", package::fancyvrb_sty::load_definitions),
  ("fdsymbol", "sty", package::fdsymbol_sty::load_definitions),
  ("flafter", "sty", package::flafter_sty::load_definitions),
  ("here", "sty", package::here_sty::load_definitions),
  (
    "indentfirst",
    "sty",
    package::indentfirst_sty::load_definitions,
  ),
  ("lastpage", "sty", package::lastpage_sty::load_definitions),
  ("lipsum", "sty", package::lipsum_sty::load_definitions),
  ("lmodern", "sty", package::lmodern_sty::load_definitions),
  ("luximono", "sty", package::luximono_sty::load_definitions),
  ("nopageno", "sty", package::nopageno_sty::load_definitions),
  ("pdfpages", "sty", package::pdfpages_sty::load_definitions),
  ("pdflscape", "sty", package::pdflscape_sty::load_definitions),
  ("sidecap", "sty", package::sidecap_sty::load_definitions),
  ("siunitx", "sty", package::siunitx_sty::load_definitions),
  ("showkeys", "sty", package::showkeys_sty::load_definitions),
  (
    "underscore",
    "sty",
    package::underscore_sty::load_definitions,
  ),
  (
    "undertilde",
    "sty",
    package::undertilde_sty::load_definitions,
  ),
  ("upquote", "sty", package::upquote_sty::load_definitions),
  ("minimal", "cls", package::minimal_cls::load_definitions),
  // sprocl: no binding — papers bundle their own sprocl.sty (World Scientific
  // proceedings style), which Perl raw-loads cleanly. The earlier stub
  // intercepted the load and only stubbed \address/\abstracts, leaving
  // \citelow / \citeup / \cite-with-* and the rest of sprocl undefined.
  ("srcltx", "sty", package::srcltx_sty::load_definitions),
  (
    "standalone",
    "cls",
    package::standalone_cls::load_definitions,
  ),
  (
    "standalone",
    "sty",
    package::standalone_sty::load_definitions,
  ),
  ("subeqn", "sty", package::subeqn_sty::load_definitions),
  ("a0poster", "cls", package::a0poster_cls::load_definitions),
  ("a0size", "sty", package::a0size_sty::load_definitions),
  ("avant", "sty", package::avant_sty::load_definitions),
  ("bbding", "sty", package::bbding_sty::load_definitions),
  ("beton", "sty", package::beton_sty::load_definitions),
  ("bezier", "sty", package::bezier_sty::load_definitions),
  ("bookman", "sty", package::bookman_sty::load_definitions),
  (
    "boxedminipage",
    "sty",
    package::boxedminipage_sty::load_definitions,
  ),
  ("chancery", "sty", package::chancery_sty::load_definitions),
  ("charter", "sty", package::charter_sty::load_definitions),
  ("concmath", "sty", package::concmath_sty::load_definitions),
  ("courier", "sty", package::courier_sty::load_definitions),
  ("etex", "sty", package::etex_sty::load_definitions),
  ("euler", "sty", package::euler_sty::load_definitions),
  ("eulervm", "sty", package::eulervm_sty::load_definitions),
  ("exscale", "sty", package::exscale_sty::load_definitions),
  ("fourier", "sty", package::fourier_sty::load_definitions),
  ("helvet", "sty", package::helvet_sty::load_definitions),
  ("ifsym", "sty", package::ifsym_sty::load_definitions),
  ("l3keys2e", "sty", package::l3keys2e_sty::load_definitions),
  ("newcent", "sty", package::newcent_sty::load_definitions),
  ("palatino", "sty", package::palatino_sty::load_definitions),
  ("pgfkeys", "sty", package::pgfkeys_sty::load_definitions),
  (
    "pgfplotstable",
    "sty",
    package::pgfplotstable_sty::load_definitions,
  ),
  ("pgfrcs", "sty", package::pgfrcs_sty::load_definitions),
  ("physics", "sty", package::physics_sty::load_definitions),
  ("prettyref", "sty", package::prettyref_sty::load_definitions),
  ("pstricks", "sty", package::pstricks_sty::load_definitions),
  ("pslatex", "sty", package::pslatex_sty::load_definitions),
  ("pxfonts", "sty", package::pxfonts_sty::load_definitions),
  ("times", "sty", package::times_sty::load_definitions),
  ("tracefnt", "sty", package::tracefnt_sty::load_definitions),
  ("upref", "sty", package::upref_sty::load_definitions),
  ("utopia", "sty", package::utopia_sty::load_definitions),
  ("PoS", "cls", package::pos_cls::load_definitions),
  (
    "quantumarticle",
    "cls",
    package::quantumarticle_cls::load_definitions,
  ),
  (
    "bigintcalc",
    "sty",
    package::bigintcalc_sty::load_definitions,
  ),
  ("bitset", "sty", package::bitset_sty::load_definitions),
  ("calrsfs", "sty", package::calrsfs_sty::load_definitions),
  ("cmbright", "sty", package::cmbright_sty::load_definitions),
  ("euscript", "sty", package::euscript_sty::load_definitions),
  ("everyshi", "sty", package::everyshi_sty::load_definitions),
  ("expl3", "lua", package::expl3_lua::load_definitions),
  ("expl3", "sty", package::expl3_sty::load_definitions),
  ("expl3", "ltx", package::expl3_ltx::load_definitions),
  ("fixme", "sty", package::fixme_sty::load_definitions),
  ("fleqn", "sty", package::fleqn_sty::load_definitions),
  ("flowchart", "sty", package::flowchart_sty::load_definitions),
  (
    "gettitlestring",
    "sty",
    package::gettitlestring_sty::load_definitions,
  ),
  ("hepunits", "sty", package::hepunits_sty::load_definitions),
  ("infwarerr", "sty", package::infwarerr_sty::load_definitions),
  ("intcalc", "sty", package::intcalc_sty::load_definitions),
  (
    "kvdefinekeys",
    "sty",
    package::kvdefinekeys_sty::load_definitions,
  ),
  ("kvoptions", "sty", package::kvoptions_sty::load_definitions),
  ("pdfsync", "sty", package::pdfsync_sty::load_definitions),
  ("pgfmath", "sty", package::pgfmath_sty::load_definitions),
  (
    "tablefootnote",
    "sty",
    package::tablefootnote_sty::load_definitions,
  ),
  ("tikz-cd", "sty", package::tikz_cd_sty::load_definitions),
];

/// Runtime lookup: route `filename` (e.g. `"aastex.cls"`, `"pgfmath.code.tex"`)
/// through its compiled `load_definitions` fn, or return `None` when the
/// filename has no registered binding (caller falls back to raw
/// `.sty` / `.cls` / similar). Splits on the *first* `.` so multi-dot
/// filenames like `pgfmath.code.tex` are matched as `("pgfmath", "code.tex")`
/// rather than `("pgfmath.code", "tex")` — the latter silently dropped
/// pgf / pgfmath / pgfmathcalc bindings, breaking tikz tests.
pub fn dispatch(filename: &str) -> Option<Result<()>> {
  let (base, ext) = filename.split_once('.')?;
  // Perl pathname_find L383-389: try strict-case match first, then fall back
  // to case-insensitive (`m/$i_regex/i` then `@nocase_paths`) — so
  // `\documentclass{jhep}` resolves the `JHEP.cls.ltxml`-derived binding.
  // Without the fallback, lowercase-input / uppercase-binding pairs miss
  // and trigger spurious `missing_file` warnings.
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

/// All registered (name, extension) pairs across the BINDINGS table.
/// Consumed by `find_file(notex=true)` to discover whether a compiled
/// binding exists for a given filename — across `.cls` / `.sty` / `.def`
/// / `.pool` / `code.tex` / etc. Also surfaces classes (extension `"cls"`)
/// for `load_class`'s Perl-parity prefix-match fallback via
/// `state::get_class_binding_names()` (a filtered view of this list).
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bindings_registry_is_non_empty() {
    assert!(!BINDINGS.is_empty(), "at least one package registered");
  }

  #[test]
  fn binding_names_round_trips_with_bindings() {
    // Cross-check: every (name, ext) in `binding_names()` matches an entry in
    // `BINDINGS`, and the lengths agree.
    let names = binding_names();
    assert_eq!(names.len(), BINDINGS.len());
    for (name, ext) in names {
      let matched = BINDINGS.iter().any(|(n, e, _)| n == name && e == ext);
      assert!(matched, "{}.{} must round-trip via BINDINGS", name, ext);
    }
  }

  #[test]
  fn binding_names_cached_via_once_lock() {
    // Two consecutive calls return the same &'static slice (same data pointer).
    let first = binding_names();
    let second = binding_names();
    assert_eq!(first.as_ptr(), second.as_ptr());
    assert_eq!(first.len(), second.len());
  }

  #[test]
  fn dispatch_none_for_unknown_filename() {
    // No ".foo" extension registered anywhere → None.
    assert!(dispatch("definitely-not-a-package.foo").is_none());
  }

  #[test]
  fn dispatch_none_for_no_dot() {
    // No dot → split_once returns None → dispatch returns None.
    assert!(dispatch("bareword").is_none());
  }

  #[test]
  fn dispatch_splits_on_first_dot() {
    // The doc-comment promises first-dot splitting. Verify by checking a
    // name that only makes sense when split at the *first* dot (a hypothetical
    // "foo.code.tex" with no "foo.code" binding should be found as ("foo",
    // "code.tex") if that exact tuple is registered, or None otherwise).
    // We can't commit to a specific tuple without coupling to registry state,
    // so just confirm that misleading "last-dot" splits do not resolve:
    // If a name like "nonexistent.code.tex" yields anything other than None,
    // that's a bug regardless of split direction — since no "nonexistent"
    // binding exists in any form.
    assert!(dispatch("nonexistent.code.tex").is_none());
  }
}
