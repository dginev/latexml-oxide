//! Guards for FM S2a.2 r7: the frontmatter observer's resolver on the r6 landing review's findings (revfms2a2_6) and the
//! main session's decisions of int66 (OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r7"; KERNEL_CAPABILITIES K23), in shadow
//! mode. Each shape was red on r6 (the int69b code, f24574e043): a class's address list read out of order under a
//! replacing `\author` (finding 1), a url spelling a co-author left in the note's text (finding 2, 2001.02600), a note
//! its links empty kept beside them before the names after it were read (finding 3), `author:N` counting a replaced
//! call's creators (finding 4), a name's mark and separator glyphs read as its ink (decision 2: 0806.2911, 0907.1243,
//! 2003.01448, 2311.00652, 2407.21626, 2511.07187, 2511.18580), and a `\footnote[1]` mark not answered by
//! `\affiliation{\footnotemark[1]…}` (1607.07280); and the note-mark reading's own hazards (eleven `\thanks`, a
//! runaway `\@fnsymbol`, 1105.2659). Every probe compiles clean with pdflatex (TL 2025) — but the eleven `\thanks`,
//! "Counter too large" as LaTeX's `\@fnsymbol` reports it — and converts with no Error or Warning, and the XML is
//! byte-identical with the switch on and off.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{
  FmEvent, FmEventKind, FmInput, FmResolved, FmSepKind, declare_frontmatter_conventions,
  fm_resolve, fm_shadow_json,
};
use serde_json::Value;

/// Convert `tex` under the shadow switch, asserting a clean conversion: the whole dump.
fn dump(tex: &str) -> Value {
  let (log, _, json) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| fm_shadow_json());
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  super::perfect_kernel_batchfms2a1::parsed(&json)
}

/// One association compactly: `<prefix> [<role> ]<text> | <rule>`.
fn item(prefix: &str, role: &str, text: &str, rule: &str, with_role: bool) -> String {
  if with_role {
    format!("{prefix} {role} {text} | {rule}")
  } else {
    format!("{prefix} {text} | {rule}")
  }
}

/// The resolved tuples compactly, as `perfect_kernel_batchfms2a2r6::tuples` writes them: `C <name> [<requests>] <rule>`,
/// then its `a` affiliations, `c <role>` contacts, `n` notes; `B <role>` the block's, `D <role>` what was dropped.
fn tuples(tex: &str) -> Vec<String> {
  let resolved = dump(tex)["resolved"].clone();
  let mut out = Vec::new();
  let json_item = |prefix: &str, a: &Value, role: bool| {
    item(
      prefix,
      a["role"].as_str().unwrap_or(""),
      a["text"].as_str().unwrap_or(""),
      a["rule"].as_str().unwrap_or(""),
      role,
    )
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
      out.push(json_item("  a", a, false));
    }
    for a in c["contacts"].as_array().expect("contacts") {
      out.push(json_item("  c", a, true));
    }
    for a in c["notes"].as_array().expect("notes") {
      out.push(json_item("  n", a, false));
    }
  }
  for a in resolved["block"].as_array().expect("block") {
    out.push(json_item("B", a, true));
  }
  for a in resolved["dropped"].as_array().expect("dropped") {
    out.push(json_item("D", a, true));
  }
  out
}

/// A resolution's tuples, as [`tuples`] writes a dump's.
fn resolved_tuples(resolved: &FmResolved) -> Vec<String> {
  let mut out = Vec::new();
  for c in &resolved.creators {
    out.push(format!(
      "C {} [{}] | {}",
      c.name,
      c.requests.join(","),
      c.rule
    ));
    for a in &c.affiliations {
      out.push(item("  a", &a.role, &a.text, &a.rule, false));
    }
    for a in &c.contacts {
      out.push(item("  c", &a.role, &a.text, &a.rule, true));
    }
    for a in &c.notes {
      out.push(item("  n", &a.role, &a.text, &a.rule, false));
    }
  }
  for a in &resolved.block {
    out.push(item("B", &a.role, &a.text, &a.rule, true));
  }
  out
}

fn name(text: &str) -> FmEvent {
  let mut event = FmEvent::new(FmEventKind::Name);
  event.text = text.to_string();
  event
}

fn and() -> FmEvent { FmEvent::new(FmEventKind::Sep(FmSepKind::And("\\and"))) }

/// An author block's input (`ab_audit fmshadow --re-resolve`'s shape): `replace` the kernel `\author` given again.
fn block(index: usize, binding: &str, replace: bool, events: Vec<FmEvent>) -> FmInput {
  FmInput::Block {
    index,
    binding: binding.to_string(),
    hook: "\\lx@add@authors@append".to_string(),
    keyvals: None,
    counter: None,
    replace,
    cleared: false,
    epoch: 0,
    events,
    calls: Vec::new(),
    markers: Vec::new(),
  }
}

/// An e-mail contact call made apart from the blocks, with its attributes.
fn email(index: usize, attrs: &str, address: &str) -> FmInput {
  FmInput::Api {
    index,
    tag: "ltx:contact".to_string(),
    role: "email".to_string(),
    attrs: attrs.to_string(),
    events: vec![name(address)],
  }
}

