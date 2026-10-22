use crate::prelude::*;

LoadDefinitions!({
  // Perl ignores the resorting of citations ("Compression" of citation lists doesn't make much sense in XML).
  // cite.sty sorts a citation's numbers and joins a run of three or more with `\citedash` (cite.sty:95, :272-297),
  // unless `nosort`/`nocompress` (:412, :415): the `ltx:bibref`'s class says so to the post stage, which knows the
  // numbers (sect11.rs `cite_order_class`; KNOWN_PERL_ERRORS #1072). Guard
  // `perfect_kernel_batch78j::cite_sty_sorts_and_compresses`.
  AssignValue!("CITE_SORT" => true);
  AssignValue!("CITE_COMPRESS" => true);
  // (a label not a number, an undefined key, printed at once, before the sorted ones: cite.sty:205-230
  // `\@cite@dump@now`, "[Knu84, Lam86, 1]"; guard `perfect_kernel_batch78j::cite_sty_prints_unsorted_labels_first`)
  AssignValue!("CITE_UNSORTED_FIRST" => true);
  DeclareOption!("nosort", {
    AssignValue!("CITE_SORT" => false);
  });
  DeclareOption!("nocompress", {
    AssignValue!("CITE_COMPRESS" => false);
  });
  // cite.sty:401-423, its other options: punctuation, penalties, superscripts, placement — not modelled.
  for option in [
    "verbose",
    "nospace",
    "space",
    "nobreak",
    "ref",
    "sort",
    "compress",
    "nomove",
    "move",
    "super",
    "superscript",
    "noadjust",
    "adjust",
    "biblabel",
  ] {
    DeclareOption!(option, None);
  }
  ProcessOptions!();

  // The other options aren't much help.
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

  // Perl: \citen is a complex closure (natbib-style). For now, delegate to \cite.
  Let!("\\citen", "\\cite");
  Let!("\\citenum", "\\cite");
  Let!("\\citeonline", "\\cite");
});
