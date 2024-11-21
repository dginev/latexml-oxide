use crate::prelude::*;

LoadDefinitions!({
  // Finer control over which (if any) raw .sty/.cls files to include
  DeclareOption!("rawstyles",      { AssignValue!("INCLUDE_STYLES"  => true, Scope::Global); });
  DeclareOption!("localrawstyles", { AssignValue!("INCLUDE_STYLES"  => "searchpaths", Scope::Global); });
  DeclareOption!("norawstyles",    { AssignValue!("INCLUDE_STYLES"  => false,             Scope::Global); });
  DeclareOption!("rawclasses",     { AssignValue!("INCLUDE_CLASSES" => true,             Scope::Global); });
  DeclareOption!("localrawclasses", { AssignValue!("INCLUDE_CLASSES" => "searchpaths", Scope::Global); });
  DeclareOption!("norawclasses",    { AssignValue!("INCLUDE_CLASSES" => false, Scope::Global); });

  ProcessOptions!();

  DefConditional!("\\iflatexml", { true });

});
