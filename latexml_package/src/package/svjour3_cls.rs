use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: svjour3.cls.ltxml
  // Intended to cover svjour.cls, svjour1.cls, svjour2.cls & svjour3.cls
  LoadClass!("svjour");
  // What editions of svjour3.cls papers ship add (run 347: 42 svjour3 papers), a copy without one leaving it to the
  // paper (`DefinitionOrigin::Fallback`):
  // - the economics edition's `\JEL{codes}` (6 copies, v3.2 and v3.3, 1402.2599's :314-317 "JEL Classification", its
  //   codes separated by `\and`), as `\subclass`: 1402.2599, 2110.08612, 2401.08594, 2404.18137, 2409.15978;
  // - the 2023 edition's back-matter statements (3 copies, 2411.13175's :1133-1159, 1315-1317): `{funding}`/`{Funding}`
  //   "Funding", `{dataavailability}`/`{data}` "Data Availability", `{conflict}`/`{Conflict}` "Conflict of interest",
  //   each a run-in bold heading over its text (2307.15129, 2411.13175, 2001.01655, 2403.11828).
  {
    let _class = OriginGuard::new(DefinitionOrigin::Fallback);
    DefMacro!("\\JELname", "JEL Classification");
    DefMacro!(
      "\\JEL{}",
      "\\lx@add@classification[scheme=JEL,name={\\JELname: }]{\\def\\and{\\unskip\\ $\\cdot$ }#1}"
    );
    RawTeX!(r"\def\fundname{Funding}
\def\dataname{Data Availability}
\def\conflictname{Conflict of interest}
\def\funding{\par\small\rmfamily\trivlist\if!\fundname!\item[]\else\item[\hskip\labelsep{\bfseries\fundname}]\fi}
\def\endfunding{\endtrivlist}
\newenvironment{Funding}{\begin{funding}}{\end{funding}}
\def\dataavailability{\par\small\rmfamily\trivlist\if!\dataname!\item[]\else\item[\hskip\labelsep{\bfseries\dataname}]\fi}
\def\enddataavailability{\endtrivlist}
\newenvironment{data}{\begin{dataavailability}}{\end{dataavailability}}
\def\conflict{\par\small\rmfamily\trivlist\if!\conflictname!\item[]\else\item[\hskip\labelsep{\bfseries\conflictname}]\fi}
\def\endconflict{\endtrivlist}
\newenvironment{Conflict}{\begin{conflict}}{\end{conflict}}");
  }
  // svjour3.cls :474-478 (every copy, 0712.0031's) `\lastand`, the separator before the last author in the title (", and"),
  // which papers write in `\author` (`\author{Silvia Chiappa\thanks{…} \lastand Ulrich Paquet}`, 1907.12906): an `\and`
  // (Perl's binding lacks it: KNOWN_PERL_ERRORS #813).
  Let!("\\lastand", "\\and");
  // The astronomical journal abbreviations 9 copies paste in (0812.0839, 1111.3868, 1406.4364).
  crate::shipped_class::provide_shipped_aas_macros("svjour3")?;
});
