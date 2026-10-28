//! rmaa.cls (Revista Mexicana de Astronomía y Astrofísica, J. Henney's class, 2000-2007): the papers ship the class (it
//! is not in TeX Live), which is read raw; a paper whose source lacks it falls to OmniBus, with the loader's
//! missing-file Warning. Its title page keeps stores only its own `\maketitle` prints (rmaa.cls:914-940, :943-999),
//! which the kernel's locked `\maketitle` refuses, so their text was lost: the affiliations `\altaffiltext` appends to
//! `\@thanks` (:500-503, printed as footnotes by :932, which the kernel's `\maketitle` empties), the Spanish abstract
//! (:450-453 `\resumen`, printed above the abstract, :975) and the keywords (:509-541 `\addkeyword`, `\keywords`, the
//! "Key Words:" line, :981-993); an `\affil` inside an `\author` (:491-494) stayed in the name; the running head's
//! `\shorttitle` (:550) was lost too. They are the frontmatter's here, defined before the class is read, whose
//! `\newcommand`s of the same names are then ignored as those of any name LaTeXML defines. Perl has no binding and
//! reads rmaa through OmniBus, whose `\altaffilmark`/`\altaffiltext` and `\shorttitle` these are; it lacks
//! `\addkeyword` (an undefined macro) and keeps the Spanish abstract in place of the English one. Witnesses 0903.4819,
//! astro-ph0208329, 1301.7157 (OXIDIZED_DESIGN_DIVERGENCES #904).
use latexml_package::prelude::*;

const RMAA: crate::shipped_class::RawRead = crate::shipped_class::RawRead {
  shipped: true,
  ..crate::shipped_class::RawRead::new("rmaa", "cls")
};

LoadDefinitions!({
  // `\author{R.E. Mennickent\altaffilmark{1} and Z. Ko{\l}aczkowski\altaffilmark{1,2}}`: each mark (a list, `1,2`) is
  // the author's request for the affiliations `\altaffiltext{n}{…}` labels, the footnotes the class prints under the
  // title page (0903.4819, astro-ph0208329); an `\altaffiltext` no author marks is the author block's (1301.7157).
  aas_support_sty::altaffil_markup()?;
  // :491-494 `\affil{…}` (every copy, v1.9 to v3.26): inside an `\author`, the affiliation the title page sets on a
  // line of its own under the names (`\\[1.5ex]…\\[1.5ex]`), which read raw stayed in the name ("S. Kimeswenger
  // Institut für Astrophysik, …", astro-ph0105072): the affiliation of the authors before it.
  DefMacro!("\\affil{}", "\\lx@add@affiliation[annotate=new]{#1}");
  // :450 `\resumen{…}` and :454 `\abstract{…}` store the abstracts in Spanish and in English, which the title page
  // prints in that order, under "Resumen" and "Abstract" (:974-980), whichever the paper gives first (1301.7157 gives
  // `\resumen` first, 0903.4819 `\abstract`): two elements, handed on in the title page's order where the authors are
  // queued and the frontmatter not yet made — at `\maketitle`, else at the first section (ahead of the kernel's
  // frontmatter fallback on `\@startsection@hook`), else at the end of the document (aguplus_cls.rs's hand-off).
  RawTeX!(
    r"\let\lx@rmaa@resumen\@empty\let\lx@rmaa@abstract\@empty
\def\lx@rmaa@hand@abstracts{\ifx\lx@rmaa@resumen\@empty\else
    \expandafter\lx@rmaa@add@abstract\expandafter{\lx@rmaa@resumen}{Resumen}\global\let\lx@rmaa@resumen\@empty\fi
  \ifx\lx@rmaa@abstract\@empty\else
    \expandafter\lx@rmaa@add@abstract\expandafter{\lx@rmaa@abstract}{Abstract}\global\let\lx@rmaa@abstract\@empty\fi}
\begingroup\toks0=\expandafter{\@startsection@hook}\xdef\@startsection@hook{\noexpand\lx@rmaa@hand@abstracts\the\toks0}\endgroup
\AtEndDocument{\lx@rmaa@hand@abstracts}"
  );
  AddToMacro!("\\lx@maketitle@begin", "\\lx@rmaa@hand@abstracts");
  DefMacro!("\\resumen{}", "\\gdef\\lx@rmaa@resumen{#1}");
  DefMacro!("\\abstract{}", "\\gdef\\lx@rmaa@abstract{#1}");
  DefMacro!(
    "\\lx@rmaa@add@abstract{}{}",
    "\\lx@add@frontmatter{ltx:abstract}[name={#2}]{#1}"
  );
  // :509-524 `\addkeyword{…}` adds one keyword to those before it, `\@kwdsep` (" --- ") between them; :543 `\keywords{…}`
  // gives them all. The title page prints them after "Key Words:" (:513 `\@keywordheader`).
  RawTeX!(r"\gdef\lx@rmaa@keywords{}");
  DefMacro!(
    "\\addkeyword{}",
    "\\ifx\\lx@rmaa@keywords\\@empty\\gdef\\lx@rmaa@keywords{#1}\\else\\g@addto@macro\\lx@rmaa@keywords{ --- #1}\\fi\\lx@rmaa@add@keywords"
  );
  DefMacro!(
    "\\keywords{}",
    "\\gdef\\lx@rmaa@keywords{#1}\\lx@rmaa@add@keywords"
  );
  DefMacro!(
    "\\lx@rmaa@add@keywords",
    "\\expandafter\\lx@rmaa@add@keywords@list\\expandafter{\\lx@rmaa@keywords}"
  );
  DefMacro!(
    "\\lx@rmaa@add@keywords@list{}",
    "\\lx@add@keywords[name={Key Words:}]{#1}"
  );
  // :550 `\shorttitle{…}` (`\@shorttitle`, the running head, :421): the toc title, as OmniBus's (Perl reads rmaa so and
  // keeps it, 0903.4819; 5 of the 11 rmaa papers give one).
  DefMacro!("\\shorttitle{}", "\\lx@add@toctitle{#1}");
  crate::shipped_class::read_raw(&RMAA)?;
});
