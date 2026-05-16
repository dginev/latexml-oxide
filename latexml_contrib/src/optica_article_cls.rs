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
  DefMacro!("\\Year{}", "");
  DefMacro!("\\Month{}", "");
  DefMacro!("\\Volume{}", "");
  DefMacro!("\\Page{}", "");

  // {abstract*} environment.
  DefEnvironment!(
    "{abstract*}",
    "<ltx:abstract>#body</ltx:abstract>"
  );
});
