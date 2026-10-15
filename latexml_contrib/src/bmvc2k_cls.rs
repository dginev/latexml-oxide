//! Stub for bmvc2k.cls (BMVC British Machine Vision Conference).
use latexml_package::prelude::*;

use crate::shipped_class::{
  SUBFIGURE_PACKAGES, require_shipped_algorithm_packages, require_shipped_packages,
};

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // bmvc2k.cls:149 loads xcolor (`\textcolor` was undefined, 2609.06007). A paper's later `\usepackage[table]{xcolor}`
  // still takes its options: a repeat load applies the options the first lacked (content.rs
  // `apply_new_options_on_reload`, witness 2605.00310).
  RequirePackage!("xcolor");
  // bmvc2k.cls:118 loads xspace (`\xspace` was undefined, 2606.17384; Perl's scan of the class loads it).
  RequirePackage!("xspace");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  // bmvc2k.cls:121 `\RequirePackage{graphicx,subfigure}` in most copies, `subcaption` or `subfig` in edited ones: the
  // paper's copy decides (shipped_class.rs SUBFIGURE_PACKAGES). `\subfigure` was undefined in 123 papers of run 353
  // that rely on the class for it (1507.05726, 1510.01553, 1605.07146). Without a copy, none.
  require_shipped_packages("cls", SUBFIGURE_PACKAGES)?;
  // bmvc2k.cls:222, 245 load geometry (a paper's own `\geometry{…}` was undefined, 2609.00981).
  RequirePackage!("geometry");
  // bmvc2k.cls's algorithm packages differ by year: `algorithm` with `algorithmic` (2002.09958's copy:132-133,
  // 2210.01640's:162-163) or with `algpseudocode` (2210.06270's:139-142). The paper's copy decides; `{algorithm}` and
  // `\State` were undefined in papers that rely on the class for them. Without a copy, none.
  require_shipped_algorithm_packages("cls")?;
  // bmvc2k.cls:516-521, 556-559: the unmarked footnote (a paper's equal-contribution note, 2609.08090) and the running
  // head's "et al." (2609.00981).
  RawTeX!(
    r"\newcommand\BMVA@blfootnote[1]{\begin{NoHyper}\renewcommand\thefootnote{}\footnote{#1}%
\addtocounter{footnote}{-1}\end{NoHyper}}
\def\bmv@EtAl{\em et al.}\def\bmvaEtAl{\protect\bmv@EtAl}"
  );
  // bmvc2k.cls:553-554 `\runninghead{<authors>}{<title>}` stores both for the running head: the short title is kept as
  // OmniBus keeps a running title, the author list is page furniture. OmniBus's one-argument `\runninghead` left the
  // title in the text as a paragraph.
  DefMacro!("\\runninghead{}{}", "\\runningtitle{#2}");

  // bmvc2k frontmatter (L167+) — preserve author content.
  def_macro_noop("\\bmvaOneDot")?;
  DefMacro!("\\bmvaHangBox{}", "#1");
  // bmvc2k.cls:254 sets T1, whose slot 95 is the underscore in every family.
  RequirePackage!("fontenc", options => vec![s!("T1")]);
  // \addauthor{name}{email}{institution-id}: the name is the author, and the mail/homepage is set as a URL in sans
  // (`\DeclareUrlCommand\bmvaUrl{\urlstyle{sf}}`, bmvc2k.cls:161, 376) — read verbatim, its `_`s are not math
  // (2609.06007).
  DefMacro!(
    "\\addauthor{} Semiverbatim {}",
    "\\author{#1}\\lx@add@email{\\sffamily #2}"
  );
  DefMacro!(
    "\\addinstitution{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}"
  );
});
