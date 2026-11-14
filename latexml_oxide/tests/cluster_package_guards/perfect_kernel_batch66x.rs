//! Red/green guards for batch 66x: a list's LaTeXML-only frame (`\list`/`\@trivlist` open no TeX group) is ended by the
//! `}`/`\endgroup` of the environment around it (K9 mode-switch study row L,
//! `agent_reports/2026-10-14_k9_mode_switch_study.md`).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The text after a list closed only by its environment's group is outside the list.
fn after_is_outside_the_list(xml: &str) {
  let at = xml
    .find("After.")
    .unwrap_or_else(|| panic!("no After.: {xml}"));
  let (opened, closed) = (xml[..at].rfind("<itemize"), xml[..at].rfind("</itemize>"));
  assert!(closed > opened, "After. inside the list: {xml}");
}

/// `\def\refs{\list{}{…}}` with no `\endrefs`: `\end{refs}`'s `\endgroup` ends the list (iopconf.sty:283 `\references`,
/// astro-ph9806019; jT.cls:170 `resume`, 0902.3464).
#[test]
fn list_closed_by_its_environments_group() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/lists/list_closed_by_its_environments_group.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<itemize").count(), 1, "{xml}");
  assert_eq!(xml.matches("<item ").count(), 2, "{xml}");
  after_is_outside_the_list(&xml);
}

/// INFORMS's `\Trivlist` (a raw `\@trivlist`) ended by its own `\endTrivlist` copy, which closes no frame: `\end{proof}`'s
/// `\endgroup` ends the list (2106.10067).
#[test]
fn trivlist_closed_by_its_environments_group() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/lists/trivlist_closed_by_its_environments_group.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Trivial."), "{xml}");
  after_is_outside_the_list(&xml);
}
