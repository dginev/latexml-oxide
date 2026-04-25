use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: expl3.sty.ltxml — just 3 lines:
  //   LoadPool('LaTeX');
  //   InputDefinitions('expl3', type => 'lua');
  //   InputDefinitions('expl3', type => 'sty', noltxml => 1);
  LoadPool!("LaTeX");
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")), notex => true);

  // NOTE: Pre-definitions for l3file functions removed. The \exp_last_unbraced:NNNNo
  // at line 11527 of expl3-code.tex now defines these naturally. Previous pre-defs
  // caused \cs_new:Npn to find them already defined, triggering \msg_error:nnee
  // which has a complex expansion chain that consumed the rest of the file.

  // Short-circuit raw expl3.sty loading when the dump already provides
  // expl3's core definitions (detected via `\tex_let:D`, which the raw
  // expl3.sty itself uses as the gate for `\input expl3-code.tex` on
  // line 54 of l3kernel/expl3.sty). This avoids re-digesting 36k lines
  // of `expl3-code.tex` whose compiled form is already in the dump.
  // Mirrors the TeX-level guard; we just inspect the same condition from
  // Rust to avoid opening the raw mouth entirely.
  let dump_has_expl3 = lookup_definition(&T_CS!("\\tex_let:D"))?.is_some();
  if !dump_has_expl3 {
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
  }

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
  // l3file fixups: the l3file section of expl3-code.tex has a subtle failure
  // where some definitions (quarks, file name functions) don't survive loading.
  // The expl3 core functions (\cs_new:Npn, \quark_new:N, etc.) ARE available
  // at this point, so we use them directly (catcodes are LETTER for _ and :).
  // Perl: all defined naturally by expl3-code.tex L12416-12430.
  // Define unconditionally using \cs_gset — ERROR stubs from suppressed-error
  // loading fool \cs_if_exist into thinking the CS is already defined.
  // \quark_new:N uses \cs_gset_nopar:Npn which overwrites any existing def.
  raw_tex(concat!(
    r"\seq_gclear_new:N \g__file_record_seq",
    r"\seq_gclear_new:N \l_file_search_path_seq",
    r"\scan_new:N \s__file_stop",
    r"\quark_new:N \q__file_nil",
    r"\quark_new:N \q__file_recursion_tail",
    r"\quark_new:N \q__file_recursion_stop",
  ))?;
  // \__kernel_file_name_sanitize:n — passthrough stub (overwrites ERROR stub)
  raw_tex(r"\cs_gset:Npn \__kernel_file_name_sanitize:n #1 {#1}")?;
  // \__file_quark_if_nil:nTF — conditional test for \q__file_nil
  raw_tex(r"\__kernel_quark_new_conditional:Nn \__file_quark_if_nil:n { TF }")?;
  // l3file IOW family fixups — these don't survive raw-load with
  // SUPPRESS_UNDEFINED_ERRORS either. Faithful LaTeXML-mode stubs
  // (all writes-to-terminal suppressed; wraps skip the wrap-and-measure
  // machinery and just invoke the callback on the raw text).
  // Perl: naturally defined by expl3-code.tex L12033/12058/12132+variant,
  //       L12457 (\__file_name_expand_end: end-marker). See cycle 60 of
  //       10k_sandbox match; paper 1611.04489 surfaces these via the
  //       msg / file-input paths.
  // Use TeX-level \protected\gdef instead of \cs_gset_protected:Npn —
  // the latter may itself be in ERROR-stub state if expl3-code.tex's
  // raw load with SUPPRESS_UNDEFINED_ERRORS=true left it broken.
  // \protected\gdef can't fail.
  raw_tex(concat!(
    r"\protected\gdef \__kernel_iow_with:Nnn #1#2#3 {#3}",
    r"\protected\gdef \iow_term:n #1 {}",
    r"\protected\gdef \iow_wrap:nnnN #1#2#3#4 {#3 #4 {#1}}",
    r"\protected\gdef \iow_wrap:nenN #1#2#3#4 {#3 #4 {#1}}",
    r"\gdef \__file_name_expand_end: {}",
  ))?;
  // Additional stubs for the post-fix dominant-undefined cluster
  // (see docs/SANDBOX_TRIAGE.md and project_explsyntax_midload.md).
  // These are L3 helpers that expl3-code.tex's raw load fails to install
  // due to SUPPRESS_UNDEFINED_ERRORS suppression of forward-ref errors.
  // Stubs use TeX-level \protected\gdef for robustness.
  raw_tex(concat!(
    // l3file: \iow_char:N produces the literal char (e.g. \iow_char:N \\ → \\)
    r"\protected\gdef \iow_char:N #1{#1}",
    // l3file: \file_input_stop: terminates input — no-op (file already
    // bounded by mouth)
    r"\gdef \file_input_stop: {}",
    // l3file: \file_input:n {file} — input a file. Stub gobbles arg.
    r"\protected\gdef \file_input:n #1 {}",
    // l3keys: define/set keys — gobble the key arguments
    r"\protected\gdef \keys_define:nn #1#2 {}",
    r"\protected\gdef \keys_set:nn #1#2 {}",
    r"\protected\gdef \keys_set:nV #1#2 {}",
    r"\protected\gdef \keys_set:nv #1#2 {}",
    // l3keys: existence tests — \keys_if_exist:nnTF returns false branch
    r"\protected\gdef \keys_if_exist:nnTF #1#2#3#4 {#4}",
    r"\protected\gdef \keys_if_exist:nnT #1#2#3 {}",
    r"\protected\gdef \keys_if_exist:nnF #1#2#3 {#3}",
    r"\protected\gdef \keys_if_exist:neT #1#2#3 {}",
    r"\protected\gdef \keys_if_exist:neF #1#2#3 {#3}",
    // l3keys: empty initial values for variant CSes
    r"\gdef \l_keys_key_str {}",
    // l3char/l3str: codepoint generation — pass through
    r"\protected\gdef \codepoint_str_generate:n #1 {#1}",
    r"\protected\gdef \__kernel_codepoint_case:nn #1#2 {#2}",
    // l3cmd / l3xparse log-bool variables — define as \c_false_bool
    // (which itself should be defined by expl3-code.tex; if not, it'll
    // be undefined too but that's a separate issue).
    r"\global\let \l__cmd_log_bool \c_false_bool",
    r"\global\let \l__xparse_log_bool \c_false_bool",
  ))?;
  // Safety net: restore catcodes if expl3.sty's \ExplSyntaxOff didn't run properly.
  // Check both space and underscore catcodes — packages using \ProvidesExplPackage
  // may restore space but leave underscore as LETTER if the restoration is group-local.
  if state::lookup_catcode(' ') != Some(Catcode::SPACE)
    || state::lookup_catcode('_') != Some(Catcode::SUB)
  {
    state::assign_catcode(' ', Catcode::SPACE, Some(Scope::Global));
    state::assign_catcode('\t', Catcode::SPACE, Some(Scope::Global));
    state::assign_catcode('~', Catcode::ACTIVE, Some(Scope::Global));
    state::assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
    state::assign_catcode('_', Catcode::SUB, Some(Scope::Global));
    raw_tex(r"\endlinechar=13\relax")?;
  }
  state::assign_value("SUPPRESS_UNDEFINED_ERRORS", false, Some(Scope::Global));
});
