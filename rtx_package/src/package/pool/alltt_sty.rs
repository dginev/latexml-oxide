use crate::package::*;

LoadDefinitions!(state, {
  DefEnv!("{alltt}", "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
  font => {family => "typewriter", series => "medium", shape => "upright"},
  before_digest => sub[stomach, inner_state] {
    for c in &['$', '&', '#', '^', '_', '%', '~'] {
     AssignCatcode!(*c, Catcode::OTHER);
    }
    AssignCatcode!(' ', Catcode::ACTIVE);
    LetI!(&T_ACTIVE!(" "), T_CS!("\\space"));
    AssignCatcode!('\r' => Catcode::ACTIVE);    // Variant of \obeylines
    LetI!(&T_ACTIVE!("\r"), T_SPACE!("\n"));    // More appropriate than \par, I think?
    AssignValue!("PRESERVE_NEWLINES", true);
    // \@noligs: This SHOULD inhibit ligature substitution! (eg quotes, dots, etc!!!)
    // \frenchspacing\@vobeyspaces1
  });
});
