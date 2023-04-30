//! # Representation of aligned structures
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

// keep in until code is completed.
#[allow(dead_code)]
pub mod template;

use crate::common::dimension::Dimension;
use crate::common::numeric_ops::NumericOps;
use crate::common::store::Stored;
use crate::common::error::*;
use crate::common::object::Object;
use crate::stomach::Stomach;
use crate::document::Document;
use crate::gullet::Gullet;
use crate::mouth::Mouth;
use crate::state::State;
use crate::token::{Token,Catcode};
use crate::tokens::Tokens;
use self::template::{Column, Row, Template, TemplateConfig};

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::fmt::{self,Display};

//DebuggableFeature('alignment', "Debug guessing headers of alignments/tables");
pub type OpenContainerFn =
  Rc<dyn Fn(&mut Document, HashMap<String, String>, &mut State) -> Result<()>>;
pub type CloseContainerFn = Rc<dyn Fn(&mut Document, &mut State) -> Result<()>>;
pub type OpenRowFn = Rc<dyn Fn(&mut Document, HashMap<String, String>, &mut State) -> Result<()>>;
pub type CloseRowFn = Rc<dyn Fn(&mut Document, &mut State) -> Result<()>>;
pub type OpenColumnFn = Rc<dyn Fn(&mut Document, HashMap<String, String>, &mut State) -> Result<()>>;
pub type CloseColumnFn = Rc<dyn Fn(&mut Document, &mut State) -> Result<()>>;

#[derive(Default)]
pub struct AlignmentConfig {
  pub template: Option<Template>,
  pub open_container: Option<OpenContainerFn>,
  pub close_container: Option<CloseContainerFn>,
  pub open_row: Option<OpenRowFn>,
  pub close_row: Option<CloseRowFn>,
  pub open_column: Option<OpenColumnFn>,
  pub close_column: Option<CloseColumnFn>,
  pub attributes: HashMap<String, Stored>,
  pub is_math: bool,
}

