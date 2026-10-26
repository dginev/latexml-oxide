//! Guards for FM S2a.2 r4: the frontmatter observer's resolver and lexer on the revtex4_support residual of the r3
//! census, and the landing review's ledger nits (OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r4"; KERNEL_CAPABILITIES
//! K23), in shadow mode. Each shape was red on r3 (the int58a code): a prefix-marked name's notes (0710.4115,
//! gr-qc0606119), an empty entry's run (1804.05750, 1507.03321), a `\lx@affiliation@withsup` line's first superscript
//! (0812.4065, 2106.12584), a `\thanks` inside a superscript (1106.3485), ORCID links printing the name (2602.03244,
//! 2301.02927), the frontmatter's position marker (cond-mat9801109), annotations before any author (2103.04797),
//! today's contact grid before the labels are answered (2311.04097); the ledger's attributes in source order, its
//! policy counter read quietly, the `\ref`-mark commands by name. Every probe compiles clean with pdflatex (TL 2025)
//! and converts with no Error or Warning, and the XML is byte-identical with the switch on and off.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

/// Convert `tex` under the shadow switch, asserting a clean conversion: the whole dump.
fn dump(tex: &str) -> Value { dump_as(tex, None) }

/// [`dump`], of a written family's paper the writer holds back (`declined: <why>`): today's XML kept.
fn dump_as(tex: &str, held: Option<&str>) -> Value {
  let (log, _, json) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  match held {
    Some(why) => super::perfect_kernel_batchfms2a1::parsed_held(&json, why),
    None => super::perfect_kernel_batchfms2a1::parsed(&json),
  }
}

/// The resolved tuples compactly, as `perfect_kernel_batchfms2a2::tuples` writes them: `C <name> [<requests>] <rule>`,
/// then its `a` affiliations, `c <role>` contacts, `n` notes; `B <role>` the block's, `D <role>` what was dropped.
fn tuples(tex: &str) -> Vec<String> { tuples_in(&dump(tex)) }

/// [`tuples`] of a written family's paper the writer holds back, its reading never written (`declined: <why>`).
fn tuples_held(tex: &str, why: &str) -> Vec<String> { tuples_in(&dump_as(tex, Some(why))) }

