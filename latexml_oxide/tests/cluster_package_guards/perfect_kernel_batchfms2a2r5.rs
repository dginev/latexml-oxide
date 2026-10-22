//! Guards for FM S2a.2 r5: the frontmatter observer's resolver and lexer on the r4 classified sample's wrong papers,
//! the comparator's name-note gap (the text of a `\footnotemark` the name shows) and the r4 landing review's nits
//! (OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r5"; KERNEL_CAPABILITIES K23), in shadow mode. Each shape was red on r4
//! (the int60a code, 3cba36f53d): a note of several marked legends (1802.03043), a glued mark opening an API entry
//! (2202.06129), a `\footnotemark` inside an `\inst` (2404.09857), an address pattern (2006.12418), an acronym row
//! (2609.10596, 1002.2573), a gap inside an accent macro (1706.00234), footnote texts read by their marks (1108.5155,
//! 1405.4582, 1703.09134), breaks inside a braced group (1710.07902, math0610612), an institution's address
//! (1106.0130), a column's first name (2006.13694), ORCID links in a mark and an e-mail (2506.11626, 2402.04675). Every probe compiles clean with pdflatex (TL 2025) and converts
//! with no Error or Warning, and the XML is byte-identical with the switch on and off.
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
  serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}: {json}"))
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

const LEGENDS: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\textsuperscript{1} and Bob Baker\textsuperscript{2}\thanks{\textsuperscript{1}Univ A, City. \textsuperscript{2}Univ B, Town.}}
\maketitle
Text.
\end{document}
"#;
const GLUED: &str = r#"\documentclass[sigconf]{acmart}
\begin{document}
\title{T}
\author{Ann Able$^{1}$}
\author{Bob Baker$^{2}$}
\affiliation{\institution{$^{1}$Univ A, City $^{2}$Univ B, Town}\country{}}
\maketitle
Text.
\end{document}
"#;
const LLNCS: &str = r#"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{2\footnotemark[1]}}
\institute{Univ A, City \and Univ B, Town}
\maketitle
\renewcommand{\thefootnote}{\fnsymbol{footnote}}
\footnotetext[1]{Equal contribution.}
Text.
\end{document}
"#;
const PATTERN: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\thanks{Corresponding author.}, Bob Baker and Cat Cole\\ Univ A, City\\ \{FirstName.LastName\}@example.org}
\maketitle
Text.
\end{document}
"#;
const ACRONYMS: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\\ IBM India, IBM CIO\\ \texttt{ann@example.org}}
\maketitle
Text.
\end{document}
"#;
const ACCENT: &str = r#"\documentclass{amsart}
\def\EES{{\accent"5E E}\kern-.5em\raise.8ex\hbox{\char'23 }}
\def\OW{O\kern-.460em\raise1.36ex\hbox{
\vrule width .13em height .0ex depth .075ex \kern-0.16em \char'56}\kern-.07em}
\begin{document}
\title{T}
\author{TI\EES N-S\OW N PH\d{A}M}
\address{Univ A, City}
\maketitle
Text.
\end{document}
"#;
const NOTETEXTS: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\footnotemark[1] \and Bob Baker\footnotemark[2] \and Cat Cole\footnotemark[2]}
\maketitle
\renewcommand{\thefootnote}{\fnsymbol{footnote}}
\footnotetext[1]{Univ A, City.}
\footnotetext[2]{Univ B, Town.}
Text.
\end{document}
"#;
const NOTETEXTS_BEFORE: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\footnotemark[1], Bob Baker\footnotemark[1] and Cat Cole\footnotemark[2]}
\renewcommand{\thefootnote}{\fnsymbol{footnote}}
\footnotetext[1]{Univ A, City.}
\footnotetext[2]{Univ B, Town.}
\maketitle
Text.
\end{document}
"#;
const GROUP: &str = r#"\documentclass{amsart}
\begin{document}
\title{T}
\author{Ann Able \\ {\em College of X, Univ A,\\ City, Country}}
\author{Bob Baker\\ \textsl{Pure Mathematics\\Univ B\\Town}}
\author{Cat Cole, Pforzheim University of Applied Sciences, Germany}
\maketitle
Text.
\end{document}
"#;
const COLUMNS: &str = r#"\documentclass{amsart}
\begin{document}
\title{T}
\author{Ann Able \and Bob Baker}
\maketitle
Text.
\end{document}
"#;

const UNSHOWN: &str = r#"\documentclass{revtex4-2}
\begin{document}
\title{T}
\author{Ann Able}
\author{Bob Baker}
\affiliation{$^{1}$Univ A, City}
\maketitle
Text.
\end{document}
"#;
const REVERSED: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Able, A., Baker, B. and Della Valle, C.}
\maketitle
Text.
\end{document}
"#;
const CORRESPONDING: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able\thanks{Corresponding author.}, Bob Baker and Cat Cole\\ Univ A, City\\ \{x1, x2\}@example.org}
\maketitle
Text.
\end{document}
"#;
const ADDRESS_LINES: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able and Bob Baker\thanks{Corresponding author}\\ Univ A, City\\ E-mails: x1@example.org, \\ bob.baker@example.org}
\maketitle
Text.
\end{document}
"#;
const ORCIDS: &str = r#"\documentclass{amsart}
\usepackage{orcidlink}
\begin{document}
\title{T}
\author{Ann Able$^{\orcidlink{0000-0002-4300-7507},1}$}
\address{$^{1}$Univ A, City}
\email{ann@example.org}
\author{Bob Baker}
\address{Univ B, Town}
\email{bob@example.org \orcidlink{0000-0003-1680-3425}}
\maketitle
Text.
\end{document}
"#;

