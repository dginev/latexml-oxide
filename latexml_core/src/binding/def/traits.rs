//! A variety of traits helpful for auto-casting between the different components of the
//! conversion toolchain
use std::collections::VecDeque;

use crate::common::arena;
use crate::common::arena::SymHashMap as HashMap;
use crate::common::color::Color;
use crate::common::error::*;
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
use crate::state::{Scope, lookup_font};
use crate::token::*;
use crate::whatsit::Whatsit;
use crate::*;

/// Build sizing options from a Whatsit's properties, matching Perl's computeSizeStore behavior.
/// Perl (Box.pm L267-271) adds width, height, depth, vattach, layout from properties to options
/// before calling computeSize, which passes them through to computeBoxesSize.
fn sizer_options_from_whatsit(w: &Whatsit) -> HashMap<Stored> {
  let mut options: HashMap<Stored> = HashMap::default();
  for key in ["width", "height", "depth", "vattach", "layout", "mode"] {
    if let Some(v) = w.get_property(key) {
      options.insert(key, v.into_owned());
    }
  }
  options
}

/// Helper for sizer string parsing: references either a numeric arg or a named property
enum SizerRef {
  Arg(usize),
  Prop(String),
}

/// A trait for auto-wrapping a generic type `T` into `Option<Y>`,
/// where Y can be inferred from context.
/// (useful in macro helpers, such as `NewDefaultV!`)
pub trait IntoOption<T>: Sized {
  /// Performs the conversion.
  fn into_option(self) -> T;
}

impl IntoOption<Option<String>> for &str {
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
      Some(Reversion::Tokens(
        mouth::tokenize_internal(self)
          .pack_parameters()
          .ok()
          .unwrap(),
      ))
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
      other => Some(Scope::Named(arena::pin(other))),
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
      _ => Some(Scope::Named(arena::pin(self))),
    }
  }
}

