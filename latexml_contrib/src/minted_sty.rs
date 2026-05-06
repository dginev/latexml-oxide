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
  DefMacro!("\\DeleteFile[]{}", "");
  DefMacro!("\\MintedPygmentize", "pygmentize");
  DefMacro!("\\ProvideDirectory{}", "");
  DefMacro!("\\TestAppExists{}", "");
  DefConditional!("\\ifAppExists");
  // \inputminted[opts]{language}{filename} — Perl L43-53 reads the
  // referenced file via FindFile + Mouth->readRawLine, then wraps the
  // contents in \begin{minted}{language}...\end{minted}. The Rust
  // \begin{minted} short-circuits to \begin{lstlisting} (via the
  // listings substrate chosen on Perl L30), so we wrap the contents
  // accordingly. Missing files silently produce an empty listing —
  // matches the Perl branch where FindFile returns undef.
  use latexml_core::binding::content::find_file;
  DefMacro!("\\inputminted[]{}{}", sub[(_opts, _lang, file_arg)] {
    let file_str = file_arg.to_string();
    let mut tokens: Vec<Token> = Vec::new();
    tokens.push(T_CS!("\\begin{lstlisting}"));
    if let Some(path) = find_file(&file_str, None) {
      if let Ok(contents) = std::fs::read_to_string(&path) {
        tokens.extend(Explode!(&contents));
      }
    }
    tokens.push(T_CS!("\\end{lstlisting}"));
    Ok(Tokens::new(tokens))
  });
  DefMacro!("\\listoflistings", "");
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
  DefMacro!("\\setminted[]{}", "");
  DefMacro!("\\setmintedinline[]{}", "");
  DefMacro!("\\usemintedstyle[]{}", "");
  DefMacro!("\\SetupFloatingEnvironment{}{}", "");
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
