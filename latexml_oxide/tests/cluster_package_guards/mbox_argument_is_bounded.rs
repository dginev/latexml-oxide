//! Batch 56ck: `\mbox`, `\@makebox`, `\@framebox` and `\raisebox` read ONE
//! macro argument (Perl `{}`; latex.ltx:16082 `\mbox[1]` =
//! `\leavevmode\hbox{#1}`), so an unbraced `\mbox\qquad` boxes exactly
//! `\qquad`. The `\hbox`-style `HBoxContents` reader scanned forward to the
//! next `{`, swallowing the `}` that closed the sectioning machinery's group
//! around the title: "\end occurred inside a group at level 4" and a title
//! truncated after the `\\` (ltnews issue 40; an `\endgroup` "non-boxing
//! group" error under a renewed `document`). Perl and pdflatex are clean.

/// The whole `<title>` survives the unbraced `\mbox\qquad` and no group
/// leaks to `\end{document}`.
#[test]
fn unbraced_box_argument_is_one_token() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\subsection{aaa bbb ccc ddd eee fff ggg\\\\\\mbox\\qquad and packages}\nBody \\fbox\\ldots \\raisebox{1pt}\\ldots \\makebox[1cm]\\ldots \\mbox\\ldots end.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("open groups"),
    "an unbraced box argument leaked a group:\n{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r##"<title><tag close=" ">0.1</tag>aaa bbb ccc ddd eee fff ggg<break/>  and packages</title>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="S0.SS1.p1"><p>Body <text cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle">…</text><text yoffset="1.0pt">…</text><text align="center" width="28.5pt">…</text>…end.</p></para>"##,
  );
}

/// An unbraced argument-TAKING token (`\raisebox{1pt}\emph{x}`; pdflatex
/// rejects it — `\hbox{\emph}` leaves `\emph` reading the `}`) is digested
/// as Perl's isolated `Tokens($token)`: `\emph` finds no argument (an
/// empty `<emph/>`), the box closes, and `{x}` stays outside it — Perl's
/// structure at 0 errors, rather than `\emph` swallowing the box's `}`.
#[test]
fn argument_taking_token_digests_in_isolation() {
  let tex =
    "\\documentclass{article}\n\\begin{document}\nA \\raisebox{1pt}\\emph{x} B.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1"><p>A <text yoffset="1.0pt"><emph font="italic"/></text>x B.</p></para>"##,
  );
}
