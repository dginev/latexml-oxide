use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Strict-Perl translation of LaTeXML/lib/LaTeXML/Package/expl3.sty.ltxml:
  //   LoadPool('LaTeX');
  //   InputDefinitions('expl3', type => 'lua');
  //   InputDefinitions('expl3', type => 'sty', noltxml => 1);
  //
  // The raw expl3.sty file has a TeX-level guard
  //   \expandafter\ifx\csname tex_let:D\endcsname\relax
  //     \expandafter\@firstofone\else\expandafter\@gobble\fi
  //     {\input expl3-code.tex }%
  // which detects the dump-loaded `\tex_let:D` PA-alias and skips
  // re-loading expl3-code.tex. So this 3-line wrapper does the right
  // thing: load lua portion, then load .sty (which short-circuits).
  LoadPool!("LaTeX");
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")), notex => true);
  let _ = input_definitions("expl3", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));
});
