use latexml_package::prelude::*;

LoadDefinitions!({
  // No upstream binding: cclayout.sty (computational complexity, loaded by cc.cls:229-230 under its default `cclayout`
  // option) is read raw, then two of its title-page stores are the frontmatter's. `\Abstract{…}` (:66
  // `\newcommand{\Abstract}{\for@cc@abstract}`, :63-64 `\gdef\@abstract@macro{\begin{@abstr}{\ccabstractname}#1
  // \end{@abstr}}`) and `{subject}` (:81 `\newenvironment{subject}{\Collect@Body\for@cc@subject}{}`, :76-80
  // `\gdef\@subject@macro{…{\ccsubjectclassificationname}…}`) store their text for the class's `\maketitle` (:885-905),
  // whose title page is not set: the abstract and the subject classification ("68Q15, 68Q17") were lost with 0 errors.
  // The setters add them where the package defines them, named by its labels (pdflatex: "Abstract. …", "Subject
  // classification. 68Q15, 68Q17"). `{keywords}` (:71-73) is K11's collected-body reroute
  // (frontmatter_stores.rs `collects_body_for_the_title_page`; 1905.07821) and stays the package's.
  InputDefinitions!("cclayout", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\long\def\for@cc@abstract#1{\lx@add@abstract[name={\ccabstractname}]{#1}}
\long\def\for@cc@subject#1{\lx@add@classification[name={\ccsubjectclassificationname}]{#1}}"
  );
});
