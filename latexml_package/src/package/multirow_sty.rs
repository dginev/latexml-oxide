use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: multirow.sty.ltxml

  DefPrimitive!("\\multirowsetup", None);
  // \lx@multirow@setup: internal primitive that sets rowspan/vattach on current cell.
  // Separated from content so that content flows naturally through alignment cell boxes.
  DefPrimitive!("\\lx@multirow@setup{Float}[]{}", sub[(nrows, attachment, _width)] {
    if let Some(alignment) = lookup_alignment() {
      if let Some(data) = alignment.alignment_cell() {
        let mut data_lock = data.borrow_mut();
        if let Some(colspec) = data_lock.current_column() {
          let rowspan_f = nrows.value_f64();
          let rowspan = if rowspan_f < 0.0 {
            Warn!("unsupported", "multirow",
              "Negative row sizes for \\multirow are not yet supported.");
            1usize
          } else if rowspan_f != rowspan_f.floor() {
            Warn!("unsupported", "multirow",
              "Fractional row sizes for \\multirow are not yet supported.");
            rowspan_f as usize
          } else {
            rowspan_f as usize
          };
          colspec.rowspan = Some(rowspan);
          // Only set vattach when optional [] is explicitly provided and non-empty
          if let Some(ref att) = attachment {
            let att_str = att.to_string();
            if !att_str.trim().is_empty() {
              colspec.vattach = Some(translate_attachment(att).to_string());
            }
          }
        }
      }
    }
    Ok(())
  });
  // \multirow[vpos]{nrows}[bigstruts]{width}[fixup]{content}
  // Macro: sets up rowspan via \lx@multirow@setup, then passes content through.
  // Content stays in the token stream so alignment cell processing picks it up.
  DefMacro!("\\multirow[]{Float}[Number]{}[Dimension]{}",
    "\\lx@multirow@setup{#2}[#1]{#4}#6");
});
