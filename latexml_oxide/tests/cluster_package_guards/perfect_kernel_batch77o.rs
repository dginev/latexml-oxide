//! Red/green guards for batch 77o (HF14, html_feedback refresh 2026-10-19): a position keyword is no printed text, so
//! the deproc title code's emptied `\@maketitle` deposit leaves no strut table (M1); the aistats family's title-block
//! setters are the frontmatter (M3); colm2024/2026 reach the COLM binding. Each asserts the three measures: no Error or
//! Warning, the authored content present, and its element.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Convert under the production preload; 0 errors, 0 warnings.
fn clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// The document's frontmatter: from its `<title>` to the first `<abstract`/`<section`/`<para`.
fn frontmatter(xml: &str) -> String {
  let xml = squash(xml);
  let start = xml.find("<title>").expect("a document title");
  let end = ["<abstract", "<section", "<para"]
    .iter()
    .filter_map(|tag| xml[start..].find(tag))
    .min()
    .map_or(xml.len(), |end| start + end);
  xml[start..end].trim().to_string()
}

const AISTATS: &str = include_str!(
  "../../../tools/perfect_kernel/repros/class-bindings/aistats_setters_are_the_frontmatter.tex"
);

const AISTATS_TWO_CREATORS: &str = r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Email: " role="email">ann@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact name="Email: " role="email">bob@b.edu</contact> </creator>"#;

/// The deproc title code (`\begin{tabular}[t]{l}\bf\rule{\z@}{24pt}\@author\end{tabular}`, iclr*_conference.sty:83-93,
/// arxiv.sty, colm, corl, nips) with the fields emptied prints nothing: the position letter is a keyword LaTeX only
/// tests (latex.ltx:16565-16566), so the deposit goes and no empty table with a zero-width rule stands before the
/// first section (it did in 94 of 6,000 sampled papers; 2610.20409, 2310.10873). Repro
/// sectioning-frontmatter/deposit_position_keyword_prints_nothing.
#[test]
fn deposit_position_keyword_prints_nothing() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/deposit_position_keyword_prints_nothing.tex"
  ));
  let flat = squash(&xml);
  let between =
    &flat[flat.find("</abstract>").expect("abstract")..flat.find("<section").expect("section")];
  assert_eq!(between, "</abstract> ", "{xml}");
  assert!(
    !flat.contains("<tabular") && !flat.contains("<rule"),
    "{xml}"
  );
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#,
    "{xml}"
  );
}

/// The `\parbox[t]` and `{minipage}[t]` positions are keywords too: a deposit of a strut in each prints nothing. Their
/// attachment is still read (`vattach`) where they print.
#[test]
fn parbox_and_minipage_positions_print_nothing() {
  let xml = clean(
    r"\documentclass{article}
\makeatletter
\def\@maketitle{\vbox{\hsize\textwidth{\LARGE \@title\par}%
  \begin{minipage}[t]{3cm}\rule{\z@}{24pt}\@author\end{minipage}\parbox[t]{2cm}{\rule{0pt}{1pt}}%
  \begin{tabular*}{5cm}[t]{l}\@author\end{tabular*}\vskip 0.3in}}
\makeatother
\title{A Title}
\author{Ann Able}
\begin{document}
\maketitle
Text \begin{minipage}[b]{2cm}m\end{minipage} \parbox[c]{1cm}{p} \begin{tabular}[b]{l}t\end{tabular}
\end{document}
",
  );
  let flat = squash(&xml);
  assert!(!flat.contains("<rule"), "{xml}");
  assert_eq!(flat.matches("<para ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "inline-block",
    &["class=\"ltx_minipage\""],
    r#"<inline-block class="ltx_minipage" vattach="bottom" width="56.9pt" xml:id="p1.1.1"><p xml:id="p1.1.1.1">m</p></inline-block>"#,
  );
  assert_element(
    &xml,
    "inline-block",
    &["class=\"ltx_parbox\""],
    r#"<inline-block class="ltx_parbox" vattach="middle" width="28.5pt" xml:id="p1.1.2"><p xml:id="p1.1.2.1">p</p></inline-block>"#,
  );
  assert_element(
    &xml,
    "tabular",
    &["xml:id=\"p1.1.3\""],
    r#"<tabular vattach="bottom" xml:id="p1.1.3"><tbody><tr xml:id="p1.1.3.1"><td align="left" xml:id="p1.1.3.1.1">t</td></tr></tbody></tabular>"#,
  );
}

/// Control: a class's `\@maketitle` whose `[t]` table and `\parbox[t]` hold text the class prints is kept. Repro
/// sectioning-frontmatter/deposit_position_table_with_text_is_kept.
#[test]
fn deposit_position_table_with_text_is_kept() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/deposit_position_table_with_text_is_kept.tex"
  ));
  assert_element(
    &xml,
    "tabular",
    &["xml:id=\"p1.2\""],
    r#"<tabular vattach="top" xml:id="p1.2"><tbody><tr xml:id="p1.2.1"><td align="left" xml:id="p1.2.1.1">Preprint. Under review.</td></tr></tbody></tabular>"#,
  );
  assert_element(
    &xml,
    "inline-block",
    &["class=\"ltx_parbox\""],
    r#"<inline-block class="ltx_parbox" vattach="top" width="142.3pt" xml:id="p1.3.1"><p xml:id="p1.3.1.1">Draft notice</p></inline-block>"#,
  );
}

