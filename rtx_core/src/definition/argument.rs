use std::borrow::Cow;
use std::fmt::{self, Display};

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
use crate::definition::register::RegisterValue;
use crate::definition::Digested;
use crate::keyvals::KeyVals;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::Token;
use crate::tokens::{Tokens,NO_BORROWED_TOKENS};
use crate::Locator;

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
  KV(KeyVals),
  AlignmentTemplate(Template),
  // TODO: what do we do with this custom case? feels iffy
  RegisterDefinition((Token, Vec<ArgWrap>)),
  #[default]
  None
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
      ArgWrap::RegisterDefinition((t, args)) => write!(f, "({t},{args:?})"),
      ArgWrap::None => write!(f, "None")
    }
  }
}

impl Object for ArgWrap {
  fn get_locator(&self) -> Option<Cow<Locator>> {
    use ArgWrap::*;
    match self {
      Token(_) | Tokens(_)  | Number(_) | Float(_)  | Dimension(_)  | AlignmentTemplate(_) => {
        Option::None
      },
      Glue(t) => t.get_locator(),
      MuGlue(t) => t.get_locator(),
      MuDimension(t) => t.get_locator(),
      KV(kv) => kv.get_locator(),
      RegisterDefinition(_) | None => Option::None,
    }
  }
  fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
    use ArgWrap::*;
    // TODO: Should we just "do nothing" for the None cases, instead of panicking?
    match self {
      Token(t) => t.be_digested(stomach, state),
      Tokens(t) => t.be_digested(stomach, state),
      Number(t) => t.be_digested(stomach, state),
      Float(t) => t.be_digested(stomach, state),
      Dimension(t) => t.be_digested(stomach, state),
      Glue(t) => t.be_digested(stomach, state),
      MuGlue(t) => t.be_digested(stomach, state),
      MuDimension(t) => t.be_digested(stomach, state),
      KV(kv) => kv.be_digested(stomach, state),
      None => Ok(Digested::default()),
      AlignmentTemplate(_) => unimplemented!(),
      RegisterDefinition(_) => unimplemented!(), // ??? not meant for direct digestion I think
    }
  }
  fn revert(&self, state: &State) -> Result<Tokens> {
    use ArgWrap::*;
    match self {
      Token(t) => Ok(Tokens!(t.clone())),
      Tokens(t) => Ok(t.clone()),
      Number(t) => t.revert(state),
      Float(t) => t.revert(state),
      Dimension(t) => t.revert(state),
      Glue(t) => t.revert(state),
      MuGlue(t) => t.revert(state),
      MuDimension(t) => t.revert(state),
      KV(kv) => kv.revert(state),
      None => Ok(Tokens!()),
      AlignmentTemplate(_) => unimplemented!(),
      RegisterDefinition(_) => unimplemented!(), // ??? not meant for direct reversion I think
    }
  }
}

impl ArgWrap {
  pub fn is_some(&self) -> bool { !self.is_none() }
  pub fn is_none(&self) -> bool {
    matches!(self, ArgWrap::None)
  }
  pub fn is_tokens(&self) -> bool {
    matches!(
      self,
      ArgWrap::Tokens(_) | ArgWrap::Token(_)
    )
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
      _ => None,
    }
  }

  pub fn try_to_token(self) -> Result<Token> {
    match self {
      ArgWrap::Token(t) => Ok(t),
      ArgWrap::Tokens(tks) => Ok(tks.unlist().remove(0)),
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

  pub fn as_tokens<'a>(&'a self, state: &mut State) -> Result<Option<Cow<'a, Tokens>>> {
    use ArgWrap::*;
    let result = match self {
      Token(t) => Some(Cow::Owned(Tokens!(t.clone()))), // ? avoid the clone ?
      Tokens(tks) => Some(Cow::Borrowed(tks)),
      Number(_) | Float(_) | Dimension(_) | Glue(_) | MuGlue(_) | MuDimension(_) | KV(_) => {
        Some(Cow::Owned(self.revert(state)?))
      },
      None => Some(Cow::Borrowed(NO_BORROWED_TOKENS)),
      AlignmentTemplate(_) => unimplemented!(),
      RegisterDefinition(_) => unimplemented!(), // ??? not meant for such use
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
  pub fn expected_keyvals(self, state: &mut State) -> KeyVals {
    match self.try_to_keyvals(state) {
      Ok(t) => t,
      Err(e) => panic!("{e}"),
    }
  }

  pub fn try_to_keyvals(self, state: &mut State) -> Result<KeyVals> {
    use ArgWrap::*;
    match self {
      KV(v) => Ok(v),
      Tokens(tks) => Ok(tks.to_keyvals(state)),
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
      _ => unimplemented!(),
    }
  }

  pub fn is_empty(&self) -> bool {
    use ArgWrap::*;
    match self {
      None => true,
      Tokens(tks) => tks.is_empty(),
      _ => false
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

impl<T> From<Option<T>> for ArgWrap where
  T: Into<ArgWrap> + Sized {
  fn from(t: Option<T>) -> Self {
    match t {
      Some(t) => t.into(),
      None => ArgWrap::None
    }
  }
}

impl From<Token> for ArgWrap {
  fn from(t: Token) -> Self { ArgWrap::Token(t) }
}

impl From<Tokens> for ArgWrap {
  fn from(t: Tokens) -> Self { ArgWrap::Tokens(t) }
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
    }
  }
}
impl From<Template> for ArgWrap {
  fn from(t: Template) -> Self { ArgWrap::AlignmentTemplate(t) }
}

impl TryFrom<ArgWrap> for Number {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Number> { aw.try_to_number() }
}
impl TryFrom<ArgWrap> for Dimension {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Dimension> { aw.try_to_dimension() }
}
impl TryFrom<ArgWrap> for Float {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Float> { aw.try_to_float() }
}

impl TryFrom<ArgWrap> for Token {
  type Error = crate::common::error::Error;
  fn try_from(aw: ArgWrap) -> Result<Token> { aw.try_to_token() }
}

// impl TryFrom<ArgWrap> for KeyVals {
//   type Error = crate::common::error::Error;
//   fn try_from(aw: ArgWrap) -> Result<KeyVals> { aw.try_to_keyvals() }
// }
