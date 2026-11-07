//! Guards for Z3 (`docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`, Stage 3; "What a rollback leaves"): what a span of a field holds, read
//! from the digested list (`writer::Field`), and the rehearsal's isolation. Each guard is a different code path: `a_cut_takes_each_formula_whole`
//! a piece's start and end in a formula, `a_cut_takes_a_group_it_closes` the brace balance, `a_mark_in_its_wrapper_…` a mark's own print and a
//! block's line, `a_declarations_span_…` the settled declarations; the `the_rehearsal_leaves_no_trace_*` table the journal and what
//! lives outside it, by theme, and its `_written` row the written family's path, where the writer runs too. Z3r2: `a_bindings_state_…` the package thread-locals
//! moved to the Value table, `a_rehearsals_read_…` the raw-read list, `a_speculative_run_evaluates_no_lua` the one state no rollback reaches, `a_declaration_in_a_body_…` where a body's definer runs,
//! `a_marks_own_group_…` the Walker's group fact, `a_cut_holds_its_environments` the run's environments, `a_looping_block_…` the budget. Z3r3: `a_group_a_mark_opens_…` the Walker's
//! line-start fact. The `#[ignore]`d `…_red` rows assert the right output of what the review filed and this batch leaves: `cargo nextest run --run-ignored only -E 'test(_red)'`.

use serde_json::json;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch77r::split_filecontents,
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

/// S2 of revZ2r2. A declaration's span is a run a replay can read: no closing brace of what opened before it (`\textbf{\def\xa{1}}`, the last
/// `\def` of an `\AtBeginDocument`), and none reads what another reads: a definition in the body of a macro the field defined runs at the
/// macro's call (`\m`, `\x` holding two, `\w`), at that call's position.
#[test]
fn a_declarations_span_is_a_balanced_run() {
  let (_, dump) = dump(repro!("event_declaration_span_is_a_balanced_run"));
  assert_eq!(
    blocks(&dump)[0]["declarations"],
    json!([
      {"kind": "Declaration", "span": [10, 15], "label": "\\xa", "nest": 1},
      {"kind": "Declaration", "span": [29, 34], "label": "\\xb"},
      {"kind": "Declaration", "span": [34, 39], "label": "\\xc"},
      {"kind": "Declaration", "span": [51, 57], "label": "\\m"},
      {"kind": "Declaration", "span": [57, 58], "label": "\\qq", "role": "global"},
      {"kind": "Declaration", "span": [67, 81], "label": "\\x"},
      {"kind": "Declaration", "span": [81, 82], "label": "\\y \\z"},
      {"kind": "Declaration", "span": [91, 100], "label": "\\w"},
      {"kind": "Declaration", "span": [100, 101], "label": "\\v"},
    ])
  );
}

/// The XML outside the creators.
fn outside_the_creators(xml: &str) -> String {
  let mut out = String::with_capacity(xml.len());
  let mut rest = xml;
  while let Some(start) = rest.find("<creator ") {
    out.push_str(&rest[..start]);
    let end = rest[start..].find("</creator>").expect("closed creator");
    rest = &rest[start + end + "</creator>".len()..];
  }
  out.push_str(rest);
  out
}

