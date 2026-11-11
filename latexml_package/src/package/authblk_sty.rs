use crate::prelude::*;

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// authblk: an unlabelled `\author` shows `\AB@thenote` and an unlabelled `\affil` is labelled by it (ruling 1,
/// review revfms2a2's answer to question 6b): when the marks show, the affiliation is the authors' showing its
/// number, a labelled author's too (2605.25017); in block mode (no marks shown) the run of authors since the last
/// `\affil`, as this binding declares it at the call (`annotate=run:<n>`). `written`: the observer's writer writes
/// authblk's creators (brief 95b, OD #942: 2511.07020, 2008.07734, 2110.01283, 2008.13643; a paper whose reading it
/// cannot apply keeps today's output). `author_tail`: its own `\@author` (authblk.sty:168-171) prints the block
/// and what a document appends to it (FX24b, OD #996: 2511.15315).
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "authblk",
  binding: "authblk.sty",
  thenote: true,
  state_counter: Some("Maxaffil"),
  author_tail: true,
  cite: "authblk.sty:114-118, 148-167 (`\\AB@thenote`, `\\newaffil`, `Maxaffil`); Perl authblk.sty.ltxml:57 \
         `annotate=new`; authblk_sty.rs `\\affil` (2605.01689, 2605.25017)",
  ..FM_KERNEL_CONVENTIONS
};

