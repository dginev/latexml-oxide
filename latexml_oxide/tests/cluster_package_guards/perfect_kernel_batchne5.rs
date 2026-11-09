//! Guards for NE5: the papers where `=2` failed or erred only there, the flip's no-harm gate. One guard per
//! mechanism, each off and at `=2`: a declaration whose span holds a macro reading past it can't be replayed, the block
//! written whole (2505.04436's and 1511.09009's `\footremember`, a Fatal each); a name's run holding a conditional cut apart is
//! its text (1409.0597's `\ifFull and \else \and \fi`); a name's run its balance grew over other names is its text (1208.0597's
//! `flushleft` of 13 names).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// Convert `tex`: the log and the XML.
fn run(tex: &str) -> (String, String) {
  let (log, xml, ()) =
    convert_files_with_setup_then(tex, &[], Some("ar5iv.sty"), move || {}, |_| ());
  (log, xml)
}

/// `\newcommand{\footremember}[2]{\footnote{#2}\newcounter{#1}\setcounter{#1}{\value{footnote}}}` after four names: its
/// definitions follow the note it printed, so each declaration's span is the macro alone, and `=2` replayed
/// `\footremember\footremember\footremember\footremember`, the last reading past its entry (2 x Missing {an undefined
/// `\lx@thistheorem`; the papers' 20 and a Fatal, 2505.04436, 1511.09009). The block is written whole, Perl's whole-block
/// creator, a counted Warning; its notes and the recalled mark kept. Off reads five creators, no Error.
#[test]
fn a_declaration_reading_past_its_span_writes_the_block_whole() {
  let (log, xml) = run(repro!("ne5_decl_reads_past"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "The frontmatter emitter can't replay a definition of its author block apart from what the block prints; the \
       block is written whole, as Perl's whole-block creator"
    ),
    "{log}"
  );
  assert_eq!(
    creators(&xml),
    [row(&[
      "Ann Able and Bob Baker and Cat Cole \u{2020} \u{2020} footnote \u{2020} and Dan Dunn and Eve Ernst",
      "note=Univ A. Email: ann@a.at.",
      "note=Univ B. Emails: bob@b.il, cat@b.il.",
      "note=Univ C.",
      "note=Univ D.",
    ])],
    "{xml}"
  );
}

/// `\ifFull and \else \and \fi Joseph A. Simons` (journal.cls): `=2` wrote the second name from a run holding the `\else
/// \and \fi` its rehearsal skipped — an `\else` outside a conditional, a tabular's group ended in the wrong mode, 6 Errors,
/// the name lost (1409.0597). A run holding a conditional cut apart is refused: the name is its text, a counted Warning.
/// Off is the old reader's, its `\else` creator an Error.
#[test]
fn a_name_run_holding_a_cut_conditional_is_its_text() {
  let (log, xml) = run(repro!("ne5_cond_cut"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(r#"The frontmatter emitter writes "Joseph A. Simons" as its plain text"#),
    "{log}"
  );
  let dept = "affiliation=Dept. of Computer Science, Univ. of California, Irvine, USA";
  assert_eq!(creators(&xml), [
    row(&["Michael T. Goodrich", dept]),
    row(&["Joseph A. Simons", dept]),
  ]);
}

/// `\parbox[t]{\textwidth}{\vspace{-1.3cm}\begin{flushleft} Ann Able$^{1,2}$\thanks{…}, Bob Baker$^{1}$, Cat Cole$^{2}$
/// \end{flushleft}}`: the first name's run, grown back to the group the names sit in, took the `flushleft` whole, every name
/// in its creator and its `\thanks` twice (merged_creators: 1208.0597's 13 names). A run its balance grew over another name
/// is refused: the name is its text, a counted Warning, the creators off's.
#[test]
fn a_name_run_grown_over_other_names_is_its_text() {
  let expected = [
    row(&[
      "Ann Able",
      "affiliation=Univ A, City A",
      "affiliation=Univ B, City B",
    ]),
    row(&["Bob Baker", "affiliation=Univ A, City A"]),
    row(&["Cat Cole", "affiliation=Univ B, City B"]),
  ];
  // (the `\thanks` once, as off writes it)
  let thanks = |xml: &str| xml.matches("E-mail: ann@a.edu").count();
  let (log, xml) = run(repro!("ne5_env_names"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(r#"The frontmatter emitter writes "Ann Able" as its plain text"#),
    "{log}"
  );
  assert_eq!(creators(&xml), expected, "{xml}");
  assert_eq!(thanks(&xml), 1);
}
