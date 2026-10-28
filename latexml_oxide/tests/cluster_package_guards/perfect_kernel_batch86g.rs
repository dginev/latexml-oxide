//! Red/green guards for batch 86g: an abstract placed ahead of title-page text pdflatex prints before it. The
//! frontmatter block `\maketitle` places already holds an abstract given after it (or, without `\maketitle`, the
//! fallback places it at the top), so a raw class's or style's own `\@maketitle` (the deposit: AAAI's `\affiliations`),
//! lines hand-typeset between `\maketitle` and the abstract, and setters that print in place followed it as body text.
//! The cover between the placed frontmatter and the abstract's source position is now wrapped as `<titlepage>`
//! (`ltx_titlepage_continued`), the abstract behind it; what is not cover — a float, a TOC, body prose, a body list —
//! keeps the abstract where it was (OXIDIZED_DESIGN_DIVERGENCES #903, extending #246). Witnesses: 2011.04405,
//! 2102.01904, 2109.06762, 2203.09121, 2208.05119, 2309.13734, 2412.11664 (aaai*), 2304.00893 (nature), 0905.1102
//! (dmtcs), 1404.2263 (pnastwo), 1609.06335 (bmc_article), 1908.08044 (INTERSPEECH), 1302.5352, 1702.07128,
//! 2309.14687 (hand-typeset), 1207.1161, 1303.3004, astro-ph0210637 (no `\maketitle`). Each guard converts its repro
//! under `tools/perfect_kernel/repros/sectioning-frontmatter/`, asserts no Error or Warning and the document's root
//! order, the titlepage's blocks by their text. One guard per path: the hand-typeset cover (and the variants on its
//! path), the deposit's provenance, an abstract inside the deposit, the fallback without `\maketitle`, what is no
//! cover, and the HTML's title beside a continued titlepage. The streamed build's guard is
//! `114_streaming_structure::streaming_late_abstract_follows_its_cover`.
use libxml::{parser::Parser, tree::NodeType};

use super::perfect_kernel_batch46::{convert_html, convert_with, error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// The document's root children after its resources, each as its name — a titlepage as `titlepage[<child>: <text>|…]`,
/// its children's text whitespace-collapsed.
fn root_order(xml: &str) -> Vec<String> {
  let doc = Parser::default()
    .parse_string(xml)
    .expect("the output parses");
  let root = doc.get_root_element().expect("a root");
  let text = |node: &libxml::tree::Node| {
    node
      .get_content()
      .split_whitespace()
      .collect::<Vec<_>>()
      .join(" ")
  };
  root
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode) && n.get_name() != "resource")
    .map(|n| {
      if n.get_name() == "titlepage" {
        let blocks: Vec<String> = n
          .get_child_nodes()
          .into_iter()
          .filter(|c| c.get_type() == Some(NodeType::ElementNode))
          .map(|c| format!("{}: {}", c.get_name(), text(&c)))
          .collect();
        format!("titlepage[{}]", blocks.join("|"))
      } else {
        n.get_name()
      }
    })
    .collect()
}

/// Converts `tex`, checks it converts clean, and compares its root order; returns the XML.
#[track_caller]
fn assert_root_order(tex: &str, expected: &[&str]) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(root_order(&xml), expected, "{xml}");
  assert!(
    !xml.contains("_Capture_"),
    "no marker reaches the output:\n{xml}"
  );
  xml
}

/// The hand-typeset repro with `lines` in place of what it sets between `\maketitle` and its first section (the cover
/// and the abstract).
fn with_cover(lines: &str) -> String {
  let tex = repro!("late_abstract_after_hand_typeset_cover.tex");
  let start = tex.find("\\begin{center}").expect("the cover lines");
  let end = tex.find("\\section").expect("the section");
  format!("{}{lines}\n{}", &tex[..start], &tex[end..])
}

