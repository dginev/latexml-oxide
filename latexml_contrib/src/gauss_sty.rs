use latexml_package::prelude::*;

// gauss.sty (gmatrix with row/column operations). Raw, `\end{gmatrix}` →
// `\g@endmatrix` (gauss.sty:1733) measures the collected matrix with two
// recursive `\lastbox` passes whose termination is a physical width
// (`\g@measureCols`, :1011: `\ifdim\wd\g@trash=100cm` on a sentinel `\hbox to
// 100cm`) — box geometry no LaTeXML has, so it recurses without end
// (gauss/gauss-ex, `PushbackLimit`; Perl loops too; pdflatex clean). The
// measurement only positions arrows LaTeXML does not draw, so the binding
// renders `gmatrix[d]` as the amsmath matrix its delimiter names (`[p]` →
// `pmatrix` … `[X]` → the user's `\newmatrix` env) and the `\rowops`/`\colops`
// operations as a trailing math annotation in the manual's own semantics
// (`\mult{i}{f}` = R_i ← f R_i, `\add[a]{i}{j}` = R_j ← R_j + a R_i, `\swap{i}
// {j}` = R_i ↔ R_j; C for column operations; indices 0-based as in the
// package). Arrow geometry and label fontifiers are accepted and discarded.
// Guard: `perfect_kernel_batch54::gauss_gmatrix_renders_with_operation_lines`.
#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("amsmath");
  // Environment `gmatrix`: renders `gmatrix[d]` natively through the
  // corresponding amsmath matrix environment (`pmatrix`, `bmatrix`, etc.),
  // avoiding gullet delimited-scan failures when nested inside outer alignments
  // (such as `alignat*`). `\rowops` and `\colops` close the inner matrix alignment
  // if still open, and switch the operation prefix (\lx@gauss@RC) to R or C.
  DefMacro!("\\gmatrix []", sub[args] {
    let delim = args.first().filter(|a| !a.is_none()).map(|d| d.to_string()).unwrap_or_default();
    let env = if delim.is_empty() { "matrix".to_string() } else { format!("{delim}matrix") };
    let mut toks = Vec::new();
    toks.push(T_CS!("\\let"));
    toks.push(T_CS!("\\lx@gauss@matrix@open"));
    toks.push(T_CS!("\\relax"));
    toks.push(T_CS!(&format!("\\{env}")));
    Ok(Tokens::new(toks))
  });
  DefMacro!("\\rowops",
    "\\ifx\\lx@gauss@matrix@open\\relax\
       \\lx@end@ams@matrix\
       \\let\\lx@gauss@matrix@open\\undefined\
     \\fi\
     \\def\\lx@gauss@RC{R}"
  );
  DefMacro!("\\colops",
    "\\ifx\\lx@gauss@matrix@open\\relax\
       \\lx@end@ams@matrix\
       \\let\\lx@gauss@matrix@open\\undefined\
     \\fi\
     \\def\\lx@gauss@RC{C}"
  );
  DefMacro!("\\endgmatrix",
    "\\ifx\\lx@gauss@matrix@open\\relax\
       \\lx@end@ams@matrix\
       \\let\\lx@gauss@matrix@open\\undefined\
     \\fi"
  );
  DefMacro!("\\g@matrix", "\\gmatrix[]");
  DefMacro!("\\endg@matrix", "\\endgmatrix");
  // The operations, as math annotation fragments (each preceded by `\;`).
  DefMacro!("\\mult{}{}", "\\;\\lx@gauss@RC_{#1}\\leftarrow #2\\,\\lx@gauss@RC_{#1}");
  DefMacro!("\\add[][]{}{}", "\\;\\lx@gauss@RC_{#4}\\leftarrow\\lx@gauss@RC_{#4}+#1\\,\\lx@gauss@RC_{#3}");
  DefMacro!("\\swap[][]{}{}", "\\;\\lx@gauss@RC_{#3}\\leftrightarrow\\lx@gauss@RC_{#4}");
  DefMacro!("\\lx@gauss@RC", "R");
  // gauss.sty `\newmatrix{l}{r}{X}`: defines the `Xmatrix` amsmath-style environment
  // and enables `\begin{gmatrix}[X]`.
  DefMacro!("\\newmatrix{}{}{}", sub[(l, r, x)] {
    let (l, r, x) = (l.to_string(), r.to_string(), x.to_string());
    if x.is_empty() || x == "g" { return Ok(Tokens!()); }
    Ok(TokenizeInternal!(TeXString::assembled(format!(
      "\\def\\{x}matrix{{\\lx@ams@matrix{{name={x}matrix,datameaning=matrix,left=\\lx@left{l},right=\\lx@right{r}}}}}\
       \\def\\end{x}matrix{{\\lx@end@ams@matrix}}\
       \\newenvironment{{{x}matrix}}{{\\{x}matrix}}{{\\end{x}matrix}}"
    ))))
  });
  // Arrow geometry and label fontifiers: presentational.
  DefRegister!("\\rowarrowsep" => Dimension::new(0));
  DefRegister!("\\colarrowsep" => Dimension::new(0));
  DefRegister!("\\opskip" => Dimension::new(0));
  DefRegister!("\\labelskip" => Dimension::new(0));
  DefRegister!("\\rowopminsize" => Dimension::new(0));
  DefRegister!("\\colopminsize" => Dimension::new(0));
  DefMacro!("\\rowmultlabel{}", "#1");
  DefMacro!("\\colmultlabel{}", "#1");
  DefMacro!("\\rowswapfromlabel{}", "#1");
  DefMacro!("\\rowswaptolabel{}", "#1");
  DefMacro!("\\colswapfromlabel{}", "#1");
  DefMacro!("\\colswaptolabel{}", "#1");
  DefMacro!("\\rowaddfromlabel{}", "#1");
  DefMacro!("\\rowaddtolabel{}", "#1");
  DefMacro!("\\coladdfromlabel{}", "#1");
  DefMacro!("\\coladdtolabel{}", "#1");
});
