#![allow(clippy::uninlined_format_args)]
use std::borrow::Cow;
use std::fmt::{self, Display};

use crate::common::dimension::Dimension;
use crate::common::error::Result;
use crate::common::float::Float;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::definition::Digested;
use crate::keyvals::KeyVals;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::Token;
use crate::tokens::Tokens;
use crate::Locator;

#[derive(Debug, Clone)]
pub enum ArgWrap {
  Token(Token),
  OptionToken(Option<Token>),
  Tokens(Tokens),
  OptionTokens(Option<Tokens>),
  Number(Number),
  OptionNumber(Option<Number>),
  Float(Float),
  OptionFloat(Option<Float>),
  Dimension(Dimension),
  OptionDimension(Option<Dimension>),
  Glue(Glue),
  OptionGlue(Option<Glue>),
  MuGlue(MuGlue),
  OptionMuGlue(Option<MuGlue>),
  MuDimension(MuDimension),
  OptionMuDimension(Option<MuDimension>),
  KV(KeyVals),
  OptionKV(Option<KeyVals>),
  // TODO: what do we do with this custom case? feels iffy
  RegisterDefinition((Token, Vec<ArgWrap>)),
}

impl Default for ArgWrap {
  fn default() -> Self { ArgWrap::OptionTokens(None) }
}

impl Display for ArgWrap {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ArgWrap::Token(t) => write!(f, "{t}"),
      ArgWrap::OptionToken(None) => write!(f, "None"),
      ArgWrap::OptionToken(Some(ot)) => write!(f, "{ot}"),
      ArgWrap::Tokens(ts) => write!(f, "{ts}"),
      ArgWrap::OptionTokens(None) => write!(f, "None"),
      ArgWrap::OptionTokens(Some(ots)) => write!(f, "{ots}"),
      ArgWrap::Number(n) => write!(f, "{n}"),
      ArgWrap::OptionNumber(None) => write!(f, "None"),
      ArgWrap::OptionNumber(Some(on)) => write!(f, "{on}"),
      ArgWrap::Float(fl) => write!(f, "{fl}"),
      ArgWrap::OptionFloat(None) => write!(f, "None"),
      ArgWrap::OptionFloat(Some(ofl)) => write!(f, "{ofl}"),
      ArgWrap::Dimension(d) => write!(f, "{d}"),
      ArgWrap::OptionDimension(None) => write!(f, "None"),
      ArgWrap::OptionDimension(Some(od)) => write!(f, "{od}"),
      ArgWrap::Glue(gl) => write!(f, "{gl}"),
      ArgWrap::OptionGlue(None) => write!(f, "None"),
      ArgWrap::OptionGlue(Some(ogl)) => write!(f, "{ogl}"),
      ArgWrap::MuGlue(mugl) => write!(f, "{mugl}"),
      ArgWrap::OptionMuGlue(None) => write!(f, "None"),
      ArgWrap::OptionMuGlue(Some(omugl)) => write!(f, "{omugl}"),
      ArgWrap::MuDimension(mudim) => write!(f, "{mudim}"),
      ArgWrap::OptionMuDimension(None) => write!(f, "None"),
      ArgWrap::OptionMuDimension(Some(omudim)) => write!(f, "{omudim}"),
      ArgWrap::KV(kv) => write!(f, "{kv}"),
      ArgWrap::OptionKV(None) => write!(f, "None"),
      ArgWrap::OptionKV(Some(okv)) => write!(f, "{okv}"),
      ArgWrap::RegisterDefinition((t, args)) => write!(f, "({t},{args:?})"),
    }
  }
}

