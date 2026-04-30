use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "tabularray.sty",
    "tabularray.sty is not implemented and will not be interpreted raw."
  );
  RequirePackage!("booktabs");
  DefMacro!("\\tblr", "\\tabular");
  DefMacro!("\\endtblr", "\\endtabular");
  DefMacro!("\\booktabs", "\\tabular");
  DefMacro!("\\endbooktabs", "\\endtabular");
  DefMacro!("\\UseTblrLibrary", "\\usepackage");
  DefMacro!("\\SetCell[]{}", "");
  DefMacro!("\\SetCells[]{}", "");
});