fn tuples_in(dump: &Value) -> Vec<String> {
  let resolved = dump["resolved"].clone();
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

/// A revtex4-2 document holding `front` (its `\author`-family calls).
fn revtex(front: &str) -> String {
  format!(
    "\\documentclass{{revtex4-2}}\n\\begin{{document}}\n\\title{{T}}\n{front}\n\\maketitle\nText.\n\\end{{document}}\n"
  )
}

const PREFIX_NOTES: &str = r"\author{$^{1}$Ann Able\thanks{E-mail address: ann@example.org}, $^{2}$Bob Baker\thanks{E-mail address: bob@example.org} and $^{1}$Cat Cole}
\affiliation{$^{1}$Univ A, City \\ $^{2}$Univ B, Town}";
const EMPTY_ENTRY: &str = r"\author{Ann Able}
\affiliation{Univ A, City}
\collaboration{The X Collaboration}\noaffiliation
\author{Bob Baker}
\affiliation{Univ B, Town}";
const WITHSUP: &str = r"\author{Ann Able$^{1}$, Bob Baker$^{2}$, and Cat Cole$^{3}$}
\affiliation{$^{1}$Univ A, City \\ Departments of Physics$^{2}$ and Chemistry, Univ B, Town \\ $^{3}$Univ C, Village}";
const SUP_THANKS: &str = r"\author{Ann Able$^{1}$, Bob Baker$^{1,\thanks{Corresponding author, Electronic mail: bob@example.org}}$}
\affiliation{$^{1}$Univ A, City}";
const BEFORE_AUTHORS: &str = r"\affiliation{Univ A, City}
\affiliation{Univ Z, Elsewhere}
\author{Ann Able}
\affiliation{Univ A, City}
\author{Bob Baker}
\affiliation{Univ A, City}";
const GRID: &str = r"\author{Ann Able$^{1}$ and Bob Baker$^{2}$}
\email{first@example.org}
\email{second@example.org}
\affiliation{$^{1}$Univ A, City}
\affiliation{$^{2}$Univ B, Town}";
const ORCID_NAMES: &str = "\\documentclass{revtex4-2}\n\\usepackage{orcidlink}\n\\begin{document}\n\\title{T}\n\\author{\\orcidlinki{Ann Able}{0000-0002-3750-9113}}\n\\affiliation{Univ A, City}\n\\author{\\href{https://orcid.org/0000-0003-4811-7913}{Bob Baker}}\n\\affiliation{Univ B, Town}\n\\maketitle\nText.\n\\end{document}\n";
const FM_MARK: &str = "\\documentclass{revtex4}\n\\begin{document}\n\\title{T}\n\\author{Ann Able}\n\\address{Univ A, City}\n\\address{\n\\begin{minipage}[t]{5.0in}\n\\begin{abstract}\nAn abstract.\n\\end{abstract}\n\\end{minipage}}\n\\maketitle\nText.\n\\end{document}\n";

/// Item 1: a prefix-marked name's notes are its own, as a names line attaches a note to the name before (0710.4115,
/// gr-qc0606119: `$^{1}$José Edgar Madriz Aguilar\thanks{E-mail address: …}`, the notes were lost).
#[test]
fn prefix_marked_names_keep_their_notes() {
  assert_eq!(tuples(&revtex(PREFIX_NOTES)), [
    "C Ann Able [1] | line:names mark:prefix",
    "  a Univ A, City | api:leading-mark",
    "  n E-mail address: ann@example.org | note:adjacent",
    "C Bob Baker [2] | line:names mark:prefix",
    "  a Univ B, Town | api:leading-mark",
    "  n E-mail address: bob@example.org | note:adjacent",
    "C Cat Cole [1] | line:names mark:prefix",
    "  a Univ A, City | api:leading-mark",
  ]);
}

/// Item 1: an empty entry is no contact, but its authors hold its role as today's attaches it, so the next
/// `annotate=new` entry's run stops at them — revtex's `\noaffiliation` under a collaboration (1804.05750), an empty
/// `\affiliation{}` (1507.03321): Univ B is Bob Baker's alone, not the collaboration's too.
#[test]
fn an_empty_entry_holds_its_run() {
  assert_eq!(tuples(&revtex(EMPTY_ENTRY)), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | api:annotate:run",
    "C The X Collaboration [] | line:names",
    "C Bob Baker [] | line:names",
    "  a Univ B, Town | api:annotate:run",
  ]);
}

/// Item 1: a `\lx@affiliation@withsup` line (revtex's marked `\affiliation`, `affiliation_calls`) takes its first
/// superscript as its label wherever it stands (`\lx@sup@setlabel@affiliation`): `Departments of Physics$^{2}$ and
/// Chemistry` is Bob Baker's, as today's (0812.4065; `N$^\circ$` a label no one requests, 2106.12584) — not the last
/// author's by `annotate=`.
#[test]
fn a_withsup_line_takes_its_first_superscript() {
  assert_eq!(tuples(&revtex(WITHSUP)), [
    "C Ann Able [1] | line:names",
    "  a Univ A, City | api:leading-mark",
    "C Bob Baker [2] | line:names sep:comma",
    "  a Departments of Physics and Chemistry, Univ B, Town | api:withsup-mark",
    "C Cat Cole [3] | line:names sep:word",
    "  a Univ C, Village | api:leading-mark",
  ]);
  let entries = dump(&revtex(WITHSUP))["entries"].clone();
  let affiliations: Vec<&Value> = entries
    .as_array()
    .expect("entries")
    .iter()
    .filter(|e| {
      e["attrs"]
        .as_str()
        .is_some_and(|a| a.contains("role=affiliation"))
    })
    .collect();
  // (the count first, so the `all` below is not vacuous: review revfms2a2_4 finding 7)
  assert_eq!(affiliations.len(), 3, "{entries}");
  assert!(
    affiliations.iter().all(|e| e["attrs"]
      .as_str()
      .is_some_and(|a| a.ends_with(",_withsup=true"))),
    "{entries}"
  );
}

