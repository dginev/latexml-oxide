use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: makecell.sty.ltxml
  // Load raw TeX first
  InputDefinitions!("makecell", noltxml => true, extension => Some(Cow::Borrowed("sty")));

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

  // \rothead: simplified override — delegates to \thead without rotation.
  // TODO: implement rotation via {turn}{90} wrapping. Raw TeX causes stack overflow.
  DefMacro!("\\rothead[]{}",
    "\\thead[#1]{#2}");
  DefMacro!("\\rotcell[]{}",
    "\\makecell[#1]{#2}");

  // \diaghead (slopex,slopey) {width}{item A}{item B}
  // Perl: arranges args in a table with appropriate alignment based on slope.
  // Simplified: uses \shortstack for each head item (matching Perl's lx@diag@head)
  DefMacro!("\\lx@diag@head{}{}",
    "{\\theadfont\\shortstack[#1]{#2}}");

  // Full \diaghead: overrides raw TeX with simplified Perl-style version
  // OptionalPair parameter reads (x,y) slope if present
  DefMacro!("\\diaghead OptionalPair {}{}{}",
    "\\lx@diag@head{r}{#3}\\lx@diag@head{l}{#4}");
});
