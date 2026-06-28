use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: llncs.cls.ltxml — Lecture Notes in Computer Science (Springer)
  for option in [
    "envcountreset", "citeauthoryear", "oribibl", "orivec",
    "envcountsame", "envcountsect", "runningheads",
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
  def_macro_noop("\\frontmatter")?;

  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");

  DefMacro!("\\emailname", "E-mail");
  DefMacro!("\\mailname",  "\\textit{Correspondence to}:");

  // Single \author, with multiple authors separated by \and  (Perl PR #2767)
  // \inst{labels} can be used within each author to identify which affiliations apply.
  // IMPROVEMENT over upstream: also split on \\ -- authors commonly use a manual
  // row break in place of \and ("A\inst{1} \\ B\inst{2}"), which upstream merges
  // into a single creator (witness 2606.19939: "Hiuyi Cheng \\ Dezhi Peng").
  DefMacro!("\\author{}",
    "\\lx@clear@creators[role=author]\\lx@splitting{\\lx@add@author}{\\and\\And,\\\\}{#1}");
  // Single \institute, with multiple institutions separated by \and.
  // The n-th institution is attached to the author which has that n in its \inst labels.
  //
  // IMPROVEMENT over upstream llncs.cls.ltxml: upstream splits \institute ONLY on
  // \and. The very common lazy LNCS style instead writes ONE \institute block with
  // hand-typed superscripts ("$^1$..\quad $^2$..") and no \and; upstream then makes
  // a single affiliation = the whole block, attached to every \inst{1} author
  // (duplicating it) while \inst{2}+ authors get nothing (witness 2606.19939;
  // reproduces identically in Perl LaTeXML 0.8.8). Instead we route through the
  // shared smart affiliation parser \lx@add@affiliations, which already handles
  // BOTH forms: with superscript markers it splits on \quad/\qquad/\\ and extracts
  // each superscript as the affiliation label that \inst{N} links to; without them
  // it splits on \and (affil_splits() was extended to include \and). labelseq gives
  // the \and-separated form its sequential 1,2,.. labels for \inst linking.
  DefMacro!("\\institute{}",
    "\\lx@add@affiliations[labelseq={affiliation}]{#1}");
  DefMacro!("\\inst{}",                 "\\lx@request@frontmatter@annotation[affiliation]{#1}");
  // \orcidID should be used within each author in \author
  DefMacro!("\\orcidID{}", "\\lx@add@orcid{#1}");
  // \email, \url, \mail should be within an institute in \institute!!
  DefMacro!("\\email Semiverbatim", "\\lx@add@email[name={\\emailname~}]{#1}");
  DefMacro!("\\url Semiverbatim",   "\\lx@add@url{#1}");
  DefMacro!("\\mail {}",            "\\lx@add@address[name={\\mailname~}]{#1}");
  DefMacro!("\\keywordname",        "\\textbf{Keywords}");
  DefMacro!("\\keywords{}",         "\\lx@add@keywords[name={\\keywordname:~}]{#1}");

  DefMacro!("\\ackname", "Acknowledgements");
  DefConstructor!("\\acknowledgements", "<ltx:acknowledgements name='#name'>",
    properties => sub[_args] {
      let name = Stored::from(digest(T_CS!("\\ackname"))?);
      Ok(stored_map!("name" => name))
    });
  DefMacro!("\\acknowledgement", "\\acknowledgements");
  DefConstructor!("\\endacknowledgements", "</ltx:acknowledgements>");
  DefConstructor!("\\endacknowledgement", "</ltx:acknowledgements>");
  Tag!("ltx:acknowledgements", auto_close => true);

  DefConstructor!("\\url Semiverbatim", "<ltx:ref href='#1'>#1</ltx:ref>");

  DefRegister!("\\instindent" => Dimension::new(0));
  DefRegister!("\\authrun" => Tokens!());
  DefRegister!("\\authorrunning" => Tokens!());
  DefRegister!("\\tocauthor" => Tokens!());
  DefRegister!("\\titrun" => Tokens!());
  DefRegister!("\\titlerunning" => Tokens!());
  // Perl llncs.cls.ltxml L73: `DefRegister('\toctitle{}' => Tokens())`
  // — a Tokens-valued register with a `{}` proto, meaning the register
  // is read/written via an argument. Rust's DefRegister! doesn't accept
  // a `{}` proto (the macro only handles name-only register shapes).
  //
  // Was: DefMacro!("\\toctitle{}", "") — but this swallows the FOLLOWING
  // TOKEN whenever `\toctitle` appears bare-name. Driver: 2112.13058
  // user wrote `\tocauthor\toctitle\maketitle` (the sample llncs
  // template). `\toctitle{}` ate `\maketitle` as its required arg,
  // skipping the frontmatter-emitting \maketitle and producing
  // "Can't close environment abstract" because abstract was processed
  // without an open document-frontmatter slot.
  //
  // Fix: drop the `{}` proto; treat `\toctitle` as a no-op CS that
  // accepts no arg. User code `\toctitle{TOC text}` will leave the
  // `{TOC text}` as a balanced group that gets digested as empty
  // text in the surrounding context — same observable output as the
  // discarding-macro path.
  def_macro_noop("\\toctitle")?;

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
  def_macro_noop("\\mainmatter")?;

  NewCounter!("chapter", "document", idprefix => "Pt", nested => vec!["section"]);
  DefMacro!("\\thechapter", "\\arabic{chapter}");
  def_macro_noop("\\chaptermark{}")?;

  // Theorem-family \xxxname definitions. The \spnewtheorem primitive itself
  // is ported further below (L133+) via define_new_theorem; capfont/bodyfont
  // are ignored per Perl precedent since visual styling isn't modeled.
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

  // Pre-define theorem environments — Perl uses \spnewtheorem with capfont/bodyfont
  RawTeX!("\\@ifundefined{theorem}{\\newtheorem{theorem}{Theorem}[section]}{}");
  RawTeX!("\\@ifundefined{claim}{\\newtheorem*{claim}{Claim}}{}");
  RawTeX!("\\@ifundefined{proof}{\\newtheorem*{proof}{Proof}}{}");
  RawTeX!("\\@ifundefined{case}{\\newtheorem*{case}{Case}}{}");
  RawTeX!("\\@ifundefined{conjecture}{\\newtheorem{conjecture}[theorem]{Conjecture}}{}");
  RawTeX!("\\@ifundefined{corollary}{\\newtheorem{corollary}[theorem]{Corollary}}{}");
  RawTeX!("\\@ifundefined{definition}{\\newtheorem{definition}[theorem]{Definition}}{}");
  RawTeX!("\\@ifundefined{example}{\\newtheorem{example}[theorem]{Example}}{}");
  RawTeX!("\\@ifundefined{exercise}{\\newtheorem{exercise}[theorem]{Exercise}}{}");
  RawTeX!("\\@ifundefined{lemma}{\\newtheorem{lemma}[theorem]{Lemma}}{}");
  RawTeX!("\\@ifundefined{note}{\\newtheorem{note}[theorem]{Note}}{}");
  RawTeX!("\\@ifundefined{problem}{\\newtheorem{problem}[theorem]{Problem}}{}");
  RawTeX!("\\@ifundefined{property}{\\newtheorem{property}[theorem]{Property}}{}");
  RawTeX!("\\@ifundefined{proposition}{\\newtheorem{proposition}[theorem]{Proposition}}{}");
  RawTeX!("\\@ifundefined{question}{\\newtheorem{question}[theorem]{Question}}{}");
  RawTeX!("\\@ifundefined{solution}{\\newtheorem{solution}[theorem]{Solution}}{}");
  RawTeX!("\\@ifundefined{remark}{\\newtheorem{remark}[theorem]{Remark}}{}");

  // \spnewtheorem*{env}[numberedlike]{caption}[within]{capfont}{bodyfont}
  // Perl llncs.cls.ltxml L101-157 is `DefMacro('\spnewtheorem ...', sub
  // { ... DefMacroI + NewCounter + MergeFont ... })` — a macro whose
  // body imperatively installs new CSes/env bindings + counters.
  // Rust uses DefPrimitive (stomach-level) because the installation
  // needs to happen at digest-stable time so subsequent uses of the
  // new `{env}` work. WISDOM #44: kind flip is intentional —
  // `\spnewtheorem` is a preamble-declaration macro only, never
  // captured by `\edef`/`\ifx`. capfont/bodyfont are TeX font
  // commands (e.g. \bfseries, \itshape) — ignored in LaTeXML (both
  // Perl and Rust do the same).
  DefPrimitive!("\\spnewtheorem OptionalMatch:* {}[]{}[] {}{}", sub[(flag, thmset, otherthmset, typ, reset, _capfont, _bodyfont)] {
    engine::latex_constructs::define_new_theorem(
      flag.filter(|f| !f.is_empty()),
      thmset,
      otherthmset.filter(|t| !t.is_empty()),
      if typ.is_empty() { None } else { Some(typ) },
      reset.filter(|t| !t.is_empty()),
    )?;
  });
  Let!("\\spdefaulttheorem", "\\spnewtheorem");

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

  // QED symbol
  DefConstructor!("\\squareforqed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})");
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
});
