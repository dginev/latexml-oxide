//! Red/green guards for batch REGFIXJ: `\end{document}`'s expansion carries latex.ltx:15255-15278's tail
//! (`\@checkend{document}\clearpage…\begingroup…\endgroup`), so an end-document hook that scans the rest of
//! `\enddocument` for those tokens finds them — ltxutil.sty:262-282's `\rvtx@enddocument@patch` (2004.09405,
//! 2401.03827, 2502.03242: `Fatal:Mouth:EoF` "Missing argument Until:\@checkend"). Repros under
//! `tools/perfect_kernel/repros/`.
use latexml::util::test::{convert_with, kpse_has};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// Both delimiters found and the grabbed tail re-emitted, as ltxutil's patch does: the hook's text lands, the tail's own
/// `\clearpage` emits no page break (the one `<pagination>` is the `\AtEndDocument` code's `\clearpage`), and the
/// `enddocument/afterlastpage` code the re-emitted tail carries runs once. A hook that drops what it grabbed before
/// `\@checkend` also ends the document cleanly.
#[test]
fn enddocument_hook_scans_the_kernel_tail() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/enddocument_hook_scans_the_kernel_tail.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert!(
    xml.contains("<p xml:id=\"p1.1\">Body.\nLast.[CHECK:document]</p>"),
    "{xml}"
  );
  assert_eq!(xml.matches("<pagination").count(), 1, "{xml}");
  assert_eq!(log.matches("AFTER-LAST-PAGE").count(), 1, "{log}");
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/enddocument_hook_scan_drops_the_tail.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  // (`\detokenize` in OT1, as pdflatex sets it: `\` is “, `_` is ˙; the dropped tokens are our tail's)
  assert!(
    xml.contains(
      "<p xml:id=\"p1.1\">Body.\nENDHOOK[GOT:“˙˙hook˙next enddocument “lx@enddocument@tail —document]</p>"
    ),
    "{xml}"
  );
  // the tail's `\clearpage`, dropped with its `\let`s, paginates: OD #1107's open residual pinned, not intended behaviour
  assert_eq!(
    xml.matches("<pagination role=\"newpage\"/>").count(),
    1,
    "{xml}"
  );
}

/// The re-emitted `\@checkend{document}` inside an environment left open: the finalizer's report of the open
/// environment (Perl's Warning) is the only diagnostic, not a second one beside it (pdflatex: `\@badend`'s Error).
#[test]
fn enddocument_hook_scan_in_an_open_environment_reports_once() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/enddocument_hook_scan_open_environment.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 1), "{log}");
  assert!(
    log.contains("Attempt to end document with open groups, environments or conditionals"),
    "{log}"
  );
  assert!(
    xml.contains("<p xml:id=\"S0.I1.i1.p1.1\">Open.\n[CHECK:document]</p>"),
    "{xml}"
  );
}

/// Raw ltxgrid/ltxutil with array (the three witnesses' classes load them): the patch runs, the document ends, and the
/// tabular keeps its cells, with no Error (array_sty defines `\@arrayleft`, REGFIXI); the two Warnings are ltxutil's own
/// (no class name, no `\@vspace` patches).
#[test]
fn ltxgrid_enddocument_patch_keeps_the_tabular() {
  if !kpse_has("ltxgrid.sty") {
    eprintln!(
      "SKIP ltxgrid_enddocument_patch_keeps_the_tabular: ltxgrid.sty is not in this TeX Live"
    );
    return;
  }
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/ltxgrid_enddocument_patch_tabular.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 2), "{log}");
  assert!(!log.contains("Error:"), "{log}");
  assert!(!log.contains("Until:\\@checkend"), "{log}");
  let cells: Vec<&str> = xml
    .split("<td ")
    .skip(1)
    .map(|cell| {
      cell
        .split_once('>')
        .map_or("", |(_, rest)| rest.split("</td>").next().unwrap_or(""))
    })
    .collect();
  assert_eq!(cells, ["a", "b", "c", "d"], "{xml}");
  assert!(xml.contains("<p xml:id=\"p1.1\">Before.\n"), "{xml}");
  // the re-emitted `\clear@document`'s `\do@output@cclv` (an output-routine message) leaves two empty paragraphs
  assert!(
    xml.trim_end().ends_with(
      "After.</p>\n    <p vattach=\"bottom\" xml:id=\"p1.2\"/>\n    <p vattach=\"bottom\" xml:id=\"p1.3\"/>\n  \
       </para>\n</document>"
    ),
    "{xml}"
  );
}

/// The three late slots share one dropped box with the tail's `\begingroup…\endgroup` inside it, so their local
/// definitions scope as in latex.ltx:15262-15277 (pdflatex): `/afterlastpage`'s `\X` reaches `/afteraux` and `/end`,
/// `/afteraux`'s `\Y` does not reach `/end`, and they see the kernel's `\clearpage` again. Their output is dropped, so the
/// probe is the log's.
#[test]
fn enddocument_late_hooks_scope_as_in_tex() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/enddocument_late_hooks_scope.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let probes: Vec<&str> = log
    .lines()
    .filter(|line| line.starts_with("LATE["))
    .collect();
  assert_eq!(
    probes,
    [
      "LATE[afteraux|X=alp|Y=aux|CP=real]",
      "LATE[end|X=alp|Y=undef|CP=real]"
    ],
    "{log}"
  );
  assert!(xml.contains("<p xml:id=\"p1.1\">Body.</p>"), "{xml}");
  assert!(!xml.contains("<pagination"), "{xml}");
}

/// etoolbox's `\patchcmd{\enddocument}{\clearpage}{…}` finds latex.ltx's token and patches, as in pdflatex (it took the
/// failure branch while our expansion had no `\clearpage`); the patched macro, now a user `\enddocument`, still ends the
/// document with no page break of its own.
#[test]
fn patchcmd_finds_the_enddocument_tail() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/enddocument_patchcmd_finds_the_kernel_tail.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert!(log.contains("PATCHED") && !log.contains("NOPATCH"), "{log}");
  assert!(xml.contains("<p xml:id=\"p1.1\">Body.</p>"), "{xml}");
  assert!(!xml.contains("<pagination"), "{xml}");
}

/// `\end{document}` expanded in an `\edef` stays one token (it is `protected`), so the tail's `\@checkend` and
/// `\clearpage` do not run at edef time: an `\edef` that runs away to the end of the file (tex.web §395's "extra }",
/// which we read as the argument) meets `\end{document}` itself, or the `\let…\enddocument` of a `\bibitem`'s
/// recovery (1012.2852: `Fatal:Timeout:PushbackLimit`). The counts are INTREGFIX4's; the text the run-away edef drops
/// is the §395 residual, pinned, not intended behaviour (pdflatex keeps it).
#[test]
fn enddocument_expanded_in_an_edef_stays_whole() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/enddocument_expanded_in_a_runaway_edef.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (3, 0), "{log}");
  assert!(!log.contains("lx@enddocument"), "{log}");
  assert!(xml.contains("<p xml:id=\"p1.1\">Text </p>"), "{xml}");
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/enddocument_expanded_in_an_edef.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (1, 0), "{log}");
  assert!(
    log.contains("readBalanced ran out of input") && !log.contains("lx@enddocument"),
    "{log}"
  );
  assert!(
    !xml.contains("<ERROR") && !xml.contains("<pagination"),
    "{xml}"
  );
}
