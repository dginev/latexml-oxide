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
use crate::common::arena;
use crate::document::Document;
use crate::gullet::Gullet;
use crate::mouth::Mouth;
use crate::state::State;
use crate::token::Catcode;
use crate::tokens::Tokens;
use crate::digested::Digested;
use self::template::{Column, Row, Template, Align, TemplateConfig, ColumnSpec, Axis, BorderSpec};

use libxml::tree::{Node, NodeType};
use rustc_hash::FxHashMap as HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::fmt::{self,Display, Debug};
use std::borrow::Cow;
use once_cell::sync::Lazy;
use regex::Regex;

//DebuggableFeature('alignment', "Debug guessing headers of alignments/tables");
pub type OpenContainerFn =
  Rc<dyn Fn(&mut Document, HashMap<String, String>, &mut State) -> Result<Option<Node>>>;
pub type CloseContainerFn = Rc<dyn Fn(&mut Document, &mut State) -> Result<Option<Node>>>;
pub type OpenRowFn = Rc<dyn Fn(&mut Document, HashMap<String, String>, &mut State) -> Result<()>>;
pub type CloseRowFn = Rc<dyn Fn(&mut Document, &mut State) -> Result<Option<Node>>>;
pub type OpenColumnFn = Rc<dyn Fn(&mut Document, HashMap<String, String>, &mut State) -> Result<Option<Node>>>;
pub type CloseColumnFn = Rc<dyn Fn(&mut Document, &mut State) -> Result<Option<Node>>>;

static SINGLE_PUNCT : Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*[\.,;]\s*$").unwrap());

pub struct AlignmentConfig {
  pub template: Option<Template>,
  pub open_container: OpenContainerFn,
  pub close_container: CloseContainerFn,
  pub open_row: OpenRowFn,
  pub close_row: CloseRowFn,
  pub open_column: OpenColumnFn,
  pub close_column: CloseColumnFn,
  pub properties: HashMap<String, Stored>,
  pub is_math: bool,
}

#[derive(Clone)]
pub struct Alignment {
  in_column: bool,
  in_row: bool,
  in_tabular_head: bool,
  is_math: bool,
  is_normalized: bool,
  current_column: usize,
  current_row: Option<usize>,
  reversion: Option<Tokens>,
  content_reversion: Option<Tokens>,
  rows: VecDeque<Row>,
  properties: HashMap<String,Stored>,
  template: Template,
  open_container: OpenContainerFn,
  close_container: CloseContainerFn,
  open_row: OpenRowFn,
  close_row: CloseRowFn,
  open_column: OpenColumnFn,
  close_column: CloseColumnFn,
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
      open_container: config.open_container,
      close_container: config.close_container,
      open_row: config.open_row,
      close_row: config.close_row,
      open_column: config.open_column,
      close_column: config.close_column,
      current_column: 0,
      is_math: false,
      in_row:false,
      in_column:false,
      in_tabular_head: false,
      is_normalized: false,
      properties: config.properties,
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
  pub fn add_before_row(&mut self, boxes:Vec<Digested>) {
    if let Some(cw) = self.current_row {
      let current_row = self.rows.get_mut(cw).unwrap();
      current_row.before.extend(boxes);
    }
  }

