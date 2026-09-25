//! Red/green guards for perfect-kernel batch 48 (PLANS P43/P44 + the
//! `{VerbatimOut}` zero-diffs trap + the virtual-file-store abort).
//! RED on the batch-47 binary, green now; each doc-comment names the
//! corpus witness whose full conversion was vetted separately.
use super::perfect_kernel_batch46::{convert, error_count};

/// P43: currfile.sty (+ filehook) raw-loads and its `\ifcurrfilename`
/// family compares against the sanitized `\filename@parse` pieces. RED:
/// the former binding left `\ifcurrfile*` undefined (pythontex/pythontex
/// ×2 → 1/0/0) and the comparison always said "no".
#[test]
fn currfile_ifcurrfilename_compares() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{currfile}
\begin{document}
[\currfilename][\currfilebase][\currfileext]
\ifcurrfilename{x.tex}{yes}{no} done
\ifcurrfilename{t.tex}{yes}{no} done
\ifcurrfilebase{t}{yes}{no} done
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[t.tex][t][tex]"), "{xml}");
  assert!(xml.contains("no done\nyes done\nyes done"), "{xml}");
}

/// `{VerbatimOut}` runs RAW (fancyvrb.sty:1053-1066 `\FVB@VerbatimOut`
/// writes each `\FV@ProcessLine` to `\FV@OutFile`), so a wrapper
/// environment opened with `\VerbatimEnvironment` (the pythontex/minted
/// idiom, fancyvrb.sty:1034) captures its body and `\VerbatimInput`s it
/// back. RED: the Rust override read the body itself and lost the wrapper.
/// Witnesses: pythontex/pythontex_gallery, fancybox/fancybox-doc (fancybox
/// re-implements the same layer, fancybox.sty:1000-1020).
#[test]
fn verbatimout_wrapper_environment_round_trips() {
  for pkg in ["fancyvrb", "fancybox"] {
    let (stderr, xml) = convert(
      &format!(
        r"\documentclass{{article}}
\usepackage{{{pkg}}}
\newenvironment{{pyg}}{{\VerbatimEnvironment\begin{{VerbatimOut}}{{\jobname.pyg}}}}{{\end{{VerbatimOut}}\VerbatimInput{{\jobname.pyg}}}}
\begin{{document}}
before
\begin{{pyg}}
x = 1
\end{{pyg}}
after
\end{{document}}
"
      ),
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{pkg}:\n{stderr}");
    assert!(xml.contains("x = 1"), "{pkg}:\n{xml}");
    assert!(xml.contains("after"), "{pkg}:\n{xml}");
  }
}

/// A package may `\let\FVB@VerbatimOut` to its own line processor
/// (pythontex.sty's `\pytx@FVB@…` counts and stores lines); the raw
/// `\FV@Scan` loop must call THAT, and a later plain `{VerbatimOut}` still
/// writes its file. RED: the override bypassed `\FVB@VerbatimOut` entirely.
#[test]
fn verbatimout_custom_fvb_hook_is_honoured() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{fancyvrb}
\makeatletter
\newcounter{mylines}
\def\my@FVB@VerbatimOut{\begingroup\let\FV@ProcessLine\my@line\let\FV@FontScanPrep\relax\let\@noligs\relax\FV@Scan}
\def\my@FVE@VerbatimOut{\endgroup}
\def\my@line#1{\stepcounter{mylines}}
\newenvironment{mycode}{\VerbatimEnvironment\let\FVB@VerbatimOut\my@FVB@VerbatimOut\let\FVE@VerbatimOut\my@FVE@VerbatimOut\begin{VerbatimOut}}{\end{VerbatimOut}[\themylines\ lines]}
\makeatother
\begin{document}
before
\begin{mycode}
x = {1
y = 2}
\end{mycode}
after
\begin{VerbatimOut}{\jobname.vo}
z = 3
\end{VerbatimOut}
\VerbatimInput{\jobname.vo}
end
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[2 lines]"), "{xml}");
  assert!(xml.contains("z = 3"), "{xml}");
}