#[rustfmt::skip]
LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
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
  } else if let Some(rows) = lines_of_names_before_affil(&author) {
    // (lines under the names that read as more names, the block's institution in the `\affil` after them: names)
    Ok(Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(
      rows
    )]))
  } else if author
    .unlist_ref()
    .iter()
    .any(|t| t.defined_as(&T_CS!("\\and")) || t.defined_as(&T_CS!("\\And")) || *t == T_OTHER!(","))
    || has_top_level_and_word(&author)
    || has_separated_names(&author)
    || author_marks_end_name(&author, &[T_CS!("\\affil")])
  {
    // `\and`/`\And` or a comma anywhere in the names (a comma inside a group too: `\textbf{Ann Able, Bob Baker}`, whose
    // split reads into the wrapper; one in a note or a mark, `\thanks{Dept. X, Univ Y}`, `$^{1,2}$`, leaves the name
    // whole), the word "and" between two names outside every group, as pdflatex prints them (`\author{Ajay
    // Chandra$^{1}$\orcidlink{…} and Harprit Singh$^2$\orcidlink{…}}`, 2405.16615), a separator the kernel's author
    // read parts the block at (`has_separated_names`: `Ann Able\thanks{…} \qquad Bob Baker\thanks{…}`, 2610.06709;
    // `\hspace{2.5em}`, 2610.18435; `\quad\quad`, 2610.28573, one name holding them all with every note after it, each
    // note's author lost; 2610.39888's `\quad` came here only by the commas in its notes; a line under a name, `Ann
    // Able\thanks{…}\\ Dept. of Physics`, 2609.17614, 2303.15964, name and line one name), or superscript marks ending
    // the name, only notes, spacing or an `\affil` after them (`\author{Cat$^{1}$}`, `\author{Ann$^1$\affil{$^1$Univ X}}`
    // under titling, 2605.03796; `author_marks_end_name`: its mark requests the affiliation labelled by it on every
    // path, as the list's names' marks do; on `\lx@add@creator` it requested nothing, and an affiliation a list author
    // took from it was lost to it, 77n r2 review — a name whose marks other text follows, `Ann Able$^{a}$ (ann@x.org)`,
    // stays on `\lx@add@creator`: the list path shows an unanswered mark after the name's last text, 77n r3 review
    // probes m2, m8, m9) → a multi-author list, split into separate creators by the reading (the same comma /
    // " and " name split the DEFAULT `\author`
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

/// Whether `author` parts its names or lines with a separator the kernel's author read splits at
/// (`has_author_block_separator`: `\quad`, `\qquad`, `\hspace`, `\hfill`, a run of spacing, `\AND`, `\\`), not one
/// inside a note's argument or a mark, and not the space between one person's notes (`Ann Able\thanks{…}\hspace{0.4em}\thanks{…}`,
/// the shape of 2610.04108). A group or a font command around the whole of it is transparent, as for
/// [`has_top_level_and_word`]. Perl's authblk checks only `\and`/`\And` (authblk.sty.ltxml:52-55);
/// OXIDIZED_DESIGN_DIVERGENCES #52(h).
///
/// Lines under the names that read as more names and that nothing tells from an institution are no such separator
/// (`lines_under_names_read_as_names`: `Ann Able\\ Bob Baker`, `Ann Able\quad Bob Baker\\ Cat Cole`, `Ann Able\\ Google
/// DeepMind`): the kernel's read would give the line to the names as their affiliation, silently, which is wrong as often
/// as it is right, so the argument stays one name, the merge the document's Error counts (`flag_merged_creators`) —
/// unless an `\affil` follows, [`lines_of_names_before_affil`]. So does a piece that is a macro printing a link alone
/// (`holds_a_macro_printing_a_link`, 2601.07891): the read would make it a person, silently.
fn has_separated_names(author: &Tokens) -> bool {
  (has_author_block_separator(author)
    && !lines_under_names_read_as_names(author)
    && !holds_a_macro_printing_a_link(author))
    || whole_wrapper_content(author).is_some_and(|inner| has_separated_names(&inner))
}

/// `author` as the row of names it prints when its lines under the names read as more names
/// (`lines_under_names_read_as_names`) and an `\affil` comes next: the block's institution is printed there, so a line
/// of the author font under the names that reads as a name is one (`Aditya Golatkar$^{*}$\hspace{1.6em}…\\[0.35em]
/// Baishakhi Ray\hspace{1.6em}…\\[0.35em] Stefano Soatto}\affil{AWS Agentic AI}`, 2610.38214: a creator, 12 of 12). Not
/// otherwise, nothing else saying which: eight real papers print `\\ Waymo LLC`, `\\ Physical Intelligence`, `\\ UC San
/// Diego`, `\\ World Agents`, `\\ Welo Data` under rows of names with no `\affil` (2404.19531, 2505.23705, 2601.06328,
/// 2610.37518, 2610.27304). Only an argument that reaches the kernel read by its separators alone, as
/// [`has_separated_names`] reads it.
fn lines_of_names_before_affil(author: &Tokens) -> Option<Tokens> {
  let bare =
    !author.unlist_ref().iter().any(|t| {
      t.defined_as(&T_CS!("\\and")) || t.defined_as(&T_CS!("\\And")) || *t == T_OTHER!(",")
    }) && !has_top_level_and_word(author);
  (bare
    && has_author_block_separator(author)
    && lines_under_names_read_as_names(author)
    && next_is_affil())
  .then(|| line_breaks_as_name_separators(author))
}

/// Whether an `\affil` is what the input holds next, past spaces and paragraph breaks.
fn next_is_affil() -> bool {
  peek_upcoming_tokens(6, 2000, 64)
    .find(|t| t.get_catcode() != Catcode::SPACE && *t != T_CS!("\\par"))
    == Some(T_CS!("\\affil"))
}

/// What a group or a font command around the whole of `author` holds, spaces around it aside: `{…}`, `\textbf{…}`,
/// `\emph{…}`; not `\textsuperscript{…}`, a mark.
fn whole_wrapper_content(author: &Tokens) -> Option<Tokens> {
  let v = trimmed_spaces(author.unlist_ref());
  // (the group a wrapper opens: `{…}`, `\cmd{…}`, when it closes on the last token)
  let open = match v.first().map(Token::get_catcode) {
    Some(Catcode::BEGIN) => 0,
    Some(Catcode::CS)
      if v[0].with_str(|cs| {
        (cs.starts_with("\\text") && cs != "\\textsuperscript") || cs == "\\emph"
      }) && v.get(1).is_some_and(|t| t.get_catcode() == Catcode::BEGIN) =>
    {
      1
    },
    _ => return None,
  };
  let mut depth = 0i32;
  let close = (open..v.len()).find(|&k| {
    match v[k].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    depth == 0
  });
  (close == Some(v.len() - 1) && v.len() > open + 2)
    .then(|| Tokens::new(v[open + 1..v.len() - 1].to_vec()))
}

/// `v` without the spaces at its ends.
fn trimmed_spaces(v: &[Token]) -> &[Token] {
  let start = v.iter().position(|t| *t != T_SPACE!()).unwrap_or(v.len());
  let end = v
    .iter()
    .rposition(|t| *t != T_SPACE!())
    .map_or(start, |e| e + 1);
  &v[start..end]
}

/// Whether `author` holds the word "and" at its top level, between spaces or ties (not inside a note's argument or a
/// group: `\thanks{Ann Able and Bob Baker are with X}`). A group or a font command around the whole of it is
/// transparent: `{Nikolai N. Leonenko\textsuperscript{a}\thanks{…} ~and Andrey Pepelyshev\textsuperscript{a}}`
/// (2506.23337), `\textbf{Ann Able and Bob Baker}` hold it at their top level.
fn has_top_level_and_word(author: &Tokens) -> bool {
  if let Some(inner) = whole_wrapper_content(author) {
    return has_top_level_and_word(&inner);
  }
  let v = trimmed_spaces(author.unlist_ref());
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
  // LOCKED, as the kernel's `\title`/`\author` are (sect05.rs): a document's, class's or package's own `\affil` is
  // refused, and its body still runs first for its side effects, its output discarded (`\lx@replay@class@setter`,
  // OXIDIZED_DESIGN #482), so the affiliation stays the binding's contact. The JOSS template's copy of authblk's own
  // `\affil` code (paper_joss.tex:162-182, a `\renewcommand{\affil}[2][]` that differs from authblk.sty:148-167 only in
  // typography) put every affiliation in `\AB@authors`, which nothing prints here (2504.08081, 2310.11002, 2107.08407,
  // 2507.13341, 2604.08855); and so did the raw classes that copy it after `\RequirePackage{authblk}`, oasics.cls:610
  // (1709.00722) and eurocg25.cls (2503.02336), dagman.cls:696 the same (1701.09007; lipics-v2016.cls:611 is read as
  // its binding's stand-in copy, which declines it unrecorded: 1603.00928). A redefinition that takes arguments is refused at
  // every origin (state.rs `install_definition`, OXIDIZED_DESIGN #500); a binding that loads authblk and defines its
  // own `\affil` loads unlocked and keeps it (lni_cls.rs). OXIDIZED_DESIGN_DIVERGENCES #959.
  DefMacro!("\\affil OptionalSemiverbatim {}", sub[(label, affil)] {
    let label = label.unwrap_or_default();
    let replay = Invocation!(T_CS!("\\lx@replay@class@setter"), vec![
      Some(Tokens::new(Explode!("affil"))),
      Some(label.clone()),
      Some(affil.clone())
    ]);
    // (a mark glued to a footnote symbol labels the line by its number, as a name's asks for it apart)
    let affil = glued_line_marks_as_marks(affil, true);
    let run = affil_run();
    // (the label printed as written before the line, `\AB@affilnote`, authblk.sty:142: the mark a legend's note shows,
    // `_labelprints`, OXIDIZED_DESIGN_DIVERGENCES #961)
    let call = if !label.is_empty() {
      Invocation!("\\lx@add@contact[role=affiliation,annotate=1,label={#1},_labelprints=true]{#2}", vec![
        Some(label.clone()),
        Some(affil)
      ])
    } else if opens_with_a_mark(&affil) && queued_authors_carry_entry_mark(&affil) {
      Invocation!(T_CS!("\\lx@add@affiliation@marked"), vec![None, Some(affil)])
    } else if let Some((unmarked, marked)) = unmarked_before_carried_marks(&affil) {
      let mut calls = Vec::new();
      for piece in unmarked {
        calls.extend(
          Invocation!("\\lx@add@contact[role=affiliation,label=addresses:block]{#1}", vec![Some(piece)])
            .unlist(),
        );
      }
      calls.extend(
        Invocation!(T_CS!("\\lx@add@affiliation@marked"), vec![None, Some(marked)]).unlist(),
      );
      Tokens::new(calls)
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
      replay.unlist(),
      call.unlist(),
      Invocation!(T_CS!("\\lx@AB@affil@state"), vec![Some(label)]).unlist(),
    ]
    .concat()))
  }, robust => true, locked => true);
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

