//! babel_support.sty — LaTeXML support macros for babel
//! Perl: babel_support.sty.ltxml (169 lines)
//!
//! Provides: Unicode quote characters, language→ISO mapping,
//! \selectlanguage hook for xml:lang attribute.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Unicode quote characters (Perl L24-42)
  // DefPrimitiveI in Perl outputs literal text — use DefMacro here
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
  DefMacro!("\\@nopatterns{}", "");

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
    let iso = match lang.as_str() {
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
    };
    if let Some(code) = iso {
      // Set cf@encoding to current encoding
      def_macro(T_CS!("\\cf@encoding"), None,
        gullet::do_expand(T_CS!("\\f@encoding"))?, None)?;
      // Merge language into font → produces xml:lang attribute
      let mut font = Font::default();
      font.language = Some(Cow::Owned(code.to_string()));
      merge_font(font);
      // Also store as a state value for the document element's after_open hook
      // (font merge may not persist through document construction phase)
      state::assign_value("DOCUMENT_LANGUAGE", Stored::from(code.to_string()), Some(Scope::Global));
    }
  });

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
