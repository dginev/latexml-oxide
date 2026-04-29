use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Strict-Perl translation of ar5iv-bindings/biblatex.sty.ltxml
  // (803 lines). Most macro definitions, conditionals, registers,
  // and the trailing RawTeX toggle block are now line-by-line
  // mirrors. The deep-closure bibliography rebuilder remains
  // DEFERRED (Perl L110-263 / L270-340 / L367-397) — those are
  // stubbed as no-op `DefMacro` so documents compile but the
  // bibliography body is not assembled.
  //
  // Audit cycle 2: caught Rust-only bugs vs Perl source
  //   * duplicate `\newtoggle{blx@citation}` (etoolbox toggle redef)
  //   * missing 38 of 60 toggles in trailing RawTeX
  //   * missing `\addbibresource` / `\printbibliography` /
  //     `Let \bibliography → \addbibresource` chain
  //   * 60+ DefMacro/DefRegister/DefConditional declarations missing

  // Perl L14-15: Warn that biblatex.sty is only minimally stubbed.
  Warn!("missing_file", "biblatex.sty",
    "biblatex.sty is only minimally stubbed and will not be interpreted raw.");

  // Perl L19-22: option processing
  DefKeyVal!("biblatex", "maxbibnames", "Number", "4");
  // Perl `DeclareOption(undef, sub { })` — ignore unknown options.
  DeclareOption!(None, {});
  ProcessOptions!();

  // Perl L24-30: dependencies. (`#RequirePackage('natbib')` etc commented out in Perl.)
  RequirePackage!("hyperref");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("babel_support");

  // Perl L37-56: cite variants
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

  // Perl L64-67: passthroughs
  DefMacro!("\\unspace", "\\relax");
  DefMacro!("\\blx@imc@resetpunctfont", "\\relax");
  DefMacro!("\\blx@postpunct", "\\@empty");
  DefRegister!("\\c@highnamepenalty" => Number(0));

  // Perl L69-72
  DefMacro!("\\addslash", "/\\hskip\\z@skip");
  DefMacro!("\\adddot", ".");
  DefMacro!("\\addcomma", ",");
  DefMacro!("\\autocap{}", "#1");

  // Perl L75-85
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

  // Perl L87-91
  DefMacro!("\\noligature",   "\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphen",       "\\nobreak-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\nbhyphen",     "\\nobreak\\mbox{-}\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphenate",    "\\nobreak\\-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\allowhyphens", "\\nobreak\\hskip\\z@skip");

  // Perl L93-99
  DefMacro!("\\bibinitperiod",      "\\adddot");
  DefMacro!("\\bibinithyphendelim", ".\\mbox{-}");
  DefMacro!("\\bibnamedelima",      "\\addhighpenspace");
  DefMacro!("\\bibnamedelimb",      "\\addlowpenspace");
  DefMacro!("\\bibnamedelimc",      "\\addhighpenspace");
  DefMacro!("\\bibnamedelimd",      "\\addlowpenspace");
  DefMacro!("\\bibnamedelimi",      "\\addnbspace");

  // Perl L101-106: \datalist / \sortlist set the `biblatex_with_keyvals`
  // flag globally — Perl's `\name` closure (Cycle 9) reads it to choose
  // 3-arg vs 4-arg / keyval-vs-positional dispatch.
  DefMacro!("\\datalist[]{}", sub[_args] {
    state::assign_value("biblatex_with_keyvals", Stored::from(1),
      Some(Scope::Global));
    Ok(Tokens::new(vec![]))
  });
  DefMacro!("\\sortlist[]{}", sub[_args] {
    state::assign_value("biblatex_with_keyvals", Stored::from(1),
      Some(Scope::Global));
    Ok(Tokens::new(vec![]))
  });
  // Perl L107-108: \lossort / \refsection — empty stubs.
  DefMacro!("\\lossort", "", locked => true);
  DefMacro!("\\refsection{}", "", locked => true);
  // Perl L122-125: \enddatalist / \endsortlist / \endlossort / \endrefsection
  // → biblatex_as_thebibliography rebuilder. DEFERRED (Cycle 7).
  DefMacro!("\\enddatalist", "", locked => true);
  DefMacro!("\\endsortlist", "", locked => true);
  DefMacro!("\\endlossort", "", locked => true);
  DefMacro!("\\endrefsection", "", locked => true);
  // Perl L127-263: \entry / \endentry deep closures. DEFERRED (Cycle 8).
  DefMacro!("\\entry{}{}{}", "", locked => true);
  DefMacro!("\\endentry", "", locked => true);

  // Perl L265-268: BiblatexAuthor keyvals
  DefKeyVal!("BiblatexAuthor", "given",   "");
  DefKeyVal!("BiblatexAuthor", "giveni",  "");
  DefKeyVal!("BiblatexAuthor", "family",  "");
  DefKeyVal!("BiblatexAuthor", "familyi", "");

  // Perl L270-346: \name (3-arg vs 4-arg dispatch + author parser),
  // \list (entry-field assigner) — DEFERRED, stubbed empty.
  DefMacro!("\\name{}{}{}", "", locked => true);
  DefMacro!("\\list{}{}{}", "", locked => true);
  // Perl L355-363: \field, \strng — Perl uses DefPrimitive to record into
  // the `biblatex_entry` hash. DEFERRED at the entry-pipeline level; stubbed
  // as no-op DefMacro (consumer is itself stubbed).
  DefMacro!("\\field{}{}", "", locked => true);
  DefMacro!("\\strng{}{}", "", locked => true);

  // Perl L348-354
  DefMacro!("\\AtEveryBibitem{}",   "");
  DefMacro!("\\AtEveryCitekey{}",   "");
  DefMacro!("\\keyw{}",             "");
  DefMacro!("\\bibinitdelim",       "");
  // Note: \bibinithyphendelim re-defined here as just "-" per Perl L352
  // (overrides the L94 definition; Perl runs them in order).
  DefMacro!("\\bibinithyphendelim", "-");
  DefMacro!("\\bibrangedash", "\u{2013}");
  DefMacro!("\\bibnamedelimi", " ");

  // Perl L364
  DefMacro!("\\range{}{}", "");

  // Perl L367-369: \preamble{...} stashes the arg into biblatex_preamble
  // for the rebuilder (Cycle 7) and *also* re-emits the arg (Perl returns
  // $_[1]) so the preamble is digested in the current context too.
  DefMacro!("\\preamble{}", sub[(arg)] {
    state::assign_value("biblatex_preamble",
      Stored::Tokens(arg.clone()), Some(Scope::Global));
    Ok(arg.clone())
  });

  // Perl L371-397: \verb / \biblatex@verb / \biblatex@endverb closures.
  // DEFERRED — stubbed empty so \verb-based bib fields don't crash.
  DefMacro!("\\biblatex@verb{}", "", locked => true);
  DefMacro!("\\biblatex@endverb", "", locked => true);

  // Perl L400-408: \addbibresource{file,...} pushes onto biblatex_resources.
  // Then `\biblatex@saved@bibliography` is bound to whatever `\bibliography`
  // means at this point (classic LaTeX bibtex), and `\bibliography` is
  // re-let to `\addbibresource` so any classic `\bibliography{...}`
  // invocation in a biblatex doc just records resources.
  // see arXiv:1502.02314 for a paper that left in classic \bibliography
  // alongside biblatex; both forms must end up populating the resource list.
  DefPrimitive!("\\addbibresource{}", sub[(file_list_arg)] {
    // Perl: split(/\s*,\s*/, ToString($_[1])) — split on commas and
    // strip surrounding whitespace.
    let raw = file_list_arg.to_string();
    for part in raw.split(',') {
      let file = part.trim();
      if !file.is_empty() {
        push_value("biblatex_resources", Stored::String(arena::pin(file)))?;
      }
    }
  });
  Let!("\\biblatex@saved@bibliography", "\\bibliography");
  Let!("\\bibliography",                "\\addbibresource");

  // Perl L410-418: \printbibliography → \biblatex@printbibliography (which
  // emits the saved \biblatex@saved@bibliography call). DEFERRED rebuilder;
  // stubbed empty so the macro is defined.
  DefMacro!("\\printbibliography[]", "", locked => true);

  // Perl L420-424
  DefMacro!("\\warn{}", "");
  DefMacro!("\\xref{}", "");
  DefMacro!("\\fakeset{}", "");

  // Perl L429-434: language API (no-ops)
  DefMacro!("\\DeclareLanguageMapping{}{}", "");
  DefMacro!("\\DeclareLanguageMappingSuffix{}", "");
  DefMacro!("\\DefineHyphenationExceptions{}{}", "");
  DefMacro!("\\DefineBibliographyExtras{}{}", "");
  DefMacro!("\\UndefineBibliographyExtras{}{}", "");
  DefMacro!("\\DefineBibliographyStrings{}{}", "");

  // Perl L436-438
  DefMacro!("\\DeclareNameFormat OptionalMatch:* []{}{}",  "");
  DefMacro!("\\DeclareListFormat OptionalMatch:* []{}{}",  "");
  DefMacro!("\\DeclareFieldFormat OptionalMatch:* []{}{}", "");

  // Perl L440-458
  DefMacro!("\\DeclareNameInputHandler{}{}", "");
  DefMacro!("\\DeclareListInputHandler{}{}", "");
  DefMacro!("\\DeclareFieldInputHandler{}{}", "");
  DefMacro!("\\DeclareSortingScheme[]{}", "");
  DefMacro!("\\DeclareSortingTemplate[]{}", "");
  DefMacro!("\\DeclareSortingNamekeyScheme[]{}", "");
  DefMacro!("\\namepart[]{}", "");
  DefMacro!("\\DeclareLabelalphaNameTemplate[]{}", "");
  DefMacro!("\\DeclareNameAlias{}{}", "");
  DefMacro!("\\DeclareIndexNameAlias{}{}", "");
  DefMacro!("\\DeclareListAlias{}{}", "");
  DefMacro!("\\DeclareIndexListAlias{}{}", "");
  DefMacro!("\\DeclareFieldAlias{}{}", "");
  DefMacro!("\\DeclareIndexFieldAlias{}{}", "");
  DefMacro!("\\DeclareNameWrapperAlias{}{}", "");
  DefMacro!("\\DeclareListWrapperAlias{}{}", "");
  DefMacro!("\\DeclareDelimcontextAlias{}{}", "");
  DefMacro!("\\UndeclareDelimcontextAlias{}", "");
  DefMacro!("\\DeclareCiteCommand OptionalMatch:* {}[]{}{}{}{}", "");

  // Perl L460-481
  DefMacro!("\\DeclareBibliographyExtras{}", "");
  DefMacro!("\\DeclareBibliographyStrings{}", "");
  DefMacro!("\\DeclareBibliographyDriver{}{}", "");
  DefMacro!("\\DeclareHyphenationExceptions{}", "");
  DefMacro!("\\InheritBibliographyExtras{}", "");
  DefMacro!("\\InheritBibliographyStrings{}", "");
  DefMacro!("\\UndeclareBibliographyExtras{}", "");
  DefMacro!("\\NewCount", "\\newcount");
  DefMacro!("\\ExecuteBibliographyOptions[]{}", "");
  DefMacro!("\\AtBeginBibliography{}", "");
  DefMacro!("\\AtEveryEntrykey{}{}{}", "");
  DefMacro!("\\UseBibitemHook", "");
  DefMacro!("\\UseUsedriverHook", "");
  DefMacro!("\\UseEveryCiteHook", "");
  DefMacro!("\\UseEveryCitekeyHook", "");
  DefMacro!("\\UseEveryMultiCiteHook", "");
  DefMacro!("\\UseNextCiteHook", "");
  DefMacro!("\\UseNextCitekeyHook", "");
  DefMacro!("\\UseNextMultiCiteHook", "");
  DefMacro!("\\UseVolciteHook", "");
  DefMacro!("\\DeferNextCitekeyHook", "");

  // Perl L483-491: bibmacro/heading/environment helpers
  DefMacro!("\\providebibmacro OptionalMatch:* {}[][]{}", "");
  DefMacro!("\\renewbibmacro OptionalMatch:* {}[][]{}", "");
  DefMacro!("\\newbibmacro OptionalMatch:* {}[][]{}", "");
  DefMacro!("\\restorebibmacro OptionalMatch:* {}", "");
  DefMacro!("\\savebibmacro OptionalMatch:* {}", "");
  DefMacro!("\\defbibheading OptionalMatch:* {}[]{}", "");
  DefMacro!("\\defbibenvironment OptionalMatch:* {}{}{}{}", "");
  DefMacro!("\\restorecommand OptionalMatch:* {}", "");
  DefMacro!("\\savecommand OptionalMatch:* {}", "");

  // Perl L493-500
  DefRegister!("\\labelnumberwidth" => Glue!("0pt"));
  DefRegister!("\\labelalphawidth" => Glue!("0pt"));
  DefRegister!("\\biblabelsep" => Glue!("0pt"));
  DefRegister!("\\bibnamesep" => Glue!("0pt"));
  DefRegister!("\\bibitemsep" => Glue!("0pt"));
  DefRegister!("\\bibinitsep" => Glue!("0pt"));
  DefRegister!("\\bibparsep" => Glue!("0pt"));
  DefRegister!("\\bibhang" => Glue!("0pt"));

  // Perl L553-604: 50 conditionals
  DefConditional!("\\ifandothers");
  DefConditional!("\\ifbibindex");
  DefConditional!("\\ifbibliography");
  DefConditional!("\\ifbibstring");
  DefConditional!("\\ifcapital");
  DefConditional!("\\ifcategory");
  DefConditional!("\\ifcitation");
  DefConditional!("\\ifciteibid");
  DefConditional!("\\ifciteidem");
  DefConditional!("\\ifciteindex");
  DefConditional!("\\ifciteseen");
  DefConditional!("\\ifcurrentfield");
  DefConditional!("\\ifcurrentlist");
  DefConditional!("\\ifcurrentname");
  DefConditional!("\\ifentrycategory");
  DefConditional!("\\ifentrykeyword");
  DefConditional!("\\ifentryseen");
  DefConditional!("\\ifentrytype");
  DefConditional!("\\iffieldbibstring");
  DefConditional!("\\iffieldequalcs");
  DefConditional!("\\iffieldequals");
  DefConditional!("\\iffieldequalstr");
  DefConditional!("\\iffieldint");
  DefConditional!("\\iffieldnum");
  DefConditional!("\\iffieldnums");
  DefConditional!("\\iffieldpages");
  DefConditional!("\\iffieldsequal");
  DefConditional!("\\iffieldundef");
  DefConditional!("\\iffirstinits");
  DefConditional!("\\iffirstonpage");
  DefConditional!("\\iffootnote");
  DefConditional!("\\ifhyperref");
  DefConditional!("\\ifinteger");
  DefConditional!("\\ifkeyword");
  DefConditional!("\\ifloccit");
  DefConditional!("\\ifmoreitems");
  DefConditional!("\\ifmorenames");
  DefConditional!("\\ifnameequalcs");
  DefConditional!("\\ifnameequals");
  DefConditional!("\\ifnamesequal");
  DefConditional!("\\ifnameundef");
  DefConditional!("\\ifnatbibmode");
  DefConditional!("\\ifnumeral");
  DefConditional!("\\ifnumerals");
  DefConditional!("\\ifopcit");
  DefConditional!("\\ifpages");
  DefConditional!("\\ifsamepage");
  DefConditional!("\\ifsingletitle");
  DefConditional!("\\ifuseauthor");
  DefConditional!("\\ifuseeditor");
  DefConditional!("\\ifuseprefix");
  DefConditional!("\\ifusetranslator");

  // Perl L608-610
  DefMacro!("\\key{}", "");
  // \keyw is already defined L348 (DefMacro empty, see above).
  DefMacro!("\\keyword{}", "");

  // Perl L632-635
  DefMacro!("\\ppspace", "\\addnbspace");
  DefMacro!("\\sqspace", "\\addnbspace");
  DefMacro!("\\labelalphaothers", "+");
  DefMacro!("\\sortalphaothers", "\\labelalphaothers");

  // Perl L638
  DefMacro!("\\sort[]{}", "");

  // Perl L641-645: bool stubs + AtBeginDocument-guarded \true/\false bind.
  // documents such as 1811.01740 conflict with unconditional binding.
  DefMacro!("\\blx@bbl@booltrue{}",  "\\relax", locked => true);
  DefMacro!("\\blx@bbl@boolfalse{}", "\\relax", locked => true);
  at_begin_document(TokenizeInternal!(
    r"\@ifundefined{true}{\let\true\blx@bbl@booltrue}{}\@ifundefined{false}{\let\false\blx@bbl@boolfalse}{}"
  ))?;

  // Perl L646-671: \the* counter-readouts (all empty)
  DefMacro!("\\type{}", "");
  DefMacro!("\\subtype{}", "");
  DefMacro!("\\theparenlevel", "");
  DefMacro!("\\therefsection", "");
  DefMacro!("\\therefsegment", "");
  DefMacro!("\\theuniquelist", "");
  DefMacro!("\\theuniquename", "");
  DefMacro!("\\themulticitecount", "");
  DefMacro!("\\themulticitetotal", "");
  DefMacro!("\\thelownamepenalty", "");
  DefMacro!("\\themaxextraalpha", "");
  DefMacro!("\\themaxextrayear", "");
  DefMacro!("\\themaxitems", "");
  DefMacro!("\\themaxnames", "");
  DefMacro!("\\themaxparens", "");
  DefMacro!("\\theminitems", "");
  DefMacro!("\\theminnames", "");
  DefMacro!("\\theabbrvpenalty", "");
  DefMacro!("\\thecitecount", "");
  DefMacro!("\\thecitetotal", "");
  DefMacro!("\\thehighnamepenalty", "");
  DefMacro!("\\theinstcount", "");
  DefMacro!("\\thelistcount", "");
  DefMacro!("\\theliststart", "");
  DefMacro!("\\theliststop", "");
  DefMacro!("\\thelisttotal", "");

  // Perl L673-688: print*/index*/entry* (all empty)
  DefMacro!("\\printtext[]{}", "");
  DefMacro!("\\printfield[]{}", "");
  DefMacro!("\\printlist[][]{}", "");
  DefMacro!("\\printnames[][]{}", "");
  DefMacro!("\\printtime", "");
  DefMacro!("\\printdate", "");
  DefMacro!("\\printdateextra", "");
  DefMacro!("\\printlabeldate", "");
  DefMacro!("\\printlabeldateextra", "");
  DefMacro!("\\printfile[]{}", "");
  DefMacro!("\\indexfield[]{}", "");
  DefMacro!("\\indexlist[][]{}", "");
  DefMacro!("\\indexnames[][]{}", "");
  DefMacro!("\\entrydata OptionalMatch:* {}{}", "");
  DefMacro!("\\entryset{}{}", "");
  DefMacro!("\\setunit OptionalMatch:* {}", "");

  // Perl L690-705
  DefMacro!("\\mkbibendnote{}", "");
  DefMacro!("\\mkbibendnotetext{}", "");
  DefMacro!("\\mkbibfootnote", "\\footnote");
  DefMacro!("\\mkbibfootnotetext", "\\footnotetext");
  DefMacro!("\\mkbibbrackets{}", "\\begingroup\\bibopenbracket#1\\bibclosebracket\\endgroup");
  DefMacro!("\\bibopenparen", "\\bibleftparen");
  DefMacro!("\\bibcloseparen", "\\bibrightparen");
  DefMacro!("\\bibopenbracket", "\\bibleftbracket");
  DefMacro!("\\bibclosebracket", "\\bibrightbracket");
  DefMacro!("\\bibleftparen", "\\blx@postpunct(");
  DefMacro!("\\bibrightparen", "\\blx@postpunct)\\midsentence");
  DefMacro!("\\bibleftbracket", "\\blx@postpunct[");
  DefMacro!("\\bibrightbracket", "\\blx@postpunct]\\midsentence");
  // Perl L704: redefine \blx@postpunct to \relax (overrides L66 \@empty).
  DefMacro!("\\blx@postpunct", "\\relax");
  DefMacro!("\\midsentence", "\\relax");

  // Perl L707-708
  DefMacro!("\\pagenote{}", "");
  DefMacro!("\\pagenotetext{}", "");

  // Perl L710-721
  DefMacro!("\\blx@uniquename", "false");
  DefMacro!("\\blx@uniquelist", "false");
  DefMacro!("\\blx@maxbibnames", "0");
  DefMacro!("\\blx@minbibnames", "0");
  DefMacro!("\\blx@maxcitenames", "0");
  DefMacro!("\\blx@mincitenames", "0");
  DefMacro!("\\blx@maxsortnames", "0");
  DefMacro!("\\blx@minsortnames", "0");
  DefMacro!("\\blx@maxalphanames", "0");
  DefMacro!("\\blx@minalphanames", "0");
  DefMacro!("\\blx@maxitems", "0");
  DefMacro!("\\blx@minitems", "0");

  // Perl L724-734: blx-internal counter registers
  DefRegister!("\\blx@tempcnta" => Number(0));
  DefRegister!("\\blx@tempcntb" => Number(0));
  DefRegister!("\\blx@tempcntc" => Number(0));
  DefRegister!("\\blx@maxsection" => Number(0));
  DefRegister!("\\blx@notetype" => Number(0));
  DefRegister!("\\blx@parenlevel@text" => Number(0));
  DefRegister!("\\blx@parenlevel@foot" => Number(0));
  DefRegister!("\\blx@maxsegment@0" => Number(0));
  DefRegister!("\\blx@sectionciteorder@0" => Number(0));
  DefRegister!("\\blx@entrysetcounter" => Number(0));
  DefRegister!("\\blx@biblioinstance" => Number(0));

  // Perl L736-801: trailing RawTeX with 9 \newbool + 60 \newtoggle
  // declarations. EXACT order and content from the Perl source.
  RawTeX!(r#"
\newbool{refcontextdefaults}
\booltrue{refcontextdefaults}%
\newbool{sourcemap}
\newbool{citetracker}
\newbool{pagetracker}
\newbool{backtracker}
\newbool{citerequest}
\booltrue{citerequest}
\newbool{sortcites}
\newtoggle{blx@bbldone}
\newtoggle{blx@tempa}
\newtoggle{blx@tempb}
\newtoggle{blx@runltx}
\newtoggle{blx@runbiber}
\newtoggle{blx@block}
\newtoggle{blx@unit}
\newtoggle{blx@skipentry}
\newtoggle{blx@insert}
\newtoggle{blx@lastins}
\newtoggle{blx@keepunit}
\newtoggle{blx@bibtex}
\newtoggle{blx@debug}
\newtoggle{blx@sortcase}
\newtoggle{blx@sortupper}
\newtoggle{blx@autolangbib}
\newtoggle{blx@autolangcite}
\newtoggle{blx@clearlang}
\newtoggle{blx@defernumbers}
\newtoggle{blx@omitnumbers}
\newtoggle{blx@footnote}
\newtoggle{blx@labelalpha}
\newtoggle{blx@labelnumber}
\newtoggle{blx@labeltitle}
\newtoggle{blx@labeltitleyear}
\newtoggle{blx@labeldateparts}
\newtoggle{blx@natbib}
\newtoggle{blx@mcite}
\newtoggle{blx@loadfiles}
\newtoggle{blx@sortsets}
\newtoggle{blx@crossrefsource}
\newtoggle{blx@xrefsource}
\newtoggle{blx@terseinits}
\newtoggle{blx@useprefix}
\newtoggle{blx@addset}
\newtoggle{blx@setonly}
\newtoggle{blx@dataonly}
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
