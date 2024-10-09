//! Generic support layer for any LaTeXML binding file

pub use libxml::tree::{Namespace, Node, NodeType};
pub use log;
pub use once_cell::sync::Lazy;
pub use regex::Regex;
pub use rustc_hash::FxHashMap as HashMap;
pub use std::borrow::Cow;
pub use std::collections::VecDeque;
pub use std::rc::Rc;
pub use std::sync::Arc;
pub use std::str::FromStr;

pub use latexml_core::alignment::cell::Cell;
pub use latexml_core::alignment::template::{Align, Template};
pub use latexml_core::alignment::{Alignment, AlignmentConfig};
pub use latexml_core::common::arena;
pub use latexml_core::common::arena::*;
pub use latexml_core::common::cleaners::*;
pub use latexml_core::common::def_parser::{parse_parameters, parse_prototype};
pub use latexml_core::common::dimension::Dimension;
pub use latexml_core::common::float::{floatformat, Float};
pub use latexml_core::common::font;
pub use latexml_core::common::font::Font;
pub use latexml_core::common::glue::Glue;
pub use latexml_core::common::locator::Locator;
pub use latexml_core::common::mudimension::MuDimension;
pub use latexml_core::common::muglue::MuGlue;
pub use latexml_core::common::number::Number;
pub use latexml_core::common::model;
pub use latexml_core::common::numeric_ops::{NumericOps,UNITY, UNITY_F64};
pub use latexml_core::common::object::Object;
pub use latexml_core::common::xml::XML_NS;
pub use latexml_core::definition::argument::ArgWrap;
pub use latexml_core::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
pub use latexml_core::definition::constructor::ConstructorOptions;
pub use latexml_core::definition::expandable::{Expandable, ExpandableOptions};
pub use latexml_core::definition::math_primitive::{MathPrimitive, MathPrimitiveOptions};
pub use latexml_core::definition::primitive::{Primitive, PrimitiveOptions};
pub use latexml_core::definition::register::{Register, RegisterType, RegisterValue};
pub use latexml_core::definition::ConditionalClosure;
pub use latexml_core::definition::{
  BeforeDigestClosure, ConstructionClosure, Definition, DigestedReversionClosure, DigestionClosure,
  ExpansionBody, ExpansionClosure, FontClosure, FontDirective, PrimitiveClosure, PrimitiveFn,
  ReplacementClosure, Reversion, PrimitiveBody,
};
pub use latexml_core::digested::{Digested, DigestedData};
pub use latexml_core::document::resource::*;
pub use latexml_core::document::tag::{TagOptionName, TagOptions};
pub use latexml_core::document::Document;
pub use latexml_core::gullet::*;
pub use latexml_core::keyval::KeyvalConfig;
pub use latexml_core::keyvals::{KeyVals, KeyvalsConfig};
pub use latexml_core::ligature::{FontTestClosure, Ligature, LigatureMatcher, MathLigatureOptions};
pub use latexml_core::list::List;
pub use latexml_core::mouth;
pub use latexml_core::mouth::{Mouth, MouthOptions};
pub use latexml_core::parameter::{Parameter, Parameters, ReaderClosure, ReversionClosure};
pub use latexml_core::rewrite::{Rewrite, RewriteOptions};
pub use latexml_core::state::*;
pub use latexml_core::stomach::*;
pub use latexml_core::tbox::Tbox;
pub use latexml_core::token::*;
pub use latexml_core::tokens::{Tokens,NO_TOKENS};
pub use latexml_core::util::pathname;
pub use latexml_core::util::radix;
pub use latexml_core::whatsit::Whatsit;
pub use latexml_core::*;
pub use latexml_core::{BoxOps, Core, TexMode};
// Macros:
pub use latexml_core::{
  Tokens, T_ACTIVE, T_ALIGN, T_ARG, T_BEGIN, T_COMMENT, T_CR, T_CS, T_LETTER, T_MARKER, T_MATH,
  T_OTHER, T_PARAM, T_SPACE, T_SUB, T_SUPER,
};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------.

// Re-export the public API available in latexml_core
pub use latexml_core::binding::content::*;
pub use latexml_core::binding::counter::dialect::*;
pub use latexml_core::binding::def::dialect::*;
pub use latexml_core::binding::def::traits::*;

// Define the binding macro layer
#[macro_use]
pub mod setup_binding_language;

// Export the package-level API
pub use crate::engine::base_functions::*;
pub use crate::engine::latex_functions::*;
pub use crate::package::*;
