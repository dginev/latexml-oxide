//! Stub for interact.cls (Taylor & Francis interact class).
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
  RequirePackage!("booktabs");
  RequirePackage!("graphicx");

  // Author-block macros — preserve author content.
  DefMacro!("\\name{}", "#1");
  DefMacro!("\\affil{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}");
  def_macro_noop("\\affilskip")?;

  // {amscode} env — interact L507.
  DefEnvironment!(
    "{amscode}",
    "<ltx:classification scheme='AMS'>#body</ltx:classification>"
  );
  // \amscodename — the "AMS CLASSIFICATION" label (interact.cls L718:
  // `\newcommand\amscodename{AMS CLASSIFICATION}`). Used inside {amscode}'s
  // body, but papers also call it standalone, e.g.
  // `\amscodename{: Primary 60H15; 37H05.}`. Define verbatim as the label so
  // the classification text survives inline. Witness 2008.01335 (1 err → 0;
  // Perl, with no interact binding → OmniBus, errors on both \amscodename and
  // \name). Mirror sibling label macros \keyname/\jelname if seen later.
  DefMacro!("\\amscodename", "AMS CLASSIFICATION");

  // Frontmatter metadata — preserve author content.
  DefMacro!("\\articletype{}",
    "\\@add@frontmatter{ltx:note}[role=articletype]{#1}");
  DefMacro!("\\authormark{}", "\\textsuperscript{#1}");
  DefMacro!("\\corres{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
  DefMacro!("\\journalname{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
});
