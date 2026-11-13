//! A file that ends in the middle of a scan (batch W9). TeX closes the file
//! and recovers by what the scanner was reading, tex.web's `scanner_status`
//! (§305, §336-339): a definition or a balanced text gets a `}` and ends at the
//! file's end; an alignment preamble gets `\cr}`; a macro argument makes the
//! macro call abandon itself, dropping the macro and its argument (§392). The
//! enclosing input then goes on. Error counts and text are pdflatex's.
//! Repros `tools/perfect_kernel/repros/expansion-primitives/file_end_*.tex`;
//! the definition cases with an `\edef` are `vfs_file_end::*`.
use latexml::util::test::{assert_element, dump_available};

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

fn assert_para(xml: &str, expected_ps: &str) {
  assert_element(
    xml,
    "para",
    &[r#"xml:id="p1""#],
    &format!(r#"<para xml:id="p1">{expected_ps}</para>"#),
  );
}

/// The document's paragraphs, whole and in order: one `<para><p>…</p></para>` each (a plain TeX document's carry
/// no `xml:id`).
fn assert_paras(xml: &str, texts: &[&str]) {
  // Indentation between elements dropped; a line break inside a text is the space it stands for.
  let compact = xml
    .lines()
    .map(str::trim)
    .collect::<Vec<_>>()
    .join(" ")
    .replace("> <", "><");
  let found: Vec<&str> = compact
    .match_indices("<para")
    .map(|(at, _)| {
      &compact[at..at + compact[at..].find("</para>").expect("closed para") + "</para>".len()]
    })
    .collect();
  let expected: Vec<String> = texts
    .iter()
    .enumerate()
    .map(|(i, text)| {
      let id = if compact.contains(r#"<para xml:id=""#) {
        format!(r#" xml:id="p{}""#, i + 1)
      } else {
        String::new()
      };
      format!("<para{id}><p>{text}</p></para>")
    })
    .collect();
  assert_eq!(found, expected, "\n{xml}");
}

fn occurrences(stderr: &str, message: &str) -> usize { stderr.matches(message).count() }

/// `absorbing`: `\toks0={abc`, `\write16{abc`, `\uppercase{abc`, `\message{abc`
/// each end at their file's end and the document goes on (pdflatex 4 errors,
/// "A B [abc ] C ABC D E"). `\message` reads a text since 56jj (`XGeneralText`).
#[test]
fn a_file_ending_inside_a_text_ends_the_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/file_end_absorbing_text.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 4, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for cs in [r"\toks", r"\write", r"\uppercase", r"\message"] {
    let message = format!("File ended while scanning text of {cs}");
    assert_eq!(occurrences(&stderr, &message), 1, "{message}:\n{stderr}");
  }
  assert_para(&xml, "<p>A B [abc ] C ABC D E</p>");
}

/// `matching`: a macro argument that runs off its file, undelimited (`\foo{abc`,
/// `\foo` alone) or delimited (`\bar abc`, `\bar a{bc` for `\def\bar#1.`),
/// abandons the call: nothing of it is typeset (pdflatex 4 errors, "A B C D E";
/// Rust ran the macro on the partial or an empty argument).
#[test]
fn a_file_ending_inside_a_macro_argument_drops_the_call() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/file_end_matching_argument.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 4, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for cs in [r"\foo", r"\bar"] {
    let message = format!("File ended while scanning use of {cs}");
    assert_eq!(occurrences(&stderr, &message), 2, "{message}:\n{stderr}");
  }
  assert_para(&xml, "<p>A B C D E</p>");
}

/// `aligning`: `\halign{#abc` ends at its file's end with `\cr}`, so the empty
/// alignment closes there and "B" is kept (pdflatex 1 error; Rust read the
/// preamble on into the document, 5 errors, "B" lost).
#[test]
fn a_file_ending_inside_a_preamble_ends_the_alignment() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/file_end_aligning_preamble.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    occurrences(&stderr, r"File ended while scanning preamble of \halign"),
    1,
    "{stderr}"
  );
  assert_para(&xml, "<p>A</p><p>B</p>");
}