  pub fn add_after_row(&mut self, boxes:Vec<Digested>) {
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
  pub fn be_absorbed(&mut self, document:&mut Document, state: &mut State) -> Result<Vec<Node>> {
    let ismath = self.is_math;
    self.normalize_alignment()?;
    let rows = &mut self.rows;
    if rows.is_empty() {
      return Ok(Vec::new())
    }

    // # Guard via the absorb limit to avoid infinite loops
    // TODO
    // if ($LaTeXML::ABSORB_LIMIT) {
    //   my $absorb_counter = $STATE->lookupValue("absorb_count") || 0;
    //   $STATE->assignValue(absorb_count => ++$absorb_counter, "global");
    //   if ($absorb_counter > $LaTeXML::ABSORB_LIMIT) {
    //     Fatal("timeout", "absorb_limit", $self,
    //       "Whatsit absorb limit of $LaTeXML::ABSORB_LIMIT exceeded, infinite loop?"); } }

    // We _should_ attach boxes to the alignment and rows,
    // but (ATM) we"ve only got sensible boxes for the cells.
      let attrs   = if let Some(Stored::HashString(attrs)) = self.properties.remove("attributes") {
        attrs
      } else {
        HashMap::default()
      };
    let open_attrs = attrs.clone();
    // TODO:
    // open_attrs.insert("cwidth", self.cwidth);
    // open_attrs.insert("cheight", self.cheight);
    // open_attrs.insert("cdepth", self.cdepth);
    // open_attrs.insert("rowheights", self.rowheights);
    // open_attrs.insert("columnwidths", self.columnwidths);
    let open_container_fn = &self.open_container;
    open_container_fn(document, open_attrs, state)?;

    for row in rows {
      let vpad = row.get_padding();
      //     # Which properties do we expose to the constructor?
      let open_row_attrs = HashMap::default();
      //     "xml:id" => $$row{id}, tags => $$row{tags},
      //     x      => $$row{x}, y => $$row{y},
      //     cwidth => $$row{cwidth}, cheight => $$row{cheight}, cdepth => $$row{cdepth},
      //   );
      let open_row_fn = &self.open_row;
      open_row_fn(document, open_row_attrs, state)?;
      for before in row.before.iter() {
        document.absorb(before, None, state)?;
      }
      for cell in row.get_columns_mut() {
        if cell.skipped {
          continue;
        }
        // Normalize the border attribute
        let mut border = String::new();
        let mut border_iter = cell.border.chars().filter(|c| !c.is_whitespace()).peekable();
        while let Some(border_c) =  border_iter.next() {
          border.push(border_c);
          if let Some(next_c) = border_iter.peek() {
            if border_c != *next_c {
              border.push(' ');
            }
          }
        }
        let empty = cell.empty || cell.boxes.is_none() || cell.boxes.as_ref().unwrap().is_empty();
        let open_column_fn = &self.open_column;
        let mut cell_attrs = HashMap::default();
        // TODO: add to cell_attrs
        //       align   => $$cell{align}, width => $$cell{width},
        //       vattach => $$cell{vattach},
        //       ($vpad                       ? (cssstyle => "padding-bottom:" . ToString($vpad))     : ()),
        //       (($$cell{colspan} || 1) != 1 ? (colspan  => $$cell{colspan})                         : ()),
        //       (($$cell{rowspan} || 1) != 1 ? (rowspan  => $$cell{rowspan})                         : ()),
        //       ($border                     ? (border   => $border)                                 : ()),
        if cell.thead_in_column || cell.thead_in_row {
          let mut thead = String::new();
          if cell.thead_in_column {
            thead.push_str(Axis::Column.name());
            if cell.thead_in_row {
              thead.push(' ');
            }
          }
          if cell.thead_in_row {
            thead.push_str(Axis::Row.name());
          }
          if !thead.is_empty() {
            cell_attrs.insert(String::from("thead"), thead);
          }
        }
        //       # Which properties do we expose to the constructor?
        //       x      => $$cell{x}, y => $$cell{y},
        //       cwidth => $$cell{cwidth}, cheight => $$cell{cheight}, cdepth => $$cell{cdepth})
        cell.cell = open_column_fn(document, cell_attrs, state)?;
        if !empty {
          let box_ref = cell.boxes.as_ref().unwrap();
          // local $LaTeXML::BOX
          document.set_box_to_absorb(Some(box_ref.clone()));
          if ismath {// Hacky!
            document.open_element("ltx:XMArg", Some(string_map!("rule" => "Anything")), None, state)?;
          }
          document.absorb(box_ref, None, state)?;
          if ismath {// Hacky!
            document.close_element("ltx:XMArg", state)?;
          }
          // expire local $LaTeXML::BOX
          document.expire_box_to_absorb();
        }
        let close_column_fn = &self.close_column;
        close_column_fn(document, state)?;
      }
      for after in row.after.iter() {
        document.absorb(after, None, state)?;
      }
      let close_row_fn = &self.close_row;
      close_row_fn(document, state)?;
    }
    let close_container_fn = &self.close_container;
    let node_opt = close_container_fn(document, state)?;

    // If we're not nested inside another tabular
    // [This should be an afterConstruct somewhere?]
    // If requested to guess headers & we're not nested inside another tabular
    if let Some(mut node) = node_opt {
      if document.findnodes("ancestor::ltx:tabular", Some(&node), state).is_empty() {
        let hashead = !document.findnodes("descendant::ltx:td[@thead]", Some(&node), state).is_empty();
        // If requested && no cells are already marked as being thead, apply heuristic
        if self.properties.contains_key("guess_headers") && !hashead {
          guess_alignment_headers(document, &mut node, self, state)?;
        }
        // Otherwise, if not a math array, group thead & tbody rows
        // TODO: Re-design asking the outer Whatsit about "!body->isMath"
        else if hashead && !ismath { // in case already marked w/thead|tbody
          alignment_regroup_rows(document, &node, state)?;
        }
      }
      Ok(vec![node])
    } else {
      Ok(Vec::new())
    }

  }

  ///======================================================================
  /// Normalize an alignment before construction
  /// * consolodating column & row spanning information
  /// * scanning for empty rows & columns and collapsing them
  ///   (while accounting for spanning, and copying borders appropriately)
  /// Note that a trailing \\ in allignment (often needed to effect \hline)
  /// causes an empty row at the end. Other fancy layout fine-tuning often
  /// involves adding extra rows & columsn for spacing.  HTML's table model
  /// is more forgiving that TeX's, so we don't need these extras
  /// and, in fact, they often mess up the html layout!
  /// However, math alignments, and those with expected structure (eg. eqnarray)
  /// should generally NOT have rows & columns collapsed --- except the last row!
  ///
  /// Also note the inconsistency between TeX & HTML's table models regarding spans.
  /// \multicolumn creates a cell that covers a certain number of columns
  /// which are then omitted from the LaTeX AND the HTML.
  /// OTOH, \multirow creates a cell which overlaps following rows!
  /// The & is still needed to allocate the cells in those rows.
  /// And in fact they need not even be empty! TeX will just pile them up!
  /// However, in HTML the spanned rows ARE omitted!
  pub fn normalize_alignment(&mut self) -> Result<()> {
    if self.is_normalized {
      return Ok(());
    }
    //======================================================================
    self.normalize_cell_sizes()?;
    self.normalize_mark_spans()?;
    self.normalize_prune_rows()?;
    self.normalize_prune_columns()?;
    self.normalize_sum_sizes()?;
    //======================================================================
    self.is_normalized = true;
    Ok(())
  }
  /// Compute (approximate) sizes of all cells
  pub fn normalize_cell_sizes(&mut self) -> Result<()> {
    // Examines: boxes, align, vattach
    // Sets: cwidth, cheight, cdepth (per cell) & empty
    for row in &mut self.rows {
      // Do we need to account for any space in the $$row{before} or $$row{after}?
      for cell in row.get_columns_mut() {
        if let Some(_boxes) = &cell.boxes {
          // TODO
          // let (w, h, d, cw, ch, cd)
          //   = boxes.get_size(align => cell.align, width => cell.width,
          //     vattach => cell.vattach);

          // Debug("CELL (" . join(',', map { $_ . "=" . ToString($$cell{$_}); } qw(align width vattach))
          //     . ") size " . showSize($w,  $h,  $d)
          //     . " csize " . showSize($cw, $ch, $cd)
          //     . " Boxes=" . ToString($boxes)) if $LaTeXML::DEBUG{halign} && $LaTeXML::DEBUG{size};

          let empty = false;
          // TODO:
          // let empty =
          //   ((!cw || cw.value_of() < 1)
          //     || (((!ch) || ch->valueOf < 1)
          //     && ((!cd) || cd->valueOf < 1))
          //     || !(grep { !_->getProperty('isSpace'); } boxes->unlist)
          //   ) && !preservedBoxes(boxes);
          // cell{cwidth}  = w || Dimension(0);
          // cell{cheight} = h || Dimension(0);
          // cell{cdepth}  = d || Dimension(0);
          cell.empty = empty;
          if empty {
            cell.align = None;
          }
        } else {
          cell.empty = true;
        }
      }
    }
    Ok(())
  }
  /// Mark any cells that are covered by rowspan or colspan
  pub fn normalize_mark_spans(&mut self) -> Result<()> {Ok(())}
  /// Scan for and remove empty rows
  /// but copying borders and adjusting rowspan's & colspan's appropriately.
  pub fn normalize_prune_rows(&mut self) -> Result<()> {Ok(())}
  /// Scan for and remove empty columns
  /// but copying borders and adjusting rowspan's & colspan's appropriately.
  pub fn normalize_prune_columns(&mut self) -> Result<()> {Ok(())}
  pub fn normalize_sum_sizes(&mut self) -> Result<()> {Ok(())}


  pub fn compute_size(
    &self,
    _options: HashMap<String, Stored>,
    _state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    Ok((Dimension::new(0),Dimension::new(0),Dimension::new(0)))
  }
}

//======================================================================
// Constructing the XML for the alignment.

impl Object for Alignment {
  fn get_locator(&self) -> Option<Cow<crate::common::locator::Locator>> {
      None
  }
}

impl Debug for Alignment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Alignment[TODO]")
  }
}

