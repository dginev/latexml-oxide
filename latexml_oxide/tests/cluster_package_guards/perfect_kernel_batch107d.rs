//! Red/green guards for batch 107d: the title-page check where no `\maketitle` of ours runs (FIDELITY_CHARTER §4,
//! every residual loss counted by a Warning; OXIDIZED_DESIGN_DIVERGENCES #953 "107d"). A class that prints its title
//! page from a trigger of its own — elsarticle's `\end{frontmatter}` (elsarticle.cls:1303), a `\maketitle` a raw class
//! appends to `\document` (cup-journal.cls:1014) — has its print noted there (sect05.rs `\lx@note@title@page`), only
//! when no `\maketitle` noted one, for the check alone: the XML is the one the base wrote. A class whose title page
//! never prints stays unchecked; the source gate reads the document's own files (no TeX Live file, no `.bbl`'s
//! definitions). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, and asserts the
//! frontmatter whole, the exact Error and Warning counts and the words the check names.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// What a conversion of `tex` gave, eager and `--streaming` alike (asserted equal): its Error and Warning counts, the
/// check's lines (its Warnings and its class-text Infos), and the document's content from its title on, whitespace
/// squashed.
fn run(tex: &str) -> (usize, usize, Vec<String>, String) {
  let mut runs = Vec::new();
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(tex, streaming);
    let lines: Vec<String> = log
      .lines()
      .filter(|line| line.contains("word(s) of the"))
      .map(ToString::to_string)
      .collect();
    let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
    let start = xml.find("<title>").unwrap_or(0);
    let end = xml[start..]
      .find("</document>")
      .map_or(xml.len(), |end| start + end);
    runs.push((
      error_count(&log),
      warning_count(&log),
      lines,
      xml[start..end].trim_end().to_string(),
    ));
  }
  assert_eq!(runs[0], runs[1]);
  runs.swap_remove(0)
}

/// elsarticle prints its title page at `\end{frontmatter}`; the document writes no `\maketitle`. A word its own macro
/// adds to `\@title` after `\title` — 2312.03765's `\subjclass`, a footnote of "2010 Mathematics subject
/// classification. Primary: 26A15; Secondary: 26A21." — is lost from the output, and now counted: one Warning naming
/// the six words (the codes are no content words; "2010" is the macro's default, which the document writes), the
/// frontmatter the base's. Distinct path: a binding's own trigger (`\end{frontmatter}` let to `\lx@note@title@page`).
#[test]
fn elsarticle_frontmatter_end_notes_the_title_page() {
  let (errors, warnings, lines, xml) = run(repro!(
    "check_elsarticle_frontmatter_end_prints_the_title_page"
  ));
  assert_eq!((errors, warnings), (0, 1));
  assert_eq!(lines, [
    "Warning:frontmatter:unclaimed The title page prints 6 word(s) of the document's that its output does not hold: 2010 mathematics subject classification primary secondary",
  ]);
  assert_eq!(
    xml,
    r#"<title>Linear extensions</title> <creator role="author"> <personname>Waldemar Sieg</personname> <contact name="Address: " role="address">Kazimierz Wielki University, Bydgoszcz, Poland</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Let X be a space.</p> </abstract> <section inlist="toc" xml:id="S1"> <tags> <tag>1</tag> <tag role="autoref">section 1<text xml:id="S1.1"/></tag> <tag role="refnum">1</tag> <tag role="typerefnum">§1</tag> </tags> <title><tag close=" ">1</tag>Intro</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Text.</p> </para> </section>"#
  );
}

