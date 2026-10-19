//! blkarray — block-array matrices.
//!
//! The raw `blkarray.sty` builds each matrix cell as inline math inside a raw
//! `\halign`/`\ialign`, and its `block` delimiter machinery, digested inside
//! display math, drives BOTH LaTeXML engines into the `\halign`-in-math runaway
//! (Rust OOMs at the 4.5 GB cap in ~12 s; same-host Perl hangs ~90 s → rc=124;
//! `pdflatex` renders fine). Full analysis + 4-line reproducer:
//! `docs/archive/known_crashes/blkarray_halign_math/`. This binding SHADOWS the raw
//! `.sty` (so it is never raw-loaded, even under `--includestyles`) and routes
//! `blockarray`/`block` through the engine's well-behaved `array` alignment
//! machinery instead. Surpass-Perl: upstream LaTeXML has no `blkarray.sty.ltxml`.
//! Witnesses: arXiv:1811.10792 (ar5iv #594), arXiv:2310.17416 (ar5iv #473).
//!
//! **Faithfulness note (documented simplification).** In `blkarray`, a `block`'s
//! column-spec delimiters (`(`/`[` in e.g. `block{c(cccccc)}`) wrap a SUB-REGION
//! of the shared matrix — a construct LaTeXML's `array` cannot express (its
//! `left=`/`right=` wrap the whole matrix). We render each `block` transparently:
//! its rows flow into the single `blockarray` alignment (correct structure +
//! content, and the label row/column are preserved), formatted by the block's own
//! columns (`>{}`/`<{}`, OXIDIZED_DESIGN_DIVERGENCES #771), but the block's delimiter
//! parentheses are DROPPED. This is chosen deliberately over the raw `.sty`'s OOM
//! — a matrix without its outer parens beats losing the whole document section.
use crate::prelude::*;

