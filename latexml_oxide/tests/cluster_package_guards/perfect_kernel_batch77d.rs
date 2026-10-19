//! Red/green guards for batch 77d: the author-name splitter reads the separators pdflatex prints between names where
//! no space token stands beside them (`spaced_name_separators`, base_utilities.rs; OXIDIZED_DESIGN_DIVERGENCES #765) —
//! "and" after a control word or a note, beside a line break (svmult `and\\`, jfm `\break and`), `$^\ast$\&`; a
//! separator macro that lowercases its "and" (cimento's `\atque`); a comma written inside a mark's math (`$^{1}, $`).
//! Perl merges each of these into one author, silently (Base_Utility.pool.ltxml:678); 67int29 merged them too, with
//! `Error:frontmatter:merged_creators` or with none. Each guard asserts the three measures: no Error or Warning, every
//! name once, each in its own `ltx:creator` with its notes and affiliations.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A `sectioning-frontmatter` repro's log and squashed XML.
macro_rules! convert_repro {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $path
      )),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// The document's `ltx:creator` elements, in order, each whole.
fn creators(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end =
        start + xml[start..].find("</creator>").expect("closed creator") + "</creator>".len();
      &xml[start..end]
    })
    .collect()
}

/// A creator holding exactly one personname, `name`, and the rest `tail` (contacts, notes), as the squashed XML writes
/// it.
fn creator(first: bool, name: &str, tail: &str) -> String {
  let before = if first { "" } else { r#"before=" " "# };
  let tail = if tail.is_empty() {
    String::new()
  } else {
    format!("{tail} ")
  };
  format!(r#"<creator {before}role="author"> <personname>{name}</personname> {tail}</creator>"#)
}

/// No personname holds a separator: the names are apart.
fn assert_no_merged_names(xml: &str) {
  for piece in xml.split("<personname>").skip(1) {
    let name = &piece[..piece.find("</personname>").unwrap()];
    assert!(
      !name.contains(" and ") && !name.contains("&amp;"),
      "merged: {name}\n{xml}"
    );
  }
}

/// "and" with no space token on one side — after `\space`, a spacing-only macro (jfm's `\ns`), a control return, a
/// `\thanks` note; before `\;`; `\&` after a star mark — is a separator: seven authors, Dan Dorn's note his, Fay Fox's
/// star hers (was one creator holding all seven names, no diagnostic).
#[test]
fn and_after_a_control_word_separates_names() {
  let xml = convert_repro!("author_and_after_a_control_word.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", ""),
    creator(false, "Bob Baker", ""),
    creator(false, "Cat Cole", ""),
    creator(
      false,
      "Dan Dorn",
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Univ D</note>"#
    ),
    creator(false, "Eve Ess", ""),
    creator(false, r#"Fay Fox<sup xml:id="id2">∗</sup>"#, ""),
    creator(false, "Gus Gray", ""),
  ]);
}

/// svmult's `and\\`, `\break and`, `\linebreak and`: four authors, Ann Able's ORCID and both affiliations on their
/// names (was one creator "Ann Able and<break/>Bob Bakerand Cat Coleand Dan Dorn").
#[test]
fn and_beside_a_line_break_separates_names() {
  let xml = convert_repro!("author_and_beside_a_line_break.tex");
  assert_no_merged_names(&xml);
  let found = creators(&xml);
  assert_eq!(found.len(), 4, "{xml}");
  assert!(
    found[0].starts_with(
      r#"<creator role="author"> <personname>Ann Able</personname> <contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0003-0647-7371""#
    ),
    "{xml}"
  );
  assert!(
    found[0].ends_with(
      "</ref></contact> <contact name=\"Affiliation: \" role=\"affiliation\">Univ A</contact> </creator>"
    ),
    "{xml}"
  );
  assert_eq!(found[1..], [
    creator(
      false,
      "Bob Baker",
      "<contact name=\"Affiliation: \" role=\"affiliation\">Univ B</contact>"
    ),
    creator(false, "Cat Cole", ""),
    creator(false, "Dan Dorn", ""),
  ]);
}

/// A separator macro whose "and" is set through `\MakeLowercase{and\ \ignorespaces}` (cimento's `\atque`) or
/// `\lowercase{AND}` prints "and": five authors (was "Bob Baker and Cat Cole and Dan Dorn and Eve Ess" one creator +
/// `Error:frontmatter:merged_creators`).
#[test]
fn lowercased_separator_macro_separates_names() {
  let xml = convert_repro!("author_separator_macro_lowercase.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", ""),
    creator(false, "Bob Baker", ""),
    creator(false, "Cat Cole", ""),
    creator(false, "Dan Dorn", ""),
    creator(false, "Eve Ess", ""),
  ]);
}

