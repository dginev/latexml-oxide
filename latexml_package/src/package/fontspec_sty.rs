use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fontspec.sty.ltxml
  // Preliminary support for xelatex
  RequirePackage!("xunicode");
  // fontspec-luatex.sty:3980 `\DeclareTextFontCommand{\strong}{\strongenv}`;
  // `\strongenv` cycles bold/upright. Under the `luatex` profile
  // nlctuserguide.sty:177 loads fontspec instead of providing its own
  // `\strong` (:174), so the missing command was every error of
  // glossariesbegin and mfirstuc-manual.
  RawTeX!(r"\providecommand\strongenv{\bfseries}
\DeclareTextFontCommand{\strong}{\strongenv}
\providecommand\strongfontdeclare[1]{}");

  // Most of this is probably ignorable... at least initially.
  // And when not ignorable, may need some font re-thinking...

  // General Font selection. fontspec v2 puts the feature list AFTER the
  // font name (`\setmonofont{DejaVu Sans Mono}[Scale=…]`) — the `[]{}[]`
  // signatures absorb both the v1 pre-optional and v2 post-optional forms
  // (hvfloat/libertinus-otf corpus preambles use the v2 form bare).
  def_macro_noop("\\fontspec[]{}[]")?;
  def_macro_noop("\\setmainfont[]{}[]")?;
  def_macro_noop("\\setsansfont[]{}[]")?;
  def_macro_noop("\\setmonofont[]{}[]")?;
  // The face/family definers (fontspec-xetex.sty:575-605, all `{ m O{} m
  // O{} }`) DEFINE their target: `\__fontspec_main_newfontfamily:NnnN`
  // (L755-767) issues `\NewDocumentCommand #1 {} {\fontfamily{…}
  // \fontencoding{…}\selectfont}` — a robust, non-empty font switch. A
  // no-op that merely reads the token left every `\newfontfamily\Foo{…}`
  // …`{\Foo text}` document with `Error:undefined:\Foo` (papiergurvan
  // `\BelleAllureGras`), and unicodefonttable.sty:257's
  // `\tl_if_empty:NF \l__fmuft_compare_font_tl` (its `\setfontface`
  // target) needs the non-empty body to take the compare branch. No
  // OpenType font resolves here, so the switch keeps the current family
  // (`\selectfont`). Perl (fontspec.sty.ltxml:35-36) shares the no-op.
  // Guard: `perfect_kernel_batch54::fontspec_definers_define_a_font_switch`.
  DefMacro!("\\lx@fontspec@definer DefToken []{}[]", sub[(cs, _pre, _font, _post)] {
    def_macro(
      cs,
      None,
      Tokens!(T_CS!("\\selectfont")),
      Some(ExpandableOptions { protected: true, ..Default::default() }),
    )?;
    Ok(Tokens::default())
  });
  for definer in [
    "\\newfontfamily",
    "\\newfontface",
    "\\renewfontfamily",
    "\\setfontfamily",
    "\\providefontfamily",
    "\\renewfontface",
    "\\setfontface",
    "\\providefontface",
  ] {
    Let!(&T_CS!(definer), "\\lx@fontspec@definer");
  }

  def_macro_noop("\\setmathrm[]{}")?;
  def_macro_noop("\\setmathsf[]{}")?;
  def_macro_noop("\\setmathtt[]{}")?;
  def_macro_noop("\\setboldmathrm[]{}")?;

  // fontspec-xetex.sty:607 — real signature `{ t+ o m }`: optional `+`
  // (append rather than replace), OPTIONAL [font-name], then the feature
  // list. The old `[]{}` mis-parsed `\defaultfontfeatures+{…}`.
  def_macro_noop("\\defaultfontfeatures OptionalMatch:+ []{}")?;
  // fontspec-xetex.sty:614/618 — both `{m}`; a leading `[]` could eat a
  // following bracket group that belongs to the document.
  def_macro_noop("\\addfontfeatures{}")?;
  def_macro_noop("\\addfontfeature{}")?;
  // fontspec-xetex.sty:1116 + :1125 (\cs_set_eq \IfFontExistsTF
  // \fontspec_font_if_exist:nTF). Was def_macro_noop, which SWALLOWED both
  // branches. No OpenType font resolves in this engine → false branch.
  // Witnesses: neoschool{,-fr}, beamerthemeCelestia{,-fr}.
  DefMacro!("\\IfFontExistsTF{}{}{}", "#3");
  // fontspec-xetex.sty:658 — no OT feature is ever active here → false.
  DefMacro!("\\IfFontFeatureActiveTF{}{}{}", "#3");

  // v1 alias of \setmainfont (fontspec-xetex.sty:571). Witnesses:
  // awesomebox, biblatex-sbl family, uowthesistitlepage_doc.
  def_macro_noop("\\setromanfont[]{}[]")?;
  // The face/family definers, all `{ m O{} m O{} }` (fontspec-xetex.sty
  // :579-605). Witnesses: tkz-doc (\renewfontfamily), texnegar ×6
  // (\setfontfamily), hvarabic (\providefontfamily), emotion-doc
  // (\renewfontface).
  // (`\renewfontfamily` … `\providefontface`: defined with `\newfontfamily`
  // above.)
  // Feature-declaration surface (fontspec-xetex.sty:622-662) — pure font
  // configuration. Witness for \newopentypefeature: tkz-doc family via
  // fourier-otf.sty:87 and the *-otf math font packages.
  def_macro_noop("\\newfontfeature{}{}")?;
  def_macro_noop("\\newAATfeature{}{}{}{}")?;
  def_macro_noop("\\newopentypefeature{}{}{}")?;
  def_macro_noop("\\newICUfeature{}{}{}")?;
  def_macro_noop("\\aliasfontfeature{}{}")?;
  def_macro_noop("\\aliasfontfeatureoption{}{}{}")?;
  def_macro_noop("\\newfontscript{}{}")?;
  def_macro_noop("\\newfontlanguage{}{}")?;
  def_macro_noop("\\DeclareFontExtensions{}")?;
  // \liningnums{m} (fontspec-xetex.sty:669) typesets its argument with
  // lining figures — the DIGITS are content, only the figure style is
  // presentation. Identity, not no-op. Witness: raleway-otf-specimen.
  def_macro_identity("\\liningnums{}")?;

  // ---- fontspec expl3 conditional layer (fontspec-xetex.sty:946-1137).
  // Every \fontspec_if_… conditional opens with \fontspec_if_fontspec_font:
  // (is the CURRENT font a fontspec-selected OpenType font?) — in our
  // pdfTeX/NFSS model that is never true, so ALL of them are constant-
  // FALSE. Each \prg_new_conditional {TF,T,F} generates three CS names.
  // Driver: \fontspec_if_language:nT via polyglossia.sty:610-621 (40 docs,
  // 17 first-errors; witnesses abnt-doc, toptesi-example-*, greektonoi,
  // churchslavonic-*).
  def_macro_noop("\\fontspec_if_language:nT{}{}")?;
  DefMacro!("\\fontspec_if_language:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_language:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_language:nnT{}{}{}")?;
  DefMacro!("\\fontspec_if_language:nnTF{}{}{}{}", "#4");
  DefMacro!("\\fontspec_if_language:nnF{}{}{}", "#3");
  def_macro_noop("\\fontspec_if_fontspec_font:T{}")?;
  DefMacro!("\\fontspec_if_fontspec_font:TF{}{}", "#2");
  DefMacro!("\\fontspec_if_fontspec_font:F{}", "#1");
  def_macro_noop("\\fontspec_if_opentype:T{}")?;
  DefMacro!("\\fontspec_if_opentype:TF{}{}", "#2");
  DefMacro!("\\fontspec_if_opentype:F{}", "#1");
  def_macro_noop("\\fontspec_if_feature:nT{}{}")?;
  DefMacro!("\\fontspec_if_feature:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_feature:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_feature:nnnT{}{}{}{}")?;
  DefMacro!("\\fontspec_if_feature:nnnTF{}{}{}{}{}", "#5");
  DefMacro!("\\fontspec_if_feature:nnnF{}{}{}{}", "#4");
  def_macro_noop("\\fontspec_if_aat_feature:nnT{}{}{}")?;
  DefMacro!("\\fontspec_if_aat_feature:nnTF{}{}{}{}", "#4");
  DefMacro!("\\fontspec_if_aat_feature:nnF{}{}{}", "#3");
  def_macro_noop("\\fontspec_if_script:nT{}{}")?;
  DefMacro!("\\fontspec_if_script:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_script:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_current_script:nT{}{}")?;
  DefMacro!("\\fontspec_if_current_script:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_current_script:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_current_language:nT{}{}")?;
  DefMacro!("\\fontspec_if_current_language:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_current_language:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_current_feature:nT{}{}")?;
  DefMacro!("\\fontspec_if_current_feature:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_if_current_feature:nF{}{}", "#2");
  def_macro_noop("\\fontspec_font_if_exist:nT{}{}")?;
  DefMacro!("\\fontspec_font_if_exist:nTF{}{}{}", "#3");
  DefMacro!("\\fontspec_font_if_exist:nF{}{}", "#2");
  def_macro_noop("\\fontspec_if_small_caps:T{}")?;
  DefMacro!("\\fontspec_if_small_caps:TF{}{}", "#2");
  DefMacro!("\\fontspec_if_small_caps:F{}", "#1");

  // ---- fontspec expl3 variables / internals.
  // :124 \l_fontspec_family_tl (empty), :125/:437 \g_fontspec_encoding_tl
  // = TU. Read by mathspec/simurgh-fonts/synthslant. Witnesses:
  // dithesis/sample, tikz-qtree-manual.
  DefMacro!("\\l_fontspec_family_tl", "");
  DefMacro!("\\g_fontspec_encoding_tl", "TU");
  // :1197 deprecated v1 alias; :1473 fontname completion; :2239 private
  // key-definition helper — all reached via luatexja-fontspec (witnesses
  // asternote, scsnowman, jpnedu* family).
  def_macro_noop("\\fontspec_select:nn{}{}")?;
  def_macro_noop("\\fontspec_complete_fontname:Nn DefToken {}")?;
  def_macro_noop("\\__fontspec_keys_define_code:nnn{}{}{}")?;
});
