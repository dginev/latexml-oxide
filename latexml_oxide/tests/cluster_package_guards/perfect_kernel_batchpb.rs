//! Guards for PB, the two REDs the int154a push carried. One guard per path: the decline split's fail-safe on a name no
//! creator of Perl's split holds (B1, 1511.09009; `emit::split_owns_falsely`, OXIDIZED_DESIGN_DIVERGENCES #988), and two
//! paths of OD #983's exclusion (the title code's definitions in force for the author content,
//! `title_code_author_definitions`): a macro of the title code that reads `\@author` (aaai2026's `\theauthors`), and a
//! tabular juggle the title code defines, read by its meaning as a separator (neurips_2022's `\NewAnd`). PBb (review revPB):
//! a name a macro spells is no lost name (S1), and a copy's title code, its juggle between unmarked names a column (S2, S4).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Convert `tex`: the log and the XML.
fn run(tex: &str) -> (String, String) { run_with(tex, &[]) }

/// Convert `tex` beside the source files `files`: the log and the XML.
fn run_with(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let (log, xml, ()) =
    convert_files_with_setup_then(tex, files, Some("ar5iv.sty"), move || {}, |_| ());
  (log, xml)
}

/// Every `<creator …>…</creator>` element of `xml`, whole, in document order, the indentation between its tags out.
fn creator_elements(xml: &str) -> Vec<String> {
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end =
        start + xml[start..].find("</creator>").expect("closed creator") + "</creator>".len();
      xml[start..end].lines().map(str::trim).collect::<String>()
    })
    .collect()
}

/// B1 (1511.09009): `Ann Able\footremember{aa} \\, Bob Baker\footremember{bb} \\, …`, each `\\` the macro's second
/// argument, so pdflatex prints one line "Ann Able∗, Bob Baker†, Carl Cole‡". The definition decline's split cuts the
/// names line at the first `\\` (the kernel's comma cut of a names line only, OD #52): Bob and Carl are in no creator's
/// names, and int155a0 lost Carl (counted). Written whole, as before the split: one creator holding every name and note.
/// FN1 (OD #1001; the user's ruling 2026-11-11, which accepts this shape): a block written whole holds its notes
/// after the joint name, each with its mark, so the name reads without their tags; pin updated (the notes' move only).
#[test]
fn a_split_losing_a_name_keeps_the_block_whole() {
  let (log, xml) = run(repro!("pb_split_loses_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("the block is written whole, as Perl's whole-block creator"),
    "{log}"
  );
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able, Bob Baker, Carl Cole <break/> Univ X <break/> Univ Y</personname><note mark=\"*\" role=\"footnotemark\" xml:id=\"footnotex1\"><tags><tag>*</tag><tag role=\"refnum\">*</tag><tag role=\"typerefnum\">footnote *</tag></tags></note><note mark=\"†\" role=\"footnotemark\" xml:id=\"footnotex2\"><tags><tag>†</tag><tag role=\"refnum\">†</tag><tag role=\"typerefnum\">footnote †</tag></tags></note><note mark=\"‡\" role=\"footnotemark\" xml:id=\"footnotex3\"><tags><tag>‡</tag><tag role=\"refnum\">‡</tag><tag role=\"typerefnum\">footnote ‡</tag></tags></note></creator>"
    ],
    "{xml}"
  );
}

/// B2(a) (aaai2026.sty:73-98; 2508.00545, 2508.04047, 2508.04749, 2508.05996, 2508.08170): the title code reads
/// `\@author` only through `\def\theauthors{\@author}`, used inside the group that defines `\coo`. Its first use is the
/// reading point, so `\coo` is in force for the author content: no `Error:undefined:\coo` (pdflatex 0), Ann's † kept.
#[test]
fn a_title_code_macro_reading_the_author_defines_its_marks() {
  let (log, xml) = run(repro!("pb_title_code_macro_reads_author"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\">†</sup></personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>"
    ],
    "{xml}"
  );
}

/// B2(b) (neurips_2022.sty:285, 2205.14141): `\NewAnd`, a juggle `\end{tabular}…\begin{tabular}[t]{c}` the title code
/// defines beside `\@author`, is a separator read by its meaning (`title_code_juggle`), as `\sharedaffiliation` is, and
/// never replayed (digested, it closes the tabular): three creators, no `Error:undefined:\NewAnd` (pdflatex 0).
#[test]
fn a_title_code_juggle_parts_two_authors() {
  let (log, xml) = run(repro!("pb_title_code_juggle_separator"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Carl Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact></creator>"
    ],
    "{xml}"
  );
}

/// PBb S1 (review revPB, `probes/a_macro_name.tex`): `\cat$^{1}$` of `\newcommand{\cat}{Cat Cole}` is a piece of Perl's
/// split printing no word, so the lost-name fail-safe read "Cat Cole" as lost and wrote the block whole (PB: one creator).
/// A piece printing no word is the name a macro spells (`emit::perl_piece_of`): no loss, and its marks are Cat's.
#[test]
fn a_name_a_macro_spells_is_no_lost_name() {
  let (log, xml) = run(repro!("pbb_macro_spelled_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X, Dept Y</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Note:\u{a0}\" role=\"note\">Zebra note.</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X, Dept Y</contact></creator>"
    ],
    "{xml}"
  );
}

/// PBb S2 + S4 (review revPB; 2205.14141, 2205.14539): the title code is a stand-in copy's (`state::copy_kept_meaning`, the
/// path no guard took: the neurips binding keeps its own `\@maketitle`), and its `\NewAnd` juggle stands between names with
/// no marks. Its `\hfil\linebreak[0]\hfil` forces no break (latex.ltx:9235-9270), so it parts authors as `\and` does: three
/// creators, as the PDF's three columns (PB made "Carl Cole" Bob's affiliation, silently).
#[test]
fn a_copys_title_code_juggle_parts_unmarked_names() {
  const COPY: &str = "\\ProvidesPackage{neurips_2022}\n\\renewcommand{\\@maketitle}{\\def\\NewAnd{\\end{tabular}\\hfil\\linebreak[0]\\hfil\\begin{tabular}[t]{c}\\ignorespaces}%\n  \\begin{tabular}[t]{c}\\@author\\end{tabular}}\n";
  let (log, xml) = run_with(repro!("pbb_copy_title_code_juggle_unmarked"), &[(
    "neurips_2022.sty",
    COPY,
  )]);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("\\@maketitle keeps LaTeXML's definition"),
    "the copy's \\@maketitle is read beside the binding's: {log}"
  );
  assert_eq!(
    creator_elements(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Carl Cole</personname></creator>"
    ],
    "{xml}"
  );
}
