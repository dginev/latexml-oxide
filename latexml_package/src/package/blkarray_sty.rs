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
//! content, and the label row/column are preserved), but the block's delimiter
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
  // delimiters live on the block specs, which we gobble — so the spec passes to
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

  // `block` / `block*` are transparent: gobble the column spec (which may carry
  // `(`, `[`, `|` delimiters we cannot render sub-region-wise) and contribute
  // their rows directly to the enclosing blockarray alignment. Magic CSes → no
  // `\begingroup`, so the block boundary does not disturb the alignment grouping.
  DefMacro!(T_CS!("\\begin{block}"), "{}", "");
  DefMacro!(T_CS!("\\end{block}"), None, "");
  DefMacro!(T_CS!("\\begin{block*}"), "{}", "");
  DefMacro!(T_CS!("\\end{block*}"), None, "");
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
