use std::borrow::Cow;
use std::fmt::{self, Display};
use std::rc::Rc;

use crate::Locator;
use crate::alignment::template::Template;
use crate::common::dimension::Dimension;
use crate::common::error::Result;
use crate::common::float::Float;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::Digested;
use crate::definition::register::RegisterValue;
use crate::keyvals::KeyVals;
use crate::token::Token;
use crate::tokens::{NO_BORROWED_TOKENS, Tokens};

#[derive(Debug, Clone, Default)]
pub enum ArgWrap {
  Token(Token),
  Tokens(Tokens),
  Number(Number),
  Float(Float),
  Dimension(Dimension),
  Glue(Glue),
  MuGlue(MuGlue),
  MuDimension(MuDimension),
  KV(Box<KeyVals>),
  AlignmentTemplate(Box<Template>),
  Pair(crate::common::pair::Pair),
  // TODO: what do we do with this custom case? feels iffy
  RegisterDefinition(Box<(Token, Vec<ArgWrap>)>),
  #[default]
  None,
}

impl Display for ArgWrap {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ArgWrap::Token(t) => write!(f, "{t}"),
      ArgWrap::Tokens(ts) => write!(f, "{ts}"),
      ArgWrap::Number(n) => write!(f, "{n}"),
      ArgWrap::Float(fl) => write!(f, "{fl}"),
      ArgWrap::Dimension(d) => write!(f, "{d}"),
      ArgWrap::Glue(gl) => write!(f, "{gl}"),
      ArgWrap::MuGlue(mugl) => write!(f, "{mugl}"),
      ArgWrap::MuDimension(mudim) => write!(f, "{mudim}"),
      ArgWrap::KV(kv) => write!(f, "{kv}"),
      ArgWrap::AlignmentTemplate(at) => write!(f, "{at}"),
      ArgWrap::Pair(p) => write!(f, "{p}"),
      ArgWrap::RegisterDefinition(dbox) => write!(f, "({},{:?})", dbox.0, dbox.1),
      ArgWrap::None => write!(f, "None"),
    }
  }
}

impl Object for ArgWrap {
  fn get_locator(&self) -> Locator {
    use ArgWrap::*;
    match self {
      Token(_) | Tokens(_) | Number(_) | Float(_) | Dimension(_) | AlignmentTemplate(_)
      | Pair(_) => Locator::default(),
      Glue(t) => t.get_locator(),
      MuGlue(t) => t.get_locator(),
      MuDimension(t) => t.get_locator(),
      KV(kv) => kv.get_locator(),
      RegisterDefinition(_) | None => Locator::default(),
    }
  }
  fn be_digested(self) -> Result<Digested> {
    use ArgWrap::*;
    // TODO: Should we just "do nothing" for the None cases, instead of panicking?
    match self {
      Token(t) => t.be_digested(),
      Tokens(t) => t.be_digested(),
      Number(t) => t.be_digested(),
      Float(t) => t.be_digested(),
      Dimension(t) => t.be_digested(),
      Glue(t) => t.be_digested(),
      MuGlue(t) => t.be_digested(),
      MuDimension(t) => t.be_digested(),
      KV(kv) => kv.be_digested(),
      Pair(p) => p.be_digested(),
      None => Ok(Digested::default()),
      AlignmentTemplate(_) => Ok(Digested::default()), // templates don't digest directly
      RegisterDefinition(_) => Ok(Digested::default()), // register defs don't digest directly
    }
  }
  fn revert(&self) -> Result<Tokens> {
    use ArgWrap::*;
    match self {
      Token(t) => Ok(Tokens!(*t)),
      Tokens(t) => Ok(t.clone()),
      Number(t) => t.revert(),
      Float(t) => t.revert(),
      Dimension(t) => t.revert(),
      Glue(t) => t.revert(),
      MuGlue(t) => t.revert(),
      MuDimension(t) => t.revert(),
      KV(kv) => kv.revert(),
      Pair(p) => p.revert(),
      None => Ok(Tokens!()),
      AlignmentTemplate(_) => Ok(Tokens!()), // templates don't revert directly
      RegisterDefinition(_) => Ok(Tokens!()), // register defs don't revert directly
    }
  }
}

