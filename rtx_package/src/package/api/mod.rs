pub mod cleaners;
pub mod content;
pub mod counter_dialect;
pub mod def_dialect;

use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use rtx_core::common::dimension::Dimension;
use rtx_core::common::error::*;
use rtx_core::common::float::Float;
use rtx_core::common::glue::Glue;
use rtx_core::common::mudimension::MuDimension;
use rtx_core::common::muglue::MuGlue;
use rtx_core::common::number::Number;
use rtx_core::common::store::Stored;
use rtx_core::definition::argument::ArgWrap;
use rtx_core::definition::register::*;
use rtx_core::definition::{Reversion, SizingClosure};
use rtx_core::keyvals::KeyVals;
use rtx_core::list::List;
use rtx_core::mouth;
use rtx_core::state::Scope;
use rtx_core::tbox::Tbox;
use rtx_core::token::*;
use rtx_core::tokens::Tokens;
use rtx_core::whatsit::Whatsit;
use rtx_core::Digested;

// Constants for the API functions stay here as well

lazy_static! {
  static ref CONDITIONAL_CS_RE: Regex = Regex::new(r"^\\(?:if(.*)|unless)$").unwrap();
  static ref LEADING_PROTOCOL_RE: Regex = Regex::new(r"^\w+:").unwrap();
  static ref TRAILING_SLASH_RE: Regex = Regex::new(r"/$").unwrap();
  static ref SPACES_RE: Regex = Regex::new(r"\s+").unwrap();
  static ref DIRTY_ID_IDIOM_RE: Regex = Regex::new(r"\$\{\}\^\{(?P<label>[^\}]*)\}\$").unwrap();
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
impl IntoOption<Option<usize>> for usize {
  fn into_option(self) -> Option<usize> { Some(self) }
}

impl IntoOption<Option<Reversion>> for Tokens {
  fn into_option(self) -> Option<Reversion> { Some(Reversion::Tokens(self)) }
}
impl IntoOption<Option<Reversion>> for &str {
  fn into_option(self) -> Option<Reversion> {
    if self.is_empty() {
      Some(Reversion::Tokens(Tokens!()))
    } else {
      Some(Reversion::Tokens(TokenizeInternal!(self).pack_parameters()))
    }
  }
}

impl IntoOption<Option<Scope>> for &str {
  fn into_option(self) -> Option<Scope> {
    match self {
      "" => None,
      "local" => Some(Scope::Local),
      "Local" => Some(Scope::Local),
      "LOCAL" => Some(Scope::Local),
      "global" => Some(Scope::Global),
      "Global" => Some(Scope::Global),
      "GLOBAL" => Some(Scope::Global),
      other => Some(Scope::Named(other.to_string())),
    }
  }
}
impl IntoOption<Option<Scope>> for String {
  fn into_option(self) -> Option<Scope> {
    match self.as_ref() {
      "" => None,
      "local" => Some(Scope::Local),
      "Local" => Some(Scope::Local),
      "LOCAL" => Some(Scope::Local),
      "global" => Some(Scope::Global),
      "Global" => Some(Scope::Global),
      "GLOBAL" => Some(Scope::Global),
      _ => Some(Scope::Named(self)),
    }
  }
}

impl IntoOption<Option<SizingClosure>> for &str {
  fn into_option(self) -> Option<SizingClosure> {
    if self.is_empty() {
      None
    } else if self == "#1" {
      Some(Arc::new(|w| unimplemented!()))
    } else {
      Some(Arc::new(|w| unimplemented!()))
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

impl IntoTokensResult<Result<Tokens>> for ArgWrap {
  // TODO: maybe this should be .revert() ?
  fn into_tokens_result(self) -> Result<Tokens> { Ok(self.owned_tokens().unwrap_or_default()) }
}
impl IntoTokensResult<Result<Tokens>> for Result<ArgWrap> {
  // TODO: maybe this should be .revert() ?
  fn into_tokens_result(self) -> Result<Tokens> { self.map(|w| w.owned_tokens().unwrap_or_default()) }
}

pub trait IntoResultOptTokens<T>: Sized {
  fn into_result_opt_tokens(self) -> Result<Option<Tokens>>;
}

impl IntoResultOptTokens<Result<Option<Tokens>>> for Token {
  fn into_result_opt_tokens(self) -> Result<Option<Tokens>> { Ok(Some(Tokens!(self))) }
}

impl IntoResultOptTokens<Result<Option<Tokens>>> for Vec<Token> {
  fn into_result_opt_tokens(self) -> Result<Option<Tokens>> { Ok(Some(Tokens::new(self))) }
}

impl IntoResultOptTokens<Result<Option<Tokens>>> for Tokens {
  fn into_result_opt_tokens(self) -> Result<Option<Tokens>> { Ok(Some(self)) }
}

impl IntoResultOptTokens<Result<Option<Tokens>>> for Result<Tokens> {
  fn into_result_opt_tokens(self) -> Result<Option<Tokens>> { self.map(Some) }
}

impl IntoResultOptTokens<Result<Option<Tokens>>> for Result<Option<Tokens>> {
  fn into_result_opt_tokens(self) -> Result<Option<Tokens>> { self }
}

impl IntoResultOptTokens<Result<Option<Tokens>>> for () {
  fn into_result_opt_tokens(self) -> Result<Option<Tokens>> { Ok(None) }
}

pub trait IntoResultArgWrap<T>: Sized {
  fn into_result_argwrap(self) -> Result<ArgWrap>;
}

impl IntoResultArgWrap<Result<ArgWrap>> for Token {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Tokens(Tokens!(self))) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Vec<Token> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Tokens(Tokens::new(self))) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Tokens {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Tokens(self)) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Result<Tokens> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { self.map(ArgWrap::Tokens) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Result<Option<Tokens>> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { self.map(ArgWrap::OptionTokens) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Number {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Number(self)) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Option<Number> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionNumber(self)) }
}
impl IntoResultArgWrap<Result<ArgWrap>> for Float {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Float(self)) }
}
impl IntoResultArgWrap<Result<ArgWrap>> for Option<Float> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionFloat(self)) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Dimension {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Dimension(self)) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Option<Dimension> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionDimension(self)) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for MuDimension {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::MuDimension(self)) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Option<MuDimension> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionMuDimension(self)) }
}
impl IntoResultArgWrap<Result<ArgWrap>> for Glue {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Glue(self)) }
}
impl IntoResultArgWrap<Result<ArgWrap>> for Option<Glue> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionGlue(self)) }
}
impl IntoResultArgWrap<Result<ArgWrap>> for MuGlue {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::MuGlue(self)) }
}
impl IntoResultArgWrap<Result<ArgWrap>> for Option<MuGlue> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionMuGlue(self)) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Result<ArgWrap> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { self }
}

impl IntoResultArgWrap<Result<ArgWrap>> for () {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionTokens(None)) }
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
impl IntoDigestedResult<Result<Vec<Digested>>> for Result<Digested> {
  fn into_digested_result(self) -> Result<Vec<Digested>> { self.map(|d| vec![d]) }
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
impl IntoRegisterValueOption<Option<RegisterValue>> for usize {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Number(Number(self as i32))) }
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
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(Some(Digested::KeyVals(Arc::new(self)))) }
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
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(Some(Digested::List(Arc::new(self)))) }
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
impl IntoFontField<Option<f32>> for f32 {
  fn into_font_field(self) -> Option<f32> { Some(self) }
}
impl IntoFontField<Option<f32>> for i32 {
  fn into_font_field(self) -> Option<f32> { Some(self as f32) }
}
