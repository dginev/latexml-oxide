use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: expl3.sty.ltxml
  LoadPool!("LaTeX");
  // Perl: InputDefinitions('expl3', type => 'lua') — looks for expl3.lua.ltxml binding.
  // We skip the raw .lua file: Lua is not TeX, loading it as raw TeX causes
  // 646 "Script _ can only appear in math mode" errors from Lua's underscored identifiers.
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")), notex => true);
  // Load raw expl3.sty — with expanded skip_one_space, f-expansion works and
  // the kernel loads much further. Temporarily lower the token limit to prevent
  // reaching the fp module (which has cascading errors). 2M tokens is enough
  // for all quark creation and basic infrastructure.
  // Load raw expl3.sty. expl3-code.tex (36K lines) has module boundaries:
  //   l3keys (12886), l3intarray (14331), l3fp (14607), l3regex (24625)
  // l3keys is critical for babel hooks, but loading past l3msg (~10K) introduces
  // undefined errors from incomplete primitives. Current limit loads through l3msg
  // safely. Full loading requires implementing: l3skip dimension parsing,
  // l3keys property handlers, \tex_expanded:D, and other internal primitives.
  // Pre-define stubs for l3file macros that would be defined later in expl3-code.tex
  // but are referenced by code that loads within the token limit window.
  // Without these, partial loading corrupts state via undefined-macro cascades.
  state::assign_catcode(':', Catcode::LETTER, Some(Scope::Global));
  state::assign_catcode('_', Catcode::LETTER, Some(Scope::Global));
  raw_tex(concat!(
    r"\cs_gset:Npn\__file_name_expand_end:#1\s__file_stop{#1}",
    r"\cs_gset:Npn\__kernel_file_name_sanitize:n#1{#1}",
    r"\cs_gset:Npn\l_file_search_path_seq{}",
    r"\cs_gset:Npn\g__file_record_seq{}",
  ))?;
  state::assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
  state::assign_catcode('_', Catcode::SUB, Some(Scope::Global));
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
  // Now apply all fixups with expl3 catcodes active.
  raw_tex(concat!(
    r"\protected\long\gdef\exp_end_continue_f:w{\number\c_zero_int}",
    // Define the cmd/define-command message (from latex.ltx line 4780).
    // xparse.sty checks this to determine which module to use.
    r"\msg_new:nnn{cmd}{define-command}{Defining~command~#1~with~sig.~'#2'~\msg_line_context:.}",
    r"\msg_new:nnn{cmd}{define-env}{Defining~environment~#1~with~sig.~'#2'~\msg_line_context:.}",
    // Suppress info messages from cmd/ltcmd modules.
    // \NewDocumentCommand calls \msg_info:nnxx which leaks text into the document
    // because the l3keys module (needed for log-declarations option) isn't loaded
    // (token limit kills loading before l3keys at line ~16K).
    r"\msg_redirect_module:nnn{cmd}{info}{none}",
    r"\msg_redirect_module:nnn{ltcmd}{info}{none}",
    // Also suppress the \__kernel_msg_info:nnxx handler that might bypass redirects.
    r"\cs_gset_protected:Npn\__kernel_msg_info:nnxx#1#2#3#4{}",
  ))?;
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
