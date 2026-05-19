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
    state::assign_value(
      "current_environment",
      Stored::String(arena::pin("lstlisting")),
      None,
    );
    def_macro(
      T_CS!("\\@currenvir"),
      None,
      Tokens!(T_OTHER!("lstlisting")),
      None,
    )?;
    let mut result = lst_process_display(None, &contents);
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
  // \csname <name>\endcsname` so the same user input now binds working
  // aliases. `\newmint{foo}{opts}` → `\foo` behaves like `\verb`;
  // `\newmintinline{foo}{opts}` → `\fooinline`; `\newminted{foo}{opts}`
  // → `\begin{foo}`/`\begin{foo*}` expand to `\begin{lstlisting}` (since
  // listings is the Perl-chosen substrate on L30). `\newmintedfile` binds
  // either the given optional macro or `\<lang>file` to `\inputminted`.
  RawTeX!(
    r#"\def\newmint#1#2{\expandafter\def\csname #1\endcsname{\verb}}
\def\newmintinline#1#2{\expandafter\def\csname #1inline\endcsname{\verb}}
\def\newminted#1#2{%
  \expandafter\def\csname #1\endcsname{\begin{lstlisting}}%
  \expandafter\def\csname end#1\endcsname{\end{lstlisting}}%
  \expandafter\def\csname #1*\endcsname{\begin{lstlisting}}%
  \expandafter\def\csname end#1*\endcsname{\end{lstlisting}}}
\def\newmintedfile{\@ifnextchar[\lx@minted@nmf@opt\lx@minted@nmf@noopt}
\def\lx@minted@nmf@opt[#1]#2{\let#1\inputminted}
\def\lx@minted@nmf@noopt#1{\expandafter\let\csname #1file\endcsname\inputminted}
"#
  );
  def_macro_noop("\\setminted[]{}")?;
  def_macro_noop("\\setmintedinline[]{}")?;
  def_macro_noop("\\usemintedstyle[]{}")?;
  def_macro_noop("\\SetupFloatingEnvironment{}{}")?;
  DefMacro!("\\mint[]{}", "\\verb");
  DefMacro!("\\mintinline[]{}", "\\verb");
  // \begin{minted}[opts]{language} — port of Perl mintedEnvBody
  // (minted.sty.ltxml L68-85). Read raw input lines until \end{minted},
  // then dispatch to listings' lst_process_display, mirroring Perl's
  // `bgroup; current_environment=lstlisting; lstProcessDisplay(...)`.
  // Without this, the legacy `\begin{minted} -> \begin{lstlisting}`
  // expansion lets lstlisting swallow the rest of the file because
  // it never sees its own `\end{lstlisting}` marker (the user wrote
  // `\end{minted}`).
  use latexml_package::package::listings_sty::{listings_read_raw_lines, lst_process_display};
  use latexml_core::stomach::bgroup;
  {
    let cs = T_CS!("\\begin{minted}");
    let params = parse_parameters("[]{}", &cs, true)?;
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |_args: Vec<ArgWrap>| {
        bgroup();
        state::assign_value(
          "current_environment",
          Stored::String(arena::pin("lstlisting")),
          None,
        );
        def_macro(
          T_CS!("\\@currenvir"),
          None,
          Tokens!(T_OTHER!("lstlisting")),
          None,
        )?;
        let text = listings_read_raw_lines("minted");
        let result = lst_process_display(None, &text);
        Ok(Tokens::new(result))
      },
    )));
    def_macro(cs, params, expansion, None)?;
  }
  // No \end{minted}: mintedEnvBody fully consumed it.
  DefMacro!(T_CS!("\\begin{listing}"), None, "\\begin{figure}");
  DefMacro!(T_CS!("\\end{listing}"), None, "\\end{figure}");
});
