//! vmsta.cls (Modern Stochastics: Theory and Applications, VTeX): the papers ship the class (it is not in TeX Live) and
//! it is read raw, with the author name markup its title page prints. A paper whose source lacks the class falls to
//! OmniBus, with the loader's missing-file Warning.
use latexml_package::prelude::*;

const VMSTA: crate::shipped_class::RawRead = crate::shipped_class::RawRead {
  shipped: true,
  // vmsta.cls:278-291 defines the author's name parts only where its `\@@author` prints the author list: `\name@m@front`
  // stores `\fnm{…}` and `\snm{…}`, drops `\inits` and `\suffix`, and prints `\@fnm~\@snm` after them — "Yuliya
  // Mishura" for `\author[a]{\inits{Yu.}\fnm{Yuliya}\snm{Mishura}\corref{cor1}}` (the initials serve its e-mail line,
  // `\name@m@ead`, "(Yu. Mishura)"). The kernel's `\author` replaces the class's, so the author content takes them as
  // its title page does (`\lx@author@markup@class`, for the author content only; was "Yu.YuliyaMishura", the
  // class's `\name@m` identities that #647 found and OmniBus's `\inits`): 71 papers, 1508.02842, 1411.4499, 1601.01109.
  // A first name no `\snm` follows is printed after the content, as `\@fnm~\@snm` prints it there (`\fnm{Ivan}~Voronov`
  // "VoronovIvan", where vmsta.cls errs on its undefined `\@snm`; the rev77t review's finding 7): flushed with the
  // markup's restore list, which runs after the author content.
  after: r"\let\lx@vmsta@fnm\@empty
\def\lx@vmsta@first#1{\def\lx@vmsta@fnm{#1}}
\def\lx@vmsta@sur#1{\ifx\lx@vmsta@fnm\@empty\else\lx@vmsta@fnm~\fi#1\let\lx@vmsta@fnm\@empty}
\def\lx@vmsta@flush{\lx@vmsta@fnm\let\lx@vmsta@fnm\@empty}
\def\lx@author@markup@class{\lx@author@markup@provide\inits\@gobble\lx@author@markup@provide\suffix\@gobble\lx@author@markup@provide\fnm\lx@vmsta@first\lx@author@markup@provide\snm\lx@vmsta@sur\expandafter\def\expandafter\lx@author@markup@unprovide\expandafter{\expandafter\lx@vmsta@flush\lx@author@markup@unprovide}}",
  ..crate::shipped_class::RawRead::new("vmsta", "cls")
};

LoadDefinitions!({
  crate::shipped_class::read_raw(&VMSTA)?;
});
