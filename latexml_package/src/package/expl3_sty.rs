use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: expl3.sty.ltxml — just 3 lines:
  //   LoadPool('LaTeX');
  //   InputDefinitions('expl3', type => 'lua');
  //   InputDefinitions('expl3', type => 'sty', noltxml => 1);
  LoadPool!("LaTeX");
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")), notex => true);

  // Pre-define l3file functions that are forward-referenced during expl3 loading.
  // In Perl's expansion engine, \exp_last_unbraced:NNNNo at line 11527 of
  // expl3-code.tex creates these functions. In Rust, the expansion chain fails,
  // producing an extra \endcsname. Pre-defining prevents undefined errors;
  // post-load fixup below provides the correct definitions.
  raw_tex(concat!(
    r"\begingroup",
    r"\catcode`\_=11\relax\catcode`\:=11\relax",
    r"\global\def\__file_name_expand_end:{}",
    r"\global\def\__kernel_file_name_sanitize:n#1{#1}",
    r"\global\def\l_file_search_path_seq{}",
    r"\endgroup",
  ))?;

  // Load raw expl3.sty — processes all 36K lines of expl3-code.tex.
  // Suppress errors during loading: expl3-code.tex has many forward references
  // (functions used before defined) and one expansion chain issue producing
  // an extra \endcsname. Pre-definitions above eliminate the l3file forward-refs;
  // SUPPRESS_UNDEFINED_ERRORS handles remaining forward-refs within the 36K lines.
  state::assign_value("SUPPRESS_UNDEFINED_ERRORS", true, Some(Scope::Global));
  state::assign_value("SUPPRESS_UNEXPECTED_ERRORS", true, Some(Scope::Global));
  // Suppress log output during loading: expl3-code.tex fires \errmessage for
  // forward-ref errors and missing Unicode data files (harmless noise).
  latexml_core::common::error::set_suppress_log_output(true);
  let _ = input_definitions("expl3", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));
  latexml_core::common::error::set_suppress_log_output(false);
  state::assign_value("SUPPRESS_UNEXPECTED_ERRORS", false, Some(Scope::Global));

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
  // Re-define l3file functions with correct definitions (the pre-defined stubs
  // were overwritten by the broken expansion chain, now fix them properly).
  raw_tex(concat!(
    r"\exp_last_unbraced:NNNNo",
    r"\cs_gset:Npn \__file_name_expand_cleanup:w #1 \tl_to_str:n { __file_name = } { }",
    r"\cs_gset:Npn \__file_name_expand_end:",
    r"{ \msg_expandable_error:nn { kernel } { filename-missing-endcsname }",
    r"  \cs_end: \__file_name_expand_end: }",
    r"\cs_gset:Npn \__kernel_file_name_sanitize:n #1",
    r"{ \exp_args:Ne \__file_name_trim_spaces:n",
    r"  { \exp_args:Ne \__file_name_strip_quotes:n",
    r"    { \__file_name_expand:n {#1} } } }",
  ))?;
  // Ensure l3file sequences exist (may have failed to create during loading)
  raw_tex(concat!(
    r"\cs_if_exist:NF \g__file_record_seq { \seq_new:N \g__file_record_seq }",
    r"\cs_if_exist:NF \l_file_search_path_seq { \seq_new:N \l_file_search_path_seq }",
  ))?;
  // Ensure cctab message exists (used by cctab validation, defined late in expl3-code.tex).
  // Use \msg_set which overwrites if already defined (avoiding "already defined" error).
  raw_tex(concat!(
    r"\msg_set:nnnn{cctab}{invalid-cctab}",
    r"  {Invalid~code~category~table~'#1'.}",
    r"  {You~tried~to~use~'#1'~as~a~catcode~table,~but~it~is~not~a~valid~catcode~table.}",
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
  state::assign_value("SUPPRESS_UNDEFINED_ERRORS", false, Some(Scope::Global));
});