/// Review revfms2a2_6 finding 1: under a `SpelledElseOrder` class (jheppub's `\emailAdd` list) a replacing `\author`
/// call's list is read in author order against the live creators — the spelled `ann.able@` is Ann's at place 1, so
/// `x2@` is Bob's by order (`owner:order`); the spelled owner's index into all creators (1: the replaced Ann is 0) was
/// compared with its live place (0), the list read as out of order and `x2@` left to the block.
#[test]
fn a_replacing_calls_address_list_is_read_in_live_order() {
  // (jheppub's conventions, as its binding declares them at load: 87a, OXIDIZED_DESIGN_DIVERGENCES #837)
  declare_frontmatter_conventions(&latexml_package::package::jheppub_sty::FRONTMATTER_CONVENTIONS);
  let inputs = [
    block(1, "jheppub.sty", false, vec![name("Ann Able")]),
    block(2, "jheppub.sty", true, vec![
      name("Ann Able"),
      and(),
      name("Bob Baker"),
    ]),
    email(3, "role=email,_sequence=1", "ann.able@example.org"),
    email(4, "role=email,_sequence=2", "x2@example.org"),
  ];
  assert_eq!(resolved_tuples(&fm_resolve(&inputs)), [
    "C Ann Able [] | line:names",
    "  c email ann.able@example.org | api:sequence owner:spelled",
    "C Bob Baker [] | line:names sep:column",
    "  c email x2@example.org | api:sequence owner:order",
  ]);
}

