use std::borrow::Cow;
use std::fmt;

use crate::common::error::*;
use crate::common::font::Font;
use crate::document::Document;
use crate::state::State;

use crate::token::Token;
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, TexMode};

/// Lists can contain any Digested items, such as boxes, whatsits or other lists
#[derive(Clone, PartialEq)]
pub struct List {
  pub boxes: Vec<Digested>,
  pub mode: Option<TexMode>,
  pub font: Option<Font>,
}

impl fmt::Debug for List {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\nList[")?;
    for tbox in &self.boxes {
      writeln!(f, "  {:?}", tbox)?;
    }
    writeln!(f, "]({:?})", self.mode)
  }
}

impl BoxOps for List {
  fn unlist(&self) -> Vec<Digested> { self.boxes.clone() }

  fn to_string(&self) -> String {
    self
      .boxes
      .iter()
      .fold(String::new(), |joined, x| joined + &x.to_string())
  }

  /// NOTE: No longer used; Document->absorb bypasses this for stack efficiency.
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    unimplemented!()
    // for digested in self.unlist() {
    //   document.absorb(digested, state)?;
    // }
    // Ok(())
  }

  fn revert(&self) -> Tokens {
    let reverted = self
      .boxes
      .iter()
      .flat_map(|tbox| tbox.revert().unlist())
      .collect::<Vec<Token>>();
    Tokens::new(reverted)
  }

  fn get_font(&self) -> Option<Cow<Font>> {
    match self.font {
      None => None,
      Some(ref f) => Some(Cow::Borrowed(&f)),
    }
  }
}

impl List {
  pub fn new(boxes: Vec<Digested>) -> Self {
    // while (defined($bx = shift(@bxs)) && (!defined $locator)) {
    //   $locator = $bx->getLocator unless defined $locator; }

    // Maybe the most representative font for a List is the font of the LAST box (that _has_ a
    // font!) ???
    let mut font: Option<Font> = None;
    for bx in boxes.iter().rev() {
      if let Some(bx_font) = bx.get_font() {
        font = Some(bx_font.into_owned());
        break;
      }
    }
    List {
      boxes,
      font,
      mode: None,
    }
  }
}

impl From<List> for Result<Vec<Digested>> {
  fn from(list: List) -> Result<Vec<Digested>> { Ok(list.boxes) }
}

impl From<List> for Result<Digested> {
  fn from(list: List) -> Result<Digested> { Ok(Digested::List(Box::new(list))) }
}
