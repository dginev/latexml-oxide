use latexml_package::prelude::*;

LoadDefinitions!({
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
});
