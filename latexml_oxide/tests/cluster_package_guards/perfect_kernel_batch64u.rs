//! Red/green guards for perfect-kernel batch 64u: journal styles that are neither in TeX Live nor shipped with the
//! papers using them — A&A's LaTeX 2.09 `l-aa`/`laa`, EPL's `epl2`/`epl`, the OSA `opex3`/`ol2`/`osajnl`/`osajnl2`
//! styles and REVTeX 2.1's frontmatter environments — whose author content and frontmatter were lost to undefined
//! commands.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// Every creator of `xml`, whole and in document order.
fn creators(xml: &str) -> Vec<String> {
  let mut creators = Vec::new();
  let mut rest = xml;
  while let Some(i) = rest.find("<creator ") {
    let end = i
      + rest[i..]
        .find("</creator>")
        .map_or(0, |j| j + "</creator>".len());
    creators.push(rest[i..end].to_string());
    rest = &rest[end..];
  }
  creators
}

/// 64u: `\documentstyle{l-aa}` is the `aa` class with L-AA's thesaurus codes as the paper's classification
/// (l-aa.sty:453-454, :502-516); its `\offprints`, `\la`, `\degr` are the A&A binding's (astro-ph9804097,
/// astro-ph9405007). Repro sectioning-frontmatter/l_aa_documentstyle_is_the_aa_class.
#[test]
fn l_aa_documentstyle_is_the_aa_class() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/l_aa_documentstyle_is_the_aa_class.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>J. Greiner</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Max-Planck-Institut für extraterrestrische Physik, Garching</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>R. Egger</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Universität Bonn</contact>\n  </creator>",
  ]);
  assert_element(
    &xml,
    "classification",
    &[],
    "<classification name=\"Thesaurus codes: \" scheme=\"thesaurus\">08(09.19.2; 13.25.4)</classification>",
  );
  assert_element(
    &xml,
    "date",
    &[],
    "<date role=\"creation\">Received February 3, 1994; accepted May 6, 1994</date>",
  );
  assert_element(
    &xml,
    "pubnote",
    &[],
    "<pubnote name=\"Preprint: \" role=\"preprint\">J. Greiner</pubnote>",
  );
  assert_eq!(xml.matches("<keywords ").count(), 1, "{xml}");
  assert_eq!(xml.matches("<acknowledgements ").count(), 1, "{xml}");
}

/// 64u: l-aa's `\thesaurus` is a `\def`, not `\long`: a `%`-commented closing brace ends its argument at the blank
/// line, with pdflatex's "Paragraph ended before \thesaurus was complete", and the document goes on (astro-ph9405007,
/// whose title, sections and bibliography were otherwise read as the codes).
/// Repro sectioning-frontmatter/l_aa_thesaurus_ends_at_par.
#[test]
fn l_aa_thesaurus_ends_at_par() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/l_aa_thesaurus_ends_at_par.tex"
  );
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Paragraph ended before \\thesaurus was complete"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(&xml, "title", &[], "<title>A young remnant</title>");
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>J. Greiner</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">MPE Garching</contact>\n  </creator>",
  ]);
  assert!(!xml.contains("<classification"), "{xml}");
  assert!(
    xml.contains("<title><tag close=\" \">1</tag>Intro</title>"),
    "{xml}"
  );
}

/// 64u: the EPL class (epl2.cls): each author with the institute its `\inst` mark names, an entry's own `\\` no new
/// entry (1803.06254); `\pacs{code}{description}` whole (:693-705); the bibliography macros as the class sets them
/// (:1347-1377; 0911.3858, 1904.03227, 1602.05654); unnumbered sections (:870).
/// Repro sectioning-frontmatter/epl_class_frontmatter_and_bibliography.
#[test]
fn epl_class_frontmatter_and_bibliography() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/epl_class_frontmatter_and_bibliography.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">School of Physics, Henan University,<break/>Luoyang, China</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Department of Physics, Nanjing University, Nanjing, China</contact>\n  </creator>",
  ]);
  assert_eq!(xml.matches("<classification ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "classification",
    &[],
    "<classification name=\"PACS \" scheme=\"pacs\">73.43.-f – Quantum Hall effects</classification>",
  );
  assert!(
    xml.contains(
      "<classification name=\"PACS \" scheme=\"pacs\">74.45.+c – Proximity effects</classification>"
    ),
    "{xml}"
  );
  assert!(xml.contains("<title>Introduction</title>"), "{xml}");
  assert_element(
    &xml,
    "bibblock",
    &[],
    "<bibblock> <text font=\"smallcaps\" xml:id=\"bib.bib1.1\">A. Kitaev</text>, <text font=\"italic\" xml:id=\"bib.bib1.2\">Phys. Usp.</text>, <text font=\"bold\" xml:id=\"bib.bib1.3\">44</text> (2001) 131.\n</bibblock>",
  );
  assert!(
    xml.contains("<bibblock> <text font=\"smallcaps\" xml:id=\"bib.bib2.1\">F. Wilczek</text>, <text font=\"italic\" xml:id=\"bib.bib2.2\">Fractional Statistics</text> (World Scientific, Singapore) 1990.\n</bibblock>"),
    "{xml}"
  );
  assert_eq!(xml.matches("<acknowledgements ").count(), 1, "{xml}");
}

