//! P5b (Option C P5, fact 2): an author call or `\and` column printing only a collaboration framed for the names before
//! it is their collaboration contact, its framing read from the words the reading digests
//! (`resolver::lines::frames_the_names`, `framed_collaboration`) — not the token reading `whole_name_collaboration` nor
//! `wrapped_collaboration_out` (both deleted) that `\lx@add@author` ran on the spelling (OXIDIZED_DESIGN_DIVERGENCES
//! #967). One table: each row a spelling or a route of the framing.

use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch61::creators_of,
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

/// elsarticle's `\author{Ann Able}\author{Bob Baker}\address{Univ X}\author{<call>}`, `pre` in the preamble.
fn elsarticle(pre: &str, call: &str) -> String {
  format!(
    "\\documentclass{{elsarticle}}\n{pre}\n\\begin{{document}}\n\\begin{{frontmatter}}\n\\title{{T}}\n\\author{{Ann \
     Able}}\n\\author{{Bob Baker}}\n\\address{{Univ X}}\n\\author{{{call}}}\n\\end{{frontmatter}}\nText.\n\\end{{document}}\n"
  )
}

/// Each row: the document, its whole creators and its exact (errors, warnings); pdflatex 0 `!` lines each (the repro
/// headers; the inline rows checked the same way), the collaboration printed under or after the names.
/// - the framing printed by a document macro (segaudit col1), a `\let` copy, `\csname`, `\MakeUppercase`, ties and a font
///   group, "Representing" by a macro, inside a `\parbox`: the base read these by the reading's `speaks_for`, Perl a
///   third creator;
/// - brackets, literal and by a macro, and dashes (`-- XYZ Collaboration --`, hep-ph0502212's framing): red on the base,
///   a third creator (the dashed one "XYZ Collaboration", its dashes lost);
/// - "on behalf of the XYZ and ABC Collaborations", literal and by a macro: red on the base, a creator "on behalf of the
///   XYZ" holding "ABC Collaborations";
/// - the kernel's `\and` column by a macro, the same under an unknown class (OmniBus; its Warning the missing class's),
///   revtex's `\author`;
/// - the safe side: a framed collaboration before any name, and "The XYZ Collaboration" after them (framed by "the"
///   alone), stay creators, as they were (1912.05868).
#[test]
fn a_framed_collaboration_call_is_the_names_contact() {
  // (Ann Able and Bob Baker, each with `contacts` then the collaboration `text`, written as its contact holds it)
  let two = |contacts: &str, text: &str| -> Vec<String> {
    [
      "<creator role=\"author\"><personname>Ann Able",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker",
    ]
    .iter()
    .map(|head| {
      format!(
        "{head}</personname>{contacts}<contact role=\"collaboration\">{text}</contact></creator>"
      )
    })
    .collect()
  };
  let address = "<contact name=\"Address:\u{a0}\" role=\"address\">Univ X</contact>";
  let behalf = two(address, "On behalf of the XYZ Collaboration");
  let bracketed = two(address, "(for the XYZ Collaboration)");
  let both = two(address, "on behalf of the XYZ and ABC Collaborations");
  let column = two("", "on behalf of the XYZ Collaboration");
  let mut col1 = two(address, "On behalf of the XYZ Consortium");
  col1[0] = col1[0].replace(
    "</contact></creator>",
    "</contact><contact name=\"Corresponding author:\u{a0}\" role=\"correspondent\">Corresponding author.</contact></creator>",
  );
  let font = |id: u8| {
    format!(
      "<text font=\"italic\" xml:id=\"id{id}\">on\u{a0}behalf\u{a0}of</text>\u{a0}the XYZ Collaboration"
    )
  };
  let mut tied = two(address, &font(1));
  tied[1] = tied[1].replace("id1", "id2");
  let parbox = |id: u8| {
    format!(
      "<inline-block class=\"ltx_parbox\" vattach=\"middle\" width=\"142.3pt\" xml:id=\"id{id}\"><p xml:id=\"id{id}.1\">On \
        behalf of the XYZ Collaboration</p></inline-block>"
    )
  };
  let mut boxed = two(address, &parbox(1));
  boxed[1] = boxed[1].replace("id1", "id2");
  let uncovered = |first: &str, second: &str, third: Option<&str>| -> Vec<String> {
    let mut out = vec![
      format!("<creator role=\"author\"><personname>{first}</personname>{address}</creator>"),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>{second}</personname>{address}</creator>"
      ),
    ];
    out.extend(third.map(|name| {
      format!("<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>{name}</personname></creator>")
    }));
    out
  };
  let omnibus = repro!("article_and_column_framed_collaboration_by_macro_is_a_contact")
    .replace("{article}", "{nosuchclass}");
  let rows: Vec<(String, Vec<String>, (usize, usize))> = vec![
    (repro!("elsarticle_author_framed_collaboration_by_macro_is_a_contact").into(), col1, (0, 0)),
    (
      elsarticle("\\newcommand\\oboA{On behalf of}\\let\\obo\\oboA", "\\obo\\ the XYZ Collaboration"),
      behalf.clone(),
      (0, 0),
    ),
    (
      elsarticle(
        "\\expandafter\\def\\csname obo\\endcsname{On behalf of}",
        "\\csname obo\\endcsname{} the XYZ Collaboration",
      ),
      behalf.clone(),
      (0, 0),
    ),
    (
      elsarticle("\\newcommand\\obo{\\MakeUppercase{on behalf of}}", "\\obo{} the XYZ Collaboration"),
      two(address, "ON BEHALF OF the XYZ Collaboration"),
      (0, 0),
    ),
    (elsarticle("", "{\\itshape on~behalf~of}~the XYZ Collaboration"), tied, (0, 0)),
    (
      elsarticle("\\newcommand\\obo{Representing}", "\\obo{} the XYZ Collaboration"),
      two(address, "Representing the XYZ Collaboration"),
      (0, 0),
    ),
    (
      elsarticle("\\newcommand\\obo{On behalf of}", "\\parbox{5cm}{\\obo{} the XYZ Collaboration}"),
      boxed,
      (0, 0),
    ),
    (repro!("elsarticle_author_bracketed_collaboration_is_a_contact").into(), bracketed.clone(), (0, 0)),
    (elsarticle("\\newcommand\\obo{for the}", "(\\obo{} XYZ Collaboration)"), bracketed, (0, 0)),
    (elsarticle("", "-- XYZ Collaboration --"), two(address, "\u{2013} XYZ Collaboration \u{2013}"), (0, 0)),
    (repro!("elsarticle_author_framed_two_collaborations_is_one_contact").into(), both.clone(), (0, 0)),
    (
      elsarticle("\\newcommand\\obo{on behalf of}", "\\obo{} the XYZ and ABC Collaborations"),
      both,
      (0, 0),
    ),
    (repro!("article_and_column_framed_collaboration_by_macro_is_a_contact").into(), column.clone(), (0, 0)),
    // (its Warning the unknown class's: `missing_file:nosuchclass`, using OmniBus)
    (omnibus, column, (0, 1)),
    (
      "\\documentclass{revtex4-2}\n\\newcommand\\obo{on behalf of}\n\\begin{document}\n\\title{T}\n\\author{Ann \
       Able}\n\\author{Bob Baker}\n\\affiliation{Univ X}\n\\author{\\obo{} the XYZ Collaboration}\n\\maketitle\nText.\n\
       \\end{document}\n"
        .into(),
      two(
        "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact>",
        "on behalf of the XYZ Collaboration",
      ),
      (0, 0),
    ),
    (
      "\\documentclass{elsarticle}\n\\newcommand\\obo{On behalf of}\n\\begin{document}\n\\begin{frontmatter}\n\\title{T}\n\
       \\author{\\obo{} the XYZ Collaboration}\n\\author{Ann Able}\n\\address{Univ X}\n\\end{frontmatter}\nText.\n\
       \\end{document}\n"
        .into(),
      uncovered("On behalf of the XYZ Collaboration", "Ann Able", None),
      (0, 0),
    ),
    (
      elsarticle("", "The XYZ Collaboration"),
      uncovered("Ann Able", "Bob Baker", Some("The XYZ Collaboration")),
      (0, 0),
    ),
  ];
  // (every row converted before the verdict, each differing row reported with what it made)
  let mut differ = Vec::new();
  for (row, (tex, expected, counts)) in rows.iter().enumerate() {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    let found = (creators_of(&xml), (error_count(&log), warning_count(&log)));
    if found != (expected.clone(), *counts) {
      differ.push(format!("row {row}: {found:#?}\n{log}"));
    }
  }
  assert!(differ.is_empty(), "{}", differ.join("\n\n"));
}
