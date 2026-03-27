use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: expl3.sty.ltxml — just 3 lines:
  //   LoadPool('LaTeX');
  //   InputDefinitions('expl3', type => 'lua');
  //   InputDefinitions('expl3', type => 'sty', noltxml => 1);
  LoadPool!("LaTeX");
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")), notex => true);
  // Load raw expl3.sty — processes all 36K lines of expl3-code.tex.
  // No token limit, no l3file stubs needed.
  let _ = input_definitions("expl3", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));

  // Post-load: set expl3 catcodes for fixup commands.
  state::assign_catcode(':', Catcode::LETTER, Some(Scope::Global));
  state::assign_catcode('_', Catcode::LETTER, Some(Scope::Global));
  // Define cmd module messages (normally from latex.ltx, not in our LaTeX pool)
  // and suppress info messages to prevent \NewDocumentCommand from leaking text.
  raw_tex(concat!(
    r"\msg_new:nnn{cmd}{define-command}{Defining~command~#1~with~sig.~'#2'~\msg_line_context:.}",
    r"\msg_new:nnn{cmd}{define-env}{Defining~environment~#1~with~sig.~'#2'~\msg_line_context:.}",
    r"\msg_redirect_module:nnn{cmd}{info}{none}",
    r"\msg_redirect_module:nnn{ltcmd}{info}{none}",
    r"\cs_gset_protected:Npn\__kernel_msg_info:nnxx#1#2#3#4{}",
  ))?;
  // Safety net: restore catcodes if expl3.sty's \ExplSyntaxOff didn't run.
  if state::lookup_catcode(' ') != Some(Catcode::SPACE) {
    state::assign_catcode(' ', Catcode::SPACE, Some(Scope::Global));
    state::assign_catcode('\t', Catcode::SPACE, Some(Scope::Global));
    state::assign_catcode('~', Catcode::ACTIVE, Some(Scope::Global));
    state::assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
    state::assign_catcode('_', Catcode::SUB, Some(Scope::Global));
    raw_tex(r"\endlinechar=13\relax")?;
  }
});