/// 64u: the OSA journal styles (opex3.sty): the `$^n$`-marked `\address` lines are the affiliations of the names
/// showing the mark, the `*`-marked email line the email of the name marked `$^{1,2*}$` (marks 1, 2 and *), the OCIS
/// codes a classification, the journal abbreviations defined (1006.2188, 0901.2165).
/// Repro sectioning-frontmatter/osa_style_marked_addresses.
#[test]
fn osa_style_marked_addresses() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/osa_style_marked_addresses.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "title",
    &[],
    "<title>Pupil remapping for high contrast astronomy</title>",
  );
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>T. Kotani</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">LESIA, Observatoire de Paris, Meudon, France</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>S. Lacour</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">LESIA, Observatoire de Paris, Meudon, France</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">School of Physics, Sydney University, NSW 2006, Australia</contact>\n    <contact name=\"Corresponding author: \" role=\"email\">sylvestre.lacour@obspm.fr</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>P. Tuthill</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">School of Physics, Sydney University, NSW 2006, Australia</contact>\n  </creator>",
  ]);
  assert_element(
    &xml,
    "classification",
    &[],
    "<classification name=\"OCIS codes: \" scheme=\"ocis\">(350.1260) Astronomical optics; (060.2430) Fibers, single-mode.</classification>",
  );
  assert!(xml.contains("A. Author, Opt. Lett. "), "{xml}");
}

/// 64u: OSA's `\author` typesets where it stands, so each call adds its authors, and an `\address` after it is theirs
/// (0906.4563, five author groups each with its own address).
#[test]
fn osa_style_author_groups_append() {
  let tex = r"\documentclass{article}
\usepackage{ol2}
\begin{document}
\title{Correlated photons}
\author{D. L. Boiko}
\address{Centre Suisse, Neuchatel, Switzerland}
\author{N. Brauer, M. Sergio}
\address{EPFL, Lausanne, Switzerland}
\begin{abstract}
We detect photons.
\end{abstract}
\ocis{030.5260.}
Text.
\end{document}
";
  let xml = convert_clean(tex);
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>D. L. Boiko</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Centre Suisse, Neuchatel, Switzerland</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>N. Brauer</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">EPFL, Lausanne, Switzerland</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>M. Sergio</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">EPFL, Lausanne, Switzerland</contact>\n  </creator>",
  ]);
}

/// 64u: REVTeX 2.1's `{title}` environment is the title, an `{instit}` environment the institution of the authors
/// before it, `\receipt` the received date, `\nonum` an unnumbered heading (revtex.sty v2.1, hep-lat9302012.tex:
/// 1057-1072, 1340-1391; hep-ph9301226, cond-mat9311041). Repro
/// sectioning-frontmatter/revtex2_title_and_instit_environments.
#[test]
fn revtex2_title_and_instit_environments() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex2_title_and_instit_environments.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "title",
    &[],
    "<title>The effects of a strongly interacting Higgs sector<break/>on scattering</title>",
  );
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>A. Abbasabadi</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Physical Sciences Department<break/>Ferris State University, Big Rapids, Michigan 49307</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>David Bowser-Chao</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Center for Particle Physics, University of Texas, Austin, Texas 78712</contact>\n  </creator>",
  ]);
  assert_element(
    &xml,
    "date",
    &[],
    "<date name=\"Received \" role=\"received\">October 1992</date>",
  );
  // `\nonum\section{Acknowledgments}` unnumbered, the section before it numbered
  assert!(xml.contains("<title>Acknowledgments</title>"), "{xml}");
  assert!(
    xml.contains("<title><tag close=\" \">I</tag>Introduction</title>"),
    "{xml}"
  );
}

/// 64u: l-aa's `\keywords` takes no argument (l-aa.sty:648): a `{keywords}` environment's body is the keywords, whole
/// (astro-ph9609156, astro-ph9803302; the aa binding's one-argument `\keywords` took the letter "C").
/// Repro sectioning-frontmatter/l_aa_keywords_without_argument.
#[test]
fn l_aa_keywords_without_argument() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/l_aa_keywords_without_argument.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Key words: \">Cosmology: theory; dark matter</keywords>",
  );
  assert_eq!(xml.matches("<keywords ").count(), 1, "{xml}");
  assert!(!xml.contains("dark matter</p>"), "{xml}");
}

