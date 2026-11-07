//! Guards for Z3 (`docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`, Stage 3; "What a rollback leaves"): what a span of a field holds, read
//! from the digested list (`writer::Field`), and the rehearsal's isolation. Each guard is a different code path: `a_cut_takes_each_formula_whole`
//! a piece's start and end in a formula, `a_cut_takes_a_group_it_closes` the brace balance, `a_mark_in_its_wrapper_…` a mark's own print and a
//! block's line, `a_declarations_span_…` the settled declarations; the `the_rehearsal_leaves_no_trace_*` table the journal and what
//! lives outside it, by theme, and its `_written` row the written family's path, where the writer runs too.

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
    // (the body's `\read` of the closed stream defines nothing, in the real digestion alone)
    ("closein", r"\closein\rs ", (1, 0)),
    ("openin", r"\openin\rs=iso_side.tex ", (0, 0)),
    ("random", r"\pdfuniformdeviate 1000 ", (0, 0)),
    ("random seed", r"\pdfsetrandomseed 5 ", (0, 0)),
    ("missing key", r"\setkeys{zz}{a=b}", (0, 1)),
  ]);
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