/// `defining` for an unexpanded body, a `\long` macro's argument, and
/// `\lstnewenvironment`, whose codes are macro arguments in listings.sty, not
/// definitions: pdflatex 3 errors ("use of \lstnewenvironment@", "definition
/// of \y", "use of \z") and "A B C D E [macro:->abc ] F". The listing codes keep
/// Perl's stop (a primitive's arguments), so the end code does not read on into
/// the document (with the definition recovery it met "B": 4 errors).
#[test]
fn a_def_body_a_long_argument_and_listing_codes() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\begin{filecontents*}{lst-child.tex}
\lstnewenvironment{foo}{start
\end{filecontents*}
\begin{filecontents*}{def-child.tex}
\def\y{abc
\end{filecontents*}
\begin{filecontents*}{long-child.tex}
\z{abc
\end{filecontents*}
\long\def\z#1{[#1]}
\begin{document}
A \input lst-child.tex B {C} D \input def-child.tex E [\meaning\y] \input long-child.tex F
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 3, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    occurrences(&stderr, "File ended while scanning definition"),
    1,
    "{stderr}"
  );
  assert_eq!(
    occurrences(&stderr, r"File ended while scanning use of \z"),
    1,
    "{stderr}"
  );
  assert_para(&xml, "<p>A B C D E [macro:-¿abc ] F</p>");
}

/// A typed argument (`\multispan{Number}`) that runs off its file is dropped
/// with its call and not re-parsed as a number (`Plain`'s inner parameters).
/// One error; pdflatex reports two because its `\multispan` (latex.ltx) runs
/// `\omit` ("Misplaced \omit") before `\@multispan` reads the argument, where
/// the binding reads it first. The tip reported 24 (`\omit`/`\span` ×23).
#[test]
fn a_typed_argument_that_runs_off_is_not_reparsed() {
  let tex = r"\documentclass{article}
\begin{filecontents*}{ms-child.tex}
\multispan{12
\end{filecontents*}
\begin{document}
A \input ms-child.tex B
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    occurrences(&stderr, r"File ended while scanning use of \multispan"),
    1,
    "{stderr}"
  );
  assert_para(&xml, "<p>A B</p>");
}

/// A non-`\long` macro's argument that holds a `\par` and runs off its file:
/// TeX stops at the `\par` ("Paragraph ended before \red was complete", tex.web
/// §396) and puts it back, so the text after the blank line is kept; a `\long`
/// macro reads on and is dropped with all of it ("File ended while scanning use
/// of \lred"). `\textcolor` (color.sty:104, not `\long`) and a delimited
/// argument alike. pdflatex 4 errors, paragraphs "A" / "DEF B" / "JKL C D" /
/// "b E" (it names `\@textcolor`, the binding `\textcolor`).
#[test]
fn a_par_in_a_runaway_argument_ends_a_non_long_call() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/file_end_matching_par.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 4, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for cs in [r"\red", r"\textcolor", r"\bar"] {
    let message = format!("Paragraph ended before {cs} was complete");
    assert_eq!(occurrences(&stderr, &message), 1, "{message}:\n{stderr}");
  }
  assert_eq!(
    occurrences(&stderr, r"File ended while scanning use of \lred"),
    1,
    "{stderr}"
  );
  for (id, text) in [
    ("p1", "A"),
    ("p2", "DEF B"),
    ("p3", "JKL C D"),
    ("p4", "b E"),
  ] {
    assert_element(
      &xml,
      "para",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(r#"<para xml:id="{id}"><p>{text}</p></para>"#),
    );
  }
}

/// The file that ended is closed when the call is dropped (tex.web §362 closes
/// it before recovering), so reading goes on in the enclosing input:
/// `\expandafter\a\foo{abc` gives `\a` the `X` after `\input`, and an `\edef`
/// around the `\input` ends at its own `}` with an empty body. pdflatex 2
/// errors, "A <X>Y B C [macro:->]" (W9 before the close: 5 errors).
#[test]
fn a_dropped_call_reads_on_in_the_enclosing_input() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/file_end_matching_closes_the_file.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    occurrences(&stderr, r"File ended while scanning use of \foo"),
    2,
    "{stderr}"
  );
  // `<`, `>` and `>` render through OT1 as `¡`, `¿`.
  assert_para(&xml, "<p>A ¡X¿Y B C [macro:-¿]</p>");
}

