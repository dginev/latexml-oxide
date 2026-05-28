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
  // Wrap body in `inline-logical-block` (Misc.class container that
  // accepts Para.model body).
  //
  // Rust-only surpass-Perl divergence: Perl ar5iv-bindings/mdframed.sty.ltxml
  // L31-34 uses `inline-block` (Block.model only), which the schema rejects
  // when an `mdframed` body contains a `\begin{theorem}` (theorem lives in
  // Para.class, not Block.class). `inline-logical-block` is the strictly
  // safer swap:
  //   * Same `Misc.class` membership as `inline-block` — accepted in every
  //     parent context where Perl's choice fits (inline AND block). The
  //     alternative `logical-block` is in `Para.class` and would BREAK
  //     inline-context uses of mdframed (`\fbox{\begin{mdframed}…}` etc.).
  //   * Same Backgroundable.attributes surface (`framed`, `framecolor`,
  //     `backgroundcolor`).
  //   * Same `display: inline-block` CSS in LaTeXML.css (no visual change).
  //   * `Para.model` body — accepts theorem/proof/para inside.
  //
  // The template emits `framecolor=` only when the #framecolor property is
  // set (via the `?#framecolor(...)` guard), so an unset color correctly
  // omits the attribute rather than emitting `framecolor=''`. Driver:
  // arXiv:2506.03074v1 (ICML 2025 paper with
  // `\begin{mdframed}\begin{theorem}…\end{theorem}\end{mdframed}`).
  DefEnvironment!(
    "{mdframed}[]",
    "<ltx:inline-logical-block framed='rectangle' ?#framecolor(framecolor='#framecolor') _noautoclose='1'>#body</ltx:inline-logical-block>",
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
