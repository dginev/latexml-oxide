use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("ifplatform");
  RequirePackage!("xcolor");
  RequirePackage!("lineno");
  RequirePackage!("framed");
  RequirePackage!("newfloat");
  RequirePackage!("calc");
  RequirePackage!("kvoptions");
  RequirePackage!("etoolbox");
  RequirePackage!("fancyvrb");
  // Stub as listing for now
  RequirePackage!("listings");
  DeclareOption!("chapter", "");
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
  use latexml_core::binding::content::find_file;
  DefMacro!("\\inputminted[]{}{}", sub[(_opts, _lang, file_arg)] {
    let file_str = file_arg.to_string();
    let contents = find_file(&file_str, None)
      .and_then(|path| std::fs::read_to_string(&path).ok())
      .unwrap_or_default();
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
    // Beyond-Perl frozencache color path (see \begin{minted}); falls back to the
    // uncolored listings display on a cache miss / no `_minted/`.
    let mut result = match crate::minted_frozencache::colored_display_body(&contents) {
      Some(body) => lst_process_display_with(None, &contents, body),
      None => lst_process_display(None, &contents),
    };
    // lst_process_display ends with T_END to balance bgroup we opened above
    // (mirrors `\begin{lstlisting}`'s convention).
    if !matches!(result.last(), Some(t) if t.get_catcode() == Catcode::END) {
      result.push(T_END!());
    }
    Ok(Tokens::new(result))
  });
  def_macro_noop("\\listoflistings")?;
  DefMacro!("\\listingscaption", "Listing");
  DefMacro!("\\listoflistingscaption", "List of listings");
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
  RawTeX!(
    r#"\def\newmint{\@ifnextchar[\lx@minted@nm@opt\lx@minted@nm@noopt}
\def\lx@minted@nm@noopt#1#2{\expandafter\def\csname #1\endcsname{\lstinline[language=#1]}}
\def\lx@minted@nm@opt[#1]#2#3{\expandafter\def\csname #1\endcsname{\lstinline[language=#2]}}
\def\newmintinline{\@ifnextchar[\lx@minted@nmi@opt\lx@minted@nmi@noopt}
\def\lx@minted@nmi@noopt#1#2{\expandafter\def\csname #1inline\endcsname{\lstinline[language=#1]}}
\def\lx@minted@nmi@opt[#1]#2#3{\expandafter\def\csname #1\endcsname{\lstinline[language=#2]}}
\def\newminted{\@ifnextchar[\lx@minted@nmd@opt\lx@minted@nmd@noopt}
\def\lx@minted@nmd@noopt#1#2{\lx@minted@nmd@def{#1code}{#1}}
\def\lx@minted@nmd@opt[#1]#2#3{\lx@minted@nmd@def{#1}{#2}}
\def\lx@minted@nmd@def#1#2{%
  \lstnewenvironment{#1}{\lstset{language=#2}}{}%
  \lstnewenvironment{#1*}[1]{\lstset{language=#2,##1}}{}}
\def\newmintedfile{\@ifnextchar[\lx@minted@nmf@opt\lx@minted@nmf@noopt}
\def\lx@minted@nmf@opt[#1]#2{\let#1\inputminted}
\def\lx@minted@nmf@noopt#1{\expandafter\let\csname #1file\endcsname\inputminted}
"#
  );
  def_macro_noop("\\setminted[]{}")?;
  def_macro_noop("\\setmintedinline[]{}")?;
  def_macro_noop("\\usemintedstyle[]{}")?;
  def_macro_noop("\\SetupFloatingEnvironment{}{}")?;
  DefMacro!("\\mint[]{}", "\\lstinline[language=#2]");
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
          out.extend(Tokenize!("language=").unlist());
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
        let body = listings_read_raw_string(until.as_ref(), &saved);
        pop_frame()?;
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
    listings_read_raw_lines, listings_read_raw_string, lst_process_display,
    lst_process_display_with,
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
        // Feed `language=<lang>,<opts>` to `\lstset`. The language activates the
        // listings language pack for keyword/string/comment highlighting; matching
        // is case-insensitive and an unknown language (e.g. `lean4`) is a silent
        // no-op, so passing minted's Pygments lexer name verbatim degrades to the
        // old plain rendering rather than erroring. The `[opts]` still ride along
        // for `escapeinside`/`mathescape` (html_feedback#1028); minted-only keys
        // (`linenos`, `fontsize`, …) are stored harmlessly.
        let mut kv: Vec<Token> = Vec::new();
        if let Some(lang) = lang.as_ref().filter(|l| !l.unlist_ref().is_empty()) {
          kv.extend(Tokenize!("language=").unlist());
          kv.extend(lang.unlist_ref().iter().cloned());
        }
        if let Some(opts) = opts.as_ref().filter(|o| !o.unlist_ref().is_empty()) {
          if !kv.is_empty() {
            kv.push(T_OTHER!(","));
          }
          kv.extend(opts.unlist_ref().iter().cloned());
        }
        if !kv.is_empty() {
          let _ = digest(Tokens::new(
            std::iter::once(T_CS!("\\lstset"))
              .chain(std::iter::once(T_BEGIN!()))
              .chain(kv)
              .chain(std::iter::once(T_END!()))
              .collect(),
          ));
        }
        let text = listings_read_raw_lines("minted");
        // Beyond-Perl: if a `_minted/` frozencache holds this block's Pygments
        // colors (content-match on the normalized body), emit colored listing
        // lines; otherwise keep the exact uncolored listings path.
        // (OXIDIZED_DESIGN_DIVERGENCES #157.)
        let result = match crate::minted_frozencache::colored_display_body(&text) {
          Some(body) => lst_process_display_with(None, &text, body),
          None => lst_process_display(None, &text),
        };
        Ok(Tokens::new(result))
      },
    )));
    def_macro(cs, params, expansion, None)?;
  }
  // No \end{minted}: mintedEnvBody fully consumed it.
  DefMacro!(T_CS!("\\begin{listing}"), None, "\\begin{figure}");
  DefMacro!(T_CS!("\\end{listing}"), None, "\\end{figure}");
});
