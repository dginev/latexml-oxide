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
  // Wrap body in `inline-logical-block` (Misc.class container with Para.model body).
  //
  // The schema offers three framed-box elements, each satisfying only TWO of
  // the three placements an `mdframed` must support (verified against
  // resources/RelaxNG: float_model ⊇ Block.model = Block.class|Misc.class|
  // Meta.class; Para.model = Para.class|Meta.class):
  //   * `inline-block`        (Misc.class, body=Block.model): in-float ✓, nests ✓
  //       (Block.model ⊇ Misc.class), theorem ✗ (Block.model ⊉ Para.class).
  //       This is what Perl ar5iv-bindings/mdframed.sty.ltxml L31-34 uses, so
  //       Perl ITSELF errors `malformed:ltx:theorem` on a theorem-in-mdframed.
  //   * `inline-logical-block`(Misc.class, body=Para.model): in-float ✓
  //       (Misc.class ⊂ Block.model ⊂ float_model), theorem ✓ (Para.model ⊇
  //       Para.class), nests ✗ — a directly-nested inner `inline-logical-block`
  //       (Misc.class) isn't in the outer's Para.model.
  //   * `logical-block`       (Para.class, body=Para.model): theorem ✓, nests ✓
  //       (Para.class ∈ Para.model), in-float ✗ — Para.class ⊄ float_model.
  //
  // No single element does all three, and the missing auto-open bridge
  // (inline-logical-block → para → inline-logical-block, which para_model =
  // Block.model WOULD admit) is intentionally suppressed by the `($tag ne $kid)`
  // self-nesting guard in BOTH Perl `Document::computeIndirectModel` (L207) and
  // our `state::compute_indirect_model` — so adding it would diverge from Perl's
  // document model. We therefore pick the element that fails the RAREST case:
  //
  // History: this was `logical-block` (theorem ✓ + nests ✓) until a fresh sweep
  // surfaced arXiv:1907.05772 — an `mdframed` inside a `\begin{algorithm}`
  // float, where Perl is clean (0 err, its `inline-block` is Misc.class) but
  // `logical-block` (Para.class) tripped `"logical-block" isn't allowed in
  // <float>` ×3 (Rust-only, Perl=0). mdframed-in-float (framed algorithm/figure
  // boxes) is far more common than nested frames, so `inline-logical-block`
  // strictly dominates `logical-block`: it FIXES the float regression and keeps
  // the theorem-in-mdframed surpass (arXiv:2506.03074, 2402.07712 — beyond Perl,
  // which errors there). The residual cost is the rare directly-nested-frame
  // case (1712.00062): inner frame as the FIRST child of an outer frame errors
  // `"inline-logical-block" isn't allowed in <inline-logical-block>` (any leading
  // text auto-opens a `para` that then admits the inner frame, so only the
  // bare-first-child variant is affected). Net: trades a moderate Rust-only
  // regression (float) for a rare one (bare-nested-frame), maximizing error-free
  // conversions.
  //
  // The template emits `framecolor=` only when the #framecolor property is
  // set (via the `?#framecolor(...)` guard), so an unset color correctly
  // omits the attribute rather than emitting `framecolor=''`.
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
