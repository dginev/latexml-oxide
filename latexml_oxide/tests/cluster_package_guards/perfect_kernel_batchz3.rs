//! Guards for Z3 (`docs/archive/FRONTMATTER_DIGESTED_REFACTOR_2026-11-07.md`, Stage 3; `FRONTMATTER_SINGLE_ENGINE_PLAN.md` "What a rollback leaves"): what a span of a field holds, read
//! from the digested list (`emitter::Field`), and the rehearsal's isolation. Each guard is a different code path: `a_cut_takes_each_formula_whole`
//! a piece's start and end in a formula, `a_cut_takes_a_group_it_closes` the brace balance, `a_mark_in_its_wrapper_…` a mark's own print and a
//! block's line, `a_declarations_span_…` the settled declarations; the `the_rehearsal_leaves_no_trace_*` table the journal and what
//! lives outside it, by theme, and its `_written` row the written family's path, where the writer runs too. Z3r2: `a_bindings_state_…` the package thread-locals
//! moved to the Value table, `a_rehearsals_read_…` the raw-read list, `a_speculative_run_evaluates_no_lua` the one state no rollback reaches, `a_declaration_in_a_body_…` where a body's definer runs,
//! `a_marks_own_group_…` the Walker's group fact, `a_cut_holds_its_environments` the run's environments, `a_looping_block_…` the budget. Z3r3: `a_group_a_mark_opens_…` the Walker's
//! line-start fact. The `#[ignore]`d `…_red` rows assert the right output of what the review filed and this batch leaves: `cargo nextest run --run-ignored only -E 'test(_red)'`.

use serde_json::json;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz1::{blocks, dump},
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `creators` as owned strings, for a table's literal.
fn creators_are(xml: &str, expected: &[&[&str]]) {
  let expected: Vec<Vec<String>> = expected
    .iter()
    .map(|creator| creator.iter().map(ToString::to_string).collect())
    .collect();
  assert_eq!(creators(xml), expected, "{xml}");
}

/// A piece of a cut call takes the formula it starts or ends in whole, whatever opens it: a switch before the script (88c's RED,
/// `$\rm ^{3}$`: no balanced run when walked back from the script, 2105.07958), an empty base, LaTeX's `\(` with spaces, `\protect`ed,
/// `${^6}$` (1410.8534, astro-ph9807043), and a formula of no mark among a piece's words.
#[test]
fn a_cut_takes_each_formula_whole() {
  let (xml, _) = dump(repro!("revtex_affiliation_requested_switched_mark_cuts"));
  creators_are(&xml, &[
    &[
      "Ann Able 1,3",
      "affiliation=1 Univ X",
      "affiliation=3 He Lab",
    ],
    &["Bob Baker 2", "affiliation=Univ Y"],
  ]);
  let (xml, _) = dump(repro!("affiliation_cut_formula_whole_whatever_opens_it"));
  creators_are(&xml, &[
    &["Ann Able 1", "affiliation=1 Univ X"],
    &["Bob Baker 2", "affiliation=2 Univ Y"],
    &["Cat Cole 3", "affiliation=Univ Z"],
    &["Dan Dow 4", "affiliation=4 Univ W"],
    &["Eve Ely 5", "affiliation=5 Univ V"],
    &["Fay Fox 6", "affiliation=Univ U, α Lab"],
  ]);
}

/// A piece closing a mark's own font group takes the group from its opening brace, its font with it; one opening a group ends past its
/// closing brace: read from the content's brace balance, forward. Before Z3 each was no balanced run, and the paper not written. A group going on
/// after its mark stays no run: `perfect_kernel_batch107e::a_wrapped_isotope_is_entry_text`.
#[test]
fn a_cut_takes_a_group_it_closes() {
  let (xml, _) = dump(repro!("affiliation_cut_group_closes_after_the_mark"));
  creators_are(&xml, &[
    &["Ann Able 1", "affiliation=1 Univ X"],
    &["Bob Baker 2", "affiliation=2 Univ Y"],
    &["Cat Cole 3", "affiliation=3 Univ Z"],
  ]);
}

