//! babel_support.sty — LaTeXML support macros for babel
//! Perl: babel_support.sty.ltxml (169 lines)
//!
//! Provides: Unicode quote characters, language→ISO mapping,
//! \selectlanguage hook for xml:lang attribute.
use crate::prelude::*;

/// Map a babel language option to its BCP 47 / ISO language tag.
/// Ported from Perl babel_support.sty.ltxml's `$bbl_language_map`.
/// Used by both `\ltx@bbl@select@language` (runtime switch) and
/// `\lx@babel@activate@mainlang` (load-time main-language resolution).
pub fn babel_language_to_iso(lang: &str) -> Option<&'static str> {
  match lang {
    "albanian" => Some("sq"),
    "acadian" | "canadien" => Some("fr-CA"),
    "afrikaans" => Some("af"),
    "american" | "USenglish" => Some("en-US"),
    "australian" => Some("en-AU"),
    "austrian" | "naustrian" => Some("de-AT"),
    "bahasa" | "bahasai" | "indon" | "indonesian" => Some("in"),
    "bahasam" | "malay" | "meyalu" => Some("ms"),
    "basque" => Some("eu"),
    "breton" => Some("br"),
    "bulgarian" => Some("bg"),
    "brazil" | "brazilian" => Some("pt-BR"),
    "british" | "UKenglish" => Some("en-GB"),
    "canadian" => Some("en-CA"),
    "catalan" => Some("ca"),
    "croatian" => Some("hr"),
    "czech" => Some("cs"),
    "danish" => Some("da"),
    "dutch" => Some("nl"),
    "english" => Some("en"),
    "esperanto" => Some("eo"),
    "estonian" => Some("et"),
    "finnish" => Some("fi"),
    "francais" | "french" | "frenchb" => Some("fr"),
    "galician" => Some("gl"),
    "german" | "germanb" | "ngerman" | "ngermanb" => Some("de"),
    "greek" | "polutonikogreek" => Some("el"),
    "hebrew" => Some("he"),
    "hindi" => Some("hi"),
    "hungarian" => Some("hu"),
    "icelandic" => Some("is"),
    "interlingua" => Some("ia"),
    "irish" => Some("ga"),
    "italian" => Some("it"),
    "latin" => Some("la"),
    "lowersorbian" => Some("dsb"),
    "newzealand" => Some("en-NZ"),
    "norsk" | "nynorsk" => Some("nn"),
    "nswissgerman" | "swissgerman" => Some("gsw"),
    "polish" => Some("pl"),
    "portuges" | "portuguese" => Some("pt"),
    "romanian" => Some("ro"),
    "romansh" => Some("rm"),
    "russian" | "russianb" => Some("ru"),
    "samin" => Some("se"),
    "scottish" => Some("gd"),
    "serbian" | "serbianc" => Some("sr"),
    "slovak" => Some("sk"),
    "slovene" => Some("sl"),
    "spanish" => Some("es"),
    "swedish" => Some("sv"),
    "thai" => Some("th"),
    "turkish" => Some("tr"),
    "ukraineb" | "ukrainian" => Some("uk"),
    "usorbian" | "uppersorbian" => Some("hsb"),
    "vietnamese" | "vietnam" => Some("vi"),
    "welsh" => Some("cy"),
    _ => None,
  }
}

/// Flip the French active punctuation (`:`, `;`, `!`, `?`) to catcode ACTIVE
/// and attach their thin-space dispatch meanings. Shared by the runtime
/// `\selectlanguage` hook (body switches) and the `\begin{document}` deferral
/// (preamble switches — see `\lx@bbl@begindoc@french@punct`). Real babel turns
/// shorthands on only at `\begin{document}`; activating in the preamble
/// corrupts packages loaded after `\selectlanguage{french}`.
pub(crate) fn activate_french_active_punct() {
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
}

