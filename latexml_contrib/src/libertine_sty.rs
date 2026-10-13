use latexml_package::prelude::*;

LoadDefinitions!({
  // libertine.sty:213 reads its options with `\ProcessOptionsX`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  RequirePackage!("ifxetex");
  RequirePackage!("ifluatex");
  RequirePackage!("xkeyval");
  RequirePackage!("textcomp");
  // libertine.sty:449-459 (the type1 branch): the family switches the
  // package documents as its API — `\libertine`, `\biolinum` and the
  // figure-style variants are `\fontfamily{…}\selectfont` (whatsnote-demo
  // uses `\biolinumLF`). Same shape as the raw file, families as named there.
  RawTeX!(
    r"\providecommand*\libertine{\fontfamily{LinuxLibertineT-TLF}\selectfont}
\providecommand*\libertineSB{\fontfamily{LinuxLibertineT-TLF}\fontseries{sb}\selectfont}
\providecommand*\libertineOsF{\fontfamily{LinuxLibertineT-TOsF}\selectfont}
\providecommand*\libertineLF{\fontfamily{LinuxLibertineT-TLF}\selectfont}
\providecommand*\libertineDisplay{\fontfamily{LinuxLibertineDisplayT-TLF}\selectfont}
\providecommand*\biolinum{\fontfamily{LinuxBiolinumT-TLF}\selectfont}
\providecommand*\biolinumOsF{\fontfamily{LinuxBiolinumT-TOsF}\selectfont}
\providecommand*\biolinumLF{\fontfamily{LinuxBiolinumT-TLF}\selectfont}
\providecommand*\libmono{\fontfamily{LinuxLibertineMonoT-TLF}\selectfont}
\providecommand*\libertineInitial{\fontfamily{LinuxLibertineInitialsT-TLF}\selectfont}"
  );
  // libertine.sty:441-523 (the type1 branch): the figure-style switches papers call after loading it
  // (`\useosf` was the first error of 233 run-340 papers; witnesses 2103.15770, 2103.03800). `\useosf`
  // records the old-style figure style the families are named by; superior figures are superscripts.
  RawTeX!(
    r"\def\libertine@figurealign{T}\def\libertine@figurestyle{LF}
\def\biolinum@figurealign{T}\def\biolinum@figurestyle{LF}
\providecommand*\useosf{\edef\libertine@figurestyle{OsF}\edef\biolinum@figurestyle{OsF}}
\providecommand*\sufigures{}
\providecommand*\textsup{\textsuperscript}
\providecommand*\textsuperior{\textsuperscript}
\providecommand*\oldstylenums{\libertineOsF}
\DeclareTextFontCommand{\liningnums}{\libertineLF}
\DeclareTextFontCommand{\oldstylenumsf}{\biolinumOsF}
\DeclareTextFontCommand{\liningnumsf}{\biolinumLF}
\DeclareTextFontCommand{\tabularnums}{\fontfamily{LinuxLibertineT-T\libertine@figurestyle}\selectfont}
\DeclareTextFontCommand{\proportionalnums}{\fontfamily{LinuxLibertineT-\libertine@figurestyle}\selectfont}
\DeclareTextFontCommand{\tabularnumsf}{\fontfamily{LinuxBiolinumT-T\biolinum@figurestyle}\selectfont}
\DeclareTextFontCommand{\proportionalnumsf}{\fontfamily{LinuxBiolinumT-\biolinum@figurestyle}\selectfont}"
  );
});
