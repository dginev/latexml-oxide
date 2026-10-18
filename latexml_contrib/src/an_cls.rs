//! an.cls (Astronomische Nachrichten, Wiley-VCH; January 2004, revised 2006-01-13): the papers ship the class (it is
//! not in TeX Live), which is read raw. A paper whose source lacks the class falls to OmniBus, with the loader's
//! missing-file Warning.
use latexml_package::prelude::*;

LoadDefinitions!({
  let Some(class) = crate::shipped_class::requested_shipped_class("an")? else {
    return crate::shipped_class::omnibus_without_copy("an");
  };
  InputDefinitions!(&class, noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // an.cls:414 `\def\aaabstract#1{\gdef\@abstract{#1}}`, which the class makes the document's `\abstract` at its end
  // (:98 `\AtEndOfClass{\let\abstract=\aaabstract}`), stores the abstract for its title page (`\@maketitle` :718
  // `{\@abstract}`): the text was one plain paragraph of the deposited title block, no abstract. The setter is the
  // frontmatter's abstract where the class defines it, before its end-of-class hook copies it onto `\abstract` (the
  // hook runs at the end of this binding's load); the title page reads an empty store, which the class never sets.
  // A class only named like this one has no such setter and is left as it is: `andp2012.cls`, which the prefix
  // fallback answers with this binding (1504.05091, 1810.06303), and the 2014 an.cls v1.12, which defines `\abstract`
  // itself (:422; 1810.02388, 2103.06625). Witness astro-ph0611016 (`\abstract{We consider the contribution of
  // microlensing …}`).
  //
  // v1.03 (2002/07/04, astro-ph0306340) prints the class's label before the store (`\@maketitle` :662
  // `\noindent{\bfseries\abstractname}\ \@abstract`; v1.09a and later comment the line out): the label is the
  // abstract's name, and the title-page deposit takes the line out of `\@maketitle` (`note_title_page_part`), so no
  // lone bold label stays behind; `\abstractname` itself is left as it is.
  const LABELLED_STORE: &str = r"\noindent{\bfseries\abstractname}\ \@abstract";
  let labelled = crate::shipped_class::shipped_class_text("an")?
    .is_some_and(|text| crate::shipped_class::class_code_contains(&text, LABELLED_STORE));
  if labelled {
    RawTeX!(
      r"\ifdefined\aaabstract\def\aaabstract#1{\lx@add@abstract[name={\abstractname}]{#1}}\fi"
    );
    latexml_engine::latex_constructs::note_title_page_part(
      "\\@maketitle",
      TokenizeInternal!(LABELLED_STORE),
    );
  } else {
    RawTeX!(r"\ifdefined\aaabstract\def\aaabstract#1{\lx@add@abstract{#1}}\fi");
  }
  RawTeX!(r"\ifdefined\aaabstract\ifdefined\@abstract\else\let\@abstract\@empty\fi\fi");
});
