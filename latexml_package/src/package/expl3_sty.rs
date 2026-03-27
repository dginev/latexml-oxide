use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: expl3.sty.ltxml
  LoadPool!("LaTeX");
  // Perl: InputDefinitions('expl3', type => 'lua') — looks for expl3.lua.ltxml binding.
  // We skip the raw .lua file: Lua is not TeX, loading it as raw TeX causes
  // 646 "Script _ can only appear in math mode" errors from Lua's underscored identifiers.
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")), notex => true);
  // Load raw expl3.sty — the 20M token limit allows full loading of all 36K lines
  // of expl3-code.tex. Module boundaries:
  //   l3keys (12886), l3intarray (14331), l3fp (14607), l3regex (24625)
  // Note: NO l3file stubs here. Pre-defining them causes \cs_new:Npn failures
  // in expl3-code.tex (line 11529+) because \cs_new checks for existing defs.
  // The l3file undefined-macro errors at early loading are benign and self-resolve
  // when the l3file module loads at line 10734.
  let saved_limit = gullet::set_token_limit(Some(20_000_000));
  let _ = input_definitions("expl3", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));
  gullet::restore_token_limit(saved_limit);

  // Post-load fixup for expl3 f-expansion.
  //
  // expl3's \exp_end_continue_f:w body is `^^@ (backtick + NUL = charcode 0).
  // Our backtick charcode reader doesn't expand the next token during space-skip
  // (unlike real TeX). Using \number\c_zero_int goes through read_digits →
  // read_x_token which DOES expand, making f-expansion work.
  //
  // After the fixup, re-create the quark functions that failed during kernel loading
  // (because they used the broken \exp_end_continue_f:w).
  // Post-load fixup: set expl3 catcodes via Rust API (global scope, not reverted).
  state::assign_catcode(':', Catcode::LETTER, Some(Scope::Global));
  state::assign_catcode('_', Catcode::LETTER, Some(Scope::Global));
  // Post-load fixups with expl3 catcodes active.
  raw_tex(concat!(
    // Fix \exp_end_continue_f:w: our backtick charcode reader doesn't expand
    // during space-skip. Use \number\c_zero_int instead of `^^@.
    r"\protected\long\gdef\exp_end_continue_f:w{\number\c_zero_int}",
    // Define cmd module messages (normally from latex.ltx, not in our LaTeX pool).
    r"\msg_new:nnn{cmd}{define-command}{Defining~command~#1~with~sig.~'#2'~\msg_line_context:.}",
    r"\msg_new:nnn{cmd}{define-env}{Defining~environment~#1~with~sig.~'#2'~\msg_line_context:.}",
    // Suppress info messages from cmd/ltcmd to prevent leaking into output.
    r"\msg_redirect_module:nnn{cmd}{info}{none}",
    r"\msg_redirect_module:nnn{ltcmd}{info}{none}",
    r"\cs_gset_protected:Npn\__kernel_msg_info:nnxx#1#2#3#4{}",
  ))?;
  // Verify l3keys loaded
  {
    let has_keys = state::lookup_meaning(&T_CS!("\\keys_define:nn")).is_some();
    if !has_keys { eprintln!("WARN: expl3 l3keys module did not load (\\keys_define:nn undefined)"); }
  }
  // Restore catcodes to LaTeX defaults.
  // Critical: expl3 sets \catcode32=9 (SPACE→IGNORE) for its internal processing.
  // If not restored, ALL spaces in the document are ignored, breaking paragraphs.
  state::assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
  state::assign_catcode('_', Catcode::SUB, Some(Scope::Global));
  state::assign_catcode(' ', Catcode::SPACE, Some(Scope::Global));
  state::assign_catcode('\t', Catcode::SPACE, Some(Scope::Global)); // TAB was set to IGNORE too
  state::assign_catcode('~', Catcode::ACTIVE, Some(Scope::Global)); // tilde was set to SPACE
  // Also restore \endlinechar to 13 (carriage return, default)
  raw_tex(r"\endlinechar=13\relax")?;
});
