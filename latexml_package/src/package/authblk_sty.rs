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
  define_author_and_affil()?;

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
    || has_top_level_and_word(&author)
    || author_marks_end_name(&author, &[T_CS!("\\affil")])
  {
    // `\and`/`\And` or a comma anywhere in the names (a comma inside a group too: `\textbf{Ann Able, Bob Baker}`, whose
    // split reads into the wrapper; one in a note or a mark, `\thanks{Dept. X, Univ Y}`, `$^{1,2}$`, leaves the name
    // whole), the word "and" between two names outside every group, as pdflatex prints them (`\author{Ajay
    // Chandra$^{1}$\orcidlink{…} and Harprit Singh$^2$\orcidlink{…}}`, 2405.16615), or superscript marks ending the
    // name, only notes, spacing or an `\affil` after them (`\author{Cat$^{1}$}`, `\author{Ann$^1$\affil{$^1$Univ X}}`
    // under titling, 2605.03796; `author_marks_end_name`: its mark requests the affiliation labelled by it on every
    // path, as the list's names' marks do; on `\lx@add@creator` it requested nothing, and an affiliation a list author
    // took from it was lost to it, 77n r2 review — a name whose marks a line or other text follows, `Ann Able$^{1}$ \\
    // Univ X`, `Ann Able$^{a}$ (ann@x.org)`, stays on `\lx@add@creator`: the list path shows an unanswered mark after
    // the name's last text and drops one before a `\\`, 77n r3 review probes m2, m7, m8, m9) → a multi-author list,
    // split into separate creators by `split_author_line` (the same comma / " and " name split the DEFAULT `\author`
    // already applies) and added after the authors before it, as authblk adds every `\author` to its list
    // (authblk.sty:114-145): `\lx@add@authors@list`. authblk's own no-`\and` branch keeps the whole comma list as ONE
    // `<personname>A, B, C</personname>`; matching the default fixes that inconsistency (html_feedback#6255,
    // googledeepmind). A single name that legitimately contains a comma ("Smith, Jr.") is mis-split, but that is the
    // pre-existing #52 comma tradeoff the default `\author` already makes, not a new divergence. OXIDIZED_DESIGN #52(h).
    // Replacing the authors before (`\lx@add@authors`) dropped them: `\author{Ann Able\thanks{Dept. X, Univ
    // Y}}\affil{X}\author{Bob Baker\thanks{Dept. Z, Univ W}}` lost Ann Able, whose note's comma sent her here too (the
    // split reads one name there: a note's comma is no separator); AIAA new-aiaa.cls papers kept 1 of 3 authors
    // (2610.21324, 2008.05535). Perl's `\and` branch (authblk.sty.ltxml:52-53, `\lx@add@authors`) replaces them too
    // (KNOWN_PERL_ERRORS #992).
    Ok(Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(
      author
    )]))
  } else {
    // (a name whose marks text follows, `Bob Baker$^{1}$\textsuperscript{equal contrib.}`, `Bob Baker$^{1}$
    // (bob@x.org)`, kept as it prints, its marks still requesting the affiliations they pick, as on the list path:
    // `author_label_marks`, 77n r5 review probes k1, k2, k5)
    let requests = author_label_marks(&author, &[T_CS!("\\affil")]);
    let mut name = author.unlist();
    for marks in requests {
      name.extend(
        Invocation!(
          "\\lx@request@frontmatter@annotation[affiliation]{#1}",
          vec![Some(marks)]
        )
        .unlist(),
      );
    }
    Ok(Invocation!(T_CS!("\\lx@add@creator"), vec![
      None,
      Some(Tokens::new(name))
    ]))
  }
}

