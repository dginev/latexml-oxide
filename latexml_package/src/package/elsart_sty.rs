use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsart.sty.ltxml

  // Generally ignorable options
  for option in [
    "12pt", "11pt", "10pt", "oneside", "twoside", "onecolumn", "twocolumn",
    "symbold", "ussrhead", "nameyear", "doublespacing", "reviewcopy",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // elsart.sty:66-71 (v2.23, 0805.0525's copy): the options are also the style's switches, `\if@seceqn`/`\if@secthm`
  // (`\ds@seceqn{\@seceqntrue}`), which a journal style built on it may test. The file defines no `\if@amsthm` (only
  // elsart.cls does, elsart_cls.rs). Guard `perfect_kernel_batchnext397::elsart_sty_options_are_its_switches`.
  RawTeX!(r"\newif\if@seceqn\@seceqnfalse\newif\if@secthm\@secthmfalse");
  DeclareOption!("seceqn", {
    AssignValue!("@seceqn" => 1i64);
    Digest!("\\@seceqntrue")?;
  });
  DeclareOption!("secthm", {
    AssignValue!("@secthm" => 1i64);
    Digest!("\\@secthmtrue")?;
  });
  DeclareOption!("amsthm", {
    AssignValue!("@amsthm" => 1i64);
  });

  ProcessOptions!();
  RequirePackage!("elsart_support");
});
