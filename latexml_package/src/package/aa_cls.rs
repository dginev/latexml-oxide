use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::{
  prelude::*,
  shipped_class::{class_text_loads, shipped_class_loads_or, shipped_class_text},
};

/// This binding's frontmatter conventions (the observer's, OXIDIZED_DESIGN_DIVERGENCES #837), declared at its load
/// (`declare_frontmatter_conventions`, user ruling 2026-10-28: each binding declares its own):
/// aa: an `\inst` after a comma is the name's before it; `\email` prints in place (aa.cls:824 `e-mail: {\aa@emailfont
/// #1}`), so `\offprints{O. Kochukhov, \email{…}}` names its address's owner while `\email` is the class's own
/// (aa_support.sty's; read when the note is rehearsed, at the flush, `\maketitle`, `ledger::email_meaning`: 0903.3512;
/// aa.cls:910-911's `\protected@edef` expands it at the call, so a redefinition between the two keeps the note's text)
/// (This binding's own `\author`, the kernel's replacing one, keys aa's blocks `aa.cls`, OD #922.) `written`: the
/// observer's writer writes aa's creators (FM stage 7, OD #922; the writer declines a paper whose reading it cannot
/// apply, which keeps today's output).
pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {
  name: "aa",
  binding: "aa.cls",
  inst_after_comma: true,
  email_in_place: true,
  written: true,
  cite: "aa.cls:759-760 `\\def\\inst#1{\\unskip\\hbox{$^{#1}$}}`, :824; OD #807 (1604.07826, 2309.00573), #913",
  ..FM_KERNEL_CONVENTIONS
};

