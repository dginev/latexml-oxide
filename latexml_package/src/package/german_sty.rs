use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: german.sty.ltxml
  // This should be essentially the same, right?
  // (considering we don't do hyphenation, etc)
  RequirePackage!("babel", options => vec!["german".to_string()]);

  // German caption strings (from germanb.ldf). \providecommand so raw
  // babel/germanb.ldf processing (if any) doesn't overwrite.
  RawTeX!(r"\providecommand\captionsgerman{%
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
  RawTeX!(r"\providecommand\dategerman{}");
  RawTeX!(r"\providecommand\captionsngerman{\captionsgerman}");
  RawTeX!(r"\providecommand\datengerman{\dategerman}");
});
