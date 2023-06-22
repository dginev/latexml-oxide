#![allow(unreachable_code)]
pub use libxml::tree::{Namespace, Node};
pub use log;
pub use once_cell::sync::Lazy;
pub use regex::Regex;
pub use rustc_hash::FxHashMap as HashMap;
pub use std::borrow::Cow;
pub use std::collections::VecDeque;
pub use std::rc::Rc;
pub use std::sync::Arc;
pub use std::str::FromStr;
pub use string_interner::symbol::SymbolU32;

pub use rtx_core::alignment::cell::Cell;
pub use rtx_core::alignment::template::{Align, Template};
pub use rtx_core::alignment::{Alignment, AlignmentConfig};
pub use rtx_core::aux_macros::*;
pub use rtx_core::common::arena::{self, EMPTY_SYM};
pub use rtx_core::common::cleaners::*;
pub use rtx_core::common::def_parser::{parse_parameters, parse_prototype};
pub use rtx_core::common::dimension::Dimension;
pub use rtx_core::common::float::{floatformat, Float};
pub use rtx_core::common::font;
pub use rtx_core::common::font::Font;
pub use rtx_core::common::glue::Glue;
pub use rtx_core::common::locator::Locator;
pub use rtx_core::common::mudimension::MuDimension;
pub use rtx_core::common::muglue::MuGlue;
pub use rtx_core::common::number::Number;
pub use rtx_core::common::numeric_ops::NumericOps;
pub use rtx_core::common::object::Object;
pub use rtx_core::common::xml::XML_NS;
pub use rtx_core::definition::argument::ArgWrap;
pub use rtx_core::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
pub use rtx_core::definition::constructor::ConstructorOptions;
pub use rtx_core::definition::expandable::{Expandable, ExpandableOptions};
pub use rtx_core::definition::math_primitive::{MathPrimitive, MathPrimitiveOptions};
pub use rtx_core::definition::primitive::{Primitive, PrimitiveOptions};
pub use rtx_core::definition::register::{Register, RegisterType, RegisterValue};
pub use rtx_core::definition::ConditionalClosure;
pub use rtx_core::definition::{
  BeforeDigestClosure, ConstructionClosure, Definition, DigestedReversionClosure, DigestionClosure,
  ExpansionBody, ExpansionClosure, FontClosure, FontDirective, PrimitiveClosure, PrimitiveFn,
  ReplacementClosure, Reversion,
};
pub use rtx_core::digested::{Digested, DigestedData};
pub use rtx_core::document::resource::*;
pub use rtx_core::document::tag::{TagOptionName, TagOptions};
pub use rtx_core::document::Document;
pub use rtx_core::gullet::Gullet;
pub use rtx_core::keyval::KeyvalConfig;
pub use rtx_core::keyvals::{KeyVals, KeyvalsConfig};
pub use rtx_core::ligature::{FontTestClosure, Ligature, LigatureMatcher, MathLigatureOptions};
pub use rtx_core::list::List;
pub use rtx_core::mouth;
pub use rtx_core::mouth::{Mouth, MouthOptions};
pub use rtx_core::parameter::{Parameter, Parameters, ReaderClosure, ReversionClosure};
pub use rtx_core::rewrite::{Rewrite, RewriteOptions};
pub use rtx_core::state::{Scope, Stored};
pub use rtx_core::stomach::Stomach;
pub use rtx_core::tbox::Tbox;
pub use rtx_core::token::*;
pub use rtx_core::tokens::Tokens;
pub use rtx_core::util::pathname;
pub use rtx_core::util::radix;
pub use rtx_core::whatsit::Whatsit;
pub use rtx_core::*;
pub use rtx_core::{BoxOps, Core, TexMode};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------

// First, re-export the main binding macros
#[macro_use]
pub mod setup_binding_language;

// Re-export the public API available in rtx_core
pub use rtx_core::binding::content::*;
pub use rtx_core::binding::counter::dialect::*;
pub use rtx_core::binding::def::dialect::*;
pub use rtx_core::binding::def::macros::*;
pub use rtx_core::binding::def::traits::*;

// At the very end, declare the pool
pub use self::latex_functions::*;
pub use self::tex_functions::*;

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
mod tex_math_fork;
mod tex_math_style;
mod tex_misc_tweaks;
mod tex_para;
mod tex_registers;
mod tex_rtx_specific;
mod rtx_math_macros;
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
pub mod amsbsy_sty;
pub mod amsgen_sty;
pub mod amsfonts_sty;
pub mod amssymb_sty;
pub mod amsthm_sty;
pub mod fullpage_sty;
pub mod article_cls;
pub mod cite_sty;
pub mod comment_sty;
pub mod etoolbox_sty;
pub mod fontenc_sty;
pub mod nameref_sty;
pub mod hyperref_sty;
pub mod ieeetran_cls;
pub mod inputenc_sty;
pub mod latexml_sty;
pub mod multido_sty;
pub mod t1_fontmap;
pub mod t1enc_def;
pub mod t1enc_sty;
pub mod textcase_sty;
pub mod textcomp_sty;
pub mod url_sty;
pub mod utf8_def;
pub mod verbatim_sty;

// TODO: This entire file may be better left generated by rtx_codegen at compile time?
// that way it will always be dynamically updated based on the files in
//  rtx_package/src/package
pub fn dispatch(filename: &str) -> Option<Result<()>> {
  Some(match filename {
    "TeX.pool" => tex::load_definitions(),
    "LaTeX.pool" => latex::load_definitions(),
    "eTeX.pool" => etex::load_definitions(),
    "pdfTeX.pool" => pdftex::load_definitions(),
    "latexml.sty" => latexml_sty::load_definitions(),
    "article.cls" => article_cls::load_definitions(),
    "alltt.sty" => alltt_sty::load_definitions(),
    "amsmath.sty" => amsmath_sty::load_definitions(),
    "amsfonts.sty" => amsfonts_sty::load_definitions(),
    "amssymb.sty" => amssymb_sty::load_definitions(),
    "amsthm.sty" => amsthm_sty::load_definitions(),
    "amsbsy.sty" => amsbsy_sty::load_definitions(),
    "amsgen.sty" => amsgen_sty::load_definitions(),
    "fullpage.sty" => fullpage_sty::load_definitions(),
    "comment.sty" => comment_sty::load_definitions(),
    "IEEEtran.cls" => ieeetran_cls::load_definitions(),
    "url.sty" => url_sty::load_definitions(),
    "etoolbox.sty" => etoolbox_sty::load_definitions(),
    "hyperref.sty" => hyperref_sty::load_definitions(),
    "nameref.sty" => nameref_sty::load_definitions(),
    "verbatim.sty" => verbatim_sty::load_definitions(),
    "fontenc.sty" => fontenc_sty::load_definitions(),
    "inputenc.sty" => inputenc_sty::load_definitions(),
    "textcomp.sty" => textcomp_sty::load_definitions(),
    "multido.sty" => multido_sty::load_definitions(),
    "t1enc.sty" => t1enc_def::load_definitions(),
    "t1enc.def" => t1enc_sty::load_definitions(),
    "t1.fontmap" => t1_fontmap::load_definitions(),
    "utf8.def" => utf8_def::load_definitions(),
    "textcase.sty" => textcase_sty::load_definitions(),
    "cite.sty" => cite_sty::load_definitions(),
    _other => return None,
  })
}