/// 64u: l-aa's `\keywords` text form, to the paragraph's end (`\keywords ISM: molecules --- …`, astro-ph0001079) or
/// to the `\end{abstract}` it stands in (astro-ph9508023).
#[test]
fn l_aa_keywords_text_to_paragraph_end() {
  for (keywords, end) in [
    (
      "\\keywords ISM: molecules --- star formation\n\n",
      "\\end{abstract}",
    ),
    ("\\keywords X-rays: general -- quasars\n", "\\end{abstract}"),
  ] {
    let tex = format!(
      "\\documentstyle{{l-aa}}\n\\begin{{document}}\n\\thesaurus{{08}}\n\\title{{T}}\n\\author{{A. Ann}}\n\\institute{{X Univ}}\n\\maketitle\n\\begin{{abstract}}\nWe study.\n{keywords}{end}\n\\section{{Intro}}\nText.\n\\end{{document}}\n"
    );
    let xml = convert_clean(&tex);
    let text = keywords.trim_start_matches("\\keywords ").trim();
    let text = text.replace("---", "\u{2014}").replace("--", "\u{2013}");
    assert_element(
      &xml,
      "keywords",
      &[],
      &format!("<keywords name=\"Key words: \">{text}</keywords>"),
    );
    assert!(
      xml.contains("<title><tag close=\" \">1</tag>Intro</title>"),
      "{xml}"
    );
  }
}

/// 64u: an l-aa `\institute` of `$^n$`-marked entries under names marked alike is an affiliation list labelled by them
/// (atom-ph9605001, astro-ph9609131), not one entry for the author of mark 1; a single institute is every author's
/// (astro-ph9405007). Repro sectioning-frontmatter/l_aa_marked_institute_names_its_authors.
#[test]
fn l_aa_marked_institute_names_its_authors() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/l_aa_marked_institute_names_its_authors.tex"
  );
  let xml = convert_clean(tex);
  let able = &creators(&xml)[0];
  let baker = &creators(&xml)[1];
  assert_eq!(creators(&xml).len(), 2, "{xml}");
  assert!(
    able.contains("Copernicus Center, Warsaw</contact>")
      && able.contains("MPE, Garching</contact>"),
    "{xml}"
  );
  assert!(!able.contains("Potsdam"), "{xml}");
  assert!(
    baker.contains("AIP, Potsdam</contact>") && !baker.contains("Warsaw"),
    "{xml}"
  );
  let one = convert_clean(
    "\\documentstyle{l-aa}\n\\begin{document}\n\\thesaurus{08}\n\\title{T}\n\\author{A. Ann, B. Bob}\n\\institute{MPE, Garching}\n\\maketitle\nText.\n\\end{document}\n",
  );
  assert_eq!(creators(&one).len(), 2, "{one}");
  for creator in creators(&one) {
    assert!(
      creator.contains("role=\"affiliation\">MPE, Garching</contact>"),
      "{one}"
    );
  }
}

/// 64u: l-aa's float captions are labelled "Fig. 1." and "Table 1." (l-aa.sty:381, :392), the caption after a space,
/// no colon (:371-377 `{\bf #1} #2`).
#[test]
fn l_aa_caption_labels() {
  let xml = convert_clean(
    "\\documentstyle{l-aa}\n\\begin{document}\n\\thesaurus{08}\n\\title{T}\n\\author{A. Ann}\n\\institute{X}\n\\maketitle\n\\begin{figure}\\caption{A plot.}\\end{figure}\n\\begin{table}\\caption{A list.}\\end{table}\nText.\n\\end{document}\n",
  );
  assert!(
    xml.contains("<caption><tag close=\" \">Fig. 1.</tag>A plot.</caption>"),
    "{xml}"
  );
  assert!(
    xml.contains("<caption><tag close=\" \">Table 1.</tag>A list.</caption>"),
    "{xml}"
  );
}

/// 64u: an unmarked EPL `\institute` is one institution, its `\\` a line break in it (0802.1166, 0704.3325).
/// Repro sectioning-frontmatter/epl_unmarked_institute_one_entry.
#[test]
fn epl_unmarked_institute_one_entry() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/epl_unmarked_institute_one_entry.tex"
  );
  let xml = convert_clean(tex);
  let creators = creators(&xml);
  assert_eq!(creators.len(), 2, "{xml}");
  for creator in creators {
    assert_eq!(creator.matches("role=\"affiliation\"").count(), 1, "{xml}");
    assert!(
      creator.contains(
        ">Laboratoire de Physique, CNRS UMR 8550,<break/>24 rue Lhomond, 75005 Paris, France</contact>"
      ),
      "{xml}"
    );
  }
}

