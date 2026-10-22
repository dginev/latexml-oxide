use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: multirow.sty.ltxml

  DefPrimitive!("\\multirowsetup", None);
  // multirow.sty:203: `\bigstrutjot` (`\jot`) unless bigstrut defined it — the `[<bigstruts>]` unit
  // (:173-190), which tables also set themselves (run 340, witnesses 1906.02927, 2311.04185, hep-ph0507175).
  RawTeX!(r"\@ifundefined{bigstrutjot}{\newdimen\bigstrutjot \bigstrutjot=\jot}{}");
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
    if let Some(alignment) = lookup_alignment()
      && let Some(data) = alignment.alignment_cell() {
        let mut data_lock = data.borrow_mut();
        if let Some(colspec) = data_lock.current_column() {
          let rowspan_f = nrows.value_f64();
          // A negative count spans the rows above (multirow.sty:172), done where the alignment is normalized
          // (latexml_core alignment/normalize.rs `normalize_upward_spans`, which warns when the cells above are not
          // empty); Perl's binding warns and spans one row (multirow.sty.ltxml:28, example arXiv:2210.15835
          // `\multirow{-8.2}{*}{\includegraphics…}`; KNOWN_PERL_ERRORS #1074). Witness 1509.00049.
          // (a fraction of a row is the box's raise, which spans the whole rows: one Warning at most, the span's)
          let rowspan = if rowspan_f < 0.0 {
            colspec.rowspan_up = Some(-rowspan_f as usize);
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
  // `[<bigstruts>]` is a count with an optional `t`/`b` prefix (multirow.sty
  // :158-166 `\multirow@piii`, `[b2]`) and `[<vmove>]` a length added to the
  // `\newlength` `\multirow@dima` (:197); both unused here, so read untyped and as
  // a skip — a `{Number}`/`{Dimension}` would put `b2` or a `plus` part back in
  // the input (OXIDIZED_DESIGN #317).
  // A width other than `*` sets the text as a paragraph box (multirow.sty:174-177 `\\@parboxrestore`): its interline
  // values are the document's, though `\\@array` zeroed them in the cell (KNOWN_PERL_ERRORS #445).
  DefMacro!("\\multirow[]{Float}[]{}[Glue]",
    "\\lx@multirow@setup{#2}[#1]{#4}\\@ifnextchar\\bgroup{\\lx@multirow@hbox{#4}}{}");
  // The text is set between two struts, `\hbox{\strut#6\strut}` / `\strut#6\strut\par` (multirow.sty:174-177): a
  // one-argument author macro as the whole text (`\multirow{5}{*}{\instructblip}`, 2507.07731) takes the closing
  // `\strut`, not the box's `}` (Perl multirow.sty.ltxml `\hbox{\multirowsetup #6}` reads the brace: KNOWN_PERL_ERRORS
  // #888). The closing strut is `\lx@multirow@strut`, which typesets nothing: the kernel's `\strut` is a `\vrule` in
  // math, which the formula's `tex=` spelled out (`\vrule height=10.49995pt,…`, 2501.19081's `array`); the opening one
  // delimits nothing and is left out. Guard `perfect_kernel_batch69a::multirow_text_is_strut_delimited`.
  DefMacro!("\\lx@multirow@strut", None);
  DefMacro!("\\lx@multirow@hbox{}{}",
    "\\hbox{\\let\\\\\\lx@newline\\if*#1\\else\\lx@restore@interline\\fi\\multirowsetup #2\\lx@multirow@strut}");

  // multirow.sty L120: \newlength\multirow@dima (+ dimb / cntb /
  // prefixt). User code that touches these internals before our
  // runtime uses them errors. Define defensively. Witness 2406.16171.
  DefRegister!("\\multirow@dima" => Dimension!("0pt"));
  DefRegister!("\\multirow@dimb" => Dimension!("0pt"));
  DefRegister!("\\multirow@cntb" => Number(0));
  DefConditional!("\\ifmultirow@prefixt");
});
