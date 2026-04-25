//! french.ldf / frenchb.ldf — French language support for babel
//! Perl: french.ldf.ltxml + frenchb.ldf.ltxml (~35 lines each)
//!
//! Provides: French superscript commands, ordinals, guillemets,
//! degree symbol, number formatting delegation.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: InputDefinitions('french', type => 'ldf', noltxml => 1)
  // We skip raw loading (it fails on babel 3.x \SetString commands)
  // and provide the essential definitions directly.

  // frenchb's ordinal/abbreviation macros trail with \xspace — load
  // xspace_sty up front so those expansions resolve, matching what
  // the raw french.ldf implicitly assumes.
  RequirePackage!("xspace");
  // Perl french.ldf.ltxml L20: load textcomp for text symbols
  // (\textdegree, \texttrademark, etc.) that French abbreviations
  // may reference via the raw frenchb ordinals.
  RequirePackage!("textcomp");

  // \captionsfrench — the French caption strings, equivalent to what
  // babel's frenchb.ldf defines. Use \providecommand so the raw load
  // (if it reaches this point) doesn't clobber our strings.
  RawTeX!(r"\providecommand\captionsfrench{%
    \def\prefacename{Pr\'eface}\def\refname{R\'ef\'erences}%
    \def\abstractname{R\'esum\'e}\def\bibname{Bibliographie}%
    \def\chaptername{Chapitre}\def\appendixname{Annexe}%
    \def\contentsname{Table des mati\`eres}%
    \def\listfigurename{Table des figures}%
    \def\listtablename{Liste des tableaux}%
    \def\indexname{Index}\def\figurename{Figure}%
    \def\tablename{Table}\def\partname{partie}%
    \def\pagename{page}\def\seename{voir}%
    \def\alsoname{voir aussi}\def\proofname{D\'emonstration}}");
  RawTeX!(r"\providecommand\datefrench{}");

  // French superscript (Perl french.ldf.ltxml L24-26)
  DefConstructor!("\\up{}", "<ltx:sup>#1</ltx:sup>", enter_horizontal => true);
  DefConstructor!("\\fup{}", "<ltx:sup>#1</ltx:sup>", enter_horizontal => true);
  DefConstructor!("\\FB@up@fake{}", "<ltx:sup>#1</ltx:sup>", enter_horizontal => true);

  // \FBthickkern — frenchb.ldf thick kern between ordinal and next token.
  // Rendered as \thinspace in our port.
  DefMacro!("\\FBthickkern", "\\thinspace");

  // Ordinal suffixes (from raw frenchb.ldf) — trail with \xspace so a
  // following punctuation/word gets proper spacing, matching babel's
  // frenchb behavior.
  DefMacro!("\\ier", "\\up{er}\\xspace");
  DefMacro!("\\iers", "\\up{ers}\\xspace");
  DefMacro!("\\iere", "\\up{re}\\xspace");
  DefMacro!("\\ieres", "\\up{res}\\xspace");
  DefMacro!("\\ieme", "\\up{e}\\xspace");
  DefMacro!("\\iemes", "\\up{es}\\xspace");

  // French enumeration (from raw frenchb.ldf) — use \FBthickkern between
  // number and following content.
  DefMacro!("\\FrenchEnumerate{}", "#1\\up{o}\\FBthickkern");
  DefMacro!("\\FrenchPopularEnumerate{}", "#1\\up{o})\\FBthickkern");
  DefMacro!("\\primo", "\\FrenchEnumerate1");
  DefMacro!("\\secundo", "\\FrenchEnumerate2");
  DefMacro!("\\tertio", "\\FrenchEnumerate3");
  DefMacro!("\\quarto", "\\FrenchEnumerate4");
  DefMacro!("\\fprimo)", "\\FrenchPopularEnumerate1");
  DefMacro!("\\fsecundo)", "\\FrenchPopularEnumerate2");
  DefMacro!("\\ftertio)", "\\FrenchPopularEnumerate3");
  DefMacro!("\\fquarto)", "\\FrenchPopularEnumerate4");

  // \No, \no, \Nos, \nos — French abbreviations for "Numéro".
  // frenchb.ldf trails these with \xspace for consistency with a following
  // number / punctuation.
  DefMacro!("\\No", "N\\up{o}\\xspace");
  DefMacro!("\\no", "n\\up{o}\\xspace");
  DefMacro!("\\Nos", "N\\up{os}\\xspace");
  DefMacro!("\\nos", "n\\up{os}\\xspace");

  // \bsc — small caps (from raw frenchb.ldf)
  DefMacro!("\\bsc{}", "{\\scshape #1}");

  // French quotes: \og and \fg (guillemets).
  // frenchb.ldf's \og ends with \nobreakspace; \fg starts with one.
  DefMacro!("\\og", "\\guillemotleft\\nobreakspace");
  DefMacro!("\\fg", "\\nobreakspace\\guillemotright\\xspace");

  // Symbols (Perl french.ldf.ltxml L32-35, AtBeginDocument)
  DefMacro!("\\degre", "\\textdegree");
  DefMacro!("\\degres", "\\hbox to 0.3em{\\degre}");
  Let!("\\tild", "\\textasciitilde");
  Let!("\\circonflexe", "\\textasciicircum");
  DefMacro!("\\at", "@");
  DefMacro!("\\boi", "\\textbackslash");

  // \nombre — delegates to numprint if loaded (Perl french.ldf.ltxml
  // L29-30 is:
  //   Let('\ltx@orig@nombre', '\nombre');
  //   DefMacro('\nombre{}',
  //     '\@ifpackageloaded{numprint}{\numprint{#1}}{\ltx@orig@nombre{#1}}');
  //
  // Rust skips the raw frenchb.ldf load, so there is no original \nombre
  // to fall back to. If numprint isn't loaded we pass the argument
  // through as-is (preserving Perl's numprint branch, falling back to
  // a reasonable identity rather than an undefined-CS).
  DefMacro!("\\nombre{}", "\\@ifpackageloaded{numprint}{\\numprint{#1}}{#1}");

  // French active-punctuation dispatch primitives for :;!? (frenchb.ldf's
  // \extrasfrench inserts a thin space before these chars). The catcode
  // flip + meaning attachment happens in babel's \select@language path
  // (babel_sty.rs \lx@babel@activate@mainlang, babel_support_sty.rs
  // \ltx@bbl@select@language). The primitives check current font language
  // and fall back to bare punctuation in non-French groups (needed because
  // `\foreignlanguage{english}{…!}` re-uses already-tokenized ACTIVE tokens).
  //
  //   ':'  → " :" (regular space, espace insécable visual)
  //   ';!?' → "\u{2006}X" (thin space, SIX-PER-EM SPACE)
  fn in_french() -> bool {
    lookup_font()
      .and_then(|f| f.get_language().map(|l| l.as_ref() == "fr" || l.as_ref() == "fr-CA"))
      .unwrap_or(false)
  }
  DefPrimitive!("\\lx@french@punct@colon", {
    enter_horizontal();
    let s = if in_french() { " :" } else { ":" };
    Tbox::new(arena::pin_static(s), None, None, Tokens!(), stored_map!())
  });
  DefPrimitive!("\\lx@french@punct@semi", {
    enter_horizontal();
    let s = if in_french() { "\u{2006};" } else { ";" };
    Tbox::new(arena::pin_static(s), None, None, Tokens!(), stored_map!())
  });
  DefPrimitive!("\\lx@french@punct@exclam", {
    enter_horizontal();
    let s = if in_french() { "\u{2006}!" } else { "!" };
    Tbox::new(arena::pin_static(s), None, None, Tokens!(), stored_map!())
  });
  DefPrimitive!("\\lx@french@punct@question", {
    enter_horizontal();
    let s = if in_french() { "\u{2006}?" } else { "?" };
    Tbox::new(arena::pin_static(s), None, None, Tokens!(), stored_map!())
  });

  // \frenchbsetup{key=value,...} — frenchb's option-tweak interface.
  // ~6 sandbox papers (1702.08652 … 1704.05389) hit this when a paper
  // configures babel-french. We don't model the per-key options; just
  // accept the keyval list and emit nothing.
  DefMacro!("\\frenchbsetup{}", "");
  DefMacro!("\\frenchsetup{}",  "");
});
