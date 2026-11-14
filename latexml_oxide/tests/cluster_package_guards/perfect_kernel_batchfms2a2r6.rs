//! Guards for FM S2a.2 r6: the frontmatter observer's resolver on the r5 landing review's findings (revfms2a2_5) and the
//! main session's decisions 2 and 3 of 2026-10-22 (OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r6"; KERNEL_CAPABILITIES
//! K23). Each shape was red on r5 (the int63a code, 410a2a85e8): a replaced `\author` call's footnote
//! mark taking the text built before it (finding 1, 2510.26354), a printed glyph dropped from a name (finding 2,
//! 1706.00234), a note's inner links given to its owner whoever they name (decision 2: 2001.02600, 1803.04996,
//! 2504.08644, 2602.13176), a braced `{a,b,c}@x` group in an `\institute` entry given whole to every owner (decision 3),
//! an address list's note never placed (finding 6). Every probe compiles clean with pdflatex (TL 2025) and converts with
//! no Error or Warning, and the XML is byte-identical with the switch on and off.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::fm_reading_json;
use serde_json::Value;

/// Convert `tex`, asserting a clean conversion: the whole dump.
fn dump(tex: &str) -> Value {
  let (log, _, json) =
    convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| fm_reading_json());
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  super::perfect_kernel_batchfms2a1::parsed(&json)
}

/// The resolved tuples compactly, as `perfect_kernel_batchfms2a2::tuples` writes them: `C <name> [<requests>] <rule>`,
/// then its `a` affiliations, `c <role>` contacts, `n` notes; `B <role>` the block's, `D <role>` what was dropped.
fn tuples(tex: &str) -> Vec<String> {
  let resolved = dump(tex)["resolved"].clone();
  let mut out = Vec::new();
  let item = |prefix: &str, a: &Value, role: bool| {
    let rule = a["rule"].as_str().unwrap_or("");
    let text = a["text"].as_str().unwrap_or("");
    if role {
      format!(
        "{prefix} {} {text} | {rule}",
        a["role"].as_str().unwrap_or("")
      )
    } else {
      format!("{prefix} {text} | {rule}")
    }
  };
  for c in resolved["creators"].as_array().expect("creators") {
    let requests: Vec<&str> = c["requests"]
      .as_array()
      .expect("requests")
      .iter()
      .filter_map(Value::as_str)
      .collect();
    out.push(format!(
      "C {} [{}] | {}",
      c["name"].as_str().unwrap_or(""),
      requests.join(","),
      c["rule"].as_str().unwrap_or("")
    ));
    for a in c["affiliations"].as_array().expect("affiliations") {
      out.push(item("  a", a, false));
    }
    for a in c["contacts"].as_array().expect("contacts") {
      out.push(item("  c", a, true));
    }
    for a in c["notes"].as_array().expect("notes") {
      out.push(item("  n", a, false));
    }
  }
  for a in resolved["block"].as_array().expect("block") {
    out.push(item("B", a, true));
  }
  for a in resolved["dropped"].as_array().expect("dropped") {
    out.push(item("D", a, true));
  }
  out
}

const REPLACED: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\footnotemark[1]\\ \{FirstName.LastName\}@example.org}
\author{Ann Able\footnotemark[1]}
\renewcommand{\thefootnote}{\fnsymbol{footnote}}
\footnotetext[1]{Univ A, City.}
\maketitle
Text.
\end{document}
"#;
const DOT: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann \textperiodcentered{} Able \and Bob Baker}
\maketitle
Text.
\end{document}
"#;
const ADDRESS_LIST: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able and Bob Baker\\ Univ A, City\\ E-mails: x1@example.org, \\ bob.baker@example.org\thanks{Corresponding author.}}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_5 finding 1: a replaced `\author` call keeps nothing — its block's own items (the pattern address,
/// `owner:pattern`, 2510.26354's first `\author`) go with its creators, and its creators' footnote marks take no text: the
/// `\footnotetext[1]` built before `\maketitle` joins the first mark of its print still standing, the replacing call's
/// (it went to the replaced Ann Able and was deleted with her).
#[test]
fn a_replaced_author_call_keeps_nothing() {
  assert_eq!(tuples(REPLACED), [
    "C Ann Able [*] | line:names",
    "  n Univ A, City. | note:footnotetext-mark",
  ]);
}

/// Review revfms2a2_5 finding 2 (the main session's decision): a resolved name is its printed text — a visible glyph
/// printing no letter or digit stays in it, as the join paths keep it and today's XML prints it (`Ann · Able`; 1706.00234's
/// "TIÊ ´N-SO . N PHẠM", `perfect_kernel_batchfms2a2r5::a_gap_inside_an_accent_macro_is_the_names`).
#[test]
fn a_name_is_its_printed_text() {
  assert_eq!(tuples(DOT), [
    "C Ann · Able [] | line:names",
    "C Bob Baker [] | line:names sep:column",
  ]);
}

/// Review revfms2a2_5 finding 6: an address list running on over the next line places the notes of its lines — the
/// block's, a line of addresses being no one name's (2406.07529's `\thanks` after its last address); the line read on to
/// was skipped and its `\thanks` lost. Since 101b today's reads so too — the line carrying the note an address line
/// (`frontmatter_api::address_line_notes`; it was an affiliation of both names, and `x1@example.org,` a nameless
/// creator's), the note the document's pubnote, where the writer holds the block's (`plan::held_as_pubnote`) — so the
/// writer's outcome is `same` (it held back at Bob Baker's e-mail).
#[test]
fn an_address_lists_lines_place_their_notes() {
  assert_eq!(tuples(ADDRESS_LIST), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | line:unmarked R12:row-above",
    "  c email x1@example.org | owner:order ruling3:one-per-print",
    "C Bob Baker [] | line:names sep:word",
    "  a Univ A, City | line:unmarked R12:row-above",
    "  c email bob.baker@example.org | owner:spelled ruling3:one-per-print",
    "B thanks Corresponding author. | note:address-line",
  ]);
}
