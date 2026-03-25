use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: expl3.sty.ltxml
  LoadPool!("LaTeX");
  // Perl: InputDefinitions('expl3', type => 'lua') — looks for expl3.lua.ltxml binding.
  // We skip the raw .lua file: Lua is not TeX, loading it as raw TeX causes
  // 646 "Script _ can only appear in math mode" errors from Lua's underscored identifiers.
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")), notex => true);
  InputDefinitions!("expl3", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
