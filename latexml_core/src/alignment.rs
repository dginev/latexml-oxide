//! # Representation of aligned structures
//! An "Alignment" is an array/tabular construct as:
//!  `<tabular><tr><td>...`
//! or, for math mode
//!   `<XMArray><XMRow><XMCell>...`
//! (where initially, each XMCell will contain an XMArg to indicate
//! individual parsing of each cell's content is desired)
//!
//! An Alignment object is a sort of fake Whatsit;
//! It takes some magic to sneak it into the Digestion stream
//! (see TeX.pool \lx@begin@alignment), but it needs to be created
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
pub mod cell;
mod normalize;
pub mod template;

use libxml::tree::{Node, NodeType};
use once_cell::sync::Lazy;

use self::{
  cell::Cell,
  normalize::*,
  template::{Align, Axis, BorderSpec, ColumnSpec, Row, Template, TemplateConfig},
};
use crate::{
  BoxOps,
  common::{
    arena::SymHashMap, dimension::Dimension, error::*, numeric_ops::NumericOps, object::Object,
  },
  digested::Digested,
  document::{Document, get_node_qname, with_node_qname},
  gullet::{self, ExpansionLevel},
  mouth::Mouth,
  state::*,
  stomach::*,
  token::Catcode,
  tokens::Tokens,
};

/// token-locators: source span of an alignment cell (its content's locator).
/// `tabular`/`tr`/`td` are opened before their content's `box_to_absorb` is set,
/// so the absorb explicitly stamps each with its cell/row/table span via
/// `Document::set_current_box_locator`. See docs/SOURCE_PROVENANCE.md §3.1.3.
#[cfg(feature = "token-locators")]
fn cell_loc(cell: &Cell) -> Option<crate::common::locator::Locator> {
  cell
    .boxes
    .as_ref()
    .and_then(|b| b.get_locator())
    .filter(|l| l.from_line != 0)
}

/// Union (first `from` → last `to`) of a row's cell spans.
#[cfg(feature = "token-locators")]
fn row_span(row: &Row) -> Option<crate::common::locator::Locator> {
  row
    .get_columns()
    .iter()
    .filter_map(cell_loc)
    .reduce(|a, b| crate::common::locator::Locator::new_range(a, b).unwrap_or(a))
}
use std::{
  borrow::Cow,
  collections::VecDeque,
  fmt::{self, Debug, Display},
  rc::Rc,
};

use regex::Regex;
use rustc_hash::FxHashMap as HashMap;

//DebuggableFeature('alignment', "Debug guessing headers of alignments/tables");
pub type OpenContainerFn =
  Rc<dyn Fn(&mut Document, HashMap<String, String>) -> Result<Option<Node>>>;
pub type CloseContainerFn = Rc<dyn Fn(&mut Document) -> Result<Option<Node>>>;
pub type OpenRowFn = Rc<dyn Fn(&mut Document, HashMap<String, Stored>) -> Result<()>>;
pub type CloseRowFn = Rc<dyn Fn(&mut Document) -> Result<Option<Node>>>;
pub type OpenColumnFn = Rc<dyn Fn(&mut Document, HashMap<String, String>) -> Result<Option<Node>>>;
pub type CloseColumnFn = Rc<dyn Fn(&mut Document) -> Result<Option<Node>>>;

static SINGLE_PUNCT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*[\.,;]\s*$").unwrap());

pub struct AlignmentConfig {
  pub template:        Option<Template>,
  pub open_container:  OpenContainerFn,
  pub close_container: CloseContainerFn,
  pub open_row:        OpenRowFn,
  pub close_row:       CloseRowFn,
  pub open_column:     OpenColumnFn,
  pub close_column:    CloseColumnFn,
  pub properties:      SymHashMap<Stored>,
  pub xml_attributes:  HashMap<String, String>,
  pub is_math:         bool,
}

