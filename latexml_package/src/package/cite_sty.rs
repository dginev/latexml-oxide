use crate::prelude::*;

LoadDefinitions!({
  // Ignore the resorting of citations
  // "Compression" of citation lists doesn't make much sense in XML.

  // Likewise, the options aren't much help.
  // These macros _could_ be used in formatting the citations,
  // but that seems something better left to whoever is formatting
  // the XML at the end.
  // An alternative would be to store various styling info,
  // but I'll leave that off, for now.
  DefMacro!("\\citeleft", "[");
  DefMacro!("\\citeright", "]");
  DefMacro!("\\citedash", "--");
  DefMacro!("\\citemid", ", ");
  DefMacro!("\\citepunct", ", ");
  DefMacro!("\\citeform{}", "#1");

  // Copy of natbib's \citet
  // DefMacro!("\\citen OptionalMatch:* [][] Semiverbatim",
  //   sub[(_star,_pre,_post,_tkeys)] {
  // my ($style, $open, $close, $ns)
  //   = map { LookupValue($_) } qw(CITE_STYLE CITE_OPEN CITE_CLOSE CITE_NOTE_SEPARATOR);
  // if (!$post) { ($pre, $post) = (undef, $pre); }
  // $pre  = undef unless $pre  && $pre->unlist;
  // $post = undef unless $post && $post->unlist;
  // my $author = ($star ? "FullAuthors" : "Authors");
  // if ($style eq 'numbers') {
  //   Invocation(T_CS('\@@cite'),
  //     Tokens(Explode('citet')),
  //     Tokens(    #($pre ? ($pre, T_SPACE) : ()),
  //       Invocation(T_CS('\@@bibref'),
  //         Tokens(Explode("$author Phrase1NumberPhrase2")),
  //         $keys,
  //         Invocation(T_CS('\@@citephrase'),
  //           Tokens($open, ($pre ? ($pre, T_SPACE) : ()))),
  //         Invocation(T_CS('\@@citephrase'),
  //           Tokens(($post ? ($ns->unlist, T_SPACE, $post->unlist) : ()), $close->unlist))
  //       )))->unlist; }
  // elsif ($style eq 'super') {
  //   Invocation(T_CS('\@@cite'),
  //     Tokens(Explode('citet')),
  //     Tokens(($pre ? ($pre, T_SPACE) : ()),
  //       Invocation(T_CS('\@@bibref'),
  //         Tokens(Explode("$author Phrase1SuperPhrase2")),
  //         $keys, undef, undef)->unlist,
  //       ($post ? ($ns, T_SPACE, $post->unlist) : ()))); }
  // else {
  //   Invocation(T_CS('\@@cite'),
  //     Tokens(Explode('citet')),
  //     Invocation(T_CS('\@@bibref'),
  //       Tokens(Explode("$author Phrase1YearPhrase2")),
  //       $keys,
  //       Invocation(T_CS('\@@citephrase'),
  //         Tokens($open, ($pre ? ($pre, T_SPACE) : ()))),
  //       Invocation(T_CS('\@@citephrase'),
  //         Tokens(($post ? ($ns, T_SPACE, $post) : ()), $close)))); }
  // });

  Let!("\\citenum", "\\citen");
  Let!("\\citeonline", "\\citen");
});