impl Object for ArgWrap {
  fn get_locator(&self) -> Option<Cow<Locator>> {
    use ArgWrap::*;
    match self {
      Token(t) => None,
      OptionToken(t) => None,
      Tokens(t) => None,
      OptionTokens(t) => None,
      Number(t) => None,
      OptionNumber(t) => None,
      Float(t) => None,
      OptionFloat(t) => None,
      Dimension(t) => None,
      OptionDimension(t) => None,
      Glue(t) => t.get_locator(),
      OptionGlue(g_opt) => match g_opt {
        Some(g) => g.get_locator(),
        None => None,
      },
      MuGlue(t) => t.get_locator(),
      OptionMuGlue(g_opt) => match g_opt {
        Some(g) => g.get_locator(),
        None => None,
      },
      MuDimension(t) => t.get_locator(),
      OptionMuDimension(d_opt) => match d_opt {
        Some(d) => d.get_locator(),
        None => None,
      },
      KV(kv) => kv.get_locator(),
      OptionKV(kv_opt) => match kv_opt {
        Some(kv) => kv.get_locator(),
        None => None,
      },
      RegisterDefinition(_) => None,
    }
  }
  fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
    use ArgWrap::*;
    // TODO: Should we just "do nothing" for the None cases, instead of panicking?
    match self {
      Token(t) => t.be_digested(stomach, state),
      OptionToken(t) => match t {
        Some(t) => t.be_digested(stomach, state),
        None => unimplemented!(),
      },
      Tokens(t) => t.be_digested(stomach, state),
      OptionTokens(t_opt) => match t_opt {
        Some(tks) => tks.be_digested(stomach, state),
        None => Ok(Digested::default()),
      },
      Number(t) => t.be_digested(stomach, state),
      OptionNumber(t) => match t {
        Some(n) => n.be_digested(stomach, state),
        None => unimplemented!(),
      },
      Float(t) => t.be_digested(stomach, state),
      OptionFloat(t) => match t {
        Some(fl) => fl.be_digested(stomach, state),
        None => unimplemented!(),
      },
      Dimension(t) => t.be_digested(stomach, state),
      OptionDimension(t) => match t {
        Some(t) => t.be_digested(stomach, state),
        None => unimplemented!(),
      },
      Glue(t) => t.be_digested(stomach, state),
      OptionGlue(g_opt) => match g_opt {
        Some(g) => g.be_digested(stomach, state),
        None => unimplemented!(),
      },
      MuGlue(t) => t.be_digested(stomach, state),
      OptionMuGlue(g_opt) => match g_opt {
        Some(g) => g.be_digested(stomach, state),
        None => unimplemented!(),
      },
      MuDimension(t) => t.be_digested(stomach, state),
      OptionMuDimension(d_opt) => match d_opt {
        Some(d) => d.be_digested(stomach, state),
        None => unimplemented!(),
      },
      KV(kv) => kv.be_digested(stomach, state),
      OptionKV(kv_opt) => match kv_opt {
        Some(kv) => kv.be_digested(stomach, state),
        None => unimplemented!(),
      },
      RegisterDefinition(_) => unimplemented!(), // ??? not meant for direct digestion I think
    }
  }
  fn revert(&self, state: &State) -> Result<Tokens> {
    use ArgWrap::*;
    match self {
      Token(t) => Ok(Tokens!(t.clone())),
      OptionToken(t) => unimplemented!(),
      Tokens(t) => Ok(t.clone()),
      OptionTokens(t_opt) => match t_opt {
        Some(tks) => Ok(tks.clone()),
        None => Ok(Tokens!()),
      },
      Number(t) => t.revert(state),
      OptionNumber(t) => unimplemented!(),
      Float(t) => t.revert(state),
      OptionFloat(t) => unimplemented!(),
      Dimension(t) => t.revert(state),
      OptionDimension(t) => unimplemented!(),
      Glue(t) => t.revert(state),
      OptionGlue(g_opt) => match g_opt {
        Some(g) => g.revert(state),
        None => unimplemented!(),
      },
      MuGlue(t) => t.revert(state),
      OptionMuGlue(g_opt) => match g_opt {
        Some(g) => g.revert(state),
        None => unimplemented!(),
      },
      MuDimension(t) => t.revert(state),
      OptionMuDimension(d_opt) => match d_opt {
        Some(d) => d.revert(state),
        None => unimplemented!(),
      },
      KV(kv) => kv.revert(state),
      OptionKV(kv_opt) => match kv_opt {
        Some(kv) => kv.revert(state),
        None => unimplemented!(),
      },
      RegisterDefinition(_) => unimplemented!(), // ??? not meant for direct reversion I think
    }
  }
}