/// Negative control: a `\scantokens` pseudo-file is no file level here, so an
/// argument that runs off one keeps the behaviour from before the scanner
/// status: a group crosses into the enclosing input (`[abc def]`), a single
/// token reads nothing (`[]`), no error. (pdflatex treats the pseudo-file as a
/// file: 3 errors, "A def B Z".)
#[test]
fn a_scantokens_end_is_not_a_file_end() {
  let tex = r"\documentclass{article}
\def\foo#1{[#1]}
\begin{document}
A \scantokens\expandafter{\expandafter\foo\string{abc}def} B \scantokens{\foo}Z
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_para(&xml, "<p>A [abc def] B []Z</p>");
}

/// 62h (user ruling 2026-10-04): a `\par` read while scanning a non-`\long` macro's arguments ends the call — tex.web
/// §392/§399 "Paragraph ended before \x was complete", the `\par` put back (§396), the call dropped — for an
/// undelimited, a delimited (one- and two-token delimiters), a braced and a `#{` argument, a `\par` that breaks a partial
/// delimiter match (§397), and a `\let` copy, named in the error (the meaning carries the long-ness). Before, the arguments were
/// read on, and the two-token delimiter ran to the end of the file (a Fatal). pdftex: 7 errors, paragraphs
/// A / B / C / D/ / E / F / G / H / I / J / K / x / !L / M / N. Witness 1001.1670.
#[test]
fn a_par_ends_a_non_long_macro_argument() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/par_ends_a_non_long_argument.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 7, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (cs, n) in [
    (r"\x", 2),
    (r"\d", 1),
    (r"\w", 1),
    (r"\u", 1),
    (r"\r", 1),
    (r"\y", 1),
  ] {
    let message = format!("Paragraph ended before {cs} was complete");
    assert_eq!(occurrences(&stderr, &message), n, "{message}:\n{stderr}");
  }
  assert_paras(&xml, &[
    "A", "B", "C", "D/", "E", "F", "G", "H", "I", "J", "K", "x", "!L", "M", "N",
  ]);
}

/// 62h: the negative controls — a `\long` macro, a macro or `\let` copy meaning `\par`, a `\par` its delimiter
/// matches, and one that continues a partial match of `x\par!` — end no call (tex.web §392 compares the token `\par`
/// only, §397 after the delimiter test). pdftex: 0 errors.
#[test]
fn a_par_the_kernel_does_not_see_is_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/par_the_kernel_does_not_see_is_kept.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_paras(&xml, &[
    "A[a",
    "b]C[a",
    "b]D[a",
    "b]E\u{a1}a",
    "b\u{bf}F\u{a1}ax",
    "b\u{bf}G",
  ]);
}

