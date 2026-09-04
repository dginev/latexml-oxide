//! unicode-math.sty — Unicode/OpenType math for Xe/LuaLaTeX (no Perl
//! binding). Part of the opt-in `luatex` profile family (user decision
//! 2026-08-31): math-FONT selection is presentation — LaTeXML's math
//! pipeline is Unicode-native already, so the configuration surface absorbs
//! silently and the standard math machinery carries the content.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsmath");
  RequirePackage!("fontspec");
  def_macro_noop("\\setmathfont[]{}[]")?;
  def_macro_noop("\\setmathfontface DefToken []{}[]")?;
  def_macro_noop("\\unimathsetup{}")?;
  def_macro_noop("\\NewNegationCommand{}{}")?;
  def_macro_noop("\\NewNegatedSymbol{}{}")?;
  // unicode-math symbol table loaders and ctex-engine-luatex.def:418-432 hooks.
  // LaTeXML's math engine is Unicode-native; absorb symbol-table loading.
  def_macro_noop("\\__um_input_math_symbol_table:")?;
  def_macro_noop("\\um_input_math_symbol_table:")?;
  def_macro_noop("\\__um_load_symbols:")?;
  def_macro_noop("\\__um_switchto_literal:")?;
  def_macro_noop("\\__um_sym:nnn")?;
  def_macro_noop("\\um_sym:nnn")?;
  def_macro_noop("\\ltjsetmathletter{}")?;
  // Unicode-math's alphabet switches (unicode-math-luatex.sty:2273-2306, the
  // `\clist_map_inline:nn { up, it, bfup, bfit, sfup, sfit, bfsfup, bfsfit,
  // bfsf, tt, bb, bbit, scr, bfscr, cal, bfcal, frak, bffrak, normal, literal,
  // sf, bf }` styles): every style exists as `\sym<x>` AND `\math<x>`
  // (`\math<x>` = `\sym<x>` for the non-text alphabets, :2288-2291;
  // `\mathup` = `\mathrm`, :2306; `\mathrm/it/bf/sf/tt` stay LaTeX's). Mapped
  // to the classical math alphabets so the CONTENT keeps its lettering.
  // Witnesses: numbersets-doc, physics2-legacy (\symbfit); shtthesis-user-guide
  // (\symbfsf); rec-thy (\symbffrak); intexgral/xfakebold/yquant docs (\symup);
  // toptesi topcoman.sty:76 `\mathup{\mu}` (toptesi-example-luatex/-xetex;
  // Perl shares the `\mathup` gap). Bold-italic prefers \boldsymbol when a
  // loaded package provides it (keeps the italic), else plain bold.
  {
    use latexml_core::common::def_parser::parse_parameters;
    let alias = |cs: String, body: String| -> Result<()> {
      let params = parse_parameters("{}", &T_CS!(&cs), true)?;
      def_macro(T_CS!(&cs), params, mouth::tokenize(TeXString::assembled(body)), None)
    };
    for (style, alphabet) in [
      ("up", "\\mathrm"), ("bfup", "\\mathbf"), ("sfup", "\\mathsf"),
      ("sfit", "\\mathsf"), ("bfsfup", "\\mathsf"), ("bfsfit", "\\mathsf"),
      ("bfsf", "\\mathsf"), ("bb", "\\mathbb"), ("bbit", "\\mathbb"),
      ("scr", "\\mathcal"), ("bfscr", "\\mathcal"), ("cal", "\\mathcal"),
      ("bfcal", "\\mathcal"), ("frak", "\\mathfrak"), ("bffrak", "\\mathfrak"),
    ] {
      alias(s!("\\sym{style}"), s!("{alphabet}{{#1}}"))?;
      // `\mathbb`/`\mathcal`/`\mathfrak` ARE the alphabet: leave LaTeX's.
      if s!("\\math{style}") != alphabet {
        alias(s!("\\math{style}"), s!("{alphabet}{{#1}}"))?;
      }
    }
    for style in ["rm", "it", "bf", "sf", "tt"] {
      alias(s!("\\sym{style}"), s!("\\math{style}{{#1}}"))?;
    }
    for style in ["normal", "literal"] {
      alias(s!("\\sym{style}"), s!("#1"))?;
      alias(s!("\\math{style}"), s!("#1"))?;
    }
  }
  DefMacro!(
    "\\symbfit{}",
    "\\ifdefined\\boldsymbol\\boldsymbol{#1}\\else\\mathbf{#1}\\fi"
  );
  DefMacro!("\\mathbfit{}", "\\symbfit{#1}");
});
