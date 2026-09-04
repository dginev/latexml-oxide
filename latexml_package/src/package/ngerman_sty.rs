use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ngerman.sty.ltxml
  // This should be essentially the same, right?
  // (considering we don't do hyphenation, etc)
  RequirePackage!("babel", options => vec!["ngerman".to_string()]);

  // Alias the `ngermanb` dialect's language number to `ngerman` (the real
  // ngermanb.ldf does `\let\l@ngermanb\l@ngerman`). Without this,
  // `\usepackage[…,ngermanb]{babel}` → `\selectlanguage{ngermanb}` errors
  // "haven't defined the language 'ngermanb'". Parallels german_sty's
  // `\l@germanb` alias (witness 1010.4065).
  RawTeX!(r"\expandafter\ifx\csname l@ngermanb\endcsname\relax
    \expandafter\ifx\csname l@ngerman\endcsname\relax
      \expandafter\newlanguage\csname l@ngermanb\endcsname
    \else
      \expandafter\let\csname l@ngermanb\expandafter\endcsname\csname l@ngerman\endcsname
    \fi
  \fi");

  // NGerman shares captions with german (reformed orthography, same strings).
  RawTeX!(r"\providecommand\captionsngerman{%
    \def\prefacename{Vorwort}\def\refname{Literatur}%
    \def\abstractname{Zusammenfassung}\def\bibname{Literaturverzeichnis}%
    \def\chaptername{Kapitel}\def\appendixname{Anhang}%
    \def\contentsname{Inhaltsverzeichnis}%
    \def\listfigurename{Abbildungsverzeichnis}%
    \def\listtablename{Tabellenverzeichnis}%
    \def\indexname{Index}\def\figurename{Abbildung}%
    \def\tablename{Tabelle}\def\partname{Teil}%
    \def\pagename{Seite}\def\seename{siehe}%
    \def\alsoname{siehe auch}\def\proofname{Beweis}}");
  RawTeX!(r"\providecommand\datengerman{}");
  // ngermanb.ldf:123-127,196: the babel language hooks. Left undefined, a
  // later `\addto\extrasngerman{…}` / cleveref's `\cref@addto` (cleveref.sty
  // :3832-3852, `\edef#1{\the\toks@…}` on a `\relax` target) made the hook
  // expand into itself at `\begin{document}`: "Token \extrasngerman expands
  // into itself!" (homework-demo-de, jwjournal-demo-de). Guard:
  // `perfect_kernel_batch54::babel_extras_hooks_are_defined`.
  RawTeX!(r#"\providecommand\extrasngerman{\@ifundefined{languageshorthands}{}{\languageshorthands{ngerman}\bbl@activate{"}}}"#);
  RawTeX!(r#"\providecommand\noextrasngerman{\@ifundefined{bbl@deactivate}{}{\bbl@deactivate{"}}}"#);
  // ngermanb.ldf: `\def\dq{"}` — `\dq` yields a literal double-quote (see the
  // matching note in german_sty.rs). [ngerman,english] babel loads this path
  // (not german_sty.rs), so define it here too. Witness 1804.06196.
  RawTeX!(r"\providecommand\dq{\textquotedbl}");
  // ngerman.sty:314-319, 662-671 (see german_sty.rs): `\ngermanTeX` is run by
  // the kernel first aid (latex2e-first-aid-for-external-files.ltx:168-174)
  // after the load, and by documents written for ngerman.sty (bibarts,
  // flacards, gu, labbook, mceinleger). The `"` shorthands come from babel's
  // ngerman here, so `\mdqon`/`\mdqoff` are its shorthand switches.
  RawTeX!(r#"\providecommand\mdqon{\shorthandon{"}}\providecommand\mdqoff{\shorthandoff{"}}"#);
  RawTeX!(r"\providecommand\umlautlow{}\providecommand\umlauthigh{}");
  RawTeX!(r"\providecommand\ngermanTeX{\mdqon\selectlanguage{ngerman}}");
  // german.sty:373-377 `\def@dqmacro{c}{text}{math}` / `\let@dqmacro` — the
  // `"c` shorthand installers a derivative patches (germkorr.sty:141-144);
  // recorded under german's own `\@grmn@@<c>dq` name.
  RawTeX!(
    r"\def\grmn@dq@macro#1#2{\expandafter#1\csname @grmn@@\string #2dq\endcsname}
\def\def@dqmacro#1#2#3{\grmn@dq@macro\def{#1}{{#2}{#3}}}
\def\let@dqmacro#1#2{\begingroup\edef\x{\endgroup\let\grmn@dq@macro\noexpand{#1}\grmn@dq@macro\noexpand{#2}}\x}"
  );
});
