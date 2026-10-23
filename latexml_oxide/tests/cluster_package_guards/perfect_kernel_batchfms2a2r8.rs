//! Guards for FM S2a.2 r8: the frontmatter observer's resolver on the r7 landing review's findings (revfms2a2_7) and the
//! main session's decisions on r7's questions (OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r8"), in shadow mode. Each shape
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
  FmEvent, FmEventKind, FmInput, FmResolved, fm_resolve, fm_shadow_json, set_fm_shadow_override,
};
use serde_json::Value;

/// Convert `tex` (beside `files`) under the shadow switch, asserting a clean conversion: the whole dump.
fn dump_with(tex: &str, files: &[(&str, &str)]) -> Value {
  let (log, _, json) = convert_files_with_setup_then(
    tex,
    files,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}: {json}"))
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

const ORDER: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\thanks{\url{ann@x.org}, \url{zed@y.org}}\thanks{Second note.} \and Bob Baker}
\maketitle
Text.
\end{document}
"#;
const ORDER_CONTACTS: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\thanks{\url{https://www.annable.org/}} \\ ann.able@y.org}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_7 finding 1: a note whose links might empty it is decided once every name is read
/// (`answer_note_links`), and it came after the notes the source gives later — S2b would print Ann's notes reversed
/// (`[Second note., ann@x.org, zed@y.org]`, int69b `[ann…, Second note.]`). A creator's notes and contacts, and the
/// block's, are in source order; the second shape's homepage before the address line under it.
#[test]
fn a_creators_notes_and_contacts_are_in_source_order() {
  assert_eq!(tuples(ORDER), [
    "C Ann Able [] | line:names",
    "  c email ann@x.org | note:adjacent ruling4:inner-contact",
    "  n ann@x.org, zed@y.org | note:adjacent",
    "  n Second note. | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
  ]);
  assert_eq!(tuples(ORDER_CONTACTS), [
    "C Ann Able [] | line:names",
    "  c url https://www.annable.org/ | note:adjacent ruling4:inner-contact",
    "  c email ann.able@y.org | owner:spelled ruling3:one-per-print",
    "D note https://www.annable.org/ | note:adjacent ruling4:emptied",
  ]);
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
const OWNERLESS: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able \and Bob Baker\\ ann@x.org, bob@x.org\thanks{Code: \url{https://example.org/proj/tool}; Ann's page: \url{https://www.annable.org/}}}
\maketitle
Text.
\end{document}
"#;
const OWNERLESS_SOLE: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\\ ann@x.org\thanks{Code: \url{https://github.com/annable/tool}}}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_7 finding 2 (main-session decision 4 on r7): a note no name holds — one on an address line — has
/// no owner to fall back to: its only link went to the block as a url contact (and on to a sole author by
/// `sole_author_takes_the_block`, 2103.05251's shape); now it stays in the note's text, and a link spelling an author
/// is that author's. The note itself stays the block's (a sole author's).
#[test]
fn an_ownerless_notes_link_is_never_the_blocks() {
  assert_eq!(tuples(OWNERLESS_ONE), [
    "C Ann Able [] | line:names",
    "  c email ann@x.org | owner:spelled ruling3:one-per-print",
    "C Bob Baker [] | line:names sep:column",
    "  c email bob@x.org | owner:spelled ruling3:one-per-print",
    "B thanks Code: https://example.org/proj/tool | note:address-line",
  ]);
  assert_eq!(tuples(OWNERLESS), [
    "C Ann Able [] | line:names",
    "  c email ann@x.org | owner:spelled ruling3:one-per-print",
    "  c url https://www.annable.org/ | note:address-line ruling4:inner-contact owner:spelled",
    "C Bob Baker [] | line:names sep:column",
    "  c email bob@x.org | owner:spelled ruling3:one-per-print",
    "B thanks Code: https://example.org/proj/tool, Ann’s page: https://www.annable.org/ | note:address-line",
  ]);
  assert_eq!(tuples(OWNERLESS_SOLE), [
    "C Ann Able [] | line:names",
    "  c email ann@x.org | owner:spelled ruling3:one-per-print",
    "  n Code: https://github.com/annable/tool | note:address-line owner:sole-author",
  ]);
}

const HOMEPAGE: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Ann Able\thanks{Code: \url{https://github.com/annable/tool}. Pages: \url{https://www.bobbaker.org/}, \url{https://sites.google.com/view/catcole}, \url{https://cole.github.io/project}, \url{https://example.org/~dole/index.html}, \url{https://example.org/dandole/talk.mp4}, \href{mailto:dan.dole@x.org}{dan.dole}.} \and Bob Baker \and Cat Cole \and Dan Dole}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_7 finding 3 (main-session decision 1 on r7): a url goes to the author it spells only when its page
/// spells them — its host, or its last path segment with at most an `index.*`/`*.html` page after it: Bob's host, Cat's
/// `sites.google.com/view/catcole`, Dan's `~dole/index.html`; a repository under Ann's account
/// (`github.com/annable/tool`: 2008.12970, 2511.08416), a project under Cat's (`cole.github.io/project`: 2503.02954) and
/// a file under Dan's (`dandole/talk.mp4`: 2509.01952) stay in the note's text. A link to an address is the address
/// (`\href{mailto:…}{dan.dole}`, 2309.10007's shape): Dan's by its letters — an e-mail, read by the address it points to
/// (FM S2a.2 r9, review revfms2a2_8 finding 1: it was a url read by its anchor; `perfect_kernel_batchfms2a2r9::
/// a_link_contact_carries_its_target` pins the target).
#[test]
fn a_notes_url_is_an_authors_only_as_their_page() {
  assert_eq!(tuples(HOMEPAGE), [
    "C Ann Able [] | line:names",
    "  n Code: https://github.com/annable/tool . Pages: https://www.bobbaker.org/, \
     https://sites.google.com/view/catcole, https://cole.github.io/project, https://example.org/~dole/index.html, \
     https://example.org/dandole/talk.mp4, dan.dole . | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
    "  c url https://www.bobbaker.org/ | note:adjacent ruling4:inner-contact owner:spelled",
    "C Cat Cole [] | line:names sep:column",
    "  c url https://sites.google.com/view/catcole | note:adjacent ruling4:inner-contact owner:spelled",
    "C Dan Dole [] | line:names sep:column",
    "  c url https://example.org/~dole/index.html | note:adjacent ruling4:inner-contact owner:spelled",
    "  c email dan.dole | note:adjacent ruling4:inner-contact owner:spelled",
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
/// 153), its policy entry `TitleMarks::Body` — 1105.2659's `\footnote[3]{Speaker}` shows "3", as its PDF, and a bare
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
/// class with no policy entry whose `\@fnsymbol` reads `\thefootnote` (the jpconf stand-in renamed: read raw, its
/// `\address` no frontmatter call), it ran away and the block's rehearsal ended in a Fatal error; now the mark shows
/// nothing and the block is read.
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
    "C Bob Baker [] | line:names sep:comma",
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
    "  a Univ A, City | ruling2:symbol-line",
    "  a Univ B, Town | ruling1:request",
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
    "  a Univ A, City | api:leading-mark api:split-by-marks",
    "C Bob Baker [1,⋆] | line:names sep:comma",
    "  a Univ A, City | api:leading-mark api:split-by-marks",
    "  n corresponding author, e-mail: bob@x.org | api:leading-mark api:split-by-marks api:legend",
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
/// its legend ruling 1's (`ruling1:request`, the names showing it), not ruling 2's symbol line.
#[test]
fn a_text_mark_glyph_is_no_footnote_symbol() {
  assert_eq!(tuples(TEXT_GLYPH), [
    "C Ann Able [1,#] | line:names",
    "  a Univ A, City. | ruling1:request",
    "  n Corresponding author. | ruling1:request",
    "C Bob Baker [1] | line:names sep:comma",
    "  a Univ A, City. | ruling1:request",
  ]);
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
    epoch:   0,
    events:  vec![
      name("Ann Able"),
      mark("1", false),
      FmEvent::new(FmEventKind::Break),
      mark("", true),
      name("Corresponding author."),
    ],
  }];
  let tuples = resolved_tuples(&fm_resolve(&inputs));
  assert!(tuples.iter().all(|t| !t.contains(" legend ")), "{tuples:?}");
}

/// The r8 shapes leave the XML byte-identical with the switch on and off (shadow only) — but `API_LEGEND`, a REVTeX
/// paper the writer writes (FM S2b, `perfect_kernel_batchfms2b::a_marked_affiliation_is_cut_at_its_marks`).
#[test]
fn r8_shapes_leave_the_xml_unchanged() {
  let jpfnx = JPCONF_CLASS.replace("\\ProvidesClass{jpconf}", "\\ProvidesClass{jpfnx}");
  let shapes: [(&str, Vec<(&str, &str)>); 10] = [
    (ORDER, vec![]),
    (ORDER_CONTACTS, vec![]),
    (OWNERLESS_ONE, vec![]),
    (OWNERLESS, vec![]),
    (OWNERLESS_SOLE, vec![]),
    (HOMEPAGE, vec![]),
    (JPCONF, vec![("jpconf.cls", JPCONF_CLASS)]),
    (JPFNX, vec![("jpfnx.cls", jpfnx.as_str())]),
    (GLUED, vec![]),
    (TEXT_GLYPH, vec![]),
  ];
  for (tex, files) in shapes {
    let run = |on: bool| {
      convert_files_with_setup_then(
        tex,
        &files,
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
