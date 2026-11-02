//! Red/green guards for batch 99a: the frontmatter API's Cut (OXIDIZED_DESIGN_DIVERGENCES #954; placement step 0 of
//! `docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, "one item per printed entry; marked lists cut at their marks"). An
//! affiliation, alternate affiliation or address contact holding two or more entries each led by a mark the queued names
//! request is queued as one contact per entry, each labelled by its mark and showing it, in
//! `\lx@annotate@frontmatter` itself — for every binding, the raw classes' stores included, and before the frontmatter
//! observer's ledger records the call. Census127 cluster 2 (acmart 2602.11605, 2506.18881, cidr 2408.03013, asp2006
//! 1002.4968, mem astro-ph0701509, aa 2410.00577, jfm physics0607157 and 1302.1594) and cluster 5 (the spacing-nucleus
//! marks: hep-ph0408088, hep-ph0302245, 1611.04447, 1203.0253, 2312.14939 — ours already right where the store reaches
//! the API; the census's reading of `$^{~a,b}$` and `\,\,\,$^{\dagger\,1\,2}$` is the scorer's). Each guard converts
//! its repros under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`,
//! asserts the same XML both ways, no Error and no Warning, and every creator (name and contacts in order), and the same
//! creators with the observer's writer off (the old reader alone: the Cut is the API's, so the two engines agree). One guard
//! per path: the Cut of a contact with no wrapper (acmart's `\lx@add@contact`, the address part's wrapper repeated, a
//! `\quad`, a `;` or a `\\` between entries), the boundary that keeps a mark no name requests as the entry's text, the
//! shared reader of a nucleus mark and a name's tie mark, a declared sequence and sharing replaced by the marks (aa's
//! sole institute, `\inst{n}` read as the mark it prints, an address-only entry), a raw package's store
//! (`\lx@add@affiliation`'s `\lx@affiliation@withinst` wrapper kept), a binding's declaration (jfm), and the bindings'
//! own cuts left as they are (authblk, IEEEtran, aa's `\and`).
use std::{path::Path, process::Command};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
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

/// The raw package [`a_raw_store_is_cut_at_its_marks`] loads, beside its repro.
const CUTINST_STY: &str =
  include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/cutinst.sty");

/// Converts `tex` (with `files` beside it) from the binary, `--streaming=true` when `streaming`, the frontmatter
/// observer switched off when `writer_off` (`LATEXML_FM_SHADOW=0`: the old reader's output alone): the log and the XML.
fn convert(
  tex: &str,
  files: &[(&str, &str)],
  streaming: bool,
  writer_off: bool,
) -> (String, String) {
  if files.is_empty() && !writer_off {
    return cli_conversion(tex, streaming);
  }
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write a file beside t.tex");
  }
  let mut command = Command::new(bin);
  command
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      if streaming {
        "--streaming=true"
      } else {
        "--streaming=false"
      },
    ])
    .current_dir(workdir.path());
  if writer_off {
    command.env("LATEXML_FM_SHADOW", "0");
  }
  let output = command.output().expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml"))
    .unwrap_or_else(|_| panic!("no t.xml: {log}"));
  (log, xml)
}

/// Converts `tex` (with `files` beside it) eager and `--streaming`, checks each has no Error and no Warning and both give
/// the same XML, and returns its creators — and, the writer off (the old reader alone), its creators then.
#[track_caller]
fn all_ways(tex: &str, files: &[(&str, &str)]) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  let (log, eager) = convert(tex, files, false, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (streamed_log, streamed) = convert(tex, files, true, false);
  assert_eq!(
    (error_count(&streamed_log), warning_count(&streamed_log)),
    (0, 0),
    "{streamed_log}"
  );
  assert_eq!(eager, streamed);
  let (off_log, off) = convert(tex, files, false, true);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  (creators(&eager), creators(&off))
}

/// [`all_ways`], where the old reader alone gives the same creators: the Cut is the API's, before either engine reads
/// the call.
#[track_caller]
fn both_ways(tex: &str, files: &[(&str, &str)]) -> Vec<Vec<String>> {
  let (written, today) = all_ways(tex, files);
  assert_eq!(written, today, "the old reader alone (the writer off)");
  written
}

