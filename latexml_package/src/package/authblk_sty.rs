use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: authblk.sty.ltxml — 100 lines
  // Author/affiliation blocks with mark-based association

  // Font/separator macros — Perl L22-27
  DefMacro!("\\Affilfont", "\\normalfont");
  DefMacro!("\\Authfont",  "\\normalfont");
  DefMacro!("\\Authsep",   ",");
  DefMacro!("\\Authand",   " and ");
  DefMacro!("\\Authands",  ", and ");
  DefMacro!("\\authorcr",  "\\\\");

  // Bookkeeping — Perl L30-38
  DefConditional!("\\ifnewaffil");
  DefRegister!("\\affilsep" =>  Dimension::from_str("1em")?);
  DefRegister!("\\@affilsep" => Dimension::from_str("1em")?);
  NewCounter!("Maxaffil");
  RawTeX!("\\setcounter{Maxaffil}{2}");
  NewCounter!("authors");
  NewCounter!("affil");
  NewCounter!("@affil");

  // authblk supports 3 distinct styles of markup for authors & affiliations
  // LaTeX style:
  //  \author{name\\ affil \and name and name\\ affil}
  // Individually: \author for each author, followed by \affil for affiliation
  //  \author{name} : REPEATED
  //  \affil{affil} : REPEATED, attaches to ALL previous authors which don't have affil!
  // Connected via marks, one \author for each author; one \affil for each affiliation
  //  \author[mark]{name} : REPEATED
  //  \affil[mark]{affil} : REPEATED, attaches to author with same mark
  // Perl: authblk.sty.ltxml (PR #2767)
  DefMacro!("\\author[]{}", sub[(label, author)] {
    if let Some(label) = label {
      // Use label attachment
      Ok(Invocation!("\\lx@add@creator[role=author,annotations={#1}]{#2}",
        vec![Some(label), Some(author)]))
    } else if author.unlist_ref().iter().any(|t|
      t.defined_as(&T_CS!("\\and")) || t.defined_as(&T_CS!("\\And"))
      || *t == T_OTHER!(","))
    {
      // `\and`/`\And` OR a top-level comma → a multi-author list; route to
      // `\lx@add@authors`, which splits it into separate creators via
      // `split_author_line` (the same comma / " and " name split the DEFAULT
      // `\author` already applies). authblk's own no-`\and` branch keeps the
      // whole comma list as ONE `<personname>A, B, C</personname>`; matching the
      // default fixes that inconsistency (html_feedback#6255, googledeepmind).
      // A single name that legitimately contains a comma ("Smith, Jr.") is
      // mis-split, but that is the pre-existing #52 comma tradeoff the default
      // `\author` already makes, not a new divergence. OXIDIZED_DESIGN #52(h).
      Ok(Invocation!(T_CS!("\\lx@add@authors"), vec![Some(author)]))
    } else {
      Ok(Invocation!(T_CS!("\\lx@add@creator"), vec![None, Some(author)]))
    }
  });

  DefMacro!("\\affil OptionalSemiverbatim {}",
    "\\lx@add@contact[role=affiliation,annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}");

  // Note formatting — Perl L95-96
  DefMacro!("\\AB@authnote{}",  "\\textsuperscript{\\normalfont#1}");
  DefMacro!("\\AB@affilnote{}", "\\textsuperscript{\\normalfont#1}");
});
