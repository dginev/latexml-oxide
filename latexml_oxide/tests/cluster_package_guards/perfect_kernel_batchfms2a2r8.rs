//! Guards for FM S2a.2 r8: the frontmatter observer's resolver on the r7 landing review's findings (revfms2a2_7) and the
//! main session's decisions on r7's questions (OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r8"). Each shape
//! was red on r7 (458382f335): a note placed once every name is read came after the later ones (finding 1), a note no
//! name holds gave its only link to the block (finding 2, decision 4), a repository or a file under an author's account
//! became that author's url (finding 3, decision 1: 2008.12970, 2511.08416, 2103.05251, 2503.02954, 2509.01952),
//! jpconf's note marks read as the kernel's `\@fnsymbol` and a bare `\footnotemark` ending the rehearsal (finding 4:
//! 1105.2659), a glued symbol run a line leads with read as its two marks (decision 2: 0907.1243), a mark glyph set in
//! text read as a footnote symbol (finding 6, decision 3) and an API entry's legend read as an affiliation (finding 8:
//! 0812.1531). Every probe compiles clean with pdflatex (TL 2025) and converts with no Error or Warning, and the XML is
//! byte-identical with the switch on and off.
use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{
  FmEvent, FmEventKind, FmInput, FmResolved, fm_reading_json, fm_resolve,
};
use serde_json::Value;

