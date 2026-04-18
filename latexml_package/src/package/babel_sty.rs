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
  // \l@polutonikogreek: allocate if not present in kernel dump (newer TeXLive
  // includes it, older may not).
  RawTeX!(r"\expandafter\ifx\csname l@polutonikogreek\endcsname\relax\newlanguage\l@polutonikogreek\fi");

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
    // Force-set \bbl@main@language to the resolved language so babel's own
    // `\AtBeginDocument{\selectlanguage{\bbl@main@language}}` picks up the
    // correct value. Without this, babel's raw-load path may leave it as
    // "nil" or pointing at a different language whose .ldf \ldf@finish
    // ran last (e.g. greek.ldf in `[polutonikogreek,english]`).
    def_macro(T_CS!("\\bbl@main@language"), None,
      Tokens!(Explode!(lang.clone())),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..ExpandableOptions::default() }))?;
    // Note: babel's now-working option pipeline dispatches
    // \InputIfFileExists{<lang>.ldf} for each option, which routes
    // through our binding dispatcher to the per-language Rust ports
    // (english_sty.rs / french_ldf.rs / german_sty.rs / ngerman_sty.rs).
    // No manual port load here — previously needed as a workaround for
    // the @currname leakage bug (now fixed, commit 56b0c35d2).
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
    }
    if lang == "german" || lang == "germanb" || lang == "ngerman" || lang == "ngermanb" {
      if let Some(defn) = lookup_meaning(&T_CS!("\\lx@german@dq@dispatch")) {
        state::assign_catcode('"', Catcode::ACTIVE, Some(Scope::Global));
        state::assign_meaning(&T_ACTIVE!('"'), defn, Some(Scope::Global));
      }
    }
  });
  // Run mainlang at load time so DOCUMENT_LANGUAGE is set before
  // \begin{document} opens (and base_schema's after_open reads it).
  RawTeX!(r"\lx@babel@activate@mainlang");

});