impl Display for Alignment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{self:?}")
  }
}
impl PartialEq for Alignment {
  fn eq(&self, other: &Alignment) -> bool {
    // TODO: Is it enough to compare the owned template?
    self.template == other.template
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


//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Experimental alignment heading heuristications.
//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// We attempt to recognize patterns of rows/columns that indicate which might be headers.
// We'll characterize the cells by alignment, content and borders.
// Then, assuming that headers will be first and be noticably `different' from data lines,
// and also that the data lines will have similar structure,  we'll attempt to
// recognize groups of header lines and groups data lines, possibly alternating.

fn guess_alignment_headers(document: &mut Document, table: &mut Node, alignment: &mut Alignment, state: &mut State) -> Result<()> {
  // Assume that headers don't make sense for nested tables.
  // OR Maybe we should only do this within table environments???
  if !document.findnodes("ancestor::ltx:tabular", Some(table), state).is_empty() {
    return Ok(())
  }
  let tag = document.get_node_qname(table, state);
  // TODO
  //   Debug(('=' x 50) . "\nGuessing alignment headers for "
  //       . (($x = $document->findnode('ancestor-or-self::*[@xml:id]', $table)) ? $x->getAttribute('xml:id') : $tag))
  //     if $LaTeXML::DEBUG{alignment};

  let ismath = tag == arena::pin_static("ltx:XMArray");
  let reversed = false;
  // Attempt to recognize header lines.
  // Build a view of the table by extracting the rows, collecting & characterizing each cell.
  classify_alignment_rows(document, table, alignment, state);
  // Flip the rows around to produce a column view.
  {
    let mut cols = collect_alignment_columns(alignment);
    // This usually does something unpleasant
    alignment_characterize_lines(document, Axis::Column, false,  cols.as_mut_slice(), state)?;
  }

  let mut rows = collect_alignment_rows(alignment);
  if rows.is_empty() {
    return Ok(());
  }
  alignment_characterize_lines(document, Axis::Row, false, rows.as_mut_slice(), state)?;

  // Did we go overboard?
  let mut n_h = 0;
  let mut n_d = 0;
  for r in rows.iter() {
    for c in r {
      match c.cell_type {
        Some('h') => n_h += 1,
        Some('d') => n_d += 1,
        Some(other) => panic!("unexpected cell_type {}", other),
        None => {}
      }
    }
  }

//   Debug("$n{h} header, $n{d} data cells") if $LaTeXML::DEBUG{alignment};
  if n_d == 1 { // Or any other heuristic?
    n_h = 0;
    for r in rows {
      for mut c in r {
        c.cell_type = Some('d');
        if let Some(ref mut cell) = c.cell {
          cell.remove_attribute("thead")?;
        }
      }
    }
  }
  // Regroup the rows into thead & tbody elements.
  // But not if it's a math array, or if reversed (since browsers get confused?)
  if !ismath && !reversed {
    alignment_regroup_rows(document, table, state)?;
  }
  if n_h > 0 { // Found some headers?
    document.add_class(table, "ltx_guessed_headers", state)?;
  }

//   # Debugging report!
//   summarize_alignment([@rows], [@cols]) if $LaTeXML::DEBUG{alignment};
  Ok(())
}

//======================================================================
// Regroup the rows into thead, tbody & tfoot
// Any leading rows, all of whose cells have attribute thead should be in thead.
// UNLESS any of them have a rowspan that extends PAST the end of the thead!!!!
// trailing rows marked as thead go into tfoot.
fn alignment_regroup_rows(document: &mut Document, table: &Node, state: &mut State) -> Result<()> {
  let mut rows     = document.findnodes("ltx:tr", Some(table), state);
  let mut heads    = Vec::new();
  let mut maxreach = 0;
  // Scan initial rows as potential thead
  while !rows.is_empty() {
    let cells = document.findnodes("ltx:td", Some(&rows[0]),state);
    // Non header cells, done.
    if cells.iter().any(|cell| cell.get_attribute("thead").is_none()) {
      break;
    }
    let line = heads.len();
    heads.push(rows.remove(0));
    for cell in cells {
      let this_rowspan = cell.get_attribute("rowspan").map(|v| v.parse::<usize>().expect("rowspan should be a usize")).unwrap_or(0) + line;
      if this_rowspan > maxreach {
        maxreach = this_rowspan;
      }
    }
  }
  if maxreach > heads.len() { // rowspan crossed over thead boundary!
    rows.extend(heads.drain(..));
  }
  // scan trailing rows as potential tfoot
  let mut foots = VecDeque::new();
  while !rows.is_empty() {
    let cells = document.findnodes("ltx:td", Some(rows.last().unwrap()), state);
    // Non header cells, done.
    if cells.iter().any(|cell| cell.get_attribute("thead").is_none()) {
      break;
    }
    foots.push_front(rows.pop().unwrap())
  }
  if !heads.is_empty() {
    document.wrap_nodes("ltx:thead", heads, state)?;
  }
  if !rows.is_empty() {
    document.wrap_nodes("ltx:tbody", rows, state)?;
  }
  if !foots.is_empty() {
    document.wrap_nodes("ltx:tfoot", foots.into_iter().collect(), state)?;
  }
  Ok(())
}

//======================================================================
/// Setup a View of the alignment, with characterized cells, for analysis -- modifying it in place.
fn classify_alignment_rows<'a>(document: &mut Document, table: &Node, alignment: &'a mut Alignment, state: &mut State) {
  let nrows = alignment.rows.len();
  let mut ncols = 0;
  for arow in &mut alignment.rows {
    let n = arow.get_columns().len();
    if n > ncols {
      ncols = n;
    }
  }
  for arow in &mut alignment.rows {
    let cols = arow.get_columns_mut();
    let this_row_len = cols.len();
    for col in cols.iter_mut() {
      col.cell_type = Some('d');
      col.content_class = Some( // Assume mixed content for any justified cell???
        if col.align == Some(Align::Justify) {
          ColumnSpec::MathAltText
        } else if col.cell.is_some() {
          classify_alignment_cell(document, col.cell.as_ref().unwrap(), state)
        } else { ColumnSpec::Unknown }
      );
      col.content_length = Some(
        if col.content_class == Some(ColumnSpec::Graphics) { 1000 }
        else if col.cell.is_some() { col.cell.as_ref().unwrap().get_content().len() } else { 0 });
      let (mut border_top, mut border_bottom, mut border_left, mut border_right) = (0,0,0,0);
      for c in col.border.chars() {
        match c {
          'l' => border_left+=1,
          'r' => border_right+=1,
          't' => border_top +=1,
          'b' => border_bottom +=1,
          _ => {}// spaces etc.
        }
      }
      let h =  border_top>0 || border_bottom>0;
      let v = border_right > 0 || border_left > 0;
      if border_top > 0 {
        col.border_top = Some(border_top);
      }
      if border_bottom > 0 {
        col.border_bottom = Some(border_bottom);
      }
      if border_left > 0 {
        col.border_left = Some(border_left);
      }
      if border_right > 0 {
        col.border_right = Some(border_right);
      }
    }
    // pad the columns out.
    let to_pad = ncols - this_row_len;
    if to_pad > 0 {
      for _ in 0..to_pad {
        let col = Column {
          align: Some(Align::Center),
          cell_type: Some('d'),
          content_class: Some(ColumnSpec::Empty),
          content_length: Some(0),
          .. Column::default()
        };
        cols.push(col);
      }
    }
  }
  // # copy the characterizations to spanned cells
  // for (my $r = 0 ; $r < $nrows ; $r++) {
  //   for (my $c = 0 ; $c < $ncols ; $c++) {
  //     my $rs = $rows[$r][$c]{rowspan} || 1;
  //     my $cs = $rows[$r][$c]{colspan} || 1;
  //     my $ca = $rows[$r][$c]{align};
  //     my $cc = $rows[$r][$c]{content_class};
  //     my $cl = $rows[$r][$c]{content_length};
  //     my $rb = $rows[$r][$c]{r}; $rows[$r][$c]{r} = 0;
  //     my $bb = $rows[$r][$c]{b}; $rows[$r][$c]{b} = 0;
  //     for (my $sc = 1 ; $sc < $cs ; $sc++) {
  //       $rows[$r][$c + $sc]{align}          = $ca;
  //       $rows[$r][$c + $sc]{content_class}  = $cc;
  //       $rows[$r][$c + $sc]{content_length} = $cl; }
  //     for (my $sr = 1 ; $sr < $rs ; $sr++) {
  //       for (my $sc = 0 ; $sc < $cs ; $sc++) {
  //         $rows[$r + $sr][$c + $sc]{align}          = $ca;
  //         $rows[$r + $sr][$c + $sc]{content_class}  = $cc;
  //         $rows[$r + $sr][$c + $sc]{content_length} = $cl; } }
  //     # move the outer borders
  //     for (my $sr = 0 ; $sr < $rs ; $sr++) {
  //       $rows[$r + $sr][$c + $cs - 1]{r} = $rb; }
  //     for (my $sc = 0 ; $sc < $cs ; $sc++) {
  //       $rows[$r + $rs - 1][$c + $sc]{b} = $bb; }
  // } }

  // # Now, do some border massaging...
  // for (my $r = 0 ; $r < $nrows ; $r++) {
  //   $rows[$r][0]{l}          = $v;
  //   $rows[$r][0]{r}          = $rows[$r][1]{l}          if ($ncols > 1) && $rows[$r][1]{l};
  //   $rows[$r][$ncols - 1]{l} = $rows[$r][$ncols - 2]{r} if ($ncols > 1) && $rows[$r][$ncols - 2]{r};
  //   $rows[$r][$ncols - 1]{r} = $v; }
  // for (my $c = 0 ; $c < $ncols ; $c++) {
  //   $rows[0][$c]{t}          = $h;
  //   $rows[0][$c]{b}          = $rows[1][$c]{t}          if ($nrows > 1) && $rows[1][$c]{t};
  //   $rows[$nrows - 1][$c]{t} = $rows[$nrows - 2][$c]{b} if ($nrows > 1) && $rows[$nrows - 2][$c]{b};
  //   $rows[$nrows - 1][$c]{b} = $h; }
  // for (my $r = 1 ; $r < $nrows - 1 ; $r++) {
  //   for (my $c = 1 ; $c < $ncols - 1 ; $c++) {
  //     $rows[$r][$c]{t} = $rows[$r - 1][$c]{b} if $rows[$r - 1][$c]{b};
  //     $rows[$r][$c]{b} = $rows[$r + 1][$c]{t} if $rows[$r + 1][$c]{t};
  //     $rows[$r][$c]{l} = $rows[$r][$c - 1]{r} if $rows[$r][$c - 1]{r};
  //     $rows[$r][$c]{r} = $rows[$r][$c + 1]{l} if $rows[$r][$c + 1]{l}; } }
  //
  // if ($LaTeXML::DEBUG{alignment}) {
  //   Debug("Cell characterizations:");
  //   for (my $r = 0 ; $r < $nrows ; $r++) {
  //     for (my $c = 0 ; $c < $ncols ; $c++) {
  //       my $col = $rows[$r][$c];
  //       Debug("[$r,$c]=>" . ($$col{cell_type} || '?')
  //           . ($$col{align} ? $ALIGNMENT_CODE{ $$col{align} } : ' ')
  //           . ($$col{content_class} || '?')
  //           . ' ' . $$col{content_length}
  //           . ' ' . $$col{border} . "=>" . join('', grep { $$col{$_} } qw(t r b l))
  //           . (($$col{rowspan} || 1) > 1 ? " rowspan=" . $$col{rowspan} : '')
  //           . (($$col{colspan} || 1) > 1 ? " colspan=" . $$col{colspan} : '')); } } }
}

fn collect_alignment_rows(alignment: &mut Alignment) -> Vec<Vec<&mut Column>> {
  alignment.rows.iter_mut().map(|x| x.get_columns_mut().into_iter().map(|x| x)
    .collect()).collect()
}

fn collect_alignment_columns(alignment: &mut Alignment) -> Vec<Vec<&mut Column>> {
  let mut columns = Vec::new();
  let mut row_cells : Vec<_> = alignment.rows.iter_mut().map(|r| r.get_columns_mut().iter_mut()).collect();
  for _ in 0..row_cells[0].len() {
    let mut column = Vec::new();
    for row_iter in row_cells.iter_mut() {
      column.push(row_iter.next().unwrap());
    }
    columns.push(column);
  }
  columns
}

/// Return one of: i(nteger), t(ext), m(ath), ? (unknown) or '_' (empty) (or some combination)
///  or 'mx' for alternating text & math.
fn classify_alignment_cell(document: &mut Document, xcell: &Node, state: &mut State) -> ColumnSpec {
  let content = xcell.get_content();
  let mut inferred_classes: Vec<ColumnSpec>   = Vec::new();
  if content.chars().all(|c| c.is_whitespace() || c.is_numeric()) {
    inferred_classes.push(ColumnSpec::Integer);
  } else {
    let mut nodes = xcell.get_child_nodes();
    while !nodes.is_empty() {
      let ch = nodes.remove(0);
      match ch.get_type() {
        Some(NodeType::TextNode) => {
          let text = ch.get_content();
          if !(text.chars().all(|c| c.is_whitespace()) || (
            inferred_classes.get(0)== Some(&ColumnSpec::Math) && SINGLE_PUNCT.is_match(&text))) {
            inferred_classes.push(ColumnSpec::Text);
          }
        },
        Some(NodeType::ElementNode) => {
          document.with_node_qname(&ch, state, |chtag| match chtag {
            "ltx:text" => if inferred_classes.first() != Some(&ColumnSpec::Text) {
            // Font would be useful, but haven't "resolved" it, yet!
              inferred_classes.push(ColumnSpec::Text); },
            "ltx:graphics" => if inferred_classes.first() != Some(&ColumnSpec::Graphics) {
              inferred_classes.push(ColumnSpec::Graphics);
            },
            "ltx:Math" => if inferred_classes.first() != Some(&ColumnSpec::Math) {
              inferred_classes.push(ColumnSpec::Math);
            },
            "ltx:XMText" => if inferred_classes.first() != Some(&ColumnSpec::Text) {
              inferred_classes.push(ColumnSpec::Text);
            },
            "ltx:XMArg" => {
              let mut children = ch.get_child_nodes();
              children.append(&mut nodes);
              nodes = children;
            },
            other if other.starts_with("ltx:XM") => {
              if inferred_classes.first() != Some(&ColumnSpec::Math) {
                inferred_classes.push(ColumnSpec::Math);
              }
            },
            _ => {
              if inferred_classes.is_empty() {
                inferred_classes.push(ColumnSpec::Unknown);
              }
            }
          })
        },
        _ => {}
      }
    }
  }

  // check if we have alternating math-and-text or text-and-math
  let mut alt_peekable = inferred_classes.iter().peekable();
  let mut is_alternating = true;
  while let Some(c) = alt_peekable.next() {
    if matches!(c, ColumnSpec::Math | ColumnSpec::Integer) {
      if let Some(peek) = alt_peekable.peek() {
        if !matches!(peek, ColumnSpec::Text) {
          is_alternating = false;
          break;
        }
      }
    } else if matches!(c, ColumnSpec::Text) {
      if let Some(peek) = alt_peekable.peek() {
        if !matches!(peek, ColumnSpec::Math | ColumnSpec::Integer) {
          is_alternating = false;
          break;
        }
      }
    } else {
      is_alternating = false;
      break;
    }
  }
  if is_alternating {
    inferred_classes = vec![ColumnSpec::MathAltText];
  }
  // Default to empty and return
  if inferred_classes.is_empty() {
    ColumnSpec::Empty
  } else if inferred_classes.len() == 1 {
    inferred_classes[0]
  } else {
    // TODO: What do we do for multi-class detection?
    ColumnSpec::Unknown
  }
}



//======================================================================
// Scan pairs of rows/columns attempting to recognize differences that
// might indicate which are headers and which are data.
// Warning: This section is full of "magic numbers"
// guessed by sampling various test cases.

const MIN_ALIGNMENT_DATA_LINES   : usize = 1; //  (or 2?) [CONSTANT]
const MAX_ALIGNMENT_HEADER_LINES : usize = 4; // [CONSTANT]

// We expect to find header lines at the beginning, noticably different from the eventual data lines.
// Both header lines and data lines can consist of several neighboring lines.
// Check that header lines are `similar' to each other.  So, the strategy is to look
// for a `hump' in the line differences and consider blocks containing these lines to be potential headers.

fn alignment_characterize_lines(document:&mut Document, axis:Axis, reversed:bool, lines: &mut [Vec<&mut Column>], state:&State) -> Result<()> {
  let n = lines.len();
  if n<2 {
    return Ok(());
  }
  // Debug("Characterizing $n " . ($axis ? "columns" : "rows"))
  //   if $LaTeXML::DEBUG{alignment};

  // Establish a scale of differences for the table.
  let (mut max_diff, mut min_diff, mut avg_diff) = (0.0, 99999999.0, 0.0);
  for l in 0..n-1 {
    let d = alignment_compare(axis, true, reversed, l, l + 1, lines);
    avg_diff += d;
    if d > max_diff {
      max_diff = d;
    }
    if d < min_diff {
      min_diff = d;
    }
  }
  let avg_diff = avg_diff / (n - 1) as f64;
  if max_diff < 0.05 { // virtually no differences.
  //   Debug("Lines are almost identical => Fail") if $LaTeXML::DEBUG{alignment};
    return Ok(());
  }
  if (n > 2) && ((max_diff - min_diff) < max_diff * 0.5) { // differences too similar to establish pattern
  //   Debug("Differences between lines are almost identical => Fail")
  //     if $LaTeXML::DEBUG{alignment};
    return Ok(());
  }
  let tab_threshold = min_diff + 0.3 * (max_diff - min_diff);
  // local $::TAB_AXIS = $axis;

  // Debug("Differences $min_diff -- $max_diff => threshold = $::tab_threshold")
  //   if $LaTeXML::DEBUG{alignment};
  // Find the first hump in differences. These are candidates for header lines.
  // Debug("Scanning for headers") if $LaTeXML::DEBUG{alignment};
  let (mut minh, mut maxh) = (1, 1);
  let mut diff;
  loop {
    diff = alignment_compare(axis, true, reversed, maxh - 1, maxh, lines);
    if diff >= tab_threshold {break;}
    maxh+=1;
  }
  if maxh > MAX_ALIGNMENT_HEADER_LINES {// too many before even finding diffs? give up!
    return Ok(()); }
  while alignment_compare(axis, true, reversed, maxh, maxh + 1, lines) > tab_threshold {
    maxh+=1;
  }
  if maxh > MAX_ALIGNMENT_HEADER_LINES {
    maxh = MAX_ALIGNMENT_HEADER_LINES;
  }
  // Debug("Found from $minh--$maxh potential headers") if $LaTeXML::DEBUG{alignment};

  let nn = lines[0].len() - 1;
  // The sets of lines 1--$minh, .. 1--$maxh are potential headers.
  for nh in (minh..=maxh).rev() {
    // Check whether the set 1..$nh is plausable.
    let heads = alignment_test_headers(nh, lines);
    if !heads.is_empty()  {
      // Now, change all cells marked as header from td => th.
      for h in heads {
        for (i, cell) in lines[h].iter_mut().enumerate() {
          cell.cell_type = Some('h');
          if let Some(ref mut xcell) = cell.cell {
            if (cell.content_class == Some(ColumnSpec::Empty)) // But NOT empty cells on outer edges.
              && ((i == 0 && (if axis == Axis::Row {
                  cell.border_left.is_none()} else {cell.border_top.is_none()}) )
               || (i == nn && (if axis == Axis::Row {
                  cell.border_right.is_none()} else {cell.border_bottom.is_none()}))) { }
            else {
              document.add_ss_values(xcell, "thead", axis.marker_name(), state)?;
            }
          }
        }
      }
      return Ok(());
    }
  }
  Ok(())
}

/// Test whether `nhead` lines makes a good fit for the headers
fn alignment_test_headers(nhead:usize, lines:&mut [Vec<&mut Column>]) -> Vec<usize> {
  // Debug("Testing $nhead headers") if $LaTeXML::DEBUG{alignment};
  let (mut head_length, mut data_length) = (0, 0);
  let heads : Vec<usize> = (0 .. nhead).collect(); // The indices of heading lines.
  let head_length = alignment_max_content_length(head_length, 0, nhead - 1, lines);
  let next_line = nhead; // Start from the end of the proposed headings.

  // Watch out for the assumed header being really data that is a repeated pattern.
  let nrep = lines.len() / nhead;
  if nhead > 1 {
  //   Debug("Check for apparent header repeated $nrep times") if $LaTeXML::DEBUG{alignment};
  //   my $matched = 1;
  //   for (my $r = 1 ; $r < $nrep ; $r++) {
  //     $matched &&= alignment_match_head(0, $r * $nhead, $nhead); }
  //   Debug("Repeated headers: " . ($matched ? "Matched=> Fail" : "Nomatch => Succeed"))
  //     if $LaTeXML::DEBUG{alignment};
  //   return if $matched;
  }

  // # And find a following grouping of data lines.
  // my $ndata = alignment_skip_data($nextline);
  // return if $ndata < $nhead;                     # ???? Well, maybe if _really_ convincing???
  // return if ($ndata < $nhead) && ($ndata < 2);
  // # Check that the content of the headers isn't dramatically larger than the content in the data
  // $data_length = alignment_max_content_length($data_length, $nextline, $nextline + $ndata - 1);
  // $nextline += $ndata;

  // my $nd;
  // # If there are more lines, they should match either the previous data block, or the head/data pattern.
  // while ($nextline < lines.len()) {
  //   # First try to match a repeat of the 1st data block;
  //   # This would be the case when groups of data have borders around them.
  //   # Could want to match a variable number of datalines, but they should be similar!!!??!?!?
  //   if (($ndata > 1) && ($nd = alignment_match_data($nhead, $nextline, $ndata))) {
  //     $data_length = alignment_max_content_length($data_length, $nextline, $nextline + $nd - 1);
  //     $nextline += $nd; }
  //   # Else, try to match the first header block; less common.
  //   elsif (alignment_match_head(0, $nextline, $nhead)) {
  //     push(@heads, $nextline .. $nextline + $nhead - 1);
  //     $head_length = alignment_max_content_length($head_length, $nextline, $nextline + $nhead - 1);
  //     $nextline += $nhead;
  //     # Then attempt to match a new data block.
  //     #      my $d = alignment_skip_data($nextline);
  //     #      return unless ($d >= $nhead) || ($d >= 2);
  //     #      $nextline += $d; }
  //     # No, better be the same data block?
  //     return unless ($nd = alignment_match_data($nhead, $nextline, $ndata));
  //     $data_length = alignment_max_content_length($data_length, $nextline, $nextline + $nd - 1);
  //     $nextline += $nd; }
  //   else { return; } }
  // // Header content seems too large relative to data?
  // Debug("header content = $head_length; data content = $data_length")
  //   if $LaTeXML::DEBUG{alignment};
  // if (($head_length > 10) && (0.25 * $head_length > $data_length)) {
  //   Debug("header content too much longer than data content")
  //     if $LaTeXML::DEBUG{alignment};
  //   return; }
  // // Or if a header cell has "large" content?
  // if ($head_length >= 1000) {    # Or if a header cell has "large" content?
  //   Debug("header content too large")
  //     if $LaTeXML::DEBUG{alignment};
  //   return; }

  // Debug("Succeeded with $nhead headers") if $LaTeXML::DEBUG{alignment};
  heads
}

/// Return the maximum "content length" for lines from $from to $to.
fn alignment_max_content_length(mut length: usize, from:usize, to:usize, tablines: &mut [Vec<&mut Column>]) -> usize {
  for item in tablines.iter().take(to + 1).skip(from) {
    let mut l = 0;
    for cell in item.iter() {
      l += cell.content_length.unwrap_or(0);
    }
    if l > length {
      length = l;
    }
  }
  length
}


//======================================================================

/// Compare two lines along `Axis` (0=row,1=column), returning a measure of the difference.
/// The borders are compared differently if
///  `for_adjacency`: we adjacent lines that might belong to the same block,
///  otherwise    : comparing two lines that ought to have identical patterns (eg. in a repeated block)
fn alignment_compare(axis: Axis, for_adjacency:bool, reversed:bool, p1:usize, p2:usize, lines: &mut [Vec<&mut Column>]) -> f64 {
  let line1 = &lines[p1];
  let line2 = &lines[p2];
  if line1.is_empty() && line2.is_empty() {
    return 0.0;
  } else if line1.is_empty() || line2.is_empty() {
    return 999999.0;
  }
  let ncells = line1.len();
  let mut diff   = 0.0;

  for (cell1,cell2) in line1.iter().zip(line2.iter()) {
    // Annoying test avoids warnings if cells inconsistent; likely due to incorrect row/col spans
    if cell1.content_class.is_none() || cell2.content_class.is_none() {
      continue;
    }
    //   next if grep { !defined $$cell1{$_} } qw(content_class r l t b);
    //   next if grep { !defined $$cell2{$_} } qw(content_class r l t b);
    if cell1.align != cell2.align && cell1.content_class != Some(ColumnSpec::Empty)
      && cell2.content_class != Some(ColumnSpec::Empty) {
      diff += 0.75;
    }
    let d = cell1.content_class.as_ref().unwrap()
      .difference_heuristic(cell2.content_class.as_ref().unwrap());
    if d > 0.0 {
      diff += d;
    }
    // compare certain edges
    if for_adjacency { // Compare edges for adjacent rows of potentially different purpose
      let mut inner_diffs = 0.0;
      if axis == Axis::Row {
        if cell1.border_right != cell2.border_right { inner_diffs += 1.0; }
        if cell1.border_left != cell2.border_left { inner_diffs += 1.0; }
      } else {
        if cell1.border_top != cell2.border_top { inner_diffs += 1.0; }
        if cell1.border_bottom != cell2.border_bottom { inner_diffs += 1.0; }
      };
      diff += 0.3 * inner_diffs;
      // Penalty for apparent divider between.
      let pedge = if axis == Axis::Row {
        if reversed { BorderSpec::Top }else{ BorderSpec::Bottom}
        } else if reversed { BorderSpec::Left }else{BorderSpec::Right};
      let border1_pedge = cell1.border_at(pedge);
      let border2_pedge = cell2.border_at(pedge);
      if let Some(b1p) = border1_pedge {
        if border1_pedge != border2_pedge {
          diff += (b1p as i64 - border2_pedge.unwrap_or(0)as i64).abs() as f64;
        }
      }
    } else { // Compare edges for rows from diff places for potential similarity
      let mut inner_diffs = 0.0;
      if cell1.border_right != cell2.border_right { inner_diffs += 1.0; }
      if cell1.border_left != cell2.border_left { inner_diffs += 1.0; }
      if cell1.border_top != cell2.border_top { inner_diffs += 1.0; }
      if cell1.border_bottom != cell2.border_bottom { inner_diffs += 1.0; }
      diff += 0.3 * inner_diffs;
    }
  }
  diff /= ncells as f64;
  // Debug("$p1-$p2 => $diff; ") if $LaTeXML::DEBUG{alignment};
  diff
}