impl ArgWrap {
  pub fn is_some(&self) -> bool { !self.is_none() }
  pub fn is_none(&self) -> bool { matches!(self, ArgWrap::None) }
  pub fn is_tokens(&self) -> bool { matches!(self, ArgWrap::Tokens(_) | ArgWrap::Token(_)) }

  /// Zero-alloc equivalent of `self.to_string() == target` for keyword
  /// checks in DefMacro bodies. Forwards to `Token::with_str` /
  /// `Tokens::eq_text` for the Token/Tokens variants (covering ~all
  /// keyword-argument checks in practice); for other variants falls
  /// back to `to_string() == target` since the Display impl allocates
  /// anyway. See `Tokens::eq_text` for the walk semantics.
  pub fn eq_text(&self, target: &str) -> bool {
    match self {
      ArgWrap::Tokens(tks) => tks.eq_text(target),
      ArgWrap::Token(t) => t.with_str(|s| s == target),
      other => other.to_string() == target,
    }
  }

  /// Zero-alloc `self.to_string().starts_with(prefix)` for Tokens/Token
  /// variants; falls back to full to_string for others.
  pub fn starts_with_text(&self, prefix: &str) -> bool {
    match self {
      ArgWrap::Tokens(tks) => tks.starts_with_text(prefix),
      ArgWrap::Token(t) => t.with_str(|s| s.starts_with(prefix)),
      other => other.to_string().starts_with(prefix),
    }
  }
  pub fn mut_tokens(&mut self) -> Option<&mut Tokens> {
    match self {
      ArgWrap::Tokens(tks) => Some(tks),
      _ => None,
    }
  }
  pub fn owned_tokens(self) -> Option<Tokens> {
    match self {
      ArgWrap::Tokens(tks) => Some(tks),
      ArgWrap::Token(t) => Some(Tokens::new(vec![t])),
      ArgWrap::Number(n) => {
        let tks: Tokens = n.into();
        Some(tks)
      },
      ArgWrap::KV(kv) => kv.revert().ok(),
      _ => None,
    }
  }

  pub fn try_to_token(self) -> Result<Token> {
    match self {
      ArgWrap::Token(t) => Ok(t),
      ArgWrap::Tokens(tks) => {
        let mut list = tks.unlist();
        if list.is_empty() {
          Err("try_to_token: empty Tokens".into())
        } else {
          Ok(list.remove(0))
        }
      },
      _ => Err(
        s!(
          "Hard assumption for Token argument failed. Got instead: {:?}",
          self
        )
        .into(),
      ),
    }
  }
  pub fn expected_token(self) -> Token {
    match self.try_to_token() {
      Ok(t) => t,
      Err(e) => panic!("{e}"),
    }
  }
  pub fn undigested(self) -> Option<Digested> {
    if self.is_none() {
      None
    } else {
      Some(Digested::from(self.owned_tokens().unwrap_or_default()))
    }
  }

  /// Convert to an attribute string value (for constructor templates)
  /// Uses attribute_format for dimensions (1 decimal place) matching Perl
  pub fn to_attribute(&self) -> String {
    match self {
      ArgWrap::Dimension(d) => d.to_attribute(),
      ArgWrap::Glue(g) => g.to_attribute(),
      ArgWrap::MuGlue(mg) => mg.to_attribute(),
      ArgWrap::MuDimension(md) => md.to_attribute(),
      _ => self.to_string(),
    }
  }