/// elsarticle's `\maketitle` written inside `{frontmatter}`: its print (the deposit's) is the one checked, and the
/// binding's `\end{frontmatter}` after it notes no second print of the fields our `\maketitle` emptied — the words the
/// document's macro added to `\@title` are counted once. Distinct path: the trigger notes only when no print was noted
/// (`title_page_check::title_page_noted`).
#[test]
fn a_maketitle_inside_frontmatter_keeps_its_print() {
  let (errors, warnings, lines, xml) = run(repro!(
    "check_elsarticle_maketitle_inside_frontmatter_keeps_its_print"
  ));
  assert_eq!((errors, warnings), (0, 1));
  assert_eq!(lines, [
    "Warning:frontmatter:unclaimed The title page prints 5 word(s) of the document's that its output does not hold: 2010 mathematics subject classification primary",
  ]);
  assert_eq!(
    xml,
    r#"<title>Linear extensions</title> <creator role="author"> <personname>Waldemar Sieg</personname> </creator> <section inlist="toc" xml:id="S1"> <tags> <tag>1</tag> <tag role="autoref">section 1<text xml:id="S1.1"/></tag> <tag role="refnum">1</tag> <tag role="typerefnum">§1</tag> </tags> <title><tag close=" ">1</tag>Intro</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Text.</p> </para> </section>"#
  );
}

/// A raw class appends `\maketitle` to `\document` (cup-journal.cls:1014), which our constructor refuses as Perl does
/// (`Warning:unexpected:\document`): the class's title page prints at `\begin{document}`, its print is noted there, and
/// the store only it prints (`\corr`'s address) is counted — "lunar org" in a Warning ("ann", "able" are the creator's),
/// the class's label "Corresponding" an Info; the frontmatter unchanged. Distinct path: a raw class's trigger, read
/// off the refused append. Witnesses 2207.13252, 2603.00371, 2302.11911, 2508.08363, 2607.18758.
#[test]
fn a_title_page_appended_to_document_is_counted() {
  let (errors, warnings, lines, xml) =
    run(repro!("check_title_page_from_begin_document_is_counted"));
  assert_eq!((errors, warnings), (0, 2));
  assert_eq!(lines, [
    "Warning:frontmatter:unclaimed The title page prints 2 word(s) of the document's that its output does not hold: lunar org",
    "Info:frontmatter:class_text The title page prints 1 word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: corresponding",
  ]);
  assert_eq!(
    xml,
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// The same raw class with no `\maketitle` and no trigger of its own prints no title page (pdflatex: "Text." alone):
/// nothing is checked, no Warning and no Info, though its store is in no output. Negative: the print is noted only
/// where the class prints, never at the frontmatter's placement.
#[test]
fn no_title_page_printed_is_unchecked() {
  let (errors, warnings, lines, xml) = run(repro!("check_no_title_page_printed_is_unchecked"));
  assert_eq!((errors, warnings), (0, 0));
  assert_eq!(lines, Vec::<String>::new());
  assert_eq!(
    xml,
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// The check's source gate reads the document's own text: a file a package reads as content from the TeX tree
/// (translations' TeX Live dictionary, which writes "Contents") and a `.bbl`'s definitions (its BibTeX style's
/// `\providecommand{\URLprefix}{URL: }`) are not it. The class's labels lost with its flushed note — "Contents lists
/// available at ScienceDirect. URL: none" — are its own text, an Info, and no Warning (int139a1 named "contents url").
/// Distinct path: `title_page_check::document_source_words`. Witnesses medima.sty 2507.07920, 2201.10981, 2409.09188.
#[test]
fn package_and_bibtex_text_is_not_the_documents() {
  let (errors, warnings, lines, xml) =
    run(repro!("check_package_and_bibtex_text_is_not_the_documents"));
  assert_eq!((errors, warnings), (0, 0));
  assert_eq!(lines, [
    "Info:frontmatter:class_text The title page prints 6 word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: contents lists available sciencedirect url none",
  ]);
  assert_eq!(
    xml,
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text <cite class="ltx_citemacro_cite">[<bibref bibrefs="able" separator="," yyseparator=","/>]</cite>.</p> </para> <bibliography inlist="toc" xml:id="bib"> <title>References</title> <biblist> <bibitem key="able" xml:id="bib.bib1"> <tags> <tag>[1]</tag> <tag role="refnum">1</tag> </tags> <bibblock> A. Able. A book. </bibblock> </bibitem> </biblist> </bibliography>"#
  );
}