/// Lines hand-typeset between `\maketitle` and a late abstract (1302.5352, 1702.07128, 2309.14687): a centred
/// affiliation and a `\noindent` keywords line are the cover, the abstract follows it. Was `title, creator, abstract,
/// para, para, section` (the cover after the abstract, out of print order). Variants on the same path: a `\noindent`
/// line running into `\begin{abstract}` (the marker inside the open paragraph); a `\clearpage` after the cover (the
/// pagebreak follows the abstract); a quoted epigraph or dedication (a title-page line too); two abstracts after
/// `\maketitle` (the second is kept, at the first's place: `{abstract}` given twice keeps the last, the known loss of
/// OXIDIZED_DESIGN #154 — pdflatex prints both); the `\abstract … \endabstract` command form; amsart's preamble
/// `\keywords`, which it prints at the page foot, follows the abstract behind the cover (as dmtcs's 0905.1102 and
/// aaspp4's astro-ph9911463 do). What the title page printed before the cover keeps its place ahead of it, only the
/// rest of the late abstract's run follows the cover: an abstract the frontmatter had before the late one (not
/// `_late`) — smfart's `{abstract}` before `\maketitle` and `{altabstract}` after it, a class's abstract store its
/// `\@maketitle` prints (86g r1 review probes q01, q02; the first moved behind the cover too). The fail-safe: a late
/// run holding an entry handed on from a raw class's store its title page may print (title-page provenance,
/// `_titlepage`) is not moved at all, int116a's order — the class's title page prints it where the frontmatter cannot
/// tell: above the cover (its stored keywords, its classification store handed on as it is set, an off-table store
/// `\maketitle` hands on as a note; 86g r2 review probe b01 and its siblings, which followed the cover and the
/// abstract, behind them in print) or at the page foot (`\footnotetext{Keywords: \@keywords}`,
/// `\footnotetext{PACS: \@pacs}`; 86g r3 review probes f1, f2, which a page-foot store's provenance put ahead of the
/// cover).
#[test]
fn late_abstract_follows_the_hand_typeset_cover() {
  assert_root_order(repro!("late_abstract_after_hand_typeset_cover.tex"), &[
    "title",
    "creator",
    "titlepage[block: Department of Repro, University of Witness|block: Keywords: order, print]",
    "abstract",
    "section",
  ]);
  let center = "\\begin{center}University of Witness\\end{center}\n";
  let abstract_ = "\\begin{abstract}Abstract words here.\\end{abstract}";
  let affiliation = "titlepage[block: University of Witness]";
  let variants: [(String, &[&str]); 6] = [
    (format!("\\noindent University of Witness\n{abstract_}"), &[
      "title",
      "creator",
      affiliation,
      "abstract",
      "section",
    ]),
    (format!("{center}\\clearpage\n{abstract_}"), &[
      "title",
      "creator",
      affiliation,
      "abstract",
      "pagination",
      "section",
    ]),
    (
      format!("\\begin{{quote}}\\emph{{To my mother.}}\\end{{quote}}\n{abstract_}"),
      &[
        "title",
        "creator",
        "titlepage[block: To my mother.]",
        "abstract",
        "section",
      ],
    ),
    (
      format!("{center}\\begin{{abstract}}First abstract.\\end{{abstract}}\n{abstract_}"),
      &["title", "creator", affiliation, "abstract", "section"],
    ),
    (
      format!("{center}\\abstract Abstract words here.\\endabstract"),
      &["title", "creator", affiliation, "abstract", "section"],
    ),
    (
      with_cover(&format!("{center}{abstract_}")).replace(
        "\\documentclass{article}\n\\title{Repro Title}\\author{Ann Author}",
        "\\documentclass{amsart}\n\\title{Repro Title}\\author{Ann Author}\\keywords{order, print}",
      ),
      &[
        "title",
        "creator",
        affiliation,
        "abstract",
        "keywords",
        "section",
      ],
    ),
  ];
  for (lines, expected) in variants {
    let tex = if lines.contains("\\documentclass") {
      lines
    } else {
      with_cover(&lines)
    };
    let xml = assert_root_order(&tex, expected);
    assert!(xml.contains("Abstract words here."), "{xml}");
  }
  let earlier: [(&str, &[&str], &[&str]); 2] = [
    (
      repro!("late_abstract_after_an_earlier_abstract.tex"),
      &[
        "title",
        "creator",
        "abstract",
        "titlepage[block: par|block: Department of Repro, University of Witness]",
        "abstract",
        "pagination",
        "para",
        "pagination",
        "section",
      ],
      &[
        "Main abstract words.",
        "Department of Repro",
        "Alternative abstract words.",
      ],
    ),
    (
      repro!("late_abstract_after_a_class_store_abstract.tex"),
      &[
        "title",
        "creator",
        "abstract",
        "titlepage[block: Department of Repro, University of Witness]",
        "abstract",
        "section",
      ],
      &[
        "Spanish abstract words.",
        "Department of Repro",
        "English abstract words.",
      ],
    ),
  ];
  for (tex, expected, print_order) in earlier {
    let xml = assert_root_order(tex, expected);
    let at: Vec<usize> = print_order
      .iter()
      .map(|text| xml.find(text).unwrap_or_else(|| panic!("{text}:\n{xml}")))
      .collect();
    assert!(at.is_sorted(), "{print_order:?} in print order:\n{xml}");
  }
  // The fail-safe: the class's stores' entries, printed above the cover or at the page foot, and the late abstract
  // keep int116a's slot, the cover a paragraph after them.
  let printed = repro!("late_abstract_after_class_printed_frontmatter.tex");
  let lines = "\\noindent Keywords: \\@keywords\\par\n\\noindent\\@pacs\\par}";
  assert!(printed.contains(lines), "the class's title-page lines");
  let stored: &[&str] = &[
    "title",
    "creator",
    "abstract",
    "keywords",
    "classification",
    "para",
    "section",
  ];
  let fail_safe: [(String, &[&str], &str); 4] = [
    (printed.to_string(), stored, "Keywords:"),
    (
      printed.replace(
        lines,
        "{\\let\\thefootnote\\relax\\footnotetext{Keywords: \\@keywords}}\n\\noindent\\@pacs\\par}",
      ),
      stored,
      "Keywords:",
    ),
    (
      printed.replace(
        lines,
        "\\noindent Keywords: \\@keywords\\par\n{\\let\\thefootnote\\relax\\footnotetext{PACS: \\@pacs}}}",
      ),
      stored,
      "PACS:",
    ),
    (
      repro!("late_abstract_after_a_class_note_store.tex").to_string(),
      &["title", "creator", "abstract", "pubnote", "para", "section"],
      "Journal of Repro 12 (2026)",
    ),
  ];
  for (tex, expected, kept) in fail_safe {
    let xml = assert_root_order(&tex, expected);
    assert!(!xml.contains("ltx_titlepage_continued"), "{xml}");
    for text in [kept, "Department of Repro", "Abstract words here."] {
      assert_eq!(xml.matches(text).count(), 1, "{text} once:\n{xml}");
    }
    // (the stores' own content, kept whole and once — an emptied entry must not pass; the 86g r4 re-review's nit 3)
    if expected.contains(&"keywords") {
      for text in ["order, print", "12.34.Ab"] {
        assert_eq!(xml.matches(text).count(), 1, "{text} once:\n{xml}");
      }
    }
  }
}

