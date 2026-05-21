use latexml_package::prelude::*;
use latexml_core::binding::content::find_file;

LoadDefinitions!({
  // Perl: ar5iv-bindings/catchfile.sty.ltxml — DefMacro with Perl closure
  // that Input()s a file and DefMacroI()s its contents into the target CS.
  //
  // Rust port: locate the file via find_file and read it as raw bytes,
  // then def_macro the target CS to the slurped contents (lossy UTF-8).
  // This lets `\CatchFileDef\paramtable{tables/main_table.tex}{}`
  // actually populate \paramtable so downstream `\paramtable` use
  // doesn't trip undefined. Witness 2210.08043 (mnras + CatchFileDef).
  DefPrimitive!("\\CatchFileDef DefToken {}{}", sub[(target, path, _opts)] {
    let path_str = path.to_string();
    let resolved = find_file(&path_str, None);
    if let Some(disk) = resolved {
      if let Ok(bytes) = std::fs::read(&disk) {
        let body = String::from_utf8_lossy(&bytes);
        let tokens = mouth::tokenize_internal(&body);
        def_macro(target, None, tokens, None)?;
      }
    }
    Ok(())
  });
  // \CatchFileEdef variant — same shape, but in Perl edef-expands
  // the contents before storing. We slurp + tokenize identically;
  // expansion happens lazily when target CS is used.
  Let!("\\CatchFileEdef", "\\CatchFileDef");
});
