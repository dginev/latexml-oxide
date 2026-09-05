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
  // Dynamic selection between `logical-block` (Para.class) and `inline-logical-block` (Misc.class):
  //
  // The schema offers three framed-box elements, each satisfying only TWO of
  // the three placements an `mdframed` must support (verified against
  // resources/RelaxNG: float_model ⊇ Block.model = Block.class|Misc.class|
  // Meta.class; Para.model = Para.class|Meta.class):
  //   * `inline-block`        (Misc.class, body=Block.model): in-float ✓, nests ✓ (Block.model ⊇
  //     Misc.class), theorem ✗ (Block.model ⊉ Para.class). This is what Perl
  //     ar5iv-bindings/mdframed.sty.ltxml L31-34 uses, so Perl ITSELF errors
  //     `malformed:ltx:theorem` on a theorem-in-mdframed.
  //   * `inline-logical-block`(Misc.class, body=Para.model): in-float ✓ (Misc.class ⊂ Block.model ⊂
  //     float_model), theorem ✓ (Para.model ⊇ Para.class), nests ✗ — a directly-nested inner
  //     `inline-logical-block` (Misc.class) isn't in the outer's Para.model.
  //   * `logical-block`       (Para.class, body=Para.model): theorem ✓, nests ✓ (Para.class ∈
  //     Para.model), in-float ✗ — Para.class ⊄ float_model.
  //
  // By inspecting `document.is_openable("ltx:logical-block")` at construction time:
  //   - inside a float (arXiv:1907.05772), `logical-block` is not openable, so we emit
  //     `inline-logical-block` (in-float ✓);
  //   - in standard flow or outer frames (arXiv:2506.03074, 2402.07712, 1712.00062),
  //     `logical-block` is openable, so we emit `logical-block` (theorems ✓, nested frames ✓).
  // Furthermore, `before_digest` issues `\par` so preceding text paragraph is closed, and
  // the frame element carries `_autoclose='true'` with `document.maybe_close_element` so that
  // block-level backmatter (such as `\printbibliography` / `\thebibliography`, biblatex-juradiss)
  // can auto-close the frame without malformed nesting or missing close errors.
  DefEnvironment!(
    "{mdframed}[]",
    sub[document, _args, props] {
      let tag = if document.is_openable("ltx:logical-block") {
        "ltx:logical-block"
      } else {
        "ltx:inline-logical-block"
      };
      let mut attr = HashMap::default();
      attr.insert("framed".to_string(), "rectangle".to_string());
      attr.insert("_autoclose".to_string(), "true".to_string());
      if let Some(Stored::String(framecolor)) = props.get("framecolor") {
        attr.insert("framecolor".to_string(), to_string(*framecolor));
      }
      document.open_element(tag, Some(attr), None)?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        document.absorb(body, None)?;
      }
      document.maybe_close_element(tag)?;
      Ok(())
    },
    before_digest => {
      digest(Tokens!(T_CS!("\\par")))?;
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
