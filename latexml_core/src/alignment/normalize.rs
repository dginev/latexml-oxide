use super::Alignment;
use super::cell::Cell;
use super::template::Align;
use crate::BoxOps;
use crate::Tokens;
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::store::Stored;
use crate::digested::Digested;
use crate::list::List;
use std::collections::VecDeque;

use crate::common::float::Float;
use crate::common::numeric_ops::{NumericOps, UNITY};
use crate::common::object::Object;
use std::str::FromStr;

/// Normalize an alignment before construction
///
/// * consolodating column & row spanning information
/// * scanning for empty rows & columns and collapsing them (while accounting for spanning, and
///   copying borders appropriately)
///
/// Note that a trailing \\ in allignment (often needed to effect \hline)
///    causes an empty row at the end. Other fancy layout fine-tuning often
///    involves adding extra rows & columsn for spacing.  HTML's table model
///    is more forgiving that TeX's, so we don't need these extras
///    and, in fact, they often mess up the html layout!
///
/// However, math alignments, and those with expected structure (eg. eqnarray)
///    should generally NOT have rows & columns collapsed --- except the last row!
///
/// Also note the inconsistency between TeX & HTML's table models regarding spans.
///
/// \multicolumn creates a cell that covers a certain number of columns
///     which are then omitted from the LaTeX AND the HTML.
///
/// OTOH, \multirow creates a cell which overlaps following rows!
///
/// The & is still needed to allocate the cells in those rows.
/// And in fact they need not even be empty! TeX will just pile them up!
///
/// However, in HTML the spanned rows ARE omitted!
pub fn normalize_alignment(alignment: &mut Alignment) -> Result<()> {
  if !alignment.is_normalized {
    normalize_cell_sizes(alignment)?;
    normalize_mark_spans(alignment)?;
    normalize_prune_rows(alignment)?;
    normalize_prune_columns(alignment)?;
    normalize_sum_sizes(alignment)?;
    alignment.is_normalized = true;
  }
  Ok(())
}
/// Compute (approximate) sizes of all cells
/// Perl: normalize_cell_sizes (Alignment.pm L423-480)
pub fn normalize_cell_sizes(alignment: &mut Alignment) -> Result<()> {
  // Examines: boxes, align, vattach, lspaces, rspaces
  // Sets: cached_width, cached_height, cached_depth, lpadding, rpadding (per cell) & empty
  for row in &mut alignment.rows {
    for cell in row.get_columns_mut() {
      if let Some(ref mut boxes) = &mut cell.boxes {
        let (w, mut h, mut d, cw, ch, cd) = boxes.get_size(Some(stored_map!(
            "align" => cell.align.as_ref().map(|a| a.char_code()), "width" => cell.width,
            "vattach" => cell.vattach.clone() )))?;
        let mut fullw = cw;
        // Perl L441-450: lspaces/rspaces size computation
        let mut lpad = None;
        let mut rpad = None;
        if let Some(ref mut lspaces) = cell.lspaces {
          let (lw, lh, ld, ..) = lspaces.get_size(None)?;
          if lw.value_of() != 0 {
            fullw = if fullw.value_of() != 0 {
              fullw.add(lw)
            } else {
              lw
            };
            lpad = Some(lw);
          }
          if lh.value_of() != 0 {
            h = if h.value_of() != 0 { h.larger(lh) } else { lh };
          }
          if ld.value_of() != 0 {
            d = if d.value_of() != 0 { d.larger(ld) } else { ld };
          }
        }
        if let Some(ref mut rspaces) = cell.rspaces {
          let (rw, rh, rd, ..) = rspaces.get_size(None)?;
          if rw.value_of() != 0 {
            fullw = if fullw.value_of() != 0 {
              fullw.add(rw)
            } else {
              rw
            };
            rpad = Some(rw);
          }
          if rh.value_of() != 0 {
            h = if h.value_of() != 0 { h.larger(rh) } else { rh };
          }
          if rd.value_of() != 0 {
            d = if d.value_of() != 0 { d.larger(rd) } else { rd };
          }
        }
        // Perl L452-456: check isrule
        let boxes_list = boxes.unlist_ref();
        let isrule = !boxes_list.is_empty()
          && boxes_list.iter().all(|b| {
            b.get_property_bool("isHorizontalRule")
              || b.get_property_bool("isVerticalRule")
              || b.get_property_bool("alignmentSkippable")
              || b.is_comment()
          });
        // Perl L457-462: ((fullw_small AND ch_cd_small) OR isrule) AND !preserved
        let empty = ((fullw.value_of() < 1 && (ch.value_of() < 1 && cd.value_of() < 1)) || isrule)
          && preserved_boxes(Some(boxes)).is_empty();
        let skippable = empty || boxes.is_skippable();
        cell.cached_width = Some(w);
        cell.cached_height = Some(h);
        cell.cached_depth = Some(d);
        // Perl L466-467: set lpadding/rpadding from lspaces/rspaces
        if let Some(lp) = lpad {
          cell.left_padding = Some(lp.value_of() as usize);
        }
        if let Some(rp) = rpad {
          cell.right_padding = Some(rp.value_of() as usize);
        }
        cell.empty = empty;
        cell.skippable = skippable;
        if skippable {
          cell.align = None;
        }
      } else {
        cell.empty = true;
        cell.skippable = true;
        cell.align = None; // Perl: skippable cells don't get align (L470)
      }
    }
  }
  Ok(())
}
/// Mark any cells that are covered by rowspan or colspan.
/// Perl: normalize_mark_spans (Alignment.pm L667-726)
pub fn normalize_mark_spans(alignment: &mut Alignment) -> Result<()> {
  // Examines: rowspan, colspan, pseudorow, empty
  // Sets: skipped, colspanned, rowspanned
  let nrows = alignment.rows.len();
  for i in 0..nrows {
    let ncols = alignment.rows[i].get_columns().len();
    for j in 0..ncols {
      let nc = alignment.rows[i].get_columns()[j].colspan.unwrap_or(1);
      // scan the row for spanned columns that also span rows! Move rowspan to leading column
      if nc > 1 {
        let mut copied_rowspan = None;
        for jj in (j + 1)..std::cmp::min(j + nc, ncols) {
          let ccol = &mut alignment.rows[i].get_columns_mut()[jj];
          ccol.skipped = true;
          ccol.colspanned = Some(j); // note that this column is spanned by column j
          if let Some(cnr) = ccol.rowspan {
            // If this spanned column has rowspan, copy to initial column
            copied_rowspan = Some(cnr);
          }
        }
        if let Some(cnr) = copied_rowspan {
          alignment.rows[i].get_columns_mut()[j].rowspan = Some(cnr);
        }
      }
      let nr = alignment.rows[i].get_columns()[j].rowspan.unwrap_or(1);
      if nr > 1 {
        // If this column spans rows
        let ncspan = alignment.rows[i].get_columns()[j].colspan.unwrap_or(1);
        let mut nrc = nr;
        let mut actual_nr = nr;
        let mut ii = i + 1;
        while nrc > 0 && ii < nrows {
          // Prescan the columns to make sure they're empty!
          let mut row_empty = true;
          let rrow = &alignment.rows[ii];
          let row_pseudo = rrow.is_pseudo();
          if !row_pseudo {
            nrc -= 1;
          }
          for jj in j..std::cmp::min(j + ncspan, rrow.get_columns().len()) {
            if !rrow.get_columns()[jj].skippable {
              row_empty = false;
            }
          }
          if nrc == 0 {
            // Done counting
          } else if !row_empty {
            // Perl: truncate rowspan if covering non-empty cells
            let old_nr = actual_nr;
            actual_nr -= nrc;
            nrc = 0;
            Info!(
              "unexpected",
              "rowspan",
              s!(
                "Rowspan {} in cell({},{}) covers non-empty cells; truncating to {}",
                old_nr,
                i,
                j,
                actual_nr
              )
            );
          } else {
            // Mark all spanned columns in this row as skipped
            for jj in j..std::cmp::min(j + ncspan, alignment.rows[ii].get_columns().len()) {
              let ccol = &mut alignment.rows[ii].get_columns_mut()[jj];
              ccol.skipped = true;
              ccol.rowspanned = Some(i); // note that this column is spanned by row i
            }
          }
          ii += 1;
        }
        // Update rowspan if it was truncated
        if actual_nr != nr {
          alignment.rows[i].get_columns_mut()[j].rowspan = Some(actual_nr);
        }
        // Copy bottom border from last skipped row to the spanning cell
        if ii > i + 1 {
          let last_row_idx = ii - 1;
          let mut sborder = String::new();
          for jj in j..std::cmp::min(j + nc, alignment.rows[last_row_idx].get_columns().len()) {
            let border = &alignment.rows[last_row_idx].get_columns()[jj].border;
            if sborder.is_empty() {
              // mask all but bottom border
              let bottom_only: String = border.chars().filter(|c| *c == 'b' || *c == 'B').collect();
              if !bottom_only.is_empty() {
                sborder = bottom_only;
              }
            }
          }
          if !sborder.is_empty() {
            alignment.rows[i].get_columns_mut()[j]
              .border
              .push_str(&sborder);
          }
        }
      }
    }
  }
  Ok(())
}
/// Scan for and remove empty rows
/// but copying borders and adjusting rowspan's & colspan's appropriately.
/// Perl: normalize_prune_rows (Alignment.pm L730-799)
pub fn normalize_prune_rows(alignment: &mut Alignment) -> Result<()> {
  // Examines: rowspan,rowspanned, border, pseudorow, empty
  // Sets: border, rowspan
  let preserve = alignment.is_math || alignment.properties.contains_key("preserve_structure");
  // We need indexed access for rowspan decrement, so collect into a Vec first
  let init_rows: Vec<_> = alignment.rows.drain(..).collect();
  // We need to be able to mutate filtered rows by index for rowspan decrement,
  // so we track: original_index -> filtered_index mapping is complex.
  // Instead, use a simpler approach: collect prunable indices first, then do border copies.
  let nrows = init_rows.len();
  let mut keep = vec![true; nrows];

  // First pass: determine which rows are prunable
  for i in 0..nrows {
    let row = &init_rows[i];
    let next = if i + 1 < nrows {
      Some(&init_rows[i + 1])
    } else {
      None
    };
    let mut prunable = true;
    let mut check_bracketting = false;
    let is_pseudo = row.is_pseudo();
    for c in row.get_columns().iter() {
      if c.skippable && !c.empty {
        check_bracketting = true;
      }
      // Perl: cells with lspaces (from \lx@intercol) have fullw > 0, making them
      // empty=false but skippable=true. Rust doesn't populate lspaces from template,
      // so we check template tokens directly: if before/after has \lx@intercol AND
      // this is not a pseudorow (\hline creates pseudorows which should always be prunable),
      // treat as "non-empty" for bracketting purposes.
      if c.skippable && c.empty && !is_pseudo && cell_has_intercol(c) {
        check_bracketting = true;
      }
      if !c.skippable {
        prunable = false;
      }
    }
    if prunable && check_bracketting {
      // Check for top borders: include borders from preceding prunable rows
      // that would be transferred during the second pass (matching Perl's
      // single-pass where pseudorow borders are already merged).
      let mut has_top = row
        .get_columns()
        .iter()
        .any(|c| c.border.contains('t') || c.border.contains('T'));
      if !has_top && i > 0 && !keep[i - 1] {
        has_top = init_rows[i - 1]
          .get_columns()
          .iter()
          .any(|c| c.border.contains('t') || c.border.contains('T'));
      }
      if has_top {
        if let Some(next_row) = next {
          let next_has_top = next_row
            .get_columns()
            .iter()
            .any(|c| c.border.contains('t') || c.border.contains('T'));
          if next_has_top {
            prunable = false;
          }
        } else {
          let has_bottom = row
            .get_columns()
            .iter()
            .any(|c| c.border.contains('b') || c.border.contains('B'));
          if has_bottom {
            prunable = false;
          }
        }
      }
    }
    if prunable {
      // Check preserve: don't remove inner rows from math EXCEPT last row
      if preserve && i + 1 < nrows {
        prunable = false;
      }
    }
    keep[i] = !prunable;
  }

  // Second pass: copy borders and handle rowspan decrements
  let mut rows: Vec<_> = init_rows;
  for i in 0..nrows {
    if keep[i] {
      continue;
    }
    // Find next kept row
    let next_kept = (i + 1..nrows).find(|&k| keep[k]);
    // Find prev kept row
    let prev_kept = (0..i).rev().find(|&k| keep[k]);

    // Perl L760-764: track pruned height/depth
    let mut pruneh: i64 = 0;
    let mut pruned: i64 = 0;
    let nc = rows[i].get_columns().len();

    if let Some(next_idx) = next_kept {
      // Remove empty row, copy border to NEXT row
      for j in 0..nc {
        let col = &rows[i].get_columns()[j];
        if let Some(ch) = col.cached_height {
          pruneh = pruneh.max(ch.value_of());
        }
        if let Some(cd) = col.cached_depth {
          pruned = pruned.max(cd.value_of());
        }
        // Perl L765-766: decrement rowspan of spanning column
        if !rows[i].is_pseudo() {
          if let Some(rowspanned_idx) = col.rowspanned {
            // Find the spanning row in filtered rows and decrement its rowspan
            if let Some(spanning_col) = rows[rowspanned_idx].get_columns_mut().get_mut(j) {
              if let Some(ref mut rs) = spanning_col.rowspan {
                if *rs > 1 {
                  *rs -= 1;
                }
              }
            }
          }
        }
        // Perl L767-771: mask all but top & bottom border, convert to top
        let mut converted_border = String::new();
        for border_c in rows[i].get_columns()[j].border.chars() {
          match border_c {
            't' | 'b' => converted_border.push('t'),
            'T' | 'B' => converted_border.push('T'),
            _ => {},
          };
        }
        if !converted_border.is_empty() && j < rows[next_idx].get_columns().len() {
          rows[next_idx].get_columns_mut()[j]
            .border
            .push_str(&converted_border);
        }
      }
      // Perl L773: save padding
      let prune_both = pruneh + pruned;
      if prune_both > 0 {
        rows[next_idx].top_padding = Some(Dimension::new(prune_both));
      }
    } else if let Some(prev_idx) = prev_kept {
      // Remove empty last row, copy top border to bottom of prev
      for j in 0..nc {
        let col = &rows[i].get_columns()[j];
        if let Some(ch) = col.cached_height {
          pruneh = pruneh.max(ch.value_of());
        }
        if let Some(cd) = col.cached_depth {
          pruned = pruned.max(cd.value_of());
        }
        // Perl L784-785: decrement rowspan
        if !rows[i].is_pseudo() {
          if let Some(rowspanned_idx) = col.rowspanned {
            if let Some(spanning_col) = rows[rowspanned_idx].get_columns_mut().get_mut(j) {
              if let Some(ref mut rs) = spanning_col.rowspan {
                if *rs > 1 {
                  *rs -= 1;
                }
              }
            }
          }
        }
        // Perl L787-789: mask all but top border, convert to bottom
        let mut converted_border = String::new();
        for c in rows[i].get_columns()[j].border.chars() {
          match c {
            't' => converted_border.push('b'),
            'T' => converted_border.push('B'),
            _ => {},
          };
        }
        if !converted_border.is_empty() && j < rows[prev_idx].get_columns().len() {
          // Perl L790-792: follow rowspanned pointer
          let ccol = &rows[prev_idx].get_columns()[j];
          if let Some(rowspanned_idx) = ccol.rowspanned {
            // Skip to spanning column
            rows[rowspanned_idx].get_columns_mut()[j]
              .border
              .push_str(&converted_border);
          } else {
            rows[prev_idx].get_columns_mut()[j]
              .border
              .push_str(&converted_border);
          }
        }
      }
      // Perl L794: save padding
      let prune_both = pruneh + pruned;
      if prune_both > 0 {
        rows[prev_idx].bottom_padding = Some(Dimension::new(prune_both));
      }
    }
  }

  // Collect kept rows
  let mut filtered = VecDeque::new();
  for (i, row) in rows.into_iter().enumerate() {
    if keep[i] {
      filtered.push_back(row);
    }
  }
  alignment.rows = filtered;
  Ok(())
}

