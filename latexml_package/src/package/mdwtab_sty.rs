//! mdwtab.sty — alternative tabular environment by Mark Wooding (1996).
//!
//! mdwtab.sty completely reimplements `\tabular` with its own
//! `\tab@*` preamble-parsing machinery (`\tab@right`,
//! `\tab@restorehlstate`, `\tab@bgroup`, `\tab@multicol`, etc.).
//! Raw-loading it would redefine our locked `\tabular` binding —
//! the redefinition is silently ignored, but then mdwtab's own
//! `\tab@*` helpers (referenced by every column type and `\@arstrut`
//! handler) end up undefined when the user actually enters a
//! tabular, cascading through `\omit`/`\@startsection` mode errors
//! to a TooManyErrors fatal.
//!
//! Perl LaTeXML has no `mdwtab.sty.ltxml`; with its default
//! `INCLUDE_STYLES=false` the raw `mdwtab.sty` is NOT loaded —
//! Perl emits a "missing binding" warning and continues with the
//! binding-aware `\tabular`. The user's tables then render
//! through standard array/tabular machinery (no mdwtab-specific
//! ornaments, but cleanly).
//!
//! In Rust we default to `INCLUDE_STYLES=true` (ar5iv preload sets
//! it). This stub suppresses the raw-load so `\tabular` keeps
//! pointing at our binding-aware constructor. Same pattern as
//! `delarray_sty.rs` / `trace_sty.rs`.
//!
//! Witness: canvas-3 stage-26 0910.3293 (uses `\usepackage{mathenv}`
//! which `\RequirePackage{mdwtab}`).
use crate::prelude::*;

LoadDefinitions!({
  // mdwtab.sty:765-790 `\hlx{letters}`: `v[dim]` = `\vgap`, which ENDS the
  // current row (`\cr`) and adds vertical space; `h` = `\hline` (`hh`
  // double); `b`, `/` are print-only. Parsed here and emitted as bare
  // `\\`/`\hline` tokens — a TeX-level loop left its bookkeeping assignment
  // as the next row's first token, which the alignment then took as cell
  // content and the following `\hline` was "`\noalign` cannot be used here"
  // (talkdoc `\hlx{hhv[1pt]}`). Guard:
  // `perfect_kernel_batch54::mdwtab_hlx_ends_the_row_and_rules`.
  DefMacro!("\\hlx{}", sub[(letters)] {
    let spec = letters.to_string();
    let mut out: Vec<Token> = Vec::new();
    // At a table's very start (talkdoc's leading `\hlx{hhv[1pt]}`) no row is
    // open, so no `\\` precedes the rules.
    let mut row_open = match lookup_alignment() {
      Some(a) => a.alignment_cell().map(|cell| cell.borrow().is_in_row()).unwrap_or(true),
      None => true,
    };
    let mut chars = spec.chars().peekable();
    while let Some(c) = chars.next() {
      match c {
        'v' | 'h' => {
          if row_open {
            out.push(T_CS!("\\\\"));
            row_open = false;
          }
          if c == 'h' {
            out.push(T_CS!("\\hline"));
          }
        },
        '[' => {
          for d in chars.by_ref() {
            if d == ']' { break; }
          }
        },
        _ => {},
      }
    }
    Ok(Tokens::new(out))
  });
});
