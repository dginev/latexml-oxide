use crate::prelude::*;

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
});
