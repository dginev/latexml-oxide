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
  // TODO: Perl has complex \inputminted closure that reads file contents
  // and wraps in \begin{minted}...\end{minted}. Stubbed for now.
  DefMacro!("\\inputminted[]{}{}", "");
  DefMacro!("\\listoflistings", "");
  DefMacro!("\\listingscaption", "Listing");
  DefMacro!("\\listoflistingscaption", "List of listings");
  // TODO: Perl has \newmint, \newmintinline, \newminted, \newmintedfile
  // primitives that dynamically define new macros. Stubbed for now.
  DefMacro!("\\newmint{}{}", "");
  DefMacro!("\\newmintinline{}{}", "");
  DefMacro!("\\newminted{}{}", "");
  DefMacro!("\\newmintedfile[]{}{}", "");
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
