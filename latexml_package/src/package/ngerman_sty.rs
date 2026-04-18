use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ngerman.sty.ltxml
  // This should be essentially the same, right?
  // (considering we don't do hyphenation, etc)
  RequirePackage!("babel", options => vec!["ngerman".to_string()]);

  // Raw-load ngermanb.ldf so babel's authoritative \captionsngerman +
  // \extrasngerman are installed from TeX Live source.
  InputDefinitions!("ngermanb", noltxml => true, extension => Some(Cow::Borrowed("ldf")));
  // \providecommand fallback — belt-and-suspenders if the raw load
  // didn't complete before \select@language fires.
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
