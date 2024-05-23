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
pub use string_interner::symbol::SymbolU32;

pub use rtx_core::alignment::cell::Cell;
pub use rtx_core::alignment::template::{Align, Template};
pub use rtx_core::alignment::{Alignment, AlignmentConfig};
pub use rtx_core::common::arena;
pub use rtx_core::common::arena::*;
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
pub use rtx_core::common::model;
pub use rtx_core::common::numeric_ops::{NumericOps,UNITY, UNITY_F64};
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
  ReplacementClosure, Reversion, PrimitiveBody,
};
pub use rtx_core::digested::{Digested, DigestedData};
pub use rtx_core::document::resource::*;
pub use rtx_core::document::tag::{TagOptionName, TagOptions};
pub use rtx_core::document::Document;
pub use rtx_core::gullet::*;
pub use rtx_core::keyval::KeyvalConfig;
pub use rtx_core::keyvals::{KeyVals, KeyvalsConfig};
pub use rtx_core::ligature::{FontTestClosure, Ligature, LigatureMatcher, MathLigatureOptions};
pub use rtx_core::list::List;
pub use rtx_core::mouth;
pub use rtx_core::mouth::{Mouth, MouthOptions};
pub use rtx_core::parameter::{Parameter, Parameters, ReaderClosure, ReversionClosure};
pub use rtx_core::rewrite::{Rewrite, RewriteOptions};
pub use rtx_core::state::*;
pub use rtx_core::stomach::*;
pub use rtx_core::tbox::Tbox;
pub use rtx_core::token::*;
pub use rtx_core::tokens::{Tokens,NO_TOKENS};
pub use rtx_core::util::pathname;
pub use rtx_core::util::radix;
pub use rtx_core::whatsit::Whatsit;
pub use rtx_core::*;
pub use rtx_core::{BoxOps, Core, TexMode};
// Macros:
pub use rtx_core::{
  Tokens, T_ACTIVE, T_ALIGN, T_ARG, T_BEGIN, T_COMMENT, T_CR, T_CS, T_LETTER, T_MARKER, T_MATH,
  T_OTHER, T_PARAM, T_SPACE, T_SUB, T_SUPER,
};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------.

// Re-export the public API available in rtx_core
pub use rtx_core::binding::content::*;
pub use rtx_core::binding::counter::dialect::*;
pub use rtx_core::binding::def::dialect::*;
pub use rtx_core::binding::def::traits::*;

// Define the binding macro layer
#[macro_use]
pub mod setup_binding_language;

// Export the package-level API
pub use crate::engine::*;
pub use crate::package::*;
