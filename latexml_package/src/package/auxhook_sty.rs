use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: auxhook.sty.ltxml
  def_primitive_noop("\\AddLineBeginAux{}")?;
  def_primitive_noop("\\AddLineBeginMainAux{}")?;
  def_primitive_noop("\\AddLineBeginPartAux{}")?;
});
