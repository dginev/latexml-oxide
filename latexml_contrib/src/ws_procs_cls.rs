//! World Scientific proceedings and review-volume classes (ws-procs9x6, ws-procs961x669, ws-rv9x6): the ws journal
//! binding, and the `\body` these three classes add to it.
use latexml_package::prelude::*;

use crate::shipped_class::require_shipped_packages;

LoadDefinitions!({
  crate::ws_journal_cls::load_definitions()?;
  // ws-procs9x6.cls:586-591 (ws-procs961x669.cls:596-601, ws-rv9x6.cls:1011 the same; the journal classes define
  // neither, so a paper's own `\newcommand{\body}` stays theirs): `\body` (and its alias `\bodymatter`) opens the main
  // matter, restarting the footnotes and lettering them; the class's `\@makefnmark` there (a roman superscript) is the
  // printed mark's look only. Witnesses 0704.0883, 1008.0906, 1903.10424, 2301.12666, 1308.0373, 2202.11015
  // (ws-procs961x669), 0902.1904 (ws-rv9x6).
  RawTeX!(
    r"\def\body{\setcounter{footnote}{0}\def\thefootnote{\alph{footnote}}}
\def\bodymatter{\body}"
  );
  // ws-rv9x6.cls:27 the chapter-numbering switch `\if@addchapnum` (the class's `addchapnum` option, :44, sets it;
  // numbering is presentation here), which papers test in their own numbering macros (run 340: 113 papers; witnesses
  // 2103.07802, 2008.12305, 1609.07109).
  RawTeX!(r"\newif\if@addchapnum");
  // ws-rv9x6.cls:1524-1528 (0706.1148's copy) `\usepackage[figuresright]{rotating}`, `{amsfonts,amssymb,amsmath}`,
  // `{boxedminipage}`, `{makeidx}`, `{url}`: a review-volume chapter ends with `\printindex` without loading makeidx
  // (33 ws-rv9x6 papers of run 353; 0706.1148, 0810.2192, 0901.1361). The paper's copy decides, for these three classes
  // alike; without one, none.
  require_shipped_packages("cls", &["rotating", "boxedminipage", "makeidx", "url"])?;
});