// TODO: Sizers need a lot more work, likely a complete rethink about organization.
impl IntoOption<Option<SizingClosure>> for i64 {
  fn into_option(self) -> Option<SizingClosure> {
    Some(Rc::new(move |_| {
      Ok((
        Dimension::new(self),
        Dimension::new(self),
        Dimension::new(self),
      ))
    }))
  }
}
impl IntoOption<Option<SizingClosure>> for &str {
  fn into_option(self) -> Option<SizingClosure> {
    if self.is_empty() {
      None
    } else if self == "0" {
      Some(Rc::new(|_| {
        Ok((
          Dimension::default(),
          Dimension::default(),
          Dimension::default(),
        ))
      }))
    } else if self.starts_with('#') {
      // Perl: /^(#\w+)*$/ — parse each #token as either numeric arg or property name
      // e.g. "#3" → getArg(3), "#alignment" → props{alignment}, "#1#2" → both combined
      let mut refs: Vec<SizerRef> = Vec::new();
      let mut rest = self;
      while let Some(stripped) = rest.strip_prefix('#') {
        let end = stripped.find('#').unwrap_or(stripped.len());
        let name = &stripped[..end];
        if let Ok(n) = name.parse::<usize>() {
          refs.push(SizerRef::Arg(n));
        } else {
          refs.push(SizerRef::Prop(name.to_string()));
        }
        rest = &stripped[end..];
      }
      Some(Rc::new(move |w| {
        let mut boxes: Vec<Digested> = Vec::with_capacity(refs.len());
        for r in &refs {
          match r {
            SizerRef::Arg(n) => {
              if let Some(arg) = w.get_arg(*n) {
                boxes.push(arg.clone());
              }
            },
            SizerRef::Prop(name) => {
              if let Some(Stored::Digested(d)) = w.get_property(name).as_deref() {
                boxes.push(d.clone());
              }
            },
          }
        }
        if boxes.len() == 1 {
          // Perl: computeBoxesSize($boxes[0], %options) — pass whatsit properties as options
          // so vattach, width, etc. propagate to compute_boxes_size
          let options = sizer_options_from_whatsit(w);
          boxes[0].compute_size(options)
        } else if boxes.is_empty() {
          Ok((
            Dimension::default(),
            Dimension::default(),
            Dimension::default(),
          ))
        } else {
          let font = match w.get_property("font").as_deref() { Some(Stored::Font(font)) => {
            font.clone()
          } _ => {
            lookup_font().unwrap()
          }};
          let options = sizer_options_from_whatsit(w);
          font.compute_boxes_size(&boxes, options)
        }
      }))
    } else {
      // literal string, get its size with the current font?
      let sized_data = String::from(self);
      Some(Rc::new(move |w| {
        let font = match *w.get_property("font").unwrap() { Stored::Font(ref font) => {
          font.clone()
        } _ => {
          lookup_font().unwrap()
        }};
        let options = sizer_options_from_whatsit(w);
        font.compute_boxes_size(
          &[Digested::from(Tbox {
            text: arena::pin(&sized_data),
            ..Tbox::default()
          })],
          options,
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

impl IntoTokensResult<Result<Tokens>> for Result<()> {
  fn into_tokens_result(self) -> Result<Tokens> {
    match self {
      Ok(()) => Ok(Tokens!()),
      Err(e) => Err(e),
    }
  }
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
  fn into_tokens_result(self) -> Result<Tokens> {
    self.map(|w| w.owned_tokens().unwrap_or_default())
  }
}

/// Create a `Result<ArgWrap>` from any concrete type that Gullet may have a reader for.
/// Used in auto-casting the data fetched by Parameter readers
pub trait IntoResultArgWrap<T>: Sized {
  /// performs the conversion
  fn into_result_argwrap(self) -> Result<ArgWrap>;
}

impl IntoResultArgWrap<Result<ArgWrap>> for crate::common::error::Error {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Err(self) }
}

impl<T> IntoResultArgWrap<Result<ArgWrap>> for Result<T>
where T: Into<ArgWrap> + Sized
{
  fn into_result_argwrap(self) -> Result<ArgWrap> { self.map(|v| v.into()) }
}

impl<T> IntoResultArgWrap<Result<ArgWrap>> for T
where T: Into<ArgWrap> + Sized
{
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(self.into()) }
}

impl IntoResultArgWrap<Result<ArgWrap>> for Vec<Token> {
  fn into_result_argwrap(self) -> Result<ArgWrap> { Ok(ArgWrap::Tokens(Tokens::new(self))) }
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
impl IntoDigestedResult<Result<Vec<Digested>>> for crate::common::error::Error {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Err(self) }
}
impl IntoDigestedResult<Result<Vec<Digested>>> for Result<()> {
  fn into_digested_result(self) -> Result<Vec<Digested>> { self.map(|_| Vec::new()) }
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
  fn into_register_value_option(self) -> Option<RegisterValue> {
    Some(RegisterValue::Number(Number(self as i64)))
  }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Number {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Number(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Dimension {
  fn into_register_value_option(self) -> Option<RegisterValue> {
    Some(RegisterValue::Dimension(self))
  }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for MuDimension {
  fn into_register_value_option(self) -> Option<RegisterValue> {
    Some(RegisterValue::MuDimension(self))
  }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Glue {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Glue(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for MuGlue {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::MuGlue(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Token {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Token(self)) }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Option<Token> {
  fn into_register_value_option(self) -> Option<RegisterValue> { self.map(RegisterValue::Token) }
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

impl IntoDigestedOptionResult<Result<Option<Digested>>> for () {
  fn into_digested_option_result(self: ()) -> Result<Option<Digested>> { Ok(None) }
}

impl IntoDigestedOptionResult<Result<Option<Digested>>> for Glue {
  fn into_digested_option_result(self) -> Result<Option<Digested>> {
    RegisterValue::Glue(self).into_digested_option_result()
  }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for MuGlue {
  fn into_digested_option_result(self) -> Result<Option<Digested>> {
    RegisterValue::MuGlue(self).into_digested_option_result()
  }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for Dimension {
  fn into_digested_option_result(self) -> Result<Option<Digested>> {
    RegisterValue::Dimension(self).into_digested_option_result()
  }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for MuDimension {
  fn into_digested_option_result(self) -> Result<Option<Digested>> {
    RegisterValue::MuDimension(self).into_digested_option_result()
  }
}

impl IntoDigestedOptionResult<Result<Option<Digested>>> for Number {
  fn into_digested_option_result(self) -> Result<Option<Digested>> {
    RegisterValue::Number(self).into_digested_option_result()
  }
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
  fn into_digested_option_result(self) -> Result<Option<Digested>> {
    Ok(Some(Digested::from(self)))
  }
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
  fn into_digested_option_result(self) -> Result<Option<Digested>> {
    Ok(Some(Digested::from(self)))
  }
}

pub trait IntoPropertiesResult {
  fn into_properties_result(self) -> Result<HashMap<Stored>>;
}
impl IntoPropertiesResult for HashMap<Stored> {
  fn into_properties_result(self) -> Result<HashMap<Stored>> { Ok(self) }
}
impl IntoPropertiesResult for Result<HashMap<Stored>> {
  fn into_properties_result(self) -> Result<HashMap<Stored>> { self }
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
impl IntoFontField<Option<Color>> for Color {
  fn into_font_field(self) -> Option<Color> { Some(self) }
}