/// A style's own `\@maketitle`, replayed by the title-page deposit (AAAI's `\affiliations`, aaai21.sty:58; 2011.04405,
/// 2102.01904 and the other aaai witnesses; dmtcs 0905.1102, bmc_article 1609.06335, INTERSPEECH 1908.08044): its
/// paragraphs are the late abstract's cover by provenance, a plain indented line too (pnastwo's `\@contributor`,
/// 1404.2263), and a footnote's text and what its `\maketitle` inserts at the page foot (AAAI's copyright slug,
/// `\insert\footins{…}`, aaai21.sty:53) follow the abstract. Was `title, creator, abstract, para, para, note, para,
/// section`. Variants: a title page whose only ink is the inserted slug (the AAAI-19 papers 1909.05089, 1904.09412,
/// whose affiliations are in `\author`) leaves the abstract where it was; an `\insert` is an internal vertical list of
/// its own (tex.web §1099-1100), set after the title page, so one inside a centred line leaves the line whole and one
/// before a `\noindent` line does not run into it (86g r1 review probes s01, s02: the slug was wrapped into the
/// titlepage, and the line after it followed the abstract with it); material opened by `\bgroup` (revtex4*, aastex701,
/// memoir) is read live to its group's end and set after the title page the same way, after a braced one before it
/// (86g r2 review probe c03: it was wrapped into the titlepage, run into the `\noindent` line after it). A raw class's
/// stored keywords its `\@maketitle` prints in an `\insert`, braced or `\bgroup`-opened: the title page's
/// speculative run reads the inserted material as printed, so the label is the entry's name and is taken out with
/// the store — one "Keywords:" (86g r3 review probes f3, i3: an orphan "Keywords:" paragraph beside the entry, the
/// `\bgroup` form in int116a too) — and the run keeps int116a's slot (the fail-safe of
/// `late_abstract_follows_the_hand_typeset_cover`).
#[test]
fn late_abstract_follows_the_class_title_page() {
  let tex = repro!("late_abstract_after_the_class_title_page.tex");
  assert_root_order(tex, &[
    "title",
    "creator",
    "titlepage[block: University of Witness|block: Submitted to the Repro Journal of Witnesses]",
    "abstract",
    "note",
    "para",
    "section",
  ]);
  let start = tex.find("\\def\\@maketitle{").expect("the title page");
  let end = start + tex[start..].find("\\makeatother").expect("its end");
  let slug_only = format!("{}\\def\\@maketitle{{}}\n{}", &tex[..start], &tex[end..]);
  let xml = assert_root_order(&slug_only, &[
    "title", "creator", "abstract", "para", "section",
  ]);
  assert!(xml.contains("Copyright 2026, the Repro Society."), "{xml}");
  let inserts: [(&str, &str, &[&str]); 3] = [
    (
      concat!(
        r"\def\@maketitle{{\LARGE\@title\par}{\large\@author\par}%",
        "\n",
        r"  \begin{center}University of Witness\insert\footins{\noindent\footnotesize Licensed 2026.} and Institute",
        r" of Repro\end{center}}",
        "\n",
      ),
      "titlepage[block: University of Witness and Institute of Repro]",
      &["Licensed 2026."],
    ),
    (
      concat!(
        r"\def\@maketitle{{\LARGE\@title\par}{\large\@author\par}{\affiliations_\par}%",
        "\n",
        r"  \insert\footins{\noindent\footnotesize Licensed 2026.}",
        "\n",
        r"  \noindent Submitted to the Repro Journal of Witnesses\par}",
        "\n",
      ),
      "titlepage[block: University of Witness|block: Submitted to the Repro Journal of Witnesses]",
      &["Licensed 2026."],
    ),
    (
      concat!(
        r"\def\@maketitle{{\LARGE\@title\par}{\large\@author\par}{\affiliations_\par}%",
        "\n",
        r"  \insert\footins{\footnotesize Licensed 2026.}",
        "\n",
        r"  \insert\footins\bgroup\footnotesize Bgroup slug.\egroup",
        "\n",
        r"  \noindent Submitted to the Repro Journal of Witnesses\par}",
        "\n",
      ),
      "titlepage[block: University of Witness|block: Submitted to the Repro Journal of Witnesses]",
      &["Licensed 2026.", "Bgroup slug."],
    ),
  ];
  for (maketitle, titlepage, slugs) in inserts {
    let tex = format!("{}{maketitle}{}", &tex[..start], &tex[end..]);
    let mut expected = vec!["title", "creator", titlepage, "abstract"];
    expected.extend(std::iter::repeat_n("para", slugs.len() + 1));
    expected.push("section");
    let xml = assert_root_order(&tex, &expected);
    let mut at = vec![xml.find("Abstract words here.").expect("the abstract")];
    at.extend(slugs.iter().map(|slug| xml.find(slug).expect("the slug")));
    assert!(at.is_sorted(), "the abstract, then {slugs:?}:\n{xml}");
  }
  let printed = repro!("late_abstract_after_class_printed_frontmatter.tex");
  let lines = "\\noindent Keywords: \\@keywords\\par\n";
  assert!(printed.contains(lines), "the class's keywords line");
  for insert in [
    "\\insert\\footins{\\footnotesize\\noindent Keywords: \\@keywords\\par}\n",
    "\\insert\\footins\\bgroup\\footnotesize\\noindent Keywords: \\@keywords\\par\\egroup\n",
  ] {
    let xml = assert_root_order(&printed.replace(lines, insert), &[
      "title",
      "creator",
      "abstract",
      "keywords",
      "classification",
      "para",
      "section",
    ]);
    assert!(
      xml.contains(r#"<keywords name="Keywords:">order, print</keywords>"#),
      "{xml}"
    );
    assert_eq!(xml.matches("Keywords:").count(), 1, "one label:\n{xml}");
  }
}

/// A class's `\@maketitle` that prints the abstract itself, between the affiliations and a "Received" line: the
/// abstract is placed after what the deposit printed before it, and the line after it stays after it. Was `title,
/// creator, abstract, para, para, section`.
#[test]
fn abstract_inside_the_class_title_page() {
  let xml = assert_root_order(repro!("abstract_inside_the_class_title_page.tex"), &[
    "title",
    "creator",
    "titlepage[block: University of Witness]",
    "abstract",
    "para",
    "section",
  ]);
  let abstract_at = xml.find("Abstract words here.").expect("the abstract");
  assert!(
    abstract_at < xml.find("Received 1 January 2026").expect("the line"),
    "{xml}"
  );
}

/// No `\maketitle`: a setter that prints in place before the abstract (newpasp's `\affil`, astro-ph0210637; arxiv.cls's
/// `\affiliation`, 1207.1161; ae100prg's `\address`, 1303.3004) is wrapped by the fallback flush, the abstract after it,
/// the line after the abstract after that. Was `title, creator, abstract, para, para, section`.
#[test]
fn abstract_without_maketitle_follows_the_in_place_setters() {
  assert_root_order(
    repro!("abstract_after_in_place_setters_without_maketitle.tex"),
    &[
      "title",
      "creator",
      "titlepage[block: University of Witness]",
      "abstract",
      "para",
      "section",
    ],
  );
}

/// What is no cover keeps the late abstract after the creators, today's place: a float (the repro), a
/// `\tableofcontents`, body prose (lips' `\DocInput`-ed abstract after body paragraphs,
/// `perfect_kernel_batch56::late_abstract_after_maketitle_leaves_no_marker_and_wraps_nothing`), a body list, and a
/// cover line followed by body prose.
#[test]
fn late_abstract_after_body_content_keeps_its_slot() {
  assert_root_order(
    repro!("late_abstract_after_body_content_keeps_its_slot.tex"),
    &["title", "creator", "abstract", "figure", "section"],
  );
  let variants: [(&str, &[&str]); 4] = [
    ("\\tableofcontents", &[
      "title", "creator", "abstract", "TOC", "section",
    ]),
    ("Some body prose before the abstract.\n", &[
      "title", "creator", "abstract", "para", "section",
    ]),
    (
      "\\begin{itemize}\\item We prove the first claim.\\item We prove the second claim.\\end{itemize}",
      &["title", "creator", "abstract", "para", "section"],
    ),
    (
      "\\begin{center}University of Witness\\end{center}\nSome body prose before the abstract.\n",
      &["title", "creator", "abstract", "para", "para", "section"],
    ),
  ];
  for (cover, expected) in variants {
    assert_root_order(
      &with_cover(&format!(
        "{cover}\n\\begin{{abstract}}Abstract words here.\\end{{abstract}}"
      )),
      expected,
    );
  }
}

/// The HTML shows the document's title beside a titlepage that continues the title block (`ltx_titlepage_continued`):
/// a titlepage otherwise stands for the title, which LaTeXML-structure-xhtml.xsl then does not display, and the
/// AAAI papers' `<h1>` was gone with the wrap.
#[test]
fn continued_titlepage_keeps_the_title_displayed() {
  let (log, html) = convert_html(repro!("late_abstract_after_hand_typeset_cover.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    html.contains("<h1 class=\"ltx_title ltx_title_document\">Repro Title</h1>"),
    "{html}"
  );
  assert!(html.contains("ltx_titlepage_continued"), "{html}");
}