#[derive(Clone)]
pub struct Alignment {
  in_column:         bool,
  in_row:            bool,
  in_tabular_head:   bool,
  is_math:           bool,
  is_normalized:     bool,
  /// True for \halign templates
  pub is_halign:     bool,
  current_column:    usize,
  current_row:       Option<usize>,
  reversion:         Option<Tokens>,
  content_reversion: Option<Tokens>,
  rows:              VecDeque<Row>,
  properties:        SymHashMap<Stored>,
  xml_attributes:    HashMap<String, String>,
  template:          Template,
  open_container:    OpenContainerFn,
  close_container:   CloseContainerFn,
  open_row:          OpenRowFn,
  close_row:         CloseRowFn,
  open_column:       OpenColumnFn,
  close_column:      CloseColumnFn,
  cached_width:      Option<Dimension>,
  cached_height:     Option<Dimension>,
  cached_depth:      Option<Dimension>,
  column_widths:     Vec<Dimension>,
  row_heights:       Vec<Dimension>,
  row_depths:        Vec<Dimension>,
  // Longtable: stored head/foot rows for reinsertion
  pub head_rows:     Vec<Row>,
  pub foot_rows:     Vec<Row>,
}
impl Alignment {
  /// Create a new Alignment.
  /// `config` can contain:
  ///   - template: an Alignment::Template object
  ///   - openContainer: creates the container element with given attributes
  ///   - closeContainer = sub($doc); closes the container
  ///   - openRow        = sub($doc,%attrib); creates the row element with given attributes
  ///   - closeRow       = closes the row
  ///   - openColumn     = sub($doc,%attrib); creates the column element with given attributes
  ///   - closeColumn    = closes the column
  ///   - properties     = hashmap containing extra attributes for the container element.
  ///   - xml_attributes = hashmap containing attributes for the main XML node
  pub fn new(config: AlignmentConfig) -> Self {
    let template = config.template.unwrap_or_default();
    // Perl Alignment.pm: Copy width/height/depth from XML attributes to main properties,
    // but REMOVE them from XML attributes so they don't appear on the element.
    let mut xml_attributes = config.xml_attributes;
    for key in ["width", "height", "depth"] {
      xml_attributes.remove(key);
    }
    Alignment {
      template,
      current_row: None,
      reversion: None,
      content_reversion: None,
      cached_width: None,
      cached_height: None,
      cached_depth: None,
      open_container: config.open_container,
      close_container: config.close_container,
      open_row: config.open_row,
      close_row: config.close_row,
      open_column: config.open_column,
      close_column: config.close_column,
      current_column: 0,
      is_math: config.is_math,
      in_row: false,
      in_column: false,
      in_tabular_head: false,
      is_normalized: false,
      is_halign: false,
      properties: config.properties,
      xml_attributes,
      rows: VecDeque::new(),
      column_widths: Vec::new(),
      row_heights: Vec::new(),
      row_depths: Vec::new(),
      head_rows: Vec::new(),
      foot_rows: Vec::new(),
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
  pub fn get_cached_height(&self) -> Option<Dimension> { self.cached_height }
  pub fn get_cached_depth(&self) -> Option<Dimension> { self.cached_depth }
  pub fn get_row_heights(&self) -> &[Dimension] { &self.row_heights }
  pub fn get_column_widths(&self) -> &[Dimension] { &self.column_widths }
  /// Run normalization (cell sizes, spans, pruning, positioning).
  /// Perl: $alignment->normalizeAlignment
  pub fn normalize(&mut self) -> Result<()> { normalize_alignment(self) }

  pub fn add_line(&mut self, border: &str, cols: Vec<usize>) {
    if let Some(row_idx) = self.current_row {
      let Some(row) = self.rows.get_mut(row_idx) else {
        return;
      };
      self.current_column = 1;
      if !cols.is_empty() {
        // Perl Alignment.pm:128-130 — `$row->column($c)` returns undef
        // for out-of-range column index and autovivifies a discarded
        // temp hash, so the assignment silently no-ops. Match that
        // here: skip indices that don't map to a real column instead
        // of panicking on `.unwrap()`. Witness: 0708.2784 with a
        // `\hline`/`\cline`-style line referencing a column past the
        // tabular's column count.
        for c in cols {
          if let Some(colspec) = row.get_column_mut(c) {
            colspec.border.push_str(border);
          }
        }
      } else {
        for colspec in row.get_columns_mut() {
          colspec.border.push_str(border)
        }
      }
    }
  }

  pub fn next_column(&mut self) -> Result<Option<&mut Cell>> {
    if self.current_row.is_none() {
      return Ok(None);
    }
    self.current_column += 1;
    let current_row = self.rows.get_mut(self.current_row.unwrap()).unwrap();
    if current_row.get_column_mut(self.current_column).is_some() {
      Ok(current_row.get_column_mut(self.current_column))
    } else {
      // Perl: Error then add fallback column with align=center
      Error!("unexpected", "&", "Extra alignment tab '&'");
      let fallback = Cell {
        align: Some(Align::Center),
        ..Cell::default()
      };
      current_row.add_column(fallback);
      Ok(current_row.get_column_mut(self.current_column))
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

  pub fn current_column_number(&self) -> usize { self.current_column }

  /// Set a property on the current row (for attributes like backgroundcolor from \rowcolor)
  pub fn set_row_property(&mut self, key: &str, value: String) {
    if let Some(row_idx) = self.current_row
      && let Some(row) = self.rows.get_mut(row_idx)
    {
      row.properties.insert(key.to_string(), Stored::from(value));
    }
  }

  pub fn current_row_number(&self) -> usize {
    self.rows.iter().filter(|row| !row.is_pseudo()).count()
  }

  pub fn current_column(&mut self) -> Option<&mut Cell> {
    self
      .current_row
      .and_then(|cw| self.rows.get_mut(cw)?.get_column_mut(self.current_column))
  }

  pub fn get_column(&mut self, n: usize) -> Option<&mut Cell> {
    // TODO: do we need an immutable variant? For now alias the mutable one
    self.get_column_mut(n)
  }

  pub fn get_column_mut(&mut self, n: usize) -> Option<&mut Cell> {
    self
      .current_row
      .and_then(|cw| self.rows.get_mut(cw)?.get_column_mut(n))
  }

  // Ugh... these take boxes; adding before/after columns takes tokens!
  pub fn add_before_row(&mut self, boxes: Vec<Digested>) {
    if let Some(cw) = self.current_row
      && let Some(current_row) = self.rows.get_mut(cw)
    {
      current_row.before.extend(boxes);
    }
  }

  pub fn add_after_row(&mut self, boxes: Vec<Digested>) {
    if let Some(cw) = self.current_row
      && let Some(current_row) = self.rows.get_mut(cw)
    {
      current_row.after.extend(boxes);
    }
  }

  pub fn omit_column(&mut self) {
    if let Some(column) = self.current_column() {
      column.omitted = true;
    }
  }

  pub fn omit_next_column(&mut self) {
    if let Some(cw) = self.current_row
      && let Some(row) = self.rows.get_mut(cw)
      && let Some(column) = row.get_column_mut(self.current_column + 1)
    {
      column.omitted = true;
    }
  }

  pub fn get_column_before(&mut self) -> Tokens {
    if let Some(column) = self.current_column() {
      if !column.omitted {
        Tokens!(
          T_CS!("\\lx@alignment@column@before"),
          column.before.clone().unwrap_or_default().unlist()
        )
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
        // Possible \lx@column@trimright ??? (if LaTeX style???)
        Tokens!(
          column.after.clone().unwrap_or_default().unlist(),
          T_CS!("\\lx@alignment@column@after")
        )
      } else {
        Tokens!()
      }
    } else {
      Tokens!()
    }
  }

  pub fn revert(&self) -> Result<Tokens> { Ok(self.reversion.clone().unwrap_or_default()) }

  pub fn set_reversion(&mut self, rev: Tokens) { self.reversion = Some(rev); }
  pub fn set_content_reversion(&mut self, rev: Tokens) { self.content_reversion = Some(rev); }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Support for building an alignment's Rows & Columns
  pub fn is_in_row(&self) -> bool { self.in_row }
  pub fn is_in_column(&self) -> bool { self.in_column }
  pub fn start_row(&mut self, pseudorow: bool) -> Result<()> {
    self.new_row();
    bgroup(); // Grouping around ROW!
    if pseudorow {
      self.current_row_mut().unwrap().set_pseudo()
    } else {
      // Store row number before digest — row hooks may need it (e.g. \rowcolors)
      // and cannot re-borrow the alignment which is already mutably borrowed.
      let row_num = self.current_row_number();
      assign_value("alignmentRowNumber", row_num as i32, None);
      let row_before = digest(T_CS!("\\lx@alignment@row@before"))?;
      push_box_list(row_before);
    }
    self.in_row = true;
    assign_value("alignmentStartColumn", 0, None); // ???
    Ok(())
  }

  pub fn end_row(&mut self) -> Result<()> {
    if self.in_row {
      if self.in_column {
        self.end_column()?;
      }
      egroup()?; // Grouping around ROW!
      self.in_row = false;
    }
    Ok(()) //  Digest(T_CS('\lx@alignment@row@after'));
  }

  pub fn start_column(&mut self, pseudorow: bool) -> Result<()> {
    if !self.in_row {
      self.start_row(pseudorow)?;
    } else if pseudorow {
      self.current_row_mut().unwrap().set_pseudo();
    }
    bgroup(); // Grouping around CELL!
    // Note: a VERY round-about way of tracking the column spanning!
    assign_value("alignmentStartColumn", self.current_column_number(), None);
    // Propagate `?` so a TooManyErrors Fatal (e.g. from a runaway
    // `&` storm in a malformed alignment, paper 1112.6246) actually
    // aborts rather than getting silently swallowed by `let _ =`.
    let _colspec = self.next_column()?;
    set_align_group_count(1000000);
    self.in_column = true;
    Ok(())
  }

  pub fn end_column(&mut self) -> Result<()> {
    if self.in_column {
      egroup()?; // Grouping around CELL!
      self.in_column = false;
    }
    Ok(())
  }

  pub fn set_in_tabular_head(&mut self) { self.in_tabular_head = true; }
  pub fn unset_in_tabular_head(&mut self) { self.in_tabular_head = false; }
  pub fn is_in_tabular_head(&self) -> bool { self.in_tabular_head }

  pub fn get_properties_mut(&mut self) -> &mut SymHashMap<Stored> { &mut self.properties }
  pub fn get_xml_attributes_mut(&mut self) -> &mut HashMap<String, String> {
    &mut self.xml_attributes
  }
}

//======================================================================
// Constructing the XML for the alignment.

impl Object for Alignment {}
impl BoxOps for Alignment {
  fn get_properties(&self) -> &SymHashMap<Stored> { &self.properties }
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&SymHashMap<Stored>) -> R {
    caller(&self.properties)
  }
  fn get_property(&self, key: &str) -> Option<Cow<'_, Stored>> {
    self.properties.get(key).map(Cow::Borrowed)
  }
  fn get_properties_mut(&mut self) -> &mut SymHashMap<Stored> { &mut self.properties }

  fn compute_size(
    &self,
    _options: SymHashMap<Stored>,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    Ok((
      Dimension::default(),
      Dimension::default(),
      Dimension::default(),
    ))
  }
  fn get_font(&self) -> Result<Option<Cow<'_, crate::common::font::Font>>> { Ok(None) }
  fn get_string(&self) -> Result<Cow<'_, str>> { Ok(Cow::Borrowed("")) }

  fn compute_size_and_cache(
    &mut self,
    _options: SymHashMap<Stored>,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    normalize_alignment(self)?;
    let w = self.cached_width.unwrap();
    let h = self.cached_height.unwrap();
    let d = self.cached_depth.unwrap();
    // Store in properties so get_size()'s has_property("cached_width") check works
    self.properties.insert("cached_width", Stored::Dimension(w));
    self
      .properties
      .insert("cached_height", Stored::Dimension(h));
    self.properties.insert("cached_depth", Stored::Dimension(d));
    Ok((w, h, d))
  }

  fn be_absorbed(&self, _document: &mut Document) -> Result<Vec<Node>> {
    // Alignments must be absorbed via `be_absorbed_mut` because
    // `normalize_alignment` mutates the carrier (rearranging rows,
    // applying border specs, etc). The `Digested::be_absorbed` dispatch
    // for `DigestedData::Alignment` correctly calls `be_absorbed_mut`;
    // this immutable path should never be reached in practice. Surface
    // a meaningful error rather than a bare `todo!()` panic if an
    // unexpected caller lands here.
    fatal!(
      Internal,
      Misdefined,
      "Alignment::be_absorbed called — use be_absorbed_mut instead \
       (alignment rearrangement requires &mut self)"
    );
  }
  fn be_absorbed_mut(&mut self, document: &mut Document) -> Result<Vec<Node>> {
    let ismath = self.is_math;
    normalize_alignment(self)?;
    // token-locators: the whole table's span (union of all cell spans), stamped
    // on the `tabular` element below. Computed before the mutable `rows` borrow.
    #[cfg(feature = "token-locators")]
    let table_span = self
      .rows
      .iter()
      .filter_map(row_span)
      .reduce(|a, b| crate::common::locator::Locator::new_range(a, b).unwrap_or(a));
    let rows = &mut self.rows;
    if rows.is_empty() {
      return Ok(Vec::new());
    }

    // Guard via the absorb limit to avoid infinite loops (Perl L478-483)
    let absorb_limit = lookup_int("absorb_limit");
    if absorb_limit > 0 {
      let mut absorb_count = lookup_int("absorb_count");
      absorb_count += 1;
      assign_value("absorb_count", absorb_count, Some(Scope::Global));
      if absorb_count > absorb_limit {
        fatal!(
          Timeout,
          Convert,
          s!(
            "Whatsit absorb limit of {} exceeded, infinite loop?",
            absorb_limit
          )
        );
      }
    }

    // We _should_ attach boxes to the alignment and rows,
    // but (ATM) we"ve only got sensible boxes for the cells.
    let mut attrs = HashMap::default();
    std::mem::swap(&mut attrs, &mut self.xml_attributes);
    // Perl Alignment.pm L311-316: pass dimension data to openContainer callback
    // Serialized as px-value strings for callbacks that need positioning (tikz matrices).
    if let Some(w) = self.cached_width {
      attrs.insert("cwidth".to_string(), format!("{}", w.px_value(None)));
    }
    if let Some(h) = self.cached_height {
      attrs.insert("cheight".to_string(), format!("{}", h.px_value(None)));
    }
    if let Some(d) = self.cached_depth {
      attrs.insert("cdepth".to_string(), format!("{}", d.px_value(None)));
    }
    let open_container_fn = &self.open_container;
    // token-locators: stamp the `tabular` with the table span before it opens.
    #[cfg(feature = "token-locators")]
    document.set_current_box_locator(table_span);
    open_container_fn(document, attrs)?;

    for row in rows {
      // token-locators: stamp this `tr` with the row's span before it opens.
      #[cfg(feature = "token-locators")]
      document.set_current_box_locator(row_span(row));
      let vpad_opt = row.get_padding().copied();
      // Perl Alignment.pm L319-324: pass position/size to openRow callback
      let mut open_row_attrs = HashMap::default();
      for (k, v) in &row.properties {
        open_row_attrs.insert(k.clone(), v.clone());
      }
      if let Some(x) = row.x {
        open_row_attrs.insert("x".to_string(), Stored::Dimension(x));
      }
      if let Some(y) = row.y {
        open_row_attrs.insert("y".to_string(), Stored::Dimension(y));
      }
      if let Some(w) = row.cached_width {
        open_row_attrs.insert("cwidth".to_string(), Stored::Dimension(w));
      }
      if let Some(h) = row.cached_height {
        open_row_attrs.insert("cheight".to_string(), Stored::Dimension(h));
      }
      if let Some(d) = row.cached_depth {
        open_row_attrs.insert("cdepth".to_string(), Stored::Dimension(d));
      }
      let open_row_fn = &self.open_row;
      open_row_fn(document, open_row_attrs)?;
      for before in row.before.iter() {
        document.absorb(before, None)?;
      }
      for cell in row.get_columns_mut().iter_mut() {
        if cell.skipped {
          continue;
        }
        // Normalize the border attribute
        // Perl: join(' ', sort(map { split(/ */, $_) } $$cell{border}));
        //       $border =~ s/(.) \1/$1$1/g;
        let mut border_chars: Vec<char> =
          cell.border.chars().filter(|c| !c.is_whitespace()).collect();
        border_chars.sort_unstable();
        let mut border = String::new();
        for (idx, &c) in border_chars.iter().enumerate() {
          border.push(c);
          // Space between different consecutive chars, no space between same chars
          if idx + 1 < border_chars.len() && border_chars[idx + 1] != c {
            border.push(' ');
          }
        }
        let open_column_fn = &self.open_column;
        let mut cell_attrs = HashMap::default();
        // Perl Alignment.pm L358-359: pass position/size to openColumn callback
        if let Some(x) = cell.x {
          cell_attrs.insert("x".to_string(), format!("{}", x.px_value(None)));
        }
        if let Some(y) = cell.y {
          cell_attrs.insert("y".to_string(), format!("{}", y.px_value(None)));
        }
        if let Some(w) = cell.cached_width {
          cell_attrs.insert("cwidth".to_string(), format!("{}", w.px_value(None)));
        }
        if let Some(h) = cell.cached_height {
          cell_attrs.insert("cheight".to_string(), format!("{}", h.px_value(None)));
        }
        if let Some(d) = cell.cached_depth {
          cell_attrs.insert("cdepth".to_string(), format!("{}", d.px_value(None)));
        }
        // Perl: always passes align attribute (Alignment.pm L350)
        if let Some(ref align) = cell.align {
          cell_attrs.insert(String::from("align"), align.name());
        }
        if let Some(ref vattach) = cell.vattach {
          cell_attrs.insert(String::from("vattach"), vattach.clone());
        }
        if let Some(w) = cell.width {
          cell_attrs.insert(String::from("width"), w.to_attribute());
        }
        if let Some(vpad) = vpad_opt {
          cell_attrs.insert(String::from("cssstyle"), s!("padding-bottom: {vpad}"));
        }
        // colortbl: backgroundcolor from \columncolor/\cellcolor
        if let Some(ref bg) = cell.backgroundcolor {
          cell_attrs.insert(String::from("backgroundcolor"), bg.clone());
        }
        // Perl: colspan/rowspan attributes for spanning cells
        if cell.colspan.unwrap_or(1) != 1 {
          cell_attrs.insert(String::from("colspan"), cell.colspan.unwrap().to_string());
        }
        if cell.rowspan.unwrap_or(1) != 1 {
          cell_attrs.insert(String::from("rowspan"), cell.rowspan.unwrap().to_string());
        }
        if !border.is_empty() {
          cell_attrs.insert(String::from("border"), border);
        }
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
        // Perl Alignment.pm L332-347: ltx_nopad_l/ltx_nopad_r CSS classes
        // Based on lspaces/rspaces width relative to 0.2em threshold
        let mut classes: Vec<String> = Vec::new();
        let empty = cell.empty;
        // Perl: $$cell{boxes} — truthy when boxes field is defined (even if empty List)
        let has_boxes = cell.boxes.is_some();
        let mut pre_absorb: Option<Digested> = None;
        let mut post_absorb: Option<Digested> = None;
        if !ismath {
          // 0.2em ≈ 131072 scaled points; 1.5em ≈ 983040 scaled points (at 10pt font)
          let threshold_02em: i64 = 131072;
          let threshold_15em: i64 = 983040;
          // Perl: $lpad = ($$cell{lspaces} ? $$cell{lspaces}->getWidth->valueOf : 0)
          // Note: In Perl, lspaces is populated from \lx@intercol (isSpace, width=tabcolsep)
          // during cell content extraction. Rust doesn't populate lspaces, so we approximate:
          // - If template has \lx@intercol → there IS intercolumn padding (assume 0.2em)
          // - Else if template has \hfil/\hfill → centering fill, treat as padding (prevents
          //   incorrect ltx_nopad_l for regular centered/right-aligned columns)
          // - Else → no padding (assume 0, enables ltx_nopad_l)
          // Perl L338-339: lpad/rpad from extracted lspaces/rspaces width.
          // When lspaces/rspaces are populated (from cell extraction), use their
          // actual width. When None, use template heuristic as fallback.
          // Perl L338-339: lpad/rpad from lspaces/rspaces width.
          // In Perl, lspaces is populated from the left-scan of digested cell boxes.
          // The left-scan encounters template-injected boxes (vrules, intercol spaces,
          // fills) and extracts spacing. When lspaces is undef, Perl returns 0.
          // In Rust, lspaces is not always populated by extraction (extraction stores
          // None for empty lspaces). We use intercol_reachable_in_before to distinguish:
          // - Regular columns (|l|): \vrule then \lx@intercol → reachable → threshold
          // - @{text} columns: text then \lx@intercol → NOT reachable → 0
          //
          // KNOWN LIMITATION: this fallback over-reports padding for some
          // numprint cells (`\lx@intercol\nprt@begin\ignorespaces`-style
          // before): Perl's extracted lspaces would be undef there → lpad=0 →
          // ltx_nopad_l added; our heuristic returns threshold_02em → no nopad_l.
          // Naively removing the heuristic regresses 21 other tabular tests
          // because Rust's extraction doesn't always populate lspaces for
          // cases where Perl's would (`|l|`, `|c|`, `|r|`, p/m/X, etc.). The
          // proper fix is to make lspaces extraction reliable to match Perl's
          // left-scan; until then, this heuristic is load-bearing.
          let lpad = cell
            .lspaces
            .as_ref()
            .and_then(|ls| ls.get_width(None).ok().flatten())
            .map(|rv| rv.value_of())
            .unwrap_or_else(|| {
              if intercol_reachable_in_before(&cell.before) || template_has_fill(&cell.before) {
                threshold_02em
              } else {
                0
              }
            });
          let rpad = cell
            .rspaces
            .as_ref()
            .and_then(|rs| rs.get_width(None).ok().flatten())
            .map(|rv| rv.value_of())
            .unwrap_or_else(|| {
              // Perl: rpad from rspaces. When not extracted, use template.
              // Only the after tokens determine right padding.
              if template_has_intercol(&cell.after) {
                threshold_02em
              } else {
                0
              }
            });
          // Perl L340-341: ltx_nopad_l, unless math mode (Perl: `unless $ismath`)
          if !ismath && (!empty || has_boxes) && lpad < threshold_02em {
            classes.push("ltx_nopad_l".to_string());
          } else if lpad < threshold_15em {
            // In math mode, absorb named spacing (like \quad) as XMHint content
            // even when below the 1.5em threshold. This preserves XMHint for
            // the math parser to convert to lpadding.
            if ismath && cell.lspaces.is_some() {
              pre_absorb = cell.lspaces.take();
            }
          } else {
            pre_absorb = cell.lspaces.take();
          }
          // Perl L344-345: ltx_nopad_r, unless math mode (Perl: `unless $ismath`)
          if !ismath && (!empty || has_boxes) && rpad < threshold_02em {
            classes.push("ltx_nopad_r".to_string());
          } else if rpad < threshold_15em {
            // do nothing — use CSS default padding
          } else {
            post_absorb = cell.rspaces.take();
          }
        }
        if let Some(ref cell_class) = cell.class {
          classes.insert(0, cell_class.clone());
        }
        let class_str: String = classes
          .into_iter()
          .filter(|s| !s.is_empty())
          .collect::<Vec<_>>()
          .join(" ");
        if !class_str.is_empty() {
          cell_attrs.insert(String::from("class"), class_str);
        }
        //       # Which properties do we expose to the constructor?
        //       x      => $$cell{x}, y => $$cell{y},
        //       cached_width => $$cell{cached_width}, cached_height => $$cell{cached_height},
        // cached_depth => $$cell{cached_depth})
        // token-locators: stamp this `td` with the cell's content span before it
        // opens (its content's box_to_absorb is set just below, after the open).
        #[cfg(feature = "token-locators")]
        document.set_current_box_locator(cell_loc(cell));
        cell.cell = open_column_fn(document, cell_attrs)?;
        // Perl L362: absorb cell content only if !skippable (not just !empty)
        if !cell.skippable {
          let box_ref = cell.boxes.as_ref().unwrap();
          // local $LaTeXML::BOX
          document.set_box_to_absorb(Some(box_ref.clone()));
          // Perl wraps cell content in XMArg for math alignments, but NOT for _Capture_ columns
          // (_Capture_ is not in the schema, so Perl's openElement validation prevents XMArg there)
          let cur_qname = get_node_qname(document.get_node());
          let wrap_xmarg =
            ismath && !crate::common::arena::with(cur_qname, |s| s.ends_with("_Capture_"));
          if wrap_xmarg {
            // Hacky!
            document.open_element("ltx:XMArg", Some(string_map!("rule" => "Anything")), None)?;
          }
          // Perl L365: absorb pre-spacing (lspaces > 1.5em)
          if let Some(ref pre) = pre_absorb {
            document.absorb(pre, None)?;
          }
          // In math mode, absorb lspaces as content (creates XMHint for \quad etc.)
          // This is needed for the math parser to convert XMHint → lpadding.
          if ismath
            && pre_absorb.is_none()
            && let Some(ref lsp) = cell.lspaces
          {
            document.absorb(lsp, None)?;
          }
          document.absorb(box_ref, None)?;
          // Perl L367: absorb post-spacing (rspaces > 1.5em)
          if let Some(ref post) = post_absorb {
            document.absorb(post, None)?;
          }
          if wrap_xmarg {
            // Hacky!
            document.close_element("ltx:XMArg")?;
          }
          // expire local $LaTeXML::BOX
          document.expire_box_to_absorb();
        } else if let Some(ref boxes) = cell.boxes {
          // Cell is skippable but may contain preserved boxes (e.g. \label wrapped
          // in \lx@hidden@noalign with alignmentPreserve=true). These boxes need
          // to be absorbed so their constructors run (e.g. \label sets labels= on
          // the parent equation element via float_to_label).
          // In Perl, \hfil from the template contributes cell width, making such
          // cells non-skippable. In Rust, \hfil doesn't contribute width.
          for item in boxes.unlist() {
            if item.get_property_bool("alignmentPreserve") {
              document.absorb(&item, None)?;
            }
          }
        }
        let close_column_fn = &self.close_column;
        close_column_fn(document)?;
      }
      for after in row.after.iter() {
        document.absorb(after, None)?;
      }
      let close_row_fn = &self.close_row;
      close_row_fn(document)?;
    }
    let close_container_fn = &self.close_container;
    let node_opt = close_container_fn(document)?;

    // If we're not nested inside another tabular
    // [This should be an afterConstruct somewhere?]
    // If requested to guess headers & we're not nested inside another tabular
    if let Some(mut node) = node_opt {
      if document
        .findnodes("ancestor::ltx:tabular", Some(&node))
        .is_empty()
      {
        let hashead = !document
          .findnodes("descendant::ltx:td[@thead]", Some(&node))
          .is_empty();
        // If requested && no cells are already marked as being thead, apply heuristic
        let guess_headers = self
          .properties
          .get("guess_headers")
          .map(|v| !matches!(v, Stored::Bool(false)))
          .unwrap_or(false);
        if guess_headers && !hashead {
          guess_alignment_headers(document, &mut node, self)?;
        }
        // Otherwise, if not a math array, group thead & tbody rows
        // TODO: Re-design asking the outer Whatsit about "!body->isMath"
        else if hashead && !ismath {
          // in case already marked w/thead|tbody
          alignment_regroup_rows(document, &node)?;
        }
      }
      Ok(vec![node])
    } else {
      Ok(Vec::new())
    }
  }
}

impl Debug for Alignment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Alignment{{template:{:?}, properties:{:?}, rows:{:?} }}",
      self.template, self.properties, self.rows
    )
  }
}