/// 64u: under EPL's `comment`/`reply` option the class prints no abstract (epl2.cls:149-157, :817): the author's
/// abstract is the document's metadata, not lost (0907.0855); a note that only labels an `\email` (`\thanks{E-mail:
/// \email{…}}`) is the author's email contact, named by that label (1110.0608).
#[test]
fn epl_comment_abstract_is_metadata() {
  let xml = convert_clean(
    "\\documentclass[comment]{epl2}\n\\title{T}\n\\author{Ann Able\\thanks{E-mail: \\email{ann@x.org}}}\n\\institute{Nanjing University}\n\\abstract{We study.}\n\\begin{document}\n\\maketitle\nText.\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "rdf",
    &[],
    "<rdf property=\"dcterms:abstract\">We study.</rdf>",
  );
  assert!(!xml.contains("<abstract"), "{xml}");
  assert!(
    xml.contains("<contact name=\"E-mail: \" role=\"email\">ann@x.org</contact>"),
    "{xml}"
  );
  assert!(!xml.contains("thanks"), "{xml}");
}

/// 64u: EPL email forms among the names: a bare `\email` after a comma is the author's email, not part of the name;
/// one in markup (`\email{\href{mailto:…}{…}}`) too (0907.0855); a note whose label sits inside the `\email`
/// (`\footnote{\email{E-mail:x@y}}`, 1212.3881) is the email named by that label.
#[test]
fn epl_email_forms_among_names() {
  let doc = |author: &str| {
    format!(
      "\\documentclass{{epl2}}\n\\usepackage{{hyperref}}\n\\title{{T}}\n\\author{{{author}}}\n\\institute{{Inst One}}\n\\abstract{{Abs.}}\n\\begin{{document}}\n\\maketitle\nText.\n\\end{{document}}\n"
    )
  };
  let xml = convert_clean(&doc("A. Author, \\email{a@b.org}"));
  assert!(xml.contains("<personname>A. Author</personname>"), "{xml}");
  assert!(
    xml
      .contains("role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">a@b.org</text></contact>"),
    "{xml}"
  );
  let xml = convert_clean(&doc("A. Author, \\email{\\href{mailto:a@b.org}{a@b.org}}"));
  assert!(xml.contains("<personname>A. Author</personname>"), "{xml}");
  assert!(
    xml.contains(
      "role=\"email\"><ref class=\"ltx_href\" href=\"mailto:a@b.org\">a@b.org</ref></contact>"
    ),
    "{xml}"
  );
  let xml = convert_clean(&doc("A. Author\\footnote{\\email{E-mail:a@b.org}}"));
  assert!(
    xml.contains("<contact name=\"E-mail: \" role=\"email\">a@b.org</contact>"),
    "{xml}"
  );
}

/// 64u: an OSA `\address` line naming an email address is the email of the author carrying its mark, named by its
/// label (0705.1274, 1006.2188, 1011.1336). Repro sectioning-frontmatter/osa_style_address_email_line.
#[test]
fn osa_style_address_email_line() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/osa_style_address_email_line.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ A, City A</contact>\n    <contact name=\"Corresponding author: \" role=\"email\">able@x.edu</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ B, City B</contact>\n  </creator>",
  ]);
}