/// Item 1: a note holding the marks of several requested labels is the legends of several entries, no one label's: it
/// stays the note of the name before, as today's (IEEEtran's `\IEEEcompsocitemizethanks{\IEEEcompsocthanksitem
/// \Mark{1}College … \IEEEcompsocthanksitem \Mark{2}College …}` was given whole to the authors of mark 1, 1802.03043).
#[test]
fn a_note_of_several_legends_is_no_one_labels() {
  assert_eq!(tuples(LEGENDS), [
    "C Ann Able [1] | line:names",
    "C Bob Baker [2] | line:names sep:word",
    "  n Univ A, City. Univ B, Town. | note:adjacent",
  ]);
}

/// Item 1: a mark glued to the ink before it that labels as the leading mark does, a name right after it, opens the
/// next entry of an API call (`\institution{$^{1}$University of Illinois …, USA $^{2}$Amazon.com Inc, CA, USA}`,
/// 2202.06129: the call's content has no positions to show the space, and the whole went to the first mark's).
#[test]
fn a_glued_mark_opens_the_next_entry() {
  // (an API creator requests the marks its name shows, as a names line's: `\author{Ann Able$^{1}$}` under acmart)
  assert_eq!(tuples(GLUED), [
    "C Ann Able [1] | api:creator",
    "  a Univ A, City | api:leading-mark api:split-by-marks",
    "C Bob Baker [2] | api:creator",
    "  a Univ B, Town | api:leading-mark api:split-by-marks",
  ]);
}

/// Item 1: a `\footnotemark` inside a mark is a mark of its own after it (`Kui Wu\inst{3\footnotemark[1]}` printed
/// "31", parted into 3 and 1, so Kui Wu had Peking University, entry 1 — 2404.09857): its footnote's, never a
/// numbered affiliation's; the parted reading is ruling 1's (`ruling1:parted`), and the footnote's text after
/// `\maketitle` is the name's note by its mark.
#[test]
fn a_footnotemark_inside_a_mark_is_its_own() {
  assert_eq!(tuples(LLNCS), [
    "C Ann Able [1] | line:names",
    "  a Univ A, City | api:labelseq",
    "C Bob Baker [2,*] | line:names",
    "  a Univ B, Town | api:labelseq ruling1:parted",
    "  n Equal contribution. | note:footnotetext-mark ruling1:parted",
  ]);
}

/// Item 1: an address pattern under the names (`\{FirstName.LastName\}@SIT.Fraunhofer.de`, 2006.12418) is no one's own
/// address: the block's, as today's, not the corresponding author's.
#[test]
fn an_address_pattern_is_the_blocks() {
  assert_eq!(tuples(PATTERN), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | line:unmarked R12:row-above",
    "  n Corresponding author. | note:adjacent",
    "C Bob Baker [] | line:names sep:comma",
    "  a Univ A, City | line:unmarked R12:row-above",
    "C Cat Cole [] | line:names sep:word",
    "  a Univ A, City | line:unmarked R12:row-above",
    "B email FirstName.LastName@example.org | owner:pattern ruling3:one-per-print",
  ]);
}

