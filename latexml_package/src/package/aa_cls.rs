use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::{
  prelude::*,
  shipped_class::{class_text_loads, shipped_class_loads_or, shipped_class_text},
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aa.cls.ltxml — Astronomy & Astrophysics
  // aa.cls v5-v7 (1201.4282's v7.0 :85-88): `\if@referee` and `\if@onecolumn`, which the `referee` and
  // `onecolumn` options set and papers test in their preambles (1201.4282, 1208.0304, 1501.05323, 1605.00944;
  // 1608.08833). v8 and later name them `\ifaa@referee`/`\ifaa@twocolumn`; the old names stay for every version
  // (one binding, the union).
  RawTeX!(r"\let\if@referee\iffalse\let\if@onecolumn\iffalse");
  DeclareOption!("referee", "\\let\\if@referee\\iftrue");
  DeclareOption!("onecolumn", "\\let\\if@onecolumn\\iftrue");
  // Ignorable options
  for option in ["10pt", "11pt", "12pt", "twoside", "twocolumn",
    "draft", "final", "longauth", "rnote", "oldversion",
    "runningheads", "envcountreset", "envcountsect",
    "structabstract", "traditabstract", "letter"].iter()
  {
    DeclareOption!(*option, None);
  }
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("aa_support");
  // Raw aa.cls unconditionally loads natbib (not just for bibnumber/bibauthoryear options).
  // Many aa papers use \citep/\citet without explicit \usepackage{natbib}.
  RequirePackage!("natbib");
  // Perl overrides \pmatrix and \cases with the plain TeX versions (TeX.pool), for the old aa.cls that loaded no amsmath
  // (astro-ph/0002145 `\pmatrix{…}`). Today's aa.cls loads it (v9.4, :205 `\RequirePackage[tbtags,fleqn]{amsmath}`),
  // whose {pmatrix}/{cases} start with `\matrix@check` (amsmath.sty:1077-1082): the environment when `\@currenvir` names
  // it, the old Plain-TeX form otherwise. Perl's plain-only override read `\begin{cases}`'s first cell as its argument,
  // and the `&`s after it went stray (2609.02726, 2609.04318: Perl errors the same way).
  // amsmath's {matrix} checks alike (amsmath.sty:1068-1071 `\matrix@check\matrix`): the old aa.cls and l-aa.sty (the
  // `l-aa` binding loads this class) give plain TeX's `\matrix{…\cr}` (plain.tex:1093, latex.ltx:15634), whose
  // amsmath reading opened an alignment that nothing closed, the display running into the next section
  // (astro-ph9609122, astro-ph9809009, astro-ph9503122).
  DefMacro!("\\matrix",
    r"\expandafter\ifx\csname\@currenvir\endcsname\matrix\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\lx@ams@matrix{name=matrix,datameaning=matrix}}
{\lx@gen@plain@matrix{name=matrix,datameaning=matrix}}");
  DefMacro!("\\pmatrix",
    r"\expandafter\ifx\csname\@currenvir\endcsname\pmatrix\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\lx@ams@matrix{name=pmatrix,datameaning=matrix,left=\lx@left(,right=\lx@right)}}
{\lx@gen@plain@matrix{name=pmatrix,datameaning=matrix,left=\lx@left(,right=\lx@right)}}");
  DefMacro!("\\cases",
    r"\expandafter\ifx\csname\@currenvir\endcsname\cases\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\lx@ams@cases{name=cases,meaning=cases,left=\lx@left\{}}
{\lx@gen@plain@cases{meaning=cases,left=\lx@left\{,conditionmode=text,style=\textstyle}}");
  // Perl aa.cls.ltxml L57 resets the amsmath loaded-flag so a document's own `\usepackage{amsmath}` runs the binding
  // again (arXiv:astro-ph/0203101, for its {cases}); kept for Perl parity — the environment forms above no longer
  // need it, and a reload replaces the old-form `\pmatrix{…}`/`\cases{…}` fallback with amsmath's.
  AssignValue!("amsmath.sty_loaded" => Stored::None, Some(Scope::Global));

  // aa.cls L1651-1664: \tablebib{...} / \tablefoot{...} emit a labeled
  // table-notes block below the tabular, inside the table float. Perl
  // (aa_support.sty.ltxml L229) renders `\tablefoot` as a plain `\footnote`
  // → `ltx:note` (Meta.class, valid in a table float, and — crucially —
  // able to hold the *multi-paragraph* note bodies A&A papers write, where a
  // blank line inside `\tablefoot{...}` becomes a `\par`).
  //
  // A bare `\par\textbf{Notes.} #1` (the old form, witnesses 2406.05044 /
  // 2406.14661) works only for single-paragraph notes: a `\par` *inside*
  // `#1` builds an `ltx:para`, which `table_model` forbids
  // (`ltx:para isn't allowed in ltx:table`; witness 1701.02312, whose
  // `\tablefoot` has a blank-line `\par`). Route through `\footnote` like
  // Perl — but keep the visible "Notes."/"References." label (real aa.cls
  // shows it) by prefixing it *inside* the note, where block content is legal.
  DefMacro!("\\tablebib{}", "\\footnote{\\textbf{References.} #1}");
  DefMacro!("\\tablefoot{}", "\\footnote{\\textbf{Notes.} #1}");
  DefMacro!("\\tablebibname", "References");
  DefMacro!("\\tablefootname", "Notes");
  // A&A authors use \orcid for the ORCID identifier. Route through the kernel
  // `\lx@add@orcid` → `ltx:contact[role=orcid]` → clickable orcid.org link + logo
  // icon (vs a bare dagger note). html_feedback#6571.
  DefMacro!("\\orcid{}", "\\lx@add@orcid{#1}");

  // Real aa.cls ends with `\RequirePackage[modulo,mathlines,switch,running,
  // columnwise,pagewise]{linenoaa}` (the A&A-bundled lineno variant), so
  // documents call \nolinenumbers / \linenumbers freely (witness
  // 2605.00223 line 123). Our lineno binding stubs the full macro set —
  // require it (options irrelevant to the XML stubs). Perl's aa.cls.ltxml
  // lacks this (\nolinenumbers undefined there); candidate to upstream.
  RequirePackage!("lineno");
  // aa.cls:249 (1501.03426's copy; :254 in 1504.01878's) `\RequirePackage{answers}` in the v8-era copies (11 of 59
  // sampled; today's v9.x loads none, nor does a paper without a copy): answers loads verbatim (answers.sty:32), whose
  // `{comment}` those papers use without loading it (23 of run 353 with `{comment}` undefined; 1501.03426, 1504.01878,
  // 1507.00595). answers has no binding and loads raw; the paper's copy decides.
  if shipped_class_loads_or("aa", "answers", false)? {
    RequirePackage!("answers");
  }

  // aa.cls's online-only floats, which the class sets at `\AtEndDocument` under an "Online Material" page (`\Online`,
  // page furniture, empty here as in Perl's aa_support). The versions differ in arity: up to v7 (41 of 98 sampled
  // copies; 0801.4139's :931-940) `\onltab{N}{table}`, `\onlfig{N}{figure}`, `\onllongtab{N}{longtable}` and
  // `\onllongtabL` defer the float numbered N and step the counter to N for what follows, and the landscape
  // `\longtabL{N}{longtable}` sets it in place, numbered N; v8.x (24 copies; 1403.1122's v8.1 :1366-1395) `\onltab
  // {table}`, `\onlfig{figure}`, `\onllongtab{longtable}` (`\onlfloat`: the float deferred, keeping the number it
  // reserves where it is called); v9.x defines none. A first argument that is a number is the older form's N. Run 340
  // first errors: `\onltab` 171, `\onlfig` 152, `\onllongtab` 111, `\longtabL` 49 (witnesses 0709.2515, 0709.0404,
  // 0709.4508, 0811.2397, 1403.1122, 1505.04356).
  DefMacro!("\\lx@aa@onl{}{}{}", sub[(mode, counter, first)] {
    let text = first.clone().untex();
    let text = text.trim();
    let numbered = !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    let deferred = mode.untex() == "d";
    Ok(match (numbered, deferred) {
      (true, true) => Invocation!(T_CS!("\\lx@aa@onl@numbered"), vec![Some(counter), Some(first)]),
      (true, false) => Invocation!(T_CS!("\\lx@aa@onl@inplace"), vec![Some(counter), Some(first)]),
      (false, true) => Invocation!(T_CS!("\\lx@aa@onl@float"), vec![Some(counter), Some(first)]),
      (false, false) => first,
    })
  });
  RawTeX!(r"\long\def\lx@aa@onl@numbered#1#2#3{\AtEndDocument{\setcounter{#1}{#2}\addtocounter{#1}{-1}#3}\setcounter{#1}{#2}}
\long\def\lx@aa@onl@inplace#1#2#3{\setcounter{#1}{#2}\addtocounter{#1}{-1}#3\setcounter{#1}{#2}}
\long\def\lx@aa@onl@float#1#2{\edef\lx@aa@nombre{\the\value{#1}}%
  \expandafter\lx@aa@onl@defer\expandafter{\lx@aa@nombre}{#1}{#2}\stepcounter{#1}}
\long\def\lx@aa@onl@defer#1#2#3{\AtEndDocument{\edef\lx@aa@saved{\the\value{#2}}\setcounter{#2}{#1}#3%
  \setcounter{#2}{\lx@aa@saved}}}
\def\onltab{\lx@aa@onl{d}{table}}\def\onlfig{\lx@aa@onl{d}{figure}}\def\onllongtab{\lx@aa@onl{d}{table}}
\def\onllongtabL{\lx@aa@onl{d}{table}}\def\longtabL{\lx@aa@onl{p}{table}}");
  // v9.x: `\AANum{N}` (56 copies), the article number the page furniture prints; `\psj` (the Planetary Science Journal,
  // `PSJ` in 4 of the 6 copies with it, 9 of 98 in all: a guess a paper's own definition replaces; 44 first errors,
  // 2508.13727).
  RawTeX!(r"\providecommand*\aa@numarticle{}\newcommand*\AANum[1]{\renewcommand*\aa@numarticle{#1}}");
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r"\providecommand*\psj{PSJ}");
  }
  // aa.cls v8.2-v9.4 (2509.20613's v9.1 :2058-2062): `\arxiv{id}`, the arXiv identifier of a reference, in typewriter
  // type after the prefix "arXiv:" (1604.01307, 1809.05581, 2103.09505, 2307.13476, 2503.05262, 2503.23114). Perl's
  // aa_support.sty.ltxml has `\eprint` but no `\arxiv` (KNOWN_PERL_ERRORS #740).
  // v9.1 (2509.20613's copy): the language editor's inline note `\LEt{…}`, a red bold "[Note N: …]" numbered by the
  // `lenote` counter (:165, :170-173), defined at the document's start, so a paper's own `\LEt` (1608.08833's) stands
  // (1607.03898); the first-page "Special feature" line `\topics{…}` (:837-838, kept as a note of the frontmatter;
  // 1810.01624); the running head's `\etalname` (:438); the gap below a table caption (:364-365; 2504.01061); the
  // copyright year `\yearCop` (:537, a token register; 2609.03060).
  // `\arxiv` (with its prefixes) and `\etalname` are the newer releases' only (14 and 13 of 139 sampled copies lack
  // them: 1201.4282's v7.0), so, as the five journals below, a paper's own definition wins (`\newcommand\arxiv[1]`:
  // the review's rev68a a1; 1201.4282).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r"\def\arxivprefixe{arXiv}\def\arxivprefixesep{:}\def\arxiv#1{{\tt \arxivprefixe\arxivprefixesep\ignorespaces#1}}
\newcommand*\etalname{et al.}");
  }
  RawTeX!(r"\newcounter{lenote}\AtBeginDocument{\providecommand{\LEt}[1]{\stepcounter{lenote}\normalfont\textbf{\color{red}[Note \arabic{lenote}: #1] }}}
\newlength{\VSpaceAfterTabCaption}\setlength{\VSpaceAfterTabCaption}{10pt}\newtoks\yearCop");
  DefMacro!("\\topics{}", "\\lx@add@frontmatter{ltx:note}[role=topics]{#1}");
  // The A&A language editor's v9.1 copy (7 identical copies, 2008.09582, 2105.01661, 2109.04752, 2209.13449,
  // 2405.13486, 2408.13105, 2509.05225; 2008.09582's :1902-1961) adds five journals to the class's list (aa_support's):
  // provide-only, a paper's own definition wins (2003.12561, 2403.04850, 2406.18661).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r"\providecommand*\adndt{At. Data Nucl. Data Tables}\providecommand*\jpb{J.~Phys.~B}
\providecommand*\jpcrd{J. Phys. Chem. Ref. Data}\providecommand*\raa{Res. Astron. Astrophys.}\providecommand*\zphys{Z.~Phys.}");
  }
  // The v9.x copies load hyperref at the document's start unless the paper has (2409.03418's :2035-2037,
  // `\AtBeginDocument{\@ifpackageloaded{hyperref}{}{\RequirePackage[colorlinks=true,…]{hyperref}}}`, 17 of 98 sampled),
  // so their papers call `\href` (60 first errors, 2406.05179): the paper's own copy decides.
  if shipped_class_text("aa")?.is_some_and(|text| class_text_loads(&text, "hyperref")) {
    RawTeX!(r"\AtBeginDocument{\@ifpackageloaded{hyperref}{}{\RequirePackage{hyperref}}}");
  }
});
