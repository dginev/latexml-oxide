use crate::prelude::*;

LoadDefinitions!({
  DefEnvironment!("{alltt}", "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
  font => {family => "typewriter", series => "medium", shape => "upright"},
  before_digest => {
    for c in &['$', '&', '#', '^', '_', '%', '~'] {
      AssignCatcode!(*c, Catcode::OTHER);
    }
    AssignCatcode!(' ', Catcode::ACTIVE);
    Let!(&T_ACTIVE!(' '), T_CS!("\\space"));
    AssignCatcode!('\r' => Catcode::ACTIVE);    // Variant of \obeylines
    Let!(&T_ACTIVE!('\r'), Token!("\n",Catcode::SPACE));    // More appropriate than \par, I think?
    AssignValue!("PRESERVE_NEWLINES", 1);
    // \@noligs: This SHOULD inhibit ligature substitution! (eg quotes, dots, etc!!!)
    // \frenchspacing\@vobeyspaces1
  });
});