  pub fn as_tokens(&self) -> Result<Option<Cow<'_, Tokens>>> {
    use ArgWrap::*;
    let result = match self {
      Token(t) => Some(Cow::Owned(Tokens!(*t))),
      Tokens(tks) => Some(Cow::Borrowed(tks)),
      Number(_) | Float(_) | Dimension(_) | Glue(_) | MuGlue(_) | MuDimension(_) | KV(_)
      | Pair(_) => Some(Cow::Owned(self.revert()?)),
      None => Some(Cow::Borrowed(NO_BORROWED_TOKENS)),
      AlignmentTemplate(_) | RegisterDefinition(_) => Some(Cow::Owned(Tokens!())),
    };
    Ok(result)
  }

  pub fn value_of(&self) -> i64 {
    use ArgWrap::*;
    match self {
      Number(v) => v.value_of(),
      Float(v) => v.value_of(),
      Dimension(v) => v.value_of(),
      Glue(v) => v.value_of(),
      MuGlue(v) => v.value_of(),
      MuDimension(v) => v.value_of(),
      None => 0,
      _ => panic!("ArgWrap::value_of not (yet?) defined on {:?}", self),
    }
  }
  pub fn value_f64(&self) -> f64 {
    use ArgWrap::*;
    match self {
      Number(v) => v.value_f64(),
      Float(v) => v.value_f64(),
      Dimension(v) => v.value_f64(),
      Glue(v) => v.value_f64(),
      MuGlue(v) => v.value_f64(),
      MuDimension(v) => v.value_f64(),
      None => 0.0,
      _ => panic!("ArgWrap::value_of not (yet?) defined on {:?}", self),
    }
  }

  pub fn try_to_number(self) -> Result<Number> {
    use ArgWrap::*;
    match self {
      Number(v) => Ok(v),
      Token(t) => Ok(t.to_number()),
      Tokens(tks) => Ok(tks.to_number()),
      None => Ok(crate::common::number::Number::new(0)),
      _ => Err(format!("ArgWrap::to_number not (yet?) defined on {:?}", self).into()),
    }
  }
  pub fn expect_number(self) -> Number {
    match self.try_to_number() {
      Ok(v) => v,
      Err(e) => panic!("{e}"),
    }
  }

  pub fn try_to_dimension(self) -> Result<Dimension> {
    use ArgWrap::*;
    match self {
      None => Ok(crate::common::dimension::Dimension::default()),
      Number(v) => Ok(v.into()),
      Dimension(v) => Ok(v),
      Token(t) => Ok(t.to_dimension()),
      Tokens(tks) => Ok(tks.to_dimension()),
      _ => Err(format!("ArgWrap::to_dimension not (yet?) defined on {:?}", self).into()),
    }
  }
  pub fn expect_dimension(self) -> Dimension {
    match self.try_to_dimension() {
      Ok(d) => d,
      Err(e) => panic!("{e}"),
    }
  }
  pub fn try_to_mu_dimension(self) -> Result<MuDimension> {
    use ArgWrap::*;
    match self {
      // Number(v) => Ok(v.into()), // ???
      MuDimension(v) => Ok(v),
      Token(t) => Ok(t.to_mu_dimension()),
      Tokens(tks) => Ok(tks.to_mu_dimension()),
      _ => Err(format!("ArgWrap::to_dimension not (yet?) defined on {:?}", self).into()),
    }
  }
  pub fn expect_mu_dimension(self) -> MuDimension {
    match self.try_to_mu_dimension() {
      Ok(d) => d,
      Err(e) => panic!("{e}"),
    }
  }

  pub fn try_to_glue(self) -> Result<Glue> {
    use ArgWrap::*;
    match self {
      Glue(v) => Ok(v),
      Number(v) => Ok(v.into()),
      Token(t) => Ok(t.to_glue()),
      Tokens(tks) => Ok(tks.to_glue()),
      _ => Err(format!("ArgWrap::try_to_glue not (yet?) defined on {:?}", self).into()),
    }
  }
  pub fn expect_glue(self) -> Glue {
    match self.try_to_glue() {
      Ok(d) => d,
      Err(e) => panic!("{e}"),
    }
  }

  pub fn try_to_mu_glue(self) -> Result<MuGlue> {
    use ArgWrap::*;
    match self {
      MuGlue(v) => Ok(v),
      Number(v) => Ok(v.into()),
      Token(t) => Ok(t.to_mu_glue()),
      Tokens(tks) => Ok(tks.to_mu_glue()),
      _ => Err(format!("ArgWrap::try_to_mu_glue not (yet?) defined on {:?}", self).into()),
    }
  }
  pub fn expect_mu_glue(self) -> MuGlue {
    match self.try_to_mu_glue() {
      Ok(d) => d,
      Err(e) => panic!("{e}"),
    }
  }
  pub fn to_mu_dimension(self) -> MuDimension {
    use ArgWrap::*;
    match self {
      MuDimension(v) => v,
      Token(t) => t.to_mu_dimension(),
      Tokens(tks) => tks.to_mu_dimension(),
      _ => panic!("ArgWrap::to_mu_dimension not (yet?) defined on {self:?}"),
    }
  }
  pub fn to_glue(self) -> Glue {
    use ArgWrap::*;
    match self {
      Glue(v) => v,
      Token(t) => t.to_glue(),
      Tokens(tks) => tks.to_glue(),
      _ => panic!("ArgWrap::to_glue not (yet?) defined on {:?}", self),
    }
  }
  pub fn to_mu_glue(self) -> MuGlue {
    use ArgWrap::*;
    match self {
      MuGlue(v) => v,
      Token(t) => t.to_mu_glue(),
      Tokens(tks) => tks.to_mu_glue(),
      _ => panic!("ArgWrap::to_mu_glue not (yet?) defined on {:?}", self),
    }
  }
  pub fn expected_keyvals(self) -> KeyVals {
    match self.try_to_keyvals() {
      Ok(t) => t,
      Err(e) => panic!("{e}"),
    }
  }

  pub fn try_to_keyvals(self) -> Result<KeyVals> {
    use ArgWrap::*;
    match self {
      KV(v) => Ok(*v),
      Tokens(tks) => tks.to_keyvals(),
      None => Ok(KeyVals::default()),
      _ => panic!("ArgWrap::to_keyvals not (yet?) defined on {:?}", self),
    }
  }

  pub fn try_to_float(self) -> Result<Float> {
    use ArgWrap::*;
    match self {
      Float(v) => Ok(v),
      Token(t) => Ok(t.to_float()),
      Tokens(tks) => Ok(tks.to_float()),
      None => Ok(crate::common::float::Float::default()),
      _ => Err(format!("ArgWrap::to_float not (yet?) defined on {:?}", self).into()),
    }
  }
  pub fn expect_float(self) -> Float {
    match self.try_to_float() {
      Ok(v) => v,
      Err(e) => panic!("{e}"),
    }
  }

  pub fn unlist(self) -> Vec<Token> {
    match self {
      ArgWrap::Tokens(tks) => tks.unlist(),
      ArgWrap::Token(t) => vec![t],
      ArgWrap::Number(n) => {
        let tks: Tokens = n.into();
        tks.unlist()
      },
      ArgWrap::None => Vec::new(),
      _ => self.revert().unwrap_or_default().unlist(),
    }
  }

  /// Borrow the token slice backing an ArgWrap::Tokens, falling back
  /// to an owned Vec<Token> for the other variants. Avoids a full
  /// Tokens clone in the common `ArgWrap::Tokens(_)` case.
  pub fn unlist_cow(&self) -> Cow<'_, [Token]> {
    match self {
      ArgWrap::Tokens(tks) => Cow::Borrowed(tks.unlist_ref()),
      ArgWrap::None => Cow::Borrowed(&[]),
      _ => Cow::Owned(self.clone().unlist()),
    }
  }

  pub fn is_empty(&self) -> bool {
    use ArgWrap::*;
    match self {
      None => true,
      Tokens(tks) => tks.is_empty(),
      _ => false,
    }
  }
}

