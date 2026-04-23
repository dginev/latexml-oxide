use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl ar5iv-bindings/biblatex.sty.ltxml is 803 lines with 16 Perl
  // closures driving a custom bibliography-rebuilder. Rust ports the
  // "shallow" half that's a mechanical 1-to-1 translation:
  //
  //   * 21 cite-variant DefMacros (Perl L37-56) — each expands to \cite
  //     with the shape `[pre][post]{key}` mapped through, locked.
  //   * ~60 passthrough DefMacros (Perl L64-107) for biblatex spacing,
  //     punctuation, and name-delimiter helpers.
  //   * 1 trailing RawTeX block (Perl L736-801) with ~60 \newbool /
  //     \newtoggle declarations.
  //   * RequirePackage deps (hyperref, ifthen, etoolbox, babel_support).
  //   * Warn!("missing_file", …) per Perl L14-15.
  //
  // DEFERRED (deep closures, won't survive a single-cycle port):
  //   * biblatex_as_thebibliography sub — walks the entry list at
  //     \enddatalist time and emits a rebuilt \thebibliography block.
  //   * \entry / \endentry closures — parse author/title/journal/doi/
  //     url/eprint fields and assign labels, with unique-label dedup.
  //   * \name closure — dispatches on 3-arg vs 4-arg \name{type}{count}
  //     {...}. Documents that use \entry / \name still type-check but
  //     their bibliographies won't render (entries become raw markup
  //     or silent empties). Filed as a follow-up cycle.

  Warn!("missing_file", "biblatex.sty",
    "biblatex.sty is only minimally stubbed and will not be interpreted raw.");

  RequirePackage!("hyperref");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("babel_support");

  // Perl L37-56: alt cite-style commands — all locked, map to plain \cite.
  DefMacro!("\\parencite", "\\cite", locked => true);
  DefMacro!("\\Parencite", "\\cite", locked => true);
  DefMacro!("\\Cite", "\\cite", locked => true);
  DefMacro!("\\citet OptionalMatch:* [][] Semiverbatim",   "\\cite[#2 ]{#4}", locked => true);
  DefMacro!("\\citep OptionalMatch:* [][] Semiverbatim",   "\\cite[#2]{#4}",  locked => true);
  DefMacro!("\\citealt OptionalMatch:* [][] Semiverbatim", "\\cite[#2]{#4}",  locked => true);
  DefMacro!("\\citealp OptionalMatch:* [][] Semiverbatim", "\\cite[#2]{#4}",  locked => true);
  DefMacro!("\\citenum", "\\cite", locked => true);
  DefMacro!("\\citem", "\\cite", locked => true);
  DefMacro!("\\autocite OptionalMatch:* [][]{}", "\\cite[#2]{#4}", locked => true);
  DefMacro!("\\Autocite OptionalMatch:* [][]{}", "\\cite[#2]{#4}", locked => true);
  DefMacro!("\\fullcite", "\\cite", locked => true);
  DefMacro!("\\footcite", "\\cite", locked => true);
  DefMacro!("\\footcitetext", "\\cite", locked => true);
  DefMacro!("\\smartcite", "\\cite", locked => true);
  DefMacro!("\\textcite", "\\cite", locked => true);
  DefMacro!("\\Textcite", "\\cite", locked => true);
  DefMacro!("\\supercite", "\\cite", locked => true);
  DefMacro!("\\citeauthor", "\\cite", locked => true);
  DefMacro!("\\citetitle", "\\cite", locked => true);

  // Perl L64-71, L85-99: passthroughs and helpers.
  DefMacro!("\\unspace", "\\relax");
  DefMacro!("\\blx@imc@resetpunctfont", "\\relax");
  DefMacro!("\\blx@postpunct", "\\@empty");
  DefRegister!("\\c@highnamepenalty" => Number(0));
  DefMacro!("\\addslash", "/\\hskip\\z@skip");
  DefMacro!("\\adddot", ".");
  DefMacro!("\\addcomma", ",");
  DefMacro!("\\autocap{}", "#1");
  DefMacro!("\\addspace",        "\\space");
  DefMacro!("\\addnbspace",      "\\space");
  DefMacro!("\\addthinspace",    "\\space");
  DefMacro!("\\addnbthinspace",  "\\space");
  DefMacro!("\\addlowpenspace",  "\\space");
  DefMacro!("\\addhighpenspace", "\\space");
  DefMacro!("\\addlpthinspace",  "\\space");
  DefMacro!("\\addhpthinspace",  "\\space");
  DefMacro!("\\addabbrvspace",   "\\space");
  DefMacro!("\\addabthinspace",  "\\space");
  DefMacro!("\\adddotspace",     "\\unspace\\adddot\\space");
  DefMacro!("\\noligature",   "\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphen",       "\\nobreak-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\nbhyphen",     "\\nobreak\\mbox{-}\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphenate",    "\\nobreak\\-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\allowhyphens", "\\nobreak\\hskip\\z@skip");
  DefMacro!("\\bibinitperiod",      "\\adddot");
  DefMacro!("\\bibinithyphendelim", ".\\mbox{-}");
  DefMacro!("\\bibnamedelima",      "\\addhighpenspace");
  DefMacro!("\\bibnamedelimb",      "\\addlowpenspace");
  DefMacro!("\\bibnamedelimc",      "\\addhighpenspace");
  DefMacro!("\\bibnamedelimd",      "\\addlowpenspace");
  DefMacro!("\\bibnamedelimi",      "\\addnbspace");

  // Perl L101-125: list/datalist/entry stubs — the Perl closures that
  // drive the bibliography-rebuild path. Stubbed to empty so documents
  // compile; deferred real port is the biggest remaining crckapb-style
  // complexity pocket.
  DefMacro!("\\datalist[]{}", "", locked => true);
  DefMacro!("\\sortlist[]{}", "", locked => true);
  DefMacro!("\\lossort", "", locked => true);
  DefMacro!("\\refsection{}", "", locked => true);
  DefMacro!("\\enddatalist", "", locked => true);
  DefMacro!("\\endsortlist", "", locked => true);
  DefMacro!("\\endlossort", "", locked => true);
  DefMacro!("\\endrefsection", "", locked => true);
  DefMacro!("\\entry{}{}{}", "", locked => true);
  DefMacro!("\\endentry", "", locked => true);

  // Perl L265-268: BiblatexAuthor keyvals.
  DefKeyVal!("BiblatexAuthor", "given",   "");
  DefKeyVal!("BiblatexAuthor", "giveni",  "");
  DefKeyVal!("BiblatexAuthor", "family",  "");
  DefKeyVal!("BiblatexAuthor", "familyi", "");

  // Perl L270+ \name / \field / \list / etc. — stubs.
  // DP-audit kind flip on `\field` / `\strng` (Perl DefPrimitive → Rust
  // DefMacro) is observational: Perl bodies stash `{field → value}` into
  // the `biblatex_entry` state hash, which is consumed at L134 to emit
  // `<ltx:bibitem>`. Rust doesn't port the entry-assembly pipeline
  // (bib post-processing handled by bibtex path), so the stub-empty
  // DefMacro is semantically equivalent to a no-op DefPrimitive —
  // WISDOM #44 (gullet-sub↔stomach-imperative); elevate to DefPrimitive
  // only if a downstream consumer of `biblatex_entry` is ported.
  DefMacro!("\\name{}{}{}", "", locked => true);
  DefMacro!("\\field{}{}", "", locked => true);
  DefMacro!("\\list{}{}{}", "", locked => true);
  DefMacro!("\\strng{}{}", "", locked => true);
  // Perl ar5iv-bindings/biblatex.sty.ltxml L641-645: define `\blx@bbl@booltrue`
  // and `\blx@bbl@boolfalse` as internal no-ops, then AtBeginDocument only
  // let `\true` / `\false` to them if the caller hasn't already defined
  // those names (document 1811.01740 conflicts with unconditional binding).
  // Prior Rust port unconditionally bound `\true` / `\false` at load time,
  // clobbering any preamble-level redefinition from another package.
  DefMacro!("\\blx@bbl@booltrue{}",  "\\relax", locked => true);
  DefMacro!("\\blx@bbl@boolfalse{}", "\\relax", locked => true);
  RawTeX!(r"\AtBeginDocument{\@ifundefined{true}{\let\true\blx@bbl@booltrue}{}\@ifundefined{false}{\let\false\blx@bbl@boolfalse}{}}");
  DefMacro!("\\keyw{}", "", locked => true);
  DefMacro!("\\range{}{}", "", locked => true);
  DefMacro!("\\lbibitem[]{}{}", "", locked => true);

  // Perl L736-801: trailing RawTeX with ~60 \newbool / \newtoggle
  // declarations that biblatex users may query via \ifbool etc.
  RawTeX!(r#"
\newbool{refcontextdefaults}
\booltrue{refcontextdefaults}%
\newbool{sourcemap}
\newbool{citetracker}
\newbool{pagetracker}
\newbool{autocite}
\newbool{autopunct}
\newbool{parentracker}
\newbool{indexing}
\newbool{loadfiles}
\newbool{backrefsetstyle}
\newbool{hyperref}
\newbool{backref}
\newbool{dashed}
\newbool{abbreviate}
\newbool{defernumbers}
\newbool{firstinits}
\newbool{sortupper}
\newbool{sortlos}
\newbool{sortcites}
\newbool{natbib}
\newbool{refsection}
\newbool{refsegment}
\newbool{mcite}
\newbool{openbib}
\newbool{punctfont}

\newtoggle{blx@citation}
\newtoggle{blx@bibentry}
\newtoggle{blx@bib}
\newtoggle{blx@biblist}
\newtoggle{blx@biblistorig}
\newtoggle{blx@skipbib}
\newtoggle{blx@skipbiblist}
\newtoggle{blx@skiplab}
\newtoggle{blx@citation}
\newtoggle{blx@volcite}
\newtoggle{blx@bibliography}
\newtoggle{blx@citeindex}
\newtoggle{blx@bibindex}
\newtoggle{blx@localnumber}
\newtoggle{blx@refcontext}
\newtoggle{blx@noroman}
\newtoggle{blx@nohashothers}
\newtoggle{blx@nosortothers}
\newtoggle{blx@singletitle}
\newtoggle{blx@uniquebaretitle}
\newtoggle{blx@uniqueprimaryauthor}
\newtoggle{blx@uniquetitle}
\newtoggle{blx@uniquework}
"#);
});
