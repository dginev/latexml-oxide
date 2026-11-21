//! Red/green guards for batch REGFIXI, four regressions of the v0.8.0 per-paper check (0 errors on 2026-10-08) in thin
//! bindings: amsppt's roster items under plain AmS-TeX, a prefix-matched shipped IUCr class, backref's `\BR@backref`,
//! array's `\@arrayleft`. Each guard converts its repro under `tools/perfect_kernel/repros/` with the ar5iv preload
//! and asserts the diagnostics by count and the whole element.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// amsppt's roster items close each other where latex_constructs is not loaded (`\input amsppt.sty`): the binding
/// declares latex_constructs' `ltx:item` Tag (latex_constructs.pool.ltxml:1277). Red (INTREGFIX4): each item opened
/// inside its predecessor's paragraph, "malformed:ltx:item" ×2, a malformed enumerate ×2. A nested roster's items close
/// in the inner list. Witnesses math0006111, gr-qc9610061, math9809193 (KNOWN_PERL_ERRORS #1301). Distinct from
/// `perfect_kernel_batchamsppt::amsppt_heads_and_theorems`: `\documentstyle{amsppt}` loads LaTeX.pool, whose Tag hid it.
#[test]
fn amsppt_input_roster_items_close() {
  let (log, xml) = convert_with(
    repro!("amstex/amsppt_input_roster_items_close.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "enumerate",
    &["xml:id=\"Ix1\""],
    r#"<enumerate xml:id="Ix1"><item xml:id="Ix1.ix1"><tags><tag>(1)</tag></tags><para xml:id="Ix1.ix1.1"><p xml:id="Ix1.ix1.1.1">first item</p></para></item><item xml:id="Ix1.ix2"><tags><tag>(2)</tag></tags><para xml:id="Ix1.ix2.1"><p xml:id="Ix1.ix2.1.1">second item:</p><enumerate xml:id="Ix2"><item xml:id="Ix2.ix1"><tags><tag>(a)</tag></tags><para xml:id="Ix2.ix1.1"><p xml:id="Ix2.ix1.1.1">inner one</p></para></item><item xml:id="Ix2.ix2"><tags><tag>(1)</tag></tags><para xml:id="Ix2.ix2.1"><p xml:id="Ix2.ix2.1.1">inner two</p></para></item></enumerate></para></item></enumerate>"#,
  );
}

/// A shipped class the prefix fallback answers with the iucr binding is the class read: `iucrjournals.cls`, not the
/// `iucr.cls` the paper lacks (shipped_class::requested_shipped_class); its e-mail links in `\author`, a logo linking to
/// the address (iucrjournals.cls:201-206), are the author's email. Red (INTREGFIX4): "Can't find binding or file for
/// 'iucr.cls'", `\IUCrCemaillink` and `\IUCrEmaillink` undefined, the addresses in the names. Witnesses 2608.11466,
/// 2609.21618, 2607.09916, 2503.09183. Distinct from `perfect_kernel_batch80a::iucr_hcaption_is_the_caption`: the
/// class is requested by another name.
#[test]
fn iucrjournals_reads_its_own_class() {
  let (log, xml) = convert_files_with(
    repro!("class-bindings/iucr/iucrjournals_reads_its_own_class.tex"),
    &[(
      "iucrjournals.cls",
      repro!("class-bindings/iucr/iucrjournals.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Can't find binding for class iucrjournals (using iucr)"),
    "{log}"
  );
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Email:\u{a0}\" \
     role=\"email\">ann@x.org</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Email:\u{a0}\" \
     role=\"email\">bob@y.org</contact></creator>",
  );
}

/// backref's stub defines `\BR@backref` (backref.sty:244, the back-reference line, not rendered: empty), which a
/// document's bibitem macro calls once `\ifbackrefparscan` is defined. Red (INTREGFIX4): `\BR@backref` undefined, an
/// ERROR element and the key in the bibblock. Witnesses 1211.3435, 1408.3859, 1609.07764.
#[test]
fn backref_document_bibitem_calls_br_backref() {
  let (log, xml) = convert_with(
    repro!("stub-dependencies/backref_document_bibitem_calls_br_backref.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // (the `[[1]]` is the pre-existing `\item[label]` in `{thebibliography}`, not this guard's subject)
  assert_element(
    &xml,
    "bibblock",
    &[],
    "<bibblock>[[1]]A. Author, Title.</bibblock>",
  );
}

/// array's `\@arrayleft`/`\@arrayright` (array.sty:428-429, empty) are defined, which ltxutil's `\@array@array@new`
/// (ltxutil.sty:1322-1343, switched in at `\begin{document}` when array is loaded) calls around a tabular. Red
/// (INTREGFIX4): `\@arrayleft` undefined. Witnesses 2004.09405, 2401.03827 (which also need `\end{document}`'s
/// `\@checkend` tail, a kernel item of its own: the repro gobbles `\rvtx@enddocument@patch`).
#[test]
fn ltxutil_array_tabular_arrayleft() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/ltxutil_array_tabular_arrayleft.tex"),
    Some("ar5iv.sty"),
  );
  // ltxutil's two class warnings ("You should define the class name", "Failed to recognize \@vspace…")
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_markedasmath" vattach="bottom" xml:id="p1.1.1"><tr xml:id="p1.1.1.1"><td align="right" border="r" xml:id="p1.1.1.1.1">a</td><td align="center" xml:id="p1.1.1.1.2">b</td></tr><tr xml:id="p1.1.1.2"><td align="right" border="r t" xml:id="p1.1.1.2.1">c</td><td align="center" border="t" xml:id="p1.1.1.2.2">d</td></tr></tabular>"#,
  );
}
