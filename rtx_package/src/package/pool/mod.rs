// TeX Pool
pub mod tex;
mod tex_accents;
mod tex_alignment;
mod tex_appendix_b_p350_to_p355;
mod tex_appendix_b_p356;
mod tex_appendix_b_p357;
mod tex_appendix_b_p358;
mod tex_appendix_b_p359;
mod tex_appendix_b_p360;
mod tex_appendix_b_p361;
mod tex_appendix_b_p362;
mod tex_appendix_b_p363;
mod tex_appendix_b_p364;
mod tex_appendix_b_to_p349;
mod tex_assignment;
mod tex_boxes;
mod tex_ch24_primitives;
mod tex_ch25_primitives;
mod tex_expandable_primitives;
mod tex_fonts;
mod tex_frontmatter;
pub mod tex_functions; // auxiliary functions
mod tex_math_accents;
mod tex_math_mode;
mod tex_math_style;
mod tex_para;
mod tex_registers;
mod tex_rtx_specific;
mod tex_scripts;
mod tex_setup;
mod tex_special_chars;
mod tex_stray_math_style;

// LaTeX Pool
pub mod latex;
mod latex_ch10_array_and_tabular;
mod latex_ch10_tabbing_environment;
mod latex_ch11_index_and_glossary;
mod latex_ch11_moving_information;
mod latex_ch11_splitting_the_input;
mod latex_ch11_terminal_io;
mod latex_ch12_line_and_page_breaking;
mod latex_ch13_boxes;
mod latex_ch14_pictures_and_color;
mod latex_ch15_font_selection;
mod latex_ch15_special_symbol;
mod latex_ch1_break_command;
mod latex_ch1_documentclass;
mod latex_ch1_environments;
mod latex_ch1_fragile_commands;
mod latex_ch2_document;
mod latex_ch3_sentences_and_paragraphs;
mod latex_ch4_sectioning_and_toc;
mod latex_ch5_packages;
mod latex_ch5_page_styles;
mod latex_ch5_title_page_and_abstract;
mod latex_ch6_displayed_paragraphs;
mod latex_ch6_list_and_trivlist_environments;
mod latex_ch6_list_making_environments;
mod latex_ch6_quotations_and_verse;
mod latex_ch6_verbatim;
mod latex_ch7_math_common_delimiters;
mod latex_ch7_math_common_structures;
mod latex_ch7_math_mode_changing_style;
mod latex_ch7_math_mode_environments;
mod latex_ch8_defining_commands;
mod latex_ch8_defining_environments;
mod latex_ch8_numbering;
mod latex_ch8_theoremlike_environments;
mod latex_ch9_figures_and_tables;
mod latex_ch9_marginal_notes;
mod latex_delimiters;
pub mod latex_functions; // auxiliary functions
mod latex_hook;
mod latex_other_in_appendices;
mod latex_semi_undocumented;
mod latex_tables_3;

// eTeX Pool
pub mod etex;

// pdfTeX Pool
pub mod pdftex;

// Supported package bindings
pub mod alltt_sty;
pub mod amsmath_sty;
pub mod amsthm_sty;
pub mod article_cls;
pub mod comment_sty;
pub mod fontenc_sty;
pub mod ieeetran_cls;
pub mod inputenc_sty;
pub mod t1_fontmap;
pub mod t1enc_def;
pub mod t1enc_sty;
pub mod textcomp_sty;
pub mod url_sty;
pub mod hyperref_sty;
pub mod etoolbox_sty;
pub mod verbatim_sty;
pub mod multido_sty;
pub mod utf8_def;
pub mod textcase_sty;
pub mod cite_sty;

use rtx_core::common::error::Result;
use rtx_core::state::State;
use rtx_core::stomach::Stomach;

pub fn dispatch(filename: &str, stomach: &mut Stomach, state: &mut State) -> Option<Result<()>> {
  Some(match filename {
    "TeX.pool" => tex::load_definitions(stomach, state),
    "LaTeX.pool" => latex::load_definitions(stomach, state),
    "eTeX.pool" => etex::load_definitions(stomach, state),
    "pdfTeX.pool" => pdftex::load_definitions(stomach, state),
    "article.cls" => article_cls::load_definitions(stomach, state),
    "alltt.sty" => alltt_sty::load_definitions(stomach, state),
    "amsmath.sty" => amsmath_sty::load_definitions(stomach, state),
    "amsthm.sty" => amsthm_sty::load_definitions(stomach, state),
    "comment.sty" => comment_sty::load_definitions(stomach, state),
    "IEEEtran.cls" => ieeetran_cls::load_definitions(stomach, state),
    "url.sty" => url_sty::load_definitions(stomach, state),
    "etoolbox.sty" => etoolbox_sty::load_definitions(stomach, state),
    "hyperref.sty" => hyperref_sty::load_definitions(stomach, state),
    "verbatim.sty" => verbatim_sty::load_definitions(stomach, state),
    "fontenc.sty" => fontenc_sty::load_definitions(stomach, state),
    "inputenc.sty" => inputenc_sty::load_definitions(stomach, state),
    "textcomp.sty" => textcomp_sty::load_definitions(stomach, state),
    "multido.sty" => multido_sty::load_definitions(stomach, state),
    "t1enc.sty" => t1enc_def::load_definitions(stomach, state),
    "t1enc.def" => t1enc_sty::load_definitions(stomach, state),
    "t1.fontmap" => t1_fontmap::load_definitions(stomach, state),
    "utf8.def"  => utf8_def::load_definitions(stomach, state),
    "textcase.sty" => textcase_sty::load_definitions(stomach, state),
    "cite.sty" => cite_sty::load_definitions(stomach, state),
    _other => return None,
  })
}