/// aistats's `\aistatstitle`/`\aistatsauthor`/`\aistatsaddress` (aistats2027.sty:173/248/355, every year alike) are
/// the title and the authors, each address column under the author of its column (2610.11580, 2604.21432, 2605.03921,
/// 1002.4058; 1806.02924's one address under four names is the shared case, `aistats_one_column_names_share_the_address`). Repro class-bindings/aistats_setters_are_the_frontmatter.
#[test]
fn aistats_setters_are_the_frontmatter() {
  let xml = clean(AISTATS);
  assert_eq!(frontmatter(&xml), AISTATS_TWO_CREATORS, "{xml}");
  assert!(
    !xml.contains("<titlepage") && !xml.contains("<tabular"),
    "{xml}"
  );
}

/// A submission under review prints "Anonymous Author" and "Anonymous Institution" (aistats2020.sty …
/// aistats2027.sty), and so does the conversion (user ruling 2026-10-06: display toggles as the PDF shows them).
#[test]
fn aistats_under_review_is_anonymous() {
  let xml = clean(&AISTATS.replace(
    r"\usepackage[accepted]{aistats2027}",
    r"\usepackage{aistats2027}",
  ));
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Anonymous Author</personname> <contact name="Affiliation: " role="affiliation">Anonymous Institution</contact> </creator>"#,
    "{xml}"
  );
  assert!(!xml.contains("Ann Able"), "{xml}");
}

/// A row of one column (a comma-separated name list) under one address: the names are the authors and the address,
/// marked, goes to the authors whose marks it names (1910.09056's `$^1$University of British Columbia, $^2$…`);
/// unmarked, to all of them (2510.09534's one University of Chicago under three names, 1806.02924's one Carnegie Mellon
/// address under four; `\lx@add@affiliation@shared`).
#[test]
fn aistats_marked_address_goes_to_its_authors() {
  let marked = AISTATS
    .replace(
      r"\aistatsauthor{Ann Able \And Bob Baker}",
      r"\aistatsauthor{Ann Able$^{1}$, Bob Baker$^{2}$}",
    )
    .replace(
      r"\aistatsaddress{Univ A \\ ann@a.edu \And Univ B \\ bob@b.edu}",
      r"\aistatsaddress{$^1$Univ A, $^2$Univ B}",
    );
  let xml = clean(&marked);
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#,
    "{xml}"
  );
  let shared = AISTATS.replace(
    r"\aistatsaddress{Univ A \\ ann@a.edu \And Univ B \\ bob@b.edu}",
    r"\aistatsaddress{Univ C}",
  );
  let xml = clean(&shared);
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator>"#,
    "{xml}"
  );
}

