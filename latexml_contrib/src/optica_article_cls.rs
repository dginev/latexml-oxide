//! Stub for optica-article.cls (Optica/OSA journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Optica-specific frontmatter / formatting.
  DefMacro!("\\authormark{}", "\\textsuperscript{#1}");
  DefMacro!("\\bmsection{}", "\\par\\medskip\\noindent\\textbf{#1.}\\enspace");
  DefMacro!("\\JournalTitle{}", "\\emph{#1}");
  // Bibliographic metadata — preserve author values.
  DefMacro!("\\Year{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\Month{}",
    "\\@add@frontmatter{ltx:note}[role=month]{#1}");
  DefMacro!("\\Volume{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\Page{}",
    "\\@add@frontmatter{ltx:note}[role=page]{#1}");

  // {abstract*} environment.
  DefEnvironment!(
    "{abstract*}",
    "<ltx:abstract>#body</ltx:abstract>"
  );
});
