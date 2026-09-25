//! Red/green guards for perfect-kernel batch 47 (PLANS P40/P41/P32/P33/P45).
use super::perfect_kernel_batch46::{convert, error_count};

/// P40: an unknown column type followed by `{arg}` makes the template
/// reader's "safety valve" (Perl Alignment.pm:906, shared) re-read the arg
/// as column letters; `m` then swallows the template's closing brace and
/// the reader runs into the table body. Batch 33's macro-valued-colspec
/// expansion made that worse by invoking primitive expandables met on the
/// way (`\csname` → scanned to EOF: nicematrix/nicematrix 109→1002+Fatal).
/// Two cuts: nicematrix registers its `V{width}` (nicematrix.sty:2541),
/// and the reader only expands token-bodied macros.
#[test]
fn nicematrix_v_column_and_macro_colspec() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{varwidth}
\usepackage{nicematrix}
\begin{document}
\begin{NiceTabular}{V{3cm}V{3cm}}
a & b \\
\end{NiceTabular}
\def\mycol{cc}
\begin{tabular}{\mycol}c&d\\\end{tabular}
\begin{tabular}{\csname mycol\endcsname}e&f\\\end{tabular}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("Unrecognized tabular template \"V\""),
    "{stderr}"
  );
  assert!(
    !stderr.contains("should not appear between \\csname"),
    "{stderr}"
  );
  assert_eq!(xml.matches("<td").count(), 6, "{xml}");
}

/// P41: `\\` outside any alignment hit a Rust-only `Err` in
/// `read_newline_args` → Fatal. Perl pool:557-571 never guards. Shape:
/// aguplus.cls:305-307 `\pt@tabular` does `\let\\\@tabularcr\@tabarray`
/// — `\@tabarray` is the bare `\@@array[c]` constructor without
/// `\@array@bindings`/`\lx@begin@alignment` (only `\array`/`tabular` add
/// them), so `\@tabularcr` (= `\lx@alignment@newline`) fires with no
/// Alignment in State. Same-host Perl: 2 `Stray alignment "&"` errors,
/// no Fatal. Witness aguplus/aguplus `planotable`: Fatal → full doc.
/// Runs non-raw too (the shape is kernel-only).
#[test]
fn newline_outside_alignment_is_not_fatal() {
  let (stderr, _xml) = convert(
    r"\documentclass{article}
\makeatletter
\begin{document}
\let\\\@tabularcr\@tabarray{lcc}\\
Brix & 45 & 90
\endtabular
\end{document}
",
    false,
  );
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert!(!stderr.contains("read_newline_args"), "{stderr}");
  // `\@tabarray` is now the full array setup (batch 54x): in text mode
  // real TeX errors too ("Missing $ inserted" for the `\vcenter`), so the
  // guard only requires the non-fatal recovery.
  assert!(stderr.contains("Error:"), "{stderr}");
}

/// P32: `[first-col]`/`[last-col]` add a label cell to every source row of
/// the NiceTabular family (nicematrix.tex:2569/2617), so the preamble must
/// grow a column — the array family already did. The trailing `[opts]`
/// counts too (nicematrix.sty:2007 merges both). RED: 2 `Extra alignment
/// tab` per row. P33: `\EmptyColumn{j}` / `\EmptyRow{i}` are
/// `\CodeBefore`-scoped (nicematrix.sty:1808) — were `undefined`
/// (nicematrix.tex:2716).
#[test]
fn nicetabular_first_col_growth_and_codebefore_surface() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{nicematrix}
\begin{document}
\begin{NiceTabular}{ccc}[hvlines,first-row,first-col]
  & 0 & 1 & 2 \\
0 & 1 & 2 & 3 \\
\end{NiceTabular}
\begin{NiceTabular}{ccc}[no-cell-nodes]
\CodeBefore
  \EmptyColumn{3}
  \EmptyRow{1}
\Body
   one & two & three \\
\end{NiceTabular}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Extra alignment tab"), "{stderr}");
  assert_eq!(xml.matches("<td").count(), 11, "{xml}");
}

/// P45: an EMPTY virtual file still exists. tcolorbox's `\tcbwritetemp`
/// over an empty `posterboxenv` body (tcbposter.code.tex:168-171) stores
/// "" and `\tcbusetemp` `\input`s it back; `find_file_aux` read the empty
/// entry as absent and fell through to disk. Witness
/// tcolorbox/tcolorbox-tutorial-poster (`missing_file:<job>.tcbtemp` ×7).
#[test]
fn empty_virtual_file_exists() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage[poster]{tcolorbox}
\begin{document}
\begin{tcbposter}[poster={columns=2,rows=2}]
\begin{posterboxenv}[adjusted title=Core]{name=algo,column=1,row=1}
\end{posterboxenv}
\posterbox[adjusted title=Contact]{name=contact,column=2,row=1}{Contact body.}
\end{tcbposter}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("missing_file"), "{stderr}");
  assert!(xml.contains("Contact body"), "{xml}");
}
