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

  // \languagename — current language name (text). Perl's latex.dump captures
  // `\def\languagename{nohyphenation}` as the format-time default
  // (latex_dump.pool.ltxml L16522). Subsequent \selectlanguage{...} calls
  // overwrite. Mirror that default exactly: apacite.sty L1422-1423 explicitly
  // tests `\ifx\languagename<nohyphenation>` and skips its language-aware
  // file lookup when the test passes, which is the Perl-clean path.
  // Witness: 0906.3507 — Rust's prior "english" default caused apacite to
  // load the system english.apc (newer than local apacite.sty), triggering
  // an undefined `\if@APAC@natbib@apa` cascade.
  DefMacro!("\\languagename", "nohyphenation");

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
      // Resolve to canonical font identity via Primitive.font_id so
      // `\let`-aliased fonts share hyphenchar storage. Mirrors the
      // fontdimen indirection in tex_fonts.rs.
      let canonical_cs = state::lookup_meaning(&font_token)
        .and_then(|m| if let Stored::Primitive(p) = m { p.font_id }
                      else { None })
        .map(|fid| {
          let s = arena::with(fid, |x| x.to_string());
          s.strip_prefix("fontinfo_").unwrap_or(&s).to_string()
        })
        .unwrap_or_else(|| cs_str.clone());
      let hc_key = state::with_value(&s!("font_shared_key_{canonical_cs}"), |v| match v {
        Some(Stored::String(s)) => arena::with(*s, |sk| s!("hyphenchar_{sk}")),
        _ => s!("hyphenchar_{canonical_cs}"),
      });
      state::with_value(&hc_key, |v| match v {
        Some(Stored::Number(n)) => *n,
        _ => Number::new(b'-' as i64),
      })
    },
    setter => sub[value, _scope, args] {
      let font_token = args.remove(0).expected_token();
      let cs_str = font_token.to_string();
      let canonical_cs = state::lookup_meaning(&font_token)
        .and_then(|m| if let Stored::Primitive(p) = m { p.font_id }
                      else { None })
        .map(|fid| {
          let s = arena::with(fid, |x| x.to_string());
          s.strip_prefix("fontinfo_").unwrap_or(&s).to_string()
        })
        .unwrap_or_else(|| cs_str.clone());
      let hc_key = state::with_value(&s!("font_shared_key_{canonical_cs}"), |v| match v {
        Some(Stored::String(s)) => arena::with(*s, |sk| s!("hyphenchar_{sk}")),
        _ => s!("hyphenchar_{canonical_cs}"),
      });
      state::assign_value(
        &hc_key,
        Stored::Number(value.into()),
        Some(Scope::Global),
      );
    }
  );
  // \defaulthyphenchar lives in tex_fonts.rs (Perl: TeX_Fonts.pool.ltxml L78);
  // referenced as a comment here in the Perl source (L53) but not defined.
  DefRegister!("\\lefthyphenmin", Number!(0));
  DefRegister!("\\righthyphenmin", Number!(0));
  DefRegister!("\\uchyph", Number!(1));
});