/// A comma written inside the math of a name's mark (`$^{1}, $`, `$,^{2}$`, `$^{2},$`) separates the names, and each
/// mark stays with the name it follows: five authors with their affiliations (was two creators holding all five names,
/// their marks lost, + 2 `Error:frontmatter:merged_creators`).
#[test]
fn comma_in_mark_math_separates_names_and_keeps_marks() {
  let xml = convert_repro!("author_comma_in_mark_math.tex");
  assert_no_merged_names(&xml);
  let univ =
    |u: &str| format!("<contact name=\"Affiliation: \" role=\"affiliation\">Univ {u}</contact>");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ("A")),
    creator(false, "Bob Baker", &univ("A")),
    creator(false, "Cat Cole", &univ("B")),
    creator(false, "Dan Dorn", &univ("B")),
    creator(false, "Eve Ess", &univ("A")),
  ]);
}

/// Negative guard: "and" inside a word stays in its name — after a letter ("Rand"), before one ("Anders"), after an
/// accent's argument (`Zo\"{e}and`), a letter command's group (`{\O}and`) or a letter command (`Ra\dj and`), and after
/// a control symbol that skips no space, `\-` or `\/` (`Rol\-and Smith`, `Bertr\/and Smith`: 77d r1 split them into
/// "Rol" | "Smith", "Bertr" | "Smith").
#[test]
fn and_inside_a_word_stays_the_name() {
  let xml = convert_repro!("author_and_inside_a_word_stays_the_name.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Rand", ""),
    creator(false, "Bob Anders", ""),
    creator(false, "Cat Zoëand Lee", ""),
    creator(false, "Dan Øand Fox", ""),
    creator(false, "Eve Sandén", ""),
    creator(false, "Roland Smith", ""),
    creator(false, "Bertrand Smith", ""),
    creator(false, "Gus Rađand Lee", ""),
  ]);
}

/// A whole-line group whose names are parted by "and" after `\break` or a `\thanks` note splits as the same names at
/// the top level do (the group's name count reads the separators written), and a declaration before a separator
/// leaves no space in the name before it (`\sc Eve Ess \rm and`; 77d r1 kept "Eve Ess "): six authors, Cat Cole's
/// note his (was "Ann Ableand Bob Baker", "Cat Coleand Dan Dorn", silently).
#[test]
fn and_after_a_break_in_a_wrapper_separates_names() {
  let xml = convert_repro!("author_and_after_a_break_in_a_wrapper.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", ""),
    creator(false, "Bob Baker", ""),
    creator(
      false,
      "Cat Cole",
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1"><text font="bold" xml:id="id1.1">Univ C</text></note>"#
    ),
    creator(false, "Dan Dorn", ""),
    creator(
      false,
      r#"<text font="smallcaps" xml:id="id2">Eve Ess</text>"#,
      ""
    ),
    creator(
      false,
      r#"<text font="smallcaps" xml:id="id3">Fay Fox</text>"#,
      ""
    ),
  ]);
}

/// amsart's and revtex's `\author` is one author unless its names line reads as several (`split_author_line_cautious`):
/// `Ann Able\break and Bob Baker` reads as two with its separator written, as the PDF prints them on two lines (was
/// one creator "Ann Ableand Bob Baker", silently).
#[test]
fn cautious_split_reads_and_after_a_break() {
  for (class, second) in [
    ("amsart", r#"<creator before=" and " role="author">"#),
    ("revtex4-2", r#"<creator before=" " role="author">"#),
  ] {
    let (log, xml) = convert_with(
      &format!(
        r"\documentclass{{{class}}}
\begin{{document}}
\title{{T}}
\author{{Ann Able\break and Bob Baker}}
\maketitle
x
\end{{document}}"
      ),
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{class}: {log}"
    );
    let xml = squash(&xml);
    assert_eq!(
      creators(&xml),
      [
        r#"<creator role="author"> <personname>Ann Able</personname> </creator>"#.to_string(),
        format!("{second} <personname>Bob Baker</personname> </creator>"),
      ],
      "{class}"
    );
  }
}

/// A separator macro reads as the text TeX prints: `\MakeUppercase` applies to the one token after it when no group
/// follows (`\ \MakeUppercase and\ ` prints "And", no separator: one author, as pdflatex shows "Ann Able And Bob
/// Baker"), and a macro of a spacing command with its braced argument (`\hspace{1em}`) is spacing beside "and".
#[test]
fn separator_macro_case_and_spacing_read_as_printed() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\newcommand\with{\ \MakeUppercase and\ }
\newcommand\sep{\hspace{1em}}
\begin{document}
\title{T}
\author{Ann Able\with Bob Baker \and Cat Cole\sep and Dan Dorn}
\maketitle
x
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able And Bob Baker", ""),
    creator(false, "Cat Cole", ""),
    creator(false, "Dan Dorn", ""),
  ]);
}