impl Display for Alignment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{self:?}") }
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
pub fn read_alignment_template() -> Result<Template> {
  gullet::skip_spaces()?;
  local_build_template(Template::default());
  let mut tokens = vec![T_BEGIN!()];
  let mut nopens = 0;
  while let Some(open) = gullet::read_token()? {
    if open.get_catcode() == Catcode::BEGIN {
      nopens += 1;
    } else {
      gullet::unread_one(open);
      break;
    }
  }
  while let Some(op) = gullet::read_token()? {
    let cc = op.get_catcode();
    if cc == Catcode::SPACE {
    } else if cc == Catcode::END {
      let mut last_op = op;
      nopens -= 1;
      while nopens > 0 {
        if let Some(next_op) = gullet::read_token()? {
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
      gullet::unread_one(last_op);
    } else {
      match lookup_expandable(&T_CS!(s!("\\NC@rewrite@{op}")), None)? {
        Some(defn) => {
          let invoked = defn.invoke(true)?;
          gullet::unread(invoked);
        },
        _ => {
          if cc == Catcode::BEGIN {
            let balanced_arg = gullet::read_balanced(ExpansionLevel::Off, false, false)?;
            if !balanced_arg.is_empty() {
              gullet::unread(balanced_arg);
            }
          } else {
            Warn!("unexpected", op, s!("Unrecognized tabular template {op:?}"));
          }
        },
      }
    }
    if nopens <= 0 {
      break;
    }
  }
  tokens.push(T_END!());
  with_current_build_template(|template_opt| {
    let t = template_opt.unwrap();
    t.set_reversion(Tokens::new(tokens));
    // Perl Alignment.pm L912: $BUILD_TEMPLATE->finish
    t.finish();
  });
  Ok(take_build_template().unwrap())
}

pub fn parse_alignment_template(spec: &str) -> Result<Template> {
  let reader_mouth = Mouth::new(&s!("{{{spec}}}"), None)?;
  gullet::reading_from_mouth(reader_mouth, read_alignment_template)
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

/// Check whether a template token list contains fill/spacing commands
/// like \hfil, \hfill, \hskip, \lx@intercol. Previously used as fallback
/// for lpad/rpad; now superseded by template_has_intercol for better @{} handling.
#[allow(dead_code)]
fn template_has_fill(tokens: &Option<Tokens>) -> bool {
  if let Some(toks) = tokens {
    for tok in toks.unlist_ref() {
      let s = tok.to_string();
      if s == "\\hfil" || s == "\\hfill" || s == "\\hskip" {
        return true;
      }
    }
  }
  false
}

/// Check if \lx@intercol in the before tokens is reachable by the left-scan.
/// The left-scan skips: isVerticalRule (\vrule), \relax, isFill (\hfil/\hfill),
/// isSpace, isHorizontalRule, alignmentSkippable, Comment.
/// It STOPS at real content (like text from @{text}).
/// For `\vrule\relax\lx@intercol\hfil`: reachable (vrule is skippable).
/// For `@{1}\lx@intercol\hfil`: NOT reachable ("1" blocks the scan).
fn intercol_reachable_in_before(tokens: &Option<Tokens>) -> bool {
  if let Some(toks) = tokens {
    for tok in toks.unlist_ref() {
      let s = tok.to_string();
      if s == "\\lx@intercol" || s.contains("intercol") {
        return true;
      }
      // These are skippable by the left-scan in Perl's extractAlignmentColumn
      if s == "\\vrule"
        || s == "\\relax"
        || s == "\\hfil"
        || s == "\\hfill"
        || s == "\\hskip"
        || s == "\\lx@column@trimright"
      {
        continue;
      }
      // Any other token blocks the scan
      return false;
    }
  }
  false
}

/// Check if template tokens contain \lx@intercol (intercolumn spacing).
/// Unlike template_has_fill, this ignores \hfil/\hfill which are alignment fill.
/// \lx@intercol indicates actual intercolumn padding; \hfil is just centering.
/// For @{}c@{} columns, \lx@intercol is disabled but \hfil remains.
fn template_has_intercol(tokens: &Option<Tokens>) -> bool {
  if let Some(toks) = tokens {
    for tok in toks.unlist_ref() {
      let s = tok.to_string();
      if s == "\\lx@intercol" || s.contains("intercol") {
        return true;
      }
    }
  }
  false
}

fn guess_alignment_headers(
  document: &mut Document,
  table: &mut Node,
  alignment: &mut Alignment,
) -> Result<()> {
  // Assume that headers don't make sense for nested tables.
  // OR Maybe we should only do this within table environments???
  if !document
    .findnodes("ancestor::ltx:tabular", Some(table))
    .is_empty()
  {
    return Ok(());
  }
  let tag = get_node_qname(table);
  // TODO
  //   Debug(('=' x 50) . "\nGuessing alignment headers for "
  //       . (($x = $document->findnode('ancestor-or-self::*[@xml:id]', $table)) ?
  // $x->getAttribute('xml:id') : $tag))     if $LaTeXML::DEBUG{alignment};

  let ismath = tag == crate::pin!("ltx:XMArray");
  let reversed = false;
  // Attempt to recognize header lines.
  // Build a view of the table by extracting the rows, collecting & characterizing each cell.
  classify_alignment_rows(alignment);

  {
    let mut rows = collect_alignment_rows(alignment);
    if rows.is_empty() {
      return Ok(());
    }
    alignment_characterize_lines(document, Axis::Row, false, rows.as_mut_slice())?;
  }
  // Flip the rows around to produce a column view.
  {
    let mut cols = collect_alignment_columns(alignment);
    if cols.is_empty() {
      return Ok(());
    }
    // This usually does something unpleasant
    alignment_characterize_lines(document, Axis::Column, false, cols.as_mut_slice())?;
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
        None => {},
      }
    }
  }
  // dbg!((n_h, n_d));
  //   Debug("$n{h} header, $n{d} data cells") if $LaTeXML::DEBUG{alignment};
  if n_d == 1 {
    // Or any other heuristic?
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
    alignment_regroup_rows(document, table)?;
  }
  if n_h > 0 {
    // Found some headers?
    document.add_class(table, "ltx_guessed_headers")?;
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
fn alignment_regroup_rows(document: &mut Document, table: &Node) -> Result<()> {
  let mut rows = document.findnodes("ltx:tr", Some(table));
  // `heads` is bounded by the initial thead-candidate rows; pre-size
  // to `rows.len()` as a conservative upper bound.
  let mut heads = Vec::with_capacity(rows.len());
  let mut maxreach = 0;
  // Scan initial rows as potential thead
  while !rows.is_empty() {
    let cells = document.findnodes("ltx:td", Some(&rows[0]));
    // Non header cells, done.
    if cells
      .iter()
      .any(|cell| cell.get_attribute("thead").is_none())
    {
      break;
    }
    let line = heads.len();
    heads.push(rows.remove(0));
    for cell in cells {
      // Malformed/non-numeric rowspan silently degrades to 0 — matches Perl's
      // lax numeric coercion and prevents crashes on unusual input XML.
      let this_rowspan = cell
        .get_attribute("rowspan")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        + line;
      if this_rowspan > maxreach {
        maxreach = this_rowspan;
      }
    }
  }
  if maxreach > heads.len() {
    // rowspan crossed over thead boundary! Put head rows back at the FRONT of body rows.
    heads.append(&mut rows);
    rows = heads;
    heads = Vec::new();
  }
  // scan trailing rows as potential tfoot
  let mut foots = VecDeque::new();
  while !rows.is_empty() {
    let cells = document.findnodes("ltx:td", Some(rows.last().unwrap()));
    // Non header cells, done.
    if cells
      .iter()
      .any(|cell| cell.get_attribute("thead").is_none())
    {
      break;
    }
    foots.push_front(rows.pop().unwrap())
  }
  if !heads.is_empty() {
    document.wrap_nodes("ltx:thead", heads)?;
  }
  if !rows.is_empty() {
    document.wrap_nodes("ltx:tbody", rows)?;
  }
  if !foots.is_empty() {
    document.wrap_nodes("ltx:tfoot", foots.into_iter().collect())?;
  }
  Ok(())
}

//======================================================================
/// Setup a View of the alignment, with characterized cells, for analysis -- modifying it in place.
fn classify_alignment_rows(alignment: &mut Alignment) {
  let mut ncols = 0;
  for arow in &mut alignment.rows {
    let n = arow.get_columns().len();
    if n > ncols {
      ncols = n;
    }
  }
  // eprintln!("classify_alignment_rows: {} rows, max {} cols", alignment.rows.len(), ncols);
  let (mut h, mut v) = (false, false);
  for arow in alignment.rows.iter_mut() {
    let cols = arow.get_columns_mut();
    let this_row_len = cols.len();
    // eprintln!("  row {_ri}: {this_row_len} cols");
    for col in cols.iter_mut() {
      col.cell_type = Some('d');
      col.content_class = Some(
        // Assume mixed content for any justified cell???
        if col.align == Some(Align::Justify) {
          ColumnSpec::MathAltText
        } else if col.cell.is_some() {
          classify_alignment_cell(col.cell.as_ref().unwrap())
        } else {
          ColumnSpec::Unknown
        },
      );
      // eprintln!("    cell: cell={}, class={:?}, border='{}'", col.cell.is_some(),
      // col.content_class, col.border);
      col.content_length = Some(if col.content_class == Some(ColumnSpec::Graphics) {
        1000
      } else if col.cell.is_some() {
        col.cell.as_ref().unwrap().get_content().chars().count()
      } else {
        0
      });
      let (mut border_top, mut border_bottom, mut border_left, mut border_right) = (0, 0, 0, 0);
      for c in col.border.chars() {
        match c {
          'l' | 'L' => border_left += 1,
          'r' | 'R' => border_right += 1,
          't' | 'T' => border_top += 1,
          'b' | 'B' => border_bottom += 1,
          _ => {}, // spaces etc.
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
          ..Cell::default()
        };
        cols.push(col);
      }
    }
  }
  // DG: cache assignments, and execute in post-loop, so that we can avoid indexing arithmetic
  let mut outer_border_right_assignments = Vec::new();
  let mut outer_border_bottom_assignments = Vec::new();
  // Perl: copy characterizations (align/content_class/content_length) from rowspan/colspan cells
  // to the cells they span over. Deferred to avoid borrow conflicts across rows.
  #[allow(clippy::type_complexity)]
  let mut rowspan_propagation: Vec<(
    usize,
    usize,
    Option<Align>,
    Option<ColumnSpec>,
    Option<usize>,
  )> = Vec::new();
  // copy the characterizations to spanned cells
  for r in 0..alignment.rows.len() {
    let row = &mut alignment.rows[r];
    let cols = row.get_columns_mut();
    for c in 0..cols.len() {
      let rs = cols[c].rowspan.unwrap_or(1);
      let cs = cols[c].colspan.unwrap_or(1);
      let ca = cols[c].align.clone();
      let cc = cols[c].content_class;
      let cl = cols[c].content_length;
      let rb = cols[c].border_right;

      cols[c].border_right = Some(0);
      let bb = cols[c].border_bottom;
      cols[c].border_bottom = Some(0);
      for row_reach in cols.iter_mut().take(c + cs).skip(c + 1) {
        row_reach.align = ca.clone();
        row_reach.content_class = cc;
        row_reach.content_length = cl;
      }
      // Perl L1073-1077: copy characterizations to rowspan-covered cells
      for sr in 1..rs {
        for sc in 0..cs {
          rowspan_propagation.push((r + sr, c + sc, ca.clone(), cc, cl));
        }
      }

      // move the outer borders
      for sr in 0..rs {
        outer_border_right_assignments.push((r + sr, c + cs - 1, rb));
      }
      for sc in 0..cs {
        outer_border_bottom_assignments.push((r + rs - 1, c + sc, bb));
      }
    }
  }
  // Apply the collected rowspan propagation assignments
  for (row_idx, col_idx, align, content_class, content_length) in rowspan_propagation.into_iter() {
    if row_idx < alignment.rows.len() {
      let cols = alignment.rows[row_idx].get_columns_mut();
      if col_idx < cols.len() {
        cols[col_idx].align = align;
        cols[col_idx].content_class = content_class;
        cols[col_idx].content_length = content_length;
      }
    }
  }
  // Apply the collected outer border assignments
  for (row_idx, col_idx, value) in outer_border_right_assignments.into_iter() {
    if let Some(row) = alignment.rows.get_mut(row_idx) {
      let cols = row.get_columns_mut();
      if col_idx < cols.len() {
        cols[col_idx].border_right = value;
      }
    }
  }
  for (row_idx, col_idx, value) in outer_border_bottom_assignments.into_iter() {
    if let Some(row) = alignment.rows.get_mut(row_idx) {
      let cols = row.get_columns_mut();
      if col_idx < cols.len() {
        cols[col_idx].border_bottom = value;
      }
    }
  }
  // Now, do some border massaging...
  // Empty-alignment guard: if ncols==0 (no columns in any row), there are no
  // borders to massage. Skip the whole block to avoid out-of-bounds panics on
  // `cols[0]` (witness: astro-ph0006087, garmire.tex deluxetable input).
  if ncols == 0 {
    return;
  }
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
  for c in 0..ncols {
    alignment.rows[0].get_columns_mut()[c].border_top = Some(if h { 1 } else { 0 });
    if nrows > 1 {
      if let Some(bt) = alignment.rows[1].get_columns_mut()[c].border_top
        && bt > 0
      {
        // only set if border is inked
        alignment.rows[0].get_columns_mut()[c].border_bottom = Some(bt);
      }
      if let Some(bb) = alignment.rows[nrows - 2].get_columns_mut()[c].border_bottom
        && bb > 0
      {
        // only set if border is inked
        alignment.rows[nrows - 1].get_columns_mut()[c].border_top = Some(bb);
      }
    }
    alignment.rows[nrows - 1].get_columns_mut()[c].border_bottom = Some(if h { 1 } else { 0 });
  }
  // Note: This propagation doesn't exist in Perl's Alignment.pm.
  // But removing it breaks fonts_test (guessTableHeaders). Needs careful audit.
  for r in 1..nrows - 1 {
    for c in 1..ncols - 1 {
      if let Some(bb) = alignment.rows[r - 1].get_columns_mut()[c].border_bottom
        && bb > 0
      {
        // only set if border is inked
        alignment.rows[r].get_columns_mut()[c].border_top = Some(bb);
      }
      if let Some(bt) = alignment.rows[r + 1].get_columns_mut()[c].border_top
        && bt > 0
      {
        // only set if border is inked
        alignment.rows[r].get_columns_mut()[c].border_bottom = Some(bt);
      }
      if let Some(br) = alignment.rows[r].get_columns_mut()[c - 1].border_right
        && br > 0
      {
        // only set if border is inked
        alignment.rows[r].get_columns_mut()[c].border_left = Some(br);
      }
      if let Some(bl) = alignment.rows[r].get_columns_mut()[c + 1].border_left
        && bl > 0
      {
        // only set if border is inked
        alignment.rows[r].get_columns_mut()[c].border_right = Some(bl);
      }
    }
  }
  // debug info
  // eprintln!("Cell characterizations:");
  // for (row_index,row) in alignment.rows.iter().enumerate() {
  //   for (col_index, cell) in row.get_columns().iter().enumerate() {
  //     eprintln!("[{row_index},{col_index}]=>{}{}{} {} {} => {}{}{}{}",
  //       cell.cell_type.as_ref().unwrap_or(&'?'),
  //       cell.align.map(|a| a.char_code()).unwrap_or(' '),
  //       cell.content_class.map(|a| a.to_string()).unwrap_or_else(|| String::from("?")),
  //       cell.content_length.unwrap_or(0),
  //       cell.border,
  //       if cell.border_top.unwrap_or(0) > 0 { "t" } else { "" },
  //       if cell.border_right.unwrap_or(0) > 0  { "r" } else { "" },
  //       if cell.border_bottom.unwrap_or(0) > 0 { "b" } else { "" },
  //       if cell.border_left.unwrap_or(0) > 0 { "l" } else {""}
  //     );
  //   }
  // }
}

fn collect_alignment_rows(alignment: &mut Alignment) -> Vec<Vec<&mut Cell>> {
  alignment
    .rows
    .iter_mut()
    .map(|x| x.get_columns_mut().iter_mut().collect())
    .collect()
}

fn collect_alignment_columns(alignment: &mut Alignment) -> Vec<Vec<&mut Cell>> {
  let mut row_cells: Vec<_> = alignment
    .rows
    .iter_mut()
    .map(|r| r.get_columns_mut().iter_mut())
    .collect();
  let n_cols = row_cells[0].len();
  let n_rows = row_cells.len();
  let mut columns = Vec::with_capacity(n_cols);
  for _ in 0..n_cols {
    let mut column = Vec::with_capacity(n_rows);
    for row_iter in row_cells.iter_mut() {
      column.push(row_iter.next().unwrap());
    }
    columns.push(column);
  }
  columns
}

/// Return one of: i(nteger), t(ext), m(ath), ? (unknown) or '_' (empty) (or some combination)
///  or 'mx' for alternating text & math.
fn classify_alignment_cell(xcell: &Node) -> ColumnSpec {
  let content = xcell.get_content();
  let mut inferred_classes: Vec<ColumnSpec> = Vec::new();
  // Perl L1123: /^[\s\d]+$/ — Perl \d is ASCII-only (0-9).
  // Also include mathematical double-struck digits (U+1D7D8-U+1D7E1, 𝟘-𝟡) since these
  // are font-decoded equivalents of ASCII 0-9 in blackboard bold fonts. In Perl, these
  // appear as ASCII digits with font attributes; in Rust they're Unicode codepoints.
  // Exclude circled/enclosed numerals (❶❷❸ U+2776-, ①②③ U+2460-) which are symbols.
  if !content.is_empty()
    && content
      .chars()
      .all(|c| c.is_whitespace() || c.is_ascii_digit() || ('\u{1D7D8}'..='\u{1D7E1}').contains(&c))
  {
    inferred_classes.push(ColumnSpec::Integer);
  } else {
    let mut nodes = xcell.get_child_nodes();
    while !nodes.is_empty() {
      let ch = nodes.remove(0);
      match ch.get_type() {
        Some(NodeType::TextNode) => {
          let text = ch.get_content();
          if !(text.chars().all(|c| c.is_whitespace())
            || (inferred_classes.first() == Some(&ColumnSpec::Math)
              && SINGLE_PUNCT.is_match(&text)))
          {
            inferred_classes.push(ColumnSpec::Text);
          }
        },
        Some(NodeType::ElementNode) => {
          with_node_qname(&ch, |chtag| match chtag {
            "ltx:text" => {
              // Perl L1136-1137: $class .= 't' unless $class eq 't'
              // Only skip if the LAST class was also Text (not just first).
              // This preserves "tt" for cells with two text elements.
              if inferred_classes.last() != Some(&ColumnSpec::Text) {
                inferred_classes.push(ColumnSpec::Text);
              }
            },
            "ltx:graphics" => {
              if inferred_classes.first() != Some(&ColumnSpec::Graphics) {
                inferred_classes.push(ColumnSpec::Graphics);
              }
            },
            "ltx:Math" => {
              if inferred_classes.first() != Some(&ColumnSpec::Math) {
                inferred_classes.push(ColumnSpec::Math);
              }
            },
            "ltx:XMText" => {
              if inferred_classes.first() != Some(&ColumnSpec::Text) {
                inferred_classes.push(ColumnSpec::Text);
              }
            },
            "ltx:XMArg" | "ltx:inline-block" | "ltx:p" => {
              // Transparent containers: look through to classify children.
              // Perl's beAbsorbed creates <text> directly in td; Rust wraps in
              // <inline-block><p> from {turn}/{rotate}. Treat these as transparent
              // so the classification matches Perl's view of the cell content.
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
            },
          })
        },
        _ => {},
      }
    }
  }

  // check if we have alternating math-and-text or text-and-math (only if 2+ classes)
  if inferred_classes.len() > 1 {
    let mut alt_peekable = inferred_classes.iter().peekable();
    let mut is_alternating = true;
    while let Some(c) = alt_peekable.next() {
      match c {
        ColumnSpec::Math | ColumnSpec::Integer => {
          if let Some(peek) = alt_peekable.peek()
            && !matches!(peek, ColumnSpec::Text)
          {
            is_alternating = false;
            break;
          }
        },
        ColumnSpec::Text => {
          if let Some(peek) = alt_peekable.peek()
            && !matches!(peek, ColumnSpec::Math | ColumnSpec::Integer)
          {
            is_alternating = false;
            break;
          }
        },
        _ => {
          is_alternating = false;
          break;
        },
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
    // Perl L1151: multi-class detection.
    // "tt" (all Text) → MultiText, mixed math+text → MathAltText
    let all_text = inferred_classes
      .iter()
      .all(|c| matches!(c, ColumnSpec::Text));
    if all_text {
      ColumnSpec::MultiText
    } else {
      ColumnSpec::Unknown
    }
  }
}

//======================================================================
// Scan pairs of rows/columns attempting to recognize differences that
// might indicate which are headers and which are data.
// Warning: This section is full of "magic numbers"
// guessed by sampling various test cases.

const MIN_ALIGNMENT_DATA_LINES: usize = 1; //  (or 2?) [CONSTANT]
const MAX_ALIGNMENT_HEADER_LINES: usize = 4; // [CONSTANT]

// We expect to find header lines at the beginning, noticably different from the eventual data
// lines. Both header lines and data lines can consist of several neighboring lines.
// Check that header lines are `similar' to each other.  So, the strategy is to look
// for a `hump' in the line differences and consider blocks containing these lines to be potential
// headers.

fn alignment_characterize_lines(
  document: &mut Document,
  axis: Axis,
  reversed: bool,
  lines: &mut [Vec<&mut Cell>],
) -> Result<()> {
  let n = lines.len();
  if n < 2 {
    return Ok(());
  }
  // eprintln!("Characterizing {n} {}", if axis == Axis::Row {"rows"} else {"columns"});

  // Establish a scale of differences for the table.
  let (mut max_diff, mut min_diff, _avg_diff) = (0.0, 99999999.0, 0.0);
  for l in 0..n - 1 {
    let d = alignment_compare(axis, true, reversed, l, l + 1, lines);
    // eprintln!("  compare({l},{}) = {d}", l + 1);
    // avg_diff += d;
    if d > max_diff {
      max_diff = d;
    }
    if d < min_diff {
      min_diff = d;
    }
  }
  // avg_diff = avg_diff / (n - 1) as f64;
  if max_diff < 0.05 {
    // virtually no differences.
    return Ok(());
  }
  if (n > 2) && ((max_diff - min_diff) < max_diff * 0.5) {
    // differences too similar to establish pattern
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
    if diff >= tab_threshold {
      break;
    }
    maxh += 1;
  }
  if maxh > MAX_ALIGNMENT_HEADER_LINES {
    // too many before even finding diffs? give up!
    return Ok(());
  }
  while alignment_compare(axis, true, reversed, maxh, maxh + 1, lines) > tab_threshold {
    maxh += 1;
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
    if !heads.is_empty() {
      // Now, change all cells marked as header from td => th.
      for h in heads {
        for (i, cell) in lines[h].iter_mut().enumerate() {
          cell.cell_type = Some('h');
          if let Some(ref mut xcell) = cell.cell {
            // But NOT empty cells on outer edges.
            // Perl: !$$cell{l} is falsy for both undef AND 0.
            if (cell.content_class == Some(ColumnSpec::Empty))
              && ((i == 0
                && (if axis == Axis::Row {
                  cell.border_left.unwrap_or(0) == 0
                } else {
                  cell.border_top.unwrap_or(0) == 0
                }))
                || (i == nn
                  && (if axis == Axis::Row {
                    cell.border_right.unwrap_or(0) == 0
                  } else {
                    cell.border_bottom.unwrap_or(0) == 0
                  })))
            {
            } else {
              document.add_ss_values(xcell, "thead", axis.marker_name())?;
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
fn alignment_test_headers(
  nhead: usize,
  tab_threshold: f64,
  axis: Axis,
  lines: &[Vec<&mut Cell>],
) -> Vec<usize> {
  // eprintln!("Testing {nhead} headers with threshold {tab_threshold} for axis {:?}", axis);
  let mut heads: Vec<usize> = (0..nhead).collect(); // The indices of heading lines.
  let mut head_length = alignment_max_content_length(0, 0, nhead - 1, lines);
  let mut next_line = nhead; // Start from the end of the proposed headings.

  // Watch out for the assumed header being really data that is a repeated pattern.
  let nrep = lines.len() / nhead;
  if nhead > 1 {
    //   Debug("Check for apparent header repeated $nrep times") if $LaTeXML::DEBUG{alignment};
    let mut matched = true;
    for r in 1..nrep {
      matched =
        matched && alignment_match_head(0, r * nhead, nhead, tab_threshold, axis, lines) > 0;
    }
    //   Debug("Repeated headers: " . ($matched ? "Matched=> Fail" : "Nomatch => Succeed"))
    //     if $LaTeXML::DEBUG{alignment};
    // eprintln!("  repeated pattern check: matched={matched}");
    if matched {
      return Vec::new();
    }
  }

  // And find a following grouping of data lines.
  let ndata = alignment_skip_data(next_line, tab_threshold, axis, lines);
  // eprintln!("  ndata={ndata} from next_line={next_line}");
  if ndata < nhead {
    // ???? Well, maybe if _really_ convincing???
    return Vec::new();
  }
  if (ndata < nhead) && (ndata < 2) {
    return Vec::new();
  }
  // Check that the content of the headers isn't dramatically larger than the content in the data
  let mut data_length = alignment_max_content_length(0, next_line, next_line + ndata - 1, lines);
  next_line += ndata;

  let mut nd;
  // If there are more lines, they should match either the previous data block, or the head/data
  // pattern.
  while next_line < lines.len() {
    // First try to match a repeat of the 1st data block;
    // This would be the case when groups of data have borders around them.
    // Could want to match a variable number of datalines, but they should be similar!!!??!?!?
    nd = if ndata > 1 {
      alignment_match_data(nhead, next_line, ndata, tab_threshold, axis, lines)
    } else {
      0
    };
    // eprintln!("  while: next_line={next_line}, nd={nd}");
    if nd > 0 {
      data_length = alignment_max_content_length(data_length, next_line, next_line + nd - 1, lines);
      next_line += nd;
    }
    // Else, try to match the first header block; less common.
    else if alignment_match_head(0, next_line, nhead, tab_threshold, axis, lines) > 0 {
      // eprintln!("  matched head at next_line={next_line}");
      for idx in next_line..next_line + nhead {
        heads.push(idx);
      }
      head_length =
        alignment_max_content_length(head_length, next_line, next_line + nhead - 1, lines);
      next_line += nhead;
      nd = alignment_match_data(nhead, next_line, ndata, tab_threshold, axis, lines);
      if nd == 0 {
        return Vec::new();
      }
      data_length = alignment_max_content_length(data_length, next_line, next_line + nd - 1, lines);
      next_line += nd;
    } else {
      // eprintln!("  no match at next_line={next_line} => fail");
      return Vec::new();
    }
  }
  // Header content seems too large relative to data?
  // eprintln!("  header content = {head_length}; data content = {data_length}");
  if (head_length > 10) && (head_length > 4 * data_length) {
    //   Debug("header content too much longer than data content")
    //     if $LaTeXML::DEBUG{alignment};
    return Vec::new();
  }
  // Or if a header cell has "large" content?
  if head_length >= 1000 {
    // Or if a header cell has "large" content?
    //   Debug("header content too large")
    //     if $LaTeXML::DEBUG{alignment};
    return Vec::new();
  }

  // eprintln!("  Succeeded with {nhead} headers: {heads:?}");
  heads
}

fn alignment_match_head(
  p1: usize,
  p2: usize,
  nhead: usize,
  tab_threshold: f64,
  axis: Axis,
  tablines: &[Vec<&mut Cell>],
) -> usize {
  let nh = alignment_match_lines(p1, p2, nhead, tab_threshold, axis, tablines);
  let ok = nhead == nh;
  // Debug("Matched $nh header lines => " . ($ok ? "Succeed" : "Failed")) if
  // $LaTeXML::DEBUG{alignment};
  if ok { nhead } else { 0 }
}

fn alignment_match_data(
  p1: usize,
  p2: usize,
  n: usize,
  tab_threshold: f64,
  axis: Axis,
  tablines: &[Vec<&mut Cell>],
) -> usize {
  let nd = alignment_match_lines(p1, p2, n, tab_threshold, axis, tablines);
  let ok = (nd as f64 * 1.0) / n as f64 > 0.66;
  //   Debug("Matched $nd data lines => " . ($ok ? "Succeed" : "Failed"))
  //     if $LaTeXML::DEBUG{alignment};
  if ok { nd } else { 0 }
}

// Match the $n lines starting at $i2 to those starting at $i1.
fn alignment_match_lines(
  p1: usize,
  p2: usize,
  n: usize,
  tab_threshold: f64,
  axis: Axis,
  tablines: &[Vec<&mut Cell>],
) -> usize {
  let max_n = tablines.len();
  for i in 0..n {
    if (p1 + i >= max_n)
      || (p2 + i >= max_n)
      || alignment_compare(axis, false, false, p1 + i, p2 + i, tablines) >= tab_threshold
    {
      return i;
    }
  }
  n
}

/// Skip through a block of lines starting at $i that appear to be data, returning the number of
/// lines. We'll assume the 1st line is data, compare it to following lines,
/// but also accept `continuation' data lines.
///
/// Note: Perl's continuation-line logic (L1336-1339) is effectively dead code:
/// `scalar($::TABLINES[0])` evaluates to an array ref's memory address (huge number),
/// making `0.4 * huge` very large, so `count_empty <= huge` is always true.
/// The condition `($n < 2) || true` = true, so the `last if` simplifies to just
/// `last if diff >= threshold`. We match this behavior.
fn alignment_skip_data(
  i: usize,
  tab_threshold: f64,
  axis: Axis,
  tablines: &[Vec<&mut Cell>],
) -> usize {
  let tab_lines_length = tablines.len();
  if i >= tab_lines_length {
    return 0;
  }
  let _header_width = if !tablines.is_empty() {
    tablines[0].len()
  } else {
    1
  };
  let mut n = 1;
  while i + n < tab_lines_length {
    if alignment_compare(axis, true, false, i + n - 1, i + n, tablines) >= tab_threshold {
      // TODO: Perl Alignment.pm L1337-1339 has continuation line check here.
      // Applying it changes behavior for fonts/bbold tables (false positive headers).
      // Need to investigate further.
      break;
    }
    n += 1;
  }
  if n >= MIN_ALIGNMENT_DATA_LINES { n } else { 0 }
}

/// Return the maximum "content length" for lines from $from to $to.
fn alignment_max_content_length(
  mut length: usize,
  from: usize,
  to: usize,
  tablines: &[Vec<&mut Cell>],
) -> usize {
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
///  otherwise    : comparing two lines that ought to have identical patterns (eg. in a repeated
/// block)
fn alignment_compare(
  axis: Axis,
  for_adjacency: bool,
  reversed: bool,
  p1: usize,
  p2: usize,
  lines: &[Vec<&mut Cell>],
) -> f64 {
  let max_guard = lines.len();
  if p1 >= max_guard || p2 >= max_guard {
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
  let mut diff = 0.0;

  for (cell1, cell2) in line1.iter().zip(line2.iter()) {
    // Annoying test avoids warnings if cells inconsistent; likely due to incorrect row/col spans
    if cell1.content_class.is_none()
      || cell2.content_class.is_none()
      || cell1.border_left.is_none()
      || cell2.border_left.is_none()
      || cell1.border_right.is_none()
      || cell2.border_right.is_none()
      || cell1.border_bottom.is_none()
      || cell2.border_bottom.is_none()
      || cell1.border_top.is_none()
      || cell2.border_top.is_none()
    {
      continue;
    }
    if cell1.align != cell2.align
      && cell1.content_class != Some(ColumnSpec::Empty)
      && cell2.content_class != Some(ColumnSpec::Empty)
    {
      diff += 0.75;
    }
    let d = cell1
      .content_class
      .as_ref()
      .unwrap()
      .difference_heuristic(cell2.content_class.as_ref().unwrap());
    if d > 0.0 {
      diff += d;
    }
    // compare certain edges
    if for_adjacency {
      // Compare edges for adjacent rows of potentially different purpose
      let mut inner_diffs = 0.0;
      if axis == Axis::Row {
        if cell1.border_right != cell2.border_right {
          inner_diffs += 1.0;
        }
        if cell1.border_left != cell2.border_left {
          inner_diffs += 1.0;
        }
      } else {
        if cell1.border_top != cell2.border_top {
          inner_diffs += 1.0;
        }
        if cell1.border_bottom != cell2.border_bottom {
          inner_diffs += 1.0;
        }
      };
      diff += 0.3 * inner_diffs;
      // Penalty for apparent divider between.
      let pedge = if axis == Axis::Row {
        if reversed {
          BorderSpec::Top
        } else {
          BorderSpec::Bottom
        }
      } else if reversed {
        BorderSpec::Left
      } else {
        BorderSpec::Right
      };
      let border1_pedge = cell1.border_at(pedge);
      let border2_pedge = cell2.border_at(pedge);
      if let Some(b1p) = border1_pedge
        && b1p > 0
        && (border1_pedge != border2_pedge)
      {
        diff += (b1p as i64 - border2_pedge.unwrap_or(0) as i64).abs() as f64;
      }
    } else {
      // Compare edges for rows from diff places for potential similarity
      let mut inner_diffs = 0.0;
      if cell1.border_right != cell2.border_right {
        inner_diffs += 1.0;
      }
      if cell1.border_left != cell2.border_left {
        inner_diffs += 1.0;
      }
      if cell1.border_top != cell2.border_top {
        inner_diffs += 1.0;
      }
      if cell1.border_bottom != cell2.border_bottom {
        inner_diffs += 1.0;
      }
      diff += 0.3 * inner_diffs;
    }
  }
  diff /= ncells as f64;
  // eprintln!("alignment_compare: {p1} - {p2} => {diff};");
  // Debug("$p1-$p2 => $diff; ") if $LaTeXML::DEBUG{alignment};
  diff
}
