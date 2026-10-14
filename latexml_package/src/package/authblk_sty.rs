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
  // `\newlength`s, i.e. skips (authblk.sty:76-77): `\setlength` scans by the
  // register's type (OXIDIZED_DESIGN #317).
  DefRegister!("\\affilsep" =>  Glue!("1em"));
  DefRegister!("\\@affilsep" => Glue!("1em"));
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
    // authblk's bookkeeping after the author (`\lx@AB@author@state`, below).
    let mut state = vec![T_CS!("\\lx@AB@author@state"), T_BEGIN!()];
    if let Some(label) = label.as_ref() {
      state.extend(label.unlist_ref().iter().copied());
    }
    state.push(T_END!());
    state.push(T_BEGIN!());
    state.extend(author.unlist_ref().iter().copied());
    state.push(T_END!());
    // What a package appends to `\author` (`\lx@author@after`, sect05.rs; titling's `\theauthor`).
    state.extend([T_CS!("\\lx@author@after"), T_BEGIN!()]);
    state.extend(author.unlist_ref().iter().copied());
    state.push(T_END!());
    let state = Tokens::new(state);
    Ok(Tokens::new([author_call(label, author)?.unlist(), state.unlist()].concat()))
  });

  // Robust, as authblk.sty:148's `\newcommand\affil[2][]` is (`\@protected@testopt`): an `\affil` inside
  // `\author` survives titling's `\protected@xdef\theauthor{\@author}` (witness 2605.03796, whose body was lost).
  DefMacro!("\\affil OptionalSemiverbatim {}",
    "\\lx@add@contact[role=affiliation,annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}\\lx@AB@affil@state{#1}",
    robust => true);

  // authblk.sty's internal state, which documents copying its `\affil` code (a `\email` or second
  // `\affil` command built from it, as ACL/NAACL templates do: `\xdef\AB@authors{\the\@temptokena
  // \AB@las\AB@au@str …}`) read: :98-113 the separators and notes, and the state `\author` (:114-145)
  // and `\affil` (:148-167) keep — the next note (`\AB@thenote`, the `affil` counter), the author
  // just given (`\AB@au@str`), the conjunctions before it (`\AB@las`, `\AB@lasx`, `\AB@as`) and
  // `\ifnewaffil`. Our `\author`/`\affil` make the frontmatter and keep the state as authblk's do;
  // the visual accumulators stay empty (below). Run 340: 163 first errors (witnesses 2103.17238,
  // 1904.03969, 2406.11632).
  RawTeX!(r"\newaffiltrue
\newcommand\AB@blk@and{\protect\Authfont\protect\AB@setsep}
\newcommand\AB@pand{\protect\and \protect\Authfont \protect\AB@setsep}
\@namedef{@sep1}{}\@namedef{@sep2}{\Authand}
\newcommand\AB@affilsep{\protect\Affilfont}
\newcommand\AB@affilsepx{\protect\\\protect\Affilfont}
\newcommand\AB@setsep{\setlength{\@affilsep}{\affilsep}}
\newcommand\AB@resetsep{\setlength{\@affilsep}{\z@}}
\gdef\AB@las{}\gdef\AB@lasx{\protect\Authand}\gdef\AB@as{}\gdef\AB@au@str{}
\def\AB@thenote{\arabic{affil}}\def\AB@note{\AB@thenote}
\xdef\AB@author{\noexpand\AB@blk@and\@author}
\let\AB@maketitle\maketitle
\def\lx@AB@author@state#1#2{\ifnewaffil\addtocounter{affil}{1}\edef\AB@thenote{\arabic{affil}}\fi
\if\relax\detokenize{#1}\relax\def\AB@note{\AB@thenote}\else\def\AB@note{#1}\setcounter{Maxaffil}{0}\fi
\ifnum\value{authors}>1\relax\@namedef{@sep\number\c@authors}{\Authsep}\fi
\addtocounter{authors}{1}%
\ifnewaffil\gdef\AB@las{}\gdef\AB@lasx{\protect\Authand}\gdef\AB@as{}%
\else\global\let\AB@las\AB@lasx\gdef\AB@lasx{\protect\Authands}\gdef\AB@as{\Authsep}\fi
\gdef\AB@au@str{#2}%
\ifnum\value{authors}>2\relax\@namedef{@sep\number\c@authors}{\Authands}\fi
\newaffilfalse}
\def\lx@AB@affil@state#1{\newaffiltrue\let\AB@blk@and\AB@pand
\if\relax\detokenize{#1}\relax\def\AB@note{\AB@thenote}\else\def\AB@note{#1}\setcounter{Maxaffil}{0}\fi
\gdef\AB@las{}\gdef\AB@au@str{}\let\AB@affilsep\AB@affilsepx}");

  // Note formatting — Perl L95-96
  DefMacro!("\\AB@authnote{}",  "\\textsuperscript{\\normalfont#1}");
  DefMacro!("\\AB@affilnote{}", "\\textsuperscript{\\normalfont#1}");
  // authblk.sty:109-112's visual accumulators, at their package-initial EMPTY
  // value: `\author`/`\affil` above emit semantic frontmatter and never fill
  // them, but a class that `\renewcommand`s `\@maketitle` to lay them out
  // (ascelike.cls:406-411 `\AB@authlist`) runs under `\lx@deposit@maketitle`
  // (OXIDIZED_DESIGN #124) and must collapse to nothing rather than hit an
  // undefined name (ascexmpl). Guard:
  // `perfect_kernel_batch54::class_maketitle_layout_over_binding_accumulators`.
  DefMacro!("\\AB@authlist", "");
  DefMacro!("\\AB@affillist", "");
  DefMacro!("\\AB@authors", "");
  DefMacro!("\\AB@empty", "");
});

/// authblk's `\author` as the frontmatter call (Perl authblk.sty.ltxml, PR #2767).
fn author_call(label: Option<Tokens>, author: Tokens) -> Result<Tokens> {
  if let Some(label) = label {
    // Use label attachment; a list of names under one label (`\author[1]{Ann Able, Bob Baker}`; deepseek.cls's
    // `\author[*]{…}` block, 2401.14196) is one author each, every one carrying the label (63e)
    Ok(Invocation!(
      "\\lx@add@authors@append[annotations={#1}]{#2}",
      vec![Some(label), Some(author)]
    ))
  } else if author
    .unlist_ref()
    .iter()
    .any(|t| t.defined_as(&T_CS!("\\and")) || t.defined_as(&T_CS!("\\And")) || *t == T_OTHER!(","))
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
    Ok(Invocation!(T_CS!("\\lx@add@creator"), vec![
      None,
      Some(author)
    ]))
  }
}