impl From<ArgWrap> for Option<Tokens> {
  fn from(t: ArgWrap) -> Option<Tokens> { t.owned_tokens() }
}

impl From<ArgWrap> for Tokens {
  fn from(t: ArgWrap) -> Tokens {
    match t.owned_tokens() {
      Some(tks) => tks,
      None => Tokens!(),
    }
  }
}

impl<T> From<Option<T>> for ArgWrap
where T: Into<ArgWrap> + Sized
{
  fn from(t: Option<T>) -> Self {
    match t {
      Some(t) => t.into(),
      None => ArgWrap::None,
    }
  }
}

impl From<Stored> for Result<ArgWrap> {
  // A maintenance detail here is that whenever a Parameter can read a new concrete type of data,
  // then ArgWrap will have a new variant, which must then be possible to store in Stored,
  // and possible to cast back from storage into ArgWrap
  fn from(t: Stored) -> Result<ArgWrap> {
    Ok(match t {
      Stored::Tokens(t) => ArgWrap::Tokens(t),
      Stored::Token(t) => ArgWrap::Token(t),
      Stored::MuDimension(d) => ArgWrap::MuDimension(d),
      Stored::Glue(g) => ArgWrap::Glue(g),
      Stored::MuGlue(g) => ArgWrap::MuGlue(g),
      Stored::Number(n) => ArgWrap::Number(n),
      Stored::Float(f) => ArgWrap::Float(f),
      Stored::Dimension(d) => ArgWrap::Dimension(d),
      // we could just map "_" to None, but it is safer to enumerate, to avoid missing
      // meaningful cases.
      Stored::None => ArgWrap::None,
      Stored::Mouth(_)
      | Stored::Primitive(_)
      | Stored::Bool(_)
      | Stored::Parameter(_)
      | Stored::MathPrimitive(_)
      | Stored::Conditional(_)
      | Stored::Constructor(_)
      | Stored::Charcode(_)
      | Stored::Expandable(_)
      | Stored::Ligature(_)
      | Stored::HashStored(_)
      | Stored::HashTagData(_)
      | Stored::HashString(_)
      | Stored::Digested(_)
      | Stored::FontDirective(_)
      | Stored::Register(_)
      | Stored::Rewrite(_)
      | Stored::Stash(_)
      | Stored::Fontmap(_)
      | Stored::Int(_)
      | Stored::String(_)
      | Stored::Strings(_)
      | Stored::Node(_)
      | Stored::IfFrame(_)
      | Stored::Font(_)
      | Stored::Reversion(_)
      | Stored::Catcode(_)
      | Stored::Locator(_)
      | Stored::VecDequeStored(_)
      | Stored::VecDigested(_)
      | Stored::Chars(_)
      | Stored::KeyVal(_)
      | Stored::KeyVals(_)
      | Stored::Template(_) => {
        Error!(
          "stored",
          "type",
          "Failed to cast to argument; Found stored:",
          t
        );
        ArgWrap::None
      },
    })
  }
}

