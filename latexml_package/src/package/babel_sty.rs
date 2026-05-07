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

  // `activeacute` was historically an option to babel-spanish.ldf that
  // activated `'` as an active accent. Some 1995-2010 papers wrote
  // `\usepackage[<lang>,activeacute]{babel}` treating it as a language;
  // modern babel doesn't recognize it as a language and `\InputIfFileExists
  // {activeacute.ldf}` silently fails (no on-disk file in TL). Babel then
  // proceeds and `\selectlanguage{...}` errors with "haven't defined the
  // language 'activeacute' yet". We pre-register `\l@activeacute` plus the
  // empty `<lang>` hooks so `\selectlanguage` resolves silently — actual
  // active-acute-on-quote semantics are not reproduced (most affected
  // papers only set this option as a side effect of preamble copy-paste).
  RawTeX!(r"%
    \expandafter\ifx\csname l@activeacute\endcsname\relax
      \newlanguage\l@activeacute
    \fi
    \providecommand\captionsactiveacute{}%
    \providecommand\extrasactiveacute{}%
    \providecommand\noextrasactiveacute{}%
    \providecommand\dateactiveacute{}");

  // \bbl@opt@safe = \@empty inhibits some risky redefinitions in babel.
  // Mirror Perl LaTeXML/lib/LaTeXML/Package/babel.def.ltxml: `Let('\bbl@opt@safe', '\@empty')`.
  // Without this, babel.sty's option processing enters an infinite loop on
  // some redefinition paths (verified: triggers token_limit:Timeout 100M).
  RawTeX!(r"\let\bbl@opt@safe\@empty");

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
    // Modern babel accepts `main=<lang>` to pin the document main language
    // (driver: 2109.00402 \usepackage[main=english]{babel}). Two cases:
    //   - `main=<lang>`  → use <lang> directly (highest priority)
    //   - bare positional option (no `=`) → treat as a language candidate
    //   - `<key>=<value>` for any other key (e.g. `shorthands=off`,
    //     `provide=*`) → SKIP, it's an option not a language. Driver:
    //     2001.00747 `\usepackage[english, shorthands=off]{babel}` was
    //     selecting "off" as the language because we promoted any value-
    //     half of any key=value option.
    let main_kv = opt_babel.split(',')
      .map(str::trim)
      .find_map(|s| s.strip_prefix("main=").map(str::trim).map(str::to_string));
    // Filter: drop key=value options AND babel-language MODIFIERS (sub-options
    // recognised by a single .ldf rather than a language by themselves).
    // Modifiers we know about:
    //   `es-*` (babel-spanish: es-tabla, es-cuadro, es-noindentfirst, …)
    // Driver: 2102.11084 `\usepackage[spanish, es-tabla]{babel}` selected
    // "es-tabla" as the language → "haven't defined the language 'es-tabla'"
    // GenericError.
    let is_lang_candidate = |s: &str| -> bool {
      !s.is_empty() && s != "nil" && !s.contains('=')
        && !s.starts_with("es-")
    };
    let pkg_last = main_kv.unwrap_or_else(|| {
      opt_babel.split(',').map(str::trim)
        .filter(|s| is_lang_candidate(s))
        .next_back().unwrap_or_default().to_string()
    });
    let lang = if !pkg_last.is_empty() {
      pkg_last
    } else if main != "nil" && !main.is_empty() {
      main
    } else {
      return Ok(vec![]);
    };
    let iso = crate::package::babel_support_sty::babel_language_to_iso(&lang);
    if let Some(code) = iso {
      state::assign_value("DOCUMENT_LANGUAGE",
        Stored::from(code.to_string()), Some(Scope::Global));
      merge_font(Font { language: Some(Cow::Owned(code.to_string())), ..Font::default() });
    }
    // Force-set \bbl@main@language globally so babel's AtBeginDocument
    // \selectlanguage{\bbl@main@language} picks up the user's intended
    // main language (the LAST option), not whichever .ldf's \ldf@finish
    // ran last. Babel's own chain then handles captions activation,
    // active-char shorthands, and per-language port dispatching.
    def_macro(T_CS!("\\bbl@main@language"), None,
      Tokens!(Explode!(lang.clone())),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..ExpandableOptions::default() }))?;
  });
  // Run mainlang at load time so DOCUMENT_LANGUAGE is set before
  // \begin{document} opens (and base_schema's after_open reads it).
  RawTeX!(r"\lx@babel@activate@mainlang");

});
