use crate::prelude::*;
LoadDefinitions!({
  // Perl: nil.ldf.ltxml — babel's "nil" (null) language.
  // Define \bbl@languages as an empty stub if not already defined; nil.ldf
  // 2020 expects it to exist. Then load the raw nil.ldf.
  if !IsDefined!(&T_CS!("\\bbl@languages")) {
    DefMacro!("\\bbl@languages", "");
  }
  InputDefinitions!("nil", extension => Some("ldf".into()), noltxml => true);
});
