pub mod cleaners;
pub mod content;
pub mod counter_dialect;
pub mod def_dialect;

use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use rtx_core::common::dimension::{Dimension, MuDimension};
use rtx_core::common::error::*;
use rtx_core::common::glue::{Glue, MuGlue};
use rtx_core::common::number::Number;
use rtx_core::common::store::Stored;
use rtx_core::definition::register::*;
use rtx_core::definition::{SizingClosure, Reversion};
use rtx_core::keyvals::KeyVals;
use rtx_core::list::List;
use rtx_core::mouth;
use rtx_core::tbox::Tbox;
use rtx_core::token::*;
use rtx_core::tokens::Tokens;
use rtx_core::whatsit::Whatsit;
use rtx_core::Digested;

// Constants for the API functions stay here as well

lazy_static! {
  static ref CSNAME_MACRO_RE: Regex = Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap();
  static ref CS_RE: Regex = Regex::new(r"^(\\[a-zA-Z@]+)").unwrap();
  static ref SINGLE_CHAR_RE: Regex = Regex::new(r"^(\\.)").unwrap();
  static ref ACTIVE_CHAR_RE: Regex = Regex::new(r"^(.)").unwrap();
  static ref CONDITIONAL_CS_RE: Regex = Regex::new(r"^\\(?:if(.*)|unless)$").unwrap();
  static ref LEADING_PROTOCOL_RE: Regex = Regex::new(r"^\w+:").unwrap();
  static ref TRAILING_SLASH_RE: Regex = Regex::new(r"/$").unwrap();
  static ref SPACES_RE: Regex = Regex::new(r"\s+").unwrap();
  static ref DIRTY_ID_IDIOM_RE: Regex = Regex::new(r"\$\{\}\^\{(?P<label>[^\}]*)\}\$").unwrap();
  static ref NESTED_CHECK_RE: Regex = Regex::new(r"^(\{([^\}]*)\})\s*").unwrap();
  static ref OPTIONAL_CHECK_RE: Regex = Regex::new(r"^(\[([^\]]*)\])\s*").unwrap();
  static ref DEFAULT_CHECK_RE: Regex = Regex::new(r"^Default:(.*)$").unwrap();
  static ref PARAMSPECT_CHECK_RE: Regex = Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap();
  static ref NON_ID_CHARSET_RE: Regex = Regex::new(r"[^\w_\-.]+").unwrap();
  static ref TILDE_NOISE_RE: Regex = Regex::new(r"\\~\{\}").unwrap();
}

// Rust-specific type wrangling stays in the main mod file for convenience

pub trait IntoOption<T>: Sized {
  /// Performs the conversion.
  fn into_option(self) -> T;
}

impl<'a> IntoOption<Option<String>> for &'a str {
  fn into_option(self) -> Option<String> { Some(self.to_string()) }
}

impl<T> IntoOption<Option<T>> for Option<T> {
  fn into_option(self) -> Option<T> { self }
}

impl IntoOption<bool> for bool {
  fn into_option(self) -> bool { self }
}

impl<T> IntoOption<Option<Vec<T>>> for Vec<T> {
  fn into_option(self) -> Option<Vec<T>> { Some(self) }
}

impl<T> IntoOption<Option<VecDeque<T>>> for VecDeque<T> {
  fn into_option(self) -> Option<VecDeque<T>> { Some(self) }
}

impl IntoOption<Option<Reversion>> for Tokens {
  fn into_option(self) -> Option<Reversion> {
    if self.is_empty() {
      None
    } else {
      Some(Reversion::Tokens(self))
    }
  }
}
impl IntoOption<Option<Reversion>> for &str {
  fn into_option(self) -> Option<Reversion> {
    if self.is_empty() {
      None
    } else {
      Some(Reversion::Tokens(TokenizeInternal!(self)))
    }
  }
}

impl IntoOption<Option<SizingClosure>> for &str {
  fn into_option(self) -> Option<SizingClosure> {
    if self.is_empty() {
      None
    } else if self == "#1" {
      Some(Rc::new(|w| unimplemented!()))
    } else {
      unimplemented!()
    }
  }
}


pub trait IntoTokensResult<T>: Sized {
  /// Performs the conversion, used for DefMacro return values etc
  fn into_tokens_result(self) -> Result<Tokens>;
}

impl IntoTokensResult<Result<Tokens>> for Token {
  fn into_tokens_result(self) -> Result<Tokens> { Ok(Tokens!(self)) }
}

impl IntoTokensResult<Result<Tokens>> for Vec<Token> {
  fn into_tokens_result(self) -> Result<Tokens> { Ok(Tokens::new(self)) }
}

impl IntoTokensResult<Result<Tokens>> for Tokens {
  fn into_tokens_result(self) -> Result<Tokens> { Ok(self) }
}

impl IntoTokensResult<Result<Tokens>> for Result<Tokens> {
  fn into_tokens_result(self) -> Result<Tokens> { self }
}

