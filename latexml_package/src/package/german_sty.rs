use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: german.sty.ltxml
  // This should be essentially the same, right?
  // (considering we don't do hyphenation, etc)
  RequirePackage!("babel", options => vec!["german".to_string()]);

  // Raw-load germanb.ldf so babel's authoritative \captionsgerman +
  // \extrasgerman are installed from TeX Live source (alongside the
  // provideommand fallback below, in case the raw load is partial).
  // Avoid recursing into this binding — noltxml=true skips the dispatcher.
  InputDefinitions!("germanb", noltxml => true, extension => Some(Cow::Borrowed("ldf")));
  // \providecommand fallback for \captionsgerman: used as a belt-and-
  // suspenders backstop if germanb.ldf's raw load didn't complete in
  // time (observed on some class+package option orderings where the
  // raw load finishes AFTER \select@language fires).
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
