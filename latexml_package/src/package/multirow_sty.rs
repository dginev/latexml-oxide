use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: multirow.sty.ltxml

  DefPrimitive!("\\multirowsetup", None);
  DefPrimitive!("\\multirow[]{Float}[Number]{}[Dimension]{}", sub[(attachment, nrows, _bigstruts, _width, _fixup, content)] {
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
          if let Some(att) = attachment {
            colspec.vattach = Some(translate_attachment(att).to_string());
          }
        }
      }
    }
    let rev: Vec<Token> = content.revert();
    let mut toks: Vec<Token> = vec![T_CS!("\\hbox"), T_BEGIN!(), T_CS!("\\multirowsetup")];
    toks.extend(rev);
    toks.push(T_END!());
    stomach::digest(Tokens::new(toks))?;
    Ok(())
  });
});
