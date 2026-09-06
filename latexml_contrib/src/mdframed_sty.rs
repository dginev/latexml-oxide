use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "mdframed.sty",
    "mdframed.sty is only minimally stubbed and will not be interpreted raw."
  );
  RequirePackage!("kvoptions");
  RequirePackage!("xparse");
  RequirePackage!("etoolbox");
  RequirePackage!("xcolor");
  // mdframed.sty:591 `\newmdtheoremenv[mdframed-opts]{env}[numbered like]
  // {caption}[within]` = `\newtheorem` inside a frame (presentational);
  // the no-op it replaced left every such theorem environment undefined
  // (beautynote: theorem, lemma, definition, proposition, problem).
  // Guard: `perfect_kernel_batch56::mdframed_theorem_environments_are_theorems`.
  DefMacro!("\\newmdtheoremenv [] {} [] {} []", sub[(_opts, env, like, caption, within)] {
    let mut toks = vec![T_CS!("\\newtheorem"), T_BEGIN!()];
    toks.extend(env.unlist());
    toks.push(T_END!());
    if let Some(like) = like {
      toks.push(T_OTHER!("["));
      toks.extend(like.unlist());
      toks.push(T_OTHER!("]"));
    }
    toks.push(T_BEGIN!());
    toks.extend(caption.unlist());
    toks.push(T_END!());
    if let Some(within) = within {
      toks.push(T_OTHER!("["));
      toks.extend(within.unlist());
      toks.push(T_OTHER!("]"));
    }
    Ok(Tokens::new(toks))
  });
  // `\newmdenv[opts]{name}` defines a new environment `name` that wraps
  // `mdframed` (mdframed.sty L578-585:
  //   \newenvironment{#2}{\mdfsetup{#1}\begin{mdframed}}{\end{mdframed}}).
  // `\mdfsetup` is our no-op, so the body reduces to a mdframed wrapper.
  // Surpass-Perl: ar5iv-bindings/mdframed.sty.ltxml L22 also no-ops this,
  // leaving the user's custom env undefined (Perl then errors with
  // `{name} is not defined`). Faithfully porting the real definer makes
  // the custom env work. Witness arXiv:2002.06879
  // (`\newmdenv[...]{mdfigure}` then `\begin{mdfigure}`).
  DefMacro!(
    "\\newmdenv[]{}",
    "\\newenvironment{#2}{\\mdfsetup{#1}\\begin{mdframed}}{\\end{mdframed}}"
  );
  DefMacro!(
    "\\renewmdenv[]{}",
    "\\renewenvironment{#2}{\\mdfsetup{#1}\\begin{mdframed}}{\\end{mdframed}}"
  );
  def_macro_noop("\\surroundwithmdframed[]{}")?;
  def_macro_noop("\\mdfsubtitle[]{}")?;
  def_macro_noop("\\mdfapptodefinestyle{}{}")?;
  def_macro_noop("\\mdfsetup{}")?;
  def_macro_noop("\\mdfdefinestyle{}{}")?;
  DefRegister!("\\mdflength" => Dimension::new(0));
  // The frame is inserted through `insert_block` (Perl TeX_Box.pool.ltxml:449
  // `insertBlock`, the shape `framed.sty.ltxml:21-26` and our `framed_sty.rs`
  // use): the body is built inside `ltx:_CaptureBlock_` and the container is
  // then chosen from content AND context — `logical-block` in flow (theorems,
  // nested frames: arXiv 2506.03074, 2402.07712, 1712.00062), the inline
  // variant inside a float (arXiv 1907.05772), a block child auto-closing
  // when backmatter follows (biblatex-juradiss `\printbibliography`). A
  // hand-rolled `open_element`/`absorb`/`maybe_close_element` with
  // `_autoclose` (the round-2 rewrite) let a box's stray nested-list state
  // leak into a low-level `\begin{list}` item and closed the OUTER list after
  // its first item (cnltx-example's `{example}` frames its output in
  // `\mdframed`; schulmathematik 1→40 errors, sweep 45; pdflatex clean,
  // RUST-ONLY). The capture-and-rename isolates that state.
  // Guards: `perfect_kernel_batch56::mdframed_inside_low_level_list_keeps_items`,
  // `perfect_kernel_gemini::{mdframed_block_bibliography_juradiss, mdframed_in_float_and_nested}`.
  DefEnvironment!(
    "{mdframed}[]",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        let mut attrs: HashMap<String, String> = HashMap::default();
        attrs.insert("framed".to_string(), "rectangle".to_string());
        if let Some(Stored::String(framecolor)) = props.get("framecolor") {
          attrs.insert("framecolor".to_string(), to_string(*framecolor));
        }
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      let mut props = SymHashMap::default();
      if let Some(font) = lookup_font()
        && let Some(color) = font.get_color() {
          props.insert("framecolor", Stored::from(color.to_attribute()));
        }
      Ok(props)
    },
    // mdframed bodies routinely contain multi-paragraph content
    // (theorems, displayed equations, multiple `$$..$$` blocks). The
    // DefEnvironment default of restricted_horizontal makes
    // BOUND_MODE never end with "vertical", so tex_math.rs:467's
    // `$$` → display-math check stays false: each `$$` is parsed as
    // open + immediate close, leaving body content in text mode and
    // cascading "Script _/^ can only appear in math mode" on subscripts.
    // Witness 2402.07712 (eqnarray + multiple `$$..$$` in mdframed).
    mode => "internal_vertical"
  );
});
