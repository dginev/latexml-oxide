//! Guards for batch FM S2a.1: the frontmatter observer core (OXIDIZED_DESIGN_DIVERGENCES #837; KERNEL_CAPABILITIES
//! K23). Each `\author`-family call is a ledger block whose stamped tokens are rehearsed under the Layer-B event bindings
//! and lexed into events (read here through the ledger's JSON, `fm_reading_json`); the guards assert those event lists
//! for the design probes (symbol lines, a user mark macro), the gretsi shared-mark repro, 2401.16957's `,\inst` shape and
//! the SYNC frontmatter rows' shapes, and that the rehearsal leaves nothing behind. (The census switch
//! `LATEXML_FM_SHADOW` and the CLI's `<dest>.fmshadow.json` went in DEL4.) Every probe compiles clean with pdflatex
//! (TL 2025) and converts with no Error or Warning.
use std::cell::Cell;

use latexml::util::test::{
  convert_files_with_setup_then, convert_with_setup_then, error_count, warning_count,
};
use latexml_engine::base_utilities::fm_reading_json;
use serde_json::Value;

/// The reading's JSON of a conversion, parsed; for a written family's paper (revtex, ams) the writer must have kept the
/// reading — its outcome `same` (today's reads so already) or `written` — so every reading a written family's guard
/// pins is the XML users see: `declined: …` (a difference no writer operation explains, today's kept) or
/// `written-different: …` (the writer's own defect) fails the guard (test audit 2026-10-26).
pub(super) fn parsed(json: &str) -> Value {
  let value: Value = serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}: {json}"));
  if let Some(outcome) = value["writer"].as_str() {
    match HELD.with(Cell::get) {
      Some(why) => assert!(
        outcome.starts_with(&format!("declined: {why}")),
        "the writer's outcome for a written family's paper held back for {why}: {outcome}"
      ),
      None => {
        let reading_only = READING_ONLY.with(Cell::get);
        assert!(
          matches!(outcome, "same" | "written")
            || (reading_only && outcome.starts_with("declined: ")),
          "the writer's outcome for a written family's paper: {outcome}"
        );
      },
    }
  }
  value
}

thread_local! {
  static READING_ONLY: Cell<bool> = const { Cell::new(false) };
  static HELD: Cell<Option<&'static str>> = const { Cell::new(None) };
}

/// Run a guard of the lexer's or the resolver's reading of a kernel `\author` shape (`article`) with the writer's
/// outcome checked only for its own defect (`written-different`): a guard written before the kernel family was
/// written (93a, OD #925), whose subject is the events or the reading, not what the writer applies. A shape the writer
/// declines there is one of the stage-7 writer gaps the kernel family's census counts (the creators' names an old
/// reader parted otherwise, an entry today's lacks): today's XML kept, the reading pinned as the observer's.
pub(super) fn reading_only<R>(guard: impl FnOnce() -> R) -> R {
  READING_ONLY.with(|r| r.set(true));
  let value = guard();
  READING_ONLY.with(|r| r.set(false));
  value
}

/// The reading's JSON of a written family's paper the writer holds back, parsed: its outcome `declined: <why>…` — a
/// reading the writer has no operation for, today's XML kept, which the guard pins as such (the written-family
/// writer gaps in OPEN_ROWS).
pub(super) fn parsed_held(json: &str, why: &str) -> Value {
  let value: Value = serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}: {json}"));
  let outcome = value["writer"].as_str().unwrap_or("none");
  assert!(
    outcome.starts_with(&format!("declined: {why}")),
    "the writer's outcome: {outcome}"
  );
  value
}

/// Convert `tex` (with `preload`): the log, the XML, and the ledger as JSON.
pub(super) fn reading_of(tex: &str, preload: Option<&str>) -> (String, String, Value) {
  let (log, xml, json) = convert_with_setup_then(tex, preload, || {}, |_| fm_reading_json());
  let json: Value = parsed(&json);
  (log, xml, json)
}