/// An affiliation whose first pieces, parted by `\quad`, no mark opens and a later piece a mark a queued author carries
/// opens: those pieces, and the rest. `\textsuperscript{}~Univ. of California \quad \textsuperscript{2}~Northwestern
/// University \quad …` (2505.14999): an empty mark is no mark (`drop_empty_marks`, user 2026-10-20), so the first piece is
/// no one's by a mark, and it is every author's by nothing: the block's, as an unmarked line among marked ones is (user
/// ruling R12), not `Luo$^2$`'s, who shows the mark of his own.
fn unmarked_before_carried_marks(affil: &Tokens) -> Option<(Vec<Tokens>, Tokens)> {
  let pieces = split_tokens_delimited(affil.clone(), vec![
    T_CS!("\\quad").into(),
    T_CS!("\\qquad").into(),
  ]);
  let first = pieces
    .iter()
    .position(|(_, piece)| opens_with_a_mark(piece) && queued_authors_carry_entry_mark(piece))
    .filter(|first| *first > 0)?;
  // (no mark on them, an empty one aside: a legend or an entry a mark leads is the list's, `$^{*}$Corresponding author \\
  // $^{1}$Univ X \quad $^{3}$Univ Z`, 2505.00989)
  if pieces[..first]
    .iter()
    .any(|(_, piece)| author_mark_position(&drop_empty_marks(piece.clone())).is_some())
  {
    return None;
  }
  let mut marked: Vec<Token> = Vec::new();
  for (at, (delimiter, piece)) in pieces.iter().enumerate().skip(first) {
    if at > first {
      marked.extend(delimiter.iter().copied());
    }
    marked.extend(piece.unlist_ref().iter().copied());
  }
  // (each alone, its empty mark no mark of a marked list, what leads it no text)
  let unmarked = pieces[..first]
    .iter()
    .map(|(_, piece)| {
      let piece = drop_empty_marks(piece.clone());
      let v = piece.unlist_ref();
      Tokens::new(v[leading_unprinted_end(v)..].to_vec())
    })
    .filter(|piece| !piece.is_empty() && !prints_nothing(piece))
    .collect();
  Some((unmarked, Tokens::new(marked)))
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
