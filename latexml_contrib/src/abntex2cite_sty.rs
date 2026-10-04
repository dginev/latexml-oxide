//! abntex2cite.sty — ABNT citations (Brazilian standard), the bibliography end.
//!
//! The raw package is loaded with the document's options (`num`/`alf`,
//! `foot`, …; abntex2cite.sty:158-174); its `\bibliography` redefinition is
//! refused by the kernel's lock and its `\cite` handed back to the kernel:
//! abntex2cite.sty:375 redefines `\bibliography` to write `\bibdata` to the `.aux` and
//! `\@input@{\jobname.bbl}` — a file only a bibtex run produces, which LaTeXML
//! never has, so the reference list vanished at 0 errors (abntex2cite's manual
//! is half bibliography; abntex2cite-alf and unbtex share it). The native
//! `\bibliography` (latex_constructs.pool.ltxml:3895; sect11.rs) prefers a
//! shipped `.bbl` and otherwise runs the `.bib` session at post
//! (`\lx@ifusebbl`), and the `.bib` list is live only here, in the
//! command's argument — the same interception Perl makes for bibunits'
//! `\bibliography` wrapper (bibunits.sty.ltxml; OXIDIZED_DESIGN_DIVERGENCES
//! #87); since batch 56cx the kernel macro is locked against that
//! redefinition (OXIDIZED_DESIGN_DIVERGENCES #236). Perl has no abntex2cite
//! binding and loses the list too. The
//! package's own `\cite` (abntex2cite.sty:748/:862, a `\DeclareRobustCommand`
//! resolving numbers from the `.aux`) typeset `(??)` and registered no
//! citation, so no entry was ever selected; `\cite` and `\citeonline` (the
//! textual form) keep the kernel's citation constructor, which the
//! bibliography stage resolves.
//!
//! 57cm (rc131 root-cause of abntex2cite, 80.5 % recall, bibliography-only):
//! - the package's `\bibliographystyle` (:409-417) only writes to the `.aux`, and its
//!   `\AtEndDocument` default (:420-424) goes through it, so no style ever reached the
//!   bibliography (`bibstyle` unset: URLs printed as "Link"). It now forwards to the
//!   kernel's, and the package's default, `abntex2-\AbntCitetype`, is recorded at load;
//! - abntex2's own `.bib` fields, which its `.bst`s print (abntex2-num.bst:26-49,
//!   :585-602, :997-1000, :1399-1400, :1645), were kept as unprinted `ltx:bib-data`;
//! - `\citeyear`/`\citeauthoronline` (:927, :1019) read values only a real BibTeX run
//!   leaves in the `.aux` and printed "??"; they are the kernel's bibrefs now, as
//!   natbib's (natbib_sty.rs), and in `alf` mode `\citeauthor` too (in `num` mode the
//!   package's own, :1343-1344, builds on them). `\citetext` (a whole reference)
//!   still prints "??": no bibref show mode prints a whole entry yet.
//!
//! 57cm.1 (review): a shipped `.bbl` carries each entry's author and year in
//! `\abntrefinfo` (abntex2-num.bst:1375), which now tags the bibitem, so a bibref shows them
//! there too; in `alf` mode `\citen`/`\citenum` are the textual form (:722-723; `num` mode has
//! neither); `alf` citations of several works are separated by ";" (`\ABCIcitecolondefault`, :453). A
//! `\bibliographystyle` inside a group, or before the package, loses to the recorded
//! default (BibTeX keeps the first `\bibstyle` line).
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!(r"\let\lx@abnt@orig@cite\cite\let\lx@abnt@kernel@bibliographystyle\bibliographystyle");
  InputDefinitions!("abntex2cite", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true);
  // `\bibliography` itself is the kernel's, locked against the package's
  // `\@input{\jobname.bbl}` redefinition since batch 56cx (sect11.rs).
  RawTeX!(r"\let\cite\lx@abnt@orig@cite\let\citeonline\lx@abnt@orig@cite");
  // The style reaches the bibliography: the kernel's `\bibliographystyle`, then the
  // package's flag (so its end-of-document default still yields to a user choice); the
  // package's default recorded now, which a later `\bibliographystyle` overrides.
  RawTeX!(r"\def\bibliographystyle#1{\lx@abnt@kernel@bibliographystyle{#1}\setboolean{ABCIbibtexstyleused}{true}}\lx@abnt@kernel@bibliographystyle{abntex2-\AbntCitetype}");
  // abntex2's fields, printed as the `.bst`s print them: credits, notes, the part cited,
  // illustrations, dimensions, the reprint's text — notes; the access date — a date.
  RawTeX!(concat!(
    r"\@namedef{bib@field@default@furtherresp}{\bib@@field{ltx:bib-note}[role=furtherresp]}",
    r"\@namedef{bib@field@default@abnt-note}{\bib@@field{ltx:bib-note}[role=abnt-note]}",
    r"\@namedef{bib@field@default@section}{\bib@@field{ltx:bib-note}[role=section]}",
    // an empty `illustrated` is "il." (abntex2-num.bst:597-605)
    r"\@namedef{bib@field@default@illustrated}#1{\bib@@field{ltx:bib-note}[role=illustrated]",
    r"{\if\relax\detokenize{#1}\relax il.\else#1\fi}}",
    r"\@namedef{bib@field@default@dimensions}{\bib@@field{ltx:bib-note}[role=dimensions]}",
    r"\@namedef{bib@field@default@reprinted-text}{\bib@@field{ltx:bib-note}[role=reprinted-text]}",
    r"\@namedef{bib@field@default@urlaccessdate}{\bib@@field{ltx:bib-date}[role=accessed]}"));
  // The citation forms that read BibTeX's `.aux` values are the kernel's bibrefs.
  DefMacro!("\\citeyear Semiverbatim", "\\@@cite[citeyear]{\\@@bibref{Year}{#1}{}{}}");
  DefMacro!("\\citeauthoronline Semiverbatim",
    "{\\lx@abnt@citesep\\@@cite[citeauthor]{\\@@bibref{Authors}{#1}{}{}}}");
  // abntex2-num.bst:1375 writes `\abntrefinfo{EXPL}{IMPL}{YEAR}` after every `\bibitem` of a
  // `.bbl`: the author as a textual citation names it, as a parenthetical one does, and the year
  // (abntex2cite.sty:557-575 records them in the `.aux` under `\abntnextkey`, which the package's
  // own `\bibitem` sets — undefined here). The author and year become the bibitem's
  // `authors`/`year` tags, as natbib's `\citeauthoryear` label does (`\NAT@@wrout`), so a
  // bibref shows them: without, `\citeyear` showed the refnum ("em 2, 2."). The parenthetical
  // (upper-case) form is presentation. A bibitem that already carries them keeps them: the
  // `\abntrefinfo` after `\hiddenbibitem` (abntex2-num.bst:1379-1390, an `@hidden` entry, which
  // opens no bibitem) would otherwise tag the entry before it (SYNC_STATUS residual). No corpus
  // witness yet (TeX Live ships the manuals' `.bib`, not a `.bbl`): the fixture's bibtex-made `.bbl`;
  // guard `abntex2cite_bbl_carries_author_and_year`.
  DefConstructor!("\\abntrefinfo{}{}{}", sub[document, args, _props] {
    let here = document.get_node().clone();
    if let Some(item) = document.findnode("ancestor-or-self::ltx:bibitem", Some(&here))
      && let Some(tags) = document.findnode("ltx:tags", Some(&item))
      && document.findnode("ltx:tag[@role='authors']", Some(&tags)).is_none()
    {
      document.set_node(&tags);
      let mut inserted = Ok(());
      for (role, arg) in [("authors", &args[0]), ("year", &args[2])] {
        if let Some(arg) = arg.as_ref() && inserted.is_ok() {
          inserted = document
            .insert_element("ltx:tag", vec![arg], Some(string_map!("role" => role)))
            .map(|_| ());
        }
      }
      document.set_node(&here);
      inserted?;
    }
  });
  // The author-date system (`alf`, NBR 10520): `\cite` is parenthetical, "(FARIA, 1994, p.
  // 225)", and `\citeonline` textual, "Faria (1994, p. 225)" — the kernel's refnum form
  // printed "[Faria (1994)]" for both; `\citeauthor` is the author alone. Several works in one
  // `\cite` are separated by ";" (`\ABCIcitecolondefault`, :453, :637), in one `\citeonline` by
  // "," (:696); an author list by ";" (:995-999, :1037-1043; a repeated author is printed again
  // here, SYNC_STATUS). Chosen by the package's own mode test (abntex2cite.sty:600).
  DefPrimitive!("\\lx@abnt@citesep", {
    assign_value("CITE_SEPARATOR", Stored::Token(T_OTHER!(";")), None);
  });
  DefMacro!("\\lx@abnt@alf@cite[] Semiverbatim", sub[(post, keys)] {
    let phrase = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(vec![T_OTHER!(","), T_SPACE!()])]);
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("AuthorsPhrase1Year")), keys, phrase, Tokens!()]);
    let mut body = Tokenize!("(").unlist();
    body.extend(bibref.unlist());
    if let Some(post) = post.filter(|p| !p.is_empty()) {
      body.extend(Tokenize!(",").unlist());
      body.push(T_SPACE!());
      body.extend(post.unlist());
    }
    body.extend(Tokenize!(")").unlist());
    let mut out = vec![T_BEGIN!(), T_CS!("\\lx@abnt@citesep")];
    out.extend(Invocation!(T_CS!("\\@@cite"), vec![Tokens::new(Explode!("citep")), Tokens::new(body)]).unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  DefMacro!("\\lx@abnt@alf@citeonline[] Semiverbatim", sub[(post, keys)] {
    let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokenize!("(")]);
    let mut close = Vec::new();
    if let Some(post) = post.filter(|p| !p.is_empty()) {
      close.extend(Tokenize!(",").unlist());
      close.push(T_SPACE!());
      close.extend(post.unlist());
    }
    close.extend(Tokenize!(")").unlist());
    let phrase2 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(close)]);
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Authors Phrase1YearPhrase2")), keys, phrase1, phrase2]);
    Ok(Invocation!(T_CS!("\\@@cite"), vec![Tokens::new(Explode!("citet")), bibref]))
  });
  DefMacro!("\\lx@abnt@alf@citeauthor Semiverbatim",
    "{\\lx@abnt@citesep\\@@cite[citeauthor]{\\@@bibref{Authors}{#1}{}{}}}");
  // `\citen`/`\citenum` are the alf branch's aliases of its textual `\citeonline` (:722-723; num
  // mode has neither — pdflatex stops there).
  RawTeX!(r"\ifx\AbntCitetype\AbntCitetypeALF
    \let\cite\lx@abnt@alf@cite \let\citeonline\lx@abnt@alf@citeonline
    \let\citeauthor\lx@abnt@alf@citeauthor \let\citen\citeonline\let\citenum\citeonline\fi");
});
