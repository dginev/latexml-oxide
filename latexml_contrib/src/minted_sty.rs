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
  // TODO: Perl has complex mintedEnvBody closure for {minted} environment
  // that collects body and delegates to lstlisting. Stubbed for now.
  DefMacro!(T_CS!("\\begin{minted}"), "[]{}", "\\begin{lstlisting}");
  DefMacro!(T_CS!("\\end{minted}"), None, "\\end{lstlisting}");
  DefMacro!(T_CS!("\\begin{listing}"), None, "\\begin{figure}");
  DefMacro!(T_CS!("\\end{listing}"), None, "\\end{figure}");
});
