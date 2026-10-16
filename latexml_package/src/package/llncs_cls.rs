use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: llncs.cls.ltxml — Lecture Notes in Computer Science (Springer)
  // llncs.cls:25-36 (TL 2025 v2.26): the switches the numbering and running-head options set, which papers and their
  // packages test (`\if@runhead`: pscproc2.sty:19, 0905.3107, 0908.0239, 1006.0809; `\if@envcntsame`: 2303.08726,
  // 2306.07698) and `\spn@wtheorem` (below) is chosen by.
  RawTeX!(r"\let\if@envcntreset\iffalse\let\if@envcntsame\iffalse\let\if@envcntsect\iffalse\let\if@runhead\iffalse");
  DeclareOption!("envcountreset", "\\let\\if@envcntreset\\iftrue");
  DeclareOption!("envcountsame", "\\let\\if@envcntsame\\iftrue");
  DeclareOption!("envcountsect", "\\let\\if@envcntsect\\iftrue");
  DeclareOption!("runningheads", "\\let\\if@runhead\\iftrue");
  for option in [
    "citeauthoryear", "oribibl", "orivec",
  ].iter() {
    DeclareOption!(*option, None);
  }
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  // Real llncs.cls L244 sets secnumdepth to 2, so subsubsections are NOT
  // numbered. Both the Perl llncs.cls.ltxml and this port previously omitted it
  // and inherited article's default of 3, giving a plain \subsubsection a
  // printed number ("7.0.1") that the PDF never shows. Witness 2605.16562
  // ("Author contributions.").
  SetCounter!("secnumdepth", Number::new(2));

  RequirePackage!("multicol");
  // LLNCS authors routinely use \boldsymbol (amsbsy) and \mathbb / \mathfrak
  // (amssymb) in math without explicit \usepackage. The real Springer
  // llncs.cls quietly tolerates these because most authors run pdflatex
  // with amssymb/amsbsy pulled in by transitive paper deps; mirror by
  // eager-loading here. Witness 2211.16852.
  RequirePackage!("amsbsy");
  RequirePackage!("amssymb");

  //======================================================================
  // Frontmatter
  // llncs.cls:250-253: the matter commands switch the page numbering (Roman, then arabic).
  DefMacro!("\\frontmatter", "\\pagenumbering{Roman}");

  // llncs.cls:862, :915 `\def\subtitle#1{\gdef\@subtitle{#1}}`: the subtitle a paper's own title code tests
  // (2112.05140, 2208.01633 `\if!\@subtitle!`).
  DefMacro!("\\subtitle{}", "\\gdef\\@subtitle{#1}\\lx@add@subtitle{#1}");
  Let!("\\@subtitle", "\\@empty");
  // llncs.cls:870-885 `\institutename` sets the `\institute` list under the title, which papers' own title code calls
  // (2112.05140, 2208.01633 `{\small\institutename}`): the institutes are the frontmatter's affiliations here, so it
  // sets nothing (Perl's llncs.cls.ltxml:70 has it commented out).
  DefMacro!("\\institutename", "");
  // llncs.cls:561 `\let\phantomsection=\relax`, hyperref's anchor command a paper calls whether or not it loads
  // hyperref (1709.06172, 1803.04282, 1810.01542); hyperref's own replaces it.
  Let!("\\phantomsection", "\\relax");

  DefMacro!("\\emailname", "E-mail");
  DefMacro!("\\mailname",  "\\textit{Correspondence to}:");

  // Single \author, with multiple authors separated by \and  (Perl PR #2767)
  // \inst{labels} can be used within each author to identify which affiliations apply.
  // IMPROVEMENT over upstream: also split on \\ -- authors commonly use a manual
  // row break in place of \and ("A\inst{1} \\ B\inst{2}"), which upstream merges
  // into a single creator (witness 2606.19939: "Hiuyi Cheng \\ Dezhi Peng").
  DefMacro!("\\author{}",
    "\\lx@clear@creators[role=author]\\lx@splitting{\\lx@add@author@split}{\\and\\And\\\\}{#1}\\lx@author@trailing");
  // Single \institute, with multiple institutions separated by \and (Perl
  // llncs.cls.ltxml L46-48). The n-th institution is attached to the author
  // which has that n in its \inst labels.
  //
  // Perl splits on \and ONLY, and each piece is ONE affiliation
  // (\lx@llncs@affiliation -> \lx@add@affiliation, singular, no further split).
  // Three shapes occur in the wild:
  //  (a) Well-formed \and-separated institutions, each possibly with an internal
  //      "\\" break between its name and its \email/\url (witness 2605.16562).
  //      Faithful \and-split; \email inside an affiliation inherits that
  //      affiliation's label, so it attaches to the matching \inst{N} authors.
  //  (b) Single institution, no \and, "\\"-separating the name from a SHARED
  //      \email covering every author (witness 2608.11332, arXiv/html_feedback
  //      #6881). This is Perl's plain case: it must stay ONE affiliation so the
  //      trailing \email inherits its "affiliation:1" label and lands on all the
  //      \inst{1} authors. Splitting the "\\" here (the pre-fix bug) put the
  //      \email in its own "affiliation:2" contact that no author carries, so it
  //      mislanded on a single creator and left an empty affiliation stub.
  //  (c) Lazy: ONE block, no \and, hand-typed superscripts ("$^1$..\quad $^2$..")
  //      (witness 2606.19939). Upstream makes a single affiliation = the whole
  //      block attached to every \inst{1} author (duplicating it) while \inst{2}+
  //      get nothing (reproduces identically in Perl LaTeXML 0.8.8). The shared
  //      \lx@add@affiliations parser handles this: it splits on \quad/\qquad/\\
  //      and extracts each superscript as the affiliation label \inst{N} links to.
  // So: clear, then \and-split (a)+(b) UNLESS there is no \and AND superscript
  // markers are present (c) -- the only shape that needs the multi-parser.
  DefMacro!("\\institute{}",
    "\\lx@clear@frontmatter{ltx:contact}[role=affiliation]\
     \\lx@llncs@institute@dispatch{#1}");
  DefMacro!("\\lx@llncs@institute@split{}", "\\lx@splitting{\\lx@llncs@affiliation}{\\and}{#1}");
  DefMacro!("\\lx@llncs@institute@sup{}",   "\\lx@add@affiliations[labelseq={affiliation}]{#1}");
  DefMacro!("\\lx@llncs@institute@dispatch{}", sub[(body)] {
    let has_and = position_of(&body, &[T_CS!("\\and")]).is_some();
    // Same markers \lx@add@affiliations keys its superscript-label branch on.
    let has_sup = position_of(&body, &[T_SUPER!(), T_CS!("\\textsuperscript")]).is_some();
    let target = if !has_and && has_sup {
      T_CS!("\\lx@llncs@institute@sup")     // shape (c): lazy superscript-labeled block
    } else {
      T_CS!("\\lx@llncs@institute@split")   // shapes (a)+(b): \and-split, single stays one
    };
    Ok(Invocation!(target, vec![Some(body)]))
  });
  DefMacro!("\\lx@llncs@affiliation{}", "\\lx@add@affiliation[labelseq={affiliation}]{#1}");
  DefMacro!("\\inst{}",                 "\\lx@request@frontmatter@annotation[affiliation]{#1}");
  // \orcidID should be used within each author in \author
  DefMacro!("\\orcidID{}", "\\lx@add@orcid{#1}");
  // \email, \url, \mail should be within an institute in \institute!!
  // llncs.cls:897 `\def\email#1{{\tt#1}}` (2101.08426, 2501.08296)
  DefMacro!("\\email Semiverbatim", "\\lx@add@email[name={\\emailname~}]{\\lx@address@urlfont{#1}}");
  DefMacro!("\\url Semiverbatim",   "\\lx@add@url{#1}");
  DefMacro!("\\mail {}",            "\\lx@add@address[name={\\mailname~}]{#1}");
  DefMacro!("\\keywordname",        "\\textbf{Keywords}");
  DefMacro!("\\keywords{}",         "\\lx@add@keywords[name={\\keywordname:~}]{#1}");

  DefMacro!("\\ackname", "Acknowledgements");
  DefConstructor!("\\acknowledgements", "<ltx:acknowledgements name='#name'>",
    properties => sub[_args] {
      let name = Stored::from(digest_in_group(T_CS!("\\ackname"))?);
      Ok(stored_map!("name" => name))
    });
  DefMacro!("\\acknowledgement", "\\acknowledgements");
  DefConstructor!("\\endacknowledgements", "</ltx:acknowledgements>");
  DefConstructor!("\\endacknowledgement", "</ltx:acknowledgements>");
  // ltx:acknowledgements Tag (autoClose + inlist=toc) is global — set in
  // latex_constructs.rs (arXiv-fork 23771504 removed the binding-local copies).

  DefConstructor!("\\url Semiverbatim", "<ltx:ref href='#1'>#1</ltx:ref>");
  // llncs.cls:909-911 (v2.21+): the class's begin-document hook defines `\doi`, a link when
  // hyperref is loaded and the bare URL otherwise; splncs04.bst writes `\doi{…}` into the
  // .bbl (2609.04690). The class's link is `\href{URL}{\url{URL}}`, the URL linked to itself:
  // `\url` alone is that link, without an anchor inside an anchor. The binding stands in for
  // whichever llncs.cls a paper ships, so this also replaces a preamble `\doi` of a paper with an
  // older class (KNOWN_PERL_ERRORS #492).
  RawTeX!(r"\AtBeginDocument{\@ifpackageloaded{hyperref}{\def\doi#1{\url{https://doi.org/#1}}}{\def\doi#1{https://doi.org/\detokenize{#1}}}}");

  DefRegister!("\\instindent" => Dimension::new(0));
  DefRegister!("\\authrun" => Tokens!());
  DefRegister!("\\authorrunning" => Tokens!());
  DefRegister!("\\tocauthor" => Tokens!());
  DefRegister!("\\titrun" => Tokens!());
  DefRegister!("\\titlerunning" => Tokens!());
  // Perl llncs.cls.ltxml L73: `DefRegister('\toctitle' => Tokens())` — a plain
  // Tokens register, like its siblings above. Upstream #2847 (brucemiller/LaTeXML,
  // fixes #2253) removed a `{}` typo in the prototype (`'\toctitle{}'`) that had
  // made the register consume a following token: the sample llncs template
  // `\tocauthor\toctitle\maketitle` (arXiv:2112.13058) then ate `\maketitle`,
  // skipping frontmatter emission ("Can't close environment abstract"). We had
  // worked around the typo with a `def_macro_noop`; now we match the corrected
  // upstream register form — which behaves exactly like the `\tocauthor` register
  // above, so it never swallows a following token.
  DefRegister!("\\toctitle" => Tokens!());

  DefRegister!("\\tocchpnum" => Dimension::new(0));
  DefRegister!("\\tocsecnum" => Dimension!("15pt"));
  DefRegister!("\\tocsubsecnum" => Dimension!("23pt"));
  DefRegister!("\\tocsubsubsecnum" => Dimension!("27pt"));
  DefRegister!("\\tocparanum" => Dimension!("35pt"));
  DefRegister!("\\tocsubparanum" => Dimension!("43pt"));
  DefRegister!("\\tocsectotal" => Dimension::new(0));
  DefRegister!("\\tocsubsectotal" => Dimension::new(0));
  DefRegister!("\\tocsubsubsectotal" => Dimension::new(0));
  DefRegister!("\\tocparatotal" => Dimension::new(0));

  def_macro_noop("\\addcontentsmark{}{}{}")?;
  def_macro_noop("\\addcontentsmarkwop{}{}{}")?;
  def_macro_noop("\\addnumcontentsmark{}{}{}")?;
  def_macro_noop("\\addtocmark[]{}{}{}")?;

  //======================================================================
  DefMacro!("\\mainmatter", "\\pagenumbering{arabic}");

  NewCounter!("chapter", "document", idprefix => "Pt", nested => vec!["section"]);
  DefMacro!("\\thechapter", "\\arabic{chapter}");
  def_macro_noop("\\chaptermark{}")?;

  // Theorem-family \xxxname definitions. The \spnewtheorem primitive itself
  // is ported further below via define_new_theorem, and the theorem
  // environments are installed there with their per-theorem bodyfont honored
  // (capfont/title font is still ignored).
  RawTeX!(r#"\def\theoremname{Theorem}
\def\claimname{Claim}
\def\proofname{Proof}
\def\conjecturename{Conjecture}
\def\corollaryname{Corollary}
\def\definitionname{Definition}
\def\examplename{Example}
\def\exercisename{Exercise}
\def\lemmaname{Lemma}
\def\notename{Note}
\def\problemname{Problem}
\def\propertyname{Property}
\def\propositionname{Proposition}
\def\questionname{Question}
\def\solutionname{Solution}
\def\remarkname{Remark}"#);

  // The theorem environments themselves are installed with `\spnewtheorem` and `\spn@wtheorem` (below, after
  // the primitive is defined), numbered by the class's options as llncs.cls numbers them.

  // \spnewtheorem*{env}[numberedlike]{caption}[within]{capfont}{bodyfont}
  // Perl llncs.cls.ltxml L101-157 is `DefMacro('\spnewtheorem ...', sub
  // { ... DefMacroI + NewCounter + MergeFont ... })` — a macro whose
  // body imperatively installs new CSes/env bindings + counters.
  // Rust uses DefPrimitive (stomach-level) because the installation
  // needs to happen at digest-stable time so subsequent uses of the
  // new `{env}` work. WISDOM #44: kind flip is intentional —
  // `\spnewtheorem` is a preamble-declaration macro only, never
  // captured by `\edef`/`\ifx`. The per-theorem `bodyfont` is honored
  // (Perl L144 `afterDigestBegin => sub { Digest($bodyfont); }`): e.g.
  // `proof`/`case` pass `\rmfamily` = upright body, overriding the amsthm
  // default `\itshape`. `capfont` (the title font) is still ignored.
  DefPrimitive!("\\spnewtheorem OptionalMatch:* {}[]{}[] {}{}", sub[(flag, thmset, otherthmset, typ, reset, _capfont, bodyfont)] {
    engine::latex_constructs::define_new_theorem(
      flag.filter(|f| !f.is_empty()),
      thmset,
      otherthmset.filter(|t| !t.is_empty()),
      if typ.is_empty() { None } else { Some(typ) },
      reset.filter(|t| !t.is_empty()),
      Some(bodyfont).filter(|t| !t.is_empty()),
    )?;
  });
  Let!("\\spdefaulttheorem", "\\spnewtheorem");
  // llncs.cls:1060-1097 (TL 2025 v2.26) `\@spynthm{env}{caption}{capfont}{bodyfont}` (own counter),
  // `\@spxnthm{env}{caption}[within]{capfont}{bodyfont}` (counter within `within`) and
  // `\@spothm{env}[numberedlike]{caption}{capfont}{bodyfont}` (another theorem's counter), the three branches of
  // `\spnewtheorem`; and :1155-1167 `\spn@wtheorem{env}{caption}{capfont}{bodyfont}`, the class's definer of its
  // own theorem set, chosen by the numbering option, which papers call for theirs (2202.10090, 2601.04573
  // `\@spothm`; 0906.4121, 1003.2851 `\spn@wtheorem{fact}{Fact}…`; 22 papers). Perl's llncs.cls.ltxml defines none
  // of them (KNOWN_PERL_ERRORS #743). Every shipped copy from v2.2 to v2.26 chooses alike (v2.2's default
  // `\let\spn@wtheorem=\@spynthm` only skips the `chapter` reset). The v2.24+ `\@addtoreset{#1}{section}` of
  // `\@spothm` under envcountsect is left out: the env has no counter of its own here (it numbers with `#2`'s,
  // OXIDIZED_DESIGN_DIVERGENCES #614).
  RawTeX!(r"\def\@spynthm#1#2#3#4{\spnewtheorem{#1}{#2}{#3}{#4}}
\def\@spxnthm#1#2[#3]#4#5{\spnewtheorem{#1}{#2}[#3]{#4}{#5}}
\def\@spothm#1[#2]#3#4#5{\spnewtheorem{#1}[#2]{#3}{#4}{#5}}
\if@envcntsame\def\spn@wtheorem#1#2#3#4{\@spothm{#1}[theorem]{#2}{#3}{#4}}
\else\if@envcntsect\def\spn@wtheorem#1#2#3#4{\@spxnthm{#1}{#2}[section]{#3}{#4}}
\else\if@envcntreset\def\spn@wtheorem#1#2#3#4{\@spynthm{#1}{#2}{#3}{#4}\@addtoreset{#1}{section}}
\else\def\spn@wtheorem#1#2#3#4{\@spynthm{#1}{#2}{#3}{#4}\@addtoreset{#1}{chapter}}\fi\fi\fi");

  // The class's own theorem set, numbered as llncs.cls:1142-1174 numbers it (KNOWN_PERL_ERRORS #743; Perl's
  // llncs.cls.ltxml L170-202 gives every env its own counter and `{theorem}` a section-numbered one whatever the
  // options, "Theorem 0.1"): `{theorem}` by section under envcountsect, else on its own (reset by section under
  // envcountreset, by chapter otherwise); `{claim}` and `{proof}` unnumbered; the rest through `\spn@wtheorem`
  // (the theorem counter under envcountsame, by section under envcountsect, on their own otherwise). pdflatex,
  // TL 2025 v2.26: "Theorem 1, Lemma 1, Theorem 2" by default, "Theorem 1, Lemma 2" under envcountsame, "1.1"
  // under envcountsect (0906.4121's "Fact 3.1"). The capfont/bodyfont pairs are the class's, the same in every
  // shipped copy: `{definition}`'s body is italic (Perl's binding, upright). The `\@ifundefined` guards are a
  // Rust-only defensive keep (skip an env a preloaded package already defined); a plain `\documentclass{llncs}`
  // load pre-defines none of them.
  RawTeX!(r#"\@ifundefined{theorem}{\if@envcntsect
  \spnewtheorem{theorem}{Theorem}[section]{\bfseries}{\itshape}
\else
  \spnewtheorem{theorem}{Theorem}{\bfseries}{\itshape}
  \if@envcntreset\@addtoreset{theorem}{section}\else\@addtoreset{theorem}{chapter}\fi
\fi}{}
\@ifundefined{claim}{\spnewtheorem*{claim}{Claim}{\itshape}{\rmfamily}}{}
\@ifundefined{proof}{\spnewtheorem*{proof}{Proof}{\itshape}{\rmfamily}}{}
\@ifundefined{case}{\spn@wtheorem{case}{Case}{\itshape}{\rmfamily}}{}
\@ifundefined{conjecture}{\spn@wtheorem{conjecture}{Conjecture}{\itshape}{\rmfamily}}{}
\@ifundefined{corollary}{\spn@wtheorem{corollary}{Corollary}{\bfseries}{\itshape}}{}
\@ifundefined{definition}{\spn@wtheorem{definition}{Definition}{\bfseries}{\itshape}}{}
\@ifundefined{example}{\spn@wtheorem{example}{Example}{\itshape}{\rmfamily}}{}
\@ifundefined{exercise}{\spn@wtheorem{exercise}{Exercise}{\itshape}{\rmfamily}}{}
\@ifundefined{lemma}{\spn@wtheorem{lemma}{Lemma}{\bfseries}{\itshape}}{}
\@ifundefined{note}{\spn@wtheorem{note}{Note}{\itshape}{\rmfamily}}{}
\@ifundefined{problem}{\spn@wtheorem{problem}{Problem}{\itshape}{\rmfamily}}{}
\@ifundefined{property}{\spn@wtheorem{property}{Property}{\itshape}{\rmfamily}}{}
\@ifundefined{proposition}{\spn@wtheorem{proposition}{Proposition}{\bfseries}{\itshape}}{}
\@ifundefined{question}{\spn@wtheorem{question}{Question}{\itshape}{\rmfamily}}{}
\@ifundefined{solution}{\spn@wtheorem{solution}{Solution}{\itshape}{\rmfamily}}{}
\@ifundefined{remark}{\spn@wtheorem{remark}{Remark}{\itshape}{\rmfamily}}{}"#);

  //======================================================================
  // Blackboard bold letters.
  //
  // Perl: DefPrimitiveI('\bbbc', undef, "\x{2102}") etc. — DefPrimitive with
  // literal glyph body. Rust uses DefConstructor with inline-literal template
  // + enter_horizontal => true for explicit horizontal-mode entry. Functionally
  // equivalent (both emit ℂ, ℕ, ℝ, etc.). The DP audit flags 13 of these as
  // DefPrimitiveI↔DefConstructor structural mismatches — kind-flip is not
  // needed; Rust's DefConstructor is the idiomatic shape for literal-glyph
  // output with mode-entry semantics.
  DefConstructor!("\\bbbc",   "\u{2102}",   enter_horizontal => true);
  DefConstructor!("\\bbbf",   "\u{1D53D}",  enter_horizontal => true);
  DefConstructor!("\\bbbh",   "\u{210D}",   enter_horizontal => true);
  DefConstructor!("\\bbbk",   "\u{1D542}",  enter_horizontal => true);
  DefConstructor!("\\bbbm",   "\u{1D544}",  enter_horizontal => true);
  DefConstructor!("\\bbbn",   "\u{2115}",   enter_horizontal => true);
  DefConstructor!("\\bbbone", "\u{1D7D9}",  enter_horizontal => true);
  DefConstructor!("\\bbbp",   "\u{2119}",   enter_horizontal => true);
  DefConstructor!("\\bbbq",   "\u{211A}",   enter_horizontal => true);
  DefConstructor!("\\bbbr",   "\u{211D}",   enter_horizontal => true);
  DefConstructor!("\\bbbs",   "\u{1D54A}",  enter_horizontal => true);
  DefConstructor!("\\bbbt",   "\u{1D54B}",  enter_horizontal => true);
  DefConstructor!("\\bbbz",   "\u{2124}",   enter_horizontal => true);

  DefMath!("\\getsto", "\u{21C6}", role => "ARROW");
  DefMath!("\\lid",    "\u{2266}", role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\gid",    "\u{2267}", role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\grole",  "\u{2277}", role => "RELOP", meaning => "greater-than-or-less-than");

  // QED symbol, sized as llncs.cls:400 `\hbox{\rlap{$\sqcap$}$\sqcup$}` (a glyph box, not a rule): a size-less
  // whatsit alone in an alignment cell or `\mbox` made it "empty" and the QED was dropped (Perl Alignment.pm:458-470;
  // repro alignment-bindings/llncs_qed_alone_in_a_cell).
  DefConstructor!("\\squareforqed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    sizer => sub[whatsit] { Ok(engine::latex_constructs::sqcup_qed_size(whatsit)) });
  DefMacro!("\\qed", "\\squareforqed");

  //======================================================================
  def_macro_noop("\\backmatter")?;

  DefMacro!("\\andname", "and");
  DefMacro!("\\chaptername", "Chapter");
  DefMacro!("\\contriblistname", "List of Contributors");
  DefMacro!("\\lastandname", ", and");
  DefMacro!("\\noteaddname", "Note added in proof");
  DefMacro!("\\seename", "see");
  DefMacro!("\\subclassname", "\\textit{Subject Classification}:");

  DefRegister!("\\fnindent" => Dimension::new(0));
  DefMacro!("\\fnmsep", "${}^{,}$");
  DefMacro!("\\fnnstart", "0");

  def_macro_noop("\\calctocindent")?;
  def_macro_noop("\\clearheadinfo")?;
  DefRegister!("\\headlineindent" => Dimension::new(0));
  def_macro_noop("\\thisbottomragged")?;
  Let!("\\ts", "\\,");
  DefEnvironment!("{theopargself}", "#body");
  DefMacro!("\\homedir", "\\~{ }");
  DefMacro!("\\idxquad", "\\hskip 10pt\\relax");

  // (Perl PR #2767 removed the old \orcidID + \@@orcid + \@@@orcid trio;
  // \orcidID is now defined above via \lx@add@orcid.)

  // LLNCS v2.22+ introduced the {credits} environment for author
  // credits / disclosure-of-interests at the end of the paper. It just
  // switches to \small inside a group; treat as transparent.
  // Witnesses 2406.00947, 2406.05596, 2406.13788.
  DefMacro!(T_CS!("\\begin{credits}"), None, "");
  DefMacro!(T_CS!("\\end{credits}"), None, "");
  // \discintname is the localised "Disclosure of Interests." header.
  DefMacro!("\\discintname", "Disclosure of Interests.");

  // llncs.cls:79-117, :121-157, :161-197: the language switches, which babel's `\extrasenglish` and the
  // `deutsch`/`francais` options call and papers and their packages call or patch (pscproc2.sty:5 `\switcht@albion`:
  // 0905.3107, 0908.0239, 1006.0809; the Springer template's `\edef\switcht@deutsch{\relax\unexpanded\expandafter
  // {\switcht@deutsch}}`: 1812.03939, 1812.03953, 1904.02547). English is this binding's own names (those of the class
  // where the binding has none), German and French the class's, in the binding's forms where it defines the name
  // (`\keywordname` without the colon `\keywords` adds; `\abstractname`/`\ackname` without the run-in period;
  // OXIDIZED_DESIGN_DIVERGENCES #613).
  RawTeX!(r#"\edef\switcht@albion{\def\noexpand\abstractname{\unexpanded\expandafter{\abstractname}}%
\def\noexpand\ackname{\unexpanded\expandafter{\ackname}}\def\noexpand\andname{\unexpanded\expandafter{\andname}}%
\def\noexpand\lastandname{\unexpanded\expandafter{\lastandname}}\def\noexpand\appendixname{\unexpanded\expandafter{\appendixname}}%
\def\noexpand\chaptername{\unexpanded\expandafter{\chaptername}}\def\noexpand\contentsname{\unexpanded\expandafter{\contentsname}}%
\def\noexpand\discintname{\unexpanded\expandafter{\discintname}}\def\noexpand\figurename{\unexpanded\expandafter{\figurename}}%
\def\noexpand\keywordname{\unexpanded\expandafter{\keywordname}}\def\noexpand\indexname{\unexpanded\expandafter{\indexname}}%
\def\noexpand\contriblistname{\unexpanded\expandafter{\contriblistname}}\def\noexpand\listfigurename{\unexpanded\expandafter{\listfigurename}}%
\def\noexpand\listtablename{\unexpanded\expandafter{\listtablename}}\def\noexpand\mailname{\unexpanded\expandafter{\mailname}}%
\def\noexpand\noteaddname{\unexpanded\expandafter{\noteaddname}}\def\noexpand\partname{\unexpanded\expandafter{\partname}}%
\def\noexpand\seename{\unexpanded\expandafter{\seename}}\def\noexpand\subclassname{\unexpanded\expandafter{\subclassname}}%
\def\noexpand\tablename{\unexpanded\expandafter{\tablename}}%
\def\noexpand\claimname{Claim}\def\noexpand\conjecturename{Conjecture}\def\noexpand\corollaryname{Corollary}%
\def\noexpand\definitionname{Definition}\def\noexpand\examplename{Example}\def\noexpand\exercisename{Exercise}%
\def\noexpand\lemmaname{Lemma}\def\noexpand\notename{Note}\def\noexpand\problemname{Problem}\def\noexpand\proofname{Proof}%
\def\noexpand\propertyname{Property}\def\noexpand\propositionname{Proposition}\def\noexpand\questionname{Question}%
\def\noexpand\remarkname{Remark}\def\noexpand\solutionname{Solution}\def\noexpand\theoremname{Theorem}}
\def\switcht@francais{\def\abstractname{R\'esum\'e}\def\ackname{Remerciements}\def\andname{et}\def\lastandname{ et}%
\def\appendixname{Appendice}\def\chaptername{Chapitre}\def\claimname{Pr\'etention}\def\conjecturename{Hypoth\`ese}%
\def\contentsname{Table des mati\`eres}\def\corollaryname{Corollaire}\def\definitionname{D\'efinition}%
\def\discintname{Mention des Int\'{e}r\^{e}ts.}\def\examplename{Exemple}\def\exercisename{Exercice}\def\figurename{Fig.}%
\def\keywordname{\textbf{Mots-cl\'e}}\def\indexname{Index}\def\lemmaname{Lemme}\def\contriblistname{Liste des contributeurs}%
\def\listfigurename{Liste des figures}\def\listtablename{Liste des tables}\def\noteaddname{Note ajout\'ee \`a l'\'epreuve}%
\def\notename{Remarque}\def\partname{Partie}\def\problemname{Probl\`eme}\def\proofname{Preuve}%
\def\propertyname{Caract\'eristique}\def\questionname{Question}\def\remarkname{Remarque}\def\seename{voir}%
\def\solutionname{Solution}\def\tablename{Tableau}\def\theoremname{Th\'eor\`eme}}
\def\switcht@deutsch{\def\abstractname{Zusammenfassung}\def\ackname{Danksagung}\def\andname{und}\def\lastandname{ und}%
\def\appendixname{Anhang}\def\chaptername{Kapitel}\def\claimname{Behauptung}\def\conjecturename{Hypothese}%
\def\contentsname{Inhaltsverzeichnis}\def\corollaryname{Korollar}\def\discintname{Offenlegung von Interessen.}%
\def\examplename{Beispiel}\def\exercisename{\"Ubung}\def\figurename{Abb.}\def\keywordname{\textbf{Schl\"usselw\"orter}}%
\def\indexname{Index}\def\contriblistname{Mitarbeiter}\def\listfigurename{Abbildungsverzeichnis}%
\def\listtablename{Tabellenverzeichnis}\def\noteaddname{Nachtrag}\def\notename{Anmerkung}\def\partname{Teil}%
\def\proofname{Beweis}\def\propertyname{Eigenschaft}\def\questionname{Frage}\def\remarkname{Anmerkung}\def\seename{siehe}%
\def\solutionname{L\"osung}\def\tablename{Tabelle}}"#);
});