/// Each hostile input in `\HOSTILE`'s place in the template: the observer on and off give the same XML outside the creators, the same counts of
/// Errors, Warnings and Fatals, and the counts the table says. What runs in the block runs in the rehearsal and again in the real digestion, so
/// what a rollback left behind shows twice in the body's state line.
fn no_trace(class: &str, cases: &[(&str, &str, (usize, usize))]) {
  let (template, name, content) = split_filecontents(repro!("observer_rehearsal_leaves_no_trace"));
  let template = template.replace(
    "\\documentclass{article}",
    &format!("\\documentclass{{{class}}}"),
  );
  let mut wrong = Vec::new();
  for (case, hostile, expected) in cases {
    let tex = template.replace("Ann Able \\HOSTILE", &format!("Ann Able {hostile}"));
    let run = |on: bool| {
      let (log, xml, ()) = latexml::util::test::convert_files_with_setup_then(
        &tex,
        &[
          (name.as_str(), content.as_str()),
          (
            "iso_side.tex",
            "\\gdef\\qfromfile{F}\\global\\advance\\ca by 1\n",
          ),
        ],
        Some("ar5iv.sty"),
        move || latexml_engine::base_utilities::set_fm_shadow_override(Some(on)),
        |_| (),
      );
      (log, xml)
    };
    let (log_on, xml_on) = run(true);
    let (log_off, xml_off) = run(false);
    let counts = |log: &str| {
      (
        error_count(log),
        warning_count(log),
        log.matches("\nFatal:").count(),
      )
    };
    if outside_the_creators(&xml_on) != outside_the_creators(&xml_off) {
      wrong.push(format!(
        "{case}: the XML outside the creators differs:\n{xml_on}\n---\n{xml_off}"
      ));
    }
    if counts(&log_on) != counts(&log_off) {
      wrong.push(format!(
        "{case}: on {:?}, off {:?}\n{log_on}\n---\n{log_off}",
        counts(&log_on),
        counts(&log_off)
      ));
    }
    if (counts(&log_on).0, counts(&log_on).1) != *expected {
      wrong.push(format!(
        "{case}: {:?}, the table says {expected:?}\n{log_on}",
        counts(&log_on)
      ));
    }
  }
  assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// What ends or unbalances the flow: the document's end in any spelling (read by a speculative run, it ends nothing: 108b), and an unbalanced
/// conditional, group, math. An undefined macro's Error is the real digestion's alone.
#[test]
fn the_rehearsal_leaves_no_trace_of_the_flow() {
  no_trace("article", &[
    ("undefined", r"\undefinedmacro", (1, 0)),
    ("if", r"\iftrue", (0, 1)),
    ("fi", r"\fi", (1, 0)),
    ("begingroup", r"\begingroup", (1, 0)),
    ("endgroup", r"\endgroup", (1, 0)),
    ("bgroup", r"\bgroup", (2, 0)),
    ("egroup", r"\egroup", (1, 0)),
    ("math", r"$x", (1, 0)),
    ("end document", r"\end{document}", (6, 1)),
    ("stop", r"\stop", (0, 0)),
    ("@@end", r"\makeatletter\@@end", (0, 0)),
  ]);
}

/// What the journal covers, run twice: a register, a global list, a counter, a definition in the block, `\aftergroup` at its top level,
/// `\AtEndDocument`.
#[test]
fn the_rehearsal_leaves_no_trace_of_its_assignments() {
  no_trace("article", &[
    ("label", r"\label{foo}", (0, 0)),
    (
      "refstepcounter",
      r"\refstepcounter{section}\label{foo}",
      (0, 0),
    ),
    ("global def", r"\global\def\qg{G}\qg", (0, 0)),
    ("count", r"\global\advance\ca by 1 ", (0, 0)),
    ("toks", r"\global\ta=\expandafter{\the\ta x}", (0, 0)),
    ("macro", r"\xdef\qaccum{\qaccum x}", (0, 0)),
    ("catcode", r"\global\advance\catcode`\Q by 1 ", (0, 0)),
    (
      "definition",
      r"\def\qm{M}\qm\newcommand\qn{N}\newcounter{qc}\stepcounter{qc}",
      (0, 0),
    ),
    ("aftergroup", r"\aftergroup\lxafter", (0, 0)),
    ("atenddocument", r"\AtEndDocument{\gdef\qq{Q}}", (0, 0)),
    (
      "figure",
      r"\begin{figure}\caption{C}\label{fig:a}\end{figure}",
      (0, 0),
    ),
    ("section", r"\section{S}\label{sec:a}", (1, 0)),
  ]);
}

/// What lives outside the journal's tables, edited in place (`FRONTMATTER_DIGESTED_REFACTOR.md` "What a rollback leaves"), each ran twice before Z3:
/// a font's parameter, a box register's dimension, a stream's next line, a stream closed, the pdfTeX generator; and a key's "seen" set, whose
/// Warning went unsaid.
#[test]
fn the_rehearsal_leaves_no_trace_outside_its_journal() {
  no_trace("article", &[
    (
      "fontdimen",
      r"\fontdimen2\font=\dimexpr\fontdimen2\font+1pt\relax ",
      (0, 0),
    ),
    (
      "fontdimen named",
      r"\fontdimen3\fa=\dimexpr\fontdimen3\fa+1pt\relax ",
      (0, 0),
    ),
    (
      "box width",
      r"\wd\pb=\dimexpr\wd\pb+1cm\relax \ht\pb=\dimexpr\ht\pb+1cm\relax ",
      (0, 0),
    ),
    ("read", r"\read\rs to\qline ", (0, 0)),
    ("readline", r"\readline\rs to\qline ", (0, 0)),
    // (each of the next shows what it found before what it repeats: the real run redoes a close, an open or a seed the same, so what a rollback left is
    // only in what it reads first. The body's `\read` of the closed stream defines nothing, in the real digestion alone)
    (
      "closein",
      r"\read\rs to\qz \global\let\qd\qz \closein\rs ",
      (1, 0),
    ),
    (
      "openin",
      r"\read\rs to\qz \global\let\qd\qz \openin\rs=iso_side.tex ",
      (0, 0),
    ),
    ("random", r"\pdfuniformdeviate 1000 ", (0, 0)),
    (
      "random seed",
      r"\xdef\qd{\pdfuniformdeviate 1000}\pdfsetrandomseed 5 ",
      (0, 0),
    ),
    ("missing key", r"\setkeys{zz}{a=b}", (0, 1)),
  ]);
  // (a close before Z3 ended the mouth with the observer off too, the title-page check's run being as speculative: on and off agreed on an empty line, where pdflatex reads the first)
  let (template, name, content) = split_filecontents(repro!("observer_rehearsal_leaves_no_trace"));
  let tex = template.replace(
    "Ann Able \\HOSTILE",
    r"Ann Able \read\rs to\qz \global\let\qd\qz \closein\rs ",
  );
  let (_, xml, ()) = latexml::util::test::convert_files_with_setup_then(
    &tex,
    &[(name.as_str(), content.as_str())],
    Some("ar5iv.sty"),
    || latexml_engine::base_utilities::set_fm_shadow_override(Some(true)),
    |_| (),
  );
  assert!(xml.contains("—readline one"), "{xml}");
}

/// What reaches a file: `\input` missing and real (read twice, what it does undone once), a `\write18` no one runs, `\openout` and `\write` to a
/// stream the body reads back.
#[test]
fn the_rehearsal_leaves_no_trace_in_files() {
  no_trace("article", &[
    ("input missing", r"\input{nonexistent_file_zz}", (1, 0)),
    ("input real", r"\input{iso_side}", (0, 0)),
    (
      "write18",
      r"\immediate\write18{touch iso_w18_touched}",
      (0, 0),
    ),
    ("write", r"\immediate\write\ow{written line}", (0, 0)),
    (
      "openout",
      r"\newwrite\qw\immediate\openout\qw=iso_other.txt \immediate\write\qw{hello}\immediate\closeout\qw",
      (0, 0),
    ),
  ]);
}

/// The written family's path, where the writer runs after the rehearsal: a row of each theme. `\affiliation` is left out of the template: the
/// block is the author's.
#[test]
fn the_rehearsal_leaves_no_trace_written() {
  no_trace("revtex4-1", &[
    ("end document", r"\end{document}", (6, 1)),
    ("bgroup", r"\bgroup", (2, 0)),
    ("count", r"\global\advance\ca by 1 ", (0, 0)),
    ("read", r"\read\rs to\qline ", (0, 0)),
    (
      "figure",
      r"\begin{figure}\caption{C}\label{fig:a}\end{figure}",
      (0, 0),
    ),
  ]);
}

/// A binding's document-wide state (`state::DocumentState`): pict2e's graph, chemnum's numbers and first uses, biblatex's categories, a stub's report, what a thread-local kept of a run
/// no one prints (revZ3 B1, "What a rollback leaves"). Each is one conversion on and off, and what pdflatex prints: the graph's body picture holds the block's unstroked path, once.
#[test]
fn a_bindings_state_stands_for_one_run() {
  let on_off = |tex: &str| {
    let run = |on: bool| {
      let (log, xml, ()) = latexml::util::test::convert_files_with_setup_then(
        tex,
        &[],
        Some("ar5iv.sty"),
        move || latexml_engine::base_utilities::set_fm_shadow_override(Some(on)),
        |_| (),
      );
      (error_count(&log), warning_count(&log), xml)
    };
    let (on, off) = (run(true), run(false));
    assert_eq!(on, off);
    on
  };
  let lines = |xml: &str| -> Vec<String> {
    xml
      .split("<line points=\"")
      .skip(1)
      .map(|line| line.split('"').next().unwrap_or_default().to_string())
      .collect()
  };
  let graph = repro!("rehearsal_binding_state_pict2e");
  let block = "\\title{T}\n\\author{Ann Able \\begin{picture}(1,1)\\moveto(0,0)\\lineto(1,1)\\end{picture} \\and Bob Baker}\n";
  assert!(graph.contains(block));
  let in_the_preamble = graph.replace(block, "").replace(
    "\\begin{document}\n",
    &format!("{block}\\begin{{document}}\n"),
  );
  for tex in [graph.to_string(), in_the_preamble.clone()] {
    let (errors, warnings, xml) = on_off(&tex);
    assert_eq!((errors, warnings), (0, 0), "{xml}");
    assert_eq!(
      lines(&xml),
      ["0,0 1.38,1.38", "2.77,2.77 4.15,4.15"],
      "{xml}"
    );
  }
  // (nothing prints the block: pdflatex's `2 2 m 3 3 l S` alone)
  let (errors, warnings, xml) = on_off(&in_the_preamble.replace("\\maketitle\n", ""));
  assert_eq!((errors, warnings), (0, 0), "{xml}");
  assert_eq!(lines(&xml), ["2.77,2.77 4.15,4.15"], "{xml}");

  let (errors, warnings, xml) = on_off(repro!("rehearsal_binding_state_chemnum"));
  assert_eq!((errors, warnings), (0, 0), "{xml}");
  assert!(
    xml.contains(
      r#"<personname>Ann Able <text class="ltx_cmpd" xml:id="cmpd.b">1</text></personname>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"Body <text class="ltx_cmpd" xml:id="cmpd.a">2</text>, <ref class="ltx_cmpd" idref="cmpd.b">1</ref>."#),
    "{xml}"
  );

  let (errors, warnings, xml) = on_off(repro!("rehearsal_binding_state_discard"));
  assert_eq!((errors, warnings), (0, 1), "{xml}");

  let (tex, _, content) = split_filecontents(repro!("rehearsal_binding_state_biblatex"));
  let run = |on: &'static str| {
    in_a_directory(&tex, &[("t.bbl", content.as_str())], &[(
      "LATEXML_FM_SHADOW",
      on,
    )])
  };
  let (log, xml) = run("1");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("Empty bibliography"), "{log}");
  assert_eq!(xml, run("0").1);
}

