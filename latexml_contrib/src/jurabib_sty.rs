//! jurabib.sty — law-style citations: the raw package, its citation commands handed to the kernel.
//!
//! jurabib's `\@citex[annotator][postnote]{keys}` (jurabib.sty:5699) typesets each citation from `\b@<key>`, the
//! record jurabib.bst writes to the `.aux` (`\bibcite{key}{{Author}{Short title}{…}}`), which LaTeXML never has: every
//! citation printed pdflatex's first pass — `?, .` in a footnote, the postnote dropped — and registered nothing, so the
//! bibliography selected no entry and stayed empty (jbtest, the package's manual; Perl has no binding and alike). The
//! raw package keeps its options, formatting macros and switches; its citation commands become the kernel's
//! `\@@cite`/`\@@bibref`, which the bibliography stage resolves, as abntex2cite's (abntex2cite_sty.rs) and natbib's.
//!
//! The arguments follow jurabib's (`\jb@@cite`, :3760-3762; `\@citex`, :5719-5752): one optional is the postnote; two
//! are annotator and postnote, swapped under `jurabiborder` (`\ifjb@old@order`) when the first is not empty. The
//! annotator follows the author after `\jbhowsepannotatorlast` (`/`, :1818) — or, with `annotatorfirst`, precedes it
//! before `\jbhowsepannotatorfirst` — and the postnote follows `\jbprformat` (:1262) after a space (a comma under
//! `commabeforerest`). A footnote citation ends with `\unskip.` (`\@cite`, :5704; `\jb@footnote@periodtrue`, :3905,
//! cleared only by the ibidem/opcit forms BibTeX decides). `\footcite`, `\fullcite`, `\footfullcite`, `\citetitle`,
//! `\footcitetitle` are jurabib's `\jb…` commands, `\let` at `\begin{document}` (:5949-5955), so the overlay is made on
//! those; `\footcite*` is `\jbfootcitenotitle` (:3881). `\cite` is a footnote under `super` (`\@citex`'s choice,
//! :5702-5711; `jb@foot` is the `\jbfoot…` commands' own switch). The citation shows the authors (`Authors`), a full
//! citation the entry (`FullEntry`), a title citation the title (`Title`).
//!
//! jurabib's `\bibstyle` (:1195-1212) replaces the kernel's constructor, which records the style on the bibliography;
//! both run. jurabib's styles are author-year (each `.bst`'s SORT: jurabib, jox, jureco sorted; jurunsrt unsorted;
//! latex_constructs `lookup_bibstyle_params`) and leave titles as written (no `change.case$` on titles).
//! Residuals: the short title jurabib.bst adds when one author has several works (decided by BibTeX, not known at
//! digestion); several authors joined "A and B" where jurabib writes "A/B" (`\jbbtasep`); the natbib-style commands
//! (`\citet`, `\citep`, …; no witness); under `super`, a `\cite` inside a document's `\footnote` is a nested note
//! (jurabib's `\ifjb@fn` is set by its `\@footnotetext` wrap, :3005-3010, which LaTeXML's `\footnote` never runs).
//! KNOWN_PERL_ERRORS #452. Witness jbtest. Guard `06_cluster_bibliography::jurabib_citations_are_bibrefs`.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!(r"\let\lx@jb@kernel@bibstyle\bibstyle");
  let opts: Vec<String> = lookup_vecdeque("opt@jurabib.sty")
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  InputDefinitions!("jurabib", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true, options => opts);
  AssignValue!("CITE_STYLE" => "authoryear", Some(Scope::Global));
  AssignValue!("BibTeX_title_case" => "asis", Some(Scope::Global));
  RawTeX!(r"\let\lx@jb@bibstyle\bibstyle\def\bibstyle#1{\lx@jb@kernel@bibstyle{#1}\lx@jb@bibstyle{#1}}");
  // \lx@jb@cite{<show>}{<cite class>}{<footnote: 1|0>}[<annotator>][<postnote>]{<keys>}
  RawTeX!(r"\def\lx@jb@cite#1#2#3{\@ifnextchar[{\lx@jb@cite@i{#1}{#2}{#3}}{\lx@jb@cite@ii{#1}{#2}{#3}[][]}}
\def\lx@jb@cite@i#1#2#3[#4]{\@ifnextchar[{\lx@jb@cite@ii{#1}{#2}{#3}[#4]}{\lx@jb@cite@ii{#1}{#2}{#3}[][#4]}}
\def\lx@jb@cite@ii#1#2#3[#4][#5]#6{\let\lx@jb@next\@secondoftwo
  \ifjb@old@order\ifx\relax#4\relax\else\let\lx@jb@next\@firstoftwo\fi\fi
  \lx@jb@next{\lx@jb@cite@iii{#1}{#2}{#3}{#5}{#4}{#6}}{\lx@jb@cite@iii{#1}{#2}{#3}{#4}{#5}{#6}}}
\def\lx@jb@cite@iii#1#2#3#4#5#6{\ifnum#3=1 \expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
  {\unskip\footnote{\lx@jb@body{#1}{#2}{#4}{#5}{#6}\unskip.}}{\lx@jb@body{#1}{#2}{#4}{#5}{#6}}}
\def\lx@jb@body#1#2#3#4#5{\@@cite[#2]{\ifjb@annotator@last
    \@@bibref{#1}{#5}{}{}\ifx\relax#3\relax\else\jbhowsepannotatorlast#3\fi
  \else
    \ifx\relax#3\relax\else#3\jbhowsepannotatorfirst\fi\@@bibref{#1}{#5}{}{}\fi
  \ifx\relax#4\relax\else\ifjb@comma@before@rest, \else\space\fi\jbprformat{#4}\fi}}
\def\lx@jb@citefoot{\ifjb@foot1\else\ifjb@super1\else0\fi\fi}
\def\cite{\@ifstar{\lx@jb@cite{Authors}{cite}{\lx@jb@citefoot}}{\lx@jb@cite{Authors}{cite}{\lx@jb@citefoot}}}
\def\jbfootcite{\@ifstar{\jbfootcitenotitle}{\lx@jb@cite{Authors}{cite}{1}}}
\def\jbfootcitenotitle{\lx@jb@cite{Authors}{cite}{1}}
\def\jbfullcite{\lx@jb@cite{FullEntry}{fullcite}{0}}
\def\jbfootfullcite{\lx@jb@cite{FullEntry}{fullcite}{1}}
\def\jbcitetitle{\lx@jb@cite{Title}{citetitle}{0}}
\def\jbfootcitetitle{\lx@jb@cite{Title}{citetitle}{1}}");
});