/// The zero-diffs trap: `\begin{VerbatimOut}[keys]{file}` (fancyvrb.sty:1053
/// takes the optional key list first). RED: the override read `[` as the
/// file name — "Cached VerbatimOut for [" — and swallowed the REST OF THE
/// DOCUMENT into that file, reporting a clean run with nothing rendered.
/// 18 corpus docs were masked this way (spath3/spath3, mhchem/mhchem,
/// kblocks/kblocks-doc, xcolor/xcolor2, yquant/yquant-doc, …).
#[test]
fn verbatimout_optional_keys_do_not_swallow_document() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{fancyvrb}
\begin{document}
before
\begin{VerbatimOut}[gobble=0]{\jobname.vo}
k = 4
\end{VerbatimOut}
\VerbatimInput{\jobname.vo}
after
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Cached VerbatimOut for ["), "{stderr}");
  assert!(xml.contains("k = 4"), "{xml}");
  assert!(xml.contains("after"), "{xml}");
}

/// fancybox's own verbatim layer (`{SaveVerbatim}`/`\UseVerbatim`/`\Verb`/
/// `{Verbatim}`, fancybox.sty:680-1000) runs raw; back-quotes stay verbatim
/// characters. RED: `undefined` errors for the whole family once the
/// fancybox-doc demos actually executed. Witness: fancybox/fancybox-doc.
#[test]
fn fancybox_verbatim_layer_raw() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{fancybox}
\begin{document}
\begin{SaveVerbatim}{\sv}
x `y' z
\end{SaveVerbatim}
A\UseVerbatim{\sv}B

\Verb|`q'| and \begin{Verbatim}
v `w'
\end{Verbatim}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("x ‘y’ z"), "{xml}");
  assert!(xml.contains("v ‘w’"), "{xml}");
}

/// A `{VerbatimOut}` left open at end of input. `\FV@Scan` re-reads one
/// `Until:^^M` line per iteration (fancyvrb.sty:1090); at true end of all
/// input the `Until` reader must report the runaway (tex.web §338 "File
/// ended while scanning"; Perl Parameter.pm:93-97 → `Missing argument
/// Until:` ×100 → `Fatal:TooManyErrors`; since batch 52 one report then
/// the tex.web §360 job-abort Fatal, see `until_miss_at_eof_is_fatal_once`).
/// RED: the reader mapped the miss
/// to an empty line quietly, so the loop spun forever while each `\write`
/// re-pinned the whole growing file in the never-freed interner — the
/// buffer offset overran `u32` and the binary ABORTED (`string-interner
/// get_unchecked` precondition). Witness: fancyvrb/fancyvrb-doc cut at
/// its open `{SideBySideExample}`.
#[test]
fn until_miss_at_true_eof_errors_like_perl() {
  let (stderr, _xml) = convert(
    r"\documentclass{article}
\usepackage{fancyvrb}
\begin{document}
\begin{VerbatimOut}{\jobname.tmp}
never closed
",
    true,
  );
  assert!(
    stderr.contains("Error:expected:Until: Missing argument Until:"),
    "{stderr}"
  );
  assert!(stderr.contains("Fatal:Mouth:EoF"), "{stderr}");
  assert!(
    !stderr.contains("panicked") && !stderr.contains("precondition"),
    "{stderr}"
  );
}

/// The `\write`-per-line append shape over a long stream stays cheap and
/// reads back whole (the owned-map virtual file store; the LSP overlay and
/// `{filecontents}` share it).
#[test]
fn virtual_file_many_line_append_reads_back() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\begin{document}
\newwrite\w\immediate\openout\w=\jobname.big
\newcount\n
\loop\immediate\write\w{L\the\n}\advance\n1 \ifnum\n<2000 \repeat
\immediate\closeout\w
\input{\jobname.big}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("L0\n") && xml.contains("L1999"), "{xml}");
}