/// A later `\aistatstitle` (the supplement's, after `\onecolumn`) prints in place as the copy does; the document's title
/// stays the first (2610.11580: "Supplementary Material").
#[test]
fn aistats_later_title_prints_in_place() {
  let xml = clean(&AISTATS.replace(
    r"\section{Intro} Body.",
    r"\section{Intro} Body.
\onecolumn
\aistatstitle{Supplementary Material}
\section{More} Body.",
  ));
  assert_eq!(frontmatter(&xml), AISTATS_TWO_CREATORS, "{xml}");
  // the heading, once, where it stands: after the first section's paragraph, as the copy's centred bold title
  assert_eq!(xml.matches("Supplementary Material").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"S1.p2\""],
    r#"<para align="center" xml:id="S1.p2"><p xml:id="S1.p2.1"><text font="bold" fontsize="144%" xml:id="S1.p2.1.1">Supplementary Material</text></p></para>"#,
  );
}

/// The family binding answers every year through the versioned fallback: aistats2019's copy (no anonymous block,
/// 1806.02924) prints the names under review too.
#[test]
fn aistats_family_answers_every_year() {
  let (log, xml) = convert_files_with(
    "\\documentclass{article}\n\\usepackage{aistats2019}\n\\begin{document}\n\\twocolumn[\\aistatstitle{T}\n\
     \\aistatsauthor{Ann Able}\n\n\\aistatsaddress{Univ A}]\nBody.\n\\end{document}\n",
    &[(
      "aistats2019.sty",
      "\\ProvidesPackage{aistats2019}\n\\newcommand{\\statePaper}{0}\\newcommand{\\acceptedPaper}{1}\n\
       \\long\\def\\aistatstitle#1{{\\centering{\\Large\\bfseries #1\\par}}}\n\
       \\long\\def\\aistatsauthor#1{{\\begin{tabular}[t]{c}\\bfseries #1\\end{tabular}}\\par}\n\
       \\long\\def\\aistatsaddress#1{{\\begin{tabular}[t]{c}#1\\end{tabular}}\\par}\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(log.contains("falling back to aistats.sty"), "{log}");
  assert_eq!(
    frontmatter(&xml),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#,
    "{xml}"
  );
}

/// colm2024_conference and colm2026_conference reach the COLM binding (colm2025_conference_sty.rs) through the versioned
/// fallback, as one binding serves a family: its `\ifcolmpreprint` (which colm2024's copy lacks) and its separators
/// (2404.05892, 2404.07921, 2406.07546, 2605.21807, 2608.09164).
#[test]
fn colm_family_answers_every_year() {
  let (log, xml) = convert_files_with(
    "\\documentclass{article}\n\\usepackage{colm2024_conference}\n\\title{T}\n\\author{Ann Able \\And Bob Baker}\n\
     \\begin{document}\n\\maketitle\n\\ifcolmpreprint Preprint.\\else Final.\\fi\n\\end{document}\n",
    &[(
      "colm2024_conference.sty",
      "\\ProvidesPackage{colm2024_conference}\n\\newif\\ifcolmfinal\n\\def\\addcontentsline#1#2#3{}\n\
       \\def\\toptitlebar{\\hrule height4pt\\vskip .25in}\n\\def\\bottomtitlebar{\\vskip .29in\\hrule height1pt}\n\
       \\def\\And{\\end{tabular}\\hfil\\begin{tabular}[t]{l}}\n\\def\\AND{\\end{tabular}\\par\\begin{tabular}[t]{l}}\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(log.contains("falling back to colm.sty"), "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p xml:id="p1.1">Final.</p></para>"#,
  );
  assert_eq!(
    frontmatter(&xml),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator>"#,
    "{xml}"
  );
}

/// Without the paper's copy the binding stands in for it, as the other stand-in bindings do (icml, neurips): the
/// family's setters are still the frontmatter.
#[test]
fn aistats_without_a_copy_keeps_the_frontmatter() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{aistats2025}\n\\begin{document}\n\\twocolumn[\\aistatstitle{T}\n\
     \\aistatsauthor{Ann Able \\And Bob Baker}\n\\aistatsaddress{Univ A \\And Univ B}]\nBody.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    frontmatter(&xml),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#,
    "{xml}"
  );
}

// ---------------------------------------------------------------------------------------------------------------------
// Round 2 (77o review): a shared address goes to every author it is printed under; an unpaired address is split at its
// columns; the 2010-2011 templates reach the binding; a cross-reference is ink; a box's position is a parameter type.

/// A row of one column (names apart by `\quad`, not `\And`) under one address: the copy centres the address under all
/// the names, so it is each author's affiliation, and its e-mail line gives each author an address in their order
/// (2510.09534's `Jeong$^\ast$ \quad Zhai$^\ast$ \quad Ro\v{c}kov\'{a}`; 1806.02924's `{asuggala, …}@cs.cmu.edu`). Repro
/// class-bindings/aistats_one_column_names_share_the_address.
#[test]
fn aistats_one_column_names_share_the_address() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aistats_one_column_names_share_the_address.tex"
  ));
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id2">able@x.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id4">baker@x.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id5">cole@x.edu</text></contact> </creator>"#,
    "{xml}"
  );
}