impl ArgWrap {
  pub fn is_some(&self) -> bool { !self.is_none() }
  pub fn is_none(&self) -> bool {
    use ArgWrap::*;
    match self {
      Token(_) | Tokens(_) | Number(_) | Float(_) | Dimension(_) | Glue(_) | MuGlue(_) | MuDimension(_) | KV(_) => false,
      OptionToken(t) => t.is_none(),
      OptionTokens(t) => t.is_none(),
      OptionNumber(t) => t.is_none(),
      OptionFloat(t) => t.is_none(),
      OptionDimension(t) => t.is_none(),
      OptionGlue(g_opt) => g_opt.is_none(),
      OptionMuGlue(g_opt) => g_opt.is_none(),
      OptionMuDimension(d_opt) => d_opt.is_none(),
      OptionKV(kv_opt) => kv_opt.is_none(),
      RegisterDefinition(_) => false,
    }
  }
  pub fn is_tokens(&self) -> bool {
    matches!(
      self,
      ArgWrap::Tokens(_) | ArgWrap::Token(_) | ArgWrap::OptionTokens(_) | ArgWrap::OptionToken(_)
    )
  }
  pub fn mut_tokens(&mut self) -> Option<&mut Tokens> {
    match self {
      ArgWrap::Tokens(tks) => Some(tks),
      ArgWrap::OptionTokens(Some(tks)) => Some(tks),
      _ => None,
    }
  }
  pub fn owned_tokens(self) -> Option<Tokens> {
    match self {
      ArgWrap::Tokens(tks) => Some(tks),
      ArgWrap::OptionTokens(tks_opt) => tks_opt,
      ArgWrap::Token(t) => Some(Tokens::new(vec![t])),
      ArgWrap::OptionToken(t_opt) => t_opt.map(|t| Tokens::new(vec![t])),
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
      ArgWrap::OptionToken(Some(t)) => Ok(t),
      ArgWrap::Tokens(tks) => Ok(tks.unlist().remove(0)),
      _ => Err(s!("Hard assumption for Token argument failed. Got instead: {:?}", self).into()),
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
      Number(tks) => Some(Cow::Owned(self.revert(state)?)),
      Float(t) => Some(Cow::Owned(self.revert(state)?)),
      Dimension(t) => Some(Cow::Owned(self.revert(state)?)),
      Glue(t) => Some(Cow::Owned(self.revert(state)?)),
      MuGlue(t) => Some(Cow::Owned(self.revert(state)?)),
      MuDimension(t) => Some(Cow::Owned(self.revert(state)?)),
      KV(t) => Some(Cow::Owned(self.revert(state)?)),
      OptionToken(opt) => opt.as_ref().map(|t| Cow::Owned(Tokens!(t.clone()))),
      OptionTokens(opt) => opt.as_ref().map(Cow::Borrowed),
      OptionNumber(opt) => match opt {
        None => None,
        Some(t) => Some(Cow::Owned(t.revert(state)?)),
      },
      OptionFloat(opt) => match opt {
        None => None,
        Some(t) => Some(Cow::Owned(t.revert(state)?)),
      },
      OptionDimension(opt) => match opt {
        None => None,
        Some(t) => Some(Cow::Owned(t.revert(state)?)),
      },
      OptionGlue(opt) => match opt {
        None => None,
        Some(t) => Some(Cow::Owned(t.revert(state)?)),
      },
      OptionMuGlue(opt) => match opt {
        None => None,
        Some(t) => Some(Cow::Owned(t.revert(state)?)),
      },
      OptionMuDimension(opt) => match opt {
        None => None,
        Some(t) => Some(Cow::Owned(t.revert(state)?)),
      },
      OptionKV(opt) => match opt {
        None => None,
        Some(t) => Some(Cow::Owned(t.revert(state)?)),
      },
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
      _ => panic!("ArgWrap::value_of not (yet?) defined on {:?}", self),
    }
  }
  pub fn value_f32(&self) -> f32 {
    use ArgWrap::*;
    match self {
      Number(v) => v.value_f32(),
      Float(v) => v.value_f32(),
      Dimension(v) => v.value_f32(),
      Glue(v) => v.value_f32(),
      MuGlue(v) => v.value_f32(),
      MuDimension(v) => v.value_f32(),
      _ => panic!("ArgWrap::value_of not (yet?) defined on {:?}", self),
    }
  }

  pub fn try_to_number(self) -> Result<Number> {
    use ArgWrap::*;
    match self {
      Number(v) => Ok(v),
      Token(t) => Ok(t.to_number()),
      Tokens(tks) => Ok(tks.to_number()),
      OptionTokens(tks_opt) => match tks_opt {
        Some(tks) => Ok(tks.to_number()),
        // None => Err("ArgWrap::try_to_number expected a Tokens for number conversion, but got None.".into()),
        // When is the None case useful? you can see it triggered with an error in the tests.
        None => Ok(crate::common::number::Number::default()),
      },
      OptionToken(tk_opt) => match tk_opt {
        Some(tk) => Ok(tk.to_number()),
        None => Err("ArgWrap::try_to_number expected a Token for number conversion, but got None.".into()),
      },
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
      OptionTokens(tks_opt) => match tks_opt {
        Some(tks) => Ok(tks.to_keyvals(state)),
        None => Ok(KeyVals::default()),
      },
      _ => panic!("ArgWrap::to_keyvals not (yet?) defined on {:?}", self),
    }
  }

