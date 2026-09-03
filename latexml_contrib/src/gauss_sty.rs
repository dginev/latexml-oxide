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
  // `\begin{gmatrix}[d] cells [\rowops ops] [\colops ops] \end{gmatrix}`: the
  // body is read unexpanded and split at the section switches.
  DefMacro!("\\begin{gmatrix} [] Until:\\end{gmatrix}", sub[(delim, body)] {
    let delim = delim.map(|d| d.to_string()).unwrap_or_default();
    let env = if delim.is_empty() { "matrix".to_string() } else { format!("{delim}matrix") };
    let mut cells: Vec<Token> = Vec::new();
    let mut rowops: Vec<Token> = Vec::new();
    let mut colops: Vec<Token> = Vec::new();
    let mut section = 0u8; // 0 cells, 1 rowops, 2 colops
    let mut depth = 0i32;
    for t in body.unlist() {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        Catcode::CS if depth == 0 && t == T_CS!("\\rowops") => { section = 1; continue; },
        Catcode::CS if depth == 0 && t == T_CS!("\\colops") => { section = 2; continue; },
        _ => {},
      }
      match section { 0 => cells.push(t), 1 => rowops.push(t), _ => colops.push(t) }
    }
    let mut out: Vec<Token> = Vec::new();
    out.extend(Tokenize!(TeXString::assembled(format!("\\begin{{{env}}}"))).unlist());
    out.extend(cells);
    out.extend(Tokenize!(TeXString::assembled(format!("\\end{{{env}}}"))).unlist());
    for (name, ops) in [("R", rowops), ("C", colops)] {
      if ops.iter().any(|t| t.get_catcode() != Catcode::SPACE) {
        out.push(T_BEGIN!());
        out.extend(Tokenize!(TeXString::assembled(format!("\\def\\lx@gauss@RC{{{name}}}"))).unlist());
        out.extend(ops);
        out.push(T_END!());
      }
    }
    Ok(Tokens::new(out))
  });
  // The operations, as math annotation fragments (each preceded by `\;`).
  DefMacro!("\\mult{}{}", "\;\\lx@gauss@RC_{#1}\\leftarrow #2\\,\\lx@gauss@RC_{#1}");
  DefMacro!("\\add[][]{}{}", "\;\\lx@gauss@RC_{#4}\\leftarrow\\lx@gauss@RC_{#4}+#1\\,\\lx@gauss@RC_{#3}");
  DefMacro!("\\swap[][]{}{}", "\;\\lx@gauss@RC_{#3}\\leftrightarrow\\lx@gauss@RC_{#4}");
  DefMacro!("\\lx@gauss@RC", "R");
  def_macro_noop("\\rowops")?;
  def_macro_noop("\\colops")?;
  // gauss.sty `\newmatrix{l}{r}{X}`: the fenced `Xmatrix` environment and the
  // `gmatrix[X]` delimiter letter.
  DefMacro!("\\newmatrix{}{}{}", sub[(l, r, x)] {
    let (l, r, x) = (l.to_string(), r.to_string(), x.to_string());
    if x.is_empty() || x == "g" { return Ok(Tokens!()); }
    Ok(Tokenize!(TeXString::assembled(format!(
      "\\newenvironment{{{x}matrix}}{{\\left{l}\\begin{{matrix}}}}{{\\end{{matrix}}\\right{r}}}"
    ))))
  });
  // Arrow geometry and label fontifiers: presentational.
  for d in ["\\rowarrowsep", "\\colarrowsep", "\\opskip", "\\labelskip", "\\rowopminsize", "\\colopminsize"] {
    DefRegister!(d => Dimension::new(0));
  }
  for m in ["\\rowmultlabel", "\\colmultlabel", "\\rowswapfromlabel", "\\rowswaptolabel",
            "\\colswapfromlabel", "\\colswaptolabel", "\\rowaddfromlabel", "\\rowaddtolabel",
            "\\coladdfromlabel", "\\coladdtolabel"] {
    DefMacro!(&format!("{m}{{}}"), "#1");
  }
});
