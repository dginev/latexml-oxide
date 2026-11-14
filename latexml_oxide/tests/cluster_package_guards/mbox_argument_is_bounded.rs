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

/// The box commands start the paragraph as latex.ltx's `\leavevmode` does
/// (`\mbox` 16082, `\makebox` 16077-16078, `\fbox` 16182-16183, `\raisebox`
/// 16373-16374, color.sty's `\colorbox` 163-164), so the space after a box that
/// opens a paragraph or an item is read in horizontal mode and kept. It was
/// dropped: "FiraSans</text>et" (matapli-doc's `\item \Verb+FiraSans+ et`;
/// fancyvrb's `\Verb` ends in `\mbox`), "Mboxet". Perl drops it for `\mbox`,
/// `\fbox` and `\colorbox`; its `\makebox` and `\raisebox` keep it
/// (enterHorizontal), which batch 54n's mode change had lost here.
#[test]
fn box_commands_start_the_paragraph() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/mbox_leavevmode_space.tex");
  let (stderr, xml) = super::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    super::perfect_kernel_batch46::warning_count(&stderr),
    0,
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r##"<para xml:id="p1">
      <itemize xml:id="S0.I1">
        <item xml:id="S0.I1.i1">
          <tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags>
          <para xml:id="S0.I1.i1.p1">
            <p xml:id="S0.I1.i1.p1.1"><text font="typewriter" xml:id="S0.I1.i1.p1.1.1">FiraSans</text> et B.</p>
          </para>
        </item>
      </itemize>
      <p xml:id="p1.1"><text xml:id="p1.1.1">Mbox</text> et B.</p>
    </para>"##,
  );
  for (id, open) in [
    ("p2", r#"<text xml:id="p2.1.1">Makebox"#),
    ("p3", r#"<text yoffset="1.0pt" xml:id="p3.1.1">Raisebox"#),
    (
      "p4",
      r##"<text cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle" xml:id="p4.1.1">Fbox"##,
    ),
    (
      "p5",
      r##"<text backgroundcolor="#FF0000" xml:id="p5.1.1">Colorbox"##,
    ),
    // `\textcolor{red}{\hbox{Hbox}}`: color.sty:104's `\leavevmode` puts the
    // `\hbox` in horizontal mode.
    ("p6", r##"<text color="#FF0000" xml:id="p6.1.1">Hbox"##),
  ] {
    latexml::util::test::assert_element(
      &xml,
      "para",
      &[&format!("xml:id=\"{id}\"")],
      &format!(r#"<para xml:id="{id}"><p xml:id="{id}.1">{open}</text> et B.</p></para>"#),
    );
  }
}

/// A box that starts the paragraph fires `\everypar` first, outside the box
/// and outside `\textcolor`'s colour group (pdflatex `[EP]Box after.`, `[EP]`
/// black; Perl's order was `Box[EP]after.`), and in math the step is a no-op:
/// the formula and its `tex` are as before.
#[test]
fn boxes_fire_everypar_first_and_stay_out_of_math() {
  let tex = r"\documentclass{article}
\usepackage{color}
\begin{document}
{\everypar{[EP]}\mbox{Box} after.\par}
{\everypar{[EP]}\textcolor{red}{Red} after.\par}
$a\fbox{$x$}\colorbox{red}{$w$}\textcolor{red}{y}$
\end{document}
";
  let (stderr, xml) = super::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">[EP]<text xml:id="p1.1.1">Box</text> after.</p>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "p",
    &["xml:id=\"p2.1\""],
    r##"<p xml:id="p2.1">[EP]<text color="#FF0000" xml:id="p2.1.1">Red</text> after.</p>"##,
  );
  assert!(
    xml.contains(r#"tex="a\framebox{$x$}\hbox{\pagecolor{red}$w$}{\color[rgb]{1,0,0}y}""#),
    "{xml}"
  );
}
