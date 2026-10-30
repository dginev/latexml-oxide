//! Guards for FM S2a.2 r6: the frontmatter observer's resolver on the r5 landing review's findings (revfms2a2_5) and the
//! main session's decisions 2 and 3 of 2026-10-22 (OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r6"; KERNEL_CAPABILITIES
//! K23), in shadow mode. Each shape was red on r5 (the int63a code, 410a2a85e8): a replaced `\author` call's footnote
//! mark taking the text built before it (finding 1, 2510.26354), a printed glyph dropped from a name (finding 2,
//! 1706.00234), a note's inner links given to its owner whoever they name (decision 2: 2001.02600, 1803.04996,
//! 2504.08644, 2602.13176), a braced `{a,b,c}@x` group in an `\institute` entry given whole to every owner (decision 3),
//! an address list's note never placed (finding 6). Every probe compiles clean with pdflatex (TL 2025) and converts with
//! no Error or Warning, and the XML is byte-identical with the switch on and off.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

/// Convert `tex` under the shadow switch, asserting a clean conversion: the whole dump.
fn dump(tex: &str) -> Value {
  let (log, _, json) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
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
const INNER: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\thanks{Univ A, City. \href{mailto:x1@example.org}{x1@example.org}} \and Bob Baker\thanks{Univ B, Town. The video is at \url{https://example.org/video}, the data at \url{https://example.org/data}; \url{https://www.bobbaker.org/}.} \and Cat Cole\thanks{Univ C. \url{https://example.org/cat}}}
\maketitle
Text.
\end{document}
"#;
const IEEE: &str = r#"\documentclass[journal]{IEEEtran}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able and Bob Baker\thanks{A. Able is with Univ A (homepage \url{https://example.org/ann}).}\thanks{B. Baker is with Univ B (\url{https://www.bobbaker.org/}).}}
\maketitle
Text.
\end{document}
"#;
const GROUP: &str = r#"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{1} \and Cat Cole\inst{2}}
\institute{Univ A, City\\ \email{\{ann.able,bob.baker,info\}@example.org} \and Univ B, Town\\ \email{cole@example.org}}
\maketitle
Text.
\end{document}
"#;
const GROUP_UNREQUESTED: &str = r#"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able \and Bob Baker}
\institute{Univ A, City\\ \email{\{ann.able,bob.baker\}@example.org}}
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

/// Decision 2 (user ruling 2026-10-20 #4, no guessing): a note's inner contact is its owner's when it is the note's only
/// link (Ann's `mailto:`, Cat's url) or spells the owner (`https://www.bobbaker.org/`, `owner:spelled`); the note's other
/// links — the paper's video and data — stay in its text.
#[test]
fn a_notes_links_are_its_owners_by_spelling_or_as_its_only_link() {
  assert_eq!(tuples(INNER), [
    "C Ann Able [] | line:names",
    "  c email x1@example.org | note:adjacent ruling4:inner-contact",
    "  n Univ A, City. x1@example.org | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
    "  c url https://www.bobbaker.org/ | note:adjacent ruling4:inner-contact owner:spelled",
    "  n Univ B, Town. The video is at https://example.org/video, the data at https://example.org/data, \
     https://www.bobbaker.org/ . | note:adjacent",
    "C Cat Cole [] | line:names sep:column",
    "  c url https://example.org/cat | note:adjacent ruling4:inner-contact",
    "  n Univ C. https://example.org/cat | note:adjacent",
  ]);
}

/// Decision 2 under a class printing no note marks (IEEEtran.cls:4756 `\def\thefootnote{}`, `Policy::unmarked_notes`): a
/// note stands beside the last name by its place alone, so even its only link is a name's only by spelling — a
/// co-author's homepage (2001.02600), a video (1803.04996), data records (2504.08644), the co-authors' ORCIDs (2602.13176)
/// all went to the last author. (FM S2a.2 r8, main-session decision 1 on r7: a url is spelled by its page, the last path
/// segment — `example.org/ann` is Ann Able's, its whole letters "exampleorgann" spelled no one.)
#[test]
fn an_unmarked_notes_link_is_a_names_only_by_spelling() {
  let dump = dump(IEEE);
  assert_eq!(
    dump["resolved"]["policies"],
    // (the block is the ledger's second entry, after the title: the title is recorded for its marks)
    serde_json::json!([[2, "IEEEtran"]])
  );
  assert_eq!(tuples(IEEE), [
    "C Ann Able [] | line:names",
    "  c url https://example.org/ann | note:adjacent ruling4:inner-contact owner:spelled",
    "C Bob Baker [] | line:names sep:word",
    "  c url https://www.bobbaker.org/ | note:adjacent ruling4:inner-contact owner:spelled",
    "  n A. Able is with Univ A (homepage https://example.org/ann ). | note:adjacent",
    "  n B. Baker is with Univ B ( https://www.bobbaker.org/ ). | note:adjacent",
  ]);
}

/// Decision 3 (user rulings 2026-10-20 #4, #8, #10): a braced `{a,b,c}@x` group an `\institute` entry's `\email` prints is
/// written out, each address to the author of the document it spells (`ruling3:written-out owner:spelled`), an unspelled
/// one to the block's shared creator (`owner:block`) — not the whole group to every author of the entry; one address
/// stays the entry's owners', as today's.
#[test]
fn an_entrys_address_group_is_written_out_to_its_spellers() {
  assert_eq!(tuples(GROUP), [
    "C Ann Able [1] | line:names",
    "  a Univ A, City | api:labelseq",
    "  c email ann.able@example.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
    "C Bob Baker [1] | line:names",
    "  a Univ A, City | api:labelseq",
    "  c email bob.baker@example.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
    "C Cat Cole [2] | line:names",
    "  a Univ B, Town | api:labelseq",
    "  c email cole@example.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
    "B email info@example.org | api:labelseq api:inner-contact ruling3:written-out owner:block",
  ]);
  // (an entry no name requests too: each address the author's it spells — `\{guyam, osherm, tomz, guykatz,
  // schapiram\}@cs.huji.ac.il` under unmarked names, 2302.05745; the institution, llncs's one `\institute` no `\inst{n}`
  // picks, every author's since 92b, `api:share:authors`: it was the block's, against today's and the PDF)
  assert_eq!(tuples(GROUP_UNREQUESTED), [
    "C Ann Able [] | api:creator",
    "  a Univ A, City | api:labelseq api:share:authors",
    "  c email ann.able@example.org | api:labelseq api:inner-contact ruling3:written-out api:share:authors \
     owner:spelled",
    "C Bob Baker [] | api:creator",
    "  a Univ A, City | api:labelseq api:share:authors",
    "  c email bob.baker@example.org | api:labelseq api:inner-contact ruling3:written-out api:share:authors \
     owner:spelled",
  ]);
}

/// Review revfms2a2_5 finding 6: an address list running on over the next line places the notes of its lines — the
/// block's, a line of addresses being no one name's (2406.07529's `\thanks` after its last address); the line read on to
/// was skipped and its `\thanks` lost.
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

/// The r6 shapes leave the XML byte-identical with the switch on and off (shadow only).
#[test]
fn r6_shapes_leave_the_xml_unchanged() {
  for tex in [
    REPLACED,
    DOT,
    INNER,
    IEEE,
    GROUP,
    GROUP_UNREQUESTED,
    ADDRESS_LIST,
  ] {
    let run = |on: bool| {
      convert_with_setup_then(
        tex,
        Some("ar5iv.sty"),
        move || set_fm_shadow_override(Some(on)),
        |_| (),
      )
    };
    let (on_log, on_xml, ()) = run(true);
    let (off_log, off_xml, ()) = run(false);
    assert_eq!(on_xml, off_xml, "{tex}");
    assert_eq!(
      (error_count(&on_log), warning_count(&on_log)),
      (0, 0),
      "{on_log}"
    );
    assert_eq!(
      (error_count(&off_log), warning_count(&off_log)),
      (0, 0),
      "{off_log}"
    );
  }
}
