use crate::prelude::*;

LoadDefinitions!({
  // Redefine \documentclass to do nothing in subfiles
  DefMacro!(
    "\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []",
    ""
  );

  // Fake \begin{document}/\end{document} for subfiles:
  // After the main document starts, nested \begin{document} just opens/closes
  // an internal_vertical mode group, keeping the rest of the main file intact.
  DefPrimitive!("\\lx@subfiles@fake@begindocument", {
    let nesting = lookup_int("subfiles_nesting") + 1;
    AssignValue!("subfiles_nesting" => nesting, Some(Scope::Global));
    // Make \end{document} close this fake \begin{document}
    Let!(
      "\\end{document}",
      "\\lx@subfiles@fake@enddocument",
      Scope::Global
    );
    stomach::begin_mode("internal_vertical")?;
  });

  DefPrimitive!("\\lx@subfiles@fake@enddocument", {
    let nesting = lookup_int("subfiles_nesting") - 1;
    AssignValue!("subfiles_nesting" => nesting, Some(Scope::Global));
    // Make sure the outermost \end{document} invokes our proper \end{document}
    if nesting == 0 {
      Let!(
        "\\end{document}",
        "\\lx@subfiles@saved@enddocument",
        Scope::Global
      );
    }
    stomach::end_mode("internal_vertical")?;
  });

  // AtBeginDocument: after the main document starts, redirect nested
  // \begin{document}/\end{document} to the fake versions
  let _ = state::push_value("@at@begin@document", Tokens!(T_CS!("\\lx@subfiles@setup")));
  DefPrimitive!("\\lx@subfiles@setup", {
    AssignValue!("subfiles_nesting" => 0i64, Some(Scope::Global));
    Let!(
      "\\lx@subfiles@saved@enddocument",
      "\\end{document}",
      Scope::Global
    );
    Let!(
      "\\begin{document}",
      "\\lx@subfiles@fake@begindocument",
      Scope::Global
    );
  });

  // Define \subfile to be \input
  Let!("\\subfile", "\\input");
});
