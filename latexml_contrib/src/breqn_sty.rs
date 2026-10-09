use latexml_package::prelude::*;

LoadDefinitions!({
  // breqn.sty:56 `\RequirePackage{keyval,calc}`: a document can rely on them
  // (OXIDIZED_DESIGN #317).
  RequirePackage!("keyval");
  RequirePackage!("calc");
  Warn!(
    "missing_file",
    "breqn.sty",
    "breqn.sty is not implemented and will not be interpreted raw."
  );
  // Forbid loading this package, even locally, until we can implement it natively
  DefMacro!("\\condition", "\\text");
  DefMacro!("\\hiderel{}", "#1");
  DefMacro!(
    T_CS!("\\begin{dmath}"),
    "OptionalUndigested",
    "\\begin{equation}"
  );
  DefMacro!(T_CS!("\\end{dmath}"), None, "\\end{equation}");
  DefMacro!(
    T_CS!("\\begin{dmath*}"),
    "OptionalUndigested",
    "\\begin{equation*}"
  );
  DefMacro!(T_CS!("\\end{dmath*}"), None, "\\end{equation*}");
  // breqn.sty loads mathstyle.sty, whose default option `mathactivechars` (mathstyle.sty:233-236, :250-261) makes `_`
  // and `^` math-active (mathcode "8000, the active forms `\sb`/`\sp`) and, from `\begin{document}`, catcode 12
  // ("other") in text: a bare URL `https://…/boiler_room` prints its `_` (2402.04396, 2403.03720; was an error). breqn's
  // `mathstyleoff` (breqn.sty:42-44) reaches mathstyle as `noactivechars` (flexisym.sty:401-402): catcodes 7 and 8
  // again. Read raw, mathstyle.sty errs ("\over no longer primitive").
  DeclareOption!(
    "mathstyleoff",
    r"\def\lx@breqn@catcodes{\catcode`\^=7\relax \catcode`\_=8\relax}"
  );
  ProcessOptions!();
  RawTeX!(
    r#"\mathcode`\_="8000 \mathcode`\^="8000
\begingroup\catcode`\_=\active\catcode`\^=\active \global\let_=\sb \global\let^=\sp \endgroup
\providecommand\lx@breqn@catcodes{\catcode`\^=12\relax \catcode`\_=12\relax}
\AtBeginDocument{\lx@breqn@catcodes}"#
  );
});
