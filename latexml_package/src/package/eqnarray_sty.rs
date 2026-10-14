//! eqnarray.sty (v1.3, 2010): the `equationarray` environment, an eqnarray whose columns are given by an array.sty
//! column specification.
//!
//! eqnarray.sty:36-100 builds the `\halign` preamble with array.sty's `\@mkpream` internals (`\@classx`,
//! `\prepnext@tok`, `\insert@column`, `\@addtopreamble`), which the array binding replaces, so a raw load was "undefined
//! control sequence `\@classx`" and every cell's scripts "can only appear in math mode" (1605.07858 `{rcl}`,
//! 2507.02169 `{@{}c@{}r@{\,}c@{\,}l>{\,}l>{\,}l@{\;}}`). Perl has no binding (the environment is undefined there). This
//! one maps it onto the native eqnarray alignment over the specification's columns, as IEEEtrantools' `\IEEEeqnarray`
//! is (`ieeetrantools_sty.rs`). The fleqn option (indentation), the `[pos]` argument (the tabskip glue), `\eqnnum`
//! and `\yesnumber` are not modelled.
use crate::{engine::latex_constructs::eqnarray_bindings_with_columns, prelude::*};

/// A column of the specification set as eqnarray.sty sets it (`\@eqnclassz`, :70-82): its `\d@llarbegin` is
/// `$\displaystyle` (:39) and its `\d@llarend` `$` (:40), around the `>{…}`/`<{…}` material and the cell (array.sty
/// `\insert@column`), inside the column's own glue.
fn display_math_column(mut cell: Cell) -> Cell {
  let mut before = cell.before.take().map(Tokens::unlist).unwrap_or_default();
  let at = cell.prefix_at.unwrap_or(before.len()).min(before.len());
  before.splice(at..at, [T_MATH!(), T_CS!("\\displaystyle")]);
  cell.prefix_at = Some(at + 2);
  cell.before = Some(Tokens::new(before));
  // The column's own `after` follows the last `\lx@column@trimright` (`Template::add_column`); `<{…}` material
  // precedes it (`Template::add_after_column`).
  let mut after = cell.after.take().map(Tokens::unlist).unwrap_or_default();
  let end = after
    .iter()
    .rposition(|t| *t == T_CS!("\\lx@column@trimright"))
    .unwrap_or(0);
  after.insert(end, T_MATH!());
  cell.after = Some(Tokens::new(after));
  cell
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("array");
  DeclareOption!("fleqn", {});
  ProcessOptions!();
  // eqnarray.sty:36-50 `\equationarray[<pos>]{<cols>}` steps the equation counter per row unless `\nonumber`;
  // :111-112 `equationarray*` numbers no row.
  DefMacro!("\\equationarray", "\\@ifnextchar[{\\lx@equationarray}{\\lx@equationarray[.]}");
  DefMacro!("\\lx@equationarray[]{}",
    "\\lx@equationarray@bindings{#2}\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,deferretract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment");
  Let!("\\endequationarray", "\\endeqnarray");
  DefMacro!("\\csname equationarray*\\endcsname",
    "\\@ifnextchar[{\\lx@equationarray@star}{\\lx@equationarray@star[.]}");
  DefMacro!("\\lx@equationarray@star[]{}",
    "\\lx@equationarray@bindings{#2}\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,retract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment");
  Let!("\\endequationarray*", "\\endeqnarray*");
  DefPrimitive!("\\lx@equationarray@bindings AlignmentTemplate", sub[(template)] {
    let columns = template.get_columns().iter().cloned().map(display_math_column).collect();
    eqnarray_bindings_with_columns(columns)?;
  });
});