/// Check if a cell has \lx@intercol in its before or after template tokens.
/// In Perl, \lx@intercol presence means lspaces/rspaces are populated with
/// intercolumn glue (tabcolsep), making fullw > 0 even for empty cells.
/// Rust doesn't populate lspaces from template, so we check tokens directly.
fn cell_has_intercol(cell: &Cell) -> bool {
  fn has_intercol(tokens: &Option<Tokens>) -> bool {
    if let Some(ref toks) = tokens {
      for tok in toks.unlist_ref() {
        let s = tok.to_string();
        if s == "\\lx@intercol" || s.contains("intercol") {
          return true;
        }
      }
    }
    false
  }
  has_intercol(&cell.before) || has_intercol(&cell.after)
}

/// Scan for and remove empty columns
/// but copying borders and adjusting rowspan's & colspan's appropriately.
/// Perl: normalize_prune_columns (Alignment.pm L801-857)
pub fn normalize_prune_columns(alignment: &mut Alignment) -> Result<()> {
  if alignment.is_math || alignment.properties.contains_key("preserve_structure") {
    // Don't remove empty columns from math.
    return Ok(());
  }
  // Now prune empty columns.
  let mut nc = 0;
  for row in alignment.rows.iter() {
    let n = row.get_columns().len();
    if n > nc {
      nc = n;
    }
  }
  for j in (0..nc).rev() {
    // Prune from RIGHT!
    let j_column_is_empty = alignment.rows.iter().all(|row| {
      row
        .get_columns()
        .get(j)
        .map(|col| col.empty)
        .unwrap_or(true)
    });
    if j_column_is_empty {
      // Empty!
      let mut prunew: i64 = 0;
      for row in &mut alignment.rows {
        let mut new_border = String::new();
        let mut preserve = Vec::new();
        let mut colspanned = None;
        let mut lspaces_to_copy = None;
        if let Some(col) = row.get_columns().get(j) {
          colspanned = col.colspanned;
          if let Some(w) = col.cached_width {
            prunew = prunew.max(w.value_of());
          }
          // Perl L819-843: border handling
          if j > 0 {
            for c in col.border.chars() {
              match c {
                'l' | 'r' => new_border.push('r'),
                'L' | 'R' => new_border.push('R'),
                _ => {},
              }
            }
          } else {
            for c in col.border.chars() {
              match c {
                'l' | 'r' => new_border.push('l'),
                'L' | 'R' => new_border.push('L'),
                _ => {},
              }
            }
          }
          preserve = preserved_boxes(col.boxes.as_ref());
          // Perl L829: copy lspaces for transfer to prev's rspaces
          lspaces_to_copy = col.lspaces.clone();
        }

        // Perl L816-817: decrement colspan of spanning column
        if let Some(col_spanned) = colspanned {
          if let Some(spanning_col) = row.get_columns_mut().get_mut(col_spanned) {
            if let Some(ref mut colspan) = spanning_col.colspan {
              if *colspan > 1 {
                *colspan -= 1;
              }
            }
          }
        }

        // Now, remove the column
        if j < row.get_columns().len() {
          row.get_columns_mut().remove(j);
        }

        if j > 0 {
          // Perl L821-833: transfer border, lspaces, and boxes to prev column
          // Follow colspanned pointer
          let prev_idx = {
            let mut idx = j - 1;
            if let Some(jj) = row.get_columns().get(idx).and_then(|c| c.colspanned) {
              idx = jj;
            }
            idx
          };
          if let Some(prev) = row.get_columns_mut().get_mut(prev_idx) {
            if !new_border.is_empty() {
              prev.border.push_str(&new_border);
            }
            // Perl L829-830: copy lspaces to prev's rspaces and update rpadding
            if let Some(ls) = lspaces_to_copy {
              let new_rspaces = if let Some(ref existing) = prev.rspaces {
                let mut items = existing.unlist();
                items.extend(ls.unlist());
                Digested::from(List::new(items))
              } else {
                ls
              };
              // Compute rpadding from combined rspaces
              if let Ok((rw, ..)) = new_rspaces.clone().get_size(None) {
                if rw.value_of() != 0 {
                  prev.right_padding = Some(rw.value_of() as usize);
                }
              }
              prev.rspaces = Some(new_rspaces);
            }
            // Perl L831-833: copy boxes over
            if !preserve.is_empty() {
              let mut new_boxes = prev.boxes.as_mut().map(|b| b.unlist()).unwrap_or_default();
              new_boxes.extend(preserve);
              prev.boxes = Some(Digested::from(List::new(new_boxes)));
            }
          }
        } else {
          // j==0: transfer to next column (now at index 0 after remove)
          if let Some(next) = row.get_columns_mut().get_mut(0) {
            if !new_border.is_empty() {
              next.border.push_str(&new_border);
            }
            // Perl L840-842: copy boxes to next
            if !preserve.is_empty() {
              let mut new_boxes = preserve;
              new_boxes.extend(next.boxes.as_mut().map(|b| b.unlist()).unwrap_or_default());
              next.boxes = Some(Digested::from(List::new(new_boxes)));
            }
          }
        }
      }
      // Perl L847-854: add padding
      let prunew_dim = Dimension::new(prunew);
      if j > 0 {
        for row in &mut alignment.rows {
          if let Some(col) = row.get_columns_mut().get_mut(j - 1) {
            col.right_padding = Some(
              col
                .right_padding
                .map(|rp| Dimension::new(rp as i64).add(prunew_dim).value_of() as usize)
                .unwrap_or(prunew as usize),
            );
          }
        }
      } else {
        for row in &mut alignment.rows {
          if let Some(col) = row.get_columns_mut().get_mut(0) {
            col.left_padding = Some(
              col
                .left_padding
                .map(|lp| Dimension::new(lp as i64).add(prunew_dim).value_of() as usize)
                .unwrap_or(prunew as usize),
            );
          }
        }
      }
    }
  }
  Ok(())
}