/// 64u: OSA email lines (osa_journal_sty.rs `email_line`), placed by `\lx@add@contact@placed` (base_utilities.rs
/// `placed_contact_owner`): an unmarked address no name spells goes to the block, not to the last author (0901.2914);
/// a mark on the address names its author, above the spelling (`*b@x` under `A$^*$`: A's); an ordinal (`5$^{th}$`) is
/// no mark and leaves the line an affiliation; a leading `\#` is the address's text, a parenthesis after it naming its
/// owner is not.
#[test]
fn osa_email_line_forms() {
  let doc = |author: &str, address: &str, email: &str| {
    format!(
      "\\documentclass{{article}}\n\\usepackage{{opex3}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{{author}}}\n\\address{{{address}}}\n{email}\n\\begin{{abstract}}A.\\end{{abstract}}\nText.\n\\end{{document}}\n"
    )
  };
  // unmarked and unspelled: the block's, no author's
  assert_eq!(
    creators(&convert_clean(&doc(
      "A. Able and B. Baker",
      "Univ X",
      "\\email{info@lab.org}"
    ))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator role=\"author\">\n    <contact name=\"Email: \" role=\"email\">info@lab.org</contact>\n  </creator>",
    ]
  );
  // the mark names the author
  assert_eq!(
    creators(&convert_clean(&doc(
      "A. Able$^*$ and B. Baker",
      "Univ X",
      "\\email{*zzz@x.org}"
    ))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able<sup xml:id=\"id1\">*</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Email: \" role=\"email\">zzz@x.org</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
    ]
  );
  // the address's own mark outranks its spelling
  assert_eq!(
    creators(&convert_clean(&doc(
      "A. Able$^*$ and B. Baker",
      "Univ X",
      "\\email{*baker@x.org}"
    ))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able<sup xml:id=\"id1\">*</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Email: \" role=\"email\">baker@x.org</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
    ]
  );
  // an ordinal is no mark, the line no email line
  assert_eq!(
    creators(&convert_clean(&doc(
      "A. Able$^{1}$ and B. Baker$^{1}$",
      "$^1$Univ X, room 5$^{th}$ floor, contact: baker@x.org",
      ""
    ))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X, room 5<sup xml:id=\"id1\">th</sup> floor, contact: baker@x.org</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X, room 5<sup xml:id=\"id2\">th</sup> floor, contact: baker@x.org</contact>\n  </creator>",
    ]
  );
  // a leading `\\#` is the address's own
  assert_eq!(
    creators(&convert_clean(&doc(
      "A. Able and B. Baker",
      "Univ X",
      "\\email{\\#able@x.org}"
    ))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Email: \" role=\"email\">#able@x.org</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
    ]
  );
  // a parenthesis after the address naming its owner is no part of the address the contact links to
  assert_eq!(
    creators(&convert_clean(&doc(
      "A. Able",
      "Univ X",
      "\\email{Email: able@x.org (A. Able)}"
    ))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Email: \" role=\"email\">able@x.org</contact>\n  </creator>",
    ]
  );
  let xml = convert_clean(&doc(
    "S. Longhi",
    "Dipartimento di Fisica, Politecnico di Milano, Italy (stefano.longhi@polimi.it)",
    "",
  ));
  assert!(!xml.contains("role=\"email\""), "{xml}");
}

/// 64u: an OSA `\address` whose entries are separated by blank lines is one line per entry (1109.5493, 1002.3196).
#[test]
fn osa_style_address_blank_lines() {
  let xml = convert_clean(
    "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able$^{1}$, B. Baker$^{2}$ and C. Cole$^{3}$}\n\\address{$^1$Univ X, City X\n\n$^2$Univ Y, City Y\n\n$^3$Univ Z, City Z}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n",
  );
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X, City X</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y, City Y</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>C. Cole</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Z, City Z</contact>\n  </creator>",
  ]);
}

/// 64u: OSA's `\homepage` is placed as an email is: a group's page the block's, not the last author's (1004.2645);
/// a sole author's that author's; the address a link.
#[test]
fn osa_homepage_placed() {
  let doc = |author: &str, url: &str| {
    format!(
      "\\documentclass{{article}}\n\\usepackage{{opex3}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{{author}}}\n\\address{{Univ X}}\n\\homepage{{{url}}}\n\\begin{{abstract}}A.\\end{{abstract}}\nText.\n\\end{{document}}\n"
    )
  };
  assert_eq!(
    creators(&convert_clean(&doc(
      "A. Able and B. Baker",
      "http://www.lab.org"
    ))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator role=\"author\">\n    <contact name=\"URL: \" role=\"url\"><ref class=\"ltx_nolink ltx_url\" font=\"typewriter\" href=\"http://www.lab.org\">http://www.lab.org</ref></contact>\n  </creator>",
    ]
  );
  assert_eq!(
    creators(&convert_clean(&doc("A. Able", "http://www.lab.org/~able"))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"URL: \" role=\"url\"><ref class=\"ltx_nolink ltx_url\" font=\"typewriter\" href=\"http://www.lab.org/~able\">http://www.lab.org/~able</ref></contact>\n  </creator>",
    ]
  );
}

