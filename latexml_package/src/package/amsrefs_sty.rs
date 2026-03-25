use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsrefs.sty.ltxml — Leverage the BibTeX implementation

  // Perl: LoadPool('BibTeX');
  // TODO: BibTeX pool not yet ported to Rust

  // Perl: DefParameterType('BibURL', ...) — semiverbatim URL reading
  // Perl: DefKeyVal('amsrefs', 'url', 'BibURL');
  // TODO: BibURL parameter type and amsrefs keyval not yet ported

  // \bib{key}{type}{keyval-pairs}
  // Perl: DefMacro('\bib{}{} RequiredKeyVals:amsrefs', sub { ... });
  // TODO: \bib requires BibTeX pool (CleanBibKey, NormalizeBibKey, ProcessBibTeXEntry)
  DefMacro!("\\bib{}{}{}", "");

  // \BibSpec — ignore
  DefMacro!("\\BibSpec{}{}", "");

  // \cites = \cite
  Let!("\\cites", "\\cite");

  // {bibdiv} environment
  // Perl: DefEnvironment('{bibdiv}', ... beforeDigest, afterDigestBegin ...)
  // TODO: beforeDigestBibliography/beginBibliography_clean not yet available
  DefEnvironment!("{bibdiv}",
    "<ltx:bibliography xml:id='#id'>\
     <ltx:title>#title</ltx:title>\
     #body\
     </ltx:bibliography>");

  // {biblist} environment
  DefEnvironment!("{biblist}", "<ltx:biblist>#body</ltx:biblist>");

  // \MR{...} — MathReviews link
  // Perl: properties => sub { ... patch up old-style MR numbers ... }
  // TODO: properties closure for MR number cleanup (regex s/\s+\\\#\s*/:/
  DefConstructor!("\\MR{}",
    "<ltx:ref class='ltx_mathreviews'>MathReviews</ltx:ref>",
    enter_horizontal => true);

  // \ndash, \mdash
  DefConstructor!("\\ndash", "\u{2013}"); // EN DASH
  DefConstructor!("\\mdash", "\u{2014}"); // EM DASH
});
