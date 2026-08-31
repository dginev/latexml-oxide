use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "memoir.cls",
    "memoir.cls is only minimally stubbed and will not be interpreted raw."
  );
  // memoir is book-derived: base on book (chapter counter, \frontmatter
  // family, \if@openright/\if@mainmatter contracts) instead of OmniBus,
  // whose article base left `chapter` undefined for every memoir doc
  // ("Counter 'chapter' was not defined" in 6/10 sweep-12 witnesses,
  // biblatex-oxref + dlfltxb families).
  LoadClass!("book");
  RequirePackage!("iftex");
  RequirePackage!("array");
  RequirePackage!("dcolumn");
  RequirePackage!("tabularx");
  RequirePackage!("textcase");
  // These are originally \EmulatedPackage directives
  RequirePackage!("appendix");
  RequirePackage!("booktabs");
  RequirePackage!("changepage");
  RequirePackage!("chngcntr");
  RequirePackage!("chngpage");
  RequirePackage!("crop");
  RequirePackage!("enumerate");
  RequirePackage!("epigraph");
  RequirePackage!("makeidx");
  RequirePackage!("needspace");
  RequirePackage!("parskip");
  RequirePackage!("setspace");
  RequirePackage!("titling");
  RequirePackage!("tocbibind");
  RequirePackage!("verbatim");

  // memoir page-geometry preamble idiom (memman.pdf §2: every memoir doc
  // sets its type block with these before \\checkandfixthelayout). STUB
  // JUSTIFICATION (policy 2026-08-31 — stubs only for clearly out-of-scope
  // features): these compute the printed page's margins/type block, a
  // paper-geometry concern with no analogue in reflowable XML/HTML output;
  // they carry no document content whatsoever. Undefined they errored across
  // the biblatex-oxref doc family (perfect-kernel sweep 2026-08-31).
  def_macro_noop("\\setlrmarginsandblock{}{}{}")?;
  def_macro_noop("\\setulmarginsandblock{}{}{}")?;
  def_macro_noop("\\checkandfixthelayout []")?;
  def_macro_noop("\\setheadfoot{}{}")?;
  def_macro_noop("\\setheaderspaces{}{}{}")?;

  // Print-geometry / page-style surface (memoir.cls line refs per entry) —
  // same stub justification as above; sweep-12 witnesses biblatex-oxref
  // oxalph/oxnotes/oxnum/oxyear-doc, dlfltxb family, willowtreebook.
  def_macro_noop("\\setlxvchars[]")?; // L711
  def_macro_noop("\\setxlvchars[]")?; // L717
  def_macro_noop("\\setlrmargins{}{}{}")?; // L891
  def_macro_noop("\\setulmargins{}{}{}")?; // L912
  def_macro_noop("\\settypeblocksize{}{}{}")?; // L879
  def_macro_noop("\\setmarginnotes{}{}{}")?; // L940
  def_macro_noop("\\setpnumwidth{}")?; // L7136
  def_macro_noop("\\setfootnoterule[]{}{}{}{}")?; // L8749
  def_macro_noop("\\makeoddhead{}{}{}{}")?; // L1543
  def_macro_noop("\\makeevenhead{}{}{}{}")?; // L1537
  def_macro_noop("\\makeoddfoot{}{}{}{}")?;
  def_macro_noop("\\makeevenfoot{}{}{}{}")?;
  def_macro_noop("\\makeheadrule{}{}{}")?; // L1586
  def_macro_noop("\\makepsmarks{}{}")?; // L1647
  def_macro_noop("\\aliaspagestyle{}{}")?; // L1807
  def_macro_noop("\\nouppercaseheads")?; // L1867
  def_macro_noop("\\nonzeroparskip")?; // L2429
  def_macro_noop("\\firmlists")?; // L4645
  def_macro_noop("\\firmlist")?; // L4700
  def_macro_noop("\\setsecheadstyle{}")?; // L3816
  def_macro_noop("\\setsubsecheadstyle{}")?; // L3838
  def_macro_noop("\\setsubsubsecheadstyle{}")?; // L3863
  def_macro_noop("\\hangsecnum")?; // L3917
  def_macro_noop("\\captionnamefont{}")?; // L5905
  def_macro_noop("\\captiontitlefont{}")?; // L5908
  def_macro_noop("\\setmpjustification{}{}")?; // L5885
  def_macro_noop("\\setfloatadjustment{}{}")?; // L5800
  def_macro_noop("\\twocolumnfootnotes")?; // L9073
  def_macro_noop("\\indexintoc")?; // L7859
  def_macro_noop("\\newsubfloat{}")?; // L5802
  DefRegister!("\\normalrulethickness" => Dimension!("0.4pt")); // L1582
  DefRegister!("\\beforechapskip" => Glue::new(0)); // L3112
  DefRegister!("\\headwidth" => Dimension::new(0)); // L2166
  DefRegister!("\\lxvchars" => Dimension::new(0)); // L63
  DefRegister!("\\xlvchars" => Dimension::new(0));
  RawTeX!(r"\newif\ifreversesidepar \newif\ifdonemaincaption");

  // \setsecnumdepth/\maxsecnumdepth/\maxtocdepth — memoir.cls L7742-7754:
  // name→depth dispatch driving the REAL secnumdepth/tocdepth counters
  // (numbering suppression is semantic — latex_constructs.rs consults
  // secnumdepth). memoir's own default is section (=1).
  RawTeX!(
    r"\newcounter{maxsecnumdepth}
\@namedef{mem@clcnt@none}{-10}\@namedef{mem@clcnt@book}{-2}\@namedef{mem@clcnt@part}{-1}
\@namedef{mem@clcnt@chapter}{0}\@namedef{mem@clcnt@section}{1}\@namedef{mem@clcnt@subsection}{2}
\@namedef{mem@clcnt@subsubsection}{3}\@namedef{mem@clcnt@paragraph}{4}\@namedef{mem@clcnt@subparagraph}{5}
\@namedef{mem@clcnt@all}{50}
\newcommand*\mem@setclcnt[2]{\setcounter{#2}{\@nameuse{mem@clcnt@#1}}}
\newcommand*\setsecnumdepth[1]{\mem@setclcnt{#1}{secnumdepth}\mem@setclcnt{#1}{maxsecnumdepth}}
\newcommand*\maxsecnumdepth[1]{\mem@setclcnt{#1}{maxsecnumdepth}}
\newcommand*\maxtocdepth[1]{\mem@setclcnt{#1}{tocdepth}}
\setsecnumdepth{section}"
  );

  // Chapter styles (memoir.cls L3176-3177): \makechapterstyle stores a
  // body of pure font/spacing hooks; \chapterstyle activates one. The
  // hooks carry no content — store-and-drop, but keep the \@namedef
  // shape so \chapterstyle of an undefined name stays quiet.
  def_macro_noop("\\makechapterstyle{}{}")?;
  def_macro_noop("\\chapterstyle{}")?;
  // \hangfrom{arg} (L4534) and \chapterprecis{arg} (L7488) TYPESET their
  // argument — identity/paragraph, never noop (content!).
  def_macro_identity("\\hangfrom{}")?;
  DefMacro!("\\chapterprecis{}", "\\par #1\\par");
  DefMacro!("\\sidepar[]{}", "\\marginpar{#2}"); // L8466 margin note

  // Output streams (memoir.cls L10965-11063) are CONTENT-BEARING: docs
  // write body fragments to \jobname.<ext> and \input them back
  // (dlfltxbmarkup-showkeys routes its whole body through \jobname.keys;
  // willowtreebook collects answers). Delegate to REAL TeX write streams
  // so the round-trip works; \writeverbatim captures until its end marker
  // and writes the detokenized text (one line — re-input retokenizes it).
  RawTeX!(
    r"\newcommand*\newoutputstream[1]{\expandafter\newwrite\csname stream@#1\endcsname\@namedef{streamopen@#1}{0}}
\newcommand*\openoutputfile[2]{\immediate\openout\csname stream@#2\endcsname #1\relax\@namedef{streamopen@#2}{1}}
\newcommand*\closeoutputstream[1]{\immediate\closeout\csname stream@#1\endcsname\@namedef{streamopen@#1}{0}}
\newcommand\addtostream[2]{\immediate\write\csname stream@#1\endcsname{#2}}
\newcommand\IfStreamOpen[3]{\ifnum0\@nameuse{streamopen@#1}=1 #2\else#3\fi}"
  );
  DefMacro!(
    "\\writeverbatim{} Until:\\endwriteverbatim",
    "\\addtostream{#1}{\\detokenize{#2}}"
  );

  // External-file glossary plumbing — .gls round-trip out of scope here.
  def_macro_noop("\\printglossary[]")?;
  def_macro_noop("\\changeglossnum{}")?;
  def_macro_noop("\\changeglossnumformat{}")?;
});
