//! Stub for agujournal2019.cls (AGU journal template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // agujournal2019.cls loads no amsmath (50 copies in a 1/20 arXiv sample): amsmath's commands for papers that use them
  // unloaded, its "Old form" error once the paper loads it; the paper's copy decides
  // (`amsmath_sty::require_amsmath_beyond_original`, KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    &[],
  )?;
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  // agujournal2019.cls:91-112 (2101.01063's copy) `\RequirePackage{xcolor}`, `{url}`, `{lineno}`, `{indentfirst}`,
  // `{ragged2e}`: papers call `\begin{linenomath*}`, `\nolinenumbers`, `\justifying`, `\textcolor` without loading
  // them (run 353: 49 papers with lineno's, 44 with ragged2e's, 18 with xcolor's commands undefined; 1803.00457,
  // 1903.07205, 1902.01137, 2101.01063, 1812.02692). Perl, with no binding, loads the bound ones through OmniBus's scan of
  // the class (Package.pm:2776-2813). The xcolor load was once left out because a document's later
  // `\usepackage[table]{xcolor}` was then a no-op; a repeat load now applies its new options (content.rs
  // `apply_new_options_on_reload`, xcolor_sty.rs `\ds@table`, witness 2605.00310).
  RequirePackage!("xcolor");
  RequirePackage!("url");
  RequirePackage!("lineno");
  RequirePackage!("indentfirst");
  RequirePackage!("ragged2e");
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
