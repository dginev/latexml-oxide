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
  // l3char/l3str: codepoint generation — Perl-faithful ASCII case mapping
  // (raw-load fallback). Full Unicode-aware bodies live in the dump
  // (`\codepoint_str_generate:n` line ~6336, `\__kernel_codepoint_case:nn`
  // line ~23789), but dump_reader gates `:`-named E entries with CS bodies
  // (dump_reader.rs L191-207). However, raw expl3 load DOES register
  // `\char_generate:nn` (verified expandable, uses `\c__char_*_tl` tables).
  //
  // CRITICAL: NO `\protected` — these CSes are invoked from `\exp_args:Ne` /
  // `\use:e` contexts and MUST be expandable. The previous `\protected` stub
  // caused the lipsum L208 cascade (73-paper cluster).
  //
  // `\__kernel_codepoint_case:nn` returns 3 brace groups (mapped-codepoint,
  // empty, empty) matching Perl's `\__codepoint_case:nn` calling convention
  // which feeds `\__str_change_case_char:nnnnn`'s 5-arg signature.
  //
  // Body uses `\lccode`/`\uccode` for ASCII case mapping. `\exp_stop_f:`
  // separators are CRITICAL — under ExplSyntaxOn (active when this raw_tex
  // runs), spaces are catcode-IGNORED, so without explicit \relax-like
  // separators between `=0` and `#2` the integer scanner would absorb #2's
  // digits into num2 (e.g. `=0 101` lexes as `=0101`, num2=101=lccode value,
  // then TRUE branch is empty → returns 0, leaks `\__int_eval_end:`).
  //
  // For non-letters where lccode/uccode is 0, fall through to passthrough
  // (#2 unchanged) — required so `\__str_change_case_char:nnnnn`'s
  // `\int_compare {#1} = {#4}` detects no-op case changes (otherwise
  // non-letters would map to char 0, generating NUL).
  //
  // Full Unicode case mapping (Greek, Cyrillic, German eszett, Turkish
  // dotted I, etc.) requires the gated `\__codepoint_case:nn{n}`,
  // `\__kernel_codepoint_data:nn`, and `\c__codepoint_*_intarray` tables —
  // future work via dump_reader gate widening.
  raw_tex(concat!(
    r"\gdef \codepoint_str_generate:n #1 {\char_generate:nn{#1}{12}}",
    r"\gdef \__kernel_codepoint_case:nn #1#2 {",
      r"{\int_eval:n{",
        r"\str_if_eq:nnTF{#1}{lowercase}",
          r"{\ifnum\lccode#2=0 \exp_stop_f:#2\else\lccode#2\exp_stop_f:\fi}",
          r"{\str_if_eq:nnTF{#1}{uppercase}",
            r"{\ifnum\uccode#2=0 \exp_stop_f:#2\else\uccode#2\exp_stop_f:\fi}",
            r"{\str_if_eq:nnTF{#1}{titlecase}",
              r"{\ifnum\uccode#2=0 \exp_stop_f:#2\else\uccode#2\exp_stop_f:\fi}",
              r"{\ifnum\lccode#2=0 \exp_stop_f:#2\else\lccode#2\exp_stop_f:\fi}}}",
      r"}}{}{}",
    r"}",
    // l3text \text_lowercase:n / \text_uppercase:n etc.: full Unicode-aware
    // text-mode case mapping is dump-gated (\__text_change_case:nnn at line
    // 18979 plus a deep helper chain). For ASCII / non-grouped text inputs,
    // the result matches \str_lowercase:n / \str_uppercase:n. Shim accordingly
    // so the ~90 packages using \text_*case:n produce *something* useful
    // instead of letting the CS expand to its own name. Real text-mode
    // semantics (handling \protect, ungrouping, etc.) require dump_reader
    // gate widening.
    //
    // The :nn variants take a language as #1 and content as #2 — for
    // ASCII parity we ignore lang. The _all variants apply titlecase to
    // every word (Perl's lib does this via word splitting); for ASCII
    // approximation we just uppercase everything. The _first variant
    // properly upcases only the first letter; full uppercase is the
    // closest expandable approximation without word-parsing logic.
    r"\gdef \text_lowercase:n #1 {\str_lowercase:n {#1}}",
    r"\gdef \text_uppercase:n #1 {\str_uppercase:n {#1}}",
    r"\gdef \text_titlecase:n #1 {\str_uppercase:n {#1}}",
    r"\gdef \text_titlecase_first:n #1 {\str_uppercase:n {#1}}",
    r"\gdef \text_titlecase_all:n #1 {\str_uppercase:n {#1}}",
    r"\gdef \text_lowercase:nn #1#2 {\str_lowercase:n {#2}}",
    r"\gdef \text_uppercase:nn #1#2 {\str_uppercase:n {#2}}",
    r"\gdef \text_titlecase:nn #1#2 {\str_uppercase:n {#2}}",
    r"\gdef \text_titlecase_first:nn #1#2 {\str_uppercase:n {#2}}",
    r"\gdef \text_titlecase_all:nn #1#2 {\str_uppercase:n {#2}}",
    // l3str \str_foldcase:n / :V — case-fold for case-insensitive comparison.
    // For ASCII this is equivalent to lowercase. Real Unicode case-folding
    // (German eszett ß → ss, etc.) requires the gated case tables.
    r"\gdef \str_foldcase:n #1 {\str_lowercase:n {#1}}",
    r"\gdef \str_foldcase:V #1 {\str_lowercase:n {#1}}",
  ))?;

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
