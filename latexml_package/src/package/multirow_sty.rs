use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: multirow.sty.ltxml

  DefPrimitive!("\\multirowsetup", None);
  // \multirow: structural split into DefMacro wrapper + internal DefPrimitive
  // setup (2-layer pattern). Perl's single DefPrimitive digests all args;
  // Rust separates the alignment-cell state mutation (primitive, below)
  // from the \hbox-wrapped content flow (DefMacro at :52). The split lets
  // content flow naturally through alignment cell boxes and enables the
  // text-mode \hbox wrap used by 1004.2626 Table 6. (Not WISDOM #41 —
  // that entry is about math-mode ParameterType adaptations.)
  //
  // \lx@multirow@setup: internal primitive that sets rowspan/vattach on current cell.
  // Separated from content so that content flows naturally through alignment cell boxes.
  DefPrimitive!("\\lx@multirow@setup{Float}[]{}", sub[(nrows, attachment, _width)] {
    if let Some(alignment) = lookup_alignment() {
      if let Some(data) = alignment.alignment_cell() {
        let mut data_lock = data.borrow_mut();
        if let Some(colspec) = data_lock.current_column() {
          let rowspan_f = nrows.value_f64();
          let rowspan = if rowspan_f < 0.0 {
            Warn!("unsupported", "multirow",
              "Negative row sizes for \\multirow are not yet supported.");
            1usize
          } else if rowspan_f != rowspan_f.floor() {
            Warn!("unsupported", "multirow",
              "Fractional row sizes for \\multirow are not yet supported.");
            rowspan_f as usize
          } else {
            rowspan_f as usize
          };
          colspec.rowspan = Some(rowspan);
          // Only set vattach when optional [] is explicitly provided and non-empty
          if let Some(ref att) = attachment {
            let att_str = att.to_string();
            if !att_str.trim().is_empty() {
              colspec.vattach = Some(translate_attachment(att).to_string());
            }
          }
        }
      }
    }
    Ok(())
  });
  // \multirow[vpos]{nrows}[bigstruts]{width}[fixup]{content}
  //
  // Perl multirow.sty.ltxml L19 defines `\multirow` as DefPrimitive
  // whose sub body reads the Alignment column, computes rowspan, and
  // digests the content inline. Rust uses DefMacro instead so that
  // the content can be wrapped in `\hbox{\multirowsetup #6}` at gullet
  // expansion time — the \hbox forces text mode so nested `$…$` cleanly
  // switches into math; without it, inside array-cell (outer math)
  // context the inner `$` toggles math OFF, landing content in text
  // mode with script errors (arxiv 1004.2626 Table 6 was the witness).
  //
  // Intentional DefPrimitive → DefMacro kind divergence (WISDOM #44).
  // The rowspan/colspec computation moved into `\lx@multirow@setup`,
  // a paired DefPrimitive that runs after gullet expansion. Observable
  // XML remains identical to the Perl port for well-formed input, and
  // strictly better (no script-mode bleed) for malformed input.
  // Inside the hbox, `\\` is still bound to the surrounding tabular's
  // `\lx@alignment@newline` which fires `\lx@begin@alignment` — invalid
  // in restricted_horizontal mode. multirow's content allows `\\` as a
  // soft line break (the package stacks rows visually). Rebind `\\` to
  // `\lx@newline` (horizontal-mode break) at the start of the hbox so
  // nested `\\` survives. Witness: arXiv:1504.01713 line 694
  // `\multirow{6}{17pt}{$\alpha_\mathrm{exp}$\\$\alpha_\mathrm{C}$}`.
  // Gate the `\hbox{...content}` body on the actual presence of a
  // brace-arg. The Perl multirow.sty.ltxml (Bruce's local version)
  // `$content = Tokens() unless $content` neutralizes a missing 6th
  // arg before reaching the `\hbox` digest. Our DefMacro substitutes
  // `#6` unconditionally, so a malformed `\multirow{3}{*}` (next
  // non-space is `&`) wraps the `&` token inside `\hbox{...&}` and
  // triggers a cascade of "Stray alignment" + mode-frame errors
  // (501 in 0903.4199, 4969 in 0908.2482). Use `\@ifnextchar\bgroup`
  // to peek for an actual `{`-arg and only emit `\hbox{...}` when
  // content is present; otherwise emit `\lx@multirow@setup` alone
  // and leave `&` (or whatever comes next) for the outer tabular.
  DefMacro!("\\multirow[]{Float}[Number]{}[Dimension]",
    "\\lx@multirow@setup{#2}[#1]{#4}\\@ifnextchar\\bgroup{\\lx@multirow@hbox}{}");
  DefMacro!("\\lx@multirow@hbox{}",
    "\\hbox{\\let\\\\\\lx@newline\\multirowsetup #1}");
});
