//! ragslab.sty (Resilient AI + Grounded Sensing preprint style, loaded on `article`), whose title API "mirrors the
//! FAIR/Meta template": the paper's copy read raw, for its palette, logos, layout and `\beginappendix`; then the
//! fairmeta family's frontmatter (`meta_class.rs`), so `\author[1,2]{…}` links each author to the `\affiliation[1]{…}`
//! its marks cite, and the metadata band's lines (`\metadata[<label>]{…}`, `\date`, `\correspondence`, `\code`,
//! `\projectpage`) are frontmatter with the label the copy prints (ragslab.sty:140-161; 2610.22523; OD #1070).
use latexml_package::{prelude::*, shipped_class::input_shipped_package};

LoadDefinitions!({
  input_shipped_package()?;
  // The copy's `\maketitle` (:278-284) typesets its lists in a tcolorbox card; the lock refused it, and its replay
  // would print the card, its lists now empty, beside the frontmatter (2610.22523: a picture after the abstract).
  Let!("\\lx@dropped@maketitle", "\\relax");
  // (the copy's `\beginappendix` prints its "Appendix" heading, :286; the shared one is a bare `\appendix`)
  Let!("\\lx@ragslab@beginappendix", "\\beginappendix");
  crate::meta_class::install_meta_class_frontmatter()?;
  if IsDefined!(&T_CS!("\\lx@ragslab@beginappendix")) {
    Let!("\\beginappendix", "\\lx@ragslab@beginappendix");
  }
  // A metadata line prints "<label>: <value>" (:151-152): a publication note named by its label. `\code` and
  // `\projectpage` are the copy's calls of it (:160-161), `\correspondence` too (:159), its role the family's.
  DefMacro!(
    "\\metadata[]{}",
    "\\lx@add@pubnote[role=metadata,name={#1:~}]{#2}"
  );
  DefMacro!(
    "\\correspondence{}",
    "\\lx@add@pubnote[role=correspondence,name={Correspondence:~}]{#1}"
  );
  // `\date` is the copy's "Date:" line (:158), the document's date.
  DefMacro!(
    "\\date{}",
    "\\def\\@date{#1}\\lx@add@date[role=creation,name={Date:}]{#1}"
  );
});
