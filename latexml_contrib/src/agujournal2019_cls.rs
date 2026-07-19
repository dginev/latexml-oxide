//! Stub for agujournal2019.cls (AGU journal template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  RequirePackage!("apacite");
  // agujournal2019.cls L118 — the class provides {sidewaystable}/{sidewaysfigure}
  // through rotating; without it the sideways floats are undefined and their
  // \caption leaks ("\caption outside any known float"). Witness 2003.03231.
  RequirePackage!("rotating");

  // agujournal2019.cls L1062-1074: the end-matter {acronyms} and {notation}
  // environments are `\section*` + a `description` list, whose items come from a
  // LOCALLY-redefined \acro{X} / \notation{X} → \item[X]. Ported verbatim (the
  // \vskip glue is visual and dropped). Witness 2003.03231.
  DefMacro!(
    "\\acronyms",
    "\\section*{Acronyms}\\begingroup\\def\\acro##1{\\item[##1]}\\begin{description}"
  );
  DefMacro!("\\endacronyms", "\\end{description}\\endgroup");
  DefMacro!(
    "\\notation",
    "\\section*{Notation}\\begingroup\\def\\notation##1{\\item[\\boldmath ##1]}\\begin{description}"
  );
  DefMacro!("\\endnotation", "\\end{description}\\endgroup");

  // AGU frontmatter (agujournal2019.cls L389+, L573-587).
  // Internal toggles — no content.
  def_macro_noop("\\draftfalse")?;
  def_macro_noop("\\drafttrue")?;
  DefConditional!("\\ifdraft");
  // Author-supplied metadata — preserve as ltx:note frontmatter.
  DefMacro!(
    "\\journalname{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}"
  );
  DefMacro!(
    "\\correspondingauthor{}{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1, #2}"
  );

  // {keypoints} env — AGU title-page key-points list.
  DefEnvironment!(
    "{keypoints}",
    "<ltx:classification scheme='keypoints'>#body</ltx:classification>"
  );
  // AGU plot-axis explanation macros — pass through #2 / #1 so
  // the explanatory text appears in the output.
  DefMacro!("\\xexplain[]{}", "#2");
  DefMacro!("\\yexplain{}", "#1");
});
