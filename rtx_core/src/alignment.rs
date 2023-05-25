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
//! like a Whatsit and be responsible for construction (be_absorbed),
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
use crate::BoxOps;
use self::template::{Cell, Row, Template, Align, TemplateConfig, ColumnSpec, Axis, BorderSpec};

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
  cwidth: Option<Dimension>,
  cheight: Option<Dimension>,
  cdepth: Option<Dimension>,
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
  ///    properties = hashref containing extra attributes for the container element.
  pub fn new(config: AlignmentConfig) -> Self {
    let template = config.template.unwrap_or_default();
    Alignment {
      template,
      current_row: None,
      reversion: None,
      content_reversion: None,
      cwidth: None,
      cheight: None,
      cdepth: None,
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

  pub fn next_column(&mut self) -> Option<&mut Cell> {
    self.current_row?;
    self.current_column +=1 ;
    let current_row = self.rows.get_mut(self.current_row.unwrap()).unwrap();
    if let Some(colspec) = current_row.get_column_mut(self.current_column) {
      Some(colspec)
    } else {
      Error!("unexpected", "&", None, None, "Extra alignment tab '&'");
      // DG: Mutability issue, should we do an alternative recovery?
      //     or change the call interface?
      //
      // let fallback_cell = Cell{align: Some(Align::Center),..Cell::default()};
      // current_row.add_column(fallback_cell);
      None
    }
  }

  pub fn last_column(&mut self) -> Option<&mut Cell> {
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
    self.rows.iter().filter(|row| !row.is_pseudo()).count()
  }

  pub fn current_column(&mut self) -> Option<&mut Cell> {
    self.current_row.and_then(|cw| self.rows.get_mut(cw).unwrap()
      .get_column_mut(self.current_column))
  }

  pub fn get_column(&mut self, n:usize) -> Option<&mut Cell> {
    // TODO: do we need an immutable variant? For now alias the mutable one
    self.get_column_mut(n)
  }

  pub fn get_column_mut(&mut self, n:usize) -> Option<&mut Cell> {
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
    self.normalize_alignment(state)?;
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
    let open_attrs = attrs;
    // TODO:
    // open_attrs.insert("cwidth", self.cwidth);
    // open_attrs.insert("cheight", self.cheight);
    // open_attrs.insert("cdepth", self.cdepth);
    // open_attrs.insert("rowheights", self.rowheights);
    // open_attrs.insert("columnwidths", self.columnwidths);
    let open_container_fn = &self.open_container;
    open_container_fn(document, open_attrs, state)?;

    for row in rows {
      let vpad_opt = row.get_padding().copied();
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
        let empty = cell.empty || dbg!(&cell.boxes).is_none() || cell.boxes.as_ref().unwrap().is_empty();
        let open_column_fn = &self.open_column;
        let mut cell_attrs = HashMap::default();
        if let Some(align) = cell.align {
          cell_attrs.insert(String::from("align"), align.name().to_owned());
        };
        if let Some(ref vattach) = cell.vattach {
          cell_attrs.insert(String::from("vattach"), vattach.clone());
        }
        // TODO: add to cell_attrs
        //  width => $$cell{width},
        if let Some(vpad) = vpad_opt {
          cell_attrs.insert(String::from("cssstyle"), s!("padding-bottom: {vpad}"));
        }
        //       (($$cell{colspan} || 1) != 1 ? (colspan  => $$cell{colspan})                         : ()),
        //       (($$cell{rowspan} || 1) != 1 ? (rowspan  => $$cell{rowspan})                         : ()),
        if !border.is_empty() { cell_attrs.insert(String::from("border"), border); }
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
  pub fn normalize_alignment(&mut self, state:&mut State) -> Result<()> {
    if self.is_normalized {
      return Ok(());
    }
    //======================================================================
    self.normalize_cell_sizes(state)?;
    self.normalize_mark_spans()?;
    self.normalize_prune_rows()?;
    self.normalize_prune_columns()?;
    self.normalize_sum_sizes()?;
    //======================================================================
    self.is_normalized = true;
    Ok(())
  }
  /// Compute (approximate) sizes of all cells
  pub fn normalize_cell_sizes(&mut self, state: &mut State) -> Result<()> {
    // Examines: boxes, align, vattach
    // Sets: cwidth, cheight, cdepth (per cell) & empty
    for row in &mut self.rows {
      // Do we need to account for any space in the $$row{before} or $$row{after}?
      for cell in row.get_columns_mut() {
        if let Some(boxes) = &cell.boxes {
          let (w, h, d) //, cw, ch, cd)
            = boxes.get_size(Some(stored_map!(
              "align" => cell.align.map(|a| a.char_code()), "width" => cell.width,
              "vattach" => cell.vattach.clone() )), state)?;
          dbg!(w);
          // Debug("CELL (" . join(',', map { $_ . "=" . ToString($$cell{$_}); } qw(align width vattach))
          //     . ") size " . showSize($w,  $h,  $d)
          //     . " csize " . showSize($cw, $ch, $cd)
          //     . " Boxes=" . ToString($boxes)) if $LaTeXML::DEBUG{halign} && $LaTeXML::DEBUG{size};
          // TODO: We can't do heights and depths yet
          let empty = w.value_of() < 1 || // h.value_of() < 1 || d.value_of() < 1 ||
            boxes.unlist_ref().iter().all(|tb| tb.get_property_bool("isSpace")) && !preserved_boxes(boxes);
          cell.width  = Some(w);
          cell.height = Some(h);
          cell.depth  = Some(d);
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
  // TODO:
  /// Mark any cells that are covered by rowspan or colspan
  pub fn normalize_mark_spans(&mut self) -> Result<()> {Ok(())}
  /// Scan for and remove empty rows
  /// but copying borders and adjusting rowspan's & colspan's appropriately.
  pub fn normalize_prune_rows(&mut self) -> Result<()> {
    // Examines: rowspan,rowspanned, border, pseudorow, empty
    // Sets: border, rowspan
    let preserve = self.is_math || self.properties.contains_key("preserve_structure");
    // First, do rows.
    let init_rows : Vec<_> = self.rows.drain(..).collect();
    let mut rows = init_rows.into_iter().peekable();
    let mut filtered = VecDeque::new();
    while let Some(row) = rows.next() {
      if row.get_columns().iter().any(|cell| !cell.empty) {    // Not empty! so keep it
        filtered.push_back(row);
      } else if let Some(next) = rows.peek_mut() {    // Remove empty row, but copy top border to NEXT row
        if preserve {
          filtered.push_back(row);
          continue;
        } // don't remove inner rows from math EXCEPT last row!!
        // let (mut pruneh, mut pruned) = (0, 0);
        for (j,col) in row.get_columns().iter().enumerate() {
          // TODO: add cheight and cdepth
          // if let Some(cheight) = col.cheight {
          //   let chv = cheight.value_of();
          //   if chv > pruneh { pruneh = chv; }
          // }
          // if let Some(cdepth) = col.cdepth {
          //   let cdv = cdepth.value_of();
          //   if cdv > pruned {
          //     pruned = cdv;
          //   }
          // }
            // TODO: add rowspanned
          // if !row.pseudorow && col.rowspanned.is_some() {
          //         $rows[$$col{rowspanned}]{columns}[$j]{rowspan}--; }    // Decrement rowspan of spanning column
          let mut converted_border = String::new();
          for border_c in col.border.chars() {
            match border_c {//  but convert to top
              't'| 'b' => converted_border.push('t'),
              'T' | 'B' => converted_border.push('T'),
              _ => {}
            };
          }
          if !converted_border.is_empty() {
            next.get_columns_mut()[j].border.push_str(&converted_border); // add to NEXT row
          }
        }
        // TODO:
        // This top_padding should be combined w/any extra rowspacing from \\[dim] !
        // let prune_both = pruneh + pruned;
        // if prune_both > 0 {
        //   next.top_padding = Some(Dimension::new(prune_both));
        // }    // And save padding.
      } else {    // Remove empty last row, but copy top border to bottom of prev.
        let mut prev_opt = filtered.back_mut();
        //     my $nc   = scalar(@{ $$row{columns} });
        // let (mut pruneh, mut pruned) = (0, 0);
        for (j,col) in row.get_columns().iter().enumerate() {
          // TODO:
          //  $pruneh = max($pruneh, $$col{cheight}->valueOf) if $$col{cheight};
          //  $pruned = max($pruned, $$col{cdepth}->valueOf)  if $$col{cdepth};
          //       if (!$$row{pseudorow} && defined $$col{rowspanned}) {
          //         $rows[$$col{rowspanned}]{columns}[$j]{rowspan}--; }    // Decrement rowspan of spanning column
          let mut converted_border = String::new();
          for c in col.border.chars() {
            match c {
              't' => converted_border.push('b'), // convert to bottom
              'T' => converted_border.push('B'), // convert to bottom
              _ => {}
            };
          }
          if let Some(ref mut prev) = prev_opt {
            let ccol = &mut prev.get_columns_mut()[j];
            // TODO: rowspanned
            //       if (defined $$ccol{rowspanned}) {                        // skip to spanning column if rowspanned!
            //         $ccol = $rows[$$ccol{rowspanned}]{columns}[$j]; }
            if !converted_border.is_empty() {
              ccol.border.push_str(&converted_border); // add to PREVIOUS row
            }
            // TODO:
            // let prune_both = pruneh + pruned;
            // if prune_both > 0 {    // And save padding.
            //   prev.bottom_padding = Some(Dimension::new(prune_both));
            // }
          }
        }
      }
    }
    self.rows = filtered;
    Ok(())}
  /// Scan for and remove empty columns
  /// but copying borders and adjusting rowspan's & colspan's appropriately.
  pub fn normalize_prune_columns(&mut self) -> Result<()> {
    let preserve = self.is_math || self.properties.contains_key("preserve_structure");
    // Now prune empty columns.
    if !preserve { // Don't remove empty columns from math.
      let mut nc   = 0;
      for row in self.rows.iter() {
        let n = row.get_columns().len();
        if n > nc {
          nc = n;
        }
      }
      for j in (0..nc).rev() { // Prune from RIGHT!
        let j_column_is_empty = self.rows.iter().all(|row|
          row.get_columns().get(j).map(|col| col.empty).unwrap_or(true));
        if j_column_is_empty {    // Empty!
          // let mut prunew = 0;
          for row in &mut self.rows {
            let mut new_border = String::new();
            if let Some(col) = row.get_columns().get(j) {
              // TODO: colspanned
              // if let Some(col_spanned) = col.colspanned {// Decrement colspan of spanning column
              //     if let Some(ref mut colspan) = &mut row.get_columns_mut()[col_spanned].colspan {
              //       *colspan -= 1;
              //     }
              // }
              // TODO: cwidth
              // if let Some(w) = col.cwidth {
              //   if w > prunew {
              //     prunew = w;
              //   }
              // }

              if j > 0 {
                // TODO:
                // if (my $jj = $$prev{colspanned}) {
                //   $prev = $$row{columns}[$jj]; }
                for c in col.border.chars() {
                  // mask all but left and right border
                  match c {
                    // convert to right
                    'l' | 'r' => { new_border.push('r'); },
                    // convert to right
                    'L' | 'R' => { new_border.push('R'); },
                    _ => {}
                  }
                }

                // TODO:
                // if (my @preserve = preservedBoxes($$col{boxes})) {    // Copy boxes over, in case side effects?
                //   $$prev{boxes} = LaTeXML::Core::List($$prev{boxes}
                //     ? ($$prev{boxes}->unlist, @preserve) : @preserve); }
              } else {
                for c in col.border.chars() {
                  // mask all but left and right border
                  match c {
                    // but convert to left
                    'l' | 'r' => { new_border.push('l'); },
                    // but convert to left
                    'L' | 'R' => { new_border.push('L'); },
                    _ => {}
                  }
                }

    //             $$next{border} .= $border;
    //             if (my @preserve = preservedBoxes($$col{boxes})) {    // Copy boxes over, in case side effects?
    //               $$next{boxes} = LaTeXML::Core::List($$col{boxes}
    //                 ? (@preserve, $$next{boxes}->unlist) : @preserve); }
              }
              // Now, remove the column
              row.get_columns_mut().remove(j);
            }
            // Changed: we need to first finish all work with "col" before we can mutably re-borrow
            // the "prev" column from the same row.
            if !new_border.is_empty() {
              if j > 0 {
                if let Some(prev) = row.get_columns_mut().get_mut(j-1) {
                  prev.border.push_str(&new_border);
                }
              } else {
                // next border case
                if let Some(next) = row.get_columns_mut().get_mut(1) {
                  // add to next row
                  next.border.push_str(&new_border);
                }
              }
            }
          }
          if j > 0 {    // If not 1st row, add right padding to previous column
    //         foreach my $row (@rows) {
    //           if (my $col = $$row{columns}[$j - 1]) {
    //             $$col{rpadding} = Dimension($prunew); } } }
    //       else {       // Else add left padding to (newly) first column
    //         foreach my $row (@rows) {    // And add the padding to previous column
    //           if (my $col = $$row{columns}[0]) {
    //             $$col{lpadding} = Dimension($prunew); } } }
          }
        }
      }
    }
    Ok(())}
  pub fn normalize_sum_sizes(&mut self) -> Result<()> {
    // let mut rowheights = Vec::new();
    // let mut colwidths  = Vec::new();
    // let mut colrights  = Vec::new();
    // let mut collefts   = Vec::new();
    // # Uses cell's cwidth,cheight,cdepth
    // # Computes net row & column sizes & positions
    // # add spacing between rows? Or only from \\[..] ?
    // my $strut = $self->getProperty('strut') || Dimension(0);
    // my $hs    = $strut->multiply(0.7);
    // my $ds    = $strut->multiply(0.3);
    // my @rows  = @{ $$self{rows} };
    // my $nrows = scalar(@rows);

    // for (my $i = 0 ; $i < $nrows ; $i++) {
    //   my $row   = $rows[$i];
    //   my @cols  = @{ $$row{columns} };
    //   my $ncols = scalar(@cols);
    //   if (my $short = $ncols - scalar(@colwidths)) {    # Extend column arrays, if needed
    //     push(@colwidths, map { 0 } 1 .. $short);
    //     push(@collefts,  map { 0 } 1 .. $short);
    //     push(@colrights, map { 0 } 1 .. $short); }
    //   my ($rowh, $rowd) = (0, 0);
    //   my ($rowt, $rowb) = (($$row{tpadding} ? $$row{tpadding}->valueOf : 0),
    //     ($$row{bpadding} ? $$row{bpadding}->valueOf : 0));
    //   for (my $j = 0 ; $j < $ncols ; $j++) {
    //     my $cell = $cols[$j];
    //     next if $$cell{skipped};
    //     next unless $$cell{boxes};
    //     my $w = $$cell{cwidth};
    //     my $h = $$cell{cheight};
    //     my $d = $$cell{cdepth};
    //     my $t = $$cell{tpadding};
    //     my $b = $$cell{bpadding};
    //     my $r = $$cell{rpadding};
    //     my $l = $$cell{lpadding};

    //     if (($$cell{colspan} || 1) == 1) {
    //       $colwidths[$j] = max($colwidths[$j], $w->valueOf) if $w;
    //       $collefts[$j]  = max($collefts[$j],  $l->valueOf) if $l;
    //       $colrights[$j] = max($colrights[$j], $r->valueOf) if $r; }
    //     if (($$cell{rowspan} || 1) == 1) {
    //       $rowh = max($rowh, $h->valueOf) if $h;
    //       $rowd = max($rowd, $d->valueOf) if $d;
    //       $rowt = max($rowt, $t->valueOf) if $t;
    //       $rowb = max($rowb, $b->valueOf) if $b; }
    //     else { }    # Ditto spanned rows
    //   }
    //   $$row{cheight}  = Dimension($rowh)->larger($hs);
    //   $$row{cdepth}   = Dimension($rowd)->larger($ds);
    //   $$row{tpadding} = Dimension($rowt);
    //   $$row{bpadding} = Dimension($rowb);
    //   # NOTE: Should be storing column widths to; individually, as well as per-column!
    //   push(@rowheights, $rowh + $rowd); }    # somehow our heights are way too short????
    // ## Now compute the positions
    // my @rowpos = ();
    // my @colpos = ();
    // my $y      = 0;
    // # Row & column positions: left,top
    // for (my $i = 0 ; $i < scalar(@rowheights) ; $i++) {
    //   my $row = $rows[$i];
    //   $y += $$row{tpadding}->valueOf if $$row{tpadding};
    //   $rowpos[$i] = Dimension($y);
    //   $y += $$row{cheight}->valueOf  if $$row{cheight};
    //   $y += $$row{cdepth}->valueOf   if $$row{cdepth};
    //   $y += $$row{bpadding}->valueOf if $$row{bpadding}; }
    // my $x = 0;
    // for (my $j = 0 ; $j < scalar(@colwidths) ; $j++) {
    //   $x += $collefts[$j];
    //   $colpos[$j] = Dimension($x);
    //   $x += $colwidths[$j];
    //   $x += $colrights[$j]; }
    // $$self{cwidth}       = Dimension($x);
    // $$self{cheight}      = Dimension($y);    # or account for vertical position of array as a whole?
    // $$self{cdepth}       = Dimension(0);
    // @colwidths           = map { Dimension($_); } @colwidths;
    // @rowheights          = map { Dimension($_); } @rowheights;
    // $$self{columnwidths} = [@colwidths];
    // $$self{rowheights}   = [@rowheights];

    // for (my $i = 0 ; $i < scalar(@rowheights) ; $i++) {
    //   my $row   = $rows[$i];
    //   my @cols  = @{ $$row{columns} };
    //   my $ncols = scalar(@cols);
    //   $$row{x}      = $colpos[0]; $$row{y} = $rowpos[$i];
    //   $$row{cwidth} = Dimension($x);
    //   for (my $j = 0 ; $j < $ncols ; $j++) {
    //     my $cell = $cols[$j];
    //     my $colx = $colpos[$j];
    //     my $a    = $$cell{align} || 'left';
    //     # Adjust position according to alignment
    //     if ($colwidths[$j] && $$cell{cwidth} && ($a ne 'left')) {    # If these are defined
    //       my $dx = $colwidths[$j]->subtract($$cell{cwidth});
    //       if    ($a eq 'center') { $colx = $colx->add($dx->multiply(0.5)); }
    //       elsif ($a eq 'right')  { $colx = $colx->add($dx); } }
    //     $$cell{x} = $colx;
    //     $$cell{y} = $rowpos[$i];
    //     Debug("CELL[$j,$i] " . showSize($$cell{cwidth}, $$cell{cheight}, $$cell{cdepth})
    //         . " @ " . ToString($$cell{x}) . "," . ToString($$cell{y})
    //         . " w/ " . join(',', map { $_ . '=' . ToString($$cell{$_}); }
    //           (qw(align vattach skipped colspan rowspan))))
    //       if $LaTeXML::DEBUG{halign} && $LaTeXML::DEBUG{size};
    // } }

    Ok(())
  }


  pub fn compute_size(
    &mut self,
    _options: HashMap<String, Stored>,
    state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    self.normalize_alignment(state)?;
    Ok((self.cwidth.unwrap(), self.cheight.unwrap(), self.cdepth.unwrap()))
  }

  pub fn get_properties_mut(&mut self) -> &mut HashMap<String,Stored> {
    &mut self.properties
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
    write!(f, "Alignment{{template:{:?}, properties:{:?}, rows:{:?} }}", self.template, self.properties, self.rows)
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


fn preserved_boxes(boxes: &Digested) -> bool {
  boxes.unlist_ref().iter().any(|tb| tb.get_property_bool("alignmentPreserve"))
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
  state.local_build_template(Template::default());
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
    repeated: vec![Cell {
      before: Some(Tokens!(T_CS!("\\hfil"))),
      after: Some(Tokens!(T_CS!("\\hfil"))),
      ..Cell::default()
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
  classify_alignment_rows(document, alignment, state);

  {
    let mut rows = collect_alignment_rows(alignment);
    if rows.is_empty() {
      return Ok(());
    }
    alignment_characterize_lines(document, Axis::Row, false, rows.as_mut_slice(), state)?;
  }
  // Flip the rows around to produce a column view.
  {
    let mut cols = collect_alignment_columns(alignment);
    if cols.is_empty() {
      return Ok(());
    }
    // This usually does something unpleasant
    alignment_characterize_lines(document, Axis::Column, false,  cols.as_mut_slice(), state)?;
  }

  // Did we go overboard?
  let rows = collect_alignment_rows(alignment);
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
  // dbg!((n_h, n_d));
//   Debug("$n{h} header, $n{d} data cells") if $LaTeXML::DEBUG{alignment};
  if n_d == 1 { // Or any other heuristic?
    n_h = 0;
    for r in rows {
      for c in r {
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
    rows.append(&mut heads);
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
fn classify_alignment_rows(document: &mut Document, alignment: &mut Alignment, state: &mut State) {
  let mut ncols = 0;
  for arow in &mut alignment.rows {
    let n = arow.get_columns().len();
    if n > ncols {
      ncols = n;
    }
  }
  let (mut h, mut v) = (false,false);
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
          'l' | 'L' => border_left+=1,
          'r' | 'R' => border_right+=1,
          't' | 'T' => border_top +=1,
          'b' | 'B' => border_bottom +=1,
          _ => {}// spaces etc.
        }
      }
      // Note: once h and v are set as true on any row, they remain globally true.
      if (border_top > 0) || (border_bottom > 0) {
        h = true;
      }
      if (border_right > 0) || (border_left > 0) {
        v = true;
      }
      col.border_top = Some(border_top);
      col.border_bottom = Some(border_bottom);
      col.border_left = Some(border_left);
      col.border_right = Some(border_right);
    }
    // pad the columns out.
    let to_pad = ncols - this_row_len;
    if to_pad > 0 {
      for _ in 0..to_pad {
        let col = Cell {
          align: Some(Align::Center),
          cell_type: Some('d'),
          content_class: Some(ColumnSpec::Empty),
          content_length: Some(0),
          .. Cell::default()
        };
        cols.push(col);
      }
    }
  }
  // DG: cache assignments, and execute in post-loop, so that we can avoid indexing arithmetic
  let mut outer_border_right_assignments = Vec::new();
  let mut outer_border_bottom_assignments = Vec::new();
  // copy the characterizations to spanned cells
  for r in 0..alignment.rows.len() {
    let row = &mut alignment.rows[r];
    let cols = row.get_columns_mut();
    for c in 0 .. cols.len() {
      let rs = cols[c].rowspan.unwrap_or(1);
      let cs = cols[c].colspan.unwrap_or(1);
      let ca = cols[c].align;
      let cc = cols[c].content_class;
      let cl = cols[c].content_length;
      let rb = cols[c].border_right;

      cols[c].border_right = Some(0);
      let bb = cols[c].border_bottom;
      cols[c].border_bottom = Some(0);
      for row_reach in cols.iter_mut().take(c+cs).skip(c+1) {
        row_reach.align          = ca;
        row_reach.content_class  = cc;
        row_reach.content_length = cl;
      }
      // TODO:
      // for irow_idx in r+1 .. r+rs {
      //   let mut irow = &mut alignment.rows[irow_idx];
      //   let mut icols = irow.get_columns_mut();
      //   for icol_idx in c .. c+cs {
      //     let icol = &mut icols[icol_idx];
      //     icol.align          = ca;
      //     icol.content_class  = cc;
      //     icol.content_length = cl;
      //   }
      // }

      // move the outer borders
      for sr in 0..rs {
        outer_border_right_assignments.push((r+sr, c+cs-1, rb));
      }
      for sc in 0 .. cs {
        outer_border_bottom_assignments.push((r+rs-1, c+sc, bb));
      }
    }
  }
  // Apply the collected outer border assignments
  for (row_idx, col_idx, value) in outer_border_right_assignments.into_iter() {
    alignment.rows[row_idx].get_columns_mut()[col_idx].border_right = value;
  }
  for (row_idx, col_idx, value) in outer_border_bottom_assignments.into_iter() {
    alignment.rows[row_idx].get_columns_mut()[col_idx].border_bottom = value;
  }
  // Now, do some border massaging...
  for row in alignment.rows.iter_mut() {
    let cols = row.get_columns_mut();
    cols[0].border_left = Some(if v { 1 } else { 0 });
    if ncols > 1 {
      if cols[1].border_left.unwrap_or(0) > 0 {
        cols[0].border_right = cols[1].border_left;
      }
      if cols[ncols - 2].border_right.unwrap_or(0) > 0 {
        cols[ncols - 1].border_left = cols[ncols - 2].border_right;
      }
    }
    cols[ncols - 1].border_right = Some(if v { 1 } else { 0 });
  }
  let nrows = alignment.rows.len();
  for c in 0 .. ncols {
    alignment.rows[0].get_columns_mut()[c].border_top = Some(if h {1}else{0});
    if nrows > 1 {
      if let Some(bt) = alignment.rows[1].get_columns_mut()[c].border_top {
        if bt > 0 { // only set if border is inked
          alignment.rows[0].get_columns_mut()[c].border_bottom = Some(bt);
        }
      }
      if let Some(bb) = alignment.rows[nrows - 2].get_columns_mut()[c].border_bottom {
        if bb > 0 { // only set if border is inked
          alignment.rows[nrows - 1].get_columns_mut()[c].border_top = Some(bb);
        }
      }
    }
    alignment.rows[nrows - 1].get_columns_mut()[c].border_bottom = Some(if h {1} else{0});
  }
  // the constant array access *HAS* to be inefficient, but how do we avoid it without encountering
  // objections from the Rust compiler? Mutability conflicts galore here if any &mut lives long enough.
  for r in 1 .. nrows-1 {
    for c in 1 .. ncols-1 {
      if let Some(bb) = alignment.rows[r - 1].get_columns_mut()[c].border_bottom {
        if bb > 0 { // only set if border is inked
          alignment.rows[r].get_columns_mut()[c].border_top = Some(bb);
        }
      }
      if let Some(bt) = alignment.rows[r + 1].get_columns_mut()[c].border_top {
        if bt > 0 { // only set if border is inked
          alignment.rows[r].get_columns_mut()[c].border_bottom = Some(bt);
        }
      }
      if let Some(br) = alignment.rows[r].get_columns_mut()[c-1].border_right {
        if br > 0 { // only set if border is inked
          alignment.rows[r].get_columns_mut()[c].border_left = Some(br);
        }
      }
      if let Some(bl) = alignment.rows[r].get_columns_mut()[c + 1].border_left {
        if bl > 0 { // only set if border is inked
          alignment.rows[r].get_columns_mut()[c].border_right = Some(bl);
        }
      }
    }
  }
  // debug info
  eprintln!("Cell characterizations:");
  for (row_index,row) in alignment.rows.iter().enumerate() {
    for (col_index, cell) in row.get_columns().iter().enumerate() {
      eprintln!("[{row_index},{col_index}]=>{}{}{} {} {} => {}{}{}{}",
        cell.cell_type.as_ref().unwrap_or(&'?'),
        cell.align.map(|a| a.char_code()).unwrap_or(' '),
        cell.content_class.map(|a| a.to_string()).unwrap_or_else(|| String::from("?")),
        cell.content_length.unwrap_or(0),
        cell.border,
        if cell.border_top.unwrap_or(0) > 0 { "t" } else { "" },
        if cell.border_right.unwrap_or(0) > 0  { "r" } else { "" },
        if cell.border_bottom.unwrap_or(0) > 0 { "b" } else { "" },
        if cell.border_left.unwrap_or(0) > 0 { "l" } else {""}
      );
    }
  }

}

fn collect_alignment_rows(alignment: &mut Alignment) -> Vec<Vec<&mut Cell>> {
  alignment.rows.iter_mut().map(|x| x.get_columns_mut().iter_mut()
    .collect()).collect()
}

fn collect_alignment_columns(alignment: &mut Alignment) -> Vec<Vec<&mut Cell>> {
  let mut columns = Vec::new();
  let mut row_cells : Vec<_> = alignment.rows.iter_mut().map(|r|
    r.get_columns_mut().iter_mut()).collect();
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
  let content = dbg!(xcell.get_content());
  let mut inferred_classes: Vec<ColumnSpec>   = Vec::new();
  if !content.is_empty() && content.chars().all(|c| c.is_whitespace() || c.is_numeric()) {
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

  // check if we have alternating math-and-text or text-and-math (only if 2+ classes)
  if inferred_classes.len() > 1 {
    let mut alt_peekable = inferred_classes.iter().peekable();
    let mut is_alternating = true;
    while let Some(c) = alt_peekable.next() {
      match c {
        ColumnSpec::Math | ColumnSpec::Integer =>
          if let Some(peek) = alt_peekable.peek() {
            if !matches!(peek, ColumnSpec::Text) {
              is_alternating = false;
              break;
            }
          },
        ColumnSpec::Text =>
          if let Some(peek) = alt_peekable.peek() {
            if !matches!(peek, ColumnSpec::Math | ColumnSpec::Integer) {
              is_alternating = false;
              break;
            }
          },
        _ => {
          is_alternating = false;
          break;
        }
      }
    }
    if is_alternating {
      inferred_classes = vec![ColumnSpec::MathAltText];
    }
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

fn alignment_characterize_lines(document:&mut Document, axis:Axis, reversed:bool, lines: &mut [Vec<&mut Cell>], state:&State) -> Result<()> {
  let n = lines.len();
  if n<2 {
    return Ok(());
  }
  // eprintln!("Characterizing {n} {}", if axis == Axis::Row {"rows"} else {"columns"});

  // Establish a scale of differences for the table.
  let (mut max_diff, mut min_diff, _avg_diff) = (0.0, 99999999.0, 0.0);
  for l in 0..n-1 {
    let d = alignment_compare(axis, true, reversed, l, l + 1, lines);
    // avg_diff += d;
    if d > max_diff {
      max_diff = d;
    }
    if d < min_diff {
      min_diff = d;
    }
  }
  // avg_diff = avg_diff / (n - 1) as f64;
  if max_diff < 0.05 { // virtually no differences.
    // eprintln!("Lines are almost identical => Fail");
    return Ok(());
  }
  if (n > 2) && ((max_diff - min_diff) < max_diff * 0.5) { // differences too similar to establish pattern
    // eprintln!("Differences between lines are almost identical => Fail");
    return Ok(());
  }
  let tab_threshold = min_diff + 0.3 * (max_diff - min_diff);

  // eprintln!("Differences {min_diff} -- {max_diff} => threshold = {tab_threshold}");
  // Find the first hump in differences. These are candidates for header lines.
  // eprintln!("Scanning for headers");
  let (minh, mut maxh) = (1, 1);
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
  // eprintln!("Found from {minh}--{maxh} potential headers");

  let nn = lines[0].len() - 1;
  // The sets of lines 1--$minh, .. 1--$maxh are potential headers.
  for nh in (minh..=maxh).rev() {
    // Check whether the set 1..$nh is plausable.
    let heads = alignment_test_headers(nh, tab_threshold, axis, lines);
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
fn alignment_test_headers(nhead:usize, tab_threshold:f64, axis: Axis, lines:&mut [Vec<&mut Cell>]) -> Vec<usize> {
  // eprintln!("Testing {nhead} headers with threshold {tab_threshold}");
  let mut heads : Vec<usize> = (0 .. nhead).collect(); // The indices of heading lines.
  let mut head_length = alignment_max_content_length(0, 0, nhead - 1, lines);
  let mut next_line = nhead; // Start from the end of the proposed headings.

  // Watch out for the assumed header being really data that is a repeated pattern.
  let nrep = lines.len() / nhead;
  if nhead > 1 {
    //   Debug("Check for apparent header repeated $nrep times") if $LaTeXML::DEBUG{alignment};
    let mut matched = true;
      for r in 1..nrep {
        matched = matched && alignment_match_head(0, r * nhead, nhead, tab_threshold, axis, lines)>0;
      }
  //   Debug("Repeated headers: " . ($matched ? "Matched=> Fail" : "Nomatch => Succeed"))
  //     if $LaTeXML::DEBUG{alignment};
    if matched {
      return Vec::new()
    }
  }

  // And find a following grouping of data lines.
  let ndata = alignment_skip_data(next_line, tab_threshold, axis, lines);
  if ndata < nhead {// ???? Well, maybe if _really_ convincing???
    return Vec::new();
  }
  if (ndata < nhead) && (ndata < 2) {
    return Vec::new();
  }
  // Check that the content of the headers isn't dramatically larger than the content in the data
  let mut data_length = alignment_max_content_length(0, next_line, next_line + ndata - 1, lines);
  next_line += ndata;

  let mut nd;
  // If there are more lines, they should match either the previous data block, or the head/data pattern.
  while next_line < lines.len() {
    // First try to match a repeat of the 1st data block;
    // This would be the case when groups of data have borders around them.
    // Could want to match a variable number of datalines, but they should be similar!!!??!?!?
    nd = if ndata > 1 {
      alignment_match_data(nhead, next_line, ndata, tab_threshold, axis, lines)
    } else { 0 };
    if nd > 0 {
      data_length = alignment_max_content_length(data_length, next_line, next_line + nd - 1, lines);
      next_line += nd;
    }
    // Else, try to match the first header block; less common.
    else if alignment_match_head(0, next_line, nhead, tab_threshold, axis, lines) > 0 {
      for idx in next_line .. next_line + nhead {
        heads.push(idx);
      }
      head_length = alignment_max_content_length(head_length, next_line, next_line + nhead -1, lines);
      next_line += nhead;
      nd = alignment_match_data(nhead, next_line, ndata, tab_threshold, axis, lines);
      if nd == 0 {
        return Vec::new();
      }
      data_length = alignment_max_content_length(data_length, next_line, next_line + nd - 1, lines);
      next_line += nd;
    }
    else { return Vec::new(); }
  }
  // Header content seems too large relative to data?
  // eprintln!("header content = {head_length}; data content = {data_length}");
  if (head_length > 10) && (head_length > 4*data_length) {
  //   Debug("header content too much longer than data content")
  //     if $LaTeXML::DEBUG{alignment};
    return Vec::new();
  }
  // Or if a header cell has "large" content?
  if head_length >= 1000 { // Or if a header cell has "large" content?
  //   Debug("header content too large")
  //     if $LaTeXML::DEBUG{alignment};
    return Vec::new();
  }

  // Debug("Succeeded with $nhead headers") if $LaTeXML::DEBUG{alignment};
  heads
}

fn alignment_match_head(p1:usize, p2:usize, nhead:usize, tab_threshold: f64, axis: Axis, tablines: &mut [Vec<&mut Cell>]) -> usize {
  let nh = alignment_match_lines(p1, p2, nhead, tab_threshold, axis, tablines);
  let ok = nhead == nh;
  // Debug("Matched $nh header lines => " . ($ok ? "Succeed" : "Failed")) if $LaTeXML::DEBUG{alignment};
  if ok { nhead } else { 0 }
}

fn alignment_match_data(p1:usize, p2:usize, n:usize, tab_threshold:f64, axis: Axis, tablines: &mut [Vec<&mut Cell>]) -> usize {
  let nd = alignment_match_lines(p1, p2, n, tab_threshold, axis, tablines);
  let ok = (nd as f64 * 1.0) / n as f64 > 0.66;
//   Debug("Matched $nd data lines => " . ($ok ? "Succeed" : "Failed"))
//     if $LaTeXML::DEBUG{alignment};
  if ok { nd } else { 0 }
}

// Match the $n lines starting at $i2 to those starting at $i1.
fn alignment_match_lines(p1:usize, p2:usize, n:usize, tab_threshold:f64, axis: Axis, tablines: &mut [Vec<&mut Cell>]) -> usize {
  let max_n = tablines.len();
  for i in 0..n {
    if (p1 + i >= max_n) || (p2 + i >= max_n)
      || alignment_compare(axis, false, false, p1 + i, p2 + i, tablines) >= tab_threshold {
      return i;
    }
  }
  n
}

/// Skip through a block of lines starting at $i that appear to be data, returning the number of lines.
/// We'll assume the 1st line is data, compare it to following lines,
/// but also accept `continuation' data lines.
fn alignment_skip_data(i:usize, tab_threshold:f64, axis:Axis, tablines: &mut [Vec<&mut Cell>]) -> usize {
  let tab_lines_length = tablines.len();
  if i >= tab_lines_length {
    return 0;
  }
  // eprintln!("Scanning for data");
  let mut n = 1;
  while i+n < tab_lines_length {
    if alignment_compare(axis, true, false, i + n - 1, i + n, tablines) >= tab_threshold
      && (n < 2 || (tablines[i + n].iter().filter(|c|
        matches!(c.content_class, Some(ColumnSpec::Empty))).count() as f64 <= 0.4 * tablines[0].len() as f64))
    {
      break;
    }
    // Accept an outlying `continuation line' as data, if mostly empty
    n+=1;
  }
  // eprintln!("Found {n} data lines at {i}");
  if n >= MIN_ALIGNMENT_DATA_LINES  { n } else { 0 }
}


/// Return the maximum "content length" for lines from $from to $to.
fn alignment_max_content_length(mut length: usize, from:usize, to:usize, tablines: &mut [Vec<&mut Cell>]) -> usize {
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
fn alignment_compare(axis: Axis, for_adjacency:bool, reversed:bool, p1:usize, p2:usize, lines: &mut [Vec<&mut Cell>]) -> f64 {
  let max_guard = lines.len();
  if p1>=max_guard || p2 >= max_guard {
    return 0.0;
  }
  let line1 = &lines[p1];
  let line2 = &lines[p2];
  if line1.is_empty() && line2.is_empty() {
    return 0.0;
  } else if line1.is_empty() || line2.is_empty() {
    return 99999.0;
  }
  let ncells = line1.len();
  let mut diff   = 0.0;

  for (cell1,cell2) in line1.iter().zip(line2.iter()) {
    // Annoying test avoids warnings if cells inconsistent; likely due to incorrect row/col spans
    if cell1.content_class.is_none() || cell2.content_class.is_none() ||
       cell1.border_left.is_none() || cell2.border_left.is_none() ||
       cell1.border_right.is_none() || cell2.border_right.is_none() ||
       cell1.border_bottom.is_none() || cell2.border_bottom.is_none() ||
       cell1.border_top.is_none() || cell2.border_top.is_none() {
      continue;
    }
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
        if b1p>0 && (border1_pedge != border2_pedge) {
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
  // eprintln!("alignment_compare: {p1} - {p2} => {diff};");
  // Debug("$p1-$p2 => $diff; ") if $LaTeXML::DEBUG{alignment};
  diff
}
