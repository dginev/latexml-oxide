//! smfart.cls (Société Mathématique de France, smflatex, TeX Live 2025): the class is loaded raw, and its
//! `{altabstract}` — the abstract in the document's other language (smfart.cls:568-583), whose body
//! `{@genabstract}` (:506-521) scans up to its `\end` into `\@altabstract`, set in a box under the heading
//! `\altabstractname` and `(\emph{\@alttitle})` when `\alttitle` gave one (:579-581) only by `\@setaltabstract`,
//! which the class's `\maketitle` (or an `\end{altabstract}` after it) runs — is the frontmatter's second abstract,
//! in the other language (:1321-1327: English under a French document, French otherwise), named by the class's
//! caption, the alternative title leading its text as on the page (a name is an attribute: its math and ligatures
//! would be raw TeX there). Under the raw class the kernel's locked `\maketitle` never set the box, so the abstract was lost (2506.20120,
//! smfbourbaki.cls `\LoadClass{smfart}`). The document's `{abstract}` stays the kernel's.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("smfart", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // (as the kernel's `{abstract}` is, sect05.rs: the body read to `\lx@end@abstract`, not clearing the main abstract; its
  // heading the class's caption in the document's language, :1232-1299, which the class sets only at
  // `\begin{document}`, :1306, while the abstracts precede it)
  RawTeX!(
    r"\def\lx@smf@french{french}
\def\lx@smf@alttitle{\ifx\@alttitle\@empty\else(\emph{\@alttitle}) \fi}
\expandafter\def\csname begin{altabstract}\endcsname{\par\begingroup\csname smf@captions\smf@language\endcsname\global\let\lx@smf@altname\altabstractname\endgroup\edef\lx@smf@altlang{\ifx\smf@language\lx@smf@french en\else fr\fi}\lx@frontmatter@mark{ltx:abstract}\expandafter\lx@smf@altabstract\expandafter{\lx@smf@altlang}}
\def\lx@smf@altabstract#1{\lx@add@frontmatter@until{ltx:abstract}[name={\lx@smf@altname},xml:lang=#1]{\lx@end@abstract}\lx@smf@alttitle}
\expandafter\def\csname end{altabstract}\endcsname{\lx@end@abstract}"
  );
});
