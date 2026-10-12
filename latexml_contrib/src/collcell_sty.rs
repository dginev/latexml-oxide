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
  // array.sty:97-112 `\insert@column`). Perl's alignment (Alignment.pm:203-217)
  // assembles cells WITHOUT that wrapping, so the raw macro scans for a
  // delimiter that never comes: Perl reports `Missing argument
  // Until:\ignorespaces` and a boxing-group cascade. Ours gives the `>{…}`
  // tokens their `\ignorespaces` since 63v (template.rs, OXIDIZED_DESIGN #473),
  // but its trim (`\lx@column@trimright`) and row end (`\lx@hidden@cr`) are not
  // the `\unskip` and `\tabularnewline`/`\cr` the raw scanner tests for
  // (collcell.sty:184-212) (witness onedown-ref.tex:469 `\begin{bidding}`,
  // onedown.sty:1326 column `B`, once a `Fatal:Mouth:EoF`).
  // Guards: `perfect_kernel_batch53::collcell_hands_cell_to_macro`,
  // `perfect_kernel_batch65e::collcell_cell_starts_after_the_template_ignorespaces`.
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
  // `\collectcell#1#2\ignorespaces` (collcell.sty:76): its delimiter is the `\ignorespaces` the cell template puts
  // after the `>{…}` tokens (array.sty:97-101 `\insert@column`, template.rs `add_before_column`), so the cell begins
  // after it (what precedes it, `#2`, is scanned as the cell's start: `\collect@cell@look#2`). The scan skips the
  // template's `\unskip` after the cell (collcell.sty:207-212, with the spaces before it), whose role our template's
  // `\lx@column@trimright` plays (Perl Template.pm:60/97), and the macro gets the cell between `\ignorespaces` and
  // `\unskip` (collcell.sty:62-64 `\collcell@beforeuser`/`@afteruser`). Since the template's `\ignorespaces`
  // (63v, OXIDIZED_DESIGN #473) the macro was handed it and the trim marks, `\ifdim\ignorespaces 0.2pt<…`
  // (1912.00582, 2604.02352).
  DefMacro!("\\collectcell Token", sub[(xfer)] {
    let end = T_CS!("\\endcollectcell");
    let newline = T_CS!("\\lx@alignment@newline");
    let cci = T_CS!("\\cci");
    let delimiter = T_CS!("\\ignorespaces");
    let unskips = [T_CS!("\\unskip"), T_CS!("\\lx@column@trimright")];
    let mut delimited = false;
    let mut trimmed = false;
    let mut cell: Vec<Token> = Vec::new();
    while let Some(t) = read_token()? {
      if t == end || t.defined_as(&end) {
        break;
      }
      match t.get_catcode() {
        // the `#2\ignorespaces` delimiter, once
        Catcode::CS if !delimited && t == delimiter => delimited = true,
        // collcell.sty:207-212: an `\unskip` is not collected, nor the spaces before it
        Catcode::CS if unskips.contains(&t) => {
          trimmed |= t == unskips[1];
          while cell.last().is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
            cell.pop();
          }
        },
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
    // `\ignorespaces` before and `\unskip` after the cell (`\collcell@beforeuser`/`@afteruser`), the latter the
    // template's trim it stands for.
    while cell.last().map(|t| t.get_catcode() == Catcode::SPACE).unwrap_or(false) {
      cell.pop();
    }
    let mut out = vec![xfer, T_BEGIN!()];
    out.extend(cell.into_iter().skip_while(|t| t.get_catcode() == Catcode::SPACE));
    out.push(T_END!());
    if trimmed {
      out.push(unskips[1]);
    }
    Ok(Tokens::new(out))
  });
  // Reached only outside a scan (collcell.sty:90 gobbles its own name).
  DefMacro!("\\endcollectcell", "");
  // collcell.sty:178-182: user-level `\protected` no-op marker and `\unskip`.
  DefMacro!("\\cci", "");
  DefMacro!("\\ccunskip", r"\unskip");
});
