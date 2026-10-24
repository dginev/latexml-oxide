use crate::prelude::*;

LoadDefinitions!({
  // subfiles.sty:171 `\ifSubfilesClassLoaded{yes}{no}`: true when the
  // subfiles CLASS is driving (`\ver@subfiles.cls` defined) — the package
  // form takes the second branch (sshrc-insight). Guard:
  // `perfect_kernel_batch54::subfiles_class_loaded_test_is_defined`.
  DefMacro!(
    "\\ifSubfilesClassLoaded",
    "\\@ifundefined{ver@subfiles.cls}\\@secondoftwo\\@firstoftwo"
  );

  // Redefine \documentclass to do nothing in subfiles
  DefMacro!(
    "\\documentclass PackageOptions SkipSpaces Semiverbatim []",
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
    begin_mode("internal_vertical")?;
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
    end_mode("internal_vertical")?;
  });

  // AtBeginDocument: after the main document starts, redirect nested
  // \begin{document}/\end{document} to the fake versions
  let _ = push_value("@at@begin@document", Tokens!(T_CS!("\\lx@subfiles@setup")));
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

  // subfiles.sty:120-131 — `\subfile{f}` is `\subimport{\filename@area}{\filename@base}` (import.sty, which it
  // requires), the base keeping its extension (`\subfiles@split`'s `\filename@simple`), and `\subfileinclude` the
  // same through `\subincludefrom`: the subfile's own directory is searched first for what it inputs and the figures it
  // names, as pdflatex finds them (2209.05589's `\subfile{./sub/OPA_v2.tex}`, whose `\includegraphics{FigScheme.eps}`
  // live in `sub/`). Perl binds `\subfile` as `\input` (subfiles.sty.ltxml), so those figures were missing. The group
  // `\subfiles@subfile` opens around the load is not taken, as in Perl's binding: a subfile's definitions stay.
  RequirePackage!("import");
  DefMacro!("\\subfile{}", sub[(file)] { Ok(subfile_import("\\subimport", &Expand!(file).to_string())) });
  DefMacro!("\\subfileinclude{}", sub[(file)] {
    Ok(subfile_import("\\subincludefrom", &Expand!(file).to_string()))
  });
});

/// `\subfiles@split` (subfiles.sty:111-116): the area of `file` up to its last `/` and the rest, given to `loader`
/// (`\subimport` or `\subincludefrom`) as its two arguments.
fn subfile_import(loader: &str, file: &str) -> Tokens {
  let file = file.trim();
  let (area, base) = file
    .rfind('/')
    .map_or(("", file), |slash| (&file[..=slash], &file[slash + 1..]));
  let mut out = vec![T_CS!(loader), T_BEGIN!()];
  out.extend(Explode!(area));
  out.extend([T_END!(), T_BEGIN!()]);
  out.extend(Explode!(base));
  out.push(T_END!());
  Tokens::new(out)
}