/// Item 1: a `\thanks` inside a superscript is a note of its own after the mark, not the mark's text (1106.3485:
/// `Liang-Jian Zou$^{1,\thanks{Correspondence author, …}}$` requested "Correspondence author, …"). Its text is
/// rehearsed in the formula, so its spaces go (the comparator reads letters and digits).
#[test]
fn a_thanks_in_a_superscript_is_a_note() {
  assert_eq!(tuples(&revtex(SUP_THANKS)), [
    "C Ann Able [1] | line:names",
    "  a Univ A, City | api:leading-mark owner:group",
    "C Bob Baker [1] | line:names sep:comma",
    "  a Univ A, City | api:leading-mark owner:group",
    "  n Correspondingauthor, Electronicmail:bob@example.org | note:adjacent",
  ]);
}

/// Item 1: an ORCID link printing the name is the name, with its ORCID — the kernel's `\lx@orcidlink` (orcidlink's
/// `\orcidlinki`, whose iD was read into the name, 2602.03244) and an `\href` to the record (2301.02927, 2312.16289).
#[test]
fn orcid_links_printing_the_name() {
  assert_eq!(tuples(ORCID_NAMES), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | api:annotate:run",
    "  c orcid 0000-0002-3750-9113 | owner:adjacent",
    "C Bob Baker [] | line:names",
    "  a Univ B, Town | api:annotate:run",
    "  c orcid 0000-0003-4811-7913 | owner:adjacent",
  ]);
}

/// Item 1: the frontmatter's position marker `\lx@frontmatter@mark{ltx:abstract}` prints nothing: an abstract begun
/// inside an `\address` read "ltx:abstract" as an affiliation (cond-mat9801109).
#[test]
fn the_frontmatter_marker_prints_nothing() {
  assert_eq!(tuples(FM_MARK), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | api:annotate:run",
  ]);
}

/// Item 1: an annotation of the new authors before any author is an orphan as `relocate_annotations` keeps one: the
/// block's, unless an author holds the same (2103.04797: 17 of its 188 pre-declared `\affiliation`s stay under the
/// names; they were all dropped).
#[test]
fn annotations_before_the_authors_are_the_blocks_unless_held() {
  assert_eq!(tuples(&revtex(BEFORE_AUTHORS)), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | api:annotate:run",
    "C Bob Baker [] | line:names",
    "  a Univ A, City | api:annotate:run",
    "B affiliation Univ Z, Elsewhere | api:no-creator owner:block",
  ]);
}

/// Item 1: today's contact grid (insert.rs `distribute_upfront_contacts`) runs before the labels are answered, so the
/// affiliations a mark places are no part of the pile: two `\email`s under `\author{A$^1$ and B$^2$}` are one each
/// (2311.04097). Where ruling 1 labels an entry today's attaches (an amsart `\address{$^1$…}`, no
/// `\lx@affiliation@withsup`), the entry is in today's pile, which then is no grid (1410.7330's two addresses and
/// three e-mails under three names: the e-mails stay the last author's).
#[test]
fn the_contact_grid_comes_before_the_labels() {
  assert_eq!(tuples(&revtex(GRID)), [
    "C Ann Able [1] | line:names",
    "  a Univ A, City | api:leading-mark",
    "  c email first@example.org | api:annotate:last owner:upfront-grid",
    "C Bob Baker [2] | line:names sep:word",
    "  a Univ B, Town | api:leading-mark",
    "  c email second@example.org | api:annotate:last",
  ]);
  let ams = "\\documentclass{amsart}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{1,2}$, Bob Baker$^{1,2}$ and Cat Cole$^{1,2}$}\n\\address{$^{1}$Univ A, City}\n\\address{$^{2}$Univ B, Town}\n\\email{first@example.org}\n\\email{second@example.org}\n\\email{third@example.org}\n\\maketitle\nText.\n\\end{document}\n";
  assert_eq!(tuples(ams), [
    "C Ann Able [1,2] | line:names",
    "  a Univ A, City | api:leading-mark",
    "  a Univ B, Town | api:leading-mark",
    "C Bob Baker [1,2] | line:names sep:comma",
    "  a Univ A, City | api:leading-mark",
    "  a Univ B, Town | api:leading-mark",
    "C Cat Cole [1,2] | line:names sep:word",
    "  a Univ A, City | api:leading-mark",
    "  a Univ B, Town | api:leading-mark",
    "  c email first@example.org | api:annotate:last",
    "  c email second@example.org | api:annotate:last",
    "  c email third@example.org | api:annotate:last",
  ]);
}

