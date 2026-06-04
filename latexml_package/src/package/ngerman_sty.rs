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
});
