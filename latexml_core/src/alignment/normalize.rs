use super::Alignment;
use super::template::Align;
use crate::BoxOps;
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::store::Stored;
use crate::digested::Digested;
use crate::list::List;
use std::collections::VecDeque;

use crate::common::float::Float;
use crate::common::numeric_ops::NumericOps;

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
pub fn normalize_cell_sizes(alignment: &mut Alignment) -> Result<()> {
  // Examines: boxes, align, vattach
  // Sets: cached_width, cached_height, cached_depth (per cell) & empty
  for row in &mut alignment.rows {
    // Do we need to account for any space in the $$row{before} or $$row{after}?
    for cell in row.get_columns_mut() {
      if let Some(ref mut boxes) = &mut cell.boxes {
        let (w, h, d, cw, _ch, _cd) = boxes.get_size(Some(stored_map!(
            "align" => cell.align.map(|a| a.char_code()), "width" => cell.width,
            "vattach" => cell.vattach.clone() )))?;
        // Debug("CELL (" . join(',', map { $_ . "=" . ToString($$cell{$_}); } qw(align width
        // vattach))     . ") size " . showSize($w,  $h,  $d)
        //     . " csize " . showSize($cw, $ch, $cd)
        //     . " Boxes=" . ToString($boxes)) if $LaTeXML::DEBUG{halign} && $LaTeXML::DEBUG{size};
        // TODO: We can't do heights and depths yet
        // || h.value_of() < 1
        // || d.value_of() < 1
        let empty = (cw.value_of() < 1
          || boxes
            .unlist_ref()
            .iter()
            .all(|tb| tb.get_property_bool("isSpace")))
          && preserved_boxes(Some(boxes)).is_empty();
        cell.cached_width = Some(w);
        cell.cached_height = Some(h);
        cell.cached_depth = Some(d);
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
pub fn normalize_mark_spans(_alignment: &mut Alignment) -> Result<()> { Ok(()) }
/// Scan for and remove empty rows
/// but copying borders and adjusting rowspan's & colspan's appropriately.
pub fn normalize_prune_rows(alignment: &mut Alignment) -> Result<()> {
  // Examines: rowspan,rowspanned, border, pseudorow, empty
  // Sets: border, rowspan
  let preserve = alignment.is_math || alignment.properties.contains_key("preserve_structure");
  // First, do rows.
  let init_rows: Vec<_> = alignment.rows.drain(..).collect();
  let mut rows = init_rows.into_iter().peekable();
  let mut filtered = VecDeque::new();
  while let Some(row) = rows.next() {
    if row.get_columns().iter().any(|cell| !cell.empty) {
      // Not empty! so keep it
      filtered.push_back(row);
    } else if let Some(next) = rows.peek_mut() {
      // Remove empty row, but copy top border to NEXT row
      if preserve {
        filtered.push_back(row);
        continue;
      } // don't remove inner rows from math EXCEPT last row!!
      // let (mut pruneh, mut pruned) = (0, 0);
      for (j, col) in row.get_columns().iter().enumerate() {
        // TODO: add cached_height and cached_depth
        // if let Some(cached_height) = col.cached_height {
        //   let chv = cached_height.value_of();
        //   if chv > pruneh { pruneh = chv; }
        // }
        // if let Some(cached_depth) = col.cached_depth {
        //   let cdv = cached_depth.value_of();
        //   if cdv > pruned {
        //     pruned = cdv;
        //   }
        // }
        // TODO: add rowspanned
        // if !row.pseudorow && col.rowspanned.is_some() {
        //         $rows[$$col{rowspanned}]{columns}[$j]{rowspan}--; }    // Decrement rowspan of
        // spanning column
        let mut converted_border = String::new();
        for border_c in col.border.chars() {
          match border_c {
            //  but convert to top
            't' | 'b' => converted_border.push('t'),
            'T' | 'B' => converted_border.push('T'),
            _ => {},
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
    } else {
      // Remove empty last row, but copy top border to bottom of prev.
      let mut prev_opt = filtered.back_mut();
      //     my $nc   = scalar(@{ $$row{columns} });
      // let (mut pruneh, mut pruned) = (0, 0);
      for (j, col) in row.get_columns().iter().enumerate() {
        // TODO:
        //  $pruneh = max($pruneh, $$col{cached_height}->value_of) if $$col{cached_height};
        //  $pruned = max($pruned, $$col{cached_depth}->value_of)  if $$col{cached_depth};
        //       if (!$$row{pseudorow} && defined $$col{rowspanned}) {
        //         $rows[$$col{rowspanned}]{columns}[$j]{rowspan}--; }    // Decrement rowspan of
        // spanning column
        let mut converted_border = String::new();
        for c in col.border.chars() {
          match c {
            't' => converted_border.push('b'), // convert to bottom
            'T' => converted_border.push('B'), // convert to bottom
            _ => {},
          };
        }
        if let Some(ref mut prev) = prev_opt {
          let ccol = &mut prev.get_columns_mut()[j];
          // TODO: rowspanned
          //       if (defined $$ccol{rowspanned}) {                        // skip to spanning
          // column if rowspanned!         $ccol = $rows[$$ccol{rowspanned}]{columns}[$j];
          // }
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
  alignment.rows = filtered;
  Ok(())
}
/// Scan for and remove empty columns
/// but copying borders and adjusting rowspan's & colspan's appropriately.
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
      let mut prunew = 0;
      let mut colspanned = None;
      for row in &mut alignment.rows {
        let mut new_border = String::new();
        let mut preserve = Vec::new();
        if let Some(col) = row.get_columns().get(j) {
          colspanned = col.colspanned;
          if let Some(w) = col.cached_width {
            prunew = prunew.max(w.value_of());
          }

          if j > 0 {
            for c in col.border.chars() {
              // mask all but left and right border
              match c {
                // convert to right
                'l' | 'r' => {
                  new_border.push('r');
                },
                // convert to right
                'L' | 'R' => {
                  new_border.push('R');
                },
                _ => {},
              }
            }
          } else {
            for c in col.border.chars() {
              // mask all but left and right border
              match c {
                // but convert to left
                'l' | 'r' => {
                  new_border.push('l');
                },
                // but convert to left
                'L' | 'R' => {
                  new_border.push('L');
                },
                _ => {},
              }
            }
          }
          preserve = preserved_boxes(col.boxes.as_ref());
        }

        // Now, remove the column
        row.get_columns_mut().remove(j);
        if let Some(col_spanned) = colspanned {
          // Decrement colspan of spanning column
          if let Some(ref mut colspan) = &mut row.get_columns_mut()[col_spanned].colspan {
            *colspan -= 1;
          }
        }
        // preserved-boxes handling moved here for mutability reasons.
        if j > 0 {
          {
            let mut prev = &mut row.get_columns_mut()[j - 1];
            if let Some(jj) = prev.colspanned {
              prev = &mut row.get_columns_mut()[jj];
            }
            if !preserve.is_empty() {
              // Copy boxes over, in case side effects?
              let mut new_boxes = prev.boxes.as_mut().map(|b| b.unlist()).unwrap_or_default();
              new_boxes.extend(preserve);
              prev.boxes = Some(Digested::from(List::new(new_boxes)));
            }
          }
        }
        // Changed: we need to first finish all work with "col" before we can mutably re-borrow
        // the "prev" column from the same row.
        if !new_border.is_empty() {
          if j > 0 {
            if let Some(prev) = row.get_columns_mut().get_mut(j - 1) {
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
      if j > 0 { // If not 1st row, add right padding to previous column
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
  Ok(())
}

pub fn normalize_sum_sizes(alignment: &mut Alignment) -> Result<()> {
  let mut rowheights = Vec::new();
  let mut colwidths = Vec::new();
  let mut colrights = Vec::new();
  let mut collefts = Vec::new();
  // Uses cell's cached_width,cached_height,cached_depth
  // Computes net row & column sizes & positions
  // add spacing between rows? Or only from \\[..] ?
  let strut = if let Some(Stored::Dimension(ref d)) = alignment.get_property("strut").as_deref() {
    *d
  } else {
    Dimension::new(0)
  };
  let hs = strut.multiply(Float::new_f64(0.7));
  let ds = strut.multiply(Float::new_f64(0.3));
  // let nrows = rows.len();

  for row in alignment.rows.iter_mut() {
    //   my $ncols = scalar(@cols);
    //   if (my $short = $ncols - scalar(@colwidths)) {    # Extend column arrays, if needed
    //     push(@colwidths, map { 0 } 1 .. $short);
    //     push(@collefts,  map { 0 } 1 .. $short);
    //     push(@colrights, map { 0 } 1 .. $short);
    //         }
    let (mut rowh, mut rowd) = (0, 0);
    let mut rowt = row.top_padding.unwrap_or_default().value_of();
    let mut rowb = row.bottom_padding.unwrap_or_default().value_of();
    //   for (my $j = 0 ; $j < $ncols ; $j++) {
    for cell in row
      .get_columns_mut()
      .iter_mut()
      .filter(|cell| !cell.skipped && cell.boxes.is_some())
    {
      let (mut colwidths_j, mut collefts_j, mut colrights_j) = (0, 0, 0);
      let w = cell.cached_width.unwrap_or_default().value_of();
      let h = cell.cached_height.unwrap_or_default().value_of();
      let d = cell.cached_depth.unwrap_or_default().value_of();
      let t = cell.top_padding.unwrap_or_default();
      let b = cell.bottom_padding.unwrap_or_default();
      let r = cell.right_padding.unwrap_or_default();
      let l = cell.left_padding.unwrap_or_default();

      if cell.colspan.unwrap_or(1) == 1 {
        if w > 0 {
          colwidths_j = w.max(colwidths_j);
        }
        if l > 0 {
          collefts_j = l.max(collefts_j);
        }
        if r > 0 {
          colrights_j = r.max(colrights_j);
        }
      }
      if cell.rowspan.unwrap_or(1) == 1 {
        rowh = rowh.max(h);
        rowd = rowd.max(d);
        rowt = rowt.max(t as i64);
        rowb = rowb.max(b as i64);
      } //else {} // Ditto spanned rows
      colwidths.push(colwidths_j);
      collefts.push(collefts_j);
      colrights.push(colrights_j);
    }
    row.cached_height = Some(Dimension::new(rowh).larger(hs));
    row.cached_depth = Some(Dimension::new(rowd).larger(ds));
    row.top_padding = Some(Dimension::new(rowt));
    row.bottom_padding = Some(Dimension::new(rowb));
    // NOTE: Should be storing column widths to; individually, as well as per-column!
    rowheights.push(rowh + rowd);
  } // somehow our heights are way too short????
  // Now compute the positions
  let mut rowpos = Vec::new();
  let mut colpos = Vec::new();
  let mut y = 0;
  // Row & column positions: left,top
  for row in alignment.rows.iter().take(rowheights.len()) {
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
  let mut x = 0i64;
  for ((w, l), r) in colwidths.iter().zip(collefts.iter()).zip(colrights.iter()) {
    x += *l as i64;
    colpos.push(Dimension::new(x));
    x += w;
    x += *r as i64;
  }
  alignment.cached_width = Some(Dimension::new(x));
  // or account for vertical position of array as a whole)?
  alignment.cached_height = Some(Dimension::new(y));
  alignment.cached_depth = Some(Dimension::new(0));
  alignment.column_widths = colwidths.iter().map(|v| Dimension::new(*v)).collect();
  alignment.row_heights = rowheights.iter().map(|v| Dimension::new(*v)).collect();

  for (i, row) in alignment.rows.iter_mut().take(rowheights.len()).enumerate() {
    row.x = Some(colpos[0]);
    row.y = Some(rowpos[i]);
    row.cached_width = Some(Dimension::new(x));
    for ((cell, colwidth), colposx) in row
      .get_columns_mut()
      .iter_mut()
      .zip(colwidths.iter())
      .zip(colpos.iter())
    {
      let a = cell.align.unwrap_or(Align::Left);
      // Adjust position according to alignment
      // If these are defined
      let cached_width = cell.cached_width.unwrap_or_default().value_of();
      let colx = if *colwidth > 0 && cached_width > 0 && a != Align::Left {
        let dx = Dimension::new(colwidth - cached_width);
        match a {
          Align::Center => colposx.add(dx.multiply(Float::new_f64(0.5))),
          Align::Right => colposx.add(dx),
          _ => *colposx,
        }
      } else {
        *colposx
      };
      cell.x = Some(colx);
      cell.y = Some(rowpos[i]);
      //     Debug("CELL[$j,$i] " . showSize($$cell{cached_width}, $$cell{cached_height},
      // $$cell{cached_depth})         . " @ " . ToString($$cell{x}) . "," .
      // ToString($$cell{y})         . " w/ " . join(',', map { $_ . '=' .
      // ToString($$cell{$_}); }           (qw(align vattach skipped colspan rowspan))))
      //       if $LaTeXML::DEBUG{halign} && $LaTeXML::DEBUG{size};
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
