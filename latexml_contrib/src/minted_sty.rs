use latexml_package::{
  package::listings_sty::{
    listings_read_raw_file, lst_group_opener, lst_process_display_scoped, lst_scoped,
  },
  prelude::*,
};

/// The listings keys every minted body starts from, minted's own defaults (minted.sty:1005 `mathescape=false`, :1013
/// `texcl=false`, :1019 `escapeinside=`; minted has no `escapechar`): minted's options are its own, so a document's
/// `\lstset{mathescape=true}` for its `{lstlisting}`s must not reach the minted bodies listings stands in for here —
/// the `$` of a shell `sed "s/$/…/"` opened math (1812.09490, whose program.sty then made it a Fatal). A minted
/// body's own `[mathescape]`/`[escapeinside=!!]` follow them (html_feedback#1028). Guard
/// `perfect_kernel_batch77q::minted_body_ignores_the_document_listings_escapes`.
const MINTED_LST_RESET: &str = "mathescape=false,escapechar={},escapeinside={},texcl=false";

LoadDefinitions!({
  // minted.sty:282 reads its options with `\ProcessPgfOptions`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  RequirePackage!("ifplatform");
  RequirePackage!("xcolor");
  RequirePackage!("lineno");
  RequirePackage!("framed");
  // minted.sty:155-165 `chapter`/`section` (the listing counter's `within`) and `newfloat`, read with
  // `\ProcessPgfOptions` (:282), the last one given winning; :284 `newfloat` or `float` for the `listing` float type.
  let (minted_newfloat, minted_within) = minted_float_options();
  if minted_newfloat {
    RequirePackage!("newfloat");
  } else {
    RequirePackage!("float");
  }
  RequirePackage!("calc");
  RequirePackage!("kvoptions");
  RequirePackage!("etoolbox");
  RequirePackage!("fancyvrb");
  // minted.sty:32 `\RequirePackage{fvextra}` (minted 2.2+ too): `Verbatim`'s `breaklines`/`breakanywhere` keys
  // are fvextra's; with fancyvrb alone they were undefined keys (2512.12360; Perl's ar5iv-bindings
  // minted.sty.ltxml:24 has the line commented out, KNOWN_PERL_ERRORS #916). Guard:
  // `perfect_kernel_batch70a::minted_loads_fvextra`.
  RequirePackage!("fvextra");
  // Stub as listing for now
  RequirePackage!("listings");
  DeclareOption!("chapter", "");
  // minted reads its options as keys; the handler is not left defined for a later package's `chapter` option.
  let_i(&T_CS!("\\ds@chapter"), &T_RELAX!(), None);
  // INCOMPLETE IMPLEMENTATION — mostly stubs that allow content-preservation
  def_macro_noop("\\DeleteFile[]{}")?;
  DefMacro!("\\MintedPygmentize", "pygmentize");
  def_macro_noop("\\ProvideDirectory{}")?;
  def_macro_noop("\\TestAppExists{}")?;
  DefConditional!("\\ifAppExists");

  // minted.sty L273: the cache/output directory, empty by default. Poked
  // by tcolorbox's minted listing engine and by docs configuring caches
  // (sweep-11 cluster: 18 doc-corpus manuals, witness csvsimple/csvsimple).
  DefMacro!("\\minted@outputdir", "");

  // `\minted@def@optcl[default]{name}{cmdline}{value}` — minted's
  // internal option-class registry (TL minted.sty L260+). Used by
  // tcolorbox's tcbminted.code.tex to register tcb's minted-options
  // — drives the cascade for arXiv:2602.00513 (paper using
  // `\usepackage{tcolorbox}` with the `minted` library).
  // No-op stub: tcolorbox just calls these to register options that
  // would otherwise be picked up by our \mint/\inputminted stubs.
  def_macro_noop("\\minted@def@optcl[]{}{}{}")?;
  def_macro_noop("\\minted@def@optcl@e[]{}{}{}")?;
  def_macro_noop("\\minted@def@optcl@switch{}{}")?;
  // \inputminted[opts]{language}{filename} — Perl L43-53 reads the
  // referenced file via FindFile + Mouth->readRawLine, then wraps the
  // contents in \begin{minted}{language}...\end{minted}, relying on
  // mintedEnvBody to read those tokens via `gullet->readUntil(T_CS('\end'))`.
  //
  // We can't go through \begin{lstlisting} the same way: lstlisting's
  // body reader is `listings_read_raw_lines("lstlisting")` which reads
  // RAW LINES FROM THE MOUTH (the underlying TeX source), not from
  // tokens we've pushed back into the gullet. Routing \inputminted
  // through `\begin{lstlisting} ... \end{lstlisting}` therefore
  // discards the file content and instead consumes lines from the
  // surrounding `.tex` file until it hits `\end{lstlisting}` —
  // which never appears, so it eats the rest of the document.
  // Driver: 1903.09408 (\inputminted inside \begin{listing} broke
  // section nesting because \end{listing} got swallowed too).
  //
  // Solution: bypass \begin{lstlisting} entirely and call
  // lst_process_display directly with the file contents, mirroring
  // what `\begin{lstlisting}` does internally but with our string in
  // place of the read_raw_lines call.
  DefMacro!("\\inputminted[]{}{}", sub[(_opts, _lang, file_arg)] {
    // The file name is expanded, as minted's `\input` of it does and as
    // `\lstinputlisting` does: tcolorbox's minted library reads a listing back
    // from `\minted@outputdir <jobname>.listing` (tcbminted.code.tex:49-55), and
    // the unexpanded `\minted@outputdir` missed the file, an empty listing
    // (tkz-grapheur-examples-integrals, 32 manuals of sweep #122; Perl alike).
    let file_str = Expand!(file_arg).to_string();
    // Read as `\lstinputlisting` reads (Perl listingsReadRawFile): the
    // session's virtual file store first (a file written by
    // `filecontents`/`\VerbatimOut` lives only there, batch 56cv), then
    // `find_file` — whose extensionless hit may itself be a store key
    // (tutodoc-en/fr) — and an `I/O` error when the file is nowhere (minted.sty
    // :1420 `\minted@error{Cannot find input file …}`).
    let contents = listings_read_raw_file(file_str.trim())?.unwrap_or_default();
    bgroup();
    let mut reset = vec![T_CS!("\\lstset"), T_BEGIN!()];
    reset.extend(Tokenize!(MINTED_LST_RESET).unlist());
    reset.push(T_END!());
    digest(Tokens::new(reset))?;
    assign_value(
      "current_environment",
      Stored::String(pin("lstlisting")),
      None,
    );
    def_macro(
      T_CS!("\\@currenvir"),
      None,
      Tokens!(T_OTHER!("lstlisting")),
      None,
    )?;
    // Beyond-Perl frozencache color path (see \begin{minted}); falls back to the
    // uncolored listings display on a cache miss / no `_minted/`.
    let result = match crate::minted_frozencache::colored_display_body(&contents) {
      Some(body) => lst_scoped(lst_process_display_with(None, &contents, body)),
      None => lst_process_display_scoped(None, &contents),
    };
    // The expansion-time `bgroup()` above only scopes the key activation and
    // is closed HERE; the listing's own group is the `\begingroup` TOKEN +
    // `\lx@lst@activate` emitted in stream order and closed by `\endgroup`
    // (`lst_scoped`) — `\begin{lstlisting}`'s 54x/55k contract. The former
    // align-neutral `\lx@hidden@egroup` closer (a) once got a `}` appended
    // that closed a tcolorbox/minipage around `\inputminted` (26-doc sweep-37
    // regression: algxpar-doc, tikzducks-doc, biblatex-oxref ×4, tcolorbox
    // posters) and (b) in a `p{}` cell hit the VBoxContents mode-switch frame
    // ("`\lx@hidden@egroup` Attempt to close a group that switched to mode
    // internal_vertical"). Guards:
    // `perfect_kernel_batch54::{inputminted_inside_a_minipage_keeps_its_box,
    // minted_in_a_p_column_keeps_the_cell}`.
    egroup()?;
    let mut out = lst_group_opener("lstlisting", None)?;
    out.extend(result);
    Ok(Tokens::new(out))
  });
  def_macro_noop("\\listoflistings")?;
  DefMacro!("\\listingscaption", "Listing");
  DefMacro!("\\listoflistingscaption", "List of Listings");
  // Perl minted.sty.ltxml L58-99 dynamically defined new CSes via runtime
  // DefMacroI closures. The TeX-level equivalent uses `\expandafter\def
  // \csname <name>\endcsname` so the same user input now binds working aliases.
  //
  // Each of `\newmint` / `\newmintinline` / `\newminted` / `\newmintedfile` takes
  // an OPTIONAL `[name]` before the two mandatory `{language}{options}` args (real
  // minted.sty: `\newcommand{..}[3][]`, each with its own default name). The prior
  // port declared the first three as two-mandatory `#1#2`, so
  // `\newminted[leancode]{lean4}{...}` bound `#1 = "["` and then ran
  // `\expandafter\def\csname [\endcsname{...}` — but `\csname [\endcsname` IS the
  // control sequence `\[`, silently redefining display-math open to
  // `\begin{lstlisting}`; every later `\[` opened a listing that ran to EOF,
  // swallowing the body and `\bibliography` with no `Error:` (arXiv:2606.05629,
  // issue #520). Each of the three now branches on `\@ifnextchar[` (as
  // `\newmintedfile` already did) and consumes all three args, so the optional
  // env/command name binds correctly and nothing (least of all `\[`) leaks. Default names match
  // minted: `\newmint{lang}` → `\lang`; `\newmintinline{lang}` → `\langinline`;
  // `\newminted{lang}` → env `langcode`; the optional form uses the given name
  // verbatim.
  //
  // The inline commands (`\mint`, `\mintinline`, and the `\newmint`/`\newmintinline`
  // aliases) route to `\lstinline[language=<lang>]`, NOT `\verb`: `\verb` is
  // delimiter-based, so the common `\mintinline{lang}{code}` BRACE form read `{` as
  // the delimiter and ran past the closing `}` (garbled code + unbalanced-group
  // errors that broke following content); `\lstinline` accepts braces and, given the
  // language, highlights the snippet. listings is the Perl-chosen substrate (L30),
  // so a `\newminted` env delegates to `\lstnewenvironment{name}` (as the tcolorbox
  // binding's `\newtcblisting` does) — whose verbatim reader stops at the env's own
  // `\end{name}` (the prior alias to `\begin{lstlisting}` read raw until the literal
  // `\end{lstlisting}`, so a `\newminted` env used normally ran to EOF) — and its
  // start code activates the language via `\lstset{language=<lang>}` for keyword/
  // string/comment highlighting. Language matching is case-insensitive and an unknown
  // language (e.g. `lean4`) is a silent no-op, so minted's Pygments lexer name passes
  // through verbatim and degrades to plain text rather than erroring. `\newmintedfile`
  // binds either the given optional macro or `\<lang>file` to `\inputminted`.
  RawTeX!(&r#"\def\newmint{\@ifnextchar[\lx@minted@nm@opt\lx@minted@nm@noopt}
\def\lx@minted@nm@noopt#1#2{\expandafter\def\csname #1\endcsname{\lstinline[@RESET@,language=#1]}}
\def\lx@minted@nm@opt[#1]#2#3{\expandafter\def\csname #1\endcsname{\lstinline[@RESET@,language=#2]}}
\def\newmintinline{\@ifnextchar[\lx@minted@nmi@opt\lx@minted@nmi@noopt}
\def\lx@minted@nmi@noopt#1#2{\expandafter\def\csname #1inline\endcsname{\lstinline[@RESET@,language=#1]}}
\def\lx@minted@nmi@opt[#1]#2#3{\expandafter\def\csname #1\endcsname{\lstinline[@RESET@,language=#2]}}
\def\newminted{\@ifnextchar[\lx@minted@nmd@opt\lx@minted@nmd@noopt}
\def\lx@minted@nmd@noopt#1#2{\lx@minted@nmd@def{#1code}{#1}}
\def\lx@minted@nmd@opt[#1]#2#3{\lx@minted@nmd@def{#1}{#2}}
\def\lx@minted@nmd@def#1#2{%
  \lstnewenvironment{#1}{\lstset{@RESET@,language=#2}}{}%
  \lstnewenvironment{#1*}[1]{\lstset{@RESET@,language=#2,##1}}{}}
\def\newmintedfile{\@ifnextchar[\lx@minted@nmf@opt\lx@minted@nmf@noopt}
\def\lx@minted@nmf@opt[#1]#2{\let#1\inputminted}
\def\lx@minted@nmf@noopt#1{\expandafter\let\csname #1file\endcsname\inputminted}
"#.replace("@RESET@", MINTED_LST_RESET));
  def_macro_noop("\\setminted[]{}")?;
  def_macro_noop("\\setmintedinline[]{}")?;
  def_macro_noop("\\usemintedstyle[]{}")?;
  def_macro_noop("\\SetupFloatingEnvironment{}{}")?;
  DefMacro!("\\mint[]{}", sub[(_opts, lang)] {
    let mut out = vec![T_CS!("\\lstinline"), T_OTHER!("[")];
    out.extend(Tokenize!(MINTED_LST_RESET).unlist());
    out.extend(Tokenize!(",language=").unlist());
    out.extend(lang.unlist());
    out.push(T_OTHER!("]"));
    Tokens::new(out)
  });
  // `\mintinline[opts]{lang}{code}` — beyond-Perl frozencache color path.
  // When NO `_minted/` frozencache is present the code arg is left untouched in
  // the stream and we reconstruct the historical `\lstinline[language=<lang>]`
  // expansion (byte-for-byte the old behavior, so non-frozencache papers are
  // unaffected). When a frozencache IS present we read the `{code}` verbatim
  // (mirroring `\lx@lstinline`) and emit a colored inline `ltx_lstlisting` span
  // on a content-match hit, or the plain code on a miss. (OXIDIZED_DESIGN_
  // DIVERGENCES #157.)
  {
    let cs = T_CS!("\\mintinline");
    let params = parse_parameters("[]{}", &cs, true)?;
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        let mut it = args.into_iter();
        let _opts = it.next();
        let lang = it.next().and_then(|a| a.owned_tokens()).unwrap_or_default();
        if !crate::minted_frozencache::cache_available() {
          // Delegate to \lstinline, which reads the following {code} verbatim.
          let mut out = vec![T_CS!("\\lstinline"), T_OTHER!("[")];
          out.extend(Tokenize!(MINTED_LST_RESET).unlist());
          out.extend(Tokenize!(",language=").unlist());
          out.extend(lang.unlist());
          out.push(T_OTHER!("]"));
          return Ok(Tokens::new(out));
        }
        // Frozencache present: read the verbatim code ourselves (mirror
        // \lx@lstinline listings.sty:2076-2108).
        let init = read_token()?;
        let until = init.as_ref().map(|t| {
          if t.get_catcode() == Catcode::BEGIN {
            T_END!()
          } else {
            *t
          }
        });
        let verbatim_chars = ['%', '\\', '{', '}', '$', '&', '#', '^', '_', '~'];
        let saved: Vec<(char, Catcode)> = verbatim_chars
          .iter()
          .map(|&c| (c, lookup_catcode(c).unwrap_or(Catcode::OTHER)))
          .collect();
        push_frame();
        for c in verbatim_chars {
          assign_catcode(c, Catcode::OTHER, Some(Scope::Local));
        }
        let (body, close_counted) = listings_read_raw_string(until.as_ref(), &saved);
        pop_frame()?;
        // The `{` delimiter's count, as \lx@lstinline (the `{…}` output below re-counts as a pair).
        settle_verbatim_brace_ledger(init.as_ref(), close_counted);
        let segs = crate::minted_frozencache::inline_body(&body);
        let mut out = vec![
          T_CS!("\\leavevmode"),
          T_CS!("\\@listings@inline"),
          T_BEGIN!(),
        ];
        out.extend(segs);
        out.push(T_END!());
        Ok(Tokens::new(out))
      },
    )));
    def_macro(cs, params, expansion, None)?;
  }
  // \begin{minted}[opts]{language} — port of Perl mintedEnvBody
  // (minted.sty.ltxml L68-85). Read raw input lines until \end{minted},
  // then dispatch to listings' lst_process_display, mirroring Perl's
  // `bgroup; current_environment=lstlisting; lstProcessDisplay(...)`.
  // Without this, the legacy `\begin{minted} -> \begin{lstlisting}`
  // expansion lets lstlisting swallow the rest of the file because
  // it never sees its own `\end{lstlisting}` marker (the user wrote
  // `\end{minted}`).
  use latexml_core::stomach::bgroup;
  use latexml_package::package::listings_sty::{
    listings_read_raw_lines, listings_read_raw_string, lst_process_display_with,
    settle_verbatim_brace_ledger,
  };
  {
    let cs = T_CS!("\\begin{minted}");
    // `\begin{minted}[opts]{language}`. Previously the `[opts]` were dropped, so
    // `escapeinside=!!` never reached the listings tokenizer: an inline
    // `!$\label{…}$!` was emitted as literal code, its `\label` never ran, and
    // `\ref` to that line vanished (html_feedback#1028, arXiv:2308.03276;
    // OXIDIZED_DESIGN #127). We now feed both the `{language}` and the options
    // through `\lstset{…}` (the same activation `\begin{lstlisting}` uses), so
    // minted's `escapeinside`/`mathescape` — shared verbatim with the listings
    // substrate — take effect and the language activates listings' keyword/string/
    // comment highlighting; minted-only keys (`linenos`, `fontsize`, …) are stored
    // harmlessly. Scoped to the environment's `bgroup`.
    let params = parse_parameters("[]{}", &cs, true)?;
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        // args[0] = `[opts]` (optional); args[1] = `{language}`.
        let mut it = args.into_iter();
        let opts = it.next().and_then(|a| a.owned_tokens());
        let lang = it.next().and_then(|a| a.owned_tokens());
        bgroup();
        assign_value(
          "current_environment",
          Stored::String(pin("lstlisting")),
          None,
        );
        def_macro(
          T_CS!("\\@currenvir"),
          None,
          Tokens!(T_OTHER!("lstlisting")),
          None,
        )?;
        // Feed `<MINTED_LST_RESET>,language=<lang>,<opts>` to `\lstset`. The language activates the
        // listings language pack for keyword/string/comment highlighting; matching
        // is case-insensitive and an unknown language (e.g. `lean4`) is a silent
        // no-op, so passing minted's Pygments lexer name verbatim degrades to the
        // old plain rendering rather than erroring. The `[opts]` still ride along
        // for `escapeinside`/`mathescape` (html_feedback#1028); minted-only keys
        // (`linenos`, `fontsize`, …) are stored harmlessly.
        let mut kv: Vec<Token> = Tokenize!(MINTED_LST_RESET).unlist();
        if let Some(lang) = lang.as_ref().filter(|l| !l.unlist_ref().is_empty()) {
          kv.extend(Tokenize!(",language=").unlist());
          kv.extend(lang.unlist_ref().iter().cloned());
        }
        if let Some(opts) = opts.as_ref().filter(|o| !o.unlist_ref().is_empty()) {
          kv.push(T_OTHER!(","));
          kv.extend(opts.unlist_ref().iter().cloned());
        }
        let _ = digest(Tokens::new(
          std::iter::once(T_CS!("\\lstset"))
            .chain(std::iter::once(T_BEGIN!()))
            .chain(kv)
            .chain(std::iter::once(T_END!()))
            .collect(),
        ));
        // fancyvrb.sty:295-297/386-403: a user environment that wraps minted
        // with `\VerbatimEnvironment` (RetoMatematico.cls:174 `{codigo}` =
        // `{\VerbatimEnvironment\begin{minted}[…]{#1}}{\end{minted}}`) names the
        // environment whose `\end` closes the verbatim body — the OUTERMOST, as
        // `\FV@EnvironName` is set only while still `\relax` — and the name is
        // reset once read. Reading to a literal `\end{minted}` ran past
        // `\end{codigo}` and swallowed the rest of the document
        // (retomatematico-ejemplo, recall 53 %). Guard:
        // `perfect_kernel_batch56::verbatim_environment_wrapper_ends_minted`.
        let fv_name = T_CS!("\\FV@EnvironName");
        let env_name =
          if lookup_meaning(&fv_name).is_some() && !x_equals(&fv_name, &T_CS!("\\relax")) {
            let name = Expand!(Tokens!(fv_name)).to_string();
            Let!(fv_name, T_CS!("\\relax"), Scope::Global);
            name
          } else {
            "minted".to_string()
          };
        let text = listings_read_raw_lines(&env_name);
        // Beyond-Perl: if a `_minted/` frozencache holds this block's Pygments
        // colors (content-match on the normalized body), emit colored listing
        // lines; otherwise keep the exact uncolored listings path.
        // (OXIDIZED_DESIGN_DIVERGENCES #157.)
        let result = match crate::minted_frozencache::colored_display_body(&text) {
          Some(body) => lst_scoped(lst_process_display_with(None, &text, body)),
          None => lst_process_display_scoped(None, &text),
        };
        // As `\inputminted`: close the key-scoping `bgroup()` here and emit
        // the `\begingroup` token opener for the listing's own group.
        egroup()?;
        let mut out = lst_group_opener("minted", None)?;
        out.extend(result);
        let mut end_tokens = vec![T_CS!("\\end"), T_BEGIN!()];
        end_tokens.extend(ExplodeText!(env_name));
        end_tokens.push(T_END!());
        unread_expansion(Tokens::new(end_tokens));
        Ok(Tokens::new(out))
      },
    )));
    def_macro(cs, params, expansion, None)?;
    DefMacro!(T_CS!("\\end{minted}"), None, Tokens!());
    DefMacro!(T_CS!("\\endminted"), None, Tokens!());
  }
  // minted.sty:1931-1944: `listing` is a float type of its own, numbered apart from the figures (within the
  // `section`/`chapter` option's counter): with the `newfloat` option newfloat's `\DeclareFloatingEnvironment
  // [fileext=lol,placement=tbp]{listing}` (2411.16612), else float.sty's `\newfloat{listing}{tbp}{lol}` named
  // `\floatname{listing}{\listingscaption}`, which a `\renewcommand{\listingscaption}{…}` renames ("JSON 1",
  // 1902.08498; "Code snippet 1", 2609.21330; "Listing 1: …", 2102.00527). Perl's ar5iv-bindings minted.sty.ltxml:110-111
  // make it a `{figure}`, so a listing printed "Figure 1" and took the figures' numbers, and wrapfig's
  // `{wrapfloat}{listing}` met a float type with no counter (KNOWN_PERL_ERRORS #1123). Guards
  // `perfect_kernel_batch78s::{minted_listing_is_a_float_of_its_own, minted_listing_is_float_sty_s_by_default}`.
  let within = minted_within
    .map(|within| s!("within={within},"))
    .unwrap_or_default();
  if minted_newfloat {
    RawTeX!(&s!(
      r"\DeclareFloatingEnvironment[{within}fileext=lol,placement=tbp]{{listing}}"
    ));
  } else {
    let within = minted_within
      .map(|within| s!("[{within}]"))
      .unwrap_or_default();
    RawTeX!(&s!(
      r"\newfloat{{listing}}{{tbp}}{{lol}}{within}\floatname{{listing}}{{\listingscaption}}"
    ));
  }
});

/// minted's float options (minted.sty:155-165, read by `\ProcessPgfOptions{/minted/pkg}`, :282): `newfloat`
/// (`newfloat/.is if`: alone or `=true`; `=false` turns it off) and the listing counter's `within`, `chapter` or
/// `section`; each the last one given.
fn minted_float_options() -> (bool, Option<&'static str>) {
  let mut newfloat = false;
  let mut within = None;
  if let Some(options) = lookup_vecdeque("opt@minted.sty") {
    for option in options.iter() {
      let option = option.to_string();
      let (key, value) = option
        .split_once('=')
        .map_or((option.trim(), "true"), |(key, value)| {
          (key.trim(), value.trim())
        });
      match key {
        "newfloat" => newfloat = value != "false",
        "chapter" => within = Some("chapter"),
        "section" => within = Some("section"),
        _ => {},
      }
    }
  }
  (newfloat, within)
}