/// A raw read in a speculative run is undone with it, so it is no read for `read_stand_in_copy` to skip the paper's copy for: a block nothing prints reads aas_macros.sty, the document's
/// own `\\input` then reads it too (review revZ3's "not checked"; the shape of 1309.5950's extras).
#[test]
fn a_rehearsals_read_is_no_read_of_a_copy() {
  let (tex, name, content) =
    split_filecontents(repro!("rehearsal_stand_in_copy_stays_the_documents"));
  let run = |on: bool| {
    let (log, xml, ()) = latexml::util::test::convert_files_with_setup_then(
      &tex,
      &[(name.as_str(), content.as_str())],
      Some("ar5iv.sty"),
      move || latexml_engine::base_utilities::set_fm_shadow_override(Some(on)),
      |_| (),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    xml
  };
  let xml = run(true);
  assert!(xml.contains("Body KMSX."), "{xml}");
  assert_eq!(xml, run(false));
}

/// A speculative run evaluates no Lua: the interpreter's globals are the one state no rollback reaches. A block nothing prints counts nothing, as lualatex prints it. Self-skips without a
/// host texlua, as `luacode_bridge` does.
#[test]
fn a_speculative_run_evaluates_no_lua() {
  if !std::process::Command::new("texlua")
    .arg("-v")
    .output()
    .is_ok_and(|output| output.status.success())
  {
    return;
  }
  let run = |on: bool| {
    let (log, xml, ()) = latexml::util::test::convert_files_with_setup_then(
      repro!("rehearsal_lua_state_stays_out"),
      &[],
      Some("ar5iv.sty"),
      move || latexml_engine::base_utilities::set_fm_shadow_override(Some(on)),
      |_| (),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    xml
  };
  let xml = run(true);
  assert!(xml.contains("Body 0."), "{xml}");
  assert_eq!(xml, run(false));
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

/// A mark's own group is what prints its formula alone, by the digested list: `{$^1$\\,} Univ X`'s spacing, a switch or a color around it, a comma after it; a command printing before it no
/// such group, nor a word after it, the isotope's shape (`perfect_kernel_batch107e::a_wrapped_isotope_is_entry_text`). A paper's shape: 1709.01760, 1203.3579, 2205.00632.
#[test]
fn a_marks_own_group_is_read_from_its_ink() {
  let template = repro!("revtex_affiliation_marks_own_group_by_its_ink");
  let shape = "{$^1$\\,} Univ X, {$^2$\\,} Univ Y";
  assert!(template.contains(shape));
  let each = |first: &str, second: &str| {
    (
      vec!["Ann Able 1".to_string(), format!("affiliation={first}")],
      vec!["Bob Baker 2".to_string(), format!("affiliation={second}")],
    )
  };
  let mut wrong = Vec::new();
  for (group, lines) in [
    // (what prints nothing beside the mark)
    (r"{$^N$\,}", ("1 Univ X", "2 Univ Y")),
    (r"{$^N$~}", ("1 Univ X", "2 Univ Y")),
    (r"{$^N$\ }", ("1 Univ X", "2 Univ Y")),
    (r"{$^N$\quad}", ("1 Univ X", "2 Univ Y")),
    (r"{$^N$\hspace{1pt}}", ("1 Univ X", "2 Univ Y")),
    (r"{$^N$\small}", ("1 Univ X", "2 Univ Y")),
    (r"{$^N$\relax}", ("1 Univ X", "2 Univ Y")),
    (r"{$^N$\bf}", ("1 Univ X", "2 Univ Y")),
    (r"{{$^N$}\,}", ("1 Univ X", "2 Univ Y")),
    (r"{\color{red}$^N$}", ("1 Univ X", "2 Univ Y")),
    (
      r"{\fontsize{9}{10}\selectfont $^N$}",
      ("1 Univ X", "2 Univ Y"),
    ),
    // (a list's comma beside it; a command printing before it, whose ink stays the entry's text)
    (r"{$^N$,}", ("1 , Univ X", "2 , Univ Y")),
    (r"{\S $^N$}", ("§ 1 Univ X", "§ 2 Univ Y")),
    (
      r"{\LaTeX $^N$}",
      ("L a T e X 1 Univ X", "L a T e X 2 Univ Y"),
    ),
  ] {
    let tex = template.replace(
      shape,
      &format!(
        "{} Univ X, {} Univ Y",
        group.replace('N', "1"),
        group.replace('N', "2")
      ),
    );
    let (xml, _) = dump(&tex);
    let (ann, bob) = each(lines.0, lines.1);
    if creators(&xml) != vec![ann.clone(), bob.clone()] {
      wrong.push(format!("{group}: {:?}", creators(&xml)));
    }
  }
  assert!(wrong.is_empty(), "{}", wrong.join("\n"));
  // (a word after it: the isotope's shape, one entry, its marks text)
  let (xml, _) = dump(&template.replace(shape, "{$^1$x} Univ X, {$^2$x} Univ Y"));
  creators_are(&xml, &[
    &["Ann Able 1", "affiliation=x Univ X, 2 x Univ Y"],
    &["Bob Baker 2"],
  ]);
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
  // (after a comma: the isotope, which the base gave Bob)
  let (xml, _) = dump(&template.replace(field, "$^1$Univ A, {$^{2}$H} Lab, Rome $^{2}$Univ B"));
  creators_are(&xml, &[
    &["Ann Able 1", "affiliation=Univ A, 2 H Lab, Rome"],
    &[
      "Bob Baker 1,2",
      "affiliation=Univ A, 2 H Lab, Rome",
      "affiliation=Univ B",
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
  let (log, xml) = in_a_directory(repro!("rehearsal_budget_ends_a_looping_block"), &[], &[(
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
  assert_eq!(warning_count(&log), 1, "{log}");
  creators_are(&xml, &[&["Ann Able"], &["Bob Baker"]]);
  assert!(xml.contains("Body stays."), "{xml}");
}

/// Converts `tex` in a directory of its own as `t.tex` beside `files`, with `env`: its log and its XML. For what a run reads by the document's name (a `.bbl`) or a variable.
fn in_a_directory(tex: &str, files: &[(&str, &str)], env: &[(&str, &str)]) -> (String, String) {
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