/// acmart's one `\institution` holding marked entries is cut at its marks in the API (`cut_marked_contact`, through
/// `\lx@add@contact[role=affiliation,annotate=new]`): the address part's wrapper repeated on each piece, the entries
/// parted by a `\quad` (2506.18881), after a `;` it trims (2602.11605) and by a run and a `\\` line (cidr's 2408.03013),
/// what follows the wrapper (`\country{X}`) with the last. Each entry shows its mark. Was: every author the whole call.
#[test]
fn an_institution_is_cut_at_its_marks() {
  assert_eq!(
    both_ways(repro!("cut_acmart_institution_entries_at_their_marks"), &[]),
    [
      vec!["Ann Able 1", "affiliation=1 Univ A"],
      vec!["Bob Baker 2", "affiliation=2 Univ B , X"],
      vec!["Cat Cole 1", "affiliation=1 Univ A"],
    ]
  );
  assert_eq!(
    both_ways(
      repro!("cut_acmart_institution_entries_after_separators"),
      &[]
    ),
    [
      vec!["Ann Able 1", "affiliation=1 Univ A, Town"],
      vec!["Bob Baker 2", "affiliation=2 Univ B, City , X"],
      vec!["Cat Cole 1", "affiliation=1 Univ A, Town"],
    ]
  );
  assert_eq!(
    both_ways(repro!("cut_acmart_institution_lines_and_runs"), &[]),
    [
      vec!["Ann Able 1", "affiliation=1 Univ A"],
      vec!["Bob Baker 2", "affiliation=2 Univ B"],
      vec!["Cat Cole 3", "affiliation=3 Univ C , X"],
    ]
  );
}

/// A number mark no queued name requests, after a word and a plain space, is its entry's text, not the next entry's
/// label (`notes::split_marked_pieces`: `$^1$Laboratory for $^3$He Studies`, the 63i isotope); the `\quad` before the
/// requested `$^2$` still parts the entries. Was: both authors the whole call.
#[test]
fn an_isotope_mark_is_its_entrys_text() {
  assert_eq!(
    both_ways(repro!("cut_acmart_isotope_mark_is_entry_text"), &[]),
    [
      vec!["Ann Able 1", "affiliation=1 Laboratory for 3 He Studies"],
      vec!["Bob Baker 2", "affiliation=2 Univ B , X"],
    ]
  );
}

/// Every spelling of a mark, the same reader: an entry's mark on a spacing nucleus (`${}^a$`, hep-ph0408088's shape) is
/// read as the `$^{a}$` it prints past the address part's wrapper (`relocate::spaced_marks_as_marks`), a name's mark
/// with a leading tie (`$^{~a,b}$`, hep-ph0302245) requests each of its labels, and so does a number range (`$^{1-3}$`,
/// 2012.15817: `queue_digest::spelled_ranges`, the observer's reader, now `clean_frontmatter_labels`'s too), the run's
/// `and` trimmed with the separator. Was: every author the whole call.
#[test]
fn every_mark_spelling_parts_the_entries() {
  assert_eq!(both_ways(repro!("cut_acmart_nucleus_marks"), &[]), [
    vec!["Ann Able a", "affiliation=a Univ A"],
    vec!["Bob Baker b", "affiliation=b Univ B , X"],
    vec!["Cat Cole a", "affiliation=a Univ A"],
  ]);
  assert_eq!(both_ways(repro!("cut_acmart_tie_marks"), &[]), [
    vec![
      "Ann Able a , b",
      "affiliation=a Univ A",
      "affiliation=b Univ B , X"
    ],
    vec!["Bob Baker a", "affiliation=a Univ A"],
    vec!["Cat Cole b", "affiliation=b Univ B , X"],
  ]);
  assert_eq!(both_ways(repro!("cut_acmart_range_marks"), &[]), [
    vec![
      "Ann Able 1-3",
      "affiliation=1 Univ A",
      "affiliation=2 Univ B",
      "affiliation=3 Univ C , X"
    ],
    vec!["Bob Baker 2", "affiliation=2 Univ B"],
    vec!["Cat Cole 3", "affiliation=3 Univ C , X"],
  ]);
}