impl From<ArgWrap> for Result<Stored> {
  fn from(t: ArgWrap) -> Result<Stored> {
    Ok(match t {
      ArgWrap::Tokens(ts) => Stored::Tokens(ts),
      ArgWrap::Token(ts) => Stored::Token(ts),
      ArgWrap::Dimension(v) => Stored::Dimension(v),
      ArgWrap::MuDimension(v) => Stored::MuDimension(v),
      ArgWrap::Number(n) => Stored::Number(n),
      ArgWrap::Glue(v) => Stored::Glue(v),
      ArgWrap::MuGlue(v) => Stored::MuGlue(v),
      ArgWrap::Float(v) => Stored::Float(v),
      ArgWrap::None => Stored::None,
      ArgWrap::AlignmentTemplate(t) => Stored::Template(Rc::new(*t)),
      ArgWrap::Pair(_) => Stored::None, // TODO: add Stored::Pair
      ArgWrap::KV(_) | ArgWrap::RegisterDefinition(_) => {
        Error!(
          "stored",
          "type",
          "Failed to cast into Stored (no equivalent). Extend Stored if intended.",
          t
        );
        Stored::None
      },
    })
  }
}

impl From<Token> for ArgWrap {
  fn from(t: Token) -> Self { ArgWrap::Token(t) }
}

impl From<Tokens> for ArgWrap {
  fn from(t: Tokens) -> Self { ArgWrap::Tokens(t) }
}

impl From<KeyVals> for ArgWrap {
  fn from(kv: KeyVals) -> Self { ArgWrap::KV(Box::new(kv)) }
}

impl From<Number> for ArgWrap {
  fn from(t: Number) -> Self { ArgWrap::Number(t) }
}

impl From<Float> for ArgWrap {
  fn from(t: Float) -> Self { ArgWrap::Float(t) }
}

impl From<Dimension> for ArgWrap {
  fn from(t: Dimension) -> Self { ArgWrap::Dimension(t) }
}

impl From<MuDimension> for ArgWrap {
  fn from(t: MuDimension) -> Self { ArgWrap::MuDimension(t) }
}

impl From<Glue> for ArgWrap {
  fn from(t: Glue) -> Self { ArgWrap::Glue(t) }
}

impl From<MuGlue> for ArgWrap {
  fn from(t: MuGlue) -> Self { ArgWrap::MuGlue(t) }
}

impl From<()> for ArgWrap {
  fn from(_: ()) -> Self { ArgWrap::default() }
}
impl From<RegisterValue> for ArgWrap {
  fn from(t: RegisterValue) -> Self {
    match t {
      RegisterValue::Number(n) => ArgWrap::Number(n),
      RegisterValue::Dimension(n) => ArgWrap::Dimension(n),
      RegisterValue::Glue(n) => ArgWrap::Glue(n),
      RegisterValue::Token(n) => ArgWrap::Token(n),
      RegisterValue::Tokens(n) => ArgWrap::Tokens(n),
      RegisterValue::MuGlue(n) => ArgWrap::MuGlue(n),
      RegisterValue::MuDimension(n) => ArgWrap::MuDimension(n),
      RegisterValue::Pair(p) => ArgWrap::Pair(p),
    }
  }
}
impl From<Template> for ArgWrap {
  fn from(t: Template) -> Self { ArgWrap::AlignmentTemplate(Box::new(t)) }
}

