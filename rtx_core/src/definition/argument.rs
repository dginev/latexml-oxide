use std::borrow::Cow;
use std::fmt::{self, Display};

use crate::state::State;
use crate::Locator;
use crate::token::Token;
use crate::tokens::Tokens;
use crate::stomach::Stomach;
use crate::definition::Digested;
use crate::common::error::Result;
use crate::common::numeric_ops::NumericOps;
use crate::common::number::Number;
use crate::common::float::Float;
use crate::common::dimension::Dimension;
use crate::common::mudimension::MuDimension;
use crate::common::glue::Glue;
use crate::common::muglue::MuGlue;
use crate::common::object::Object;
use crate::keyvals::KeyVals;

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
  OptionKV(Option<KeyVals>)
}

impl Default for ArgWrap {
  fn default() -> Self {
    ArgWrap::OptionTokens(None)
  }
}

impl Display for ArgWrap {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
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
        None => None },
      MuGlue(t) => t.get_locator(),
      OptionMuGlue(g_opt) => match g_opt {
        Some(g) => g.get_locator(),
        None => None },
      MuDimension(t) => t.get_locator(),
      OptionMuDimension(d_opt) => match d_opt {
        Some(d) => d.get_locator(),
        None => None },
      KV(kv) => kv.get_locator(),
      OptionKV(kv_opt) => match kv_opt {
        Some(kv) => kv.get_locator(),
        None => None },
    }
  }
  fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
    use ArgWrap::*;
    match self {
      Token(t) => t.be_digested(stomach,state),
      OptionToken(t) => unimplemented!(),
      Tokens(t) => t.be_digested(stomach,state),
      OptionTokens(t_opt) => match t_opt {
        Some(tks) => tks.be_digested(stomach,state),
        None => Ok(Digested::default()),
      },
      Number(t) => t.be_digested(stomach,state),
      OptionNumber(t) => unimplemented!(),
      Float(t) => t.be_digested(stomach,state),
      OptionFloat(t) => unimplemented!(),
      Dimension(t) => t.be_digested(stomach,state),
      OptionDimension(t) => unimplemented!(),
      Glue(t) => t.be_digested(stomach,state),
      OptionGlue(g_opt) => match g_opt {
        Some(g) => g.be_digested(stomach,state),
        None => unimplemented!() },
      MuGlue(t) => t.be_digested(stomach,state),
      OptionMuGlue(g_opt) => match g_opt {
        Some(g) => g.be_digested(stomach,state),
        None => unimplemented!()
      },
      MuDimension(t) => t.be_digested(stomach,state),
      OptionMuDimension(d_opt) => match d_opt {
        Some(d) => d.be_digested(stomach,state),
        None => unimplemented!() },
      KV(kv) => kv.be_digested(stomach,state),
      OptionKV(kv_opt) => match kv_opt {
        Some(kv) => kv.be_digested(stomach,state),
        None => unimplemented!()
      },
    }
  }
}

impl ArgWrap {
  pub fn is_some(&self) -> bool {
    !self.is_none()
  }
  pub fn is_none(&self) -> bool {
    use ArgWrap::*;
    match self {
      Token(_) | Tokens(_) | Number(_) | Float(_) |
      Dimension(_) | Glue(_) | MuGlue(_) | MuDimension(_) | KV(_) => false,
      OptionToken(t) => t.is_none(),
      OptionTokens(t) => t.is_none(),
      OptionNumber(t) => t.is_none(),
      OptionFloat(t) => t.is_none(),
      OptionDimension(t) => t.is_none(),
      OptionGlue(g_opt) => g_opt.is_none(),
      OptionMuGlue(g_opt) => g_opt.is_none(),
      OptionMuDimension(d_opt) => d_opt.is_none(),
      OptionKV(kv_opt) => kv_opt.is_none()
    }
  }
  pub fn is_tokens(&self) -> bool {
    matches!(self, ArgWrap::Tokens(_) | ArgWrap::Token(_) | ArgWrap::OptionTokens(_) | ArgWrap::OptionToken(_))
  }
  pub fn mut_tokens(&mut self) -> Option<&mut Tokens> {
    match self {
      ArgWrap::Tokens(tks) => Some(tks),
      ArgWrap::OptionTokens(Some(tks)) => Some(tks),
      _ => None
    }
  }
  pub fn owned_tokens(self) -> Option<Tokens> {
    match self {
      ArgWrap::Tokens(tks) => Some(tks),
      ArgWrap::OptionTokens(tks_opt) => tks_opt,
      ArgWrap::Token(t) => Some(Tokens::new(vec![t])),
      ArgWrap::OptionToken(t_opt) => t_opt.map(|t| Tokens::new(vec![t])),
      _ => None
    }
  }

  pub fn expected_token(self) -> Result<Token> {
    match self {
      ArgWrap::Token(t) => Ok(t),
      ArgWrap::OptionToken(Some(t)) => Ok(t),
      ArgWrap::Tokens(tks) => Ok(tks.unlist().remove(0)),
      _ => Err(s!("Hard assumption for Token argument failed. Got instead: {:?}", self).into())
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
    };
    Ok(result)
  }

  pub fn value_of(&self) -> i32 {
    use ArgWrap::*;
    match self {
      Number(v) => v.value_of(),
      Float(v) => v.value_of(),
      Dimension(v) => v.value_of(),
      Glue(v) => v.value_of(),
      MuGlue(v) => v.value_of(),
      MuDimension(v) => v.value_of(),
      _ => panic!("ArgWrap::value_of not (yet?) defined on {:?}",self)
    }
  }

