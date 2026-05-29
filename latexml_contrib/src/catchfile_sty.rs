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
    // Perl catchfile.sty.ltxml: `my $contents = Input($_[2]); DefMacroI($_[1],
    // undef, $contents)`. DefMacroI ALWAYS defines the target CS — even when
    // the file is missing (Input of a non-existent file yields empty content).
    // The earlier Rust port only def_macro'd the target inside the
    // `if let Some(disk) … if let Ok(bytes) …` guards, so a missing/unreadable
    // file left the target UNDEFINED, unlike Perl. That breaks the common
    // pattern of reading a previous-run aux file absent from the arXiv source
    // — e.g. makron.sty L61 `\CatchFileEdef\tmp{\jobname.runs}{…}` then
    // `\setcounter{…}{\tmp}`: `\tmp` was undefined where Perl is clean. Always
    // define the target; empty body when the file can't be read. Witness
    // 1611.01359.
    let body = find_file(&path_str, None)
      .and_then(|disk| std::fs::read(&disk).ok())
      .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
      .unwrap_or_default();
    let tokens = mouth::tokenize_internal(&body);
    def_macro(target, None, tokens, None)?;
    Ok(())
  });
  // \CatchFileEdef variant — same shape, but in Perl edef-expands
  // the contents before storing. We slurp + tokenize identically;
  // expansion happens lazily when target CS is used.
  Let!("\\CatchFileEdef", "\\CatchFileDef");
});
