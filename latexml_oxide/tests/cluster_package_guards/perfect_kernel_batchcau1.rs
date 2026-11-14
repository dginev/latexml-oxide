//! Guard for CAU1: biblatex makes every name and list field a counter (biblatex.sty:7660-7680), so a document's own
//! `\stepcounter{author}` and `\the\c@author` count as in pdflatex (2606.16762, 2606.26281: `\c@author` undefined,
//! 1,000 errors and a Fatal in the 2606 sandbox rerun).

use super::perfect_kernel_batch46::{convert, error_count};

#[test]
fn biblatex_name_list_fields_are_counters() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/biblatex_name_list_fields_are_counters.tex"
  );
  let (log, xml) = convert(tex, false);
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(
    xml.contains("Authors: 2 (2); Ann Able and Bob Baker."),
    "{xml}"
  );
}