/// Item 1: an empty entry is in today's contact pile too: amsart's `\email{}` before three `\address`/`\email` pairs
/// makes seven contacts on the last of three authors, no grid (1302.1274, 1408.1840, 2009.01013; they were dealt out
/// one pair each).
#[test]
fn an_empty_entry_is_in_todays_pile() {
  let tex = "\\documentclass{amsart}\n\\begin{document}\n\\title{T}\n\\author{Ann Able, Bob Baker and Cat Cole}\n\\email{}\n\\address{Univ A, City}\n\\email{ann@example.org}\n\\address{Univ B, Town}\n\\email{bob@example.org}\n\\address{Univ C, Village}\n\\email{cat@example.org}\n\\maketitle\nText.\n\\end{document}\n";
  assert_eq!(tuples(tex), [
    // (the e-mails each to the author it spells, FIDELITY_CHARTER §4, never by bare position — 82x; the addresses stay
    // in the pile, no grid)
    "C Ann Able [] | line:names",
    "  c email ann@example.org | api:annotate:last owner:spelled",
    "C Bob Baker [] | line:names sep:comma",
    "  c email bob@example.org | api:annotate:last owner:spelled",
    "C Cat Cole [] | line:names sep:word",
    "  a Univ A, City | api:annotate:last",
    "  a Univ B, Town | api:annotate:last",
    "  a Univ C, Village | api:annotate:last",
    "  c email cat@example.org | api:annotate:last",
  ]);
}

/// Item 1: a sole author's orphans made before it are its own, an affiliation too whatever it holds, as
/// `relocate_annotations` places an unlabelled pre-author annotation (2401.15779's `\affiliation{https://ifisr.net}`);
/// the one it holds again is nothing.
#[test]
fn a_sole_authors_annotations_before_it() {
  assert_eq!(
    tuples(&revtex(
      "\\email{ann@example.org}\n\\affiliation{Univ A, City}\n\\affiliation{https://example.org}\n\\author{Ann Able}\n\\affiliation{Univ A, City}"
    )),
    [
      "C Ann Able [] | line:names",
      "  a Univ A, City | api:annotate:run",
      "  a https://example.org | api:no-creator owner:block owner:sole-author",
      "  c email ann@example.org | api:owner:block owner:sole-author",
    ]
  );
}

/// Item 1: a piece of no letter or digit (`${\,\!}^2$`'s spacing) between a separator and the mark after it does not
/// join the marked entries of one call, nor end one (2112.13115: `${\,\!}^1$Dept A\\ and ${\,\!}^2$Dept B`).
#[test]
fn spacing_before_a_mark_parts_the_entries() {
  // (written family, held back: the writer does not cut today's one contact at its `{\,\!}^{2}` mark — OPEN_ROWS
  // "written-family writer gaps"; today's XML keeps "1Department of Mathematics and 2Department of Physics, …" on both)
  assert_eq!(
    tuples_held(
      &revtex(
        "\\author{Ann Able${\\,\\!}^{1,2}$}\n\\author{Bob Baker${\\,\\!}^1$}\n\\affiliation{${\\,\\!}^1$Department of Mathematics\\\\ and ${\\,\\!}^2$Department of Physics, Univ A, City}"
      ),
      "Affiliation departmentofmathematics"
    ),
    // (no zero-width space in a name or an entry: the lexer reads none as ink, and a name keeps no piece printing no
    // letter or digit — review revfms2a2_4 finding 6, FM S2a.2 r5)
    [
      "C Ann Able [1,2] | line:names",
      "  a Department of Mathematics | api:leading-mark api:split-by-marks",
      "  a Department of Physics, Univ A, City | api:leading-mark api:split-by-marks",
      "C Bob Baker [1] | line:names",
      "  a Department of Mathematics | api:leading-mark api:split-by-marks",
    ]
  );
}