/// An event, compactly: `N:<name>`, `M:<label>` (`*` a footnote symbol, `^` its line's first ink), `S:and` (`\and`;
/// `S:and:\And`, `S:and:\AND`), `S:column:&` (an alignment tab), `S:,`, `S:word:<word>`, `S:gap:<items>` (`S:fill:<items>` holding fill glue), `B` (a row), `C:<role>:<text>`, `T:<text>` (`#<label>`), `L:<what>`,
/// `A:<role>` (an API call after the block).
fn compact(event: &Value) -> String {
  let text = event["text"].as_str().unwrap_or("");
  let role = event["role"].as_str().unwrap_or("");
  match event["kind"].as_str().unwrap_or("") {
    "Name" => format!("N:{text}"),
    "Mark" => format!(
      "M:{}{}{}",
      event["label"].as_str().unwrap_or(""),
      if event["symbol"].as_bool() == Some(true) {
        "*"
      } else {
        ""
      },
      if event["line_first"].as_bool() == Some(true) {
        "^"
      } else {
        ""
      }
    ),
    "Sep" => match event["sep"].as_str().unwrap_or("") {
      "and" => match event["command"].as_str() {
        Some("\\and") | None => "S:and".to_string(),
        Some(command) => format!("S:and:{command}"),
      },
      "column" => format!("S:column:{}", event["command"].as_str().unwrap_or("")),
      "comma" => "S:,".to_string(),
      "word" => format!("S:word:{}", event["word"].as_str().unwrap_or("")),
      "gap" if event["fill"].as_bool() == Some(true) => format!("S:fill:{}", event["items"]),
      "gap" => format!("S:gap:{}", event["items"]),
      other => format!("S:{other}"),
    },
    "Break" => "B".to_string(),
    "Contact" => format!("C:{role}:{text}"),
    "Note" => match event["label"].as_str() {
      Some(label) => format!("T:{text}#{label}"),
      None => format!("T:{text}"),
    },
    "Layout" => format!("L:{role}"),
    "ApiCall" => format!("A:{}", role.split(' ').nth(1).unwrap_or(role)),
    other => other.to_string(),
  }
}