/// An address whose columns are not the names' is split at its columns, each an affiliation of every author of the row
/// — never one affiliation "Univ A &Univ B" — and `\ANDD` is a column break among the names (review probes `a_mismatch`,
/// `a_noaddr`). Repro class-bindings/aistats_unpaired_address_columns_split.
#[test]
fn aistats_unpaired_address_columns_split() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aistats_unpaired_address_columns_split.tex"
  ));
  let both = r#"<contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Affiliation: " role="affiliation">Univ B</contact>"#;
  assert_eq!(
    frontmatter(&xml),
    format!(
      r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> {both} </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> {both} </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> {both} </creator>"#
    ),
    "{xml}"
  );
}

/// A marked row over an address with one mark only — a note's (`$^*$equal contribution`, `Univ A$^*$` twice) — still
/// pairs column by column, as the PDF prints column i under author i: an address is a key the names' marks index only
/// when it shows two or more distinct marks (2312.06786's marked affiliations, `aistats_marked_address_goes_to_its_authors`).
/// 77o r3 review probes m_lead, m_star; 1905.11190, 2405.18281, 2011.03885, 2301.11862, 2410.15516 (r3 gave every author
/// every column).
#[test]
fn aistats_note_marked_address_still_pairs_by_column() {
  for (names, address, expected) in [
    (
      r"Ann Able$^*$ \And Bob Baker$^*$ \And Cy Cole",
      r"Univ A \And Univ B \\ $^*$equal contribution \And Univ C",
      // (the `$^*$` legend line the starred names' note, ruling 2 — the writer's block-line cut, 94a: it was Bob Baker's
      // affiliation, the cut declining a mark whose formula's closing `$` stands before its line)
      r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id2"><sup xml:id="id2.1">∗</sup>equal contribution</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id4"><sup xml:id="id4.1">∗</sup>equal contribution</note> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator>"#,
    ),
    (
      r"Ann Able \And Bob Baker \And Cy Cole$^\dagger$",
      r"Univ A$^*$ \And Univ B \And Univ A$^*$",
      r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A<sup xml:id="id1">*</sup></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole<sup xml:id="id2">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A<sup xml:id="id3">*</sup></contact> </creator>"#,
    ),
  ] {
    let xml = clean(&format!(
      "\\documentclass[twoside]{{article}}\n\\usepackage[accepted]{{aistats2027}}\n\\begin{{document}}\n\\twocolumn[\\aistatstitle{{A Title}}\n\\aistatsauthor{{{names}}}\n\\aistatsaddress{{{address}}}]\n\\section{{Intro}} Body.\n\\end{{document}}\n"
    ));
    assert_eq!(frontmatter(&xml), expected, "{names}: {xml}");
  }
}

