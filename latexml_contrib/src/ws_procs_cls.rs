//! World Scientific proceedings and review-volume classes (ws-procs9x6, ws-procs961x669, ws-rv9x6): the ws journal
//! binding, and the `\body` these three classes add to it.
use latexml_package::prelude::*;

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
});
