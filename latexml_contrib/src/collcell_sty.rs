use latexml_package::prelude::*;

LoadDefinitions!({
  // collcell.sty — hand a tabular cell's content to a macro as ONE argument:
  //   \newcolumntype{B}{>{\collectcell\Xfer}c<{\endcollectcell}}
  // means every B cell is typeset as `\Xfer{<cell>}` (collcell.sty:89
  // `\ccell@swap`).
  //
  // Raw-impossibility justification (perfect-kernel README protocol): the
  // real `\collectcell#1#2\ignorespaces` (collcell.sty:76) is DELIMITED by
  // the `\ignorespaces` that the LaTeX kernel inserts into every l/c/r cell
  // template (latex.ltx:16671-16675 `\ignorespaces\@sharp\unskip`,
  // array.sty:97-112 `\insert@column`). LaTeXML's alignment (Perl
  // Alignment.pm:203-217, alignment.rs `get_column_before/after`) assembles
  // cells WITHOUT that wrapping, so the raw macro scans for a delimiter that
  // never comes: Perl reports `Missing argument Until:\ignorespaces` and a
  // boxing-group cascade; Rust's Until-at-EOF abort (base_parameter_types.rs)
  // made it a `Fatal:Mouth:EoF` that lost the whole document (witness
  // onedown-ref.tex:469 `\begin{bidding}`, onedown.sty:1326 column `B`).
  // Guard: `perfect_kernel_batch53::collcell_hands_cell_to_macro`.
  RequirePackage!("array");
  // `verb`/`robustcr` (collcell.sty:12-32) only tune the raw scanner.
  DeclareOption!("verb", "");
  DeclareOption!("noverb", "");
  DeclareOption!("robustcr", "");
  DeclareOption!("norobustcr", "");
  ProcessOptions!();
  // The scanner mirrors `\collect@cell@look` (collcell.sty:91-146): the cell
  // is collected token by token UNEXPANDED (`\futurelet` + `#1` grabs, groups
  // kept as `{…}`), until the `<{\endcollectcell}` column-after template
  // arrives. TeX inserts that v-part when `&`/`\cr` is scanned at
  // `align_state`=0 (tex.web §342); our `read_token` does the same for
  // `&`/`\cr`/`\lx@hidden@cr` (gullet.rs `is_column_end`+`handle_template`).
  // A row end spelled `\\`/`\tabularnewline` is a MACRO, though, invisible to
  // an unexpanding scan (a plain `Until:\endcollectcell` swallowed the rest
  // of the document); collcell handles it by `\ifx`-matching `\tabularnewline`
  // and letting it expand down to `\cr` (L184-192 `\collect@cell@cr`) — we
  // expand it once so `\lx@hidden@cr` reaches the column-end check.
  DefMacro!("\\collectcell Token", sub[(xfer)] {
    let end = T_CS!("\\endcollectcell");
    let newline = T_CS!("\\lx@alignment@newline");
    let cci = T_CS!("\\cci");
    let mut cell: Vec<Token> = Vec::new();
    while let Some(t) = read_token()? {
      if t == end || t.defined_as(&end) {
        break;
      }
      match t.get_catcode() {
        // collcell.sty:231-233: `\cci` is skipped by the scanner, so
        // `\cci{ ...}` hands the macro a braced group it will treat as
        // literal text (onedown.sty:1338 bidding headers).
        Catcode::CS if t == cci => {},
        Catcode::BEGIN => {
          cell.push(t);
          cell.extend(read_balanced(ExpansionLevel::Off, false, false)?.unlist());
          cell.push(T_END!());
        },
        Catcode::CS | Catcode::ACTIVE if t.defined_as(&newline) => {
          if let Some(defn) = lookup_expandable(&t, None)? {
            unread(defn.invoke(true)?);
          } else {
            cell.push(t);
          }
        },
        _ => cell.push(t),
      }
    }
    // `\ignorespaces` before and `\unskip` after the cell (the kernel template,
    // collcell.sty:62/64 `\collcell@beforeuser`/`@afteruser`).
    while cell.last().map(|t| t.get_catcode() == Catcode::SPACE).unwrap_or(false) {
      cell.pop();
    }
    let mut out = vec![xfer, T_BEGIN!()];
    out.extend(cell.into_iter().skip_while(|t| t.get_catcode() == Catcode::SPACE));
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  // Reached only outside a scan (collcell.sty:90 gobbles its own name).
  DefMacro!("\\endcollectcell", "");
  // collcell.sty:178-182: user-level `\protected` no-op marker and `\unskip`.
  DefMacro!("\\cci", "");
  DefMacro!("\\ccunskip", r"\unskip");
});
