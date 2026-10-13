use crate::{
  prelude::*,
  shipped_class::{class_text_loads, shipped_class_text},
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aa.cls.ltxml — Astronomy & Astrophysics
  // Ignorable options
  for option in ["10pt", "11pt", "12pt", "twoside", "onecolumn", "twocolumn",
    "draft", "final", "referee", "longauth", "rnote", "oldversion",
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

  // aa.cls's online-only floats (65 of 98 sampled copies; 0709.2515's :925-941): `\onltab{N}{table}`,
  // `\onlfig{N}{figure}`, `\onllongtab{N}{longtable}`, `\onllongtabL` and the landscape `\longtabL{N}{longtable}`. The
  // class sets the float after the document under an "Online Material" page (`\Online`, empty here as in Perl's
  // aa_support) numbered N, and steps the counter to N for what follows; as `\longtab` (aa_support), the float stays in
  // place, numbered N. Run 340 first errors: `\onltab` 171, `\onlfig` 152, `\onllongtab` 111, `\longtabL` 49
  // (witnesses 0709.2515, 0709.0404, 0709.4508, 0811.2397).
  RawTeX!(r"\long\def\onltab#1#2{\setcounter{table}{#1}\addtocounter{table}{-1}#2\setcounter{table}{#1}}
\long\def\onlfig#1#2{\setcounter{figure}{#1}\addtocounter{figure}{-1}#2\setcounter{figure}{#1}}
\let\onllongtab\onltab \let\onllongtabL\onltab \let\longtabL\onltab");
  // v9.x: `\AANum{N}` (56 copies), the article number the page furniture prints; `\psj` (the Planetary Science Journal,
  // `PSJ` in 4 of the 6 copies with it; 44 first errors, 2508.13727).
  RawTeX!(r"\providecommand*\aa@numarticle{}\newcommand*\AANum[1]{\renewcommand*\aa@numarticle{#1}}
\providecommand*\psj{PSJ}");
  // The v9.x copies load hyperref unless the paper has (`\@ifpackageloaded{hyperref}{}{\RequirePackage[colorlinks=true,
  // …]{hyperref}}`, 17 of 98 sampled), so their papers call `\href` (60 first errors, 2406.05179): the paper's own copy
  // decides.
  if shipped_class_text("aa")?.is_some_and(|text| class_text_loads(&text, "hyperref")) {
    RequirePackage!("hyperref");
  }
});
