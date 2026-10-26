//! Guards for batch FM S2a.1: the frontmatter observer core in shadow mode (OXIDIZED_DESIGN_DIVERGENCES #837;
//! KERNEL_CAPABILITIES K23). Under the shadow switch (`LATEXML_FM_SHADOW=1`, here the thread's override) each
//! `\author`-family call is a ledger block whose stamped tokens are rehearsed under the Layer-B event bindings and lexed
//! into events; the guards assert those event lists for the design probes (symbol lines, a user mark macro), the
//! gretsi shared-mark repro, 2401.16957's `,\inst` shape and the SYNC frontmatter rows' shapes, that the conversion's
//! XML is byte-identical with the switch on and off, and that the CLI writes `<dest>.fmshadow.json` only under the
//! switch. Every probe compiles clean with pdflatex (TL 2025) and converts with no Error or Warning.
use std::process::Command;

use latexml::util::test::{
  convert_files_with_setup_then, convert_with_setup_then, error_count, warning_count,
};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

/// Convert `tex` (with `preload`) under the shadow switch: the log, the XML, and the shadow ledger as JSON.
pub(super) fn shadow(tex: &str, preload: Option<&str>) -> (String, String, Value) {
  let (log, xml, json) = convert_with_setup_then(
    tex,
    preload,
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  let json: Value = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}: {json}"));
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

/// The author blocks of a shadow ledger, each its compact event list (a skipped block: `["skipped: <reason>"]`).
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

/// The author blocks of a shadow ledger, each its events [`Spanned`].
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

/// The API calls of a shadow ledger: (role, made inside its block's expansion, flushes before it, queue position).
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

/// [`shadow`] of an article document holding `author` (and `preamble`), asserting a clean conversion: its blocks.
fn article_blocks(preamble: &str, author: &str) -> Vec<Vec<String>> {
  let tex = format!(
    "\\documentclass{{article}}\n{preamble}\n\\begin{{document}}\n\\title{{T}}\n{author}\n\\maketitle\nBody.\n\\end{{document}}\n"
  );
  let (log, _, json) = shadow(&tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  blocks(&json)
}

/// The design's symbol-line probes (ruling 2's shape, the design note's `dag`, `dagger`, `dagger2`): text `\dag`,
/// `$^{\dagger}$` and `$\dagger$` at a line's start are one footnote-symbol mark, the line's first ink, whatever command
/// printed it; the names' superscript symbols are marks after them, the `\and`s columns, the `\\`s rows.
#[test]
fn symbol_lines_are_symbol_marks_whatever_prints_them() {
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
}

/// The design's `mac` probe (2304.13998's shape): a user macro printing `\textsuperscript{#1}` is a mark by what it
/// prints, `~~ ` between two names a gap of three items, a line led by a mark labelled by it.
#[test]
fn user_mark_macro_prints_marks() {
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
}

/// The gretsi v1.3 repro (2508.03934's shape; class-bindings/gretsi_v13_shared_mark_affiliation, RED for its
/// output): each `\auteur` is a block of its own (`\lx@add@authors@append`) printing the name and its marks, its
/// e-mail and the `\affils` list API calls after it.
#[test]
fn gretsi_author_blocks_and_their_api_calls() {
  let (log, _, json) = shadow(
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
  let (log, _, json) = shadow(
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
}

/// SYNC rows 47 and 48's shapes: a link column under a name (`\href{…}{Project Page}` a URL contact printing its text,
/// a bare address a name-like run for the classifier) in amsart's per-author `\author` (`\lx@add@authors@append`); an
/// organization link and a braced line under the kernel `\author`'s columns.
#[test]
fn sync_row_shapes_47_48_links() {
  let (log, _, json) = shadow(
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
  let (log, _, json) = shadow(
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
  assert_eq!(
    article_blocks(
      r"\usepackage{authblk}",
      "\\author{Ann$^1$ and Bob$^2$}\n\\affil{$^1$X}\n\\affil{$^3$Z}"
    ),
    [[
      "N:Ann",
      "M:1",
      "S:word:and",
      "N:Bob",
      "M:2",
      "A:affiliation",
      "A:affiliation"
    ]]
  );
}

/// A `\thanks` holding a `\label` is a note labelled by it; a font switch between runs is a layout event; `\&` the
/// word separator; a tabular's cells layout events with its rows' end a break.
#[test]
fn notes_labels_fonts_and_cells() {
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
}

/// A box's or an alignment's placement argument is not ink (review revfms2a1_2 finding 1: the `[t]` of
/// `\begin{tabular}[t]{c}` and `\parbox[t]`, the `[l]` of `\makebox[3cm][l]`, the `[b]` of `\begin{minipage}[b]` were
/// read as names, "tAnn AbleUniv A"): the walk reads only the arguments a whatsit shows (util_text.rs
/// `shown_arguments`, a `Position` parameter is none); an image shows none of its arguments but is a Layout event of
/// its own (2503.08161's logo before each affiliation; not its file name, nor nothing — the spaces around it once
/// merged into a gap).
#[test]
fn placement_arguments_are_not_names() {
  assert_eq!(
    article_blocks(
      "",
      r"\author{\begin{tabular}[t]{c}Ann Able\\ Univ A\end{tabular} \and \parbox[t]{5cm}{Bob Baker\\ Univ B} \and \makebox[3cm][l]{Cat Cole} \and \begin{minipage}[b]{0.3\textwidth}Dan Doe\end{minipage}}"
    ),
    [[
      "L:\\@@tabular",
      "L:\\lx@begin@alignment",
      "L:cell",
      "N:Ann Able",
      "B",
      "L:cell",
      "N:Univ A",
      "S:and",
      "L:\\lx@hidden@bgroup",
      "L:\\lx@parbox",
      "N:Bob Baker",
      "B",
      "N:Univ B",
      "S:and",
      "L:\\@makebox",
      "N:Cat Cole",
      "S:and",
      "L:\\begin{minipage}",
      "N:Dan Doe"
    ]]
  );
  assert_eq!(
    article_blocks(
      r"\usepackage{graphicx}",
      r"\author{Ann Able$^{1}$\\ $^{1}$ \includegraphics[width=1em]{example-image} Univ A}"
    ),
    [[
      "N:Ann Able",
      "M:1",
      "B",
      "M:1^",
      "L:\\@includegraphicx",
      "N:Univ A"
    ]]
  );
}

/// A line break the block rebinds past the rehearsal's `\\` is still a break (review revfms2a1_2 finding 2):
/// `\centering` and `\raggedright` let `\\` be `\@normalcr`, a parbox's and a minipage's body `\lx@parboxnewline` and
/// `\lx@newline`, each the kernel's `\lx@newline` constructor, once "Ann AbleUniv A" one name. The in-box shapes
/// compile clean with pdflatex; the review's bare `{\centering …\\…}` (pdflatex: "Not allowed in LR mode", the author
/// line is a `tabular` cell) converts clean here and breaks the same.
#[test]
fn breaks_survive_a_rebound_newline() {
  assert_eq!(
    article_blocks(
      "",
      r"\author{\parbox[t]{5cm}{\centering Ann Able\\ Univ A} \and \begin{minipage}[t]{4cm}\raggedright Bob Baker\\ Univ B\end{minipage} \and \parbox[t]{5cm}{Cat Cole\\ Univ C}}"
    ),
    [[
      "L:\\lx@hidden@bgroup",
      "L:\\lx@parbox",
      "N:Ann Able",
      "B",
      "N:Univ A",
      "S:and",
      "L:\\begin{minipage}",
      "N:Bob Baker",
      "B",
      "N:Univ B",
      "S:and",
      "L:\\lx@hidden@bgroup",
      "L:\\lx@parbox",
      "N:Cat Cole",
      "B",
      "N:Univ C"
    ]]
  );
  assert_eq!(
    article_blocks(
      "",
      r"\author{{\centering Ann Able\\ Univ A} \and {\raggedright Bob\\ Univ B}}"
    ),
    [[
      "N:Ann Able",
      "B",
      "N:Univ A",
      "S:and",
      "N:Bob",
      "B",
      "N:Univ B"
    ]]
  );
}

/// A stray alignment tab in the field is a column separator (review revfms2a1_2 finding 4): the `&` constructor errs
/// only when it is built into the document (tex_tables.rs), so the rehearsal holds nothing for it, and `Cat & Dan` is
/// two names a column apart, not a gap; a printed `\&` stays the word separator. The document's own `&` errs once (as
/// pdflatex's "Extra alignment tab"), the same with the switch on and off.
#[test]
fn ampersand_is_a_column_separator() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\& Bob Baker \\and Cat & Dan}\n\\maketitle\nBody.\n\\end{document}\n";
  let (log, _, json) = shadow(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(blocks(&json), [[
    "N:Ann Able",
    "S:word:&",
    "N:Bob Baker",
    "S:and",
    "N:Cat",
    "S:column:&",
    "N:Dan"
  ]]);
  let block = json["entries"]
    .as_array()
    .and_then(|entries| entries.iter().find(|e| e["kind"] == "author"))
    .expect("an author block");
  assert_eq!(
    (block["errors"].as_u64(), block["warnings"].as_u64()),
    (Some(0), Some(0))
  );
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
  let (log, _, json) = shadow(
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

/// Tokens an `\input` in the field reads come from the file, not the field, and carry no position (review revfms2a1_2
/// finding 3; OXIDIZED_DESIGN_DIVERGENCES #837): their events have no span, and the `\and` the file holds separates
/// the names the token analyser merges.
#[test]
fn input_field_events_carry_no_span() {
  let (log, _, json) = convert_files_with_setup_then(
    "\\documentclass{article}\n\\title{T}\n\\author{\\input{auth_names}}\n\\begin{document}\n\\maketitle\nBody after.\n\\end{document}\n",
    &[(
      "auth_names.tex",
      "Ann Able\\thanks{From file.} \\and Bob Baker\n",
    )],
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  // (the one Error is the token analyser's, named: review revfms2a1_3 nit 2)
  assert!(log.contains("Error:frontmatter:merged_creators"), "{log}");
  let json: Value = serde_json::from_str(&json).expect("json");
  assert_eq!(block_spans(&json), [vec![
    ("N:Ann Able".to_string(), None),
    ("T:From file.".to_string(), None),
    ("S:and".to_string(), None),
    ("N:Bob Baker".to_string(), None),
  ]]);
}

/// The log of a conversion without its run-specific parts (the timings, the sizes written, the temporary source path).
fn stable_log(log: &str) -> Vec<String> {
  log
    .lines()
    .filter(|line| !line.contains(" sec") && !line.contains("bytes)") && !line.contains("started "))
    .map(|line| match line.find("/tmp/") {
      Some(at) => line[..at].to_string(),
      None => line.to_string(),
    })
    .collect()
}

/// The observer decides nothing: a conversion's XML and log are the same with the shadow switch on and off — here the
/// symbol-line probe, the gretsi repro (blocks rehearsed, API calls recorded) and the review's adversarial shapes,
/// whose rehearsal must leave nothing behind: an `\aftergroup` (it once leaked "LEAKED" into the body), a `\thanks`
/// holding a `\label` with a `\ref` to it and a `\footnote` in `\author`, a `\gdef` and a `\stepcounter`, an unclosed
/// `\begingroup` and an undefined command (each with its own diagnostics, counted exactly, the same either way); an
/// `\input` in the field (review revfms2a1_2 finding 3: the rehearsal read the file again and the log gained a second
/// `(Processing content …` note; the rehearsal silences progress notes).
#[test]
fn shadow_switch_leaves_the_xml_unchanged() {
  let doc = |preamble: &str, author: &str, body: &str| {
    format!(
      "\\documentclass{{article}}\n{preamble}\n\\title{{T}}\n\\author{{{author}}}\n\\begin{{document}}\n\\maketitle\n{body}\n\\end{{document}}\n"
    )
  };
  const AUTH_NAMES: &str = "Ann Able\\thanks{From file.} \\and Bob Baker\n";
  let cases: Vec<(String, (usize, usize))> = vec![
    (
      "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{\\dag}$ \\and Bob Baker\\thanks{Funded.}\\\\ \\dag Univ A}\n\\maketitle\nBody.\n\\end{document}\n".to_string(),
      (0, 0),
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/gretsi_v13_shared_mark_affiliation.tex"
      )
      .to_string(),
      (0, 0),
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/observer_rehearsal_aftergroup_stays_inside.tex"
      )
      .to_string(),
      (0, 0),
    ),
    (
      doc(
        "",
        r"Ann Able\thanks{Supported by X.\label{fn:a}} \and Bob Baker\footnote{Second.}",
        r"See note \ref{fn:a}. Text\footnote{Body note.}",
      ),
      (0, 0),
    ),
    (
      doc(
        r"\newcounter{probe}",
        r"Ann Able\stepcounter{probe}\ifx\foo\undefined\gdef\foo{ONE}\else\gdef\foo{TWO}\fi \and Bob",
        r"Value \foo{} and \theprobe.",
      ),
      (0, 0),
    ),
    (
      doc("", r"Ann Able\begingroup\bfseries Bob \and Carl", "Body after."),
      (1, 0),
    ),
    (
      doc(
        "",
        r"Ann Able\undefinedprobecs \and Bob\email{b@x.org}",
        r"Body \undefinedprobecs.",
      ),
      (2, 0),
    ),
    // (the token analyser cannot see the `\and` the file holds: one Error, `merged_creators`, as on int51a — what
    // the observer reads past, `input_field_events_carry_no_span`)
    (doc("", r"\input{auth_names}", "Body after."), (1, 0)),
  ];
  let files = [("auth_names.tex", AUTH_NAMES)];
  for (tex, counts) in cases {
    let (on_log, on_xml, on_json) = convert_files_with_setup_then(
      &tex,
      &files,
      Some("ar5iv.sty"),
      || set_fm_shadow_override(Some(true)),
      |_| fm_shadow_json(),
    );
    let on_json: Value = serde_json::from_str(&on_json).expect("json");
    let (off_log, off_xml, off_json) = convert_files_with_setup_then(
      &tex,
      &files,
      Some("ar5iv.sty"),
      || set_fm_shadow_override(Some(false)),
      |_| fm_shadow_json(),
    );
    assert_eq!(on_xml, off_xml, "{tex}");
    assert_eq!(stable_log(&on_log), stable_log(&off_log), "{tex}");
    assert_eq!(
      (error_count(&off_log), warning_count(&off_log)),
      counts,
      "{tex}\n{off_log}"
    );
    if tex.contains(r"\input{auth_names}") {
      assert!(
        off_log.contains("Error:frontmatter:merged_creators"),
        "{tex}\n{off_log}"
      );
    }
    // (on: each block rehearsed; off: nothing recorded)
    assert!(
      on_json["summary"]["with_events"].as_u64() >= Some(1),
      "{tex}\n{on_json}"
    );
    let off: Value = serde_json::from_str(&off_json).expect("json");
    assert_eq!(off["entries"], Value::Array(Vec::new()));
  }
}

/// Spans and widths (review finding 2): each event's span of the author field's positions (the kernel `\author`'s
/// stamp, from 0) in two shapes; `\hfill` a fill gap (it always separates, natural width 0) where it was read as an
/// interword space; `\hspace{3em}` a gap of its `width` (3 em of its font), not of the characters printed for it; the
/// kernel block's creators API calls inside its expansion, queued in the first flush's queue after the title.
#[test]
fn spans_fill_gaps_and_widths() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/observer_fill_gap_and_footnotemark_labels.tex"
  );
  let (log, _, json) = shadow(tex, Some("ar5iv.sty"));
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
  assert_eq!(api_calls(&json)[..3], [
    ("author".to_string(), true, 0, 1),
    ("author".to_string(), true, 0, 2),
    ("author".to_string(), true, 0, 3),
  ]);
}

/// A bare `\footnotemark` is labelled with the mark it prints (review finding 3): `\thefootnote` at the counter's next
/// value — the `\@fnsymbol` of the kernel `\maketitle` —, after the steps the block's `\thanks` and earlier bare marks
/// made, `\footnotemark[5]` at 5 (¶); none of it steps `\c@footnote`. Which `\and` separates is kept (finding 4:
/// NeurIPS's and ICLR's `\AND` is a new row). The spans of this second shape: a mark's is its own command's.
#[test]
fn footnotemark_labels_and_and_commands() {
  let (log, _, json) = shadow(
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
  let (log, _, json) = shadow(
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

/// A block past the field's positions (14 bits, cumulative per document: three 6,000-letter `\author` calls) is not
/// rehearsed, and says so; the title's stamp is recorded beside it.
#[test]
fn out_of_positions_block_is_skipped() {
  let long = "a".repeat(6000);
  let author = format!("\\author{{X{long}}}\n\\author{{Y{long}}}\n\\author{{Z{long}}}");
  assert_eq!(article_blocks("", &author), [
    vec![format!("N:X{long}")],
    vec![format!("N:Y{long}")],
    vec!["skipped: unstamped: out of positions".to_string()],
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

/// The CLI writes `<dest>.fmshadow.json` beside the output under `LATEXML_FM_SHADOW=1` — the ledger's blocks with their
/// events — and nothing without it; the XML is the same either way.
#[test]
fn cli_writes_the_shadow_dump_only_under_the_switch() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("tempdir");
  std::fs::write(
    workdir.path().join("t.tex"),
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{1}$ \\and Bob Baker$^{2}$\\\\ $^1$Univ A\\\\ $^2$Univ B}\n\\maketitle\nBody.\n\\end{document}\n",
  )
  .expect("write t.tex");
  let run = |dest: &str, on: bool| {
    let mut command = Command::new(bin);
    command
      .args(["t.tex", "--dest", dest, "--nocomments", "--timeout=110"])
      .current_dir(workdir.path())
      .env_remove("LATEXML_FM_SHADOW");
    if on {
      command.env("LATEXML_FM_SHADOW", "1");
    }
    let output = command.output().expect("spawn");
    String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "")
  };
  let on_log = run("on.xml", true);
  let off_log = run("off.xml", false);
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
  let read = |name: &str| std::fs::read_to_string(workdir.path().join(name));
  assert_eq!(
    read("on.xml").expect("on.xml"),
    read("off.xml").expect("off.xml")
  );
  assert!(read("off.xml.fmshadow.json").is_err());
  let json: Value =
    serde_json::from_str(&read("on.xml.fmshadow.json").expect("the shadow dump")).expect("json");
  assert_eq!(blocks(&json), [[
    "N:Ann Able",
    "M:1",
    "S:and",
    "N:Bob Baker",
    "M:2",
    "B",
    "M:1^",
    "N:Univ A",
    "B",
    "M:2^",
    "N:Univ B"
  ]]);
  let block = json["entries"]
    .as_array()
    .and_then(|entries| entries.iter().find(|e| e["kind"] == "author"))
    .expect("an author block");
  assert_eq!(
    (
      block["hook"].as_str(),
      block["policy"].as_str(),
      block["stamp"].as_str(),
      block["replace"].as_bool()
    ),
    (
      Some("\\lx@add@authors@adaptive"),
      Some("kernel"),
      Some("kernel"),
      Some(true)
    )
  );
  assert_eq!(
    json["summary"]["with_events"].as_u64(),
    Some(1),
    "{}",
    json["summary"]
  );
}