/// The same row and address with material that prints nothing between the setters: the address is still paired column
/// by column, as the PDF prints column i under author i (77o review scan: 29 papers — `\vspace` 2003.03080, 2305.16147;
/// `\runningauthor` 1804.04378, 2402.12508; `\medskip` 2602.12181; `~` 1603.09584; `\renewcommand\thefootnote`
/// 2011.03610; `{\fontsize…\selectfont` 2501.00988; `\newcommand` 1302.5348, 1902.01239). Repro
/// class-bindings/aistats_address_after_unprinted_material_is_paired.
#[test]
fn aistats_address_after_unprinted_material_is_paired() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aistats_address_after_unprinted_material_is_paired.tex"
  ));
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ M</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dorn</personname> <contact name="Affiliation: " role="affiliation">Univ D</contact> </creator> <creator before=" " role="author"> <personname>Eve East</personname> <contact name="Affiliation: " role="affiliation">Univ E</contact> <contact name="Affiliation: " role="affiliation">Univ F</contact> </creator> <creator before=" " role="author"> <personname>Fay Fox</personname> <contact name="Affiliation: " role="affiliation">Univ E</contact> <contact name="Affiliation: " role="affiliation">Univ F</contact> </creator>"#,
    "{xml}"
  );
  let paired = r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#;
  for (between, after) in [
    (r"\vspace{2pt}", ""),
    (r"\vspace*{-0.1cm}", ""),
    (r"\runningauthor{Able, Baker}", ""),
    (r"\medskip", ""),
    (r"\vskip 12pt", ""),
    (r"~\\", ""),
    (r"\renewcommand{\thefootnote}{\arabic{footnote}}", ""),
    (r"\begingroup \renewcommand\thefootnote{} \endgroup", ""),
    (
      r"\newcommand{\authorsize}{\tiny} \newcommand{\emailsize}{\bf \scriptsize}",
      "",
    ),
    (r"{\fontsize{9.6pt}{12pt}\selectfont", "}"),
  ] {
    // (the copy's running-head setter, aistats2027.sty:171)
    let xml = clean(
      &AISTATS
        .replace(
          r"\begin{document}",
          r"\makeatletter\def\runningauthor#1{\gdef\@runningauthor{#1}}\makeatother
\begin{document}",
        )
        .replace(
          r"\aistatsaddress{Univ A \\ ann@a.edu \And Univ B \\ bob@b.edu}",
          &format!("{between}\n\\aistatsaddress{{Univ A \\And Univ B}}{after}"),
        ),
    );
    assert_eq!(frontmatter(&xml), paired, "{between}: {xml}");
  }
}

/// A paragraph the block prints between the setters ends the look-ahead: the address, met on its own, is split at its
/// columns, each an affiliation of the row, and the paragraph is printed where it stands.
#[test]
fn aistats_address_after_printed_material_is_split() {
  let xml = clean(&AISTATS.replace(
    r"\aistatsaddress{Univ A \\ ann@a.edu \And Univ B \\ bob@b.edu}",
    r"\textbf{Equal contribution.}

\aistatsaddress{Univ A \And Univ B}",
  ));
  let both = r#"<contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Affiliation: " role="affiliation">Univ B</contact>"#;
  assert_eq!(
    frontmatter(&xml),
    format!(
      r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> {both} </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> {both} </creator>"#
    ),
    "{xml}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Equal contribution.</text></p>"#,
  );
}

/// A row with no address: the names are rebuilt from their columns, `\ANDD` a column break as the copy defines it, not
/// an undefined command inside one name "Bob Baker \ANDD Cat Cole" (review probe `a_noaddr`).
#[test]
fn aistats_row_without_address_splits_at_every_column_break() {
  let xml = clean(
    &include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aistats_unpaired_address_columns_split.tex"
    )
    .replace("\n\\aistatsaddress{Univ A \\And Univ B}", ""),
  );
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> </creator>"#,
    "{xml}"
  );
}

