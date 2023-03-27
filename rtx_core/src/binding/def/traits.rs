///! A variety of traits helpful for auto-casting between the different components of the conversion toolchain
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::float::Float;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::store::Stored;
use crate::definition::argument::ArgWrap;
use crate::definition::register::*;
use crate::definition::{Reversion, SizingClosure};
use crate::keyvals::KeyVals;
use crate::list::List;
use crate::state::Scope;
use crate::tbox::Tbox;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::*;
use crate::{BoxOps, Digested};

/// A trait for auto-wrapping a generic type T into Option<Y>,
/// where Y can be inferred from context.
/// (useful in macro helpers, such as `NewDefaultV!`)
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
impl IntoOption<Option<bool>> for bool {
  fn into_option(self) -> Option<bool> { Some(self) }
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
      Some(Reversion::Tokens(mouth::tokenize_internal(self).pack_parameters()))
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

// TODO: Sizers need a lot more work, likely a complete rethink about organization.
impl IntoOption<Option<SizingClosure>> for &str {
  fn into_option(self) -> Option<SizingClosure> {
    if self.is_empty() {
      None
    } else if let Some(stripped) = self.strip_prefix('#') {
      let arg = stripped.parse::<usize>().unwrap_or(1);
      Some(Arc::new(move |w, state| match w.get_arg(arg) {
        Some(arg) => arg.compute_size(HashMap::new(), state),
        None => Ok((Dimension::default(), Dimension::default(), Dimension::default())),
      }))
    } else if self.is_empty() || self == "0" {
      Some(Arc::new(|_, _| Ok((Dimension::default(), Dimension::default(), Dimension::default()))))
    } else {
      // literal string, get its size with the current font?
      let sized_data = String::from(self);
      Some(Arc::new(move |w, state| {
        let font = if let Stored::Font(ref font) = *w.get_property("font").unwrap() {
          font.clone()
        } else {
          state.lookup_font().unwrap()
        };
        font.compute_boxes_size(
          &[Digested::from(Tbox {
            text: sized_data.clone(),
            ..Tbox::default()
          })],
          HashMap::new(),
          state,
        )
      }))
    }
  }
}

/// A trait for creating `Result<Tokens>` from all sensible concrete types one could
/// return from e.g. a DefMacro closure
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

/// Create a `Result<ArgWrap>` from any concrete type that Gullet may have a reader for.
/// Used in auto-casting the data fetched by Parameter readers
pub trait IntoResultArgWrap<T>: Sized {
  /// performs the conversion
  fn into_result_argwrap(self) -> Result<ArgWrap>;
}

impl IntoResultArgWrap<Result<ArgWrap>> for Token {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Token(self)) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Option<Token> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionToken(self)) }
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

impl IntoResultArgWrap<Result<ArgWrap>> for Option<Tokens> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionTokens(self)) }
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

impl IntoResultArgWrap<Result<ArgWrap>> for RegisterValue {
  fn into_result_argwrap(self) -> Result<ArgWrap> {
    match self {
      RegisterValue::Number(n) => n.into_result_argwrap(),
      RegisterValue::Dimension(n) => n.into_result_argwrap(),
      RegisterValue::Glue(n) => n.into_result_argwrap(),
      RegisterValue::Token(n) => n.into_result_argwrap(),
      RegisterValue::Tokens(n) => n.into_result_argwrap(),
      RegisterValue::MuGlue(n) => n.into_result_argwrap(),
      RegisterValue::MuDimension(n) => n.into_result_argwrap(),
    }
  }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Result<ArgWrap> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { self }
}

impl IntoResultArgWrap<Result<ArgWrap>> for () {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::OptionTokens(None)) }
}

/// Creates `Result<bool>` from some type `T`
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

/// Creates a `Result<Vec<Digested>>` from some type `T`
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
impl IntoDigestedResult<Result<Vec<Digested>>> for Result<Tbox> {
  fn into_digested_result(self) -> Result<Vec<Digested>> { self.map(|tb| vec![tb.into()]) }
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

/// Creates an `Option<RegisterValue>` from some type `T`.
/// Useful for Register `getter` closures
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
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Number(Number(self as i64))) }
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
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(Some(Digested::from(self))) }
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
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(Some(Digested::from(self))) }
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
impl IntoFontField<Option<f64>> for f64 {
  fn into_font_field(self) -> Option<f64> { Some(self) }
}
impl IntoFontField<Option<f64>> for i32 {
  fn into_font_field(self) -> Option<f64> { Some(self as f64) }
}
