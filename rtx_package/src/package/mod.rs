#![allow(unreachable_code)]
pub use lazy_static::lazy_static;
pub use libxml::tree::{Namespace, Node};
pub use log::{debug, error, info, warn};
pub use regex::Regex;
pub use std::borrow::Cow;
pub use std::cell::RefCell;
pub use std::collections::HashMap;
pub use std::collections::VecDeque;
pub use std::rc::Rc;

pub use rtx_core::common::dimension::{Dimension, MuDimension};
pub use rtx_core::common::error::*;
pub use rtx_core::common::font;
pub use rtx_core::common::font::Font;
pub use rtx_core::common::glue::{Glue, MuGlue};
pub use rtx_core::common::ligature::{FontTestClosure, Ligature};
pub use rtx_core::common::number::Number;
pub use rtx_core::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
pub use rtx_core::definition::constructor::ConstructorOptions;
pub use rtx_core::definition::expandable::{Expandable, ExpandableOptions};
pub use rtx_core::definition::math_primitive::{MathPrimitive, MathPrimitiveOptions};
pub use rtx_core::definition::primitive::{Primitive, PrimitiveOptions};
pub use rtx_core::definition::register::{NumericOps, Register, RegisterType, RegisterValue};
pub use rtx_core::definition::ConditionalClosure;
pub use rtx_core::definition::{
  BeforeDigestClosure, ConstructionClosure, Definition, DigestionClosure, ExpansionBody, ExpansionClosure, PrimitiveClosure, PrimitiveFn,
  ReplacementClosure,
};
pub use rtx_core::document::resource::*;
pub use rtx_core::document::tag::{TagOptionName, TagOptions};
pub use rtx_core::document::Document;
pub use rtx_core::gullet::Gullet;
pub use rtx_core::keyvals::KeyVals;
pub use rtx_core::list::List;
pub use rtx_core::mouth;
pub use rtx_core::mouth::{Mouth, MouthOptions};
pub use rtx_core::parameter::{Parameter, ParameterExtra, Parameters, ReaderClosure, ReversionClosure};
pub use rtx_core::state::{Scope, State, Stored};
pub use rtx_core::stomach::Stomach;
pub use rtx_core::tbox::Tbox;
pub use rtx_core::token::*;
pub use rtx_core::tokens::Tokens;
pub use rtx_core::util::pathname;
pub use rtx_core::util::radix;
pub use rtx_core::whatsit::Whatsit;
pub use rtx_core::{BoxOps, Core, Digested};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------

// First, re-export the main binding macros
#[macro_use]
pub mod setup_binding_language;

// Second, declare the rust boilerplate and
#[macro_use]
pub mod api_macros;
pub mod api;
// Re-export the public API
pub use self::api::*;
pub use self::api::content::*;
pub use self::api::def_dialect::*;
pub use self::api::counter_dialect::*;
pub use self::api::cleaners::*;

// At the very end, declare the pool
pub use self::pool::tex_functions::*;
pub mod pool;