/// A macro the paper defines inside the title block's group, between the setters (1302.5348's
/// `\newcommand{\UMDAddress}{University of Maryland \\ College Park, MD 20742, USA}`), is gone by the time the queued
/// frontmatter is digested: the setter reads it where it stands, after the definition, which prints nothing, so the
/// address is still paired column by column (each line of a column an affiliation of its author, as the kernel `\author`
/// reads `Name \\ line \\ line`; the e-mail line theirs).
#[test]
fn aistats_title_block_macro_is_read_in_place() {
  let xml = clean(&AISTATS.replace(
    r"\aistatsaddress{Univ A \\ ann@a.edu \And Univ B \\ bob@b.edu}",
    r"\newcommand{\Addr}{Univ M \\ College Park}
\aistatsaddress{\Addr \\ \texttt{ann@m.edu} \And \Addr \\ \texttt{bob@m.edu}}",
  ));
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ M</contact> <contact name="Affiliation: " role="affiliation">College Park</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">ann@m.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ M</contact> <contact name="Affiliation: " role="affiliation">College Park</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id2">bob@m.edu</text></contact> </creator>"#,
    "{xml}"
  );
}

/// A column that assigns or tests a meaning is not read in place: the macro it renews is the one it names (review
/// probe `g_renewin`: "rstSecondFirst" for pdflatex's "Second"), and 2511.05163's names, a table under
/// `\renewcommand{\arraystretch}{1.0}`, keep their command; the next column is read in place, and a global macro renewed
/// inside the block with its local meaning, as pdflatex prints it (probe `g_globlocal`). Repro
/// class-bindings/aistats_in_place_leaves_assignments_alone.
#[test]
fn aistats_in_place_leaves_assignments_alone() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aistats_in_place_leaves_assignments_alone.tex"
  ));
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Second</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Local Univ</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Other</contact> </creator>"#,
    "{xml}"
  );
  let stretched = clean(&AISTATS.replace(
    r"\aistatsauthor{Ann Able \And Bob Baker}",
    r"\renewcommand{\arraystretch}{1.5}
\aistatsauthor{\begingroup\renewcommand{\arraystretch}{1.0}\begin{tabular}{cc}Ann Able & Bob Baker\end{tabular}\endgroup}",
  ));
  assert_element(
    &stretched,
    "tabular",
    &["xml:id=\"id1\""],
    r#"<tabular vattach="middle" xml:id="id1"><tbody><tr xml:id="id1.1"><td align="center" xml:id="id1.1.1">Ann Able</td><td align="center" xml:id="id1.1.2">Bob Baker</td></tr></tbody></tabular>"#,
  );
}

/// Each row's shared address goes to that row's authors only (`annotate=run`): a second row of one column under its own
/// address does not give it to the first row's authors.
#[test]
fn aistats_shared_address_stays_with_its_row() {
  let xml = clean(&AISTATS.replace(
    r"\aistatsauthor{Ann Able \And Bob Baker}

\aistatsaddress{Univ A \\ ann@a.edu \And Univ B \\ bob@b.edu}",
    r"\aistatsauthor{Ann Able, Bob Baker}

\aistatsaddress{Univ A}

\aistatsauthor{Cat Cole, Dan Dorn}

\aistatsaddress{Univ C}",
  ));
  let a = r#"<contact name="Affiliation: " role="affiliation">Univ A</contact>"#;
  let c = r#"<contact name="Affiliation: " role="affiliation">Univ C</contact>"#;
  assert_eq!(
    frontmatter(&xml),
    format!(
      r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> {a} </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> {a} </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> {c} </creator> <creator before=" " role="author"> <personname>Dan Dorn</personname> {c} </creator>"#
    ),
    "{xml}"
  );
}

