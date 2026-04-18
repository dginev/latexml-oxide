//! babel.sty — multilingual support
//!
//! Perl: babel.sty.ltxml (30 lines) — `InputDefinitions('babel', noltxml=>1)`.
//! Our Rust port carries a thin orchestration layer on top of the raw babel
//! load. With the @currname leakage fix in commit 56b0c35d2, babel's own
//! option pipeline (and therefore its entire language-loading / shorthand /
//! captions story) now works end-to-end. Only two small workarounds remain
//! here: pre-allocating `\l@polutonikogreek` for older TeX Live builds that
//! don't include it in the kernel dump, and setting DOCUMENT_LANGUAGE +
//! `\bbl@main@language` globally (babel's own raw-load path may resolve
//! main to a language whose .ldf happened to run last — not always the
//! user's intended last option).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // \l@polutonikogreek: allocate if not present in kernel dump (newer TeXLive
  // includes it, older may not).
  RawTeX!(r"\expandafter\ifx\csname l@polutonikogreek\endcsname\relax\newlanguage\l@polutonikogreek\fi");

  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Sets DOCUMENT_LANGUAGE and force-sets \bbl@main@language from
  // \opt@babel.sty so babel's `\AtBeginDocument{\selectlanguage{\bbl@main
  // @language}}` picks up the user's intended main language (the last
  // option), not whichever .ldf's \ldf@finish happened to run last.
  // Everything else (captions activation, active-char shorthands, port
  // dispatching) is handled end-to-end by babel's own chain.
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
    // Force-set \bbl@main@language to the resolved language so babel's own
    // `\AtBeginDocument{\selectlanguage{\bbl@main@language}}` picks up the
    // correct value. Without this, babel's raw-load path may leave it as
    // "nil" or pointing at a different language whose .ldf \ldf@finish
    // ran last (e.g. greek.ldf in `[polutonikogreek,english]`).
    def_macro(T_CS!("\\bbl@main@language"), None,
      Tokens!(Explode!(lang.clone())),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..ExpandableOptions::default() }))?;
    // Note: babel's now-working pipeline handles:
    // - Loading per-language ports via \InputIfFileExists{<lang>.ldf}
    //   (routed through our binding dispatcher).
    // - Activating \captions<lang> via \AtBeginDocument's
    //   \selectlanguage{\bbl@main@language} → our \select@language
    //   override → babel's saved \bbl@switch which calls
    //   \captions<lang>.
    // - Active-char shorthands (German " / French :;!? if frenchb
    //   path uses babel's \declare@shorthand mechanism).
    // We used to invoke these manually here as a workaround for the
    // @currname leakage bug (fixed in commit 56b0c35d2); dropping the
    // redundant paths now.
  });
  // Run mainlang at load time so DOCUMENT_LANGUAGE is set before
  // \begin{document} opens (and base_schema's after_open reads it).
  RawTeX!(r"\lx@babel@activate@mainlang");

});
