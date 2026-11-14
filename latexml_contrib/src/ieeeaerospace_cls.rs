//! Stub for IEEEAerospaceCLS (IEEE Aerospace conference template).
//!
//! IEEEAerospaceCLS is an IEEEtran-derived class for the IEEE Aerospace
//! Conference. Route to IEEEtran which supplies \appendices, \PARstart
//! and similar IEEE-template macros. Witness 2408.05924, 2408.06274.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  // IEEEAerospaceCLS.cls L290-292 defines its own sectioning
  // `\def\acknowledgments{\section*{Acknowledgments}\addcontentsline{toc}
  // {section}{Acknowledgments}}` — a NO-arg command that opens an
  // "Acknowledgments" section (content runs until the next section; there is
  // no `\endacknowledgments`, so this is NOT the bounded ltx:acknowledgements
  // environment form). Perl ships no IEEEAerospaceCLS binding → raw-loads the
  // bundled cls and gets this def. Our binding intercepts the cls (routing to
  // IEEEtran), so `\acknowledgments` was undefined where Perl is clean. Mirror
  // the real cls: an unnumbered "Acknowledgments" section (the `\addcontents
  // line` is TOC-only, moot in HTML). Witness 1610.07252. RUST 1 → 0.
  DefMacro!("\\acknowledgments", "\\section*{Acknowledgments}");
  // The current class (2026 template) spells it `\acknowledgements` (IEEEAerospaceCLS.cls:267-272) and opens the
  // biographies the same way, `\thebiography` (:403-408); each biography is `{biography}{<name>}` (a framed photo
  // placeholder, :528-536), `{biographywithpic}{<name>}{<file>}` (:541-549, the name bold italic, :549
  // `\textit{\textbf{#1}}`) or `{biographywithjpg}{<name>}{<file>}` (:554-562, its file name as the class writes it,
  // `{\fighome #2}` :556, `\fighome` the document's to define): the IEEEtran biography, photo and bold name before the
  // text. Witnesses 2601.06000, 2601.00238, 2603.17229.
  DefMacro!("\\acknowledgements", "\\section*{Acknowledgements}");
  DefMacro!("\\thebiography", "\\section*{Biography}");
  DefMacro!(
    "\\biographywithpic{}{}",
    "\\IEEEbiography[{\\includegraphics[width=25mm,height=32mm]{#2}}]{\\textit{#1}}"
  );
  Let!("\\endbiographywithpic", "\\endIEEEbiography");
  DefMacro!(
    "\\biographywithjpg{}{}",
    "\\IEEEbiography[{\\includegraphics[height=32mm]{\\fighome #2}}]{#1}"
  );
  Let!("\\endbiographywithjpg", "\\endIEEEbiography");
});