#[rustfmt::skip]
LoadDefinitions!({
  // (this binding's frontmatter conventions, first: OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS);
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
  // The page a float's panels are set in rows against (the panel grid rule, `declare_panel_page`): aa.cls's, as
  // pdflatex gives it — 523.531pt in two 256.075pt columns, one column under `onecolumn`. Witness 1510.07016.
  declare_panel_page(256.0748, 523.5307, last_class_option(&["onecolumn", "twocolumn"]) != Some("onecolumn"));
  RequirePackage!("aa_support");
  // aa.cls defines no `\author`: the kernel's `\def\author#1{\gdef\@author{#1}}` keeps the last call's names, so this
  // binding's `\author` clears the authors queued before it, then splits as inst_support's (its `[marks]` and its
  // `\and`/`\And` pieces, OD #53) — the class's own, said with the API's primitives (OXIDIZED_DESIGN #920, #922; Perl
  // inst_support.sty.ltxml:33-35 clears on every call). inst_support's own `\author` keeps stacking, for the classes
  // that reach it through OmniBus. Defining it here makes this binding the owner of aa's `\author`, so its blocks are
  // keyed `aa.cls` and read by this binding's conventions (`ledger::note_binding_load`). The 16 aa census papers with
  // two calls (pop92b6: their second calls in comments or conditionals) change nothing.
  DefMacro!("\\author[]{}",
    "\\lx@clear@creators[role=author]\\lx@splitting{\\lx@add@author@split}{\\and\\And}{#2}\\lx@author@trailing");
  // aa.cls:736-757 counts the `\institute`'s pieces at its `\and`s (`\aa@nbinstitutes`), and `\aainstitutename`
  // (:886-899) prints a sole institute unnumbered under the whole author row (`\ifnum\aa@nbinstitutes=1
  // \noindent\ignorespaces`), several each numbered: the sole one is every author's unless the class's own `\inst{n}`
  // picks it, never by a mark typed by hand — `_share=authors`, as llncs's (llncs_cls.rs `\lx@llncs@institute@sole`,
  // OXIDIZED_DESIGN_DIVERGENCES #901, #924; `Ann Able$^{1,2}$` under `\institute{{1} Univ A \\ {2} Univ B}` is the
  // block's, astro-ph0305539 — the reading had given its four institutes to J. Daszyńska-Daszkiewicz, review rev92c
  // B3). Several institutes split as inst_support's (OD #53); an empty `\institute{}` adds none.
  DefMacro!("\\institute{}",
    "\\lx@clear@frontmatter{ltx:contact}[role=affiliation]\\lx@aa@institute@dispatch{#1}");
  DefMacro!("\\lx@aa@institute@dispatch{}", sub[(body)] {
    let mut depth = 0i32;
    let mut pieces = 1usize;
    for t in body.unlist_ref() {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        _ if depth == 0 && *t == T_CS!("\\and") => pieces += 1,
        _ => {},
      }
    }
    let ink = body.unlist_ref().iter().any(|t| t.get_catcode() != Catcode::SPACE);
    let target = if pieces == 1 && ink {
      T_CS!("\\lx@aa@institute@sole")
    } else {
      T_CS!("\\lx@aa@institute@split")
    };
    Ok(Invocation!(target, vec![Some(body)]))
  });
  DefMacro!("\\lx@aa@institute@split{}",
    "\\lx@splitting{\\lx@add@contact[role=affiliation,labelseq=affiliation]}{\\and\\And}{#1}");
  // (its `\email`s are read as llncs's are — aa_support's `\email` the kernel's `\lx@add@email`, a class e-mail command
  // the shared row parts its lines by: `\institute{Univ A, City A, \email{ann@a.org}}` gives the address to the author
  // it spells and the institute to every author; both had been the block's, 1312.1871, 2310.16880 — review rev92c r2
  // B-2. Its content is not llncs's `\lx@affiliation@withinst`, whose `\inst{n}` sets the entry's label: aa's `\inst`
  // there is the request it is everywhere, `\institute{\inst{1} Univ X\\ \inst{2} Univ Y}` cut at its marks, 1003.3020)
  DefMacro!("\\lx@aa@institute@sole{}",
    "\\lx@add@contact[role=affiliation,labelseq=affiliation,_share=authors]{#1}");
  // Raw aa.cls unconditionally loads natbib (not just for bibnumber/bibauthoryear options).
  // Many aa papers use \citep/\citet without explicit \usepackage{natbib}.
  RequirePackage!("natbib");
  // Perl overrides \pmatrix and \cases with the plain TeX versions (TeX.pool), for the old aa.cls that loaded no amsmath
  // (astro-ph/0002145 `\pmatrix{…}`). Today's aa.cls loads it (v9.4, :205 `\RequirePackage[tbtags,fleqn]{amsmath}`),
  // whose {matrix}/{pmatrix}/{cases} start with `\matrix@check` (amsmath.sty:1068-1082): the environment when
  // `\@currenvir` names it, the old Plain-TeX form otherwise — amsmath's binding does both (amsmath_sty.rs
  // `\lx@ams@matrix@check`; the environment forms 2609.02726, 2609.04318; the old `\matrix{…\cr}` of l-aa.sty and the
  // old aa.cls, astro-ph9609122, astro-ph9809009, astro-ph9503122). Only amsmath's "Old form `\pmatrix'" error belongs
  // to the class that loads amsmath: the old aa.cls has none, and its Plain-TeX `\pmatrix{…}` is no error there; a
  // paper's own copy of aa.cls decides, an old one assumed without a copy — aa_support's amsmath is then one loaded
  // beyond the original (`amsmath_sty::amsmath_loaded_beyond_original`, the rule of every such binding). A document's
  // own `\usepackage{amsmath}` restores the error, as pdflatex gives it.
  if !shipped_class_loads_or("aa", "amsmath", false)? {
    amsmath_sty::amsmath_loaded_beyond_original();
  }
  // Perl aa.cls.ltxml L57 resets the amsmath loaded-flag so a document's own `\usepackage{amsmath}` runs the binding
  // again (arXiv:astro-ph/0203101, for its {cases}); kept for Perl parity where the copy loads amsmath too.
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
  // aa.cls's appendix (v9.4, 2607.25052's copy :237-262, the `{appendix}` environment, whose begin code a bare
  // `\appendix` runs, from v6.1; v5.01-v6.0 the `\appendix` command, 0705.1240's :897-919; 372 of 377 copies papers ship
  // alike): the sections "A", subsections "A.1", the equations, figures and tables numbered within each appendix,
  // "A.1" (reset by each appendix section). The kernel's `\appendix` (article_cls.rs `\@appendix`) runs first for the
  // `ltx:appendix` units. (v4.03, 5 copies of 1998, numbers "A1": not followed.) Perl's aa_support.sty.ltxml:397-399
  // has the class's `\theapsection` commented out and leaves the kernel's (astro-ph0609025: equations 25, 26, 29 where
  // the PDF prints A.1, A.2, A.5): KNOWN_PERL_ERRORS #1098. Guard `perfect_kernel_batch78q::appendix_counters_aa`.
  RawTeX!(r"\def\appendix{\@appendix\par
  \setcounter{section}{0}\setcounter{subsection}{0}\setcounter{figure}{0}\setcounter{table}{0}%
  \renewcommand\thesection{\@Alph\c@section}\renewcommand\thesubsection{\@Alph\c@section.\@arabic\c@subsection}%
  \renewcommand\thesubsubsection{\thesubsection.\@arabic\c@subsubsection}%
  \renewcommand\theequation{\@Alph\c@section.\@arabic\c@equation}%
  \renewcommand\thefigure{\@Alph\c@section.\@arabic\c@figure}\renewcommand\thetable{\@Alph\c@section.\@arabic\c@table}%
  \@addtoreset{equation}{section}\@addtoreset{figure}{section}\@addtoreset{table}{section}}");
});
