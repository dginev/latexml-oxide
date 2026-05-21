use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: crop.sty.ltxml
  def_primitive_noop("\\crop []")?;
  def_primitive_noop("\\cropdef [] DefToken DefToken DefToken DefToken {}")?;
});
