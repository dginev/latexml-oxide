use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsart.sty.ltxml

  // Generally ignorable options
  for option in [
    "12pt", "11pt", "10pt", "oneside", "twoside", "onecolumn", "twocolumn",
    "symbold", "ussrhead", "doublespacing", "reviewcopy",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // elsart.sty:66-78 (v2.23, 0805.0525's copy): the options are also the style's switches, `\if@seceqn`, `\if@secthm`,
  // `\if@secfloat`, `\if@nameyear` (`\ds@seceqn{\@seceqntrue}`; `\ds@secnum` sets the first three), which the style
  // and a journal style built on it test. No `\if@amsthm` (only elsart.cls has one, elsart_cls.rs).
  // Guard `perfect_kernel_batchnext397::elsart_sty_options_are_its_switches`.
  RawTeX!(r"\newif\if@seceqn\@seceqnfalse\newif\if@secthm\@secthmfalse\newif\if@secfloat\@secfloatfalse
    \newif\if@nameyear\@nameyearfalse");
  DeclareOption!("seceqn", {
    AssignValue!("@seceqn" => 1i64);
    Digest!("\\@seceqntrue")?;
  });
  DeclareOption!("secthm", {
    AssignValue!("@secthm" => 1i64);
    Digest!("\\@secthmtrue")?;
  });
  DeclareOption!("secfloat", {
    Digest!("\\@secfloattrue")?;
  });
  DeclareOption!("secnum", {
    AssignValue!("@seceqn" => 1i64);
    AssignValue!("@secthm" => 1i64);
    Digest!("\\@seceqntrue\\@secfloattrue\\@secthmtrue")?;
  });
  DeclareOption!("nameyear", {
    Digest!("\\@nameyeartrue")?;
  });
  DeclareOption!("amsthm", {
    AssignValue!("@amsthm" => 1i64);
  });

  ProcessOptions!();
  RequirePackage!("elsart_support");
});
