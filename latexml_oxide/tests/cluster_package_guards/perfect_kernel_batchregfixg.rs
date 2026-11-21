//! Red/green guards for batch REGFIXG, a regression of the v0.8.0 per-paper check (clean on 2026-10-07, Error on the
//! release candidate): bullsrsl.cls's author count. Repros under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/class_author_helper/`.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// bullsrsl.cls:326-327 (2312.08217, 2607.02675): the refused `\def\author{\@ifnextchar[\@author{\@author[]}}` reads
/// each call with the class's own `\@author[#1]#2#3`, which counts the authors (:358) that `{authorcontributions}`
/// tests (:958-961, "Unwanted authorcontributions" for one). The kernel's `\author` stores its value in `\@author`
/// after the first call's replay; every replay binds the class's helper back (sect05.rs `class_store_helper`), so the
/// count is the number of calls — two, and three with a third call (each later call, not only the second).
#[test]
fn class_author_helper_counts_every_author() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_author_helper/class_author_helper_counts_every_author.tex"
  );
  let files = [(
    "cntauth.cls",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_author_helper/cntauth.cls"
    ),
  )];
  let three = tex.replace(
    "\\author[y]{Bob}{Beta}\n",
    "\\author[y]{Bob}{Beta}\n\\author{Cy}{Gamma}\n",
  );
  assert_ne!(three, tex);
  for (source, count) in [(tex, 2), (three.as_str(), 3)] {
    let (log, xml) = convert_files_with(source, &files, Some("ar5iv.sty"));
    assert!(!log.contains("Fatal:"), "{log}");
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_element(
      &xml,
      "p",
      &[],
      &format!(r#"<p xml:id="p1.1">Authors: {count}.</p>"#),
    );
  }
}