/// 64u: the OSA styles' journal abbreviations are the ones every opex3.sty version `\def`s; a paper's own definition
/// of a later version's name (0909.4968 `\newcommand{\as}{\alpha\ssub{s}}`) is the paper's, and of one of the style's
/// names a `\def` or `\renewcommand` replaces it, where `\providecommand` and `\newcommand` keep the style's.
#[test]
fn osa_paper_abbreviation_is_the_papers() {
  let paragraph = |preamble: &str, text: &str| {
    let xml = convert_clean(&format!(
      "\\documentclass{{article}}\n\\usepackage{{opex3}}\n{preamble}\\begin{{document}}\n\\title{{T}}\n\\author{{A. Able}}\n\\address{{Univ X}}\n{text}\n\\begin{{abstract}}A.\\end{{abstract}}\nText.\n\\end{{document}}\n"
    ));
    let start = xml.find("<p xml:id=\"p1.1\">").unwrap_or(0);
    let end = xml[start..]
      .find("</p>")
      .map_or(start, |i| start + i + "</p>".len());
    xml[start..end].to_string()
  };
  assert_eq!(
    paragraph("\\newcommand{\\as}{\\alpha_{s}}\n", "We use $q\\as$ here."),
    "<p xml:id=\"p1.1\">We use <Math mode=\"inline\" tex=\"q\\alpha_{s}\" text=\"q * alpha _ s\" xml:id=\"p1.m1\">\n        <XMath xml:id=\"p1.m1.1\">\n          <XMApp xml:id=\"p1.m1.1.1\">\n            <XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">q</XMTok>\n            <XMApp xml:id=\"p1.m1.1.1.3\">\n              <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n              <XMTok font=\"italic\" name=\"alpha\" role=\"UNKNOWN\">α</XMTok>\n              <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">s</XMTok>\n            </XMApp>\n          </XMApp>\n        </XMath>\n      </Math> here.</p>"
  );
  assert_eq!(
    paragraph(
      "\\def\\nat{Nature}\n\\renewcommand{\\ol}{Optics Letters}\n\\providecommand{\\oc}{Optics Comm}\n",
      "See \\nat{} and \\ol{} and \\oc."
    ),
    "<p xml:id=\"p1.1\">See Nature and Optics Letters and Opt. Commun. .</p>"
  );
  assert_eq!(
    paragraph("\\newcommand{\\nat}{Nature}\n", "See \\nat."),
    "<p xml:id=\"p1.1\">See Nature (London).</p>"
  );
}

/// 64u: a mark set in bold (`Hui Cao$^{\bf{1,4},*}$`, 1109.3726) labels what its plain twin labels: the font switch is
/// no part of a label (base_utilities.rs `clean_frontmatter_labels`, OXIDIZED_DESIGN #34).
#[test]
fn osa_mark_font_switch_is_no_label() {
  // p1
  let p1 = "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able,$^{\\bf{1}}$ B. Baker,$^{\\bf{2,3},\\dagger}$ C. Cole$^{\\bf{1,3},*}$}\n\\address{$^1$Univ X\\\\\n$^2$Univ Y\\\\\n$^3$Univ Z\\\\\n}\n\\email{*cole@x.edu,$^\\dagger$baker@y.edu}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n";
  assert_eq!(creators(&convert_clean(p1)), vec![
    "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Z</contact>\n    <contact name=\"Email: \" role=\"email\">baker@y.edu</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>C. Cole</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Z</contact>\n    <contact name=\"Email: \" role=\"email\">cole@x.edu</contact>\n  </creator>",
  ]);
  // (65e) a `\#` mark places the same email as `\dagger`: its key is no parameter token (`symbol_mark_key`)
  assert_eq!(
    creators(&convert_clean(&p1.replace("\\dagger", "\\#"))),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Z</contact>\n    <contact name=\"Email: \" role=\"email\">baker@y.edu</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>C. Cole</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Z</contact>\n    <contact name=\"Email: \" role=\"email\">cole@x.edu</contact>\n  </creator>",
    ]
  );
}

/// 64u: an address no mark or name places, after an `\author` call naming one author, is that group's only author's
/// (`\lx@frontmatter@group@start`, 0906.4563), not the block's.
#[test]
fn osa_group_only_author_owns_the_email() {
  // p2
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able and B. Baker}\n\\address{Univ X}\n\\author{C. Cole}\n\\address{Univ Y}\n\\email{info@lab.org}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>C. Cole</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y</contact>\n    <contact name=\"Email: \" role=\"email\">info@lab.org</contact>\n  </creator>",
    ]
  );
}

