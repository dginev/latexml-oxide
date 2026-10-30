use crate::prelude::*;

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// llncs: `\institute{A \and B}` numbered by its `\and`s (`labelseq=affiliation`), `\inst` the request; an `\inst`
/// after a comma is the name's before it; an unmarked `\\` line under names is their affiliation; `\email` prints in
/// place (llncs.cls:897), so a `\thanks{Corresponding: A. Able, \email{…}}` names its address's owner. `written`:
/// the observer's writer writes llncs's creators (FM stage 7, OD #922; a paper whose reading it cannot apply keeps
/// today's output).
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "llncs",
  binding: "llncs.cls",
  inst_after_comma: true,
  and_inline: true,
  email_in_place: true,
  // written: true, (step 3)
  cite: "llncs.cls:870-885, 894-897; OD #791 (2405.16615, 2512.08621), #792 (2604.11038), #807 (2401.16957), #913",
  ..FM_KERNEL_CONVENTIONS
};

#[rustfmt::skip]
LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
  // Perl: llncs.cls.ltxml — Lecture Notes in Computer Science (Springer)
  // llncs.cls:25-36 (TL 2025 v2.26): the switches the numbering and running-head options set, which papers and their
  // packages test (`\if@runhead`: pscproc2.sty:19, 0905.3107, 0908.0239, 1006.0809; `\if@envcntsame`: 2303.08726,
  // 2306.07698) and `\spn@wtheorem` (below) is chosen by.
  RawTeX!(r"\let\if@envcntreset\iffalse\let\if@envcntsame\iffalse\let\if@envcntsect\iffalse\let\if@runhead\iffalse");
  DeclareOption!("envcountreset", "\\let\\if@envcntreset\\iftrue");
  DeclareOption!("envcountsame", "\\let\\if@envcntsame\\iftrue");
  DeclareOption!("envcountsect", "\\let\\if@envcntsect\\iftrue");
  DeclareOption!("runningheads", "\\let\\if@runhead\\iftrue");
  // llncs.cls:29-30 `\let\if@custvec\iftrue`, `orivec` false: the class's `\vec` (below) or the kernel's.
  RawTeX!(r"\let\if@custvec\iftrue");
  DeclareOption!("orivec", "\\let\\if@custvec\\iffalse");
  for option in [
    "citeauthoryear", "oribibl",
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
  // llncs.cls:379-389 `\DeclareMathSymbol{\Gamma}{\mathalpha}{letters}{"00}`…: the capital Greek letters are the math
  // italic's, italic as a Latin letter unless a math alphabet is in force (`$\Omega$` CMMI10, `\mathrm{\Omega}` CMR10,
  // `\vec{\Omega}` CMMIB10 in pdflatex; Perl's binding leaves them upright, KNOWN_PERL_ERRORS #1147). Witnesses
  // 2606.09074 (Δ, Π CMMI10/CMMI7 in its PDF), 2604.00509, 2607.20650.
  declare_capital_greek("", Some(LettersClass::Variable))?;
  // llncs.cls:393-397 (TL 2025, v2.26; `\def\vec` in 367 of the census's 587 copies, the robust command in 213):
  // `\vec{x}` is `\mbox{\boldmath$x$}`, the bold italic letter (CMMIB10 in pdflatex), not the kernel's arrow accent,
  // unless `orivec` (Perl's binding ignores it: KNOWN_PERL_ERRORS #1143). In maths only, as there (no `\ensuremath`).
  // 44 of the census's 690 llncs papers use it (2607.20650, 2607.18197, 2604.00509).
  sv_support_sty::define_bold_vectors()?;
  RawTeX!(r"\if@custvec\def\vec#1{\lx@sv@vec@boldmath{#1}}\fi");

  RequirePackage!("multicol");
  // LLNCS authors routinely use \boldsymbol (amsbsy) and \mathbb / \mathfrak
  // (amssymb) in math without explicit \usepackage. The real Springer
  // llncs.cls quietly tolerates these because most authors run pdflatex
  // with amssymb/amsbsy pulled in by transitive paper deps; mirror by
  // eager-loading here. Witness 2211.16852.
  RequirePackage!("amsbsy");
  RequirePackage!("amssymb");
  // llncs.cls:521-522 (every version sampled, the v2.x copies of 1006.0220, 1006.0673 too): the first level's item
  // label is a bold en dash, the second a bullet (`$\m@th\bullet$`, set as the kernel's text bullet) — article's
  // the other way round, which Perl's binding keeps (KNOWN_PERL_ERRORS #1017). Witness 2607.29279 (waica.cls,
  // a copy repeating it). Guard `perfect_kernel_batch78a::llncs_item_labels_are_the_class_s`.
  RawTeX!(r"\renewcommand\labelitemi{\normalfont\bfseries --}\renewcommand\labelitemii{\textbullet}");

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
  // An `\institute{…}` written inside `\author` (`\author{Ann Able and Bob Baker \institute{Univ X}}`, 1609.01220) is
  // the class's store all the same, `\gdef\@institute` (llncs.cls:868), printed under the names by `\institutename`
  // (:1036): one after the names, not a part of the last name. It runs when `\@maketitle` sets `\@author`, before
  // `\institutename` prints the store, so it wins over any `\institute` given outside, before or after (the 85b
  // review's r02: `\author{Ann Able \and Bob Baker \institute{Univ X}}\institute{Univ Y}` prints "Univ X"): taken
  // here, it is the store, and an outer `\institute` given while it stands sets nothing (`\lx@llncs@inner@institute`;
  // a later `\author` without one gives the store back to the outer calls).
  DefMacro!("\\author{}", sub[(body)] {
    let (names, institutes) = take_institutes(&body);
    let mut out = mouth::tokenize_internal("\\lx@clear@creators[role=author]").unlist();
    out.extend(Invocation!(T_CS!("\\lx@llncs@author"), vec![Some(names)]).unlist());
    let inner = !institutes.is_empty();
    out.extend(mouth::tokenize_internal("\\gdef\\lx@llncs@inner@institute{}").unlist());
    out.extend(institutes);
    if inner {
      out.extend(mouth::tokenize_internal("\\gdef\\lx@llncs@inner@institute{1}").unlist());
    }
    out.push(T_CS!("\\lx@author@trailing"));
    Ok(Tokens::new(out))
  });
  // llncs.cls:1019-1036 sets `\@author` as one centred paragraph, so a `\\` there is a line break of the author list,
  // and its `\and` is inline text (llncs.cls:1013-1018, `\unskip,` or `\lastand` in `\@maketitle`): a line reading as
  // an affiliation (`\author{Ann Able\\University of Somewhere\\\email{…}}`) is printed under the names since the last
  // line under names, an affiliation or an e-mail — their affiliation, whichever `\and` joined them (`Ann Able \and
  // Bob Baker\\ University of Somewhere`, 2604.11038's Simon Fraser University on its three authors; `Ann Able\\
  // University of X\\ Bob Baker\\ University of Y`, Y Bob Baker's only: the rev77t review's L1, L2, L4). Was a
  // creator named "University of Somewhere" holding the e-mail (review probe rv63k/llncs1). It goes into each name's
  // author content, as an `\email` there does, so `\institute`'s clearing of the affiliations queued before it (below)
  // leaves it; only in a list with no `\inst` marks, since a contact digested with the authors comes before
  // `\institute`'s entries in the count `labelseq=affiliation` numbers them by, which the marks request.
  DefMacro!("\\lx@llncs@author{}", sub[(body)] {
    // (a list whose authors carry `\inst` marks keeps the split, its `\institute` entries picked by the marks: the
    // rev77t2 re-review's L11)
    if author_mark_position(&body).is_some() || position_of(&body, &[T_CS!("\\inst")]).is_some() {
      return Ok(Invocation!(T_CS!("\\lx@llncs@author@split"), vec![Some(body)]));
    }
    // (each author's content, the affiliation lines under it appended; the names of the row an affiliation line goes
    // to, and whether a line under them came, so that the next names start a new row)
    let mut authors: Vec<Vec<Token>> = Vec::new();
    let mut row: Vec<usize> = Vec::new();
    let mut under_row = false;
    let pieces = split_tokens_delimited(strip_linebreak_options(body),
      vec![T_CS!("\\and").into(), T_CS!("\\And").into(), T_CS!("\\\\").into()]);
    for (delimiter, line) in pieces {
      // (only a line after `\\` is printed under the names; a piece after the inline `\and` is more names, `Ann Able
      // \and Bob Baker \and the ATLAS Collaboration`, the rev77t2 re-review's L6)
      let line_break = delimiter.first() == Some(&T_CS!("\\\\"));
      if line_break && !row.is_empty() && reads_as_affiliation(&line) {
        let call = Invocation!(T_CS!("\\lx@add@affiliation"), vec![None, Some(line)]).unlist();
        for &at in &row {
          authors[at].extend(call.iter().copied());
        }
        under_row = true;
      } else if line_is_email_list(&line) {
        under_row |= !row.is_empty();
        authors.extend(split_author_line(line).into_iter().map(Tokens::unlist));
      } else {
        let names = split_author_line(line);
        if under_row && !names.is_empty() {
          row.clear();
          under_row = false;
        }
        for name in names {
          row.push(authors.len());
          authors.push(name.unlist());
        }
      }
    }
    let mut calls: Vec<Token> = Vec::new();
    for author in authors {
      calls.extend(Invocation!(T_CS!("\\lx@add@author"), vec![None, Some(Tokens::new(author))]).unlist());
    }
    Ok(Tokens::new(calls))
  });
  DefMacro!("\\lx@llncs@author@split{}", "\\lx@splitting{\\lx@add@author@split}{\\and\\And\\\\}{#1}");
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
  //      #6881). This is Perl's plain case: it must stay ONE affiliation, on all
  //      the \inst{1} authors. Splitting the "\\" here (the pre-fix bug) put the
  //      \email in its own "affiliation:2" contact that no author carries, so it
  //      mislanded on a single creator and left an empty affiliation stub. Its
  //      \email lines are each address its owner's (85b, below), not the brace
  //      list whole on every author, which inheriting "affiliation:1" made it.
  //  (c) Lazy: ONE block, no \and, hand-typed superscripts ("$^1$..\quad $^2$..")
  //      (witness 2606.19939). Upstream makes a single affiliation = the whole
  //      block attached to every \inst{1} author (duplicating it) while \inst{2}+
  //      get nothing (reproduces identically in Perl LaTeXML 0.8.8). The shared
  //      \lx@add@affiliations parser handles this: it splits on \quad/\qquad/\\
  //      and extracts each superscript as the affiliation label \inst{N} links to.
  // So: clear, then \and-split (a), one affiliation (b) — shared by every author when no
  // `\inst` number picks it (85b, below) — UNLESS there is no \and AND superscript
  // markers are present (c) -- the only shape that needs the multi-parser.
  // (an `\institute` inside the last `\author` is the store, which an outer one does not replace: above)
  DefMacro!("\\lx@llncs@inner@institute", "");
  DefMacro!("\\institute{}",
    "\\ifx\\lx@llncs@inner@institute\\@empty\\expandafter\\@firstofone\\else\\expandafter\\@gobble\\fi\
     {\\lx@clear@frontmatter{ltx:contact}[role=affiliation]\\lx@llncs@institute@dispatch{#1}}");
  DefMacro!("\\lx@llncs@institute@split{}", "\\lx@splitting{\\lx@llncs@affiliation}{\\and}{#1}");
  DefMacro!("\\lx@llncs@institute@sup{}",   "\\lx@add@affiliations[labelseq={affiliation}]{#1}");
  DefMacro!("\\lx@llncs@institute@dispatch{}", sub[(body)] {
    let has_and = position_of(&body, &[T_CS!("\\and")]).is_some();
    // The markers \lx@add@affiliations keys its superscript-label branch on (a `\let` alias of one too).
    let has_sup = author_mark_position(&body).is_some();
    let target = if !has_and && has_sup {
      T_CS!("\\lx@llncs@institute@sup")     // shape (c): lazy superscript-labeled block
    } else if !has_and && body.unlist_ref().iter().any(|t| t.get_catcode() != Catcode::SPACE) {
      // (an empty `\institute{}` stays none, as the split of nothing makes it: the `\inst` marks it answers no one
      // stay shown, 2407.13764)
      T_CS!("\\lx@llncs@institute@sole")    // shape (b): one institute
    } else {
      T_CS!("\\lx@llncs@institute@split")   // shape (a): \and-split
    };
    Ok(Invocation!(target, vec![Some(body)]))
  });
  DefMacro!("\\lx@llncs@affiliation{}", "\\lx@add@affiliation[labelseq={affiliation}]{#1}");
  // llncs.cls:868-886: one institute (`\c@@inst=1`, no `\and` in it) is printed under the names unnumbered, every
  // author's, unless the authors' `\inst` marks pick it (`\inst{1}`): `_share=authors`, which the frontmatter's flush
  // reads (`share_row_contacts`, OXIDIZED_DESIGN_DIVERGENCES #901; Perl strands it on the first author for want of an
  // `\inst{1}`, KNOWN_PERL_ERRORS #1195, and the kernel had kept it on a nameless creator under the names,
  // OXIDIZED_DESIGN #159, 2402.19043). Only `\inst{n}` picks it: hand-typed `$^{1}$` (2607.02596) and a paper's mark
  // macro (`\printfnsymbol{1}`, 1804.05870, 2007.07423, 2102.12877, 2403.11865) are answered by no one. Its `\email`s
  // are lines under the names, picked or shared, each decided on its own: the address the author it spells, or in a
  // list one per author the author at its place (2410.04091's `\{…\}` addresses), a call the reader cannot part the
  // block's (2206.10050; FIDELITY_CHARTER §4). Witnesses 1002.2793, 1307.8067,
  // 1405.1167, 1509.00818, 1609.01220, 1809.11127, 1910.11599, 2011.00616, 2101.06781, 2106.04289, 2205.06740,
  // 2207.04574 (an `\inst{*}`), 2207.11464, 2306.03360, 2405.11276, 2502.08785, 2511.12796, 2603.26486, 2604.20229
  // (`\inst{}`), 1701.08125; the shape (b) witness 2608.11332 (`\inst{1}` on every author) is picked by its marks, as
  // before, its brace list each address its owner's.
  // Guards `perfect_kernel_batch85b`.
  DefMacro!("\\lx@llncs@institute@sole{}", "\\lx@add@affiliation[labelseq={affiliation},_share={authors}]{#1}");
  // (an address of such a line is set as llncs's `\email` sets its own, under its name, llncs.cls:897)
  DefMacro!("\\lx@row@email{}{}", "\\lx@add@email[name={\\emailname~},label={#1}]{#2}");
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
  // …but the class's `\appendix` is article's (llncs.cls:51 `\LoadClass[twoside]{article}`; article.cls:439-442), the
  // appendices sections numbered within the document, "A" (its subsections "A.1", the equations running on), not
  // within the `\chapter` the class adds for proceedings volumes (:302): the kernel numbers them within chapters where
  // the class defines one (`begin_appendices`), "0.A" and ids `Pt0.A1`. Perl the same (latex_constructs.pool.ltxml:689-692
  // tests `\c@chapter`; "Appendix 0.A"; 2609.39362, 2303.08726): KNOWN_PERL_ERRORS #1099. The appendices' ids are
  // `A1`, `A2`, … as in article. Guard `perfect_kernel_batch78q::llncs_appendices_are_sections`.
  DefPrimitive!("\\@appendix", { engine::latex_constructs::begin_appendices_within("section", false); });

  // Theorem-family \xxxname definitions. The class's `\spnewtheorem` is ported further below (its branches over the
  // `\lx@spnewtheorem` primitive, via define_new_theorem), and the theorem environments are installed there with
  // their per-theorem bodyfont honored (capfont/title font is still ignored).
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

  // `\lx@spnewtheorem*{env}[numberedlike]{caption}[within]{capfont}{bodyfont}` defines the environment the class's
  // `\spnewtheorem` branches (below) define. Perl llncs.cls.ltxml L101-157 is this as `\spnewtheorem` itself:
  // `DefMacro('\spnewtheorem ...', sub { ... DefMacroI + NewCounter + MergeFont ... })` — a macro whose
  // body imperatively installs new CSes/env bindings + counters.
  // Rust uses DefPrimitive (stomach-level) because the installation
  // needs to happen at digest-stable time so subsequent uses of the
  // new `{env}` work. WISDOM #44: kind flip is intentional —
  // it is a preamble-declaration only, never
  // captured by `\edef`/`\ifx`. The per-theorem `bodyfont` is honored
  // (Perl L144 `afterDigestBegin => sub { Digest($bodyfont); }`): e.g.
  // `proof`/`case` pass `\rmfamily` = upright body, overriding the amsthm
  // default `\itshape`. `capfont` (the title font) is still ignored.
  DefPrimitive!("\\lx@spnewtheorem OptionalMatch:* {}[]{}[] {}{}", sub[(flag, thmset, otherthmset, typ, reset, _capfont, bodyfont)] {
    engine::latex_constructs::define_new_theorem(
      flag.filter(|f| !f.is_empty()),
      thmset,
      otherthmset.filter(|t| !t.is_empty()),
      if typ.is_empty() { None } else { Some(typ) },
      reset.filter(|t| !t.is_empty()),
      Some(bodyfont).filter(|t| !t.is_empty()),
    )?;
  });
  // llncs.cls:1061 (the block :1060-1125, TL 2025 v2.26): `\spnewtheorem` is the class's, dispatching on its `*` and
  // `[`s to `\@sthm`/`\@Ynthm` (unnumbered), `\@spothm{env}[numberedlike]{caption}{capfont}{bodyfont}` (another
  // theorem's counter), `\@spxnthm{env}{caption}[within]{capfont}{bodyfont}` (a counter within `within`) and
  // `\@spynthm{env}{caption}{capfont}{bodyfont}` (its own counter); :1155-1167 `\spn@wtheorem{env}{caption}{capfont}
  // {bodyfont}`, the class's definer of its own theorem set, chosen by the numbering option, which papers call for
  // theirs (2202.10090, 2601.04573 `\@spothm`; 0906.4121, 1003.2851 `\spn@wtheorem{fact}{Fact}…`; 22 papers). Perl's
  // llncs.cls.ltxml defines none of them (KNOWN_PERL_ERRORS #743). Each branch, as the class's, refuses an env
  // already defined (`\@ifdefinable`: "Command \lemma already defined", an Info here as every kernel "already
  // defined", base_utilities.rs `is_typesetting_only_message`; the first definition stands, as in the PDF), and
  // defines `\<env>name` as the caption (hyperref's `\autoref` names a paper's own `{obs}` "Observation 4" by it)
  // and titles the env through `\csname <env>name\endcsname` (llncs.cls:1073/1082/1096/1122), so a paper's
  // `\renewcommand\lemmaname{Lemme}` renames title and autoref alike ("Lemme 2").
  // Every shipped copy from v2.2 to v2.26 chooses alike (v2.2's default `\let\spn@wtheorem=\@spynthm` only skips the
  // `chapter` reset).
  RawTeX!(r"\def\spnewtheorem{\@ifstar{\@sthm}{\@Sthm}}
\def\@spnthm#1#2{\@ifnextchar[{\@spxnthm{#1}{#2}}{\@spynthm{#1}{#2}}}
\def\@Sthm#1{\@ifnextchar[{\@spothm{#1}}{\@spnthm{#1}}}
\def\@spynthm#1#2#3#4{\expandafter\@ifdefinable\csname #1\endcsname
  {\expandafter\xdef\csname #1name\endcsname{#2}\lx@spnewtheorem{#1}{\csname #1name\endcsname}{#3}{#4}}}
\def\@spxnthm#1#2[#3]#4#5{\expandafter\@ifdefinable\csname #1\endcsname
  {\expandafter\xdef\csname #1name\endcsname{#2}\lx@spnewtheorem{#1}{\csname #1name\endcsname}[#3]{#4}{#5}}}
\def\@sthm#1#2{\@Ynthm{#1}{#2}}
\def\@Ynthm#1#2#3#4{\expandafter\@ifdefinable\csname #1\endcsname
  {\expandafter\xdef\csname #1name\endcsname{#2}\lx@spnewtheorem*{#1}{\csname #1name\endcsname}{#3}{#4}}}");
  Let!("\\spdefaulttheorem", "\\spnewtheorem");
  // `\@spothm` differs between versions; the paper's copy of the class decides (the TL 2025 v2.26 one without a
  // copy). Each raises the class's "No theorem environment `#2' defined" (an Error) for an undefined counter `#2`.
  // - v2.2-v2.14: `\@namedef{the#1}{\@nameuse{the#2}}` and `\@spthm{#2}`: the env steps `#2`, the type hyperref
  //   names it by ("Theorem 2" for an envcountsame lemma).
  // - v2.16-v2.23: `\newaliascnt{#1}{#2}` (aliascnt.sty, which these copies load) and `\@spthm{#1}`: the env steps
  //   its own alias of `#2`'s counter, so it shares `#2`'s number and is its own type ("Lemma 2": 2202.10090).
  // - v2.24+: also `\if@envcntsect\@addtoreset{#1}{section}\fi` ("fixes incorrect hypertexnames when envcountsect is
  //   used in combination with envcountsame"): the section resets `#2`'s counter through the alias, which matters
  //   where `#2` is not `theorem` (already reset by the section): under envcountsect a paper's
  //   `\spnewtheorem{conj}[hyp]` over its own `\spnewtheorem{hyp}{Hypothesis}` numbers "Hypothesis 1, Conj 2" in
  //   every section in v2.26 and "Hypothesis 3, Conj 4" in section 2 in v2.20 (OXIDIZED_DESIGN_DIVERGENCES #614).
  // aliascnt itself is not loaded (`\lx@llncs@newaliascnt` is its `\newaliascnt`, aliascnt.sty v1.5 :40-61, without
  // the `\@addtoreset` patch, which only maps a `within` alias to its root): `\c@#1`, `\the#1`, `\theH#1` and `\p@#1`
  // are `#2`'s, `\AC@cnt@#1` and `\cl@#1` record the alias as aliascnt.sty :54-57 does (a paper loading aliascnt
  // itself calls `\aliascntresetthe{lemma}` on it), and `register_counter_alias` records it for the numbering and for
  // hyperref.
  DefPrimitive!("\\lx@llncs@aliascnt@register{}{}", sub[(alias, base)] {
    register_counter_alias(&alias.to_string(), &base.to_string());
  });
  RawTeX!(r"\def\lx@llncs@newaliascnt#1#2{\expandafter\@ifdefinable\csname c@#1\endcsname
  {\global\expandafter\let\csname c@#1\expandafter\endcsname\csname c@#2\endcsname
  \global\expandafter\let\csname the#1\expandafter\endcsname\csname the#2\endcsname
  \global\expandafter\let\csname theH#1\expandafter\endcsname\csname theH#2\endcsname
  \global\expandafter\let\csname p@#1\expandafter\endcsname\csname p@#2\endcsname
  \expandafter\gdef\csname AC@cnt@#1\endcsname{#2}%
  \expandafter\gdef\csname cl@#1\expandafter\endcsname\expandafter{\csname cl@#2\endcsname}%
  \lx@llncs@aliascnt@register{#1}{#2}}}");
  let spothm = crate::shipped_class::shipped_class_text("llncs")?
    .and_then(|text| crate::shipped_class::class_code_argument(&text, "\\def\\@spothm#1[#2]#3#4#5"));
  let (alias, section_reset) = spothm
    .as_deref()
    .map_or((true, true), |body| (body.contains("\\newaliascnt"), body.contains("\\@addtoreset")));
  let reset = if section_reset { r"\if@envcntsect\@addtoreset{#1}{section}\fi" } else { "" };
  let define = if alias {
    s!(r"\lx@llncs@newaliascnt{{#1}}{{#2}}\expandafter\xdef\csname #1name\endcsname{{#3}}{reset}\lx@spnewtheorem{{#1}}[#1]{{\csname #1name\endcsname}}{{#4}}{{#5}}")
  } else {
    r"\expandafter\xdef\csname #1name\endcsname{#3}\lx@spnewtheorem{#1}[#2]{\csname #1name\endcsname}{#4}{#5}".to_string()
  };
  RawTeX!(&s!(r"\def\@spothm#1[#2]#3#4#5{{\@ifundefined{{c@#2}}{{\@latexerr{{No theorem environment `#2' defined}}\@eha}}{{\expandafter\@ifdefinable\csname #1\endcsname{{{define}}}}}}}"));
  RawTeX!(r"\if@envcntsame\def\spn@wtheorem#1#2#3#4{\@spothm{#1}[theorem]{#2}{#3}{#4}}
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
  // load pre-defines none of them. The separator between a within-counter's number and its own, `\@thmcountersep`,
  // is the class's (llncs.cls:1058, :1141; every one of 613 census copies): empty, "." only under envcountsect, so a
  // paper's own `\newtheorem{x}{X}[section]` prints "11", "A1", as pdflatex (KNOWN_PERL_ERRORS #1111). Guard
  // `perfect_kernel_batch78q::theorem_counter_separator_is_the_class_s`.
  RawTeX!(r"\def\@thmcountersep{}\if@envcntsect\def\@thmcountersep{.}\fi");
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

/// `body` without its top-level `\institute{…}` calls, and the calls.
fn take_institutes(body: &Tokens) -> (Tokens, Vec<Token>) {
  let v = body.unlist_ref();
  let (mut names, mut institutes) = (Vec::with_capacity(v.len()), Vec::new());
  let (mut at, mut depth) = (0usize, 0usize);
  while at < v.len() {
    let t = v[at];
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    if depth == 0 && t == T_CS!("\\institute") {
      let mut arg = at + 1;
      while v.get(arg) == Some(&T_SPACE!()) {
        arg += 1;
      }
      if v
        .get(arg)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      {
        let mut end = arg;
        let mut inner = 0usize;
        while end < v.len() {
          match v[end].get_catcode() {
            Catcode::BEGIN => inner += 1,
            Catcode::END => {
              inner -= 1;
              if inner == 0 {
                break;
              }
            },
            _ => {},
          }
          end += 1;
        }
        if end < v.len() {
          institutes.extend_from_slice(&v[at..=end]);
          at = end + 1;
          continue;
        }
      }
    }
    names.push(t);
    at += 1;
  }
  (Tokens::new(names), institutes)
}