/// Convert `tex` (beside `files`), asserting a clean conversion: the whole dump.
fn dump_with(tex: &str, files: &[(&str, &str)]) -> Value {
  let (log, _, json) =
    convert_files_with_setup_then(tex, files, Some("ar5iv.sty"), || {}, |_| fm_reading_json());
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

/// A dump's resolved tuples compactly, as `perfect_kernel_batchfms2a2r7::tuples` writes them, in their order: `C <name>
/// [<requests>] <rule>`, then its `a` affiliations, `c <role>` contacts, `n` notes; `B <role>` the block's, `D <role>`
/// what was dropped.
fn dump_tuples(dump: &Value) -> Vec<String> {
  let resolved = &dump["resolved"];
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

fn tuples(tex: &str) -> Vec<String> { dump_tuples(&dump_with(tex, &[])) }

/// A resolution's tuples, as [`dump_tuples`] writes a dump's.
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

/// The dump's author block entry.
fn block_entry(dump: &Value) -> &Value {
  dump["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .find(|e| e["kind"] == "author")
    .expect("a block")
}

const OWNERLESS_ONE: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able \and Bob Baker\\ ann@x.org, bob@x.org\thanks{Code: \url{https://example.org/proj/tool}}}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_7 finding 2 (main-session decision 4 on r7): a note no name holds — one on an address line — has
/// no owner to fall back to: its only link went to the block as a url contact (and on to a sole author by
/// `sole_author_takes_the_block`, 2103.05251's shape); now it stays in the note's text, and a link spelling an author
/// is that author's. The note itself stays the block's (a sole author's). Since 101b today's reads the address line
/// carrying the note as one too (`frontmatter_api::address_line_notes`; it was an affiliation, the writer held back at
/// the first e-mail), its note the document's pubnote, where the writer holds the block's (`plan::held_as_pubnote`):
/// the one-link shape reads as today's; the writer still holds back the others, at Ann's page (a url contact today's
/// leaves in the note) and at the sole author's note (today's the document's).
#[test]
fn an_ownerless_notes_link_is_never_the_blocks() {
  assert_eq!(tuples(OWNERLESS_ONE), [
    "C Ann Able [] | line:names",
    "  c email ann@x.org | owner:spelled ruling3:one-per-print",
    "C Bob Baker [] | line:names sep:column",
    "  c email bob@x.org | owner:spelled ruling3:one-per-print",
    "B thanks Code: https://example.org/proj/tool | note:address-line",
  ]);
}

const JPCONF_CLASS: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jpconf_copy/jpconf.cls"
);
const JPCONF: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jpconf_copy/observer_jpconf_note_marks.tex"
);
const JPFNX: &str = r#"\documentclass{jpfnx}
\begin{document}
\title{T}
\author{Ann Able\footnotemark, Bob Baker}
\address{Univ A, City}
\begin{abstract}Abs.\end{abstract}
\section{S} Body.
\end{document}
"#;

/// Review revfms2a2_7 finding 4: jpconf prints its notes' marks as the body's arabic `\thefootnote` (jpconf.cls:139,
/// 153), its declared conventions' `TitleMarks::Body` — 1105.2659's `\footnote[3]{Speaker}` shows "3", as its PDF, and a bare
/// `\footnotemark` "1"; under the kernel's `\@fnsymbol\c@footnote` the bare `\footnotemark` ran away and the block's
/// rehearsal ended in a Fatal error, the block unobserved.
#[test]
fn jpconfs_note_marks_are_its_arabic_numbers() {
  let dump = dump_with(JPCONF, &[("jpconf.cls", JPCONF_CLASS)]);
  let block = block_entry(&dump);
  assert_eq!(
    (block["status"].as_str(), block["errors"].as_u64()),
    (Some("events"), Some(0))
  );
  let marks: Vec<(&str, &str)> = block["events"]
    .as_array()
    .expect("events")
    .iter()
    .filter(|e| {
      e["kind"] == "Note"
        || e["role"]
          .as_str()
          .is_some_and(|r| r.starts_with("footnotemark"))
    })
    .map(|e| {
      (
        e["kind"].as_str().unwrap_or(""),
        e["role"].as_str().unwrap_or(""),
      )
    })
    .collect();
  assert_eq!(marks, [
    ("Mark", "footnotemark"),
    ("Note", "thanks mark=3 shows=3")
  ]);
  assert_eq!(dump_tuples(&dump), [
    "C Ann Able [a,1] | line:names",
    "  a Univ A, City | api:leading-mark",
    "C Bob Baker [b] | line:names sep:comma",
    "  a Univ B, Town | api:leading-mark",
    "  n Speaker | note:adjacent",
    "C Cat Cole [] | line:names sep:word",
  ]);
}

/// Review revfms2a2_7 finding 4's residual: a bare `\footnotemark`'s print is read quietly as a note's is — under a
/// class declaring no conventions whose `\@fnsymbol` reads `\thefootnote` (the jpconf stand-in renamed: read raw), it
/// ran away and the block's rehearsal ended in a Fatal error; now the mark shows nothing and the block is read. The
/// raw class's in-place `\address` (jpconf.cls's `\begin{indented}\item[]…`) is the API's since 94b (OD #936): its
/// unmarked line is both names' (`annotate=run`), as pdflatex prints it under them.
#[test]
fn a_runaway_footnotemark_is_unwound() {
  let class = JPCONF_CLASS.replace("\\ProvidesClass{jpconf}", "\\ProvidesClass{jpfnx}");
  let dump = dump_with(JPFNX, &[("jpfnx.cls", class.as_str())]);
  let block = block_entry(&dump);
  assert_eq!(
    (block["status"].as_str(), block["errors"].as_u64()),
    (Some("events"), Some(0))
  );
  assert_eq!(dump_tuples(&dump), [
    "C Ann Able [] | line:names",
    "  a Univ A, City | api:annotate:run",
    "C Bob Baker [] | line:names sep:comma",
    "  a Univ A, City | api:annotate:run",
  ]);
}

const GLUED: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able$^{\S}$, Bob Baker$^{\S\dagger}$, Cat Cole$^{\dagger}$, Dan Dole$^{\dagger\star}$ \\
$^{\dagger}$Univ A, City \\
$^{\star}$Univ B, Town \\
$^{\S}$Univ C, Village \\
$^{\S\dagger}$Univ D, Hamlet}
\maketitle
Text.
\end{document}
"#;

/// Main-session decision 2 on r7: a glued symbol run a line leads with and a name shows (`§†`) is one mark, as `††` is
/// — 0907.1243's Ching (`$^{\S\dagger}$`) has only the Missouri line, French (`§`) only DuPont; the `†` and `§` lines
/// went to every carrier of either symbol. A run no line leads with (`†⋆`) stays the marks it is written of.
#[test]
fn a_glued_symbol_run_a_line_leads_with_is_one_mark() {
  assert_eq!(tuples(GLUED), [
    "C Ann Able [§] | line:names",
    "  a Univ C, Village | ruling2:symbol-line",
    "C Bob Baker [§†] | line:names sep:comma",
    "  a Univ D, Hamlet | ruling2:symbol-line",
    "C Cat Cole [†] | line:names sep:comma",
    "  a Univ A, City | ruling2:symbol-line",
    "C Dan Dole [†,⋆] | line:names sep:comma",
    "  a Univ A, City | ruling2:symbol-line ruling1:parted",
    "  a Univ B, Town | ruling1:request ruling1:parted",
  ]);
}

const API_LEGEND: &str = r#"\documentclass[aps,prl]{revtex4-2}
\begin{document}
\title{T}
\author{Ann Able$^{1}$, Bob Baker$^{1,\star}$}
\affiliation{$^1$Univ A, City \\ $^\star${corresponding author, e-mail: bob@x.org}}
\begin{abstract}
Abstract.
\end{abstract}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_7 finding 8: an API entry a symbol leads whose text is a legend is that legend, a note of the names
/// showing the mark, as a block's labelled line reads it — 0812.1531's `$^\star${corresponding author, e-mail: …}` in
/// revtex's `\affiliation` was Curtarolo's affiliation (the role change tagged `api:legend`: FM S2a.2 r9, review
/// revfms2a2_8 finding 2).
#[test]
fn an_api_entrys_legend_is_a_note() {
  assert_eq!(tuples(API_LEGEND), [
    "C Ann Able [1] | line:names",
    "  a Univ A, City | api:leading-mark",
    "C Bob Baker [1,⋆] | line:names sep:comma",
    "  a Univ A, City | api:leading-mark",
    "  n corresponding author, e-mail: bob@x.org | api:leading-mark api:legend",
  ]);
}

const TEXT_GLYPH: &str = r#"\documentclass{article}
\begin{document}
\title{T}
\author{Ann Able$^{1,\#}$, Bob Baker$^{1}$\\ $^1$Univ A, City.\\ \# Corresponding author.}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_7 finding 6 (main-session decision 3 on r7): a mark the resolver parts from a line's text (`#`,
/// `split_text_marks`) is a footnote symbol only when its characters are `\@fnsymbol`'s — `#` is a mark of no such kind,
/// its legend ruling 1's (`ruling1:request`, the names showing it), not ruling 2's symbol line. Written since 97c (the
/// writer cuts a block entry by its whole span, OD #945; it had declined, "Univ A, City." and the legend one affiliation
/// of both names): the XML pinned whole, the legend Ann's note.
#[test]
fn a_text_mark_glyph_is_no_footnote_symbol() {
  assert_eq!(tuples(TEXT_GLYPH), [
    "C Ann Able [1,#] | line:names",
    "  a Univ A, City. | ruling1:request",
    "  n Corresponding author. | ruling1:request",
    "C Bob Baker [1] | line:names sep:comma",
    "  a Univ A, City. | ruling1:request",
  ]);
  assert_eq!(
    super::perfect_kernel_batch93a::default_mode_creators(TEXT_GLYPH),
    super::perfect_kernel_batch93a::owned(&[
      &[
        "Ann Able",
        "affiliation=Univ A, City.",
        "note=# Corresponding author."
      ],
      &["Bob Baker", "affiliation=Univ A, City."],
    ])
  );
}

fn name(text: &str) -> FmEvent {
  let mut event = FmEvent::new(FmEventKind::Name);
  event.text = text.to_string();
  event
}

fn mark(label: &str, line_first: bool) -> FmEvent {
  let mut event = FmEvent::new(FmEventKind::Mark);
  event.text = label.to_string();
  event.label = Some(label.to_string());
  event.line_first = line_first;
  event
}

/// Review revfms2a2_7 finding 7: an entry no label leads (a mark printing nothing) has no symbol, so no legend — `.all()`
/// of no labels was true, and `Corresponding author.` under it a legend note of the block.
#[test]
fn an_unlabelled_entry_is_no_legend() {
  let inputs = [FmInput::Block {
    index:   1,
    binding: String::new(),
    hook:    "\\lx@add@authors@append".to_string(),
    keyvals: None,
    counter: None,
    replace: false,
    cleared: false,
    epoch:   0,
    events:  vec![
      name("Ann Able"),
      mark("1", false),
      FmEvent::new(FmEventKind::Break),
      mark("", true),
      name("Corresponding author."),
    ],
    calls:   Vec::new(),
    markers: Vec::new(),
  }];
  let tuples = resolved_tuples(&fm_resolve(&inputs));
  assert_eq!(tuples, [
    "C Ann Able [1] | line:names",
    "  a Corresponding author. | ruling1:request label:unrequested owner:block owner:sole-author",
  ]);
}