/// Item 1: a row whose pieces hold an acronym reads as institutions — its pieces alone are no evidence of a names row
/// (`Bhanwar Gupta \\ IBM India, IBM CIO`, 2609.10596; `Allianz Investment Management, LMU Munich`, 1002.2573).
#[test]
fn an_acronym_row_is_no_names_row() {
  assert_eq!(tuples(ACRONYMS), [
    "C Ann Able [] | line:names",
    "  a IBM India, IBM CIO | line:unmarked R12:row-above",
    "  c email ann@example.org | owner:spelled ruling3:one-per-print",
  ]);
}

/// Item 1: a gap one command printed inside a word (a Vietnamese accent drawn with `\raise\hbox{…}`, the ink on both
/// its sides running on past it) is no space between two names (`TI\EES N-S\OW N PH\d{A}M`, 1706.00234).
#[test]
fn a_gap_inside_an_accent_macro_is_the_names() {
  assert_eq!(tuples(ACCENT), [
    "C TIÊ ´N-SO . N PHẠM [] | line:names name:layout-joined name:macro-joined",
    "  a Univ A, City | api:annotate:last",
  ]);
}

/// Item 2 (review revfms2a2_4 finding 1): a footnote's text is the note of the names showing its mark, as today's
/// `relocate_footnote` sets it there — made after `\maketitle`, in every name of that mark (1108.5155, 1405.4582);
/// made before it, in the first alone (1703.09134).
#[test]
fn footnote_texts_are_the_notes_of_their_marks() {
  assert_eq!(tuples(NOTETEXTS), [
    "C Ann Able [*] | line:names",
    "  n Univ A, City. | note:footnotetext-mark",
    "C Bob Baker [†] | line:names sep:column",
    "  n Univ B, Town. | note:footnotetext-mark",
    "C Cat Cole [†] | line:names sep:column",
    "  n Univ B, Town. | note:footnotetext-mark",
  ]);
  assert_eq!(tuples(NOTETEXTS_BEFORE), [
    "C Ann Able [*] | line:names",
    "  n Univ A, City. | note:footnotetext-mark",
    "C Bob Baker [*] | line:names sep:comma",
    "C Cat Cole [†] | line:names sep:word",
    "  n Univ B, Town. | note:footnotetext-mark",
  ]);
  // (the dump records each `\footnotetext` with its printed mark and the flushes before it)
  let texts: Vec<(u64, String)> = dump(NOTETEXTS_BEFORE)["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .filter(|e| e["kind"] == "notetext")
    .map(|e| {
      (
        e["epoch"].as_u64().unwrap_or(9),
        e["mark"].as_str().unwrap_or("").to_string(),
      )
    })
    .collect();
  assert_eq!(texts, [(0, "*".to_string()), (0, "†".to_string())]);
}

/// Item 5 (ams_support's residual): lines parted by a break inside a braced group or argument are one print unit, as
/// today's one contact (`Xuhui Peng \\ {\em\tiny College …,\\ Changsha, …}`, 1710.07902; `\textsl{Pure
/// Mathematics\\University of Waterloo\\…}`, math0610612); a piece after an institution's comma that reads as no name
/// is its address (`Frank Schadt, Pforzheim University of Applied Sciences, Germany`, 1106.0130).
#[test]
fn group_lines_and_an_institutions_address() {
  assert_eq!(tuples(GROUP), [
    "C Ann Able [] | line:names",
    "  a College of X, Univ A, City, Country | line:unmarked R12:row-above line:continued",
    "C Bob Baker [] | line:names",
    "  a Pure Mathematics Univ B Town | line:unmarked R12:row-above line:continued",
    "C Cat Cole [] | line:names",
    "  a Pforzheim University of Applied Sciences, Germany | line:names classify:institution",
  ]);
}

/// Item 5: a later column's first name is parted by the `\and` that opened the column (`sep:column`: the comparator's
/// list-of-names class, `Chris Kapulkin \arxivortac{\and}{and} Peter LeFanu Lumsdaine`, 2006.13694).
#[test]
fn a_columns_first_name_is_parted_by_its_and() {
  assert_eq!(tuples(COLUMNS), [
    "C Ann Able [] | line:names",
    "C Bob Baker [] | line:names sep:column",
  ]);
}

/// Item 5: an ORCID link inside a mark's script is the name's ORCID, not the mark's text
/// (`Martin Bachratý$^{\orcidlink{0000-0002-4300-7507},1,\ast}$`, 2506.11626: the ORCID was lost), and one inside an
/// e-mail call a contact of its own (`\email{giulio.pascale@unina.it \orcidlink{…}}`, 2402.04675: the address read
/// "giulio.pascale@unina.it0000-0003-1680-3425").
#[test]
fn orcid_links_in_a_mark_and_an_email() {
  assert_eq!(tuples(ORCIDS), [
    "C Ann Able [1] | line:names",
    "  a Univ A, City | api:leading-mark",
    "  c orcid 0000-0002-4300-7507 | owner:adjacent",
    "  c email ann@example.org | api:annotate:last",
    "C Bob Baker [] | line:names",
    "  a Univ B, Town | api:annotate:last",
    "  c email bob@example.org | api:annotate:last",
    "  c orcid 0000-0003-1680-3425 | api:annotate:last api:inner-contact",
  ]);
}

/// Item 1, the r5 sample's wrong classes (round two): an entry labelled by a mark of a kind no name shows is where its
/// declared `annotate=` run put it, as today's (`$^{1}$School of Physics, Liaoning University` under unmarked names went
/// to the block, 2407.20481, 2005.13493); names written surname first are one name each (`Santucho, V., Luparello, H.
/// E.`, 2005.00053; 1503.08374, 1609.07297's `Della Valle, M.`); an address left over in a print of several is no
/// corresponding author's (`{hong3924.cs10, evan20010126.cs12, …}@nycu.edu.tw`, 2512.08223).
#[test]
fn unshown_labels_reversed_names_and_address_prints() {
  assert_eq!(tuples(UNSHOWN), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | api:leading-mark label:unshown owner:annotate",
    "C Bob Baker [] | line:names",
    "  a Univ A, City | api:leading-mark label:unshown owner:annotate",
  ]);
  assert_eq!(tuples(REVERSED), [
    "C Able, A. [] | line:names name:reversed",
    "C Baker, B. [] | line:names sep:comma name:reversed",
    "C Della Valle, C. [] | line:names sep:word name:reversed",
  ]);
  assert_eq!(tuples(CORRESPONDING), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | line:unmarked R12:row-above",
    "  n Corresponding author. | note:adjacent",
    "C Bob Baker [] | line:names sep:comma",
    "  a Univ A, City | line:unmarked R12:row-above",
    "C Cat Cole [] | line:names sep:word",
    "  a Univ A, City | line:unmarked R12:row-above",
    "B email x1@example.org | owner:block ruling3:one-per-print",
    "B email x2@example.org | owner:block ruling3:one-per-print",
  ]);
  // (an address list running on over the next line is one print: `E-mails: x1@…, \\ bob.baker@…` is in author order, not
  // the corresponding author's first address, 2312.05446)
  assert_eq!(tuples(ADDRESS_LINES), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | line:unmarked R12:row-above",
    "  c email x1@example.org | owner:order ruling3:one-per-print",
    "C Bob Baker [] | line:names sep:word",
    "  a Univ A, City | line:unmarked R12:row-above",
    "  c email bob.baker@example.org | owner:spelled ruling3:one-per-print",
    "  n Corresponding author | note:adjacent",
  ]);
}

/// The r5 shapes leave the XML byte-identical with the switch on and off (shadow only).
#[test]
fn r5_shapes_leave_the_xml_unchanged() {
  for tex in [
    LEGENDS,
    GLUED,
    LLNCS,
    PATTERN,
    ACRONYMS,
    ACCENT,
    NOTETEXTS,
    NOTETEXTS_BEFORE,
    GROUP,
    COLUMNS,
    ORCIDS,
    UNSHOWN,
    REVERSED,
    CORRESPONDING,
    ADDRESS_LINES,
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