/// The marked branch: `A$^1$ \And B$^2$ \And C$^1$` over one `$^1$X \\ $^2$Y` is a row whose address is not its columns,
/// so each marked line goes to the authors whose marks name it (1910.09056's `$^1$University of British Columbia, …`).
#[test]
fn aistats_marked_address_over_columns_goes_by_mark() {
  let xml = clean(
    &AISTATS
      .replace(
        r"\aistatsauthor{Ann Able \And Bob Baker}",
        r"\aistatsauthor{Ann Able$^1$ \And Bob Baker$^2$ \And Cat Cole$^1$}",
      )
      .replace(
        r"\aistatsaddress{Univ A \\ ann@a.edu \And Univ B \\ bob@b.edu}",
        r"\aistatsaddress{$^1$Univ X \\ $^2$Univ Y}",
      ),
  );
  let x = r#"<contact name="Affiliation: " role="affiliation">Univ X</contact>"#;
  let y = r#"<contact name="Affiliation: " role="affiliation">Univ Y</contact>"#;
  assert_eq!(
    frontmatter(&xml),
    format!(
      r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> {x} </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> {y} </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> {x} </creator>"#
    ),
    "{xml}"
  );
}

/// A later row's shared e-mail list, one address for each author of that row, goes to that row's authors in order —
/// not to the block for not being one for each queued author (2302.10910's four e-mails on an authorless `<creator>`;
/// 1710.05101, 2006.04012); and an e-mail list that is not one for each author is one block contact an address, never
/// the raw line (1802.07182's `\{jrr41,$\,$wct23,…\}@cam.ac.uk`, shown as `{jrr41,$ $wct23,…}`). Repro
/// class-bindings/aistats_shared_email_list_goes_to_its_row.
#[test]
fn aistats_shared_email_list_goes_to_its_row() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aistats_shared_email_list_goes_to_its_row.tex"
  ));
  assert_eq!(
    frontmatter(&xml),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Email: " role="email">ann@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact name="Email: " role="email">bob@b.edu</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">cat@c.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Dan Dorn</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id2">dan@c.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Eve East</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id3">eve@c.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Gil Gray</personname> <contact name="Affiliation: " role="affiliation">Univ G</contact> </creator> <creator before=" " role="author"> <personname>Hal Hunt</personname> <contact name="Affiliation: " role="affiliation">Univ G</contact> </creator> <creator before=" " role="author"> <personname>Ida Irwin</personname> <contact name="Affiliation: " role="affiliation">Univ G</contact> </creator> <creator role="author"> <contact name="Email: " role="email"><text font="typewriter" xml:id="id4">gil@g.edu</text></contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id5">hal@g.edu</text></contact> </creator>"#,
    "{xml}"
  );
}

/// The 2010-2011 templates, aistats2e.sty and aistats2eM.sty, keep a letter after the year's digits, which the versioned
/// fallback does not strip: registered by name, they reach the family binding, and print the names under review too
/// (their copies have no anonymous block; 1003.2751, 1302.5134, 1202.1568). Repro
/// class-bindings/aistats2e_template_reaches_the_binding.
#[test]
fn aistats2e_template_reaches_the_binding() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aistats2e_template_reaches_the_binding.tex"
  );
  let expected = r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#;
  assert_eq!(frontmatter(&clean(tex)), expected);
  let margin = tex.replace("aistats2e", "aistats2eM");
  assert_eq!(frontmatter(&clean(&margin)), expected);
}

/// A class `\@maketitle` printing only a cross-reference prints the label's number (pdflatex "1"), which the
/// post-processor fills in: the deposit is kept with its `<ref>`; one printing only a `\label` prints nothing and goes
/// (77o review probe `m1_depref`). Repro sectioning-frontmatter/deposit_reference_is_ink.
#[test]
fn deposit_reference_is_ink() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/deposit_reference_is_ink.tex"
  );
  let xml = clean(tex);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p align="center" xml:id="p1.1"><ref labelref="LABEL:s1"/></p></para>"#,
  );
  let cited = clean(&tex.replace(r"\ref{s1}", r"\cite{k}").replace(
    r"\end{document}",
    r"\begin{thebibliography}{1}\bibitem{k} A. Author, Title.\end{thebibliography}
\end{document}",
  ));
  assert_element(
    &cited,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p align="center" xml:id="p1.1"><cite class="ltx_citemacro_cite">[<bibref bibrefs="k" separator="," yyseparator=","/>]</cite></p></para>"#,
  );
  let labelled = clean(&tex.replace(r"\ref{s1}", r"\label{t}"));
  let flat = squash(&labelled);
  assert!(
    !flat.contains("<para xml:id=\"p1\"") && flat.contains("<section"),
    "{labelled}"
  );
}