/// A mark heading a block's entry inside the entry's own wrapper: its `formula_span` is the formula's, not the wrapper's cover its first box
/// took (2605.25195), and a `\textsuperscript` call is read by its own position. Each entry is its name's and keeps the wrapper's font; the
/// writer made them (today's one contact on one line).
#[test]
fn a_mark_in_its_wrapper_is_read_by_its_own_print() {
  let (xml, dump) = dump(repro!("block_line_mark_in_its_wrapper_whatever_prints_it"));
  creators_are(&xml, &[
    &["Ann Able", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=Univ Y"],
    &["Cat Cole", "affiliation=Univ Z"],
    &[],
  ]);
  assert!(
    xml.contains("<note") && xml.contains("Equal Contribution"),
    "{xml}"
  );
  // (the wrappers' bold on both, opened again round each entry made)
  assert_eq!(xml.matches(r#"<text font="bold""#).count(), 2, "{xml}");
  assert!(
    xml.contains(">Univ X</text>") && xml.contains(">Univ Y</text>"),
    "{xml}"
  );
  let marks: Vec<_> = blocks(&dump)[0]["events"]
    .as_array()
    .expect("events")
    .iter()
    .filter(|e| e["kind"] == "Mark")
    .map(|e| (e["span"].clone(), e["formula_span"].clone()))
    .collect();
  assert_eq!(marks, vec![
    (json!([9, 13]), json!([8, 14])),
    (json!([26, 30]), json!([25, 31])),
    (json!([42, 46]), json!([41, 47])),
    (json!([49, 62]), json!([51, 55])),
    (json!([63, 76]), json!([63, 76])),
    (json!([80, 82]), json!([79, 83])),
    (json!([93, 95]), json!([92, 96])),
  ]);
}

/// A definer in the body of a macro the field defined: its span is the call it runs at (the stamp on the invocation), wherever that is, past what prints between; what one call defines is
/// one declaration; a box of the body takes it back; and what encloses the call is no box of its (review revZ3 S1: Z4 would print those words twice).
#[test]
fn a_declaration_in_a_body_holds_its_call() {
  let (_, dump) = dump(repro!("event_declaration_in_a_body_holds_its_call"));
  assert_eq!(
    blocks(&dump)[0]["declarations"],
    json!([
      // (`\\s`'s call, past " Ann Bob")
      {"kind": "Declaration", "span": [0, 9], "label": "\\s"},
      {"kind": "Declaration", "span": [17, 18], "label": "\\t"},
      // (`\\m`'s, past " Cat"; `\\n`'s, past "\\ Dan")
      {"kind": "Declaration", "span": [19, 32], "label": "\\m"},
      {"kind": "Declaration", "span": [36, 37], "label": "\\n"},
      {"kind": "Declaration", "span": [41, 42], "label": "\\o"},
      // (each on the call that defines it, not the first's)
      {"kind": "Declaration", "span": [48, 61], "label": "\\ma"},
      {"kind": "Declaration", "span": [61, 62], "label": "\\na"},
      {"kind": "Declaration", "span": [62, 63], "label": "\\oa"},
      // (a body's definer reading on past the call: `\\q{1}` whole)
      {"kind": "Declaration", "span": [68, 74], "label": "\\q"},
      {"kind": "Declaration", "span": [78, 82], "label": "\\r"},
      // (`\\u`'s body prints X: no declaration holds its call)
      {"kind": "Declaration", "span": [88, 98], "label": "\\u"},
      // (one call, one declaration)
      {"kind": "Declaration", "span": [107, 121], "label": "\\w"},
      {"kind": "Declaration", "span": [125, 126], "label": "\\xa \\xb"},
      // (the `\\textbf` round the call is no box of the call's)
      {"kind": "Declaration", "span": [137, 146], "label": "\\y", "nest": 1},
      {"kind": "Declaration", "span": [146, 147], "label": "\\z", "nest": 1},
    ])
  );
}

/// A group a mark's formula opens where a line starts, at the field's start or right after a line break by what digestion made, starts at its brace whatever goes on in it
/// (`Formula::opens_line_in`, 2005.08290: `~\\ \normalsize{$^{1}$ Shemyakin…,} \normalsize{Moscow, Russia}\\ …`, every entry on a nameless creator and no Error, review revZ3r2). After a
/// comma it stays the fail-safe: `{$^{2}$H} Lab` is an isotope (the row above, and `a_wrapped_isotope_is_entry_text`).
#[test]
fn a_group_a_mark_opens_at_a_lines_start_starts_at_its_brace() {
  let template = repro!("revtex_affiliation_line_opening_group_starts_at_its_brace");
  let (log, xml) = in_a_directory(template, &[], &[]);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators_are(&xml, &[
    &["Ann Able 1", "affiliation=1 Univ X, Moscow, Russia"],
    &[
      "Bob Baker 1,2",
      "affiliation=1 Univ X, Moscow, Russia",
      "affiliation=2 Univ Y, Moscow, Russia",
    ],
    &["Cat Cole 3", "affiliation=3 Lab, ENS, Paris, France"],
  ]);
  // (the field's whole, and the same two entries however the line starts: `\\[1mm]`, a switch, a group around it, LaTeX's `\(`; an isotope right after a line break is cut as on
  // the base, the digested list no telling it from an entry)
  let field = template
    .split_once("\n\\affiliation{")
    .and_then(|(_, rest)| rest.split_once("\n}\n"))
    .expect("the repro's field")
    .0;
  let (ann, bob) = (
    &["Ann Able 1", "affiliation=1 Univ X, Moscow"][..],
    &[
      "Bob Baker 1,2",
      "affiliation=1 Univ X, Moscow",
      "affiliation=2 Univ Y, Rome",
    ][..],
  );
  for row in [
    "~\\\\[1mm] {$^1$Univ X,} Moscow\\\\ {$^2$Univ Y,} Rome",
    "~\\\\ {\\small $^1$Univ X,} Moscow\\\\ {\\small $^2$Univ Y,} Rome",
    "~\\\\ {{$^1$Univ X,} Moscow}\\\\ {{$^2$Univ Y,} Rome}",
    "~\\\\ {\\( ^1 \\) Univ X,} Moscow\\\\ {\\( ^2 \\) Univ Y,} Rome",
  ] {
    let (log, xml) = in_a_directory(&template.replace(field, row), &[], &[]);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{row}: {log}"
    );
    creators_are(&xml, &[ann, bob, &["Cat Cole 3"]]);
  }
  let (xml, _) = dump(&template.replace(field, "~\\\\ {$^1$Univ X,} Moscow\\\\ {$^2$H} Lab, Rome"));
  creators_are(&xml, &[
    ann,
    &[
      "Bob Baker 1,2",
      "affiliation=1 Univ X, Moscow",
      "affiliation=2 H Lab, Rome",
    ],
    &["Cat Cole 3"],
  ]);
}

/// RED: a mark's group that is no formula's alone and stands at no line's start, cut on the base by walking back, uncut now: nothing lost, the marks in the text. A `\textsuperscript` mark at a
/// line's start goes to a nameless creator. Review revZ3r2.
#[test]
#[ignore = "RED: `starts_at_its_brace` asks formulas, and of a group going on after one, only at a line's start"]
fn affiliation_cut_group_siblings_stay_uncut_red() {
  let template = repro!("affiliation_cut_group_siblings_stay_uncut_red");
  let (author, field) = (
    "Ann Able$^{1}$, Bob Baker$^{1,2}$, Cat Cole$^{3}$",
    "{$^{1}$ Univ X,} Moscow, {$^{2}$Univ Y,} Moscow, {$^{3}$Univ Z,} Moscow",
  );
  let unmarked = |lines: (&str, &str)| {
    vec![vec![
      "Ann Able".to_string(),
      format!("affiliation={}", lines.0),
      format!("affiliation={}", lines.1),
    ]]
  };
  let mut wrong = Vec::new();
  for (row, expected) in [
    (
      "{\\textsuperscript{1}} Univ X, {\\textsuperscript{2}} Univ Y",
      unmarked(("1 Univ X", "2 Univ Y")),
    ),
    (
      "{\\textsuperscript{1}x} Univ X, {\\textsuperscript{2}x} Univ Y",
      unmarked(("1 x Univ X", "2 x Univ Y")),
    ),
    (
      "{$^1$2} Univ X, {$^2$3} Univ Y",
      unmarked(("1 2 Univ X", "2 3 Univ Y")),
    ),
  ] {
    let (xml, _) = dump(
      &template
        .replace(author, "Ann Able\\thanks{note}")
        .replace(field, row),
    );
    if creators(&xml) != expected {
      wrong.push(format!("{row}: {:?}", creators(&xml)));
    }
  }
  let marked = |ann: &str, bob: &str, cat: &str| {
    let mut creators = vec![
      vec!["Ann Able 1".to_string(), format!("affiliation={ann}")],
      vec![
        "Bob Baker 1,2".to_string(),
        format!("affiliation={ann}"),
        format!("affiliation={bob}"),
      ],
      vec!["Cat Cole 3".to_string()],
    ];
    if !cat.is_empty() {
      creators[2].push(format!("affiliation={cat}"));
    }
    creators
  };
  for (row, expected) in [
    (
      field,
      marked("1 Univ X, Moscow", "2 Univ Y, Moscow", "3 Univ Z, Moscow"),
    ),
    (
      "~\\\\ {\\textsuperscript{1}Univ X,} Moscow\\\\ {\\textsuperscript{2}Univ Y,} Rome",
      marked("1 Univ X, Moscow", "2 Univ Y, Rome", ""),
    ),
    (
      "{$^1$Univ X,} Moscow\\endgraf {$^2$Univ Y,} Rome",
      marked("1 Univ X, Moscow", "2 Univ Y, Rome", ""),
    ),
    // (review revZ3r3: separators the Walker reads no line break from, each cut on int142a)
    (
      "{$^1$Univ X,} Moscow\\break {$^2$Univ Y,} Rome",
      marked("1 Univ X, Moscow", "2 Univ Y, Rome", ""),
    ),
    (
      "{$^1$Univ X,} Moscow\\hfill\\break {$^2$Univ Y,} Rome",
      marked("1 Univ X, Moscow", "2 Univ Y, Rome", ""),
    ),
    (
      "{$^1$Univ X,} Moscow\n{$^2$Univ Y,} Rome",
      marked("1 Univ X, Moscow", "2 Univ Y, Rome", ""),
    ),
    (
      "{$^1$Univ X,} Moscow; {$^2$Univ Y,} Rome",
      marked("1 Univ X, Moscow", "2 Univ Y, Rome", ""),
    ),
    // (review revZ3r3: a Break in a box already closed is no line start, so this stays the isotope's fail-safe)
    (
      "$^1$Univ X \\parbox{3cm}{A\\\\} {$^{2}$H} Lab, Rome $^{2}$Univ B",
      marked("1 Univ X A 2 H Lab, Rome", "2 Univ B", ""),
    ),
  ] {
    let (xml, _) = dump(&template.replace(field, row));
    if creators(&xml) != expected {
      wrong.push(format!("{row}: {:?}", creators(&xml)));
    }
  }
  assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A group of no formula printing symbols alone is a mark's own group: a piece starting at its symbol starts at the brace. Witness 1605.02212. Repro
/// sectioning-frontmatter/address_symbol_mark_in_its_own_group.
#[test]
fn a_symbols_group_is_a_marks_own() {
  let tex = repro!("address_symbol_mark_in_its_own_group");
  let (xml, _) = dump(tex);
  creators_are(&xml, &[&["Ann Able*", "address=* Univ X"], &[
    "Bob Baker**",
    "address=** Univ Y",
  ]]);
}

/// RED: a speculative run evaluates no Lua (`a_speculative_run_evaluates_no_lua`), so a cut that needs what a chunk prints does not come: two entries on the base, one now, nothing lost. Review
/// revZ3r2.
#[test]
#[ignore = "RED: what a chunk prints is not in the rehearsal's reading (`lua_bridge::lua_exec`)"]
fn rehearsal_no_lua_loses_a_cut_red() {
  let (xml, _) = dump(repro!("rehearsal_no_lua_loses_a_cut_red"));
  creators_are(&xml, &[&[
    "Ann Able",
    "affiliation=1 Univ X",
    "affiliation=2 Univ Y",
    "email=ann@x.org",
  ]]);
}

/// A run of a cut call holds its environments: what `\\end` closes or prints holds a position no box stops at (review revZ3 S4; the hostile suite's `long_tab`, `nice`, `newenv` in revtex's
/// `\\affiliation`).
#[test]
fn a_cut_holds_its_environments() {
  let template = repro!("affiliation_cut_holds_its_environments");
  let table = "\\begin{tabular}{cc}a&b\\\\ c&d\\end{tabular}";
  assert!(template.contains(table));
  let (xml, _) = dump(template);
  creators_are(&xml, &[&[
    "Ann Able",
    "affiliation=1 Univ X a b c d",
    "affiliation=2 Univ Y",
  ]]);
  let (xml, _) = dump(&template.replace(table, "\\newenvironment{ee}{B}{E}\\begin{ee}x\\end{ee}"));
  creators_are(&xml, &[&[
    "Ann Able",
    "affiliation=1 Univ X BxE",
    "affiliation=2 Univ Y",
  ]]);
}

/// A block nothing prints may loop: the rehearsal stops at its budget (`LATEXML_REHEARSAL_READ_BUDGET`, a guard's small one) and says so in one Warning, with the document kept: the names of
/// the call that counts and the body.
#[test]
fn a_looping_block_ends_in_the_rehearsal_budget() {
  let (log, _) = in_a_directory(repro!("rehearsal_budget_ends_a_looping_block"), &[], &[(
    "LATEXML_REHEARSAL_READ_BUDGET",
    "20000",
  )]);
  assert_eq!(
    (error_count(&log), log.matches("\nFatal:").count()),
    (0, 0),
    "{log}"
  );
  assert_eq!(
    log.matches("Warning:frontmatter:rehearsal_budget").count(),
    1,
    "{log}"
  );
}

/// Converts `tex` in a directory of its own as `t.tex` beside `files`, with `env`: its log and its XML. For what a run reads by the document's name (a `.bbl`) or a variable.
pub(super) fn in_a_directory(
  tex: &str,
  files: &[(&str, &str)],
  env: &[(&str, &str)],
) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write a file");
  }
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .envs(env.iter().copied())
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml"))
    .unwrap_or_else(|_| panic!("no t.xml: {log}"));
  (log, xml)
}