/// Item 3 (review revfms2a2_3 nit 8): a mark's `\ref` key is read only off the cross-reference commands
/// (`events::REF_COMMANDS`): `\nameref` prints a title, never a label's mark (its key was read as `ref:x`); `\ref`
/// still is (A&A's `\inst{\ref{…}}`, 1809.08354).
#[test]
fn ref_marks_only_by_the_cross_reference_commands() {
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\textsuperscript{\\href{https://example.org/a}{a}} and Bob Baker\\textsuperscript{\\nameref{x}} and Cat Cole\\textsuperscript{\\ref{x}}\\\\\n\\textsuperscript{a}Univ A}\n\\maketitle\n\\section{Sec}\\label{x}Text.\n\\end{document}\n";
  assert_eq!(tuples(tex), [
    "C Ann Able [a] | line:names",
    "  a Univ A | ruling1:request",
    "C Bob Baker [] | line:names sep:word",
    "C Cat Cole [ref:x] | line:names sep:word",
  ]);
}

/// Item 3 (nit 7): an API call's attributes are written in source order, each pair as given (`get_pairs`):
/// `\altaffiliation[Also at ]{…}` is `\lx@add@contact[annotate=new,role=affiliation,name={#1}]`.
#[test]
fn api_attributes_in_source_order() {
  let tex = revtex(
    "\\author{Ann Able}\n\\altaffiliation[Also at ]{Univ Z, Elsewhere}\n\\affiliation{Univ A, City}",
  );
  let attrs: Vec<String> = dump(&tex)["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .filter(|e| e["tag"] != "ltx:title")
    .filter_map(|e| e["attrs"].as_str().map(str::to_string))
    .collect();
  assert_eq!(attrs, [
    "role=author,annotations=",
    "role=altaffiliation,name=Also at ",
    "role=affiliation,annotate=run",
  ]);
}

/// Item 3 (nit 6): the policy's state counter is read quietly (`lookup_register_quiet`): authblk's `\c@Maxaffil` not a
/// register at the call warned "expected:register" under the switch alone.
#[test]
fn the_policy_counter_is_read_quietly() {
  let tex = "\\documentclass{article}\n\\usepackage{authblk}\n\\begin{document}\n\\title{T}\n\\makeatletter\n\\let\\lx@saved@maxaffil\\c@Maxaffil\n\\let\\c@Maxaffil\\relax\n\\author{Ann Able and Bob Baker}\n\\let\\c@Maxaffil\\lx@saved@maxaffil\n\\makeatother\n\\affil{Univ A, City}\n\\maketitle\nText.\n\\end{document}\n";
  let blocks: Vec<Value> = dump(tex)["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .filter(|e| e["kind"] == "author")
    .cloned()
    .collect();
  assert_eq!(blocks.len(), 1, "{blocks:?}");
}

/// The r4 shapes leave the XML byte-identical with the switch on and off (shadow only).
#[test]
fn r4_shapes_leave_the_xml_unchanged() {
  for tex in [
    revtex(PREFIX_NOTES),
    revtex(EMPTY_ENTRY),
    revtex(WITHSUP),
    revtex(SUP_THANKS),
    revtex(BEFORE_AUTHORS),
    revtex(GRID),
    ORCID_NAMES.to_string(),
    FM_MARK.to_string(),
  ] {
    let run = |on: bool| {
      convert_with_setup_then(
        &tex,
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
