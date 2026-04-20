use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ngerman.sty.ltxml
  // This should be essentially the same, right?
  // (considering we don't do hyphenation, etc)
  RequirePackage!("babel", options => vec!["ngerman".to_string()]);

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