/// Perl: normalize_sum_sizes (Alignment.pm L514-664)
pub fn normalize_sum_sizes(alignment: &mut Alignment) -> Result<()> {
  let mut rowheights: Vec<i64> = Vec::new();
  let mut rowdepths: Vec<i64> = Vec::new();
  // Perl: indexed arrays for per-column max values
  let mut colwidths: Vec<i64> = Vec::new();
  let mut colrights: Vec<i64> = Vec::new();
  let mut collefts: Vec<i64> = Vec::new();
  // Uses cell's cached_width,cached_height,cached_depth
  // Computes net row & column sizes & positions
  let strut = match alignment.get_property("strut").as_deref() {
    Some(Stored::Dimension(ref d)) => *d,
    Some(Stored::Glue(ref g)) => Dimension::new(g.value_of()),
    Some(Stored::MuGlue(ref g)) => Dimension::new(g.value_of()),
    _ => Dimension::new(0),
  };
  // Perl: Glue->new($pts * 0.7) uses kround (adds 0.5 before int), not plain truncation
  let strut_val = strut.value_of() as f64;
  let hs = Dimension::new((strut_val * 0.7 + 0.5).floor() as i64);
  let ds = Dimension::new((strut_val * 0.3 + 0.5).floor() as i64);
  let is_latex = alignment.properties.contains_key("isLaTeX");
  let nrows = alignment.rows.len();

  for (i, row) in alignment.rows.iter_mut().enumerate() {
    let ncols = row.get_columns().len();
    // Perl L534-537: extend column arrays if needed
    if ncols > colwidths.len() {
      colwidths.resize(ncols, 0);
      collefts.resize(ncols, 0);
      colrights.resize(ncols, 0);
    }
    let (mut rowh, mut rowd): (i64, i64) = (0, 0);
    let mut rowt = row.top_padding.unwrap_or_default().value_of();
    let mut rowb = row.bottom_padding.unwrap_or_default().value_of();
    let (mut bordert, mut borderb) = (false, false);
    // Perl L542-577: iterate over columns by index
    for j in 0..ncols {
      let cell = &row.get_columns()[j];
      if cell.skipped || cell.boxes.is_none() {
        continue;
      }
      let w = cell.cached_width.map(|d| d.value_of()).unwrap_or(0);
      let h = cell.cached_height.map(|d| d.value_of()).unwrap_or(0);
      let d = cell.cached_depth.map(|d| d.value_of()).unwrap_or(0);
      let t = cell.top_padding.unwrap_or(0) as i64;
      let b = cell.bottom_padding.unwrap_or(0) as i64;
      let r = cell.right_padding.unwrap_or(0) as i64;
      let l = cell.left_padding.unwrap_or(0) as i64;
      let cs = cell.colspan.unwrap_or(1);
      // Perl L555-567: column width tracking
      if cs == 1 {
        if w > 0 {
          colwidths[j] = colwidths[j].max(w);
        }
        if l > 0 {
          collefts[j] = collefts[j].max(l);
        }
        if r > 0 {
          colrights[j] = colrights[j].max(r);
        }
      } else {
        // Perl L559-567: divide up spanned columns
        let inner_w = w - (cs as i64 - 1) * l - (cs as i64 - 1) * r;
        let per_col = if cs > 0 { inner_w / cs as i64 } else { 0 };
        let per_l = if cs > 0 { l / cs as i64 } else { 0 };
        let per_r = if cs > 0 { r / cs as i64 } else { 0 };
        for jj in j..std::cmp::min(j + cs, ncols) {
          if w > 0 {
            colwidths[jj] = colwidths[jj].max(per_col);
          }
          if l > 0 {
            collefts[jj] = collefts[jj].max(per_l);
          }
          if r > 0 {
            colrights[jj] = colrights[jj].max(per_r);
          }
        }
      }
      // Perl L569-573: row height/depth
      if cell.rowspan.unwrap_or(1) == 1 {
        rowh = rowh.max(h);
        rowd = rowd.max(d);
        rowt = rowt.max(t);
        rowb = rowb.max(b);
      }
      // Perl L575-577: border detection
      if !cell.border.is_empty() {
        if cell.border.contains('t') || cell.border.contains('T') {
          bordert = true;
        }
        if cell.border.contains('b') || cell.border.contains('B') {
          borderb = true;
        }
      }
    }
    // Perl L579-586: first/last row special handling (non-LaTeX only)
    if i == 0 && !is_latex {
      row.cached_height = Some(Dimension::new(rowh));
    } else {
      row.cached_height = Some(Dimension::new(rowh).larger(hs));
    }
    if i == nrows - 1 && !is_latex {
      row.cached_depth = Some(Dimension::new(rowd));
    } else {
      row.cached_depth = Some(Dimension::new(rowd).larger(ds));
    }
    // Perl L587-588: border padding (0.4 * UNITY)
    let border_pad = (0.4 * UNITY as f64) as i64;
    row.top_padding = Some(Dimension::new(rowt + if bordert { border_pad } else { 0 }));
    row.bottom_padding = Some(Dimension::new(rowb + if borderb { border_pad } else { 0 }));
    // Perl L590-591: store row height and depth separately
    rowdepths.push(row.cached_depth.unwrap().value_of());
    rowheights.push(row.cached_height.unwrap().value_of());
  }
  // Now compute the positions — one entry per row/col.
  let mut rowpos = Vec::with_capacity(alignment.rows.len());
  let mut colpos = Vec::with_capacity(
    alignment
      .rows
      .front()
      .map(|r| r.get_columns().len())
      .unwrap_or(0),
  );
  let mut y: i64 = 0;
  // Row & column positions: left,top
  for (i, row) in alignment.rows.iter().enumerate() {
    if i >= rowheights.len() {
      break;
    }
    if let Some(tp) = row.top_padding {
      y += tp.value_of();
    }
    rowpos.push(Dimension::new(y));
    if let Some(ch) = row.cached_height {
      y += ch.value_of();
    }
    if let Some(cd) = row.cached_depth {
      y += cd.value_of();
    }
    if let Some(bp) = row.bottom_padding {
      y += bp.value_of();
    }
  }
  let mut x: i64 = 0;
  for j in 0..colwidths.len() {
    x += collefts[j];
    colpos.push(Dimension::new(x));
    x += colwidths[j];
    x += colrights[j];
  }
  // Perl L610: vattach from $$self{properties}{attributes}{vattach}
  // In Rust, xml_attributes holds what Perl calls properties.attributes
  let vattach = alignment
    .get_xml_attributes_mut()
    .get("vattach")
    .cloned()
    .unwrap_or_else(|| "middle".to_string());
  alignment.cached_width = Some(Dimension::new(x));
  match vattach.as_str() {
    "top" => {
      let h = rowheights.first().copied().unwrap_or(0);
      alignment.cached_height = Some(Dimension::new(h));
      alignment.cached_depth = Some(Dimension::new(y - h));
    },
    "bottom" => {
      let d = rowdepths.last().copied().unwrap_or(0);
      alignment.cached_height = Some(Dimension::new(y - d));
      alignment.cached_depth = Some(Dimension::new(d));
    },
    _ => {
      // middle (default)
      // Perl L622-623: math axis approximation
      let c = {
        use crate::state::with_value_sym;

        with_value_sym(crate::pin!("font"), |v_opt| {
          v_opt.and_then(|v| {
            if let Stored::Font(ref f) = v {
              f.get_size().map(|s| (s * UNITY as f64) as i64 / 2)
            } else {
              None
            }
          })
        })
        .unwrap_or_else(|| {
          Dimension::from_str("1ex")
            .map(|d| d.value_of())
            .unwrap_or(0)
        })
      };
      alignment.cached_height = Some(Dimension::new((y + c) / 2));
      alignment.cached_depth = Some(Dimension::new((y - c) / 2));
    },
  }
  alignment.column_widths = colwidths.iter().map(|v| Dimension::new(*v)).collect();
  alignment.row_heights = rowheights.iter().map(|v| Dimension::new(*v)).collect();
  alignment.row_depths = rowdepths.iter().map(|v| Dimension::new(*v)).collect();

  if colpos.is_empty() {
    return Ok(());
  }
  for (i, row) in alignment.rows.iter_mut().enumerate() {
    if i >= rowheights.len() {
      break;
    }
    row.x = Some(colpos[0]);
    row.y = Some(rowpos[i]);
    row.cached_width = Some(Dimension::new(x));
    let ncols = row.get_columns().len();
    for j in 0..ncols {
      let cell = &row.get_columns()[j];
      let a = cell.align.clone().unwrap_or(Align::Left);
      let cached_width = cell.cached_width.unwrap_or_default().value_of();
      let colposx = colpos.get(j).copied().unwrap_or_default();
      let colwidth = colwidths.get(j).copied().unwrap_or(0);
      // Perl L644-647: adjust position according to alignment
      let colx = if colwidth > 0 && cached_width > 0 && a != Align::Left {
        let dx = Dimension::new(colwidth - cached_width);
        match a {
          Align::Center | Align::Char(_) => colposx.add(dx.multiply(Float::new_f64(0.5))),
          Align::Right => colposx.add(dx),
          _ => colposx,
        }
      } else {
        colposx
      };
      let cell = &mut row.get_columns_mut()[j];
      cell.x = Some(colx);
      cell.y = Some(rowpos[i]);
    }
  }

  Ok(())
}

fn preserved_boxes(boxes_opt: Option<&Digested>) -> Vec<Digested> {
  match boxes_opt {
    Some(boxes) => boxes
      .unlist()
      .into_iter()
      .filter(|tb| tb.get_property_bool("alignmentPreserve"))
      .collect(),
    None => Vec::new(),
  }
}
