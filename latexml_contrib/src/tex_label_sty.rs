//! tex-label.sty: the package is loaded raw; `\labels{<keywords>}` sets the token register the page foot prints
//! (`\rfoot{… \the\labels}`, tex-label.sty:30-32), a page style LaTeXML never typesets (fancyhdr's `\rfoot` is a
//! no-op here, fancyhdr_sty.rs). The settings become a frontmatter `keywords` entry named "Labels", each page's list
//! after the last, separated by `;` — its own entry, so an author's `\keywords` stays (a later entry of the same name
//! replaces the earlier; user ruling 2026-10-03: they are content, not furniture). Witness tex-label/tex-label-demo.
//! Guard `perfect_kernel_batch61::tex_label_keywords_are_frontmatter`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("tex-label", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\let\lx@texlabel@toks\labels
\def\labels{\@ifnextchar={\lx@texlabel@eq}{\lx@texlabel@set}}
\def\lx@texlabel@eq=#1{\lx@texlabel@set{#1}}
\gdef\lx@texlabel@all{}
\def\lx@texlabel@set#1{\lx@texlabel@toks{#1}%
  \ifx\lx@texlabel@all\@empty\gdef\lx@texlabel@all{#1}\else\g@addto@macro\lx@texlabel@all{; #1}\fi
  \expandafter\lx@texlabel@keywords\expandafter{\lx@texlabel@all}}
\def\lx@texlabel@keywords#1{\lx@add@frontmatter{ltx:keywords}[name={Labels:~}]{#1}}"
  );
});