/// 64u: the mark shapes of OSA email lines (osa_journal_sty.rs `email_line`): a mark inside a font command
/// (`\textit{$^3$}`, 1306.0672), inside a colour group (`{\color{blue}$^*$…}`, 1303.3906), a numeric mark on a labelled
/// line (`$^2$e-mail: …`, 1306.2982: answered, so it leaves the name), a bare `*` after the comma ending a name
/// (1104.1413), and babel French's active `:` in the label (1210.1049).
#[test]
fn osa_email_line_mark_shapes() {
  // p4
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able$^{1,3}$ and B. Baker$^{2}$}\n\\address{\\textit{$^1$}Univ X\\\\\n\\textit{$^2$}Univ Y\\\\}\n\\email{\\textit{$^3$}able@x.edu}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Email: \" role=\"email\">able@x.edu</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y</contact>\n  </creator>",
    ]
  );
  // p5
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\usepackage{xcolor}\n\\begin{document}\n\\title{T}\n\\author{A. Able and B. Baker$^{*}$}\n\\address{Univ X\\\\\n{\\color{blue}$^*$baker@x.gov}\n}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker<sup xml:id=\"id1\">*</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Email: \" role=\"email\">baker@x.gov</contact>\n  </creator>",
    ]
  );
  // p6
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able,$^{1,*}$ and B. Baker$^{1,2}$}\n\\address{$^1$Univ X\\\\\n$^2$e-mail: baker@x.edu\\\\\n$^*$e-mail: able@x.edu, aa@y.com}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"e-mail: \" role=\"email\">able@x.edu</contact>\n    <contact name=\"e-mail: \" role=\"email\">aa@y.com</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"e-mail: \" role=\"email\">baker@x.edu</contact>\n  </creator>",
    ]
  );
  // p7
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able,* B. Baker, \\\\ and C. Cole}\n\\address{Univ X}\n\\email{*able@x.gov}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able<sup xml:id=\"id1\">*</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Email: \" role=\"email\">able@x.gov</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>C. Cole</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
    ]
  );
  // p8
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\usepackage[english,french]{babel}\n\\begin{document}\n\\title{T}\n\\author{A. Able and B. Baker$^*$}\n\\address{Univ X\\\\\n$^*$Corresponding author: baker@x.fr}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker<sup xml:id=\"id1\">*</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Corresponding author : \" role=\"email\">baker@x.fr</contact>\n  </creator>",
    ]
  );
}

/// 64u: an OSA `\address` line marked with several labels is one line a label, and the next piece of the line is not
/// part of it (`$^{1,2}$Dept of EE, …, $^3$Dept of CS, …`, 1406.0824, with its `$^1$a@x, $^2$b@x, and $^3$c@y` email
/// line); an author marked with two of the labels holds the affiliation once, and the copy for a label no author
/// carries is not shared on the block creator.
#[test]
fn osa_several_labels_one_line_each() {
  // p3
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able$^1$, B. Baker$^2$, and C. Cole$^3$}\n\\address{$^1$able@x.edu, $^2$baker@x.edu, and $^3$cole@y.edu}\n\\address{$^{1,2}$Dept of EE, Univ X, $^3$Dept of CS, Univ X}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Email: \" role=\"email\">able@x.edu</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Dept of EE, Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Email: \" role=\"email\">baker@x.edu</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Dept of EE, Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>C. Cole</personname>\n    <contact name=\"Email: \" role=\"email\">cole@y.edu</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Dept of CS, Univ X</contact>\n  </creator>",
    ]
  );
  // p9
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able$^{1,2}$ and B. Baker$^{2}$}\n\\address{$^{1,2}$Univ X\\\\\n$^2$Univ Y}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y</contact>\n  </creator>",
    ]
  );
  // a label no author carries: its copy is no orphan for the block (1101.5841 `$^{1,5}$`), in either order
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able$^1$ and B. Baker$^2$}\n\\address{$^{1,3}$Univ X\\\\\n$^2$Univ Y}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y</contact>\n  </creator>",
    ]
  );
  assert_eq!(
    creators(&convert_clean(
      "\\documentclass{article}\n\\usepackage{opex3}\n\\begin{document}\n\\title{T}\n\\author{A. Able$^1$ and B. Baker$^2$}\n\\address{$^{3,1}$Univ X\\\\\n$^2$Univ Y}\n\\begin{abstract}A.\\end{abstract}\nText.\n\\end{document}\n"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>A. Able</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ X</contact>\n  </creator>",
      "<creator before=\"  \" role=\"author\">\n    <personname>B. Baker</personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ Y</contact>\n  </creator>",
    ]
  );
}

