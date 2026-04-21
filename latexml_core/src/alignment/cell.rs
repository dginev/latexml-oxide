use super::template::{Align, BorderSpec, ColumnSpec};
use crate::common::dimension::Dimension;
use crate::common::glue::Glue;
use crate::digested::Digested;
use crate::tokens::Tokens;
use libxml::tree::Node;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Cell {
  pub empty:               bool,
  pub skippable:           bool,
  pub omitted:             bool,
  pub skipped:             bool,
  pub thead_in_row:        bool,
  pub thead_in_column:     bool,
  pub before:              Option<Tokens>,
  pub after:               Option<Tokens>,
  pub align:               Option<Align>,
  pub width:               Option<Dimension>,
  pub height:              Option<Dimension>,
  pub depth:               Option<Dimension>,
  pub cached_width:        Option<Dimension>,
  pub cached_height:       Option<Dimension>,
  pub cached_depth:        Option<Dimension>,
  pub colspan:             Option<usize>,
  pub colspanned:          Option<usize>,
  pub rowspan:             Option<usize>,
  pub rowspanned:          Option<usize>,
  pub boxes:               Option<Digested>,
  pub cell_type:           Option<char>,
  pub content_class:       Option<ColumnSpec>,
  pub content_length:      Option<usize>,
  pub border_left:         Option<usize>,
  pub border_right:        Option<usize>,
  pub border_top:          Option<usize>,
  pub border_bottom:       Option<usize>,
  pub top_padding:         Option<usize>,
  pub bottom_padding:      Option<usize>,
  pub right_padding:       Option<usize>,
  pub left_padding:        Option<usize>,
  pub vattach:             Option<String>,
  pub cell:                Option<Node>,
  pub x:                   Option<Dimension>,
  pub y:                   Option<Dimension>,
  pub border:              String,
  /// Perl: $$colspec{tabskip} — intercolumn glue from \halign template
  pub tabskip:             Option<Glue>,
  /// Perl: $$colspec{lspaces} — left-side spacing (incl. tabskip)
  pub lspaces:             Option<Digested>,
  /// Perl: $$colspec{rspaces} — right-side spacing
  pub rspaces:             Option<Digested>,
  /// Perl: $$cell{class} — CSS class for the cell
  pub class:               Option<String>,
  /// Whether this column has \lx@intercol in its before tokens (from template).
  /// Set during template building, used by nopad heuristic to distinguish
  /// regular columns (has intercolumn padding) from @{}-disabled columns.
  pub has_intercol_before: bool,
  /// Whether this column has \lx@intercol in its after tokens (from template).
  /// Set during template building, used by nopad heuristic to distinguish
  /// regular columns (has intercolumn padding) from @{}-disabled columns.
  pub has_intercol_after:  bool,
  /// Background color from \columncolor/\cellcolor (set during digestion by \@setcellcolor)
  pub backgroundcolor:     Option<String>,
}
impl Cell {
  pub fn border_at(&self, side: BorderSpec) -> Option<usize> {
    match side {
      BorderSpec::Left => self.border_left,
      BorderSpec::Right => self.border_right,
      BorderSpec::Top => self.border_top,
      BorderSpec::Bottom => self.border_bottom,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cell_default_flags_all_false() {
    let c = Cell::default();
    assert!(!c.empty);
    assert!(!c.skippable);
    assert!(!c.omitted);
    assert!(!c.skipped);
    assert!(!c.thead_in_row);
    assert!(!c.thead_in_column);
    assert!(!c.has_intercol_before);
    assert!(!c.has_intercol_after);
  }

  #[test]
  fn cell_default_options_all_none() {
    let c = Cell::default();
    assert!(c.before.is_none());
    assert!(c.after.is_none());
    assert!(c.align.is_none());
    assert!(c.width.is_none());
    assert!(c.height.is_none());
    assert!(c.depth.is_none());
    assert!(c.cached_width.is_none());
    assert!(c.cached_height.is_none());
    assert!(c.cached_depth.is_none());
    assert!(c.colspan.is_none());
    assert!(c.colspanned.is_none());
    assert!(c.rowspan.is_none());
    assert!(c.rowspanned.is_none());
    assert!(c.boxes.is_none());
    assert!(c.cell_type.is_none());
    assert!(c.border_left.is_none());
    assert!(c.border_right.is_none());
    assert!(c.border_top.is_none());
    assert!(c.border_bottom.is_none());
    assert!(c.backgroundcolor.is_none());
  }

  #[test]
  fn cell_default_border_is_empty_string() {
    let c = Cell::default();
    assert_eq!(c.border, "");
  }

  #[test]
  fn cell_border_at_reads_correct_side() {
    let mut c = Cell::default();
    c.border_left = Some(1);
    c.border_right = Some(2);
    c.border_top = Some(3);
    c.border_bottom = Some(4);
    assert_eq!(c.border_at(BorderSpec::Left), Some(1));
    assert_eq!(c.border_at(BorderSpec::Right), Some(2));
    assert_eq!(c.border_at(BorderSpec::Top), Some(3));
    assert_eq!(c.border_at(BorderSpec::Bottom), Some(4));
  }

  #[test]
  fn cell_border_at_none_by_default() {
    let c = Cell::default();
    assert_eq!(c.border_at(BorderSpec::Left), None);
    assert_eq!(c.border_at(BorderSpec::Right), None);
    assert_eq!(c.border_at(BorderSpec::Top), None);
    assert_eq!(c.border_at(BorderSpec::Bottom), None);
  }

  #[test]
  fn cell_partial_eq_defaults() {
    // Default cells are equal.
    let a = Cell::default();
    let b = Cell::default();
    assert_eq!(a, b);
  }

  #[test]
  fn cell_partial_eq_distinguishes_flags() {
    let mut a = Cell::default();
    let b = Cell::default();
    a.empty = true;
    assert_ne!(a, b);
  }
}