#[derive(Debug,Clone,Default, PartialEq)]
pub struct Alignment {
  in_column: bool,
  in_row: bool,
  in_tabular_head: bool,
  current_column: usize,
  current_row: Option<usize>,
  reversion: Option<Tokens>,
  content_reversion: Option<Tokens>,
  rows: VecDeque<Row>,
  template: Template,
}
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
  pub fn new(config: AlignmentConfig) -> Self {
    let template = config.template.unwrap_or_default();
    Alignment {
      template,
      current_row: None,
      reversion: None,
      content_reversion: None,
      current_column: 0,
      in_row:false,
      in_column:false,
      in_tabular_head: false,
      rows: VecDeque::new(),
    }
  }

  pub fn get_template(&self) -> &Template { &self.template }

  pub fn current_row(&self) -> Option<&Row> {
    match self.current_row {
      Some(idx) => self.rows.get(idx),
      None => None,
    }
  }
  pub fn current_row_mut(&mut self) -> Option<&mut Row> {
    match self.current_row {
      Some(idx) => self.rows.get_mut(idx),
      None => None,
    }
  }

  pub fn new_row(&mut self) -> Option<&Row> {
    let row = self.template.clone();
    self.current_row = Some(self.rows.len());
    self.rows.push_back(row);
    self.current_column = 0;
    self.rows.back()
  }

  pub fn remove_row(&mut self) -> Option<Row> { self.rows.pop_back() }

  pub fn prepend_rows(&mut self, new_rows: Vec<Row>) {
    for new_row in new_rows.into_iter().rev() {
      self.rows.push_front(new_row)
    }
  }

  pub fn append_rows(&mut self, new_rows: Vec<Row>) {
    for new_row in new_rows.into_iter() {
      self.rows.push_back(new_row)
    }
  }

  pub fn rows(&self) -> &VecDeque<Row> { &self.rows }

  pub fn add_line(&mut self, border: &str, cols: Vec<usize>) {
    if let Some(row_idx) = self.current_row {
      let row = self.rows.get_mut(row_idx).unwrap();
      self.current_column = 1;
      if !cols.is_empty() {
        for c in cols {
          let colspec = row.get_column_mut(c).unwrap();
          colspec.border.push_str(border);
        }
      } else {
        for colspec in row.get_columns_mut() {
          colspec.border.push_str(border)
        }
      }
    }
  }

  pub fn next_column(&mut self) -> Option<&mut Column> {
    self.current_row?;
    self.current_column +=1 ;
    let current_row = self.rows.get_mut(self.current_row.unwrap()).unwrap();
    if let Some(colspec) = current_row.get_column_mut(self.current_column) {
      Some(colspec)
    } else {
      Error!("unexpected", "&", None, None, "Extra alignment tab '&'");
      // current_row.add_column(Column{align: Some(Align::Center),..Column::default()});
      // let current_row = self.rows.get_mut(self.current_row.unwrap()).unwrap();
      None
    }
  }

  pub fn last_column(&mut self) -> Option<&mut Column> {
    if let Some(row_idx) = self.current_row {
      if let Some(row) = self.rows.get_mut(row_idx) {
        self.current_column = row.get_columns().len();
        row.get_column_mut(self.current_column)
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn current_column_number(&self) -> usize {
    self.current_column
  }

  pub fn current_row_number(&self) -> usize {
    let mut n = 0;
    for row in &self.rows {
      if !row.is_pseudo() {
        n+=1;
      }
    }
    n
  }

  pub fn current_column(&mut self) -> Option<&mut Column> {
    self.current_row.and_then(|cw| self.rows.get_mut(cw).unwrap()
      .get_column_mut(self.current_column))
  }

  pub fn get_column(&mut self, n:usize) -> Option<&mut Column> {
    // TODO: do we need an immutable variant? For now alias the mutable one
    self.get_column_mut(n)
  }

  pub fn get_column_mut(&mut self, n:usize) -> Option<&mut Column> {
    self.current_row.and_then(|cw|
      self.rows.get_mut(cw).unwrap().get_column_mut(n)) }

  // Ugh... these take boxes; adding before/after columns takes tokens!
  pub fn add_before_row(&mut self, boxes:Vec<Token>) {
    if let Some(cw) = self.current_row {
      let current_row = self.rows.get_mut(cw).unwrap();
      current_row.before.extend(boxes);
    }
  }

  pub fn add_after_row(&mut self, boxes:Vec<Token>) {
    if let Some(cw) = self.current_row {
      let current_row = self.rows.get_mut(cw).unwrap();
      current_row.after.extend(boxes);
    }
  }

  pub fn omit_column(&mut self) {
    if let Some(column) = self.current_column() {
      column.omitted = true;
    }
  }

  pub fn omit_next_column(&mut self) {
    if let Some(cw) = self.current_row {
      if let Some(column) = self.rows.get_mut(cw).unwrap().get_column_mut(cw + 1) {
        column.omitted = true;
      }
    }
  }

  pub fn get_column_before(&mut self) -> Tokens {
    if let Some(column) = self.current_column() {
      if !column.omitted {
        Tokens!(T_CS!("\\@column@before"), column.before.clone().unwrap_or_default().unlist())
      } else {
        Tokens!()
      }
    } else {
      Tokens!()
    }
  }

  pub fn get_column_after(&mut self) -> Tokens {
    if let Some(column) = self.current_column() {
      if !column.omitted {
        // Possible \@@eat@space ??? (if LaTeX style???)
        Tokens!(column.after.clone().unwrap_or_default().unlist(), T_CS!("\\@column@after"))
      } else {
        Tokens!()
      }
    } else {
      Tokens!()
    }
  }

  pub fn revert(&self, _state: &State) -> Result<Tokens> { Ok(self.reversion.clone().unwrap_or_default()) }

  pub fn set_reversion(&mut self, rev: Tokens) {
    self.reversion = Some(rev);
  }
  pub fn set_content_reversion(&mut self, rev: Tokens) {
    self.content_reversion = Some(rev);
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Support for building an alignment's Rows & Columns
  pub fn is_in_row(&self) -> bool { self.in_row }
  pub fn is_in_column(&self) -> bool { self.in_column }
  pub fn start_row(&mut self,pseudorow:bool,stomach:&mut Stomach, state:&mut State) -> Result<()> {
    self.new_row();
    stomach.bgroup(state);    // Grouping around ROW!
    if pseudorow {
      self.current_row_mut().unwrap().set_pseudo()
    } else {
      let row_before = stomach.digest(T_CS!("\\@row@before"), state)?;
      stomach.box_list.push( row_before );
    }
    self.in_row = true;
    state.assign_value("alignmentStartColumn", 0, None);    // ???
    Ok(())
  }

  pub fn end_row(&mut self, stomach: &mut Stomach, state:&mut State) -> Result<()> {
    if self.in_row {
      if self.in_column {
        self.end_column(stomach, state)?;
      }
      stomach.egroup(state)?;                        // Grouping around ROW!
      self.in_row = false;
    }
    Ok(())    //  Digest(T_CS('\@row@after'));
  }

  pub fn start_column(&mut self, pseudorow:bool, stomach:&mut Stomach, state:&mut State) -> Result<()> {
    if !self.in_row {
      self.start_row(pseudorow, stomach, state)?;
    } else if pseudorow {
      self.current_row_mut().unwrap().set_pseudo();
    }
    stomach.bgroup(state);    // Grouping around CELL!
                              // Note: a VERY round-about way of tracking the column spanning!
    state.assign_value("alignmentStartColumn", self.current_column_number(), None);
    let _colspec = self.next_column();
    state.set_align_group_count(1000000);
    self.in_column = true;
    Ok(())
  }

  pub fn end_column(&mut self, stomach: &mut Stomach, state:&mut State) -> Result<()> {
    if self.in_column {
      stomach.egroup(state)?; // Grouping around CELL!
      self.in_column = false;
    }
    Ok(())
  }

  pub fn set_in_tabular_head(&mut self) {
    self.in_tabular_head = true;
  }
  pub fn unset_in_tabular_head(&mut self) {
    self.in_tabular_head = false;
  }
  pub fn is_in_tabular_head(&self) -> bool {
    self.in_tabular_head
  }
  pub fn be_absorbed(&self, document:&mut Document, state: &mut State) -> Result<Vec<Node>> {
    unimplemented!();
  }
  pub fn compute_size(
    &self,
    options: HashMap<String, Stored>,
    state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    Ok((Dimension::new(0),Dimension::new(0),Dimension::new(0)))
  }
}

//======================================================================
// Constructing the XML for the alignment.

impl Object for Alignment {
  fn get_locator(&self) -> Option<std::borrow::Cow<crate::common::locator::Locator>> {
      None
  }
}


impl Display for Alignment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{self:?}")
  }
}


//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Dealing with templates

// newcolumntype
//  defines \NC@rewrite@<char>
//    As macro
//    or "constructor" (or just sub that creates a column)

/// a reader for the Template parameter type
pub fn read_alignment_template(gullet: &mut Gullet, state: &mut State) -> Result<Template> {
  gullet.skip_spaces(state)?;
  state.set_build_template(Template::default());
  let mut tokens = vec![T_BEGIN!()];
  let mut nopens = 0;
  while let Some(open) = gullet.read_token(state)? {
    if open.get_catcode() == Catcode::BEGIN {
      nopens += 1;
    } else {
      gullet.unread_one(open);
      break;
    }
  }
  while let Some(op) = gullet.read_token(state)? {
    let cc = op.get_catcode();
    if cc == Catcode::SPACE {
    } else if cc == Catcode::END {
      let mut last_op = op;
      nopens -= 1;
      while nopens > 0 {
        if let Some(next_op) = gullet.read_token(state)? {
          last_op = next_op;
          if last_op.get_catcode() != Catcode::END {
            break;
          }
        } else {
          break;
        }
        nopens -= 1;
      }
      if nopens <= 0 {
        break;
      }
      gullet.unread_one(last_op);
    } else if let Some(defn) = state.lookup_expandable(&T_CS!(s!("\\NC@rewrite@{op}")), true) {
      let invoked = defn.invoke(gullet, true, state)?;
      gullet.unread(invoked);
    } else if cc == Catcode::BEGIN {
      if let Some(balanced_tks) = gullet.read_balanced(false, state)? {
        gullet.unread(balanced_tks);
      }
    } else {
      Warn!(
        "unexpected",
        op,
        gullet,
        state,
        s!("Unrecognized tabular template {op:?}")
      );
    }
    if nopens <= 0 {
      break;
    }
  }
  tokens.push(T_END!());
  state.current_build_template().unwrap().set_reversion(Tokens::new(tokens));
  Ok(state.take_build_template().unwrap())
}

pub fn parse_alignment_template(
  spec: &str,
  gullet: &mut Gullet,
  ostate: &mut State,
) -> Result<Template> {
  let reader_mouth = Mouth::new(&s!("{{{spec}}}"), None, ostate)?;
  gullet.reading_from_mouth(reader_mouth, ostate, |gulletx: &mut Gullet, state| {
    read_alignment_template(gulletx, state)
  })
}

pub fn matrix_template() -> Template {
  Template::new(TemplateConfig {
    repeated: vec![Column {
      before: Some(Tokens!(T_CS!("\\hfil"))),
      after: Some(Tokens!(T_CS!("\\hfil"))),
      ..Column::default()
    }],
    ..TemplateConfig::default()
  })
}
