//! TeX Hyphenation
//!
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Hyphenation Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // explicit hyphenation
  //----------------------------------------------------------------------
  // - (discretionary hyphen)        d       inserts a discretionary hyphen.
  // \discretionary    c  specifies a discretionary break in a paragraph.
  DefPrimitive!("\\-", None);
  DefMacro!("\\discretionary{}{}{}", "#3"); // No hyphenation here!

  //======================================================================
  // hyphenation tables
  //----------------------------------------------------------------------
  // \hyphenation      c  adds words to the hyphenation exception dictionary for the current
  // language. \patterns         c  is used in INITEX to add patterns to the pattern dictionary
  // for the current language.
  DefMacro!("\\hyphenation GeneralText", None);
  DefMacro!("\\patterns{}", None);

  //======================================================================
  // language choice
  //----------------------------------------------------------------------
  // \setlanguage      c  inserts a language whatsit in restricted horizontal mode.
  // \language         pi selects a language to use with hyphenation and \patterns.
  DefRegister!("\\language", Number!(0));
  DefPrimitive!("\\setlanguage Number", None);

  // \languagename — current language name (text). babel's hyphen.cfg L175
  // sets `\def\languagename{english}` as the format-time default; subsequent
  // \selectlanguage{...} calls overwrite. Format-load isn't replicated in
  // the Rust port, so packages that probe \languagename (lipsum, blindtext,
  // hyperref language switches) hit undefined unless babel is loaded.
  // Mirror the format-time default here so any paper has a baseline name.
  DefMacro!("\\languagename", "english");

  //======================================================================
  // codepoints used for hyphenation
  //----------------------------------------------------------------------
  // \hyphenchar       iq holds the current hyphen character used with hyphenation.
  // \defaulthyphenchar pi is the \hyphenchar value to use when a new font is loaded.
  // \lefthyphenmin    pi is the minimum number of characters that must appear before the first
  // hyphen in an hyphenated word. \righthyphenmin   pi is the minimum number of characters that
  // must appear after the last hyphen in an hyphenated word. \uchyph           pi prevents
  // hyphenation of uppercase words unless this is positive.

  // Perl: getter looks up $$fontinfo{hyphenchar}, setter stores in fontinfo
  // Uses shared font key so fonts with same name+size share hyphenchar
  // Perl: $$fontinfo{hyphenchar} = $value (modifies shared hash directly, unscoped)
  DefRegister!("\\hyphenchar FontToken", Number::new(b'-' as i64),
    getter => sub[args] {
      let font_token = args.remove(0).expected_token();
      let cs_str = font_token.to_string();
      // Look up the shared font key via with_value to avoid cloning the
      // Stored envelope on the String arm (just carrying a Copy SymStr).
      let hc_key = state::with_value(&s!("font_shared_key_{cs_str}"), |v| match v {
        Some(Stored::String(s)) => arena::with(*s, |sk| s!("hyphenchar_{sk}")),
        _ => s!("hyphenchar_{cs_str}"),
      });
      state::with_value(&hc_key, |v| match v {
        Some(Stored::Number(n)) => *n,
        _ => Number::new(b'-' as i64),
      })
    },
    setter => sub[value, _scope, args] {
      let font_token = args.remove(0).expected_token();
      let cs_str = font_token.to_string();
      // Look up the shared font key via with_value — same reasoning as above.
      let hc_key = state::with_value(&s!("font_shared_key_{cs_str}"), |v| match v {
        Some(Stored::String(s)) => arena::with(*s, |sk| s!("hyphenchar_{sk}")),
        _ => s!("hyphenchar_{cs_str}"),
      });
      // Perl stores directly in fontinfo hash (unscoped/global)
      state::assign_value(
        &hc_key,
        Stored::Number(value.into()),
        Some(Scope::Global),
      );
    }
  );
  DefRegister!("\\defaulthyphenchar", Number!(45));
  DefRegister!("\\lefthyphenmin", Number!(0));
  DefRegister!("\\righthyphenmin", Number!(0));
  DefRegister!("\\uchyph", Number!(1));
});