/// Whether `author` holds the word "and" at its top level, between spaces or ties (not inside a note's argument or a
/// group: `\thanks{Ann Able and Bob Baker are with X}`).
fn has_top_level_and_word(author: &Tokens) -> bool {
  let v = author.unlist_ref();
  let gap = |t: Option<&Token>| t.is_some_and(|t| *t == T_SPACE!() || *t == T_ACTIVE!('~'));
  let mut depth = 0i32;
  (0..v.len()).any(|i| {
    match v[i].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    depth == 0
      && v[i..].starts_with(&[T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")])
      && gap(i.checked_sub(1).and_then(|b| v.get(b)))
      && gap(v.get(i + 3))
  })
}

/// authblk's `\author[mark]{name}` and `\affil[mark]{affiliation}` (authblk.sty:114-167) as frontmatter calls; also
/// run by a class that copies authblk's code over them (lni.cls:384-464, `lni_cls.rs` in latexml_contrib), so the
/// copy's accumulators — read only by its refused `\maketitle` — do not take the author block.
#[rustfmt::skip]
pub fn define_author_and_affil() -> Result<()> {
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
  // An unlabelled `\affil` is every author's given since the last one (`annotate=run:N`, [`affil_run`]): its note is
  // `\AB@thenote`, which only the next `\author` after an `\affil` steps (authblk.sty:115-116), so consecutive `\affil`s
  // (`\author{Ann Able and Bob Baker}\affil{Univ X}\affil{Univ Y}`) are all printed under the same authors; an author
  // of the group with an affiliation already (a `\\` line under the name) is passed over, as the run does. One that
  // opens with a hand-written mark a queued author carries too (`\author{Ann Able$^{1}$ and Bob Baker$^2$}\affil{$^1$Univ
  // X}\affil{$^2$Univ Y}`; 1507.08653, 2510.01944) goes to the authors showing its mark, as a marked affiliation list's
  // lines do (`\lx@add@affiliation@marked`) — beyond authblk, whose `\AB@thenote` prints it under the whole block
  // (OXIDIZED_DESIGN #765). A footnote symbol (`$^*$Equal contribution`) or a mark no author carries (a typo, an author
  // without marks after marked ones) keeps authblk's run: a label no one requests would go to a nameless author.
  DefMacro!("\\affil OptionalSemiverbatim {}", sub[(label, affil)] {
    let label = label.unwrap_or_default();
    let run = affil_run();
    let call = if !label.is_empty() {
      Invocation!("\\lx@add@contact[role=affiliation,annotate=1,label={#1}]{#2}", vec![
        Some(label.clone()),
        Some(affil)
      ])
    } else if opens_with_a_mark(&affil) && queued_authors_carry_entry_mark(&affil) {
      Invocation!(T_CS!("\\lx@add@affiliation@marked"), vec![None, Some(affil)])
    } else if run > 0 {
      Invocation!("\\lx@add@contact[role=affiliation,annotate=run:#1]{#2}", vec![
        Some(Tokens::new(Explode!(run.to_string()))),
        Some(affil)
      ])
    } else {
      // (no author queued as it expands — inside `\author{Ann \affil{Univ X}}`, read at the flush, or after `\maketitle`
      // (2605.01689): the run uncapped, as Perl's `annotate=new` (authblk.sty.ltxml:57) reaches them; a cap of 0
      // attached it to no one, 77n r3 review)
      Invocation!("\\lx@add@contact[role=affiliation,annotate=run]{#1}", vec![Some(affil)])
    };
    Ok(Tokens::new([
      call.unlist(),
      Invocation!(T_CS!("\\lx@AB@affil@state"), vec![Some(label)]).unlist(),
    ]
    .concat()))
  }, robust => true);
  Ok(())
}

/// The number of creators an unlabelled `\affil` is printed under at most: the ones queued since the last `\affil`
/// before them, which share its `\AB@thenote` (authblk.sty:115-116 step it at the first `\author` after an `\affil`)
/// — the same group as the `\affil` before when no author came between (`\author{Ann}\author{Bob}\affil{X}\affil{Y}`:
/// both under both). Counted here, as every `\affil` passes: a mark-picked one too, whose affiliation reaches its
/// authors only once their marks are matched, so the run alone would take its authors for new ones
/// (`\author{Ann$^1$}\affil{$^1$X}\author{Bob}\affil{$^2$Y}` printed Y under Ann as well, 77n r2 review); 0 before
/// any author.
fn affil_run() -> usize {
  let count = queued_creator_count();
  let (before, run) = match lookup_value("lx_AB_affil_run") {
    Some(Stored::String(last)) => with(last, |s| {
      let (before, run) = s.split_once('|').unwrap_or_default();
      (before.parse().unwrap_or(0), run.parse().unwrap_or(0))
    }),
    _ => (0, 0),
  };
  // (a flush that cleared the queue since starts the count anew)
  let run = match count.checked_sub(before) {
    Some(0) => run,
    Some(new) => new,
    None => count,
  };
  assign_value(
    "lx_AB_affil_run",
    Stored::String(pin(s!("{count}|{run}"))),
    Some(Scope::Global),
  );
  run
}

/// Whether an affiliation opens with a superscript mark, `$^{1}$` or `\textsuperscript{1}`, after what prints nothing
/// before it: spaces, spacing and declarations (`leading_unprinted_end`, as an author block's lines are read: 2605.11124's
/// `\affil{\vspace{-1.05em}$^2$…}` went to every author since the last `\affil`, 77n r5 review).
fn opens_with_a_mark(affil: &Tokens) -> bool {
  let v = affil.unlist_ref();
  let mut i = leading_unprinted_end(v);
  while v.get(i) == Some(&T_SPACE!()) {
    i += 1;
  }
  match v.get(i) {
    Some(t) if *t == T_CS!("\\textsuperscript") => true,
    Some(t) if t.get_catcode() == Catcode::MATH => v.get(i + 1) == Some(&T_SUPER!()),
    _ => false,
  }
}