/// The author blocks of a ledger, each its compact event list (a skipped block: `["skipped: <reason>"]`).
pub(super) fn blocks(json: &Value) -> Vec<Vec<String>> {
  json["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .filter(|entry| entry["kind"] == "author")
    .map(|block| match block["events"].as_array() {
      Some(events) => events.iter().map(compact).collect(),
      None => vec![format!(
        "skipped: {}",
        block["reason"].as_str().unwrap_or("")
      )],
    })
    .collect()
}

/// An event as (compact event, span `[start, end]`).
type Spanned = (String, Option<(u64, u64)>);

/// The author blocks of a ledger, each its events [`Spanned`].
fn block_spans(json: &Value) -> Vec<Vec<Spanned>> {
  json["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .filter(|entry| entry["kind"] == "author")
    .map(|block| {
      block["events"]
        .as_array()
        .expect("a rehearsed block")
        .iter()
        .map(|event| {
          let span = event["span"].as_array().map(|span| {
            (
              span[0].as_u64().expect("start"),
              span[1].as_u64().expect("end"),
            )
          });
          (compact(event), span)
        })
        .collect()
    })
    .collect()
}

/// The API calls of a ledger: (role, made inside its block's expansion, flushes before it, queue position).
fn api_calls(json: &Value) -> Vec<(String, bool, u64, u64)> {
  json["entries"]
    .as_array()
    .expect("entries")
    .iter()
    // (the title is recorded for its marks: no creator's call)
    .filter(|entry| entry["kind"] == "api" && entry["tag"] != "ltx:title")
    .map(|call| {
      (
        call["role"].as_str().unwrap_or("").to_string(),
        call["inside"].as_bool().expect("inside"),
        call["epoch"].as_u64().expect("epoch"),
        call["queue_position"].as_u64().expect("queue_position"),
      )
    })
    .collect()
}

/// [`reading_of`] of an article document holding `author` (and `preamble`), asserting a clean conversion: its blocks.
fn article_blocks(preamble: &str, author: &str) -> Vec<Vec<String>> {
  let tex = format!(
    "\\documentclass{{article}}\n{preamble}\n\\begin{{document}}\n\\title{{T}}\n{author}\n\\maketitle\nBody.\n\\end{{document}}\n"
  );
  let (log, _, json) = reading_of(&tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  blocks(&json)
}

/// The design's symbol-line probes (ruling 2's shape, the design note's `dag`, `dagger`, `dagger2`): text `\dag`,
/// `$^{\dagger}$` and `$\dagger$` at a line's start are one footnote-symbol mark, the line's first ink, whatever command
/// printed it; the names' superscript symbols are marks after them, the `\and`s columns, the `\\`s rows.
#[test]
fn symbol_lines_are_symbol_marks_whatever_prints_them() {
  reading_only(|| {
    let names = [
      "N:Ann Able",
      "M:†*",
      "S:and",
      "N:Bob Baker",
      "M:‡*",
      "S:and",
      "N:Cat Cole",
      "M:†*",
    ];
    let expected: Vec<String> = names
      .iter()
      .chain(&["B", "M:†*^", "N:Univ A", "B", "M:‡*^", "N:Univ B"])
      .map(|s| s.to_string())
      .collect();
    let names_line = r"\author{Ann Able$^{\dag}$ \and Bob Baker$^{\ddag}$ \and Cat Cole$^{\dag}$\\";
    for lines in [
      r"\dag Univ A\\ \ddag Univ B}",
      r"$^{\dagger}$Univ A\\ $^{\ddagger}$Univ B}",
      r"$\dagger$ Univ A\\ $\ddagger$ Univ B}",
    ] {
      assert_eq!(
        article_blocks("", &format!("{names_line}\n{lines}")),
        std::slice::from_ref(&expected),
        "{lines}"
      );
    }
  });
}

/// The design's `mac` probe (2304.13998's shape): a user macro printing `\textsuperscript{#1}` is a mark by what it
/// prints, `~~ ` between two names a gap of three items, a line led by a mark labelled by it.
#[test]
fn user_mark_macro_prints_marks() {
  reading_only(|| {
    assert_eq!(
      article_blocks(
        r"\newcommand*{\affmark}[1][*]{\textsuperscript{#1}}",
        r"\author{Ann Able\affmark[1]~~ Bob Baker\affmark[1,2]\\ \affmark[1]Univ A\\ \affmark[2]Univ B}"
      ),
      [[
        "N:Ann Able",
        "M:1",
        "S:gap:3",
        "N:Bob Baker",
        "M:1,2",
        "B",
        "M:1^",
        "N:Univ A",
        "B",
        "M:2^",
        "N:Univ B"
      ]]
    );
  });
}

/// The gretsi v1.3 repro (2508.03934's shape; class-bindings/gretsi_v13_shared_mark_affiliation, RED for its
/// output): each `\auteur` is a block of its own (`\lx@add@authors@append`) printing the name and its marks, its
/// e-mail and the `\affils` list API calls after it.
#[test]
fn gretsi_author_blocks_and_their_api_calls() {
  let (log, _, json) = reading_of(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/gretsi_v13_shared_mark_affiliation.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(blocks(&json), [
    vec!["N:Simon Lepage", "M:1,2", "A:email"],
    vec!["N:Jeremie Mary", "M:1", "A:email"],
    vec![
      "N:David Picard",
      "M:2",
      "A:email",
      "A:affiliation",
      "A:affiliation"
    ],
  ]);
}

/// 2401.16957's `,\inst` shape (repro sectioning-frontmatter/llncs_comma_before_inst_is_the_name_before): llncs's
/// `\inst` marks follow the comma after the name, the `\fnmsep` `$^,$` between them a comma, the `\thanks` a note;
/// llncs splits its `\author` at `\and` into blocks, and the `\institute` list is API calls after the last.
#[test]
fn llncs_comma_before_inst_events() {
  let (log, _, json) = reading_of(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_comma_before_inst_is_the_name_before.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(blocks(&json), [
    vec![
      "N:Ann Able",
      "S:,",
      "M:1",
      "S:,",
      "M:2",
      "T:Email: ann@x.org",
      "N:Bob Baker",
      "S:,",
      "M:3",
      "S:,",
      "M:2"
    ],
    vec![
      "N:Cat Cole",
      "M:1",
      "A:affiliation",
      "A:affiliation",
      "A:affiliation"
    ],
  ]);
}

/// SYNC frontmatter rows' shapes (the silent-loss inventory): row 46's spacing-run names over merged symbol lines and
/// the unmarked rows after them; row 53's `$^{*1}$` (a mark, not a footnote symbol: it holds a number); row 66's mark
/// with a trailing comma and spacing alone between two names, and `\hspace{1em}and\hspace{1em}` (a gap, the word, a
/// gap); an empty `\textsuperscript{}` no mark.
#[test]
fn sync_row_shapes_46_53_66() {
  reading_only(|| {
    assert_eq!(
      article_blocks(
        "",
        r"\author{Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\ $^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Physics}"
      ),
      [[
        "N:Ann Able",
        "M:†*",
        "S:gap:2",
        "N:Bob Baker",
        "M:‡*",
        "B",
        "M:†*^",
        "N:Univ A",
        "S:gap:2",
        "M:‡*",
        "N:Univ B",
        "B",
        "N:Cat Cole",
        "B",
        "N:Physics"
      ]]
    );
    assert_eq!(
      article_blocks(
        "",
        r"\author{Ann Able$^{*1}$, Bob Baker$^{1}$ \\ $^1$Univ A}"
      ),
      [[
        "N:Ann Able",
        "M:\u{2217}1",
        "S:,",
        "N:Bob Baker",
        "M:1",
        "B",
        "M:1^",
        "N:Univ A"
      ]]
    );
    assert_eq!(
      article_blocks("", r"\author{Ann Able$^{1,}$ Bob Baker$^{2}$}"),
      [["N:Ann Able", "M:1,", "N:Bob Baker", "M:2"]]
    );
    assert_eq!(
      article_blocks("", r"\author{Ann Able\hspace{1em}and\hspace{1em}Bob Baker}"),
      [[
        "N:Ann Able",
        "S:gap:1",
        "S:word:and",
        "S:gap:1",
        "N:Bob Baker"
      ]]
    );
    // (an empty superscript prints nothing and is no mark, the design note's row 4)
    assert_eq!(
      article_blocks("", r"\author{Ann Able\textsuperscript{}, Bob Baker}"),
      [["N:Ann Able", "S:,", "N:Bob Baker"]]
    );
  });
}

/// SYNC rows 47 and 48's shapes: a link column under a name (`\href{…}{Project Page}` a URL contact printing its text,
/// a bare address a name-like run for the classifier) in amsart's per-author `\author` (`\lx@add@authors@append`); an
/// organization link and a braced line under the kernel `\author`'s columns.
#[test]
fn sync_row_shapes_47_48_links() {
  let (log, _, json) = reading_of(
    "\\documentclass{amsart}\n\\usepackage{hyperref}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\\\ \\href{https://x.org}{Project Page} \\\\ Univ B \\\\ contact@x.org}\n\\maketitle\nBody.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(blocks(&json), [[
    "N:Ann Able",
    "B",
    "C:url:Project Page",
    "B",
    "N:Univ B",
    "B",
    "N:contact@x.org"
  ]]);
  assert_eq!(
    article_blocks(
      r"\usepackage{hyperref}",
      r"\author{Hua Wei \\ \href{https://www.huawei.com}{Huawei} \and Hua Wu \\ {HW}}"
    ),
    [[
      "N:Hua Wei",
      "B",
      "C:url:Huawei",
      "S:and",
      "N:Hua Wu",
      "B",
      "N:HW"
    ]]
  );
}

/// SYNC rows 62 and 64's shapes (authblk): `\author[1]{Ann}` is a block (`\lx@add@authors@append`), the later
/// unlabelled `\author{Bob}` a single-author API call after it (authblk's `\lx@add@creator`), and so are the `\affil`s;
/// `\author{Ann$^1$ and Bob$^2$}` one list block with the word "and".
#[test]
fn sync_row_shapes_62_64_authblk() {
  assert_eq!(
    article_blocks(
      r"\usepackage{authblk}",
      "\\author[1]{Ann}\n\\author{Bob}\n\\affil[1]{Univ X}\n\\affil{Univ Y}"
    ),
    [["N:Ann", "A:author", "A:affiliation", "A:affiliation"]]
  );
  // (the block's own creator is queued inside its expansion; the later `\author{Bob}` and the `\affil`s outside it)
  let (log, _, json) = reading_of(
    "\\documentclass{article}\n\\usepackage{authblk}\n\\begin{document}\n\\title{T}\n\\author[1]{Ann}\n\\author{Bob}\n\\affil[1]{Univ X}\n\\affil{Univ Y}\n\\maketitle\nBody.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(api_calls(&json), [
    ("author".to_string(), true, 0, 1),
    ("author".to_string(), false, 0, 2),
    ("affiliation".to_string(), false, 0, 3),
    ("affiliation".to_string(), false, 0, 4),
  ]);
}

/// A `\thanks` holding a `\label` is a note labelled by it; a font switch between runs is a layout event; `\&` the
/// word separator; a tabular's cells layout events with its rows' end a break.
#[test]
fn notes_labels_fonts_and_cells() {
  reading_only(|| {
    assert_eq!(
      article_blocks(
        "",
        r"\author{Ann Able\thanks{\label{fn:a}Corresponding author: \texttt{ann@x.org}} \and Bob Baker \and {\bf Cat Cole}, \textsc{Dan Doe}\\ Dept of X, Univ Y \& Z\\ \begin{tabular}{c} Eve \\ Fay \end{tabular}}"
      ),
      [[
        "N:Ann Able",
        "T:Corresponding author: ann@x.org#fn:a",
        "S:and",
        "N:Bob Baker",
        "S:and",
        "L:font",
        "N:Cat Cole",
        "S:,",
        "L:font",
        "N:Dan Doe",
        "B",
        "L:font",
        "N:Dept of X",
        "S:,",
        "N:Univ Y",
        "S:word:&",
        "N:Z",
        "B",
        "L:\\@@tabular",
        "L:\\lx@begin@alignment",
        "L:cell",
        "N:Eve",
        "B",
        "L:cell",
        "N:Fay"
      ]]
    );
  });
}

/// A stray alignment tab in the field is a column separator (review revfms2a1_2 finding 4): the `&` constructor errs
/// only when it is built into the document (tex_tables.rs), so the rehearsal holds nothing for it, and `Cat & Dan` is
/// two names a column apart, not a gap; a printed `\&` stays the word separator. The document's own `&` errs once (as
/// pdflatex's "Extra alignment tab"), the same with the switch on and off.
#[test]
fn ampersand_is_a_column_separator() {
  reading_only(|| {
    let tex = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\& Bob Baker \\and Cat & Dan}\n\\maketitle\nBody.\n\\end{document}\n";
    let (..) = reading_of(tex, Some("ar5iv.sty"));
  });
}

/// The title's footnote marks are the kernel `\maketitle`'s `\@fnsymbol` unless a binding declares its own through the
/// hook `\lx@title@thefootnote` (review revfms2a1_2 finding 5; revtex4-2.cls:3040-3041 `\frontmatter@thefootnote`,
/// `\@alph` under `altaffilletter` only): here `\@alph`, so the bare `\footnotemark` after one `\thanks` prints "b" and
/// `\footnotemark[5]` "e".
#[test]
fn title_footnote_hook_overrides_fnsymbol() {
  assert_eq!(
    article_blocks(
      r"\makeatletter\def\lx@title@thefootnote{\@alph\c@footnote}\makeatother",
      r"\author{Ann Able\thanks{A.} \and Bob Baker\footnotemark \and Cat Cole\footnotemark[5]}"
    ),
    [[
      "N:Ann Able",
      "T:A.",
      "S:and",
      "N:Bob Baker",
      "M:b",
      "S:and",
      "N:Cat Cole",
      "M:e"
    ]]
  );
}

/// A font switched inside a run of ink does not end the name (review revfms2a1_2 note 9, recorded as a departure in
/// OXIDIZED_DESIGN_DIVERGENCES #837): the Layout event compares a run's first font with the previous run's, so it shows
/// at the next run after the switch — `Ann Able\begingroup\bfseries Bob \and Carl` puts it after the `\and`, before
/// "Carl" —, and a name with a small-caps or bold surname stays one Name (gretsi's `first \textsc{last}`, 2508.03934,
/// `gretsi_author_blocks_and_their_api_calls`).
#[test]
fn font_switch_inside_a_run_keeps_the_name() {
  assert_eq!(
    article_blocks(
      "",
      r"\author{Ann {\bfseries Able} Bob \and Cat {\itshape Cole}}"
    ),
    [["N:Ann Able Bob", "S:and", "N:Cat Cole"]]
  );
  let (log, _, json) = reading_of(
    "\\documentclass{article}\n\\title{T}\n\\author{Ann Able\\begingroup\\bfseries Bob \\and Carl}\n\\begin{document}\n\\maketitle\nBody after.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  // (the unclosed `\begingroup` closes with the author item since 79l, `digest_in_own_group`: its own Error only, the
  // body not bold)
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(blocks(&json), [[
    "N:Ann AbleBob",
    "S:and",
    "L:font",
    "N:Carl"
  ]]);
}

/// The observer's rehearsal leaves nothing behind (the census switch that compared it with the observer off went in DEL4):
/// nothing it reads lands in the body — here the
/// symbol-line probe, the gretsi repro (blocks rehearsed, API calls recorded) and the review's adversarial shapes,
/// whose rehearsal must leave nothing behind: an `\aftergroup` (it once leaked "LEAKED" into the body), a `\thanks`
/// holding a `\label` with a `\ref` to it and a `\footnote` in `\author`, a `\gdef` and a `\stepcounter`, an unclosed
/// `\begingroup` and an undefined command; an `\input` in the field (review revfms2a1_2 finding 3: the rehearsal read the
/// file again and the log gained a second `(Processing content …` note; the rehearsal silences progress notes).
#[test]
fn the_rehearsal_leaves_nothing_behind() {
  let doc = |preamble: &str, author: &str, body: &str| {
    format!(
      "\\documentclass{{article}}\n{preamble}\n\\title{{T}}\n\\author{{{author}}}\n\\begin{{document}}\n\\maketitle\n{body}\n\\end{{document}}\n"
    )
  };
  const AUTH_NAMES: &str = "Ann Able\\thanks{From file.} \\and Bob Baker\n";
  // (each shape: its source, its whole body text as the binary gives it, its Error and Warning counts — at `=2`, since
  // the flip; the body after the frontmatter, before post-processing, so a `\ref` is still empty)
  let cases: Vec<(String, &str, (usize, usize))> = vec![
    (
      "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{\\dag}$ \\and Bob Baker\\thanks{Funded.}\\\\ \\dag Univ A}\n\\maketitle\nBody.\n\\end{document}\n".to_string(),
      "Body.",
      (0, 0),
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/gretsi_v13_shared_mark_affiliation.tex"
      )
      .to_string(),
      "1 section 1 1 §1 1 Introduction Body text.",
      (0, 0),
    ),
    // (the `\aftergroup` once leaked "LEAKED" into the body; the one Warning counts the LEAKED box the creators lack,
    // where pdflatex prints LEAKED on the title page)
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/observer_rehearsal_aftergroup_stays_inside.tex"
      )
      .to_string(),
      "Body after.",
      (0, 1),
    ),
    (
      doc(
        "",
        r"Ann Able\thanks{Supported by X.\label{fn:a}} \and Bob Baker\footnote{Second.}",
        r"See note \ref{fn:a}. Text\footnote{Body note.}",
      ),
      "See note . Text 1 1 footnote 1 Body note.",
      (0, 0),
    ),
    // (RED, SYNC item 39: since the flip the author's `\stepcounter` is lost, "0" where Perl and pdflatex print "1";
    // the `\gdef` is kept — repro sectioning-frontmatter/observer_rehearsal_loses_an_author_stepcounter)
    (
      doc(
        r"\newcounter{probe}",
        r"Ann Able\stepcounter{probe}\ifx\foo\undefined\gdef\foo{ONE}\else\gdef\foo{TWO}\fi \and Bob",
        r"Value \foo{} and \theprobe.",
      ),
      "Value ONE and 0.",
      (0, 0),
    ),
    // (the unclosed `\begingroup` closes with the author item, its own Error only, the body not bold)
    (
      doc("", r"Ann Able\begingroup\bfseries Bob \and Carl", "Body after."),
      "Body after.",
      (1, 0),
    ),
    // (RED, SYNC item 39: since the flip the author field's undefined commands lose their Error and their `ERROR`
    // markup — `\email` (undefined in article) is not reported, and the one Error is the body's `\undefinedprobecs`;
    // the Warning counts "Bob", lost as a name. Perl: 2 Errors (`\undefinedprobecs`, `\email`), the creators Ann Able
    // and Bob; pdflatex 3 undefined control sequences)
    (
      doc(
        "",
        r"Ann Able\undefinedprobecs \and Bob\email{b@x.org}",
        r"Body \undefinedprobecs.",
      ),
      "Body \\undefinedprobecs .",
      (1, 1),
    ),
    // (the token analyser reads the sole `\input`'s file since 107b, `sole_input_file`, and so sees the `\and` it
    // holds: no `merged_creators` Error, which int51a-106t raised — `input_field_events_carry_no_span`; the three
    // Warnings are the emitter's plain-text names and note)
    (
      doc("", r"\input{auth_names}", "Body after."),
      "Body after.",
      (0, 3),
    ),
  ];
  let files = [("auth_names.tex", AUTH_NAMES)];
  // (what a rehearsal leaks lands in the body, `<p>LEAKEDBody after.</p>`, and a `\gdef` or `\stepcounter` it rolls back
  // too far is missing there)
  let got: Vec<(String, (usize, usize))> = cases
    .iter()
    .map(|(tex, ..)| {
      let (log, xml, ()) =
        convert_files_with_setup_then(tex, &files, Some("ar5iv.sty"), || {}, |_| ());
      (
        super::perfect_kernel_batch94b::body_text(&xml),
        (error_count(&log), warning_count(&log)),
      )
    })
    .collect();
  let want: Vec<(String, (usize, usize))> = cases
    .iter()
    .map(|(_, body, counts)| (body.to_string(), *counts))
    .collect();
  assert_eq!(got, want);
}