/// aa's sole `\institute` (`labelseq=affiliation,_share=authors`, both replaced by the marks) holding `\inst{n}` lines,
/// read as the `\textsuperscript{n}` aa.cls:759 prints, and an `\inst{*}` line holding only an address — the e-mail of
/// the author showing `*`, no empty entry left (2410.00577); and its address line behind an `email:` label, the list's
/// address run, each address its speller's (`affiliation_calls`, astro-ph0405186). Was: one nameless creator holding
/// every institute.
#[test]
fn a_sole_institute_is_cut_at_its_inst_marks() {
  assert_eq!(
    both_ways(repro!("cut_aa_sole_institute_inst_lines"), &[]),
    [
      vec!["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"],
      vec!["Bob Baker", "affiliation=2 Univ B"],
      vec!["Cat Cole", "affiliation=1 Univ A"],
    ]
  );
  assert_eq!(
    both_ways(repro!("cut_aa_institute_email_label_line"), &[]),
    [
      vec![
        "Ann Able 1",
        "affiliation=1 Univ A, Town",
        "email=ann@a.edu"
      ],
      vec![
        "Bob Baker 2",
        "affiliation=2 Univ B, City",
        "email=bob@b.edu"
      ],
      vec![
        "Cat Cole 2",
        "affiliation=2 Univ B, City",
        "email=cat@b.edu"
      ],
    ]
  );
}

/// A raw package's institute store (`frontmatter_stores.rs`'s `\lx@add@affiliation[annotate=run]`, its
/// `\lx@affiliation@withinst` wrapper kept on each entry) holding a marked run is cut by the API, its trailing address
/// the e-mail of the author it spells (mem's astro-ph0701509: 40 authors each given all 16 institutes; asp2006's
/// 1002.4968). Was: every author the whole store (the old reader; the observer's writer re-cut this minimal shape, and
/// declined the witnesses').
#[test]
fn a_raw_store_is_cut_at_its_marks() {
  let (written, today) = all_ways(repro!("cut_raw_institute_store_run"), &[(
    "cutinst.sty",
    CUTINST_STY,
  )]);
  assert_eq!(written, [
    vec!["Ann Able", "affiliation=1 Univ A", "email=ann@a.edu"],
    vec!["Bob Baker", "affiliation=2 Univ B"],
    vec!["Cat Cole", "affiliation=1 Univ A"],
  ]);
  // (the old reader alone: the store cut by the API, its address still in the entry it is printed in — the take-out of
  // an entry's address by its spelling is the observer's, OD #938)
  assert_eq!(today, [
    vec!["Ann Able", "affiliation=1 Univ A"],
    vec!["Bob Baker", "affiliation=2 Univ B; ann@a.edu"],
    vec!["Cat Cole", "affiliation=1 Univ A"],
  ]);
}

/// jfm's `\affiliation` list led by marks the names show is the names' affiliations (jfm_cls.rs, through
/// `\lx@add@affiliation` and the API's Cut: physics0607157, 1302.1594). Was: one note at the block, no author's.
#[test]
fn a_jfm_affiliation_list_is_cut_at_its_marks() {
  assert_eq!(both_ways(repro!("cut_jfm_affiliation_marked_list"), &[]), [
    vec!["Ann Able 1", "affiliation=1 Univ A"],
    vec!["Bob Baker 2", "affiliation=2 Univ B"],
    vec!["Cat Cole 1", "affiliation=1 Univ A"],
  ]);
}

/// The controls: a list a binding's own call cut before the API (authblk's `\affil{$^1$Univ A $^2$Univ B}`, IEEEtran's
/// `\IEEEauthorblockA{$^1$Univ A\\ $^2$Univ B}`, both through `affiliation_calls`) or split itself (aa's
/// `\institute{Univ A \and Univ B}`) reaches the API as its entries, which it cuts no further: each author exactly the
/// entries its marks request, as before.
#[test]
fn a_binding_cut_list_is_not_cut_again() {
  let lines = [
    vec!["Ann Able", "affiliation=Univ A"],
    vec!["Bob Baker", "affiliation=Univ B"],
    vec!["Cat Cole", "affiliation=Univ A"],
  ];
  assert_eq!(
    both_ways(repro!("cut_control_authblk_affil_run"), &[]),
    lines
  );
  assert_eq!(
    both_ways(repro!("cut_control_aa_institute_and"), &[]),
    lines
  );
  assert_eq!(both_ways(repro!("cut_control_ieee_blocka_lines"), &[]), [
    vec!["Ann Able 1", "affiliation=Univ A"],
    vec!["Bob Baker 2", "affiliation=Univ B"],
    vec!["Cat Cole 1", "affiliation=Univ A"],
  ]);
}
