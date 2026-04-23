use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: multirow.sty.ltxml

  DefPrimitive!("\\multirowsetup", None);
  // \multirow: structural split into DefMacro wrapper + internal DefPrimitive
  // setup (WISDOM #41 2-layer pattern). Perl's single DefPrimitive digests
  // all args; Rust separates the alignment-cell state mutation (primitive,
  // below) from the \hbox-wrapped content flow (DefMacro at :52). The
  // split lets content flow naturally through alignment cell boxes and
  // enables the text-mode \hbox wrap used by 1004.2626 Table 6.
  //
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
  //
  // Perl multirow.sty.ltxml L38 wraps content in `\hbox{\multirowsetup #6}`
  // and digests the whole thing. The \hbox forces text mode so nested
  // `$…$` cleanly switches into math — otherwise in array cell context
  // (outer math), the inner `$` toggles math OFF, landing the content in
  // text mode with script errors. arxiv 1004.2626 Table 6 was the witness.
  DefMacro!("\\multirow[]{Float}[Number]{}[Dimension]{}",
    "\\lx@multirow@setup{#2}[#1]{#4}\\hbox{\\multirowsetup #6}");
});
