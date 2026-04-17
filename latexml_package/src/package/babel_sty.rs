//! babel.sty — multilingual support
//!
//! Perl: babel.sty.ltxml (30 lines) — `InputDefinitions('babel', noltxml=>1)`.
//! Our Rust port carries several pre-raw-load and post-raw-load workarounds
//! to cover engine gaps Perl doesn't have (precompiled kernel, proper
//! `\initiate@active@char`, kpsewhich-backed ini reading). See
//! `wisdom_babel_fidelity_plan.md` in project memory for the staged plan
//! to shrink this file back to ~30 lines.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // --- Pre-raw-load workarounds -------------------------------------------
  // \l@<lang> registers: 108 are in the kernel dump; pre-allocate the two
  // that aren't, so babel's \bbl@iflanguage and nil.ldf's \l@nil check pass.
  RawTeX!(r"\expandafter\ifx\csname l@polutonikogreek\endcsname\relax\newlanguage\l@polutonikogreek\fi");
  RawTeX!(r"\expandafter\ifx\csname l@nil\endcsname\relax\newlanguage\l@nil\fi");
  // \bbl@languages empty: babel's language.def loading uses \openin on
  // hyphenation-pattern files we can't find. Without this, \bbl@languages
  // stays undefined → error recovery emits <ltx:ERROR/> → list corruption → OOM.
  RawTeX!(r"\def\bbl@languages{}");
  // \bbl@opt@hyphenmap: normally set by the `.ini` loading path we skip.
  RawTeX!(r"\chardef\bbl@opt@hyphenmap\@ne");
  // Clear \CurrentOption before raw load so prior-package leakage (e.g.
  // keyval.sty's "unknownkeyserror") doesn't get mis-interpreted as a
  // babel language option at L4177.
  RawTeX!(r"\let\CurrentOption\@empty");

  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // --- Post-raw-load workarounds ------------------------------------------
  // Caption strings come from the per-language Rust ports
  // (english_sty.rs, french_ldf.rs, german_sty.rs, ngerman_sty.rs). Our
  // `\lx@babel@activate@mainlang` below loads the matching port on demand
  // and then calls `\captions<lang>`. This mirrors what Perl's precompiled
  // kernel + babel's own \selectlanguage{\bbl@main@language} do end-to-end
  // (which we can't rely on yet — \bbl@main@language is "nil" in our
  // engine because the two-phase \ProcessOptions* pipeline doesn't fire
  // \bbl@load@language cleanly; see SYNC_STATUS D0).

  // French active-punctuation dispatch primitives for :;!? (frenchb.ldf's
  // \extrasfrench insets a thin space before these chars). Dispatch CSes
  // are defined unconditionally; `\ltx@bbl@select@language` (babel_support)
  // flips the catcodes and attaches meanings when French is entered. The
  // primitives check current font language and fall back to bare
  // punctuation in non-French groups (needed because `\foreignlanguage
  // {english}{…!}` re-uses already-tokenized ACTIVE tokens).
  //
  //   ':'  → " :" (regular space, espace insécable visual)
  //   ';!?' → "\u{2006}X" (thin space, SIX-PER-EM SPACE)
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

  // Main-language activation: sets DOCUMENT_LANGUAGE, calls \captions<lang>,
  // wires French :;!? and German " active-char dispatch. Bypasses babel's
  // own \selectlanguage{\bbl@main@language} (which is "nil" in our engine
  // due to the option-pipeline gap) by resolving the effective language
  // from \opt@babel.sty directly.
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
      return Ok(vec![]);
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
    // Load the matching per-language Rust port so \captions<lang> (and
    // language-specific macros) are defined. Idempotent — load_definitions
    // is guarded against re-entry.
    match lang.as_str() {
      "english" | "american" | "USenglish" | "british" | "UKenglish" =>
        { let _ = crate::package::english_sty::load_definitions(); },
      "french" | "francais" | "frenchb" =>
        { let _ = crate::package::french_ldf::load_definitions(); },
      "german" | "germanb" =>
        { let _ = crate::package::german_sty::load_definitions(); },
      "ngerman" | "ngermanb" =>
        { let _ = crate::package::ngerman_sty::load_definitions(); },
      _ => {}
    }
    // Activate captions. @ may be OTHER (at \begin{document}) so flip it
    // temporarily to LETTER to tokenize `\captions<lang>` as one CS.
    state::assign_catcode('@', Catcode::LETTER, None);
    let cs = s!("\\captions{}", lang);
    if lookup_definition(&T_CS!(cs.clone()))?.is_some() {
      stomach::digest(Tokenize!(&cs))?;
    }
    state::assign_catcode('@', Catcode::OTHER, None);
    if lang == "french" || lang == "francais" || lang == "frenchb" {
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
      if lookup_definition(&T_CS!("\\up"))?.is_none() {
        let _ = crate::package::french_ldf::load_definitions();
        if lookup_definition(&T_CS!("\\xspace"))?.is_none() {
          let _ = crate::package::xspace_sty::load_definitions();
        }
      }
    }
    if lang == "german" || lang == "germanb" || lang == "ngerman" || lang == "ngermanb" {
      if let Some(defn) = lookup_meaning(&T_CS!("\\lx@german@dq@dispatch")) {
        state::assign_catcode('"', Catcode::ACTIVE, Some(Scope::Global));
        state::assign_meaning(&T_ACTIVE!('"'), defn, Some(Scope::Global));
      }
    }
  });
  // Load all per-language Rust ports mentioned in \opt@babel.sty so any
  // language can be entered via \begin{otherlanguage}{<lang>} or
  // \selectlanguage{<lang>} without waiting for the runtime dispatch.
  // Needed because babel's own \select@language calls \captions<lang>
  // before our \ltx@bbl@select@language hook can load the port.
  DefPrimitive!("\\lx@babel@load@ports", {
    let opt_babel = gullet::do_expand(Tokenize!(r"\csname opt@babel.sty\endcsname"))
      .map(|t| t.to_string()).unwrap_or_default();
    for raw in opt_babel.split(',') {
      match raw.trim() {
        "english" | "american" | "USenglish" | "british" | "UKenglish" =>
          { let _ = crate::package::english_sty::load_definitions(); },
        "french" | "francais" | "frenchb" =>
          { let _ = crate::package::french_ldf::load_definitions(); },
        "german" | "germanb" =>
          { let _ = crate::package::german_sty::load_definitions(); },
        "ngerman" | "ngermanb" =>
          { let _ = crate::package::ngerman_sty::load_definitions(); },
        _ => {}
      }
    }
  });
  RawTeX!(r"\lx@babel@load@ports");

  // Run mainlang at load time so DOCUMENT_LANGUAGE is set before
  // \begin{document} opens (and base_schema's after_open reads it). Also
  // re-run via AtBeginDocument so any late state is refreshed.
  RawTeX!(r"\lx@babel@activate@mainlang");
  RawTeX!(r"\AtBeginDocument{\lx@babel@activate@mainlang}");

  // German " shorthand dispatch (from germanb.ldf). babel's
  // \initiate@active@char mechanism doesn't survive our raw load; we
  // read the next char after " and emit the umlaut/ß/guillemet directly.
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
