use crate::prelude::*;
LoadDefinitions!({
  // Perl pstricks.tex.ltxml (post-#2777 / fdc8bf91):
  //   InputDefinitions('pstricks', type => 'tex', noltxml => 1);
  //   RequirePackage('pstricks_support');
  //
  // Load raw pstricks.tex so its internal registers (\pst@dima, etc.),
  // utility macros (\pstheader, \pst@number, etc.), and key-value handlers
  // are available for downstream raw packages (pst-3d.tex, pst-plot.tex,
  // pstricks-add.tex, etc.).
  InputDefinitions!("pstricks", extension => Some("tex".into()), noltxml => true);
  // Then the SVG overlay for cases where pstricks.tex was used directly.
  RequirePackage!("pstricks_support");
});
