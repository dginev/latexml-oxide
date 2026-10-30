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
  fm_resolve, fm_shadow_json, set_fm_shadow_override,
};
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

/// Review revfms2a2_6 finding 4: a contact placed by `label=author:N` (`\lx@add@contact@placed`'s `ContactOwner::Author`,
/// numbered among the queued authors as `place.rs` numbers the live ones) goes to the N-th live creator — a replaced
/// `\author` call's creators shift no label (`author:2` went to Bob, the second of all, not Cat, the second live one).
#[test]
fn an_author_label_numbers_the_live_creators() {
  let inputs = [
    block(1, "", false, vec![name("Ann Able")]),
    block(2, "", true, vec![
      name("Bob Baker"),
      and(),
      name("Cat Cole"),
    ]),
    email(3, "role=email,label=author:2", "x2@example.org"),
  ];
  assert_eq!(resolved_tuples(&fm_resolve(&inputs)), [
    "C Bob Baker [] | line:names",
    "C Cat Cole [] | line:names sep:column",
    "  c email x2@example.org | api:label",
  ]);
}

const URL: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\thanks{Univ A, City. The code is at \url{https://example.org/code}; Bob's page \url{https://www.bobbaker.org/}.} \and Bob Baker}
\maketitle
Text.
\end{document}
"#;
const URL_GROUP: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\thanks{Univ A, City. Contact: \url{able, baker@example.org}; code: \url{https://example.org/code}.} \and Bob Baker}
\maketitle
Text.
\end{document}
"#;
const EMPTIED: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\thanks{\href{mailto:ann.able@example.org}{ann.able@example.org}, \href{mailto:bob.baker@example.org}{bob.baker@example.org}} \and Bob Baker}
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

/// Review revfms2a2_6 finding 2 (the main session's decision: urls through the spelled-author pass): a url in a note
/// that spells a co-author goes to that author, as an e-mail does — read once every name is (Bob is named after Ann's
/// note), `owner:spelled`; the note keeps its whole text (2001.02600's `http://www.pengxu.net/` in Liang Wang's note is
/// Peng Xu's). The unspelled link stays in the text (decision 2).
#[test]
fn a_notes_url_spelling_a_coauthor_is_theirs() {
  super::perfect_kernel_batchfms2a1::reading_only(|| {
    assert_eq!(tuples(URL), [
      "C Ann Able [] | line:names",
      "  n Univ A, City. The code is at https://example.org/code, Bob’s page https://www.bobbaker.org/ . | \
     note:adjacent",
      "C Bob Baker [] | line:names sep:column",
      "  c url https://www.bobbaker.org/ | note:adjacent ruling4:inner-contact owner:spelled",
    ]);
  });
}

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

/// Review revfms2a2_6 finding 3: whether a note's links emptied it (ruling 4) is decided once every name is read — Ann's
/// note holds Bob's address too, which no name spelled at the note: it was kept beside both addresses made contacts (S2b
/// would print them twice); now dropped, each address its speller's.
#[test]
fn a_notes_links_empty_it_once_every_name_is_read() {
  super::perfect_kernel_batchfms2a1::reading_only(|| {
    assert_eq!(tuples(EMPTIED), [
      "C Ann Able [] | line:names",
      "  c email ann.able@example.org | note:adjacent ruling4:inner-contact",
      "C Bob Baker [] | line:names sep:column",
      "  c email bob.baker@example.org | note:adjacent ruling4:inner-contact owner:spelled",
      "D note ann.able@example.org, bob.baker@example.org | note:adjacent ruling4:emptied",
    ]);
  });
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
  super::perfect_kernel_batchfms2a1::reading_only(|| {
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
  });
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

const GLYPH_LEGEND: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able,$^{1,\#}$ Bob Baker,$^{2,\#}$ Cat Cole$^{2,\star}$\\ $^1$Univ A, City.\\ $^2$Univ B, Town.\\ \normalsize{\# authors contributed equally and ${\star}$ to whom correspondence should be addressed.}}
\maketitle
Text.
\end{document}
"#;

/// Decision 2 of int66 on a legend line: a mark glyph mid-line opens an entry of its own (`${\star}$`), and a legend's
/// text led by a symbol of no `\@fnsymbol` kind is a legend, a note of the names showing it (2401.00056's `\# authors
/// contributed equally and ${\star}$ to whom correspondence should be addressed`, the `⋆` line an affiliation of Pace
/// and Ahrar before). The `#` parted from the line's text is no footnote symbol either: ruling 1's (FM S2a.2 r8, review
/// revfms2a2_7 finding 6 — it was read as ruling 2's symbol line).
#[test]
fn a_mark_glyphs_legend_is_a_note() {
  super::perfect_kernel_batchfms2a1::reading_only(|| {
    assert_eq!(tuples(GLYPH_LEGEND), [
      "C Ann Able [1,#] | line:names mark:after-comma",
      "  a Univ A, City. | ruling1:request",
      "  n authors contributed equally | ruling1:request",
      "C Bob Baker [2,#] | line:names sep:comma mark:after-comma",
      "  a Univ B, Town. | ruling1:request",
      "  n authors contributed equally | ruling1:request",
      "C Cat Cole [2,⋆] | line:names sep:comma",
      "  a Univ B, Town. | ruling1:request",
      "  n to whom correspondence should be addressed. | ruling1:request",
    ]);
  });
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

/// The r7 shapes leave the XML byte-identical under the census switch and in the default mode (the fleet's; shadow
/// only, or written in both — the kernel family since 93a, OD #925) — but `FOOTNOTE_MARK`, a
/// REVTeX paper the writer writes (FM S2b, `perfect_kernel_batchfms2b::footnote_marked_affiliations_go_to_their_names`).
#[test]
fn r7_shapes_leave_the_xml_unchanged() {
  let many = many_thanks();
  for tex in [
    URL,
    URL_GROUP,
    EMPTIED,
    GLYPHS,
    SEPARATORS,
    many.as_str(),
    RUNAWAY_MARK,
    GLYPH_LEGEND,
  ] {
    let run = |on: Option<bool>| {
      convert_with_setup_then(
        tex,
        Some("ar5iv.sty"),
        move || set_fm_shadow_override(on),
        |_| (),
      )
    };
    let (on_log, on_xml, ()) = run(Some(true));
    // (the default, as the fleet runs: a written family's paper written there too)
    let (off_log, off_xml, ()) = run(None);
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
