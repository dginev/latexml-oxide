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

  // Perl french.ldf.ltxml L31-37: AtBeginDocument(sub { ... }) — defer
  // so any later package's redefinition of \textdegree/\textasciitilde/
  // \textasciicircum is captured (e.g. textcomp loaded after french.ldf).
  at_begin_document(TokenizeInternal!(
    r"\let\degre\textdegree\def\degres{\hbox to 0.3em{\degre}}\let\tild\textasciitilde\let\circonflexe\textasciicircum"
  ))?;

  // babel-french/french.ldf L1094-1098 + L1183-1184: French itemize labels
  // are an em-dash, and \labelitemi-iv get \let'd to the Fr-prefixed
  // versions when language is activated. The \let happens inside
  // \extrasfrench, which fires at \begin{document} via babel's main
  // language switch — so AT-BEGIN-DOCUMENT order is what makes any
  // user `\renewcommand{\labelitemi}{...}` get clobbered (matches
  // raw french.ldf semantics; Perl's babel pipeline runs the same
  // sequence). Without this, papers that "renewcommand \labelitemi"
  // to a typo CS like `\bullets` (1312.7418) error in itemize lookup,
  // even though the body is unreachable in real French rendering.
  RawTeX!(r"\providecommand\FrenchLabelItem{\textemdash}");
  RawTeX!(r"\providecommand\Frlabelitemi{\FrenchLabelItem}");
  RawTeX!(r"\providecommand\Frlabelitemii{\FrenchLabelItem}");
  RawTeX!(r"\providecommand\Frlabelitemiii{\FrenchLabelItem}");
  RawTeX!(r"\providecommand\Frlabelitemiv{\FrenchLabelItem}");
  at_begin_document(TokenizeInternal!(
    r"\let\labelitemi\Frlabelitemi\let\labelitemii\Frlabelitemii\let\labelitemiii\Frlabelitemiii\let\labelitemiv\Frlabelitemiv"
  ))?;
  DefMacro!("\\at", "@");
  DefMacro!("\\boi", "\\textbackslash");

  // `\NoAutoSpaceBeforeFDP` / `\AutoSpaceBeforeFDP` — French double-
  // punctuation auto-spacing controls. Defined inside raw french.ldf
  // (TL `babel-french/french.ldf` L500-510) inside `\ifLaTeXe`. We
  // skip raw-load entirely, so stub them as `\relax`. The visual
  // effect (thin space before `;`, `!`, `?`, `:`) is already handled
  // by our `\lx@french@punct@*` primitives above, which can't be
  // toggled per-paper anyway. Witnesses: arXiv:2511.22710 (frenchb
  // paper with `\NoAutoSpaceBeforeFDP{}` call).
  DefMacro!("\\NoAutoSpaceBeforeFDP", "");
  DefMacro!("\\AutoSpaceBeforeFDP", "");
  // `\FBautospacing` toggle (legacy) — same family.
  DefMacro!("\\FBautospacing", "");

  // \frenchsetup — babel-french 3.x configuration command. Takes a
  // keyval list `\frenchsetup{key=val,...}` (e.g. `OldFigTabCaptions=true`,
  // `ItemLabelsspaceitem=true`). Per babel-french/french.ldf L712-713,
  // `\frenchbsetup` is `\let`'d to `\frenchsetup` (legacy alias).
  // Configurations affect formatting subtleties that don't translate
  // meaningfully to HTML — read-and-discard the keyval arg.
  DefMacro!("\\frenchsetup RequiredKeyVals", "");
  Let!("\\frenchbsetup", "\\frenchsetup");

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

  // Babel-level `frenchb` language aliases — TL2025 babel-french 3.7e
  // turned `frenchb.ldf` into a deprecation shim that only `\chardef`s
  // `\l@frenchb=\l@french` and sets `\CurrentOption{french}`. It does
  // NOT chain `\input french.ldf`, so when `\usepackage[frenchb]{babel}`
  // runs, babel's `\selectlanguage{\bbl@main@language}` later errors
  // with "haven't defined the language 'frenchb' yet". (Perl LaTeXML
  // hits the same regression on TL2025 — 2 errors on 0909.3444.) We
  // compensate by aliasing `\l@frenchb` and the `<lang>`-suffixed babel
  // hooks to their `french` counterparts. No-op when the user only
  // requested `french`. Idempotent — safe if the raw shim already ran.
  RawTeX!(r"%
    \expandafter\ifx\csname l@frenchb\endcsname\relax
      \expandafter\ifx\csname l@french\endcsname\relax\newlanguage\l@french\fi
      \chardef\l@frenchb=\l@french
    \fi
    \expandafter\let\csname captionsfrenchb\expandafter\endcsname
                    \csname captionsfrench\endcsname
    \expandafter\let\csname extrasfrenchb\expandafter\endcsname
                    \csname extrasfrench\endcsname
    \expandafter\let\csname noextrasfrenchb\expandafter\endcsname
                    \csname noextrasfrench\endcsname
    \expandafter\let\csname datefrenchb\expandafter\endcsname
                    \csname datefrench\endcsname");
});