/// Spans and widths (review finding 2): each event's span of the author field's positions (the kernel `\author`'s
/// stamp, from 0) in two shapes; `\hfill` a fill gap (it always separates, natural width 0) where it was read as an
/// interword space; `\hspace{3em}` a gap of its `width` (3 em of its font), not of the characters printed for it; the
/// kernel block's creators API calls inside its expansion, queued in the first flush's queue after the title.
#[test]
fn spans_fill_gaps_and_widths() {
  reading_only(|| {
    let tex = include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/observer_fill_gap_and_footnotemark_labels.tex"
    );
    let (log, _, json) = reading_of(tex, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let spans = block_spans(&json);
    let first: Vec<(&str, (u64, u64))> = spans[0]
      .iter()
      .map(|(event, span)| (event.as_str(), span.expect("a span")))
      .collect();
    assert_eq!(first, [
      ("N:Ann Able", (0, 8)),
      ("S:fill:1", (8, 9)),
      ("N:Bob Baker", (9, 18)),
      ("S:gap:1", (18, 24)),
      ("N:Cat Cole", (24, 32)),
      ("M:**", (32, 33)),
      ("N:Dan", (34, 37)),
      ("B", (37, 43)),
      ("N:Univ A", (44, 50)),
    ]);
    let gaps: Vec<(f64, bool)> = json["entries"]
      .as_array()
      .expect("entries")
      .iter()
      .filter(|entry| entry["kind"] == "author")
      .flat_map(|block| block["events"].as_array().cloned().unwrap_or_default())
      .filter(|event| event["sep"] == "gap")
      .map(|event| {
        (
          event["width"].as_f64().expect("width"),
          event["fill"].as_bool().expect("fill"),
        )
      })
      .collect();
    assert_eq!(gaps, [(0.0, true), (3.0, false)]);
  });
}

