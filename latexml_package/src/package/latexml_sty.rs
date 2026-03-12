use crate::prelude::*;

LoadDefinitions!({
  // 'nobibtex': used for arXiv-like build harnesses where only ".bbl" is available
  // (bibtex will not be ran). 'bibtex' is the default (try bib, fall back to bbl).
  DeclareOption!("bibtex", {
    AssignValue!("BIB_CONFIG", Stored::Strings(Rc::new([arena::pin("bib"), arena::pin("bbl")])),
      Scope::Global);
  });
  DeclareOption!("nobibtex", {
    AssignValue!("BIB_CONFIG", Stored::Strings(Rc::new([arena::pin("bbl")])), Scope::Global);
  });

  // bibconfig KeyVal: comma-separated list of bib config values
  // e.g. \usepackage[bibconfig=bib,bbl]{latexml}
  // TODO: DefKeyVal!("LTXML", "bibconfig", "Semiverbatim", "", code => ...)
  // For now, the bibtex/nobibtex options cover the main use cases.

  // Lexeme serialization for math formulas
  DeclareOption!("mathlexemes", {
    AssignValue!("LEXEMATIZE_MATH" => true, Scope::Global);
  });

  // Header guessing for tabular environments
  DeclareOption!("guesstabularheaders", {
    AssignValue!("GUESS_TABULAR_HEADERS" => true, Scope::Global);
  });
  DeclareOption!("noguesstabularheaders", {
    AssignValue!("GUESS_TABULAR_HEADERS" => false, Scope::Global);
  });

  // Finer control over which (if any) raw .sty/.cls files to include
  DeclareOption!("rawstyles", {
    AssignValue!("INCLUDE_STYLES"  => true, Scope::Global);
  });
  DeclareOption!("localrawstyles", {
    AssignValue!("INCLUDE_STYLES"  => "searchpaths", Scope::Global);
  });
  DeclareOption!("norawstyles", {
    AssignValue!("INCLUDE_STYLES"  => false,             Scope::Global);
  });
  DeclareOption!("rawclasses", {
    AssignValue!("INCLUDE_CLASSES" => true,             Scope::Global);
  });
  DeclareOption!("localrawclasses", {
    AssignValue!("INCLUDE_CLASSES" => "searchpaths", Scope::Global);
  });
  DeclareOption!("norawclasses", {
    AssignValue!("INCLUDE_CLASSES" => false, Scope::Global);
  });

  ProcessOptions!();

  DefConditional!("\\iflatexml", { true });
});