  pub fn try_to_float(self) -> Result<Float> {
    use ArgWrap::*;
    match self {
      Float(v) => Ok(v),
      Token(t) => Ok(t.to_float()),
      Tokens(tks) => Ok(tks.to_float()),
      OptionTokens(tks_opt) => match tks_opt {
        Some(tks) => Ok(tks.to_float()),
        // None => Err("ArgWrap::try_to_number expected a Tokens for number conversion, but got None.".into()),
        // When is the None case useful? you can see it triggered with an error in the tests.
        None => Ok(crate::common::float::Float::default()),
      },
      OptionToken(tk_opt) => match tk_opt {
        Some(tk) => Ok(tk.to_float()),
        None => Err("ArgWrap::try_to_float expected a Token for float conversion, but got None.".into()),
      },
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
      ArgWrap::OptionTokens(tks_opt) => match tks_opt {
        Some(tks) => tks.unlist(),
        None => Vec::new(),
      },
      ArgWrap::Token(t) => vec![t],
      ArgWrap::OptionToken(t_opt) => match t_opt {
        Some(t) => vec![t],
        None => Vec::new(),
      },
      ArgWrap::Number(n) => {
        let tks: Tokens = n.into();
        tks.unlist()
      },
      other => {
        panic!("{other:?}");
      },
    }
  }

  pub fn is_empty(&self) -> bool {
    use ArgWrap::*;
    match self {
      Token(_) | Number(_) | Float(_) | Dimension(_) | Glue(_) | MuGlue(_) | MuDimension(_) | KV(_) => false,
      Tokens(tks) => tks.is_empty(),
      OptionTokens(Some(tks)) => tks.is_empty(),
      OptionToken(None)
      | OptionTokens(None)
      | OptionNumber(None)
      | OptionFloat(None)
      | OptionDimension(None)
      | OptionGlue(None)
      | OptionMuGlue(None)
      | OptionMuDimension(None)
      | OptionKV(None) => true,
      _ => false,
    }
  }

  pub fn is_option(&self) -> bool {
    use ArgWrap::*;
    matches!(
      self,
      OptionTokens(_)
        | OptionToken(_)
        | OptionNumber(_)
        | OptionFloat(_)
        | OptionDimension(_)
        | OptionGlue(_)
        | OptionMuGlue(_)
        | OptionMuDimension(_)
        | OptionKV(_)
    )
  }
}

impl From<Token> for ArgWrap {
  fn from(t: Token) -> Self { ArgWrap::Token(t) }
}
impl From<Option<Token>> for ArgWrap {
  fn from(t: Option<Token>) -> Self { ArgWrap::OptionToken(t) }
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

impl From<Tokens> for ArgWrap {
  fn from(t: Tokens) -> Self { ArgWrap::Tokens(t) }
}
impl From<Option<Tokens>> for ArgWrap {
  fn from(t: Option<Tokens>) -> Self { ArgWrap::OptionTokens(t) }
}
impl From<Number> for ArgWrap {
  fn from(t: Number) -> Self { ArgWrap::Number(t) }
}
impl From<Option<Number>> for ArgWrap {
  fn from(t: Option<Number>) -> Self { ArgWrap::OptionNumber(t) }
}
impl From<Float> for ArgWrap {
  fn from(t: Float) -> Self { ArgWrap::Float(t) }
}
impl From<Option<Float>> for ArgWrap {
  fn from(t: Option<Float>) -> Self { ArgWrap::OptionFloat(t) }
}
impl From<Dimension> for ArgWrap {
  fn from(t: Dimension) -> Self { ArgWrap::Dimension(t) }
}
impl From<Option<Dimension>> for ArgWrap {
  fn from(t: Option<Dimension>) -> Self { ArgWrap::OptionDimension(t) }
}
impl From<MuDimension> for ArgWrap {
  fn from(t: MuDimension) -> Self { ArgWrap::MuDimension(t) }
}
impl From<Option<MuDimension>> for ArgWrap {
  fn from(t: Option<MuDimension>) -> Self { ArgWrap::OptionMuDimension(t) }
}
impl From<Glue> for ArgWrap {
  fn from(t: Glue) -> Self { ArgWrap::Glue(t) }
}
impl From<Option<Glue>> for ArgWrap {
  fn from(t: Option<Glue>) -> Self { ArgWrap::OptionGlue(t) }
}
impl From<MuGlue> for ArgWrap {
  fn from(t: MuGlue) -> Self { ArgWrap::MuGlue(t) }
}
impl From<Option<MuGlue>> for ArgWrap {
  fn from(t: Option<MuGlue>) -> Self { ArgWrap::OptionMuGlue(t) }
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
