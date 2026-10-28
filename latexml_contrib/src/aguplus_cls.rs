//! aguplus.cls (AGU++, P. W. Daly, 1999/08/19 1.6b): the class is loaded raw — it reads the AGUTeX package its option
//! names (aguplus.cls:47-71 `\getAGU`, `\InputIfFileExists{\AGUpack.sty}`: agupp.sty, agums.sty, jgrga.sty, …) — and
//! the title-page commands the AGUTeX packages share with AASTeX, which they derive from, become the frontmatter as the
//! AAS binding makes them (aas_support_sty.rs, Perl aas_support.sty.ltxml:121-134). Under the raw class (the
//! rawclasses lever) they were the class's typesetting: `\affil` set its text into the `\@title` box (aguplus.cls:236)
//! that `\xmaketitle` typesets at the first `\section` (:192, :225), so the affiliations were body paragraphs, and
//! `\authoraddr` (agupp.sty:154-158) gathered `\authaddr@list`, printed only by the class's own end of the references
//! (agupp.sty:266-306 `\endreferences`, `\endthebibliography`), which the kernel's bibliography replaces, so every
//! author address was lost (1107.5305, 1203.5042, physics9702009, 1010.3998, 0811.3118).
//!
//! The class also carries a copy of natbib 7.0 (aguplus.cls:861-1173, `\ModuleVersion{natbib}[1999/05/28 7.0 (PWD)]`)
//! with square brackets (:864 `\NAT@open{[} \NAT@close{]}`): it loads natbib's binding with `square`, as the copy
//! is natbib's. Without it `\citet`/`\citep` were the copy's, printing a bold "?" for every reference (the kernel's
//! bibliography writes no `\bibcite`; 1010.3998: 75), and `\cite[][]{key}` the kernel's (1107.5305).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("aguplus", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  RequirePackage!("natbib", options => vec![s!("square")]);
  // The class's `\title`, `\author` and `\affil` only build its title-page box (aguplus.cls:227-238 `\setbox\@title=
  // \vbox{… \box\@title \aguauthor{#1}}`), which the frontmatter replaces: the refused `\author` is not replayed
  // for its side effects (`replay_class_setter`, sect05.rs), as the box it would build is the kernel's `\@title` text
  // ("Missing number" for every author, 1010.3998).
  RawTeX!(r"\global\let\lx@dropped@author\relax\global\let\lx@dropped@title\relax");
  // An affiliation is every preceding author's that has none yet (`\author{…}\affil{…}` per author,
  // physics9702009), as the AAS binding's (aas_support_sty.rs).
  DefMacro!("\\affil{}", "\\lx@add@affiliation[annotate=new]{#1}");
  // The author addresses the class gathers (agupp.sty:154-158 `\authoraddress`, `\let\authoraddr`) and prints in a
  // block after the references, each naming its author ("S. G. Claudepierre, Laboratory …"; 1010.3998): each is
  // placed with the author it spells, else the only author, else the block (`\lx@add@contact@placed`). They are
  // gathered as the class gathers them, `\authoraddr` being given before `\author` too (0811.3118), and handed on
  // where the authors are queued and the frontmatter not yet made: at `\maketitle`, else at the first section (ahead
  // of the kernel's frontmatter fallback on `\@startsection@hook`), else at the end of the document.
  RawTeX!(
    r"\let\lx@agu@authoraddrs\@empty
\def\lx@agu@hand@authoraddrs{\lx@agu@authoraddrs\global\let\lx@agu@authoraddrs\@empty}
\begingroup\toks0=\expandafter{\@startsection@hook}\xdef\@startsection@hook{\noexpand\lx@agu@hand@authoraddrs\the\toks0}\endgroup
\AtEndDocument{\lx@agu@hand@authoraddrs}"
  );
  AddToMacro!("\\lx@maketitle@begin", "\\lx@agu@hand@authoraddrs");
  DefMacro!(
    "\\authoraddress{}",
    "\\g@addto@macro\\lx@agu@authoraddrs{\\lx@add@contact@placed[role=address]{}{#1}}"
  );
  Let!("\\authoraddr", "\\authoraddress");
  // `\altaffilmark` marks an author, `\altaffiltext{n}{…}` gives the marked authors the affiliation (agupp.sty:150,
  // aguplus.cls:239-242 a title footnote), as the AAS binding's.
  aas_support_sty::altaffil_markup()?;
  // The dates printed under the references (agupp.sty:68-70, :119-126 `\@dates`), and the slug comment over the
  // title (agupp.sty:110-117 `\@makeslugcmmnt`; aguplus.cls:245 `\let\date=\slugcomment`), as the AAS binding's.
  DefMacro!(
    "\\received{}",
    "\\lx@add@date[role=received,name=Received]{#1}"
  );
  DefMacro!(
    "\\revised{}",
    "\\lx@add@date[role=revised,name=Revised]{#1}"
  );
  DefMacro!(
    "\\accepted{}",
    "\\lx@add@date[role=accepted,name=Accepted]{#1}"
  );
  DefMacro!("\\slugcomment{}", "\\lx@add@pubnote[role=note]{#1}");
  Let!("\\date", "\\slugcomment");
});
