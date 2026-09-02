use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: german.sty.ltxml
  // This should be essentially the same, right?
  // (considering we don't do hyphenation, etc)
  RequirePackage!("babel", options => vec!["german".to_string()]);

  // Alias the `germanb` dialect's language number to `german` (kernel/dump
  // `\l@german`), exactly as the real germanb.ldf does via
  // `\let\l@germanb\l@german`. Without this, `\usepackage[…,germanb]{babel}`
  // selects `germanb` as the main language and babel's
  // `\selectlanguage{germanb}` → `\bbl@iflanguage{germanb}` errors "You haven't
  // defined the language 'germanb' yet" — because this binding REPLACES the raw
  // germanb.ldf load (which is where `\l@germanb` would otherwise come from).
  // Witness: arXiv:1010.4065 (`\usepackage[english,germanb]{babel}`).
  RawTeX!(r"\expandafter\ifx\csname l@germanb\endcsname\relax
    \expandafter\ifx\csname l@german\endcsname\relax
      \expandafter\newlanguage\csname l@germanb\endcsname
    \else
      \expandafter\let\csname l@germanb\expandafter\endcsname\csname l@german\endcsname
    \fi
  \fi");

  // German caption strings (from germanb.ldf). \providecommand so raw
  // babel/germanb.ldf processing (if any) doesn't overwrite.
  RawTeX!(r"\providecommand\captionsgerman{%
    \def\prefacename{Vorwort}\def\refname{Literatur}%
    \def\abstractname{Zusammenfassung}\def\bibname{Literaturverzeichnis}%
    \def\chaptername{Kapitel}\def\appendixname{Anhang}%
    \def\contentsname{Inhaltsverzeichnis}%
    \def\listfigurename{Abbildungsverzeichnis}%
    \def\listtablename{Tabellenverzeichnis}%
    \def\indexname{Index}\def\figurename{Abbildung}%
    \def\tablename{Tabelle}\def\partname{Teil}%
    \def\pagename{Seite}\def\seename{siehe}%
    \def\alsoname{siehe auch}\def\proofname{Beweis}}");
  RawTeX!(r"\providecommand\dategerman{}");
  RawTeX!(r"\providecommand\captionsngerman{\captionsgerman}");
  RawTeX!(r"\providecommand\datengerman{\dategerman}");

  // The `"` shorthand dispatch (`\lx@german@dq@dispatch`), `\mdqon` and
  // `\mdqoff` live in babel_support_sty.rs: they belong to babel's German
  // (every `\usepackage[ngerman]{babel}` document, not only german.sty).
  // germanb.ldf L173: `\def\dq{"}` — `\dq` yields a literal double-quote (the
  // saved catcode-12 `"`, since `"` itself becomes the active shorthand). Map to
  // `\textquotedbl`, LaTeXML's literal double-quote. (An earlier note recorded
  // `\dq` as "actively undefined after this binding runs" — that was while the
  // binding truncated past L57, before `\bbl@allowhyphens` below was added; the
  // truncation is resolved, so this now sticks.)
  RawTeX!(r"\providecommand\dq{\textquotedbl}");
  // germanb.ldf helper stubs — no-op in Rust (no hyphenation / ligature phase).
  RawTeX!(r"\providecommand\bbl@allowhyphens{}");
  RawTeX!(r"\providecommand\bbl@ss{\ss}\providecommand\bbl@SS{SS}");
  RawTeX!(r"\providecommand\bbl@sz{\ss}\providecommand\bbl@SZ{SZ}");
  // german.sty:314-319, 662-671: the user-level switches. `\germanTeX` is
  // what the kernel's first aid (latex2e-first-aid-for-external-files.ltx:160-166,
  // `file/german.sty/after`) runs once our binding reports the file's version
  // — and what documents written for german.sty call themselves (a0poster
  // a0/a0_eng, adrconv, akletter, cryst … 11 TL manuals errored
  // `undefined:\germanTeX`). `\umlautlow`/`\umlauthigh` choose the accent
  // placement of `\"` — a font matter with no XML counterpart.
  RawTeX!(r"\providecommand\umlautlow{}\providecommand\umlauthigh{}");
  RawTeX!(r"\providecommand\germanTeX{\mdqon\selectlanguage{german}}");
});
