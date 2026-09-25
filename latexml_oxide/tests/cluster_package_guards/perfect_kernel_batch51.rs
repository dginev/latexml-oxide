//! Red/green guards for perfect-kernel batch 51 (sweep 28 "Until … at end
//! of input" cluster A and the `\endlx@list` cluster B). Each test is the
//! minimal reproduction distilled during triage; the doc-comment names the
//! ORIGINAL corpus witness (TeX Live doc corpus) whose larger conversion
//! was vetted separately.

use super::perfect_kernel_batch46::{convert, error_count};

/// Like [`convert`] (raw preload), but first drops extra `(name, content)`
/// files into the tempdir so the snippet can `\input` them.
pub(super) fn convert_with_files(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  super::perfect_kernel_batch46::convert_files(tex, files)
}

fn kpsewhich_has(name: &str) -> bool {
  std::process::Command::new("kpsewhich")
    .arg(name)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
}

/// P15 (file side): eTeX §362 begins the `\everyeof` token list at the end
/// of EVERY `\input` file, before the file is closed — so a delimited
/// argument opened across the `\input` PRIMITIVE (`\expandafter\eat
/// \@@input f` — LaTeX's `\input` is a macro, and real TeX runs away on
/// it too) is terminated by the register and never scans past the file.
/// pdflatex oracle: `[alpha beta ]Tail.`, no errors. RED: the register was
/// inserted only for `\scantokens`; `\eat#1\stopper` ran to the end of the
/// document ("Missing argument Until:\stopper at end of input"), captured
/// nothing, and `Tail.` followed an empty `[]`. Witnesses:
/// tikzmarmots-doc.tex:44-105 `\CommentInput` (`\tex_everyeof:D` +
/// `\tex_input:D`; 0 → 501 errors + Fatal on the sweep-28 binary),
/// tikzlings-doc (35 → 536), stex.sty:2633 smsmode
/// `\everyeof{\q__stex_smsmode_break\exp_not:N}`, expl3-code.tex
/// `\__file_get_do:Nw`.
#[test]
fn input_file_end_inserts_everyeof() {
  let (stderr, xml) = convert_with_files(
    r"\documentclass{article}
\begin{document}
\makeatletter
\def\stopper{STOP}
\begingroup
\everyeof{\stopper}%
\def\eat#1\stopper{[\detokenize{#1}]}%
\expandafter\eat\@@input lines.tex
\endgroup
Tail.
\end{document}
",
    &[("lines.tex", "alpha\nbeta\n")],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("[]"), "{xml}");
  assert!(xml.contains("[alpha beta ]"), "{xml}");
  assert!(xml.contains("Tail."), "{xml}");
}

/// amsgen.sty:54-62 `\new@ifnextchar` does NOT skip spaces — Perl
/// (amsgen.sty.ltxml:42) Lets it to the space-skipping `\@ifnextchar`,
/// KNOWN_PERL_ERRORS #113. bibleref.sty:969 `\bibleverse` uses it to look
/// for an immediately-following `(`; with the space skipped,
/// `\bibleverse{Psalms} (Einzahl)` opened `\@bibleverse(#1:` and scanned to
/// the end of the document. RED: `<relationaltoken>` ×2 + `Until::` at end
/// of input, the whole paragraph lost. Witnesses: en-bibleref-german,
/// de-bibleref-german (bibleref-german-preamble.tex:120; 12 `Until::`
/// misses each, sweep 28).
#[test]
fn new_ifnextchar_keeps_space() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("bibleref.sty") {
    return;
  }
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{bibleref}
\begin{document}
Beispiel: \bibleverse{Psalms} (Einzahl) und \bibleverse{Psalms}(23:1) hier.
Ende.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("(Einzahl)"), "{xml}");
  assert!(xml.contains("23:1"), "{xml}");
  assert!(xml.contains("Ende."), "{xml}");
}

/// The contrib `\printbibliography` (mirroring ar5iv-bindings
/// biblatex.sty.ltxml:410) rebinds `\verb` to `\biblatex@verb{} Until:
/// \endverb` for reading the `.bbl` and never restored it, so every
/// `\verb+x+` after the bibliography scanned to the end of the document
/// (KNOWN_PERL_ERRORS #114). RED: two "Missing argument Until:\endverb at
/// end of input", delimiters leaked as text (`foo.dtx+`), no verbatim
/// element. Witnesses: docsurvey.tex:2876-2898 (7 `\verb+.dtx+` after the
/// bibliographies, ~500 lines of body lost), rub-kunstgeschichte-example.
#[test]
fn verb_survives_printbibliography() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{filecontents}
\begin{filecontents}{t.bib}
@book{knuth84, author={Donald Knuth}, title={The TeXbook}, year={1984}, publisher={Addison-Wesley}}
\end{filecontents}
\usepackage[backend=biber]{biblatex}
\addbibresource{t.bib}
\begin{document}
Cite \cite{knuth84}.
\printbibliography
Files: \verb+foo.dtx+ and \verb|bar.ins| here.
Trailing text survives.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("foo.dtx+"), "{xml}");
  assert!(
    xml.contains(">foo.dtx<") && xml.contains(">bar.ins<"),
    "{xml}"
  );
  assert!(xml.contains("Trailing text survives."), "{xml}");
}

/// memoir.cls:4580 defines `\list` raw, ending in `\@trivlist` — no group
/// of its own — while `\endlist` is still our `\endlx@list`, which
/// unconditionally `egroup`ed — popping the ENCLOSING frame whenever it
/// was a plain `{` group, after which every later `\global`/`\let` in the
/// document cascaded. The closer now pops only a frame that `\lx@list`
/// itself opened (groupInitiator) and otherwise reports Perl's
/// `endMode` error without popping (Stomach.pm:524-531). RED here:
/// "Attempt to close boxing group … due to \begingroup". Witnesses: memman
/// (144 → 1001 errors on the sweep-28 binary), biblatex-oxref ×4,
/// verbatimcopy, dlfltxb.
#[test]
fn endlist_without_lx_list_frame() {
  let (stderr, xml) = convert(
    r"\documentclass{memoir}
\begin{document}
\chapter{Test}
\begin{list}{--}{}
\item one
\item two
\end{list}
After.
\end{document}
",
    true,
  );
  // OXIDIZED_DESIGN #180 (P38): `\@trivlist` now opens the list the raw
  // `\list` asked for, so the items are real items and nothing cascades
  // (before P38 this was Perl's one "Attempt to end mode" per list).
  assert!(!stderr.contains("Attempt to close"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<itemize") && xml.matches("<item ").count() == 2,
    "{xml}"
  );
  assert!(xml.contains("one") && xml.contains("two"), "{xml}");
  assert!(xml.contains("After."), "{xml}");
}
