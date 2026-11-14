//! ijsra.cls: the class is loaded raw, with its frontmatter slots free and its abstract environment semantic.
//!
//! The class keeps its frontmatter in `\def` slots the document fills: `\def\abstract{…}`, which `\IJSRAopening` prints
//! through `\begin{IJSRAabstract}\abstract\end{IJSRAabstract}` when `\ifdef\abstract` (ijsra.cls:475-479), after
//! `\newcommand\abstract{}\undef\abstract{}` (:372-373) left it undefined. Under the kernel's lock on `\abstract` (sect05.rs,
//! as Perl, pool:1136) the document's value was refused, so the bare use opened a frontmatter capture inside the
//! `\list` of `{IJSRAabstract}`, whose `\endlist` landed in it: the text was lost (Error `\endlx@list`). This class's name
//! is free of that lock (as amsppt's, amsppt_sty.rs), so `\ifdef\abstract` means what it says, and an ijsra document
//! without an abstract prints none. `{IJSRAabstract}` (:415-420), the opening's and a document's own second one
//! (ijsra.tex:410, `\foreignlanguage{french}`), is the kernel's `{abstract}`, its keywords line (:417) a `ltx:keywords`.
//! The `\list`, `\footnotesize\sffamily` and `\itshape` are its print layout. Witness ijsra/ijsra (TeX Live manual); guard
//! `perfect_kernel_batchm1::the_ijsra_abstract_is_semantic`. Open: `\IJSRAopening` prints the `\authorone`… slots as a
//! `{center}`, so the document has no `ltx:creator`.
use latexml_package::prelude::*;

LoadDefinitions!({
  AssignValue!("\\abstract:locked" => false, Some(Scope::Global));
  RawTeX!(r"\let\abstract\@undefined");
  InputDefinitions!("ijsra", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  RawTeX!(
    r"\renewenvironment{IJSRAabstract}
  {\ifx\keywords\@empty\else\lx@add@frontmatter@now{ltx:keywords}[name={\keywordname:~}]{\keywords}\fi\begin{abstract}}
  {\end{abstract}}"
  );
});