/// A bare `\footnotemark` is labelled with the mark it prints (review finding 3): `\thefootnote` at the counter's next
/// value — the `\@fnsymbol` of the kernel `\maketitle` —, after the steps the block's `\thanks` and earlier bare marks
/// made, `\footnotemark[5]` at 5 (¶); none of it steps `\c@footnote`. Which `\and` separates is kept (finding 4:
/// NeurIPS's and ICLR's `\AND` is a new row). The spans of this second shape: a mark's is its own command's.
#[test]
fn footnotemark_labels_and_and_commands() {
  let (log, _, json) = reading_of(
    "\\documentclass{article}\n\\newcommand\\And{\\and}\n\\newcommand\\AND{\\and}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\thanks{First.} \\And Bob Baker\\footnotemark \\AND Cat Cole\\footnotemark[5] \\and Dan Doe\\footnotemark}\n\\maketitle\n\\footnotetext{Note.}\nBody: \\thefootnote.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let spans = block_spans(&json);
  let first: Vec<(&str, (u64, u64))> = spans[0]
    .iter()
    .map(|(event, span)| (event.as_str(), span.expect("a span")))
    .collect();
  assert_eq!(first, [
    ("N:Ann Able", (0, 8)),
    ("T:First.", (8, 17)),
    ("S:and:\\And", (18, 19)),
    ("N:Bob Baker", (19, 28)),
    ("M:†*", (28, 29)),
    ("S:and:\\AND", (29, 30)),
    ("N:Cat Cole", (30, 38)),
    ("M:¶*", (38, 42)),
    ("S:and", (43, 44)),
    ("N:Dan Doe", (44, 51)),
    ("M:‡*", (51, 52)),
  ]);
  // (a `\footnotemark[<n>]` ending the block covers its argument, 2211.10835's shared mark; not the closing `]`, the
  // block's last token: core `field_cover_of` ends a cover at the next token of the field, and there is none)
  let blocks = article_blocks(
    "",
    r"\author{Ann Able\thanks{A.} \and Bob Baker\footnotemark[1]}",
  );
  let (log, _, json) = reading_of(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\thanks{A.} \\and Bob Baker\\footnotemark[1]}\n\\maketitle\nBody.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(blocks, [[
    "N:Ann Able",
    "T:A.",
    "S:and",
    "N:Bob Baker",
    "M:**"
  ]]);
  let spans: Vec<Option<(u64, u64)>> = block_spans(&json)[0]
    .iter()
    .map(|(_, span)| *span)
    .collect();
  assert_eq!(spans, [
    Some((0, 8)),
    Some((8, 13)),
    Some((14, 15)),
    Some((15, 24)),
    Some((24, 27))
  ]);
}

/// `\lx@field@stamp{<field>}{<tokens>}` gives back its tokens, each stamped with its position in the field
/// (`Token::prov`: field 2, the author's, positions 0.. — no earlier stamp), and advances the field's positions
/// (`lx_field_base_2`) past them; an unknown field's come back as they are, unstamped.
#[test]
fn field_stamp_gives_back_its_argument() {
  use latexml_core::{
    definition::ExpansionBody,
    state::{lookup_definition, lookup_int},
    token::{field_prov_field, field_prov_pos},
  };
  let (log, xml, (base, provs)) = convert_with_setup_then(
    "\\documentclass{article}\n\\begin{document}\n\\makeatletter\\edef\\x{\\lx@field@stamp{author}{Ann Able}/\\lx@field@stamp{nofield}{Bob}}\\makeatother\n\\x\n\\end{document}\n",
    None,
    || {},
    |_| {
      let provs: Vec<(String, u16, u16)> = match lookup_definition(&latexml_core::T_CS!("\\x"))
        .ok()
        .flatten()
      {
        Some(defn) => match defn.get_expansion() {
          Some(ExpansionBody::Tokens(tokens)) => tokens
            .unlist_ref()
            .iter()
            .map(|t| {
              (
                t.to_string(),
                field_prov_field(t.prov),
                field_prov_pos(t.prov),
              )
            })
            .collect(),
          _ => Vec::new(),
        },
        None => Vec::new(),
      };
      (lookup_int("lx_field_base_2"), provs)
    },
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Ann Able/Bob"), "{xml}");
  let expected: Vec<(String, u16, u16)> = "Ann Able"
    .chars()
    .enumerate()
    .map(|(i, c)| (c.to_string(), 2, i as u16))
    .chain("/Bob".chars().map(|c| (c.to_string(), 0, 0)))
    .collect();
  assert_eq!(provs, expected);
  assert_eq!(base, 8);
}