const URL_GROUP: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\thanks{Univ A, City. Contact: \url{able, baker@example.org}; code: \url{https://example.org/code}.} \and Bob Baker}
\maketitle
Text.
\end{document}
"#;
const GLYPHS: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able$\|$, Bob Baker $\diamond$, Cat Cole$\dagger$$\star$ \\
$\|$ Univ A, City \\
$\diamond$ Univ B, Town \\
$\dagger$ Univ C, Village \\
$\star$ Univ D, Hamlet}
\maketitle
Text.
\end{document}
"#;
const SEPARATORS: &str = r#"\documentclass{article}
\usepackage{url}
\begin{document}
\title{T}
\author{Ann Able $\cdot$ Bob Baker $\cdot$ Cat Cole \and Dan Dole -- \url{dan.dole@example.org} \and Eve Ern$^{1}$ | PhD \and The ABC Collaboration\thanks{Corresponding authors.}: Fay Fox, Gus Gill \and Hal M\"uller--S\'anchez}
\maketitle
Text.
\end{document}
"#;
const FOOTNOTE_MARK: &str = r#"\documentclass[aps,pra,preprint]{revtex4-2}
\begin{document}
\title{T}
\author{Ann Able\footnote[1]{E-mail: ann.able@example.org} and Bob Baker\footnote[2]{E-mail: bob.baker@example.org}}
\affiliation{\footnotemark[1]Univ A, City}
\affiliation{\footnotemark[1]Univ B, Town}
\affiliation{\footnotemark[2]Univ C, Village}
\begin{abstract}
Abstract.
\end{abstract}
\maketitle
Text.
\end{document}
"#;

/// A url printing an address group (local parts before one domain set as a link) is no co-author's by the first spelling
/// it holds: it stays in the note's text, as before r7, unless it spells the name it is on (1407.7882's
/// `guy,medinamo,danar@eng.tau.ac.il`, 1810.11748, 2001.10403 went whole to one author of the group on the first r7 build).
#[test]
fn a_notes_address_group_url_is_no_coauthors() {
  assert_eq!(tuples(URL_GROUP), [
    "C Ann Able [] | line:names",
    "  n Univ A, City. Contact: able, baker@example.org, code: https://example.org/code . | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
  ]);
}

/// Decision 2 of int66 (the lexer reads a name's edge glyphs as the marks they are): the formula `\|` (`∥`) is
/// `\@fnsymbol`'s `‖` (0806.2911's `R.~E.~Hughes$\|$` under `\address{$\|$ The Ohio State University …}`); `⋄` and `⋆`
/// are marks of no footnote-symbol kind (0907.1243's `A. \v Siber $\diamond$`, `R. Podgornik $\dagger$$\star$`) — each
/// answered by its line, not ink of the name and of an unmarked line under every name.
#[test]
fn a_names_mark_glyphs_are_marks() {
  assert_eq!(tuples(GLYPHS), [
    "C Ann Able [‖] | line:names",
    "  a Univ A, City | ruling2:symbol-line",
    "C Bob Baker [⋄] | line:names sep:comma",
    "  a Univ B, Town | ruling1:request",
    "C Cat Cole [†,⋆] | line:names sep:comma",
    "  a Univ C, Village | ruling2:symbol-line ruling1:parted",
    "  a Univ D, Hamlet | ruling1:request ruling1:parted",
  ]);
}

/// Decision 2 of int66 (the lexer reads a separator glyph standing alone as the separator it is): a dot (2511.18580's
/// `$\cdot$` between names), a dash before a contact (2511.07187's `Chiara Simeoni -- \url{…}`), a bar before a degree
/// (2311.00652's `| PhD`, the kernel's OT1 `|` an em dash, the degree the name's qualifier), a colon after a
/// collaboration's note (2407.21626's `MeerKLASS Collaboration\thanks{…}: Matilde …`); a dash inside a name stays as
/// typed (2003.01448's `F.~M\"uller--S\'anchez`: the classify helpers read `--` as a hyphen, hep-ex9811040).
#[test]
fn a_separator_glyph_parts_names() {
  assert_eq!(tuples(SEPARATORS), [
    "C Ann Able [] | line:names",
    "C Bob Baker [] | line:names sep:word",
    "C Cat Cole [] | line:names sep:word",
    "C Dan Dole [] | line:names sep:column",
    "  c email dan.dole@example.org | owner:adjacent",
    "C Eve Ern PhD [1] | line:names name:qualifier sep:column",
    "C The ABC Collaboration [] | line:names sep:column",
    "  n Corresponding authors. | note:adjacent",
    "C Fay Fox [] | line:names sep:word",
    "C Gus Gill [] | line:names sep:comma",
    "C Hal Müller--Sánchez [] | line:names sep:column",
  ]);
}

/// 1607.07280 (r6's revtex4_support sample, wrong): the mark a `\footnote[1]` shows on a name is the request an entry led
/// by `\footnotemark[1]` answers — the same print (`mark:footnote`; ruling 1, the marks shown on a name are its
/// requests); both affiliations were the block's.
#[test]
fn a_notes_mark_is_answered_by_its_footnotemark() {
  assert_eq!(tuples(FOOTNOTE_MARK), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | api:leading-mark mark:footnote",
    "  a Univ B, Town | api:leading-mark mark:footnote",
    "  n E-mail: ann.able@example.org | note:adjacent",
    "C Bob Baker [] | line:names sep:word",
    "  a Univ C, Village | api:leading-mark mark:footnote",
    "  n E-mail: bob.baker@example.org | note:adjacent",
  ]);
}

/// An `\author` of eleven names, each with a `\thanks`.
fn many_thanks() -> String {
  let names: Vec<String> = (0..11)
    .map(|i| format!("Ann{} Able\\thanks{{Note {i}.}}", char::from(b'A' + i)))
    .collect();
  format!(
    "\\documentclass{{article}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{{}}}\n\\maketitle\nText.\n\\end{{document}}\n",
    names.join(" \\and ")
  )
}

/// The note marks the rehearsal reads (`thanks shows=…`, item 6) past `\@fnsymbol`'s nine ("Counter too large", as
/// pdflatex reports it at the tenth `\thanks`; today's conversion raises nothing) set their diagnostics aside: the
/// block's rehearsal is not held (`errors` 0), the tenth and eleventh notes show no mark, the ninth its `‡‡`.
#[test]
fn a_note_mark_past_the_symbols_holds_no_rehearsal() {
  let dump = dump(&many_thanks());
  let block = dump["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .find(|e| e["kind"] == "author")
    .expect("a block");
  assert_eq!(
    (block["status"].as_str(), block["errors"].as_u64()),
    (Some("events"), Some(0))
  );
  let roles: Vec<&str> = block["events"]
    .as_array()
    .expect("events")
    .iter()
    .filter(|e| e["kind"] == "Note")
    .filter_map(|e| e["role"].as_str())
    .collect();
  assert_eq!(roles.len(), 11);
  assert_eq!(roles[..2], ["thanks shows=*", "thanks shows=†"]);
  assert_eq!(roles[8..], ["thanks shows=‡‡", "thanks", "thanks"]);
}

const RUNAWAY_MARK: &str = r#"\documentclass{article}
\makeatletter
\def\@fnsymbol#1{\ifnum\thefootnote=99\hbox{*}\else\textsuperscript{\thefootnote}\fi\relax}
\makeatother
\begin{document}
\title{T}
\author{Ann Able\footnote[3]{Speaker} \and Bob Baker}
Text.
\end{document}
"#;

/// A note's mark whose print runs away under the rehearsal's `\thefootnote` (`\@fnsymbol\c@footnote`, while the
/// document's `\@fnsymbol` reads `\thefootnote` — jpconf.cls:153; 1105.2659's `\footnote[3]{Speaker}` left 12,000
/// conditionals open and its conversion Fatal on the first r7 build): the print is unwound where it stopped, the note
/// shows no mark, the block's rehearsal goes on (Ann Able was lost, a Warning raised). No `\maketitle`: pdflatex never
/// typesets the line, and the loop is the rehearsal's alone.
#[test]
fn a_runaway_note_mark_is_unwound() {
  let dump = dump(RUNAWAY_MARK);
  let block = dump["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .find(|e| e["kind"] == "author")
    .expect("a block");
  assert_eq!(
    (block["status"].as_str(), block["errors"].as_u64()),
    (Some("events"), Some(0))
  );
  assert_eq!(tuples(RUNAWAY_MARK), [
    "C Ann Able [] | line:names",
    "  n Speaker | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
  ]);
}