/// 62h: LaTeX's starred `\newcommand*` makes a non-`\long` macro (latex.ltx:1229-1233), so its argument ends at a
/// blank line; `\newcommand` and `\def` keep theirs as TeX does (the unstarred `\long` is not recorded, as in Perl,
/// and is not checked). pdflatex: "Paragraph ended before" \a and \bb, two "Too many }'s", "y" / "v [C:p" / "q]".
#[test]
fn a_starred_newcommand_argument_ends_at_a_par() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\def\a#1{[A:#1]}
\newcommand*\bb[1]{[B:#1]}
\newcommand\cc[1]{[C:#1]}
\begin{document}
\a{x

y}
\bb{u\par v}
\cc{p

q}
\end{document}",
    false,
  );
  // The two `}` left over after the dropped calls are TeX's "Too many }'s".
  assert_eq!(error_count(&stderr), 4, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for cs in [r"\a", r"\bb"] {
    let message = format!("Paragraph ended before {cs} was complete");
    assert_eq!(occurrences(&stderr, &message), 1, "{message}:\n{stderr}");
  }
  assert_paras(&xml, &["y", "v [C:p", "q]"]);
}

/// 62h: the format's own non-`\long` macros check too — plain.tex's `\textindent` and `\loop` (plain.tex:634, 527),
/// whose raw definitions the dump carries (flag `T`, `dump_writer`) over the bindings'. The bindings themselves stay
/// unchecked, so without a dump (`LATEXML_NODUMP`) the kernel's bound `\textindent` reads on, as Perl does.
/// pdftex: "Paragraph ended before" \textindent and \loop, "Too many }'s", "A" / "bB" / "C".
#[test]
fn a_format_macro_argument_ends_at_a_par() {
  if !dump_available() {
    eprintln!(
      "SKIP a_format_macro_argument_ends_at_a_par: no kernel dump in resources/dumps/ (run tools/make_formats.sh)"
    );
    return;
  }
  let (stderr, xml) = convert(
    r"\count255=0 A\textindent{a\par b}B\loop x\par\advance\count255 1 \ifnum\count255<2 \repeat C
\bye",
    true,
  );
  assert_eq!(error_count(&stderr), 3, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for cs in [r"\textindent", r"\loop"] {
    let message = format!("Paragraph ended before {cs} was complete");
    assert_eq!(occurrences(&stderr, &message), 1, "{message}:\n{stderr}");
  }
  assert_paras(&xml, &["A", "bB", "C"]);
}

/// 62h: the declarators that keep TeX's long-ness reach the check too — starred `\DeclareRobustCommand*` (its inner
/// `\rb␣`) and `\newenvironment*`, both `\@star@or@long` in latex.ltx; and etoolbox's `\patchcmd`, which re-`\def`s
/// the macro with its prefix (etoolbox.sty:1358-1371), so a patched non-`\long` macro still checks. pdflatex:
/// "Paragraph ended before" \rb␣, \e and \foo (each followed by its stray `}`), "A" / "bB" / "cC" / "D" / "bE".
#[test]
fn starred_robust_environment_and_patched_macros_check() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{etoolbox}
\DeclareRobustCommand*\rb[1]{[#1]}
\newenvironment*{e}[1]{<#1>}{}
\def\foo#1{[#1]}\patchcmd\foo{[}{(}{}{}
\begin{document}
A\rb{a\par b}B

\begin{e}{a\par c}C\end{e}

D\foo{a\par b}E
\end{document}",
    false,
  );
  for cs in [r"\rb ", r"\e", r"\foo"] {
    let message = format!("Paragraph ended before {cs} was complete");
    assert_eq!(occurrences(&stderr, &message), 1, "{message}:\n{stderr}");
  }
  assert_paras(&xml, &["A", "bB", "cC", "D", "bE"]);
}

/// 62h: `\@notdefinable` passes latex.ltx's help text `\@eha` (latex.ltx:8969-8974), so `\@latex@error`'s second
/// argument is not the document's next token: the `X` is kept (pdflatex "AXYZ"; Perl "AYZ", KPE #473), and a blank line
/// after it is no "Paragraph ended before \@latex@error" (jlreq's `\NewBlockHeading{section}`).
#[test]
fn a_notdefinable_error_keeps_the_next_token() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\makeatletter
\begin{document}
A\@ifdefinable\section{}XYZ

B\@ifdefinable\section{}

C
\end{document}",
    false,
  );
  assert_eq!(occurrences(&stderr, "Paragraph ended"), 0, "{stderr}");
  assert_paras(&xml, &["AXYZ", "B", "C"]);
}
