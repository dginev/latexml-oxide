//! babel.sty — multilingual support
//! Perl: babel.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Language registers (\l@english, \l@german, etc.) now come from the
  // kernel dump (108 \l@<lang> CharDefs). For the two that aren't in the
  // dump (\l@polutonikogreek, \l@nil), allocate them defensively so
  // babel's \bbl@iflanguage and nil.ldf's \l@nil check both pass.
  RawTeX!(r"\expandafter\ifx\csname l@polutonikogreek\endcsname\relax\newlanguage\l@polutonikogreek\fi");
  // Pre-define \l@nil so nil.ldf skips its \edef\bbl@languages block (which would
  // fail because \bbl@languages is undefined at that point). With \l@nil defined,
  // nil.ldf's \ifx\l@nil\@undefined check fails and the block is skipped.
  RawTeX!(r"\expandafter\ifx\csname l@nil\endcsname\relax\newlanguage\l@nil\fi");

  // Pre-define \bbl@languages as empty. Babel's language.def loading uses \openin
  // to read hyphenation pattern files, which our system can't find (.ini search paths).
  // Without this, \bbl@languages stays undefined, and our error recovery defines it
  // as <ltx:ERROR/> which corrupts babel's list accumulation → infinite recursion → OOM.
  RawTeX!(r"\def\bbl@languages{}");

  // Pre-define babel internals that are normally set by the ini-based loading path
  // (\babelprovide) which we skip. Without these, \ifcase\bbl@opt@hyphenmap fails.
  RawTeX!(r"\chardef\bbl@opt@hyphenmap\@ne");

  // Clear \CurrentOption before loading babel to prevent leakage from
  // previously loaded packages (e.g., keyval.sty's "unknownkeyserror" option
  // leaks into babel's \bbl@load@language which uses \CurrentOption at L4177).
  RawTeX!(r"\let\CurrentOption\@empty");

  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Caption-string workaround.
  //
  // In Perl, `make formats` precompiles the kernel with every .ldf's
  // \captions<lang> macros already in place; babel's own AtBeginDocument
  // then calls \selectlanguage{\bbl@main@language} which activates them.
  // Our engine can load english.ldf / germanb.ldf etc. via dispatch
  // (english_sty.rs, german_sty.rs, ngerman_sty.rs, french_ldf.rs carry
  // the caption strings), BUT babel's two-phase \ProcessOptions* option
  // pipeline — \bbl@language@opts collection, then \DeclareOption fan-out
  // — doesn't successfully fire \bbl@load@language{<lang>} for the
  // package option in our engine. `\bbl@main@language` ends up "nil".
  //
  // Pre-defining the captions here gives our `\lx@babel@activate@mainlang`
  // hook something to call at \begin{document} time (resolving the
  // effective main language via \opt@babel.sty instead of relying on
  // \bbl@main@language). This will be removable once the two-phase
  // option pipeline works end-to-end; see SYNC_STATUS D0 "AtBeginDocument
  // hook chain ordering".
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

  // French active-punctuation dispatch primitives.
  //
  // Perl frenchb.ldf's `\extrasfrench` hook activates `:`, `;`, `!`, `?`
  // so they insert a thin space before them in French text. Our port
  // splits the work:
  //
  //   1. Here: define the dispatch CSes unconditionally (cheap, inert
  //      until a character's catcode is ACTIVE and its meaning points
  //      at one of them).
  //   2. In `babel_support_sty.rs::\ltx@bbl@select@language`: when the
  //      selected language is French, flip the four character catcodes
  //      to ACTIVE and attach these dispatch primitives as their
  //      meanings. Fires for both `\selectlanguage{french}` and
  //      `\foreign@language{french}` / `\begin{otherlanguage}{french}`.
  //
  // Previously the dispatch primitives were defined INSIDE
  // `\lx@babel@activate@mainlang`'s main-language-is-french branch,
  // which meant \begin{otherlanguage}{french} inside a
  // german/english document couldn't find them → no French spacing.
  // Unifying the definitions here fixes page545's French paragraph.
  //
  // Spacing rules (matching Perl frenchb.ldf):
  //   before ':'  → regular space (espace insécable, rendered " :")
  //   before ';!?' → thin space (U+2006, SIX-PER-EM SPACE)
  //
  // Active on ACTIVE-catcode :;!? only when current language is French.
  // Tokens tokenized inside a French group keep their ACTIVE catcode
  // when consumed in a non-French context (e.g. \foreignlanguage{english}{…!}),
  // but the dispatch primitive must respect the *current* language —
  // Perl frenchb.ldf checks \languagename via \@ensuredmath etc.
  // If current language is not French, fall back to emitting just the
  // bare punctuation (no thin space).
  fn in_french() -> bool {
    lookup_font()
      .and_then(|f| f.get_language().map(|l| l.as_ref() == "fr" || l.as_ref() == "fr-CA"))
      .unwrap_or(false)
  }
  DefPrimitive!("\\lx@french@punct@colon", {
    enter_horizontal();
    let s = if in_french() { " :" } else { ":" };
    Tbox::new(arena::pin_static(s), None, None, Tokens!(), stored_map!())
  });
  DefPrimitive!("\\lx@french@punct@semi", {
    enter_horizontal();
    let s = if in_french() { "\u{2006};" } else { ";" };
    Tbox::new(arena::pin_static(s), None, None, Tokens!(), stored_map!())
  });
  DefPrimitive!("\\lx@french@punct@exclam", {
    enter_horizontal();
    let s = if in_french() { "\u{2006}!" } else { "!" };
    Tbox::new(arena::pin_static(s), None, None, Tokens!(), stored_map!())
  });
  DefPrimitive!("\\lx@french@punct@question", {
    enter_horizontal();
    let s = if in_french() { "\u{2006}?" } else { "?" };
    Tbox::new(arena::pin_static(s), None, None, Tokens!(), stored_map!())
  });

  // After babel loads: activate the main language's captions and set xml:lang.
  // Rationale: babel's own \AtBeginDocument runs \selectlanguage{\bbl@main@language}
  // which SHOULD handle all of this via our \select@language override, but
  // \bbl@main@language may not reflect the user's intended last-loaded
  // package option in our engine. We resolve the effective main language
  // from \opt@babel.sty (the package option list) with a fallback to
  // \bbl@main@language, then mirror babel's own behavior.
  DefPrimitive!("\\lx@babel@activate@mainlang", {
    let main = gullet::do_expand(T_CS!("\\bbl@main@language"))
      .map(|t| t.to_string()).unwrap_or_default();
    let opt_babel = gullet::do_expand(Tokenize!(r"\csname opt@babel.sty\endcsname"))
      .map(|t| t.to_string()).unwrap_or_default();
    let pkg_last = opt_babel.split(',').map(|s| s.trim().to_string())
      .rfind(|s| !s.is_empty() && s != "nil").unwrap_or_default();
    let lang = if !pkg_last.is_empty() {
      pkg_last
    } else if main != "nil" && !main.is_empty() {
      main
    } else {
      String::new()
    };
    if !lang.is_empty() {
      // At \begin{document} time @ is OTHER; temporarily flip to LETTER
      // so `\captions<lang>` parses as one CS.
      state::assign_catcode('@', Catcode::LETTER, None);
      let cs = s!("\\captions{}", lang);
      if lookup_definition(&T_CS!(cs.clone()))?.is_some() {
        stomach::digest(Tokenize!(&cs))?;
      }
      state::assign_catcode('@', Catcode::OTHER, None);
      let iso: Option<&'static str> = match lang.as_str() {
        "german" | "germanb" | "ngerman" | "ngermanb" => Some("de"),
        "french" | "francais" | "frenchb" => Some("fr"),
        "english" => Some("en"),
        "american" | "USenglish" => Some("en-US"),
        "british" | "UKenglish" => Some("en-GB"),
        "greek" | "polutonikogreek" => Some("el"),
        _ => None,
      };
      if let Some(code) = iso {
        merge_font(Font { language: Some(Cow::Owned(code.to_string())), ..Font::default() });
      }
      if lang == "french" || lang == "francais" || lang == "frenchb" {
        // Active-punctuation dispatch for :;!?
        for &(ch, cs_name) in &[
          (':', "\\lx@french@punct@colon"),
          (';', "\\lx@french@punct@semi"),
          ('!', "\\lx@french@punct@exclam"),
          ('?', "\\lx@french@punct@question"),
        ] {
          if let Some(defn) = lookup_meaning(&T_CS!(cs_name)) {
            state::assign_catcode(ch, Catcode::ACTIVE, Some(Scope::Global));
            state::assign_meaning(&T_ACTIVE!(ch), defn, Some(Scope::Global));
          }
        }
        // Load our Rust french.ldf port (frenchb ordinals etc.) on first need.
        if lookup_definition(&T_CS!("\\up"))?.is_none() {
          let _ = crate::package::french_ldf::load_definitions();
          if lookup_definition(&T_CS!("\\xspace"))?.is_none() {
            let _ = crate::package::xspace_sty::load_definitions();
          }
        }
      }
      if lang == "german" || lang == "germanb" || lang == "ngerman" || lang == "ngermanb" {
        // " shorthand: babel's germanb.ldf does this via \initiate@active@char;
        // we wire the dispatch meaning directly.
        if let Some(defn) = lookup_meaning(&T_CS!("\\lx@german@dq@dispatch")) {
          state::assign_catcode('"', Catcode::ACTIVE, Some(Scope::Global));
          state::assign_meaning(&T_ACTIVE!('"'), defn, Some(Scope::Global));
        }
      }
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
  // DOCUMENT_LANGUAGE: set from the last explicit babel option so the
  // document-root xml:lang reflects the user's intent, matching what Perl's
  // precompiled-kernel path produces. (babel's internal \bbl@main@language
  // may only reflect class options in our engine.)
  DefPrimitive!("\\lx@babel@set@doclang", {
    let opt_babel = gullet::do_expand(Tokenize!(r"\csname opt@babel.sty\endcsname"))
      .map(|t| t.to_string()).unwrap_or_default();
    let pkg_last = opt_babel.split(',').map(|s| s.trim().to_string())
      .rfind(|s| !s.is_empty() && s != "nil").unwrap_or_default();
    let main = gullet::do_expand(T_CS!("\\bbl@main@language"))
      .map(|t| t.to_string()).unwrap_or_default();
    let lang = if !pkg_last.is_empty() {
      pkg_last
    } else if main != "nil" && !main.is_empty() {
      main
    } else {
      String::new()
    };
    let iso: Option<&'static str> = match lang.as_str() {
      "german" | "germanb" | "ngerman" | "ngermanb" => Some("de"),
      "french" | "francais" | "frenchb" => Some("fr"),
      "english" => Some("en"),
      "american" | "USenglish" => Some("en-US"),
      "british" | "UKenglish" => Some("en-GB"),
      "greek" | "polutonikogreek" => Some("el"),
      _ => None,
    };
    if let Some(code) = iso {
      state::assign_value("DOCUMENT_LANGUAGE",
        Stored::from(code.to_string()), Some(Scope::Global));
      merge_font(Font { language: Some(Cow::Owned(code.to_string())), ..Font::default() });
    }
  });
  RawTeX!(r"\lx@babel@set@doclang");
});
