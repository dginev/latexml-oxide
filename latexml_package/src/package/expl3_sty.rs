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

  // Post-load fixup for expl3 f-expansion.
  //
  // expl3's \exp_end_continue_f:w body is `^^@ (backtick + NUL = charcode 0).
  // Our backtick charcode reader doesn't expand the next token during space-skip
  // (unlike real TeX). Using \number\c_zero_int goes through read_digits →
  // read_x_token which DOES expand, making f-expansion work.
  //
  // After the fixup, re-create the quark functions that failed during kernel loading
  // (because they used the broken \exp_end_continue_f:w).
  raw_tex(concat!(
    r"\catcode58=11\relax\catcode95=11\relax",
    r"\protected\long\gdef\exp_end_continue_f:w{\number\c_zero_int}",
    r"\__kernel_quark_new_test:N\__tl_if_recursion_tail_break:nN",
    r"\__kernel_quark_new_test:N\__str_if_recursion_tail_break:NN",
    r"\__kernel_quark_new_test:N\__str_if_recursion_tail_stop_do:Nn",
    r"\__kernel_quark_new_test:N\__int_if_recursion_tail_stop_do:Nn",
    r"\__kernel_quark_new_test:N\__int_if_recursion_tail_stop:N",
    r"\__kernel_quark_new_test:N\__bool_if_recursion_tail_stop_do:nn",
    r"\__kernel_quark_new_test:N\__prop_if_recursion_tail_stop:n",
    // Define the cmd/define-command message (from latex.ltx line 4780).
    // This is normally in the LaTeX format but our pool doesn't include it.
    // xparse.sty checks \msg_if_exist:nnTF { cmd } { define-command } to
    // determine which module to use for command definitions.
    r"\msg_new:nnn{cmd}{define-command}{Defining~command~#1~with~sig.~'#2'~\msg_line_context:.}",
    r"\msg_new:nnn{cmd}{define-env}{Defining~environment~#1~with~sig.~'#2'~\msg_line_context:.}",
    r"\catcode58=12\relax\catcode95=8\relax",
  ))?;
});
