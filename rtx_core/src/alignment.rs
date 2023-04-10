//! An "Alignment" is an array/tabular construct as:
//!   <tabular><tr><td>...
//! or, for math mode
//!   <XMArray><XMRow><XMCell>...
//! (where initially, each XMCell will contain an XMArg to indicate
//! individual parsing of each cell's content is desired)
//!
//! An Alignment object is a sort of fake Whatsit;
//! It takes some magic to sneak it into the Digestion stream
//! (see TeX.pool \@start@alignment), but it needs to be created
//! BEFORE the contents of the alignment are digested,
//! since we stuff a lot of information into it
//! (row, column boxes, borders, spacing, etc...)
//! But once it has been captured, it should otherwise act
//! like a Whatsit and be responsible for construction (beAbsorbed),
//! and sizing estimation (computeSize)
//!
//! Ultimately, this should be better tied into DefConstructor
//! because an Alignment currently doesn't know what CS created it (debugging!);
//! Also, it would better connect the things being constructed, reversion, etc.

use crate::token::{Catcode,Token};
use crate::tokens::Tokens;
use crate::common::error::*;
use crate::definition::argument::ArgWrap;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::state::State;
use crate::document::Document;
use crate::common::object::Object;

use rustc_hash::FxHashMap as HashMap;
use std::sync::Arc;
use std::fmt::{self,Display};

//DebuggableFeature('alignment', "Debug guessing headers of alignments/tables");
pub type OpenContainerFn = Arc<dyn Fn(&mut Document, HashMap<String,String>, &mut State) -> Result<()>>;
pub type CloseContainerFn = Arc<dyn Fn(&mut Document, &mut State) -> Result<()>>;
pub type OpenRowFn = Arc<dyn Fn(&mut Document, HashMap<String,String>, &mut State) -> Result<()>>;
pub type CloseRowFn = Arc<dyn Fn(&mut Document, &mut State) -> Result<()>>;
pub type CloseColumnFn = Arc<dyn Fn(&mut Document, &mut State) -> Result<()>>;

pub struct AlignmentConfig {
  template: Option<AlignmentTemplate>,
  open_container: Option<OpenContainerFn>,
  close_container: Option<CloseContainerFn>,
  open_row: Option<OpenRowFn>,
  close_row: Option<CloseRowFn>,
  close_column: Option<CloseColumnFn>,
  attributes: HashMap<String,String>
}

pub struct Alignment {}
impl Alignment {
  /// Create a new Alignment.
  /// `config` can contain:
  ///    template : an Alignment::Template object
  ///    openContainer  = sub($doc,%attrib); creates the container element with given attributes
  ///    closeContainer = sub($doc); closes the container
  ///    openRow        = sub($doc,%attrib); creates the row element with given attributes
  ///    closeRow       = closes the row
  ///    openColumn     = sub($doc,%attrib); creates the column element with given attributes
  ///    closeColumn    = closes the column
  ///    attributes = hashref containing extra attributes for the container element.
  pub fn new(config:AlignmentConfig) -> Self {
    Alignment {}
  }
}

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Dealing with templates
#[derive(Debug,Clone)]
pub struct AlignmentTemplate {
  columns: Vec<Tokens>,
  tokens: Vec<Token>,
  reversion: Option<Tokens>
}
impl Default for AlignmentTemplate {
  fn default() -> Self {
    AlignmentTemplate { columns: Vec::new(), tokens: Vec::new(), reversion: None }
  }
}
impl Display for AlignmentTemplate {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Alignment[]",
    )
  }
}
impl AlignmentTemplate {
  fn set_reversion(&mut self, tks: Tokens) {
    self.reversion = Some(tks);
  }
}

// newcolumntype
//  defines \NC@rewrite@<char>
//    As macro
//    or "constructor" (or just sub that creates a column)

/// a reader for the AlignmentTemplate parameter type
pub fn read_alignment_template(gullet: &mut Gullet, inner: Option<&Parameters>, extra: &[Tokens], state: &mut State) -> Result<ArgWrap> {
  gullet.skip_spaces(state);
  let mut build_template = AlignmentTemplate::default();
  let mut tokens = vec![T_BEGIN!()];
  let mut nopens = 0;
  while let Some(open) = gullet.read_token(state) {
    if open.get_catcode() == Catcode::BEGIN {
      nopens +=1;
    } else {
      gullet.unread_one(open);
      break;
    }
  }
  while let Some(op) = gullet.read_token(state) {
    let cc = op.get_catcode();
    if cc == Catcode::SPACE {}
    else if cc == Catcode::END {
      let mut last_op = op;
      nopens -=1;
      while nopens > 0 {
        if let Some(next_op) = gullet.read_token(state) {
          last_op = next_op;
          if last_op.get_catcode() != Catcode::END {
            break;
          }
        } else {
          break;
        }
        nopens -=1;
      }
      if nopens <= 0 { break; }
      gullet.unread_one(last_op);
    } else if let Some(defn) = state.lookup_expandable(&T_CS!("\\NC@rewrite@{op}"),true) {
      let invoked = defn.invoke(gullet, true, state)?;
      gullet.unread(invoked);
    } else if cc == Catcode::BEGIN {
      if let Some(balanced_tks) = gullet.read_balanced(false, state)? {
        gullet.unread(balanced_tks);
      }
    } else {
      Warn!("unexpected", op, gullet, state, "Unrecognized tabular template {op:?}");
    }
    if nopens <= 0 { break; }
  }
  tokens.push(T_END!());
  build_template.set_reversion(Tokens::new(tokens));
  Ok(ArgWrap::AlignmentTemplate(build_template))
}