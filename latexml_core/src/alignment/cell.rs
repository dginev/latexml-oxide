use super::template::{Align, BorderSpec, ColumnSpec};
use crate::common::dimension::Dimension;
use crate::common::glue::Glue;
use crate::digested::Digested;
use crate::tokens::Tokens;
use libxml::tree::Node;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Cell {
  pub empty:           bool,
  pub skippable:       bool,
  pub omitted:         bool,
  pub skipped:         bool,
  pub thead_in_row:    bool,
  pub thead_in_column: bool,
  pub before:          Option<Tokens>,
  pub after:           Option<Tokens>,
  pub align:           Option<Align>,
  pub width:           Option<Dimension>,
  pub height:          Option<Dimension>,
  pub depth:           Option<Dimension>,
  pub cached_width:    Option<Dimension>,
  pub cached_height:   Option<Dimension>,
  pub cached_depth:    Option<Dimension>,
  pub colspan:         Option<usize>,
  pub colspanned:      Option<usize>,
  pub rowspan:         Option<usize>,
  pub rowspanned:      Option<usize>,
  pub boxes:           Option<Digested>,
  pub cell_type:       Option<char>,
  pub content_class:   Option<ColumnSpec>,
  pub content_length:  Option<usize>,
  pub border_left:     Option<usize>,
  pub border_right:    Option<usize>,
  pub border_top:      Option<usize>,
  pub border_bottom:   Option<usize>,
  pub top_padding:     Option<usize>,
  pub bottom_padding:  Option<usize>,
  pub right_padding:   Option<usize>,
  pub left_padding:    Option<usize>,
  pub vattach:         Option<String>,
  pub cell:            Option<Node>,
  pub x:               Option<Dimension>,
  pub y:               Option<Dimension>,
  pub border:          String,
  /// Perl: $$colspec{tabskip} — intercolumn glue from \halign template
  pub tabskip:         Option<Glue>,
  /// Perl: $$colspec{lspaces} — left-side spacing (incl. tabskip)
  pub lspaces:         Option<Digested>,
  /// Perl: $$colspec{rspaces} — right-side spacing
  pub rspaces:         Option<Digested>,
  /// Perl: $$cell{class} — CSS class for the cell
  pub class:           Option<String>,
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
