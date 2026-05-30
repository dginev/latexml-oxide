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

  // German " shorthand dispatch (from germanb.ldf). We replace germanb.ldf
  // entirely via our binding dispatcher, so babel's \initiate@active@char
  // + \declare@shorthand{german}{"a}{...} calls in germanb.ldf never fire.
  // Simpler to implement the dispatch here as a native primitive that
  // reads the next token and emits the umlaut/ß/guillemet directly. A
  // future refactor could load germanb.ldf raw in parallel (the
  // \initiate@active@char machinery does work in our engine now — verified
  // 2026-04-17) and drop this custom primitive.
  DefPrimitive!("\\lx@german@dq@dispatch", {
    let tok = gullet::read_token()?;
    let ch = tok.as_ref().map(|t| t.with_str(|s| s.to_string())).unwrap_or_default();
    let expansion: &str = match ch.as_str() {
      "a" => "\u{00E4}", "o" => "\u{00F6}", "u" => "\u{00FC}",
      "e" => "\u{00EB}", "i" => "\u{00EF}",
      "A" => "\u{00C4}", "O" => "\u{00D6}", "U" => "\u{00DC}",
      "E" => "\u{00CB}", "I" => "\u{00CF}",
      "s" | "z" => "\u{00DF}",
      "S" => "SS", "Z" => "SZ",
      "`" => "\u{201E}", "'" => "\u{201C}",
      "<" => "\u{00AB}", ">" => "\u{00BB}",
      "~" => "-", "=" => "-",
      // consonants/unknowns: pass-through (below)
      _ => "",
    };
    if !expansion.is_empty() {
      gullet::unread(Tokenize!(expansion));
    } else if !ch.is_empty() {
      if let Some(t) = tok { gullet::unread(Tokens!(t)); }
    }
  });
  DefPrimitive!("\\mdqon", { state::assign_catcode('"', Catcode::ACTIVE, None); });
  DefPrimitive!("\\mdqoff", { state::assign_catcode('"', Catcode::OTHER, None); });
  // germanb.ldf helper stubs — no-op in Rust (no hyphenation / ligature phase).
  RawTeX!(r"\providecommand\bbl@allowhyphens{}");
  RawTeX!(r"\providecommand\bbl@ss{\ss}\providecommand\bbl@SS{SS}");
  RawTeX!(r"\providecommand\bbl@sz{\ss}\providecommand\bbl@SZ{SZ}");
});
