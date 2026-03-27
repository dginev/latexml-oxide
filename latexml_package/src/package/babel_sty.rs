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
  // English captions (from english.ldf) — must reset names when switching from other languages
  RawTeX!(r"\providecommand\captionsenglish{%
    \def\prefacename{Preface}\def\refname{References}%
    \def\abstractname{Abstract}\def\bibname{Bibliography}%
    \def\chaptername{Chapter}\def\appendixname{Appendix}%
    \def\contentsname{Contents}%
    \def\listfigurename{List of Figures}%
    \def\listtablename{List of Tables}%
    \def\indexname{Index}\def\figurename{Figure}%
    \def\tablename{Table}\def\partname{Part}%
    \def\pagename{Page}\def\seename{see}%
    \def\alsoname{see also}\def\proofname{Proof}}");
  RawTeX!(r"\providecommand\dateenglish{}");
  // French captions (from frenchb.ldf)
  RawTeX!(r"\providecommand\captionsfrench{%
    \def\prefacename{Pr\'eface}\def\refname{R\'ef\'erences}%
    \def\abstractname{R\'esum\'e}\def\bibname{Bibliographie}%
    \def\chaptername{Chapitre}\def\appendixname{Annexe}%
    \def\contentsname{Table des mati\`eres}%
    \def\listfigurename{Table des figures}%
    \def\listtablename{Liste des tableaux}%
    \def\indexname{Index}\def\figurename{Figure}%
    \def\tablename{Table}\def\partname{partie}%
    \def\pagename{page}\def\seename{voir}%
    \def\alsoname{voir aussi}\def\proofname{D\'emonstration}}");
  RawTeX!(r"\providecommand\datefrench{}");
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
    // Also check \bbl@cl@<lang> which babel sets for each language option
    // This catches languages from \usepackage[lang]{babel} even when babel
    // was already loaded from a class option.
    let opt_list = gullet::do_expand(T_CS!("\\@raw@classoptionslist"))
      .map(|t| t.to_string()).unwrap_or_default();
    let lang = if main != "nil" && !main.is_empty() {
      main
    } else if loaded.contains(',') || loaded.len() > 2 {
      // Multiple languages loaded: last one is main
      loaded.split(',').map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()).last().unwrap_or_default()
    } else if !loaded.is_empty() && loaded != "nil" {
      loaded
    } else {
      // Fallback: use class options
      opt_list.split(',').map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()).last().unwrap_or_default()
    };
    if !lang.is_empty() {
      // Temporarily set @ to LETTER for CS name tokenization
      // (at \begin{document} time, @ is OTHER which breaks \captions<lang>)
      state::assign_catcode('@', Catcode::LETTER, None);
      // Call \captions<lang> to set localized names
      let cs = s!("\\captions{}", lang);
      if lookup_definition(&T_CS!(cs.clone()))?.is_some() {
        stomach::digest(Tokenize!(&cs))?;
      }
      // Call \ltx@bbl@select@language to set xml:lang
      let select_cs = s!("\\ltx@bbl@select@language{{{}}}", lang);
      stomach::digest(Tokenize!(&select_cs))?;
      // Restore @ to OTHER
      state::assign_catcode('@', Catcode::OTHER, None);

      // French active punctuation: make :;!? insert thin space BEFORE the char.
      // Safe to activate here since we're at \begin{document} time (all packages loaded).
      let is_french = lang == "french" || lang == "francais" || lang == "frenchb";
      if is_french {
        // Define dispatch primitives for French punctuation (if not already defined)
        if lookup_definition(&T_CS!("\\lx@french@punct@colon"))?.is_none() {
          // These are defined as Primitives that output thin_space + char
          // U+2006 = SIX-PER-EM SPACE (matches Perl's thin space output)
          DefPrimitive!("\\lx@french@punct@colon", {
            enter_horizontal();
            Tbox::new(arena::pin_static("\u{2006}:"), None, None, Tokens!(), stored_map!())
          });
          DefPrimitive!("\\lx@french@punct@semi", {
            enter_horizontal();
            Tbox::new(arena::pin_static("\u{2006};"), None, None, Tokens!(), stored_map!())
          });
          DefPrimitive!("\\lx@french@punct@exclam", {
            enter_horizontal();
            Tbox::new(arena::pin_static("\u{2006}!"), None, None, Tokens!(), stored_map!())
          });
          DefPrimitive!("\\lx@french@punct@question", {
            enter_horizontal();
            Tbox::new(arena::pin_static("\u{2006}?"), None, None, Tokens!(), stored_map!())
          });
        }
        for &(ch, cs_name) in &[
          (':', "\\lx@french@punct@colon"),
          (';', "\\lx@french@punct@semi"),
          ('!', "\\lx@french@punct@exclam"),
          ('?', "\\lx@french@punct@question"),
        ] {
          state::assign_catcode(ch, Catcode::ACTIVE, Some(Scope::Global));
          let active_tok = T_ACTIVE!(ch);
          let dispatch_cs = T_CS!(cs_name);
          if let Some(defn) = lookup_meaning(&dispatch_cs) {
            state::assign_meaning(&active_tok, defn, Some(Scope::Global));
          }
        }
      }
    }
  });
  // Register activation in @at@begin@document (fires at \begin{document}).
  // Note: babel's own AtBeginDocument code (~700 tokens) includes
  // \selectlanguage{\bbl@main@language} which must run first.
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
  // Activate the main language's captions, shorthands, and xml:lang
  DefPrimitive!("\\lx@babel@activate@lang@post", {
    let main = gullet::do_expand(T_CS!("\\bbl@main@language"))
      .map(|t| t.to_string()).unwrap_or_default();
    let loaded = gullet::do_expand(T_CS!("\\bbl@loaded"))
      .map(|t| t.to_string()).unwrap_or_default();
    // Prefer the last entry in \bbl@loaded (explicit \usepackage options)
    // over \bbl@main@language (which may come from class options).
    // In babel, the last explicitly loaded language is the main language.
    let loaded_last = loaded.split(',').map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty()).last().unwrap_or_default();
    let lang_name = if !loaded_last.is_empty() && loaded_last != "nil" {
      loaded_last
    } else if main != "nil" && !main.is_empty() {
      main
    } else {
      String::new()
    };
    // Map language name to ISO code
    let iso = match lang_name.as_str() {
      "german" | "germanb" | "ngerman" | "ngermanb" => Some("de"),
      "french" | "francais" | "frenchb" => Some("fr"),
      "spanish" => Some("es"), "italian" => Some("it"),
      "english" => Some("en"),
      "american" | "USenglish" => Some("en-US"),
      "british" | "UKenglish" => Some("en-GB"),
      "portuguese" | "portuges" => Some("pt"),
      "russian" | "russianb" => Some("ru"),
      "greek" | "polutonikogreek" => Some("el"),
      "dutch" => Some("nl"), "polish" => Some("pl"),
      _ => None,
    };
    // Set DOCUMENT_LANGUAGE for xml:lang on <document>
    if let Some(code) = iso {
      state::assign_value("DOCUMENT_LANGUAGE", Stored::from(code.to_string()), Some(Scope::Global));
      let mut font = Font::default();
      font.language = Some(Cow::Owned(code.to_string()));
      merge_font(font);
    }
    // Call \captions<lang> to set localized names
    let captions_cs = s!("\\captions{}", lang_name);
    if lookup_definition(&T_CS!(captions_cs.clone()))?.is_some() {
      stomach::digest(Tokenize!(&captions_cs))?;
    }
    // French-specific: define French macros directly
    let is_french = lang_name == "french" || lang_name == "francais"
      || lang_name == "frenchb" || loaded.contains("french");
    if is_french && lookup_definition(&T_CS!("\\up"))?.is_none() {
      // Core French macros from french.ldf.ltxml / frenchb.ldf.ltxml
      stomach::digest(Tokenize!(r"\def\up#1{\textsuperscript{#1}}"))?;
      stomach::digest(Tokenize!(r"\def\fup#1{\textsuperscript{#1}}"))?;
      stomach::digest(Tokenize!(r"\def\No{N\up{o}\xspace}\def\no{n\up{o}\xspace}"))?;
      stomach::digest(Tokenize!(r"\def\Nos{N\up{os}\xspace}\def\nos{n\up{os}\xspace}"))?;
      stomach::digest(Tokenize!(r"\def\bsc#1{{\scshape #1}}"))?;
      // Note: \ier/\iere/\ieme do NOT use \xspace (matching raw frenchb.ldf).
      // Only \primo/\secundo/... and \No/\Nos use \xspace.
      stomach::digest(Tokenize!(r"\def\ieme{\up{e}}\def\iemes{\up{es}}"))?;
      stomach::digest(Tokenize!(r"\def\ier{\up{er}}\def\iers{\up{ers}}"))?;
      stomach::digest(Tokenize!(r"\def\iere{\up{re}}\def\ieres{\up{res}}"))?;
      stomach::digest(Tokenize!(r"\def\FrenchEnumerate#1{#1\up{o}}"))?;
      stomach::digest(Tokenize!(r"\def\FrenchPopularEnumerate#1{#1\up{o})}"))?;
      stomach::digest(Tokenize!(r"\def\primo{1\up{o}\xspace}\def\secundo{2\up{o}\xspace}"))?;
      stomach::digest(Tokenize!(r"\def\tertio{3\up{o}\xspace}\def\quarto{4\up{o}\xspace}"))?;
      stomach::digest(Tokenize!(r"\def\fprimo){1\up{o})\xspace}\def\fsecundo){2\up{o})\xspace}"))?;
      stomach::digest(Tokenize!(r"\def\ftertio){3\up{o})\xspace}\def\fquarto){4\up{o})\xspace}"))?;
      stomach::digest(Tokenize!(r"\def\og{\guillemotleft\nobreakspace}"))?;
      stomach::digest(Tokenize!(r"\def\fg{\nobreakspace\guillemotright\xspace}"))?;
      stomach::digest(Tokenize!(r"\def\degre{\textdegree}"))?;
      stomach::digest(Tokenize!(r"\def\degres{\hbox to 0.3em{\degre}}"))?;
      stomach::digest(Tokenize!(r"\let\tild\textasciitilde"))?;
      stomach::digest(Tokenize!(r"\let\circonflexe\textasciicircum"))?;
      stomach::digest(Tokenize!(r"\def\at{@}\def\boi{\textbackslash}"))?;
      stomach::digest(Tokenize!(r"\def\nombre#1{\numprint{#1}}"))?;
      // Note: Perl has \let\xspace\relax here, but we have a proper
      // \xspace implementation in xspace_sty.rs. Load it for French macros.
      if lookup_definition(&T_CS!("\\xspace"))?.is_none() {
        crate::package::xspace_sty::load_definitions();
      }
      // TODO: French active punctuation (:;!? → thin space + char).
      // Implemented but produces U+2006 encoding while Perl produces regular spaces.
      // Need to match Perl's serialization of \hskip thin spaces before enabling.
    }
    // German-specific: activate " shorthand
    let is_german = lang_name == "german" || lang_name == "ngerman"
      || lang_name == "germanb" || loaded.contains("german");
    if is_german {
      state::assign_catcode('"', Catcode::ACTIVE, Some(Scope::Global));
      let active_dq = T_ACTIVE!('"');
      let dispatch_cs = T_CS!("\\lx@german@dq@dispatch");
      if let Some(defn) = lookup_meaning(&dispatch_cs) {
        state::assign_meaning(&active_dq, defn, Some(Scope::Global));
      }
    }
  });
  RawTeX!(r"\lx@babel@activate@lang@post");
});