  pub fn to_number(self) -> Number {
    use ArgWrap::*;
    match self {
      Number(v) => v,
      Token(t) => t.to_number(),
      Tokens(tks) => tks.to_number(),
      _ => panic!("ArgWrap::to_number not (yet?) defined on {:?}",self)
    }
  }

  pub fn to_dimension(self) -> Dimension {
    use ArgWrap::*;
    match self {
      Dimension(v) => v,
      Token(t) => t.to_dimension(),
      Tokens(tks) => tks.to_dimension(),
      _ => panic!("ArgWrap::to_dimension not (yet?) defined on {:?}",self)
    }
  }
  pub fn to_mu_dimension(self) -> MuDimension {
    use ArgWrap::*;
    match self {
      MuDimension(v) => v,
      Token(t) => t.to_mu_dimension(),
      Tokens(tks) => tks.to_mu_dimension(),
      _ => panic!("ArgWrap::to_mu_dimension not (yet?) defined on {:?}",self)
    }
  }
  pub fn to_glue(self) -> Glue {
    use ArgWrap::*;
    match self {
      Glue(v) => v,
      Token(t) => t.to_glue(),
      Tokens(tks) => tks.to_glue(),
      _ => panic!("ArgWrap::to_glue not (yet?) defined on {:?}",self)
    }
  }
  pub fn to_mu_glue(self) -> MuGlue {
    use ArgWrap::*;
    match self {
      MuGlue(v) => v,
      Token(t) => t.to_mu_glue(),
      Tokens(tks) => tks.to_mu_glue(),
      _ => panic!("ArgWrap::to_glue not (yet?) defined on {:?}",self)
    }
  }
  pub fn to_keyvals(self, state: &mut State) -> KeyVals {
    use ArgWrap::*;
    match self {
      KV(v) => v,
      Tokens(tks) => tks.to_keyvals(state),
      _ => panic!("ArgWrap::to_keyvals not (yet?) defined on {:?}",self)
    }
  }

  pub fn unlist(self) -> Vec<Token> {
    match self {
      ArgWrap::Tokens(tks) => tks.unlist(),
      ArgWrap::Token(t) => vec![t],
      _ => unimplemented!()
    }
  }

  pub fn is_empty(&self) -> bool {
    use ArgWrap::*;
    match self {
      Token(_) | Number(_) | Float(_) | Dimension(_) | Glue(_) | MuGlue(_) | MuDimension(_) | KV(_) => false,
      Tokens(tks) => tks.is_empty(),
      OptionTokens(Some(tks)) => tks.is_empty(),
      OptionToken(None) | OptionTokens(None) | OptionNumber(None) | OptionFloat(None) |
      OptionDimension(None) | OptionGlue(None) | OptionMuGlue(None) | OptionMuDimension(None) |
      OptionKV(None) => true,
      _ => false,
    }
  }
}

impl From<Token> for ArgWrap {
  fn from(t: Token) -> Self {
    ArgWrap::Token(t)
  }
}
impl From<Option<Token>> for ArgWrap {
  fn from(t: Option<Token>) -> Self {
    ArgWrap::OptionToken(t)
  }
}
impl From<ArgWrap> for Option<Tokens> {
  fn from(t: ArgWrap) -> Option<Tokens> {
    t.owned_tokens()
  }
}
impl From<Tokens> for ArgWrap {
  fn from(t: Tokens) -> Self {
    ArgWrap::Tokens(t)
  }
}
impl From<Option<Tokens>> for ArgWrap {
  fn from(t: Option<Tokens>) -> Self {
    ArgWrap::OptionTokens(t)
  }
}
impl From<Number> for ArgWrap {
  fn from(t: Number) -> Self {
    ArgWrap::Number(t)
  }
}
impl From<Option<Number>> for ArgWrap {
  fn from(t: Option<Number>) -> Self {
    ArgWrap::OptionNumber(t)
  }
}
impl From<Float> for ArgWrap {
  fn from(t: Float) -> Self {
    ArgWrap::Float(t)
  }
}
impl From<Option<Float>> for ArgWrap {
  fn from(t: Option<Float>) -> Self {
    ArgWrap::OptionFloat(t)
  }
}
impl From<Dimension> for ArgWrap {
  fn from(t: Dimension) -> Self {
    ArgWrap::Dimension(t)
  }
}
impl From<Option<Dimension>> for ArgWrap {
  fn from(t: Option<Dimension>) -> Self {
    ArgWrap::OptionDimension(t)
  }
}
impl From<MuDimension> for ArgWrap {
  fn from(t: MuDimension) -> Self {
    ArgWrap::MuDimension(t)
  }
}
impl From<Option<MuDimension>> for ArgWrap {
  fn from(t: Option<MuDimension>) -> Self {
    ArgWrap::OptionMuDimension(t)
  }
}
impl From<Glue> for ArgWrap {
  fn from(t: Glue) -> Self {
    ArgWrap::Glue(t)
  }
}
impl From<Option<Glue>> for ArgWrap {
  fn from(t: Option<Glue>) -> Self {
    ArgWrap::OptionGlue(t)
  }
}
impl From<MuGlue> for ArgWrap {
  fn from(t: MuGlue) -> Self {
    ArgWrap::MuGlue(t)
  }
}
impl From<Option<MuGlue>> for ArgWrap {
  fn from(t: Option<MuGlue>) -> Self {
    ArgWrap::OptionMuGlue(t)
  }
}