/// Names parted only by their marks and jfm's `\ns` (1911.02873: `FLORIAN REETZ$^1$ \ns PRIYA SUBRAMANIAN$^{2,3}$ \ns
/// \& \ns TOBIAS M.\ns SCHNEIDER$^1$`) stay one creator, and the merge check, reading a mark as ending the name before
/// it, flags it: exactly one `Error:frontmatter:merged_creators`, the `\&` still parting the last name (77d r1 dropped
/// the Error, a silent merge).
#[test]
fn names_parted_only_by_marks_are_flagged() {
  let (log, xml) = convert_with(
    r"\documentclass{jfm}
\begin{document}
\title{T}
\author[F. Reetz \textit{et al.}]{FLORIAN REETZ$^1$ \ns
PRIYA SUBRAMANIAN$^{2,3}$ \ns \& \ns TOBIAS M.\ns SCHNEIDER$^1$}
\affiliation{$^1$ Univ A\\ $^2$ Univ B\\ $^3$ Univ C}
\maketitle
x
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(
    log
      .lines()
      .filter(|l| l.contains("Error:frontmatter:merged_creators"))
      .count(),
    1,
    "{log}"
  );
  let xml = squash(&xml);
  let found = creators(&xml);
  assert_eq!(found.len(), 2, "{xml}");
  assert!(
    found[0].starts_with(
      r#"<creator role="author"> <personname>FLORIAN REETZ<sup xml:id="id1">1</sup> PRIYA SUBRAMANIAN<sup"#
    ),
    "{xml}"
  );
  assert!(
    found[1].starts_with(
      r#"<creator before=" " role="author"> <personname>TOBIAS M. SCHNEIDER<sup xml:id="id3">1</sup></personname>"#
    ),
    "{xml}"
  );
}

/// Negative guard for the mark-as-comma re-read of `flag_merged_creators`: one author carrying several marks — a mark
/// list, two superscripts, marks around an ORCID link, llncs `\inst`s with an `\orcidID`, a mark before a suffix — is
/// one creator, unflagged.
#[test]
fn single_names_with_several_marks_are_not_flagged() {
  for (class, packages, author) in [
    ("article", "", r"Ann Able$^{1,2}$"),
    (
      "article",
      "",
      r"Ann Able\textsuperscript{a}\textsuperscript{b}",
    ),
    (
      "article",
      r"\usepackage{orcidlink}",
      r"Ann Able$^{1}$\orcidlink{0000-0002-1825-0097}$^{,2}$",
    ),
    (
      "llncs",
      "",
      r"Ann Able\inst{1}\inst{2}\orcidID{0000-0002-1825-0097}",
    ),
    ("article", "", r"Ann Able$^{1}$ Jr."),
  ] {
    let tex = format!(
      "\\documentclass{{{class}}}\n{packages}\n\\begin{{document}}\n\\title{{T}}\n\\author{{{author}}}\n\\maketitle\nx\n\\end{{document}}"
    );
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{author}: {log}"
    );
    let xml = squash(&xml);
    let found = creators(&xml);
    assert_eq!(found.len(), 1, "{author}: {xml}");
    assert!(found[0].contains("<personname>Ann Able"), "{author}: {xml}");
  }
}

/// Negative guard: `\&` glued between letters is one word (`AT\&T`), and a group whose text ends in a letter continues
/// the word an "and" after it (`\textbf{Ann}and` prints "Annand"): each line stays one author.
#[test]
fn ampersand_and_and_inside_words_stay_one_author() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able \and AT\&T Labs \and Bob \textbf{Bak}and}
\maketitle
x
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", ""),
    creator(false, "AT&amp;T Labs", ""),
    creator(
      false,
      r#"Bob <text font="bold" xml:id="id1">Bak</text>and"#,
      ""
    ),
  ]);
}
