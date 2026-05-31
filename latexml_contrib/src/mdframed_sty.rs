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
  def_macro_noop("\\newmdtheoremenv[]{}{}[]")?;
  // `\newmdenv[opts]{name}` defines a new environment `name` that wraps
  // `mdframed` (mdframed.sty L578-585:
  //   \newenvironment{#2}{\mdfsetup{#1}\begin{mdframed}}{\end{mdframed}}).
  // `\mdfsetup` is our no-op, so the body reduces to a mdframed wrapper.
  // Surpass-Perl: ar5iv-bindings/mdframed.sty.ltxml L22 also no-ops this,
  // leaving the user's custom env undefined (Perl then errors with
  // `{name} is not defined`). Faithfully porting the real definer makes
  // the custom env work. Witness arXiv:2002.06879
  // (`\newmdenv[...]{mdfigure}` then `\begin{mdfigure}`).
  DefMacro!("\\newmdenv[]{}",
    "\\newenvironment{#2}{\\mdfsetup{#1}\\begin{mdframed}}{\\end{mdframed}}");
  DefMacro!("\\renewmdenv[]{}",
    "\\renewenvironment{#2}{\\mdfsetup{#1}\\begin{mdframed}}{\\end{mdframed}}");
  def_macro_noop("\\surroundwithmdframed[]{}")?;
  def_macro_noop("\\mdfsubtitle[]{}")?;
  def_macro_noop("\\mdfapptodefinestyle{}{}")?;
  def_macro_noop("\\mdfsetup{}")?;
  def_macro_noop("\\mdfdefinestyle{}{}")?;
  DefRegister!("\\mdflength" => Dimension::new(0));
  // Wrap body in `logical-block` (Para.class container with Para.model body).
  //
  // Rust-only surpass-Perl divergence (history): Perl ar5iv-bindings/
  // mdframed.sty.ltxml L31-34 uses `inline-block` (Block.model only), which the
  // schema rejects when an `mdframed` body contains a `\begin{theorem}`
  // (theorem is Para.class, not Block.class). We first swapped to
  // `inline-logical-block` (Misc.class, Para.model) to admit theorems — but
  // `inline-logical-block` is in Misc.class, NOT Para.class, so its `Para.model`
  // body does NOT readmit a nested `inline-logical-block` → two NESTED
  // `\begin{mdframed}` (outer frame around an inner frame) tripped
  // `"inline-logical-block" isn't allowed in <inline-logical-block>` where
  // Perl's `inline-block` (Block.model ⊇ Misc.class) nests fine. Witness
  // 1712.00062 (algorithm box: outer mdframed wrapping an inner titled
  // mdframed).
  //
  // `logical-block` resolves BOTH: it is the block-level sibling of
  // inline-logical-block (schema: "like block can appear in inline or block
  // mode, but typesets its contents as para"), with the SAME
  // Backgroundable.attributes (`framed`/`framecolor`) and the SAME `Para.model`
  // body (admits theorem/proof/para) — AND, being itself in `Para.class`, it
  // nests inside another `logical-block`'s `Para.model`. mdframed already
  // digests in `internal_vertical` (block) mode, so the block-level positioning
  // is consistent with its semantics. Keeps the theorem-in-mdframed surpass
  // (arXiv:2506.03074, 2402.07712) while fixing the nested-frame regression.
  //
  // The template emits `framecolor=` only when the #framecolor property is
  // set (via the `?#framecolor(...)` guard), so an unset color correctly
  // omits the attribute rather than emitting `framecolor=''`.
  DefEnvironment!(
    "{mdframed}[]",
    "<ltx:logical-block framed='rectangle' ?#framecolor(framecolor='#framecolor') _noautoclose='1'>#body</ltx:logical-block>",
    properties => sub[_args] {
      let mut props = arena::SymHashMap::default();
      if let Some(font) = latexml_core::state::lookup_font() {
        if let Some(color) = font.get_color() {
          props.insert("framecolor", Stored::from(color.to_attribute()));
        }
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
    mode => "internal_vertical");
});
