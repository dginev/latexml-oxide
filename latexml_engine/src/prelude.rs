//! Generic support layer for any LaTeXML binding file

pub use libxml::tree::{Namespace, Node, NodeType};
pub use log;
pub use once_cell::sync::Lazy;
pub use regex::Regex;
pub use rustc_hash::FxHashMap as HashMap;
pub use std::borrow::Cow;
pub use std::collections::VecDeque;
pub use std::rc::Rc;
pub use std::str::FromStr;
pub use std::sync::Arc;

pub use latexml_core::alignment::cell::Cell;
pub use latexml_core::alignment::template::{Align, Template};
pub use latexml_core::alignment::{Alignment, AlignmentConfig};
pub use latexml_core::common::LabelMappingHook;
pub use latexml_core::common::arena;
pub use latexml_core::common::arena::*;
pub use latexml_core::common::cleaners::*;
pub use latexml_core::common::def_parser::{parse_parameters, parse_prototype};
pub use latexml_core::common::dimension::Dimension;
pub use latexml_core::common::float::{Float, floatformat};
pub use latexml_core::common::font;
pub use latexml_core::common::font::Font;
pub use latexml_core::common::glue::Glue;
pub use latexml_core::common::locator::Locator;
pub use latexml_core::common::model;
pub use latexml_core::common::mudimension::MuDimension;
pub use latexml_core::common::muglue::MuGlue;
pub use latexml_core::common::number::Number;
pub use latexml_core::common::numeric_ops::{NumericOps, UNITY, UNITY_F64};
pub use latexml_core::common::object::Object;
pub use latexml_core::common::xml::XML_NS;
pub use latexml_core::definition::ConditionalClosure;
pub use latexml_core::definition::argument::ArgWrap;
pub use latexml_core::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
pub use latexml_core::definition::constructor::ConstructorOptions;
pub use latexml_core::definition::expandable::{Expandable, ExpandableOptions};
pub use latexml_core::definition::math_primitive::{MathPrimitive, MathPrimitiveOptions};
pub use latexml_core::definition::primitive::{Primitive, PrimitiveOptions};
pub use latexml_core::definition::register::{CharDefProps, Register, RegisterType, RegisterValue};
pub use latexml_core::definition::{
  BeforeDigestClosure, ConstructionClosure, Definition, DigestedReversionClosure, DigestionClosure,
  ExpansionBody, ExpansionClosure, FontClosure, FontDirective, PrimitiveBody, PrimitiveClosure,
  PrimitiveFn, PropertiesClosure, ReplacementClosure, Reversion,
};
pub use latexml_core::digested::{Digested, DigestedData};
pub use latexml_core::document::Document;
pub use latexml_core::document::resource::*;
pub use latexml_core::document::tag::{TagOptionName, TagOptions};
pub use latexml_core::gullet::*;
pub use latexml_core::keyval::KeyvalConfig;
pub use latexml_core::keyvals::{KeyVals, KeyvalsConfig};
pub use latexml_core::ligature::{FontTestClosure, Ligature, LigatureMatcher, MathLigatureOptions};
pub use latexml_core::list::List;
pub use latexml_core::mouth;
pub use latexml_core::mouth::{Mouth, MouthOptions};
pub use latexml_core::parameter::{Parameter, Parameters, ReaderClosure, ReversionClosure};
pub use latexml_core::pin;
pub use latexml_core::rewrite::{Rewrite, RewriteOptions};
pub use latexml_core::state::*;
pub use latexml_core::stomach::*;
pub use latexml_core::tbox::Tbox;
pub use latexml_core::token::*;
pub use latexml_core::tokens::{NO_TOKENS, Tokens};
pub use latexml_core::util::pathname;
pub use latexml_core::util::radix;
pub use latexml_core::whatsit::Whatsit;
pub use latexml_core::*;
pub use latexml_core::{BoxOps, Core, TexMode};
// Macros:
pub use latexml_core::{
  T_ACTIVE, T_ALIGN, T_ARG, T_BEGIN, T_COMMENT, T_CR, T_CS, T_LETTER, T_MARKER, T_MATH, T_OTHER,
  T_PARAM, T_SPACE, T_SUB, T_SUPER, Tokens,
};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------.

// Re-export the public API available in latexml_core
pub use latexml_core::binding::content::*;
pub use latexml_core::binding::counter::dialect::*;
pub use latexml_core::binding::def::dialect::*;
pub use latexml_core::binding::def::traits::*;

// (setup_binding_language is declared at the crate-root in lib.rs with
//  #[macro_use] so its #[macro_export]-ed macros are visible from every
//  engine module without `crate::DefMacro!` qualification. Keeping it
//  out of this prelude avoids a duplicate-definition error.)

// Export the engine-level API. After the latexml_engine extraction, the
// `engine` module is the crate itself — paths shorten by one hop.
pub use crate::base_utilities::*;
pub use crate::latex_constructs::{
  begin_appendices, end_appendices, make_note_tags, only_preamble, relocate_footnote,
  start_appendices, tabular_bindings,
};
// Note: `pub use crate::package::*` was here when the prelude lived in
// latexml_package; it doesn't apply at the engine layer (engine has zero
// references to package). The latexml_package prelude re-exports
// `latexml_engine::prelude::*` and adds its own `pub use crate::package::*`.

// Functions callable from constructor templates via &GetKeyVal(#1,key) syntax.
// Returns Option<Digested> to be compatible with both attribute (to_attribute/to_string)
// and body absorption (Into<Option<Digested>>) contexts in constructor templates.
#[allow(non_snake_case)]
pub fn GetKeyVal(keyval_opt: &Option<Digested>, key: &str) -> Option<Digested> {
  match keyval_opt {
    Some(digested) => match digested.data() {
      DigestedData::KeyVals(keyval) => keyval.get_value_digested(key).cloned(),
      _ => None,
    },
    _ => None,
  }
}
