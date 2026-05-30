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

  // Pre-define `\bbl@main@language` as `english` so papers that load babel
  // transitively (e.g. via aastex62 → revtex4 → bibstyles → babel chain
  // without any explicit `\usepackage[<lang>]{babel}`) don't error with
  // "Token \bbl@main@language is not defined" when `\selectlanguage` or
  // `\lx@babel@activate@mainlang` later expands it. The real `\ldf@finish`
  // overrides this when a language .ldf actually runs. Witness 2301.13322
  // (aastex62 + lipsum + blindtext, no explicit babel options).
  RawTeX!(r"\providecommand\bbl@main@language{english}");

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
    // ALSO drop babel's own bare KEYWORD options — `\DeclareOption{<kw>}` in
    // babel.sty that are package switches, NOT languages (so babel consumes
    // them via their `\ds@` handler and never treats them as a language to
    // load/select). Without this, `\usepackage[english,strings]{babel}`
    // selected the LAST bare option "strings" as the main language →
    // `\selectlanguage{strings}` → "You haven't defined the language
    // 'strings'". babel.sty L296/L336-379 (the no-`=` ones). Driver
    // 2006.10240 (`[english,strings]`).
    const BABEL_KEYWORD_OPTS: &[&str] = &[
      "base", "showlanguages", "KeepShorthandsActive", "activeacute",
      "activegrave", "debug", "noconfigs", "silent", "strings", "nocase",
      "leqno", "fleqn",
    ];
    let is_lang_candidate = |s: &str| -> bool {
      !s.is_empty() && s != "nil" && !s.contains('=')
        && !s.starts_with("es-")
        && !BABEL_KEYWORD_OPTS.contains(&s)
    };
    let pkg_last = main_kv.clone().unwrap_or_else(|| {
      opt_babel.split(',').map(str::trim).rfind(|s| is_lang_candidate(s)).unwrap_or_default().to_string()
    });
    // Pick the active main-lang. The non-trivial case is when the user's
    // OPTION NAME differs from the .ldf's CANONICAL language name —
    // e.g. `\usepackage[russianb]{babel}` loads russianb.ldf which calls
    // `\ldf@finish{russian}`. babel's `\select@language` later needs
    // `\l@<canonical>` (russian), so we must use the canonical name
    // when an alias is in play. But when our Rust binding for a `.ldf`
    // (e.g. french_ldf) bypasses raw load, no `\main@language` runs,
    // and `\bbl@main@language` retains whatever the FIRST option's
    // raw-loaded .ldf set — which is wrong (user wants the LAST option).
    //
    // Heuristic: trust `main` (the `\bbl@main@language` value) only
    // when it maps to the same ISO code as `pkg_last`. That captures
    // the canonical-alias case (russianb → russian, ukrainianb →
    // ukrainian, brazilian → brazil) while still falling through to
    // `pkg_last` for the no-Rust-ldf-binding case (where `main`
    // points at the first option, not the user's intended last one).
    //
    // Witnesses: 2312.08012 (russianb → russian alias, needed canonical),
    // tests/babel/elsart_keyword_brace_form (`[english,french]` →
    // french_ldf binding doesn't update main, but user wants french).
    let same_alias_class = !main.is_empty() && !pkg_last.is_empty()
      && main != pkg_last
      && crate::package::babel_support_sty::babel_language_to_iso(&main)
         == crate::package::babel_support_sty::babel_language_to_iso(&pkg_last)
      && crate::package::babel_support_sty::babel_language_to_iso(&main).is_some();
    let lang = if let Some(m) = main_kv {
      m
    } else if same_alias_class {
      // canonical aliases — use the canonical name from \ldf@finish
      main
    } else if !pkg_last.is_empty() {
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
    // Set \bbl@main@language so babel's AtBeginDocument
    // \selectlanguage{\bbl@main@language} picks up the canonical name.
    def_macro(T_CS!("\\bbl@main@language"), None,
      Tokens!(Explode!(lang.clone())),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..ExpandableOptions::default() }))?;
  });
  // Run mainlang at load time so DOCUMENT_LANGUAGE is set before
  // \begin{document} opens (and base_schema's after_open reads it).
  RawTeX!(r"\lx@babel@activate@mainlang");

  // English-family caption/date/extras hook backfill. Modern babel's .ini
  // path defines the per-variant `\captions<v>`/`\date<v>`/`\extras<v>`
  // hooks only for the variant(s) whose .ini actually loaded (e.g.
  // babel-british for `[british]`). A paper listing SEVERAL english
  // variants — `\usepackage[british,USenglish]{babel}` — then invokes a
  // variant whose .ini never ran (`\dateUSenglish`) or the `\captionsenglish`
  // base, erroring at `\selectlanguage`/option dispatch. The classic .ldf
  // loaders (`babel_lang_stubs::load_*`) also miss this because the .ini
  // path bypasses them, and `english.sty`'s aliasing loop only runs for
  // `\usepackage{english}`, not a direct `\usepackage[...]{babel}`. Backfill
  // with `\@ifundefined` guards: never overrides a real definition; captions
  // stay English (our HTML default); `\date<v>` aliases to `\dateenglish`
  // (keeps `\today` faithful). Runs after babel's own option processing
  // (InputDefinitions above), before the invocation in later preamble.
  // Witness arXiv:1508.06150 (`\usepackage[british,USenglish]{babel}`); Perl rc=0.
  // NB: no \makeatletter/\makeatother — RawTeX already digests with `@` as a
  // letter; emitting \makeatother here would leave `@` catcode-12 globally and
  // break babel's later `\l@<lang>`/`\bbl@…` parsing (manifested as a spurious
  // "haven't defined the language" error).
  RawTeX!(r"%
    \@ifundefined{dateenglish}{\@namedef{dateenglish}{}}{}%
    \@for\lx@bbl@engtmp:={english,USenglish,UKenglish,american,british,canadian,australian,newzealand}\do{%
      \@ifundefined{captions\lx@bbl@engtmp}{\expandafter\let\csname captions\lx@bbl@engtmp\endcsname\@empty}{}%
      \@ifundefined{extras\lx@bbl@engtmp}{\expandafter\let\csname extras\lx@bbl@engtmp\endcsname\@empty}{}%
      \@ifundefined{noextras\lx@bbl@engtmp}{\expandafter\let\csname noextras\lx@bbl@engtmp\endcsname\@empty}{}%
      \@ifundefined{date\lx@bbl@engtmp}{\expandafter\let\csname date\lx@bbl@engtmp\endcsname\dateenglish}{}}");

  // Override `\shorthandoff` / `\shorthandon` to no-op. Babel's raw
  // implementation (babel.sty L1492-1496) iterates the argument and
  // calls `\bbl@switch@sh` for each character, which fires
  // `\PackageError{babel}{I can't switch '<c>' on or off--not a
  // shorthand}` when the character isn't a registered shorthand.
  //
  // Authors call `\shorthandoff{;:!?}` in macros like
  // `\def\diag{\shorthandoff{;:!?}...}` defensively, expecting the
  // French babel shorthand-spacing rules (added by french.ldf when
  // those chars are made active) to be temporarily disabled. Our
  // language stubs (`install_lang_stub` in babel_lang_stubs.rs) don't
  // actually install the shorthand machinery — we mimic Perl's "file
  // missing → skip" behavior for language packages — so the toggle
  // has no shorthand state to switch, and the error fires whenever a
  // multilingual paper invokes the toggle.
  //
  // Shorthand on/off is a typesetting-only concern (controls active-
  // character spacing behavior in TeX); our XML output never observes
  // those decisions, so a no-op is semantically correct for our
  // pipeline. 6 R-stage papers in the babel cluster cleaned by this.
  // Witness arXiv:1912.08056 (`\def\diag{\shorthandoff{;:!?}...}`).
  DefMacro!("\\shorthandoff{}", None);
  DefMacro!("\\shorthandon{}", None);

});