impl IntoTokensResult<Result<Tokens>> for () {
  fn into_tokens_result(self) -> Result<Tokens> { Ok(Tokens!()) }
}

pub trait IntoBoolResult<T>: Sized {
  /// Performs the conversion, used for DefConditional return values etc
  fn into_bool_result(self) -> Result<bool>;
}
impl IntoBoolResult<Result<bool>> for bool {
  fn into_bool_result(self) -> Result<bool> { Ok(self) }
}
impl IntoBoolResult<Result<bool>> for Result<bool> {
  fn into_bool_result(self) -> Result<bool> { self }
}

pub trait IntoDigestedResult<T>: Sized {
  /// Performs the conversion, used for DefPrimitive return values etc
  fn into_digested_result(self) -> Result<Vec<Digested>>;
}
impl IntoDigestedResult<Result<Vec<Digested>>> for () {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(Vec::new()) }
}
impl IntoDigestedResult<Result<Vec<Digested>>> for Tbox {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(vec![self.into()]) }
}

impl IntoDigestedResult<Result<Vec<Digested>>> for Whatsit {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(vec![self.into()]) }
}

impl IntoDigestedResult<Result<Vec<Digested>>> for List {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(vec![self.into()]) }
}

impl IntoDigestedResult<Result<Vec<Digested>>> for Digested {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(vec![self]) }
}

impl IntoDigestedResult<Result<Vec<Digested>>> for Vec<Digested> {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(self) }
}

impl IntoDigestedResult<Result<Vec<Digested>>> for Result<Vec<Digested>> {
  fn into_digested_result(self) -> Result<Vec<Digested>> { self }
}

pub trait IntoRegisterValueOption<T>: Sized {
  fn into_register_value_option(self) -> Option<RegisterValue>;
}
impl IntoRegisterValueOption<Option<RegisterValue>> for () {
  fn into_register_value_option(self) -> Option<RegisterValue> { None }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Option<RegisterValue> {
  fn into_register_value_option(self) -> Option<RegisterValue> { self }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Number {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Number(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Dimension {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Dimension(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for MuDimension {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::MuDimension(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Glue {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Glue(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for MuGlue {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::MuGlue(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Tokens {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Tokens(self)) }
}

impl IntoRegisterValueOption<Option<RegisterValue>> for Option<Number> {
  fn into_register_value_option(self) -> Option<RegisterValue> { self.map(RegisterValue::Number) }
}

// Convenience methods for predigest closures that require Result<Option<Digested>>
pub trait IntoDigestedOptionResult<T>: Sized {
  fn into_digested_option_result(self) -> Result<Option<Digested>>;
}

impl IntoDigestedOptionResult<Result<Option<Digested>>> for Glue {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::Glue(self).into_digested_option_result() }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for MuGlue {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::MuGlue(self).into_digested_option_result() }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for Dimension {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::Dimension(self).into_digested_option_result() }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for MuDimension {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::MuDimension(self).into_digested_option_result() }
}

impl IntoDigestedOptionResult<Result<Option<Digested>>> for Number {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::Number(self).into_digested_option_result() }
}

impl IntoDigestedOptionResult<Result<Option<Digested>>> for RegisterValue {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(Some(self.into())) }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for Option<Digested> {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(self) }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for Result<Option<Digested>> {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { self }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for KeyVals {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(Some(Digested::KeyVals(Rc::new(self)))) }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for Option<KeyVals> {
  fn into_digested_option_result(self) -> Result<Option<Digested>> {
    match self {
      None => Ok(None),
      Some(kv) => kv.into(),
    }
  }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for List {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(Some(Digested::List(Rc::new(self)))) }
}

pub trait IntoPropertiesResult {
  fn into_properties_result(self) -> Result<HashMap<String, Stored>>;
}
impl IntoPropertiesResult for HashMap<String, Stored> {
  fn into_properties_result(self) -> Result<HashMap<String, Stored>> { Ok(self) }
}
impl IntoPropertiesResult for Result<HashMap<String, Stored>> {
  fn into_properties_result(self) -> Result<HashMap<String, Stored>> { self }
}

pub trait IntoFontField<T>: Sized {
  fn into_font_field(self) -> T;
}

impl IntoFontField<Option<bool>> for bool {
  fn into_font_field(self) -> Option<bool> { Some(self) }
}

impl IntoFontField<bool> for bool {
  fn into_font_field(self) -> bool { self }
}

impl IntoFontField<Option<Cow<'static, str>>> for &'static str {
  fn into_font_field(self) -> Option<Cow<'static, str>> { Some(Cow::Borrowed(self)) }
}
impl IntoFontField<Option<Cow<'static, str>>> for f32 {
  fn into_font_field(self) -> Option<Cow<'static, str>> { Some(Cow::Owned(self.to_string())) }
}
impl IntoFontField<Option<Cow<'static, str>>> for i32 {
  fn into_font_field(self) -> Option<Cow<'static, str>> { Some(Cow::Owned(self.to_string())) }
}
