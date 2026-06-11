//! Generic support layer for any LaTeXML binding file

pub use std::{borrow::Cow, collections::VecDeque, rc::Rc, str::FromStr, sync::Arc};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------.

// Re-export the public API available in latexml_core
pub use latexml_core::binding::content::*;
pub use latexml_core::{
  BoxOps, Core, TexMode,
  alignment::{
    Alignment, AlignmentConfig,
    cell::Cell,
    template::{Align, Template},
  },
  binding::{
    counter::dialect::*,
    def::{dialect::*, traits::*},
  },
  common::{
    LabelMappingHook, arena,
    arena::*,
    cleaners::*,
    def_parser::{parse_parameters, parse_prototype},
    dimension::Dimension,
    float::{Float, floatformat},
    font,
    font::Font,
    glue::Glue,
    locator::Locator,
    model,
    mudimension::MuDimension,
    muglue::MuGlue,
    number::Number,
    numeric_ops::{NumericOps, UNITY, UNITY_F64},
    object::Object,
    xml::XML_NS,
  },
  definition::{
    BeforeDigestClosure, ConditionalClosure, ConstructionClosure, Definition,
    DigestedReversionClosure, DigestionClosure, ExpansionBody, ExpansionClosure, FontClosure,
    FontDirective, PrimitiveBody, PrimitiveClosure, PrimitiveFn, PropertiesClosure,
    ReplacementClosure, Reversion,
    argument::ArgWrap,
    conditional::{Conditional, ConditionalOptions, ConditionalType},
    constructor::ConstructorOptions,
    expandable::{Expandable, ExpandableOptions},
    math_primitive::{MathPrimitive, MathPrimitiveOptions},
    primitive::{Primitive, PrimitiveOptions},
    register::{CharDefProps, Register, RegisterType, RegisterValue},
  },
  digested::{Digested, DigestedData},
  document::{
    Document,
    resource::*,
    tag::{TagOptionName, TagOptions},
  },
  gullet::*,
  keyval::KeyvalConfig,
  keyvals::{KeyVals, KeyvalsConfig},
  ligature::{FontTestClosure, Ligature, LigatureMatcher, MathLigatureOptions},
  list::List,
  mouth,
  mouth::{Mouth, MouthOptions},
  parameter::{Parameter, Parameters, ReaderClosure, ReversionClosure},
  pin,
  rewrite::{Rewrite, RewriteOptions},
  state::*,
  stomach::*,
  tbox::Tbox,
  token::*,
  tokens::{NO_TOKENS, Tokens},
  util::{pathname, radix},
  whatsit::Whatsit,
  *,
};
// Macros:
pub use latexml_core::{
  T_ACTIVE, T_ALIGN, T_ARG, T_BEGIN, T_COMMENT, T_CR, T_CS, T_LETTER, T_MARKER, T_MATH, T_OTHER,
  T_PARAM, T_SPACE, T_SUB, T_SUPER, Tokens,
};
pub use libxml::tree::{Namespace, Node, NodeType};
pub use log;
pub use once_cell::sync::Lazy;
pub use regex::Regex;
pub use rustc_hash::FxHashMap as HashMap;

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

// ============================================================
// DEP-18/19/20 data-drive helpers — shared across all binding files
// (txfonts, mathabx, amssymb, math_common, biblatex, beamer, …).
// Each replaces a compile-time-inlined macro arm with a runtime call.
// LTO inlines the helper at each call site, so the binary impact is
// the same as a per-file copy; defining once here removes ~200
// source-level duplicates of the same body. See memory:
//   - wisdom_data_drive_min_call_sites (≥5 sites per file threshold)
//   - wisdom_helper_monomorphization_trap (no generic T: Into<X>)
// ============================================================

/// Empty-body `DefMacro!("\\cs[opt-spec]", "")` stub via runtime call.
pub fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

/// Identity `DefMacro!("\\cs{}", "#1")` — CS takes one mandatory arg
/// and expands to it unchanged.
pub fn def_macro_identity(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("#1");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

/// Empty-body `DefPrimitive!("\\cs[opt-spec]", None);` stub —
/// digestion-time no-op primitive (no Box emitted).
pub fn def_primitive_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  def_primitive(cs_tok, params, None, PrimitiveOptions::default())?;
  Ok(())
}

/// DEP-17 `DefMath!("\\cs", "char"[, role => "X"[, meaning => "Y"]])`
/// shape. 2-arg form: prototype includes the CS; params come from
/// `parse_prototype` (Some(empty)).
pub fn def_math_sym(
  cs: &str,
  present: &str,
  role: Option<&str>,
  meaning: Option<&str>,
) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let mut opts = MathPrimitiveOptions::default();
  if let Some(r) = role {
    opts.role = Some(r.to_string());
  }
  if let Some(m) = meaning {
    opts.meaning = Some(m.to_string());
  }
  def_math(cs_tok, params, present.to_string(), opts)?;
  Ok(())
}

/// DEP-17d `DefMath!("\\cs", None, "char"[, ...])` 3-arg form —
/// Token built directly via `T_CS!`, params stays None.
pub fn def_math_atom(
  cs: &str,
  present: &str,
  role: Option<&str>,
  meaning: Option<&str>,
) -> Result<()> {
  let cs_tok = T_CS!(cs);
  let mut opts = MathPrimitiveOptions::default();
  if let Some(r) = role {
    opts.role = Some(r.to_string());
  }
  if let Some(m) = meaning {
    opts.meaning = Some(m.to_string());
  }
  def_math(cs_tok, None, present.to_string(), opts)?;
  Ok(())
}
