//! A file that ends inside a definition (batch 56ja). TeX closes the file,
//! reports "File ended while scanning definition", inserts a `}` and carries
//! on in the enclosing input (tex.web §362, §338-339), so the rest of the
//! document is kept. The input level is a file whether the file is on disk or
//! held in memory (filecontents, an `\openout` file); the in-memory one was
//! read as a string, crossed, and the `\edef` swallowed the document. Repro
//! `tools/perfect_kernel/repros/expansion-primitives/vfs_file_end_runaway.tex`.
//! Error counts are pdflatex's.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, convert_files, error_count, warning_count};

fn assert_para(xml: &str, expected_p: &str) {
  assert_element(
    xml,
    "para",
    &[r#"xml:id="p1""#],
    &format!(r#"<para xml:id="p1"><p>{expected_p}</p></para>"#),
  );
}

/// A filecontents file `A{B` ends inside `\edef\y{…}`: one error, at the
/// file's end, and "After." is kept.
#[test]
fn filecontents_end_inside_a_definition() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/vfs_file_end_runaway.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("File ended while scanning definition"),
    "{stderr}"
  );
  assert_para(&xml, "After.");
}

/// The same through `\openout`/`\write`, with `\y` = `A{B }` (the inserted
/// `}` closes the inner group, the document's closes the definition).
#[test]
fn openout_file_end_inside_a_definition() {
  let tex = r"\documentclass{article}
\begin{document}
\makeatletter
\immediate\openout15=w.txt \immediate\write15{A\@charlb B}\immediate\closeout15
\edef\y{\@@input w.txt }After.\meaning\y
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "After.macro:-¿A–B ˝");
}

/// A balanced in-memory file still ends inside the definition: the inserted
/// `}` ends it, so the document's own `}` is one too many — two errors, as
/// in pdflatex, and "After." kept. (`\y` is `A `; the paragraph's end trims
/// the space.)
#[test]
fn balanced_in_memory_file_inside_a_definition() {
  let tex = r"\documentclass{article}
\begin{filecontents*}{v3.txt}
A
\end{filecontents*}
\begin{document}
\makeatletter
\edef\y{\@@input v3.txt }After.\meaning\y
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "After.macro:-¿A");
}

/// A disk file behaves the same: one error (not two), `\y` = `A{B }`.
#[test]
fn disk_file_end_inside_a_definition() {
  let tex = r"\documentclass{article}
\begin{document}
\makeatletter
\edef\y{\@@input d6.txt }After.\meaning\y
\end{document}
";
  let (stderr, xml) = convert_files(tex, &[("d6.txt", "A{B\n")]);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "After.macro:-¿A–B ˝");
}

/// Controls: `\everyeof{\noexpand}` lets a balanced read cross the end of an
/// in-memory file and of `\scantokens` with no error (xint's idiom). The body
/// is `A{B` + the crossed `}`, so the document's next `}` ends the `\xdef` and
/// the outer `{` stays open: one warning at `\end{document}`, as pdflatex's
/// "(\end occurred inside a group at level 1)".
#[test]
fn everyeof_noexpand_still_crosses() {
  let tex = r"\documentclass{article}
\begin{filecontents*}{v4.txt}
A\iftrue{\else}\fi B
\end{filecontents*}
\begin{document}
\makeatletter
{\everyeof{\noexpand}\endlinechar=-1 \xdef\y{\@@input v4.txt }}After.\meaning\y
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert_para(&xml, "After.macro:-¿A–B˝");
  let tex = r"\documentclass{article}
\begin{document}
{\everyeof{\noexpand}\endlinechar=-1 \xdef\y{\scantokens{A\iftrue{\else}\fi B}}}After.\meaning\y
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert_para(&xml, "After.macro:-¿A–B˝");
}