#[rustfmt::skip]
LoadDefinitions!({
  // Many TL2025 babel language files (e.g. babel-italian italian.ldf,
  // babel-spanish spanish.ldf) use etoolbox CSes like \ifdefstring inside
  // hooks (e.g. `\bbl@beforestart`) and queue `\AtEndOfPackage{
  // \RequirePackage{etoolbox}}` to satisfy them. Our `\AtEndOfPackage`
  // machinery for raw-loaded `.ldf` chains doesn't reliably fire that
  // queued load before babel hooks evaluate, so the test paper 0710.5177
  // (`[english,italian]{article}`) fails with `\ifdefstring` undefined
  // at `\bbl@beforestart`. Pre-loading etoolbox once for any babel-driven
  // language tag closes that window. (Slight Perl divergence: Perl's
  // `\AtEndOfPackage` chain succeeds organically.)
  RequirePackage!("etoolbox");

  // Pre-define `\bbl@languages` as an empty stub so any babel-XX.tex
  // language file that references it at L21 (e.g. babel-german.tex,
  // babel-italian.tex) finds something. babel.sty itself guards with
  // `\ifx\bbl@languages\@undefined` (L266, L978) but the per-language
  // .tex files do NOT — they assume `\bbl@languages` was set up by
  // an earlier babel.sty load. With our `\usepackage[<lang>]{babel}`
  // path, the language `.tex` is sometimes processed before babel.sty
  // finishes its own setup; pre-defining here covers that load order.
  // Mirrors the conditional definition in `nil.ldf.ltxml` (Perl L20-21
  // / Rust nil_ldf.rs:6-8) but applied at the `babel_support` layer
  // so it's always in scope before any lang.tex loads.
  if !IsDefined!(&T_CS!("\\bbl@languages")) {
    def_macro_noop("\\bbl@languages")?;
  }

  // Unicode quote characters (Perl L24-42)
  //
  // Perl: DefPrimitiveI('\ij', undef, "ij") etc. — DefPrimitive with literal
  // string body, emits a Box. Rust uses DefMacro with string body which
  // expands and re-tokenizes into character tokens. Both produce the same
  // HTML output; for these simple language-shortcut/digraph CSes the DP
  // audit flags 15 structural DefPrimitiveI↔DefMacro mismatches that are
  // intentional — DefMacro is idiomatic Rust for plain-text CS aliases.
  DefMacro!("\\ij", "ij");
  DefMacro!("\\IJ", "IJ");

  DefMacro!("\\flq", "\u{2039}");
  DefMacro!("\\frq", "\u{203A}");
  DefMacro!("\\flqq", "\u{00AB}");
  DefMacro!("\\frqq", "\u{00BB}");

  DefMacro!("\\glq", "\u{201A}");
  DefMacro!("\\grq", "\u{2018}");
  DefMacro!("\\glqq", "\u{201E}");
  DefMacro!("\\grqq", "\u{201C}");

  DefMacro!("\\SS", "SS");

  DefMacro!("\\guilsinglleft", "\u{2039}");
  DefMacro!("\\guilsinglright", "\u{203A}");
  DefMacro!("\\guillemotleft", "\u{00AB}");
  DefMacro!("\\guillemotright", "\u{00BB}");

  // Shutup about hyphenation patterns (Perl L45)
  def_macro_noop("\\@nopatterns{}")?;

  // Hook into \select@language, \foreign@language, \bbl@switch
  // to set xml:lang attribute via MergeFont(language)
  Let!("\\ltx@save@bbl@switch", "\\bbl@switch");
  Let!("\\ltx@save@select@language", "\\select@language");
  Let!("\\ltx@save@foreign@language", "\\foreign@language");

  RawTeX!(r#"\def\select@language#1{\ltx@save@select@language{#1}\ltx@bbl@select@language{#1}}"#);
  RawTeX!(r#"\def\foreign@language#1{\ltx@save@foreign@language{#1}\ltx@bbl@select@language{#1}}"#);
  RawTeX!(r#"\def\bbl@switch#1{\ltx@save@bbl@switch{#1}\ltx@bbl@select@language{#1}}"#);

  DefPrimitive!("\\ltx@bbl@select@language{}", sub[(language)] {
    let lang = language.to_string();
    let iso = babel_language_to_iso(&lang);
    if let Some(code) = iso {
      // Set cf@encoding to current encoding
      def_macro(T_CS!("\\cf@encoding"), None,
        gullet::do_expand(T_CS!("\\f@encoding"))?, None)?;
      // Merge language into font → produces xml:lang attribute
      merge_font(Font { language: Some(Cow::Owned(code.to_string())), ..Font::default() });
      // Perl: greek.ldf does \fontencoding{LGR}\selectfont in \extrasgreek
      // and restores via \noextrasgreek. We replicate this here since our babel
      // intercept doesn't load the real .ldf files.
      if code == "el" {
        load_font_map("LGR");
        MergeFont!(encoding => "LGR");
        // Greek accent shorthand: redefine active ~ to produce perispomeni
        // (U+1FC0) for LGR ligature composition. In standard TeX, ~ produces
        // tie/nobreakspace, but in Greek mode it's the circumflex accent
        // combining character that triggers ligatures like ~a → ᾶ.
        state::let_i(&T_CS!("\\ltx@save@greek@tilde"), &T_ACTIVE!('~'), None);
        def_macro(T_ACTIVE!('~'), None, TokenizeInternal!("\u{1FC0}"), None)?;
      } else {
        // Restore non-Greek encoding: check if we're coming from LGR
        let current_enc = lookup_font()
          .and_then(|f| f.get_encoding().map(|e| e.to_string()))
          .unwrap_or_else(|| "OT1".to_string());
        if current_enc == "LGR" {
          // Restore to OT1 (default Latin encoding) when leaving Greek
          load_font_map("OT1");
          MergeFont!(encoding => "OT1");
          // Restore ~ to its pre-Greek meaning (tie/nobreakspace)
          state::let_i(&T_ACTIVE!('~'), &T_CS!("\\ltx@save@greek@tilde"), None);
        }
      }
      // French active punctuation: Perl's frenchb.ldf `\extrasfrench`
      // hook activates `:`, `;`, `!`, `?` to emit a thin space before them;
      // `\noextrasfrench` deactivates on language exit. We mirror that.
      //
      // Dispatch primitives are defined in babel_sty.rs unconditionally.
      // Real babel defers shorthand activation to `\begin{document}`:
      // flipping `:;!?` to active while still in the PREAMBLE corrupts any
      // package loaded after `\selectlanguage{french}` — e.g. adjustbox,
      // whose graphicx `!` natural-size sentinel then tokenizes as an active
      // char and the glue parser hits it: "Missing close parenthesis in Glue
      // expr. Got T_ACTIVE[!]". Skip activation in the preamble; babel
      // re-fires `\selectlanguage{\bbl@main@language}` at `\begin{document}`
      // (via its AtBeginDocument hook), running this hook again with
      // `inPreamble` cleared, which performs the activation then. A body-level
      // `\selectlanguage` switch (inPreamble already false) still activates
      // immediately. Witness 1712.07003 (`\selectlanguage{french}` in the
      // preamble, then `\usepackage{adjustbox}`).
      if code == "fr" && !lookup_bool("inPreamble") {
        // Entering French in the document body: activate immediately. Preamble
        // switches are deferred to \begin{document} by the begin-document hook
        // below (so packages loaded after \selectlanguage see catcode-12 `!`).
        activate_french_active_punct();
      }
      // German: activate " as the shorthand dispatch for umlauts + opens
      // the \mdqon / \mdqoff toggle. Babel's germanb.ldf normally does this
      // via \initiate@active@char; we reproduce it with a direct meaning.
      if code == "de" || code == "de-AT" {
        if let Some(defn) = lookup_meaning(&T_CS!("\\lx@german@dq@dispatch")) {
          state::assign_catcode('"', Catcode::ACTIVE, Some(Scope::Global));
          state::assign_meaning(&T_ACTIVE!('"'), defn, Some(Scope::Global));
        }
      }
      // Leaving French/German does not automatically deactivate the
      // active-char meanings. The dispatch primitives (\lx@french@punct@*)
      // check \languagename themselves and fall back to bare punctuation in
      // non-French groups. DOCUMENT_LANGUAGE is only set once at babel-init
      // by \lx@babel@activate@mainlang (setting per-\selectlanguage would
      // clobber the root xml:lang when the body switches languages).
    }
  });

  // Deferred French-shorthand activation. The `\selectlanguage` hook above
  // skips activation while `inPreamble`, so `\selectlanguage{french}` followed
  // by `\usepackage{adjustbox}` (or any package whose macros embed a literal
  // `!`, e.g. graphicx's natural-size sentinel) loads with `!` still catcode-12
  // — matching real babel, which turns shorthands on at `\begin{document}`.
  // Re-run the activation here if the resolved document language is French.
  // Witness 1712.07003. (Body-level `\selectlanguage` switches happen after
  // `inPreamble` is cleared and activate immediately in the hook above.)
  DefPrimitive!("\\lx@bbl@begindoc@french@punct", {
    if lookup_string("DOCUMENT_LANGUAGE") == "fr" {
      activate_french_active_punct();
    }
  });
  RawTeX!(r"\AtBeginDocument{\lx@bbl@begindoc@french@punct}");

  // Pretend we've got hyphenation patterns for ANY language (Perl L158-167)
  DefMacro!("\\iflanguage{}", r#"\expandafter\ifx\csname l@#1\endcsname\relax
  \expandafter\newlanguage\csname l@#1\endcsname\fi
\expandafter\edef\expandafter\@@@@lang\expandafter{\csname l@#1\endcsname}
\ifnum\csname l@#1\endcsname=\language
  \expandafter\@firstoftwo
\else
  \expandafter\@secondoftwo
\fi"#);
});
