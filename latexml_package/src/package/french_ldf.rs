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
  // babel-french french.ldf L694: `\AtEndOfPackage{\RequirePackage{scalefnt}}`
  // — French superscript scaling (`\FBsupS`/`\textsuperscript` at L702 uses
  // `\scalefont`). Documents loading babel-French therefore expect `\scalefont`
  // to be defined even when they never `\usepackage{scalefnt}` themselves.
  // Our raw-load is skipped (see below), so pull scalefnt explicitly to match.
  // Witness 2010.03230 (`\usepackage{babel}`[french] + bare `\scalefont{0.78}`).
  RequirePackage!("scalefnt");

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

  // french.ldf L1169-1175: `\ifFB@mainlanguage@FR` — true iff French (or
  // its Acadian dialect) is babel's *main* language. The real french.ldf
  // declares the `\newif` then resolves it at `\AtEndOfPackage` time, once
  // babel has set `\bbl@main@language`. Classes built on babel-french probe
  // the boolean DIRECTLY in their preamble — e.g. ems-journal.sty L605:
  //   `\ifFB@mainlanguage@FR \frenchsetup{...} \fi`
  // Our curated french.ldf skips the raw-load, so the bare `\if` was
  // undefined → a spurious `Error:undefined:\ifFB@mainlanguage@FR` plus a
  // mis-nested `\fi` (Perl, which raw-loads french.ldf via
  // `InputDefinitions('french', noltxml=>1)`, defines it and is silent).
  // Port the conditional verbatim. The layout body the real macro gates
  // (`\FBGlobalLayoutFrenchtrue`, beamer list tweaks) is French-typesetting
  // nuance already no-op'd by `\FrenchLayout`/`\FrenchLists` &c. below, so
  // only the boolean itself is needed. Witness 2110.10227 (ems-journal.sty,
  // `[american,british,french]{babel}` — French is NOT main → false).
  RawTeX!(r"\def\FB@french{french}\def\FB@acadian{acadian}\newif\ifFB@mainlanguage@FR
    \AtEndOfPackage{%
      \ifx\bbl@main@language\FB@french \FB@mainlanguage@FRtrue
      \else \ifx\bbl@main@language\FB@acadian \FB@mainlanguage@FRtrue \fi
      \fi}");

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

  // \frquote — french.ldf L601-610 (\ifLaTeXe branch): the modern
  // babel-french quotation command, `\frquote[*]{text}` → guillemets
  // around the text. The real macro routes both the starred and unstarred
  // forms through `\fr@quote` (a multi-level `\FBguill@level` guillemet
  // engine, L611+); the multi-level nesting is a visual nuance with no
  // semantic effect, so we render single-level guillemets via the `\og`/
  // `\fg` pair defined just above (Perl, which raw-loads french.ldf, emits
  // `«⁠text⁠»` with French spacing — same content). Our curated french.ldf
  // skips the raw-load (babel-3.x `\SetString` failure), so `\frquote` was
  // undefined where Perl is clean. Witness 1808.04243 (`[french]{babel}` +
  // `\frquote`). `\@ifstar` handles the `\frquote*` multi-paragraph form.
  RawTeX!(r"\DeclareRobustCommand\frquote{\@ifstar\lx@fr@quote\lx@fr@quote}");
  RawTeX!(r"\newcommand\lx@fr@quote[1]{\og #1\fg}");

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
  def_macro_noop("\\NoAutoSpaceBeforeFDP")?;
  def_macro_noop("\\AutoSpaceBeforeFDP")?;
  // `\FBautospacing` toggle (legacy) — same family.
  def_macro_noop("\\FBautospacing")?;
  // `\NoAutoSpacing` — french.ldf L506 (`\DeclareRobustCommand*`): the
  // user-facing French auto-spacing kill-switch (`\FB@spacing@off` +
  // `\ifFB@active@punct\shorthandoff{;:!?}\fi`). Same family as the FDP
  // toggles above: the thin-space-before-`;:!?` is font-language-driven via
  // our `\lx@french@punct@*` primitives (not per-paper toggleable), and
  // babel's `\shorthandoff` is itself a no-op in Rust (babel_sty.rs L182),
  // so the faithful semantic-output behavior is a no-op. Our curated
  // french.ldf skips the raw-load, so `\NoAutoSpacing` was undefined where
  // Perl (raw-loads french.ldf) is clean. Witness 1810.02869
  // (`[frenchb]{babel}` + `\NoAutoSpacing`).
  def_macro_noop("\\NoAutoSpacing")?;
  // `\AddThinSpaceBeforeFootnotes` — french.ldf L1976
  // (`\newcommand*{\AddThinSpaceBeforeFootnotes}{\FBAutoSpaceFootnotestrue}`):
  // toggles a thin space before footnote markers in French. Pure typeset
  // spacing — moot in our HTML paradigm — so a no-op matches its net output,
  // same family as the FDP/auto-spacing toggles above. Our curated french.ldf
  // skips the raw-load, so it was undefined where Perl (raw-loads french.ldf)
  // is clean. Witness 1610.09195. (No `\No…` companion exists in french.ldf.)
  def_macro_noop("\\AddThinSpaceBeforeFootnotes")?;

  // `\DecimalMathComma` / `\StandardMathComma` — french.ldf L815-877.
  // French writes decimals with a comma (`3,14`); the real macros toggle
  // the math comma's `\mathcode` between *punctuation* (class 6, small
  // trailing space → list "1, 5") and *ordinary* (class 0, no space →
  // decimal "1,5") via `\dec@math@comma`/`\std@math@comma` (L838-841).
  // This is a purely *visual* spacing nuance: LaTeXML's number tokenizer
  // already recognizes a digit-comma-digit run as a single decimal NUMBER
  // independent of the mathcode, so the toggle has NO effect on LaTeXML's
  // semantic output. Verified against Perl (which raw-loads french.ldf via
  // `InputDefinitions('french', noltxml=>1)` and runs the real macros):
  // converting the same `[francais]{babel}` + `\DecimalMathComma` document
  // WITH vs WITHOUT the call produces *byte-identical* XML — i.e. the macro
  // is an effective no-op in LaTeXML. (Attempting to honor the `\mathcode`
  // change literally is also wrong: it reroutes `,` through a family-1 font
  // slot that maps to `;`, corrupting the glyph.) Our curated french.ldf
  // skips the raw-load (babel-3.x `\SetString` failure), so these were
  // undefined where Perl is clean. Faithful port = no-op. Witness 1812.03061
  // (`[francais]{babel}` + `\DecimalMathComma` in the preamble, revtex4):
  // RUST 1 -> 0, matching Perl 0.
  def_macro_noop("\\DecimalMathComma")?;
  def_macro_noop("\\StandardMathComma")?;

  // \frenchsetup — babel-french 3.x configuration command. Takes a
  // keyval list `\frenchsetup{key=val,...}` (e.g. `OldFigTabCaptions=true`,
  // `ItemLabelsspaceitem=true`). Per babel-french/french.ldf L712-713,
  // `\frenchbsetup` is `\let`'d to `\frenchsetup` (legacy alias).
  // Configurations affect formatting subtleties that don't translate
  // meaningfully to HTML — read-and-discard the keyval arg.
  def_macro_noop("\\frenchsetup RequiredKeyVals")?;
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

  // babel.def `\initiate@active@char{?}` (TL `babel/babel.def` L1372)
  // evaluates `\bbl@add@special\csname?\endcsname`; expanding
  // `\csname?\endcsname` turns the (previously undefined) escaped `\?`
  // into `\relax` per TeX's csname rule — a permanent, global,
  // language-INDEPENDENT side-effect of *loading* french (the catcode
  // flip to active is separate, in `\extrasfrench`). `\:`/`\;`/`\!` are
  // already math-spacing commands, so only `\?` is affected. A bare `\?`
  // (e.g. a stray set-builder `D([0,T];\R^k):\? u_C=v_C`) therefore
  // silently vanishes under Perl rather than erroring. We skip the raw
  // french.ldf load, so replicate the exact end-state: an undefined `\?`
  // becomes `\relax` (a pre-existing `\?` is left untouched). Witness
  // 2007.04819 (`\usepackage[frenchb,english]{babel}`, `:\? u_C=v_C`).
  RawTeX!(r"\@ifundefined{?}{\let\?\relax}{}");

  // french.ldf user-facing typesetting knobs that some papers call
  // directly (rather than via `\frenchsetup{key=value}`). All are
  // typographical no-ops in our XML/HTML pipeline since we don't
  // render French punctuation spacing or footnote-style switches.
  // Witness 2503.17701 (`\FrenchFootnotes` in frenchPhi-n.tex).
  def_macro_noop("\\FrenchFootnotes")?;
  def_macro_noop("\\StandardFootnotes")?;
  def_macro_noop("\\FrenchPunctuation")?;
  def_macro_noop("\\StandardPunctuation")?;
  def_macro_noop("\\FrenchLayout")?;
  def_macro_noop("\\StandardLayout")?;
  def_macro_noop("\\AutoSpaceFootnotes")?;
  def_macro_noop("\\NoAutoSpaceFootnotes")?;
  def_macro_noop("\\FrenchSuperscripts")?;
  def_macro_noop("\\NoFrenchSuperscripts")?;
  def_macro_noop("\\GOfrench")?;
  def_macro_noop("\\StandardLists")?;
  def_macro_noop("\\FrenchLists")?;
  def_macro_noop("\\StandardItemLabels")?;
  def_macro_noop("\\StandardItemizeEnv")?;
  def_macro_noop("\\StandardEnumerateEnv")?;
  def_macro_noop("\\StandardListSpacing")?;
  def_macro_noop("\\InTitleNumber")?;
  def_macro_noop("\\AutoSpacePunctuation")?;
  def_macro_noop("\\NoAutoSpacePunctuation")?;
  def_macro_noop("\\ThinSpaceInFrenchNumbers")?;
  // french.ldf L?? — AutoSpace switches for the period (point) before
  // a footnote number marker. Typesetting-only — no-op in XML.
  def_macro_noop("\\AutoSpaceBeforeFDP")?;
  def_macro_noop("\\NoAutoSpaceBeforeFDP")?;

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
                    \csname datefrench\endcsname
    %
    % french.ldf L88-92: `acadian` and `canadien` are dialects of
    % `french` (`\adddialect\l@acadian\l@french` / `\l@canadien`), and its
    % `\StartBabelCommands*{\BabelLanguages}{captions|date}` defines the
    % `acadian`-suffixed hooks (BabelLanguages = {french,acadian}). The
    % thin wrappers `acadian.ldf` / `canadien.ldf` just `\input french.ldf`.
    % This binding doesn't replicate `\StartBabelCommands`, so alias the
    % `\l@` slots + babel hooks to their `french` counterparts (parallels
    % the `frenchb` shim above). Without this `\usepackage[canadien]{babel}`
    % / `[acadian]{babel}` error haven-not-defined-the-language at the final
    % `\selectlanguage{\bbl@main@language}` (e.g. 1712.07952). canadien.ldf
    % already `\def`s the `canadien`-suffixed hooks → `acadian`; we fill in
    % the `acadian` ones + the language slots.
    \expandafter\ifx\csname l@acadian\endcsname\relax
      \chardef\l@acadian=\l@french
    \fi
    \expandafter\ifx\csname l@canadien\endcsname\relax
      \chardef\l@canadien=\l@french
    \fi
    \expandafter\let\csname captionsacadian\expandafter\endcsname
                    \csname captionsfrench\endcsname
    \expandafter\let\csname extrasacadian\expandafter\endcsname
                    \csname extrasfrench\endcsname
    \expandafter\let\csname noextrasacadian\expandafter\endcsname
                    \csname noextrasfrench\endcsname
    \expandafter\let\csname dateacadian\expandafter\endcsname
                    \csname datefrench\endcsname
    \expandafter\let\csname captionscanadien\expandafter\endcsname
                    \csname captionsfrench\endcsname
    \expandafter\let\csname extrascanadien\expandafter\endcsname
                    \csname extrasfrench\endcsname
    \expandafter\let\csname noextrascanadien\expandafter\endcsname
                    \csname noextrasfrench\endcsname
    \expandafter\let\csname datecanadien\expandafter\endcsname
                    \csname datefrench\endcsname");
});
