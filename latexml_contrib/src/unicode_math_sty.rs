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
  // unicode-math-luatex.sty:329 `\NewDocumentCommand\setoperatorfont{m}`:
  // operator-font bookkeeping (asmeconf.cls). Witness asmeconf-template.
  def_macro_noop("\\setoperatorfont{}")?;
  // unicode-math-table.tex: 2,448 rows `\UnicodeMathSymbol{"HHHH}{\cs}
  // {\class}{description}` — the symbol NAMES a unicode-math document reaches
  // (`\coloneq` :412 derivative, `\mathhyphen` :123 / `\nvrightarrow` :317
  // rec-thy, `\oiint`/`\intclockwise` :373-375 shtthesis). Each row becomes
  // a DefMath of its code point with the role its class implies, unless the
  // kernel or a loaded package already defines the name (their richer
  // `meaning=` wins); accents and fences are left to those definitions.
  // Guard: `perfect_kernel_batch56::unicode_math_symbol_table_defines_names`.
  DefPrimitive!("\\UnicodeMathSymbol {}{}{}{}", sub[(code, cs, class, _desc)] {
    let code = code.to_string();
    let code = code.trim().trim_start_matches('"');
    let cs = cs.to_string();
    let cs = cs.trim();
    let class = class.to_string();
    let class = class.trim().to_string();
    if let Ok(cp) = u32::from_str_radix(code, 16)
      && let Some(ch) = char::from_u32(cp)
      && cs.starts_with('\\')
      && lookup_definition(&T_CS!(cs))?.is_none()
    {
      let lower = cs.to_ascii_lowercase();
      let role = match class.as_str() {
        "\\mathrel" if lower.contains("arrow") || lower.contains("harpoon") => Some("ARROW"),
        "\\mathrel" => Some("RELOP"),
        "\\mathbin" => Some("ADDOP"),
        "\\mathop" if lower.contains("int") => Some("INTOP"),
        "\\mathop" => Some("SUMOP"),
        "\\mathopen" => Some("OPEN"),
        "\\mathclose" => Some("CLOSE"),
        "\\mathpunct" => Some("PUNCT"),
        "\\mathord" | "\\mathalpha" => Some("ID"),
        _ => None,
      };
      if let Some(role) = role {
        def_math(T_CS!(cs), None, ch.to_string(),
          MathPrimitiveOptions { role: Some(role.to_string()), ..Default::default() })?;
      } else if let Some(role) = match class.as_str() {
        // combining marks: `\vec{x}`-shaped accents (rec-thy `\notaccent`)
        "\\mathaccent" | "\\mathaccentwide" | "\\mathaccentoverlay" => Some("OVERACCENT"),
        "\\mathbotaccent" | "\\mathbotaccentwide" => Some("UNDERACCENT"),
        _ => None,
      } {
        use latexml_core::common::def_parser::parse_parameters;
        let params = parse_parameters("{}", &T_CS!(cs), true)?;
        // an argument-taking DefMath carries the role on the operator
        // (`operator_role`, like the kernel's `\vec{}` in math_common.rs)
        def_math(T_CS!(cs), params, ch.to_string(),
          MathPrimitiveOptions { operator_role: Some(role.to_string()), ..Default::default() })?;
      }
    }
  });
  InputDefinitions!("unicode-math-table", noltxml => true, extension => Some(Cow::Borrowed("tex")));
  // unicode-math-luatex.sty:338 `\removenolimits{\op}`: strips the `\nolimits`
  // an operator was declared with (shtthesis.cls:715) — limits placement is
  // the renderer's.
  def_macro_noop("\\removenolimits{}")?;
  // unicode-math-luatex.sty:3600-3620 provides `\overbracket`/`\underbracket`
  // (`[rule thickness][bracket height]{arg}`; also `\Uoverbracket`/
  // `\Uunderbracket`) with mathtools' interface — derivative.tex:1344 uses
  // `\underbracket` without mathtools. Same rendering as mathtools_sty.rs.
  DefMacro!("\\overbracket[][][]{}",  "\\lx@um@overbracket{#4}");
  DefMacro!("\\underbracket[][][]{}", "\\lx@um@underbracket{#4}");
  Let!("\\Uoverbracket", "\\overbracket");
  Let!("\\Uunderbracket", "\\underbracket");
  DefMath!("\\lx@um@overbracket{}", "\u{FE47}",
    operator_role => "OVERACCENT", scriptpos => "mid",
    alias => "\\overbracket");
  DefMath!("\\lx@um@underbracket{}", "\u{FE48}",
    operator_role => "UNDERACCENT", scriptpos => "mid",
    alias => "\\underbracket");
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
