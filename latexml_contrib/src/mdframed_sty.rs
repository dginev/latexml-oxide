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
  def_macro_noop("\\newmdenv[]{}")?;
  def_macro_noop("\\renewmdenv[]{}")?;
  def_macro_noop("\\surroundwithmdframed[]{}")?;
  def_macro_noop("\\mdfsubtitle[]{}")?;
  def_macro_noop("\\mdfapptodefinestyle{}{}")?;
  def_macro_noop("\\mdfsetup{}")?;
  def_macro_noop("\\mdfdefinestyle{}{}")?;
  DefRegister!("\\mdflength" => Dimension::new(0));
  // Wrap body in a `logical-block` (block-level Para.model container).
  // Rust-only surpass-Perl divergence: Perl ar5iv-bindings/mdframed.sty.ltxml
  // L31-34 uses `inline-block` (Block.model only), which the schema rejects
  // when an `mdframed` body contains a `\begin{theorem}` (theorem lives in
  // Para.class, not Block.class). `logical-block` is the semantic
  // equivalent that ACCEPTS Para.model — same Backgroundable.attributes
  // (`framed`, `framecolor`, `backgroundcolor`), same `<div>` HTML
  // rendering. The template emits `framecolor=` only when the
  // #framecolor property is set (via the `?#framecolor(...)` guard), so
  // an unset color correctly omits the attribute rather than emitting
  // `framecolor=''`. Driver: arXiv:2506.03074v1 (ICML 2025 paper with
  // `\begin{mdframed}\begin{theorem}…\end{theorem}\end{mdframed}`).
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