/// The position or alignment argument of a box is a keyword LaTeX only tests — declared `OptionalPosition`/`Position`,
/// not shown — so a deposit holding only `\makebox[0pt][l]`, `\framebox[0pt][r]` and `\shortstack[l]` around struts
/// prints nothing (77o review probes `m1_makebox_l`, `m1_shortstack_l`); the letter is still read where the box prints.
/// Repro sectioning-frontmatter/deposit_box_position_keywords_print_nothing.
#[test]
fn deposit_box_position_keywords_print_nothing() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/deposit_box_position_keywords_print_nothing.tex"
  ));
  let flat = squash(&xml);
  assert!(!flat.contains("<rule"), "{xml}");
  assert!(
    flat
      .find("<section")
      .is_some_and(|section| flat.find("<para").is_none_or(|para| para > section)),
    "{xml}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S1.p1.1\""],
    r#"<p xml:id="S1.p1.1"><text align="left" width="56.9pt" xml:id="S1.p1.1.1">left</text> Body <inline-block align="right" xml:id="S1.p1.1.2"><p xml:id="S1.p1.1.2.1">x</p></inline-block></p>"#,
  );
}

/// The position arguments of tabularx, tabulary, longtable, a `p` column and wrapfig are `OptionalPosition`/`Position`
/// too, and are still read where they print: the tables' `vattach`, the `p` column cell's, the figure's `float` (whole
/// elements; 77o review).
#[test]
fn package_box_positions_are_still_read() {
  let xml = clean(
    r"\documentclass{article}
\usepackage{tabularx,tabulary,longtable,wrapfig}
\begin{document}
\begin{tabularx}{5cm}[t]{X}x\end{tabularx}
\begin{tabulary}{5cm}[b]{L}y\end{tabulary}
\begin{tabular}{p{2cm}}z\end{tabular}
\begin{longtable}[l]{l}w\end{longtable}
\begin{wrapfigure}{r}{3cm}v\end{wrapfigure}
Text.
\end{document}
",
  );
  for (tag, id, element) in [
    (
      "tabular",
      "p1.1",
      r#"<tabular vattach="top" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="left" xml:id="p1.1.1.1"><inline-block vattach="top" xml:id="p1.1.1.1.1"><p xml:id="p1.1.1.1.1.1">x</p></inline-block></td></tr></tbody></tabular>"#,
    ),
    (
      "tabular",
      "p1.2",
      r#"<tabular vattach="bottom" xml:id="p1.2"><tbody><tr xml:id="p1.2.1"><td align="left" xml:id="p1.2.1.1"><inline-block vattach="top" xml:id="p1.2.1.1.1"><p xml:id="p1.2.1.1.1.1">y</p></inline-block></td></tr></tbody></tabular>"#,
    ),
    // (the `p{2cm}` cell: its width and top attachment)
    (
      "tabular",
      "p1.3",
      r#"<tabular vattach="middle" xml:id="p1.3"><tbody><tr xml:id="p1.3.1"><td align="left" vattach="top" xml:id="p1.3.1.1"><inline-block vattach="top" width="56.9pt" xml:id="p1.3.1.1.1"><p xml:id="p1.3.1.1.1.1">z</p></inline-block></td></tr></tbody></tabular>"#,
    ),
    // (`longtable[l]`: the position is read, as Perl's `\@@longtable []` reads it, and not printed)
    (
      "table",
      "S0.T1",
      r#"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><tabular xml:id="S0.T1.1"><tr xml:id="S0.T1.1.1"><td align="left" xml:id="S0.T1.1.1.1">w</td></tr></tabular></table>"#,
    ),
    (
      "figure",
      "fig1",
      r#"<figure float="right" width="24%" xml:id="fig1"><p xml:id="fig1.1">v</p></figure>"#,
    ),
  ] {
    assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
}
