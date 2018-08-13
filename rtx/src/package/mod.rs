pub use libxml::tree::{Namespace, Node};
pub use regex::Regex;
pub use std::collections::HashMap;
pub use std::collections::VecDeque;
pub use std::rc::Rc;

pub use rtx_core::common::error::*;
pub use rtx_core::common::font::Font;
pub use rtx_core::common::ligature::Ligature;
pub use rtx_core::common::number::Number;
pub use rtx_core::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
pub use rtx_core::definition::constructor::ConstructorOptions;
pub use rtx_core::definition::expandable::Expandable;
pub use rtx_core::definition::math_primitive::{MathPrimitive, MathPrimitiveOptions};
pub use rtx_core::definition::primitive::{Primitive, PrimitiveOptions};
pub use rtx_core::definition::register::{Register, RegisterValue};
pub use rtx_core::definition::ConditionalClosure;
pub use rtx_core::definition::{
  BeforeDigestClosure, ConstructionClosure, Definition, DigestionClosure, ExpansionClosure,
  ReplacementClosure,
};
pub use rtx_core::document::resource::*;
pub use rtx_core::document::tag::{TagOptionName, TagOptions};
pub use rtx_core::document::Document;
pub use rtx_core::gullet::Gullet;
pub use rtx_core::mouth;
pub use rtx_core::mouth::Mouth;
pub use rtx_core::parameter::{Parameter, Parameters};
pub use rtx_core::state::{Scope, State, Stored};
pub use rtx_core::stomach::Stomach;
pub use rtx_core::tbox::Tbox;
pub use rtx_core::token::Token;
pub use rtx_core::token::*;
pub use rtx_core::tokens::Tokens;
pub use rtx_core::util::pathname;
pub use rtx_core::whatsit::Whatsit;
pub use rtx_core::{BoxOps, Core, Digested};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------

#[macro_use]
pub mod binding_macros;
#[macro_use] // Re-export the main binding macros
pub mod setup;

// Next, import the functions and
pub mod functions;
// Re-export the public API
pub use self::functions::*;
// pub use self::functions::{input_definitions, input_content, parse_prototype, merge_font,
// def_macro, InputDefinitionOptions, RequireOptions};

// At the very end, declare the pool
pub mod pool;