impl TryFrom<ArgWrap> for Number {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Number> { aw.try_to_number() }
}
impl TryFrom<ArgWrap> for Dimension {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Dimension> { aw.try_to_dimension() }
}
impl TryFrom<ArgWrap> for MuDimension {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<MuDimension> { aw.try_to_mu_dimension() }
}
impl TryFrom<ArgWrap> for Glue {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Glue> { aw.try_to_glue() }
}
impl TryFrom<ArgWrap> for MuGlue {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<MuGlue> { aw.try_to_mu_glue() }
}
impl TryFrom<ArgWrap> for Float {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Float> { aw.try_to_float() }
}

impl TryFrom<ArgWrap> for Token {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Token> { aw.try_to_token() }
}

impl TryFrom<ArgWrap> for KeyVals {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<KeyVals> { aw.try_to_keyvals() }
}

impl From<ArgWrap> for Option<KeyVals> {
  fn from(aw: ArgWrap) -> Option<KeyVals> {
    match aw {
      ArgWrap::KV(kv) => Some(*kv),
      _ => None,
    }
  }
}

impl From<ArgWrap> for Template {
  fn from(aw: ArgWrap) -> Template {
    match aw {
      ArgWrap::AlignmentTemplate(t) => *t,
      other => panic!("illegal auto-cast to alignment::Template on {other:?}"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn argwrap_default_is_none() {
    let a = ArgWrap::default();
    assert!(a.is_none());
    assert!(!a.is_some());
  }

  #[test]
  fn argwrap_number_is_some() {
    let a = ArgWrap::Number(Number::new(42));
    assert!(a.is_some());
    assert!(!a.is_none());
  }

  #[test]
  fn argwrap_is_tokens_for_token_and_tokens() {
    let a = ArgWrap::Tokens(Tokens::new(vec![]));
    assert!(a.is_tokens());
    let b = ArgWrap::Token(Token::default());
    assert!(
      b.is_tokens(),
      "is_tokens is true for both Token and Tokens variants"
    );
    let c = ArgWrap::Number(Number::new(0));
    assert!(!c.is_tokens());
    let d = ArgWrap::None;
    assert!(!d.is_tokens());
  }

  #[test]
  fn argwrap_value_of_number() {
    let a = ArgWrap::Number(Number::new(42));
    assert_eq!(a.value_of(), 42);
  }

  #[test]
  fn argwrap_value_of_dimension() {
    let a = ArgWrap::Dimension(Dimension::new(65536));
    assert_eq!(a.value_of(), 65536);
  }

  #[test]
  fn argwrap_value_f64_float() {
    let a = ArgWrap::Float(Float(3.14));
    assert!((a.value_f64() - 3.14).abs() < 1e-6);
  }

  #[test]
  #[allow(non_snake_case)]
  fn argwrap_display_none_is_the_word_None() {
    // Discovered: ArgWrap::None's Display writes "None" (the variant
    // name), not empty string. Capital-N in fn name preserves the
    // literal distinction — don't "fix" to lowercase.
    let a = ArgWrap::None;
    assert_eq!(format!("{a}"), "None");
  }

  #[test]
  fn argwrap_display_number() {
    let a = ArgWrap::Number(Number::new(42));
    assert_eq!(format!("{a}"), "42");
  }

  #[test]
  fn argwrap_try_to_number_from_number() {
    let a = ArgWrap::Number(Number::new(42));
    let n = a.try_to_number().unwrap();
    assert_eq!(n.value_of(), 42);
  }

  #[test]
  fn argwrap_expect_number_from_number() {
    let a = ArgWrap::Number(Number::new(42));
    assert_eq!(a.expect_number().value_of(), 42);
  }

  #[test]
  fn argwrap_owned_tokens_from_none_is_none() {
    let a = ArgWrap::None;
    assert!(a.owned_tokens().is_none());
  }
}
