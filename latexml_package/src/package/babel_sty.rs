//! babel.sty — multilingual support
//! Perl: babel.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Pre-define language registers (\l@german, \l@french, etc.) so babel's
  // \bbl@iflanguage check passes. Without these, \selectlanguage skips the
  // \captionsgerman/\bbl@switch call because the language isn't "recognized".
  // In Perl's precompiled kernel, these come from language.def hyphenation loading.
  RawTeX!(r"\expandafter\ifx\csname l@english\endcsname\relax\chardef\l@english=0\fi");
  RawTeX!(r"\expandafter\ifx\csname l@german\endcsname\relax\newlanguage\l@german\fi");
  RawTeX!(r"\expandafter\ifx\csname l@ngerman\endcsname\relax\newlanguage\l@ngerman\fi");
  RawTeX!(r"\expandafter\ifx\csname l@french\endcsname\relax\newlanguage\l@french\fi");
  RawTeX!(r"\expandafter\ifx\csname l@spanish\endcsname\relax\newlanguage\l@spanish\fi");
  RawTeX!(r"\expandafter\ifx\csname l@italian\endcsname\relax\newlanguage\l@italian\fi");
  RawTeX!(r"\expandafter\ifx\csname l@portuguese\endcsname\relax\newlanguage\l@portuguese\fi");
  RawTeX!(r"\expandafter\ifx\csname l@russian\endcsname\relax\newlanguage\l@russian\fi");
  RawTeX!(r"\expandafter\ifx\csname l@greek\endcsname\relax\newlanguage\l@greek\fi");
  RawTeX!(r"\expandafter\ifx\csname l@dutch\endcsname\relax\newlanguage\l@dutch\fi");
  RawTeX!(r"\expandafter\ifx\csname l@polish\endcsname\relax\newlanguage\l@polish\fi");
  RawTeX!(r"\expandafter\ifx\csname l@turkish\endcsname\relax\newlanguage\l@turkish\fi");
  RawTeX!(r"\expandafter\ifx\csname l@czech\endcsname\relax\newlanguage\l@czech\fi");

  // Pre-define \bbl@languages as empty. Babel's language.def loading uses \openin
  // to read hyphenation pattern files, which our system can't find (.ini search paths).
  // Without this, \bbl@languages stays undefined, and our error recovery defines it
  // as <ltx:ERROR/> which corrupts babel's list accumulation → infinite recursion → OOM.
  RawTeX!(r"\def\bbl@languages{}");

  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // After babel loads, clear \@fontenc@load@list to prevent stray commas
  // from babel's AtBeginDocument font encoding iteration.
  RawTeX!(r"\def\@fontenc@load@list{}");

  // Fix \bbl@main@language: babel's option processing sets it to "nil"
  // because the ini-based loading path doesn't call \main@language.
  // Read the last option from \@raw@classoptionslist or babel's internal
  // state and call \main@language explicitly.
  // Fix \bbl@main@language: use \bbl@loaded (set by babel option processing)
  // to call \main@language with the correct language name.
  DefPrimitive!("\\lx@babel@fix@mainlang", {
    let main = gullet::do_expand(T_CS!("\\bbl@main@language"))
      .map(|t| t.to_string()).unwrap_or_default();
    if main == "nil" || main.is_empty() {
      // \bbl@loaded contains the last loaded language
      let loaded = gullet::do_expand(T_CS!("\\bbl@loaded"))
        .map(|t| t.to_string()).unwrap_or_default();
      let last_lang = loaded.split(',').map(|s| s.trim())
        .filter(|s| !s.is_empty()).last().unwrap_or("").to_string();
      if !last_lang.is_empty() {
        gullet::unread(Tokenize!(&s!("\\main@language{{{}}}", last_lang)));
      }
    }
  });
  RawTeX!(r"\lx@babel@fix@mainlang");

  // Emulate Perl's precompiled kernel: pre-define \captions<lang> and \date<lang>
  // for common languages. In Perl, `make formats` precompiles the kernel so these
  // macros exist. Without them, babel's \bbl@provide@locale calls \babelprovide
  // which loads .ini files — a path that our engine can't handle (multiple undefined
  // macros hit error recovery → <ltx:ERROR/> corruption → OOM).
  // Pre-defining makes \bbl@provide@locale skip the heavy \babelprovide path.
  RawTeX!(r"\providecommand\captionsenglish{}\providecommand\dateenglish{}");
  RawTeX!(r"\providecommand\captionsfrench{}\providecommand\datefrench{}");
  // German captions (from germanb.ldf) — not empty, actual text.
  // Avoids OOM from \babelprovide AND provides correct localization.
  DefPrimitive!("\\lx@babel@setup@german", {
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
  });
  RawTeX!(r"\lx@babel@setup@german");
  RawTeX!(r"\providecommand\captionsspanish{}\providecommand\datespanish{}");
  RawTeX!(r"\providecommand\captionsitalian{}\providecommand\dateitalian{}");
  RawTeX!(r"\providecommand\captionsportuges{}\providecommand\dateportuges{}");
  RawTeX!(r"\providecommand\captionsbrazilian{}\providecommand\datebrazilian{}");
  RawTeX!(r"\providecommand\captionsrussian{}\providecommand\daterussian{}");
  RawTeX!(r"\providecommand\captionspolish{}\providecommand\datepolish{}");
  RawTeX!(r"\providecommand\captionsdutch{}\providecommand\datedutch{}");
  RawTeX!(r"\providecommand\captionsczech{}\providecommand\dateczech{}");
  RawTeX!(r"\providecommand\captionsgreek{}\providecommand\dategreek{}");
  RawTeX!(r"\providecommand\captionsturkish{}\providecommand\dateturkish{}");
  RawTeX!(r"\providecommand\captionshungarian{}\providecommand\datehungarian{}");
  RawTeX!(r"\providecommand\captionsswedish{}\providecommand\dateswedish{}");
  RawTeX!(r"\providecommand\captionsdanish{}\providecommand\datedanish{}");
  RawTeX!(r"\providecommand\captionsfinnish{}\providecommand\datefinnish{}");
  RawTeX!(r"\providecommand\captionsnorsk{}\providecommand\datenorsk{}");
  RawTeX!(r"\providecommand\captionsromanian{}\providecommand\dateromanian{}");
  RawTeX!(r"\providecommand\captionscroatian{}\providecommand\datecroatian{}");
  RawTeX!(r"\providecommand\captionsbulgarian{}\providecommand\datebulgarian{}");
  RawTeX!(r"\providecommand\captionsamerican{}\providecommand\dateamerican{}");
  RawTeX!(r"\providecommand\captionsbritish{}\providecommand\datebritish{}");
  RawTeX!(r"\providecommand\captionsaustrian{}\providecommand\dateaustrian{}");
  RawTeX!(r"\providecommand\captionsnaustrian{}\providecommand\datenaustrian{}");
  RawTeX!(r"\providecommand\captionsfrancais{}\providecommand\datefrancais{}");

  // After babel loads: activate the main language's captions and shorthands.
  // In Perl this happens via the precompiled kernel; we do it explicitly.
  DefPrimitive!("\\lx@babel@activate@mainlang", {
    let main = gullet::do_expand(T_CS!("\\bbl@main@language"))
      .map(|t| t.to_string()).unwrap_or_default();
    let loaded = gullet::do_expand(T_CS!("\\bbl@loaded"))
      .map(|t| t.to_string()).unwrap_or_default();
    let lang = if main == "nil" || main.is_empty() {
      loaded.split(',').map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()).last().unwrap_or_default()
    } else {
      main
    };
    if !lang.is_empty() {
      // Call \captions<lang> to set localized names
      let cs = s!("\\captions{}", lang);
      if lookup_definition(&T_CS!(cs.clone()))?.is_some() {
        gullet::unread(Tokenize!(&cs));
      }
      // Call \ltx@bbl@select@language to set xml:lang
      gullet::unread(Tokenize!(&s!("\\ltx@bbl@select@language{{{}}}", lang)));
    }
  });
  RawTeX!(r"\AtBeginDocument{\lx@babel@activate@mainlang}");

  // German " shorthand system (from germanb.ldf).
  // babel's \initiate@active@char mechanism often fails during raw loading;
  // implement the dispatch entirely in Rust as a Primitive that reads the
  // next token and expands to the appropriate shorthand.
  DefPrimitive!("\\lx@german@dq@dispatch", {
    // Read the next token (the character after ")
    let tok = gullet::read_token()?;
    let ch = tok.as_ref().map(|t| t.with_str(|s| s.to_string())).unwrap_or_default();
    // Map shorthand to expansion. Use Unicode directly for umlauts to avoid
    // catcode issues with active " interfering with \" command.
    let expansion: &str = match ch.as_str() {
      "a" => "\u{00E4}", "o" => "\u{00F6}", "u" => "\u{00FC}",
      "e" => "\u{00EB}", "i" => "\u{00EF}",
      "A" => "\u{00C4}", "O" => "\u{00D6}", "U" => "\u{00DC}",
      "E" => "\u{00CB}", "I" => "\u{00CF}",
      "s" | "z" => "\u{00DF}", // ß
      "S" => "SS", "Z" => "SZ",
      "`" => "\u{201E}", // „ (German opening quote)
      "'" => "\u{201C}", // " (German closing quote)
      "<" => "\u{00AB}", // «
      ">" => "\u{00BB}", // »
      "~" => "-", "=" => "-",
      // "" → empty (hskip) — handled below via empty check
      // Consonant shorthands: discretionary hyphens, ignored in LaTeXML
      _ => "",
    };
    if !expansion.is_empty() {
      gullet::unread(Tokenize!(expansion));
    } else if !ch.is_empty() {
      // For consonants (c,f,l,m,n,p,r,t,...) and unknowns: just output the character
      if let Some(t) = tok { gullet::unread(Tokens!(t)); }
    }
  });
  // \mdqoff/\mdqon: toggle " catcode between active (13) and other (12)
  DefPrimitive!("\\mdqon", { state::assign_catcode('"', Catcode::ACTIVE, None); });
  DefPrimitive!("\\mdqoff", { state::assign_catcode('"', Catcode::OTHER, None); });
  // Stubs for babel helper macros used by germanb.ldf
  RawTeX!(r"\providecommand\bbl@allowhyphens{}");
  RawTeX!(r"\providecommand\bbl@ss{\ss}");
  RawTeX!(r"\providecommand\bbl@SS{SS}");
  RawTeX!(r"\providecommand\bbl@sz{\ss}");
  RawTeX!(r"\providecommand\bbl@SZ{SZ}");
  // Activate: make " active and assign it the dispatch primitive's meaning.
  DefPrimitive!("\\lx@babel@setup@german@shorthands", {
    state::assign_catcode('"', Catcode::ACTIVE, Some(Scope::Global));
    let active_dq = T_ACTIVE!('"');
    let dispatch_cs = T_CS!("\\lx@german@dq@dispatch");
    state::assign_meaning(&active_dq, dispatch_cs, Some(Scope::Global));
  });
  // Activate German shorthands if German is the main language
  DefPrimitive!("\\lx@babel@maybe@german@shorthands", {
    let main = gullet::do_expand(T_CS!("\\bbl@main@language"))
      .map(|t| t.to_string()).unwrap_or_default();
    let loaded = gullet::do_expand(T_CS!("\\bbl@loaded"))
      .map(|t| t.to_string()).unwrap_or_default();
    let is_german = main == "german" || main == "ngerman" || main == "germanb"
      || loaded.contains("german");
    if is_german {
      // Make " active and assign the dispatch primitive's definition to it
      state::assign_catcode('"', Catcode::ACTIVE, Some(Scope::Global));
      let active_dq = T_ACTIVE!('"');
      let dispatch_cs = T_CS!("\\lx@german@dq@dispatch");
      // Look up the actual definition and assign it directly
      if let Some(defn) = lookup_meaning(&dispatch_cs) {
        state::assign_meaning(&active_dq, defn, Some(Scope::Global));
      }
      // Directly invoke \captionsgerman and set xml:lang
      stomach::digest(Tokenize!(r"\captionsgerman"))?;
      // Set DOCUMENT_LANGUAGE directly (bypasses tokenization issues with @ catcode)
      let lang_name = loaded.split(',').map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()).last().unwrap_or_default();
      let iso = match lang_name.as_str() {
        "german" | "germanb" | "ngerman" | "ngermanb" => Some("de"),
        "french" | "francais" | "frenchb" => Some("fr"),
        "spanish" => Some("es"),
        "italian" => Some("it"),
        "english" => Some("en"),
        "american" | "USenglish" => Some("en-US"),
        "british" | "UKenglish" => Some("en-GB"),
        "portuguese" | "portuges" => Some("pt"),
        "russian" | "russianb" => Some("ru"),
        _ => None,
      };
      if let Some(code) = iso {
        state::assign_value("DOCUMENT_LANGUAGE", Stored::from(code.to_string()), Some(Scope::Global));
        // Also merge font language for text-level xml:lang
        let mut font = Font::default();
        font.language = Some(Cow::Owned(code.to_string()));
        merge_font(font);
      }
    }
  });
  RawTeX!(r"\lx@babel@maybe@german@shorthands");
});
