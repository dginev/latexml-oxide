use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: makecell.sty.ltxml
  // Load raw TeX first
  InputDefinitions!("makecell", noltxml => true);

  // Mark thead et.al as headers (row & column)
  DefPrimitive!("\\lx@makecell@head", sub[_args] {
    if let Some(alignment) = lookup_alignment() {
      if let Some(data) = alignment.alignment_cell() {
        if let Some(col) = data.borrow_mut().current_column() {
          col.thead_in_column = true;
          col.thead_in_row = true;
        }
      }
    }
    Ok(())
  });

  // Redefine \theadfont at BeginDocument to include heading marker
  RawTeX!(r"\AtBeginDocument{\let\lx@orig@theadfont\theadfont\def\theadfont{\lx@orig@theadfont\lx@makecell@head}}");

  // Since we use \thead, disable guessing
  AssignValue!("GUESS_TABULAR_HEADERS" => false, Scope::Global);

  // \rothead: override to prevent infinite recursion from raw TeX
  // Simplify: just delegate to \thead with the content
  DefMacro!("\\rothead[]{}",
    "\\thead[#1]{#2}");
  DefMacro!("\\rotcell[]{}",
    "\\makecell[#1]{#2}");
});