/// 64u: the OSA styles' visible strings: "Fig." and "Table" captioned `#1. #2` (opex3.sty:84-96, ol2.sty:91-100,
/// osajnl2.sty:89-100), the section number with a period (`\@seccntformat`, opex3.sty:156), each version's
/// `\thesubsection`, and the reference list's heading "References and links" (opex3.sty:86) or "References"; what a
/// `\ref` prints, the refnum, has no period (`1`, `A`, `1.A`).
#[test]
fn osa_names_captions_and_headings() {
  let heads = |sty: &str| {
    let xml = convert_clean(&format!(
      "\\documentclass{{article}}\n\\usepackage{{{sty}}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{A. Able}}\n\\address{{Univ X}}\n\\section{{Intro}}\\subsection{{Sub}}\\subsubsection{{Subsub}}\n\\begin{{figure}}\\caption{{Cap F.}}\\end{{figure}}\\begin{{table}}\\caption{{Cap T.}}\\end{{table}}\n\\begin{{thebibliography}}{{1}}\\bibitem{{a}} A.\\end{{thebibliography}}\n\\begin{{abstract}}A.\\end{{abstract}}\nText.\n\\end{{document}}\n"
    ));
    let mut heads = Vec::new();
    for (open, close) in [
      ("<title>", "</title>"),
      ("<caption>", "</caption>"),
      ("<tag role=\"refnum\">", "</tag>"),
    ] {
      let mut rest = xml.as_str();
      while let Some(i) = rest.find(open) {
        let end = i + rest[i..].find(close).map_or(0, |j| j + close.len());
        heads.push((i + (xml.len() - rest.len()), rest[i..end].to_string()));
        rest = &rest[end..];
      }
    }
    heads.sort();
    heads.into_iter().map(|(_, head)| head).collect::<Vec<_>>()
  };
  assert_eq!(heads("opex3"), vec![
    "<title>T</title>",
    "<tag role=\"refnum\">1</tag>",
    "<title><tag close=\". \">1</tag>Intro</title>",
    "<tag role=\"refnum\">1.1</tag>",
    "<title><tag close=\". \">1.1</tag>Sub</title>",
    "<tag role=\"refnum\">1.1.1</tag>",
    "<title><tag close=\". \">1.1.1</tag>Subsub</title>",
    "<tag role=\"refnum\">1</tag>",
    "<caption><tag close=\". \">Fig. 1</tag>Cap F.</caption>",
    "<tag role=\"refnum\">1</tag>",
    "<caption><tag close=\". \">Table 1</tag>Cap T.</caption>",
    "<title>References and links</title>",
    "<tag role=\"refnum\">1</tag>",
  ]);
  assert_eq!(heads("ol2"), vec![
    "<title>T</title>",
    "<tag role=\"refnum\">1</tag>",
    "<title><tag close=\". \">1</tag>Intro</title>",
    "<tag role=\"refnum\">A</tag>",
    "<title><tag close=\". \">A</tag>Sub</title>",
    "<tag role=\"refnum\">1</tag>",
    "<title><tag close=\". \">1</tag>Subsub</title>",
    "<tag role=\"refnum\">1</tag>",
    "<caption><tag close=\". \">Fig. 1</tag>Cap F.</caption>",
    "<tag role=\"refnum\">1</tag>",
    "<caption><tag close=\". \">Table 1</tag>Cap T.</caption>",
    "<title>References</title>",
    "<tag role=\"refnum\">1</tag>",
  ]);
  assert_eq!(heads("osajnl2"), vec![
    "<title>T</title>",
    "<tag role=\"refnum\">1</tag>",
    "<title><tag close=\". \">1</tag>Intro</title>",
    "<tag role=\"refnum\">1.A</tag>",
    "<title><tag close=\". \">1.A</tag>Sub</title>",
    "<tag role=\"refnum\">1.A.1</tag>",
    "<title><tag close=\". \">1.A.1</tag>Subsub</title>",
    "<tag role=\"refnum\">1</tag>",
    "<caption><tag close=\". \">Fig. 1</tag>Cap F.</caption>",
    "<tag role=\"refnum\">1</tag>",
    "<caption><tag close=\". \">Table 1</tag>Cap T.</caption>",
    "<title>References</title>",
    "<tag role=\"refnum\">1</tag>",
  ]);
}

/// 64u fix: EPL's `{widetext}` is a TeX group, as epl2.cls:1057 defines it, so display math `$$…$$` in it stays a
/// display equation (0810.4982: 14 `Script _` errors from the boxed `#body` environment). Repro
/// block-model/epl_widetext_dollardollar_is_display.
#[test]
fn epl_widetext_dollardollar_is_display() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/epl_widetext_dollardollar_is_display.tex"
  ));
  assert_element(
    &xml,
    "equation",
    &[],
    r##"<equation xml:id="S0.Ex1"><Math mode="display" tex="a_{1}+b_{2}=c^{3}" text="a _ 1 + b _ 2 = c ^ 3" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMApp xml:id="S0.Ex1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="S0.Ex1.m1.1.1.2"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp xml:id="S0.Ex1.m1.1.1.2.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMApp xml:id="S0.Ex1.m1.1.1.2.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp><XMApp xml:id="S0.Ex1.m1.1.1.3"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp></XMApp></XMath></Math></equation>"##,
  );
}