LoadDefinitions!({
  // `\begin{blockarray}[pos]{spec}` is defined as a MAGIC control sequence
  // (`\begin{...}` takes the defined-CS fast path in latex_constructs.rs:3064,
  // which does NOT inject a `\begingroup`). That is essential: a transparent
  // `block` nested inside must not open a group that crosses the alignment's
  // `\\` row boundaries. Route to the same machinery as `\array`
  // (latex_constructs.rs `\@array@bindings`/`\@@array`/`\lx@begin@alignment`).
  // blkarray's own blockarray column specs are plain (`c cccccc`, `cccc`) — the
  // delimiters live on the block specs, which drop them (below) — so the spec passes to
  // the AlignmentTemplate parser unchanged.
  DefMacro!(
    T_CS!("\\begin{blockarray}"),
    "[]{}",
    "\\@array@bindings[#1]{#2}\\lx@blkarray@repeating\\@@array[#1]{#2}\\lx@begin@alignment"
  );
  // blkarray.sty:1206-1208 aligns with a repeating preamble, `\BA@upart##\BA@vpart&&\BA@upart##\BA@vpart\cr`, so
  // a row may run past its spec's columns; such a cell gets no format (`\BA@col@use`'s `\csname` of a column the spec
  // never defined is `\relax`, :838-839). Witness 2301.06399 (a 16-cell row under a 15-column spec).
  DefPrimitive!("\\lx@blkarray@repeating", {
    if let Some(alignment) = lookup_alignment()
      && let Some(alignment) = alignment.alignment_cell()
    {
      alignment
        .borrow_mut()
        .get_template_mut()
        .repeat_past_columns(Cell::default());
    }
  });
  // blkarray.sty:749-750; papers set it (`\setlength\BA@colsep{4pt}`, 2301.06399).
  RawTeX!(r"\newdimen\BA@colsep \BA@colsep=\tabcolsep");
  // 68b (binding gaps G3): the public blkarray commands papers use inside a `{blockarray}`.
  // blkarray.sty:934-947 `\BAmulticolumn{n}{spec}{text}`, blkarray's `\multicolumn` (1202.0500, 1204.2176,
  // 1301.3189, 1311.0327, 1401.4921, 1407.4836).
  Let!("\\BAmulticolumn", "\\multicolumn");
  // :1677-1683 `\BAnewcolumntype[n]{c}{spec}`, a column type for blkarray's preambles (1302.2340, 1402.5950,
  // 1408.0807, 1702.06723, 2005.02853: `\BAnewcolumntype{s}{>{$\small$}c}`): the rewrite the alignment template
  // reads a column letter through, as array's `\newcolumntype` makes it (`\NC@rewrite@<c>`, alignment.rs).
  RawTeX!(
    r"\def\BAnewcolumntype{\@ifnextchar[{\lx@BA@nct}{\lx@BA@nct[0]}}
\def\lx@BA@nct[#1]#2#3{\expandafter\@reargdef\csname NC@rewrite@\string#2\endcsname[#1]{#3}}"
  );
  // :1063 `\BAextrarowheight` (1312.6738, 1503.04543, 1503.08375 set it), the height blkarray adds to each row.
  RawTeX!(r"\newdimen\BAextrarowheight");
  // :1005-1012 `\BAnoalign{material}`/`\BAnoalign*{material}`: material between rows, which the star also lets
  // the current block's delimiters span (1912.05110 `\BAnoalign{\vskip -7ex}`).
  RawTeX!(
    r"\def\BAnoalign{\@ifstar\lx@BA@noalign\lx@BA@noalign}\long\def\lx@BA@noalign#1{\noalign{#1}}"
  );
  // blkarray.sty:2057-2068: `\BAhline` rules every column (`\BAhhline{*{\BA@col@max}{-}}`), as `\hline` does.
  Let!("\\BAhline", "\\hline");
  DefMacro!("\\BAhhline Semiverbatim", sub[(spec)] {
    let rules: String = ba_hhline_ruled_columns(&spec.to_string())
      .into_iter()
      .map(|(first, last)| s!("\\cline{{{first}-{last}}}"))
      .collect();
    Ok(Tokenize!(TeXString::assembled(rules)))
  });
  DefMacro!(
    T_CS!("\\end{blockarray}"),
    None,
    "\\lx@end@alignment\\@end@array"
  );

  // A `block`'s own preamble formats its rows (blkarray.sty:868-922 `\BA@beginblock`, `\begin{block}` inside a
  // `{blockarray}` by :1099-1102 `\BA@begin`): a `\noalign` that parses the block's columns, which its rows use
  // (`\BA@col@use` keys each column's `>{}`/`<{}` parts by the block, :838-839) until `\BA@endblock`'s `\noalign`
  // puts the enclosing preamble back (:885-892). The delimiters are not columns: `(`, `)`, `[`, `]`, `\{`, `\}`
  // are blkarray's `\Left`/`\Right` (:1998-2003), which draw a parenthesis beside the block's sub-region, a shape
  // the `array` alignment cannot express, so they are dropped. A block that changed no format (`{c(cc)}`) is the
  // transparent block it was. `\begin{block}{*{2}{>{$\footnotesize}c<{$}}}` keeps its cells' `$`/`$` wrappers
  // (1706.05431, 2207.06508, 2210.16910; OXIDIZED_DESIGN_DIVERGENCES #771). Magic CSes: no `\begingroup` around the
  // block, so its boundary does not cross the alignment's rows.
  // The `\noalign` is the hidden one, which adds no row of its own.
  DefMacro!(
    T_CS!("\\begin{block}"),
    "{}",
    r"\lx@hidden@noalign{\lx@blkarray@columntypes\lx@blkarray@delimiters\lx@blkarray@block{#1}}"
  );
  DefMacro!(
    T_CS!("\\end{block}"),
    None,
    r"\lx@hidden@noalign{\lx@blkarray@endblock}"
  );
  DefMacro!(
    T_CS!("\\begin{block*}"),
    "{}",
    r"\lx@hidden@noalign{\lx@blkarray@columntypes\lx@blkarray@delimiters\lx@blkarray@block{#1}}"
  );
  DefMacro!(
    T_CS!("\\end{block*}"),
    None,
    r"\lx@hidden@noalign{\lx@blkarray@endblock}"
  );
  // blkarray's own column types for a block's preamble, local to the group it is read in: `>{}`/`<{}` (:1714-1725,
  // :1818-1831, array.sty's without it) and the delimiters, which are no columns.
  DefPrimitive!("\\lx@blkarray@columntypes", {
    DefColumnType!(">{}", sub[(before)] {
      let before = VecDeque::from(before.unlist());
      with_building_template(|template| template.add_before_column(before));
    });
    DefColumnType!("<{}", sub[(after)] {
      with_building_template(|template| template.add_after_column(after.unlist()));
    });
  });
  RawTeX!(
    r"\def\lx@blkarray@delimiters{\BAnewcolumntype{(}{}\BAnewcolumntype{)}{}\BAnewcolumntype{[}{}\BAnewcolumntype{]}{}\BAnewcolumntype{\{}{}\BAnewcolumntype{\}}{}\BAnewcolumntype[2]{\Left}{}\BAnewcolumntype[2]{\Right}{}}"
  );
  DefPrimitive!("\\lx@blkarray@block AlignmentTemplate", sub[(block)] {
    let Some(alignment) = lookup_alignment() else { return Ok(Vec::new()) };
    let Some(alignment) = alignment.alignment_cell() else { return Ok(Vec::new()) };
    let mut alignment = alignment.borrow_mut();
    let template = alignment.get_template_mut();
    push_value("lx@blkarray@preambles", Stored::Template(Rc::new(template.clone())))?;
    for (n, column) in block.get_columns().iter().enumerate() {
      let mut column = column.clone();
      column.mark_intercol_after();
      match template.get_columns_mut().get_mut(n) {
        Some(slot) => *slot = column,
        None => template.get_columns_mut().push(column),
      }
    }
  });
  DefPrimitive!("\\lx@blkarray@endblock", {
    if let Some(Stored::Template(outer)) = pop_value("lx@blkarray@preambles")?
      && let Some(alignment) = lookup_alignment()
      && let Some(alignment) = alignment.alignment_cell()
    {
      *alignment.borrow_mut().get_template_mut() = (*outer).clone();
    }
  });
});

/// The column runs `\BAhhline{spec}` rules (blkarray.sty:2074-2210), for one `\cline` each: `-` `=` `.` `"` rule a
/// column and `~` leaves one bare, each moving to the next column unless the current one is still unused; `&` moves to
/// the next; `*{n}{x}` repeats `x`; `|` `:` `#` `t` `b` draw vertical pieces between columns.
fn ba_hhline_ruled_columns(spec: &str) -> Vec<(usize, usize)> {
  let mut ruled: Vec<usize> = Vec::new();
  let (mut column, mut unused) = (1, true);
  for c in ba_hhline_repeats(spec).chars() {
    match c {
      '-' | '=' | '.' | '"' | '~' => {
        if !unused {
          column += 1;
        }
        unused = false;
        if c != '~' {
          ruled.push(column);
        }
      },
      '&' => {
        column += 1;
        unused = true;
      },
      _ => {},
    }
  }
  let mut runs: Vec<(usize, usize)> = Vec::new();
  for column in ruled {
    match runs.last_mut() {
      Some((_, last)) if *last + 1 == column => *last = column,
      _ => runs.push((column, column)),
    }
  }
  runs
}

/// `spec` with each `*{n}{x}` written out `n` times (blkarray.sty:2198-2207; either argument may be one character).
fn ba_hhline_repeats(spec: &str) -> String {
  fn argument(chars: &[char], mut i: usize) -> Option<(String, usize)> {
    while chars.get(i) == Some(&' ') {
      i += 1;
    }
    match chars.get(i)? {
      '{' => {
        let mut depth = 0;
        for (j, &c) in chars.iter().enumerate().skip(i) {
          match c {
            '{' => depth += 1,
            '}' => {
              depth -= 1;
              if depth == 0 {
                return Some((chars[i + 1..j].iter().collect(), j + 1));
              }
            },
            _ => {},
          }
        }
        None
      },
      &c => Some((c.to_string(), i + 1)),
    }
  }
  let chars: Vec<char> = spec.chars().collect();
  let mut out = String::new();
  let mut i = 0;
  while i < chars.len() {
    if chars[i] == '*'
      && let Some((count, next)) = argument(&chars, i + 1)
      && let Some((body, next)) = argument(&chars, next)
    {
      out.push_str(&ba_hhline_repeats(&body).repeat(count.trim().parse().unwrap_or(0)));
      i = next;
    } else {
      out.push(chars[i]);
      i += 1;
    }
  }
  out
}
