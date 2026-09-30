//! unicode-math.sty — Unicode/OpenType math for Xe/LuaLaTeX (no Perl
//! binding). Part of the opt-in `luatex` profile family (user decision
//! 2026-08-31): math-FONT selection is presentation — LaTeXML's math
//! pipeline is Unicode-native already, so the configuration surface absorbs
//! silently and the standard math machinery carries the content.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // unicode-math-luatex.sty:3405/3426 `\__um_define_prime_chars:` re-binds the
  // ACTIVE math `'` to its prime scanner at `\AtBeginDocument` — after every
  // preamble package: hanging.sty:85/101 `\gdef'{\futurelet\next\h@ngrqtest}`
  // globally takes the active prime over, and its `\h@ngrquote` re-emits a
  // catcode-12 `'` that mathcode "8000 makes active again → an unbounded
  // pushback recursion on `$f'(x)$` (kaytannollista-latexia, 420 s; real
  // lualatex is clean because unicode-math has the last word). The kernel's
  // scanner is `\active@math@prime` (math_common.rs). Batch 56bu.
  at_begin_document(Tokens!(
    T_CS!("\\let"),
    T_ACTIVE!('\''),
    T_CS!("\\active@math@prime")
  ))?;
  RequirePackage!("amsmath");
  RequirePackage!("fontspec");
  // unicode-math-luatex.sty:1585-1592: the `version=<name>` key declares the
  // math version the font is set for (`\DeclareMathVersion`), in either option
  // list (`\setmathfont[…]{font}` or `\setmathfont{font}[…]`); asmeconf.cls
  // :698-750 declares `sansbold` so and `\mathversion{sansbold}`s its title
  // (asmeconf-template: "Unknown math version"). The font stays presentation.
  // Guard: `perfect_kernel_batch56::setmathfont_version_declares_the_math_version`.
  DefPrimitive!("\\setmathfont[]{}[]", sub[(pre, _font, post)] {
    for opts in [pre, post] {
      if let Some(name) = opts.and_then(|o| setmathfont_version(&o.to_string())) {
        assign_value(&s!("MATH_VERSION_{name}"), Stored::Bool(true), Some(Scope::Global));
      }
    }
  });
  def_macro_noop("\\setmathfontface DefToken []{}[]")?;
  // unicode-math-xetex.sty:1203 `\__um_setmathfont:nn {<options>}{<font>}`, the
  // implementation behind `\setmathfont`, called directly by classes (fduthesis,
  // njuthesis under the `xetex` profile; class census 2026-09-24).
  RawTeX!(r"\expandafter\def\csname __um_setmathfont:nn\endcsname#1#2{\setmathfont[#1]{#2}}");
  // The `unicode-math` key family (unicode-math.sty's `\keys_define:nn
  // {unicode-math}` options: math-style, bold-style, partial, …), set directly by
  // classes with `\keys_set:nn {unicode-math}` (fduthesis, njuthesis; class census
  // 2026-09-24). The keys choose glyph styles, which are presentation: accept them.
  RawTeX!(r"\ExplSyntaxOn \keys_define:nn { unicode-math } { unknown .code:n = { } } \ExplSyntaxOff");
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
    // The upright Greek letters' code points, for `\lx@um@resolve@greek` below.
    if let Some(name) = cs.strip_prefix("\\mup")
      && RESOLVED_GREEK.contains(&name)
      && let Some(ch) = u32::from_str_radix(code, 16).ok().and_then(char::from_u32)
    {
      assign_value(&s!("unicode-math:greek:{name}"), Stored::from(ch.to_string()), Some(Scope::Global));
    }
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
  // unicode-math-luatex.sty:3719-3734 `\__um_resolve_greek:` at `\AtBeginDocument`: every Greek
  // name — `\Alpha`, `\omicron`, `\varTheta`, … the kernel lacks — becomes its letter
  // (`\mit<name>`; the style is presentation here, so the upright code point, as the table rows
  // above). Names a package or the document defined are left alone, as the table's. Without it
  // `$\Alpha$` was undefined (kaytannollista-latexia: 12 errors; Perl alike, no binding).
  DefPrimitive!("\\lx@um@resolve@greek", {
    for name in RESOLVED_GREEK {
      let cs = T_CS!(&s!("\\{name}"));
      if lookup_definition(&cs)?.is_none()
        && let Some(ch) = lookup_value(&s!("unicode-math:greek:{name}"))
      {
        def_math(cs, None, ch.to_string(), MathPrimitiveOptions::default())?;
      }
    }
  });
  at_begin_document(Tokens!(T_CS!("\\lx@um@resolve@greek")))?;
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

/// The `version=<name>` value of a fontspec-style option list, if any.
/// The names unicode-math-luatex.sty:3724-3728 resolves at `\begin{document}`.
const RESOLVED_GREEK: [&str; 56] = [
  "Alpha",
  "Beta",
  "Gamma",
  "Delta",
  "Epsilon",
  "Zeta",
  "Eta",
  "Theta",
  "Iota",
  "Kappa",
  "Lambda",
  "alpha",
  "beta",
  "gamma",
  "delta",
  "epsilon",
  "zeta",
  "eta",
  "theta",
  "iota",
  "kappa",
  "lambda",
  "Mu",
  "Nu",
  "Xi",
  "Omicron",
  "Pi",
  "Rho",
  "Sigma",
  "Tau",
  "Upsilon",
  "Phi",
  "Chi",
  "Psi",
  "Omega",
  "mu",
  "nu",
  "xi",
  "omicron",
  "pi",
  "rho",
  "sigma",
  "tau",
  "upsilon",
  "phi",
  "chi",
  "psi",
  "omega",
  "varTheta",
  "varsigma",
  "vartheta",
  "varkappa",
  "varrho",
  "varpi",
  "varepsilon",
  "varphi",
];

fn setmathfont_version(opts: &str) -> Option<String> {
  opts
    .split(',')
    .find_map(|kv| {
      let (key, value) = kv.split_once('=')?;
      (key.trim() == "version").then(|| value.trim().trim_matches(['{', '}']).trim().to_string())
    })
    .filter(|name| !name.is_empty())
}
