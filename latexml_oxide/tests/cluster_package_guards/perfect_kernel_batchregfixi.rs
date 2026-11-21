//! Red/green guards for batch REGFIXI, four regressions of the v0.8.0 per-paper check (0 errors on 2026-10-08) in thin
//! bindings: amsppt's roster items under plain AmS-TeX, a prefix-matched shipped IUCr class, backref's `\BR@backref`,
//! array's `\@arrayleft`. Each guard converts its repro under `tools/perfect_kernel/repros/` with the ar5iv preload
//! and asserts the diagnostics by count and the whole element.
use latexml::util::test::{
  assert_element, convert_files_with, convert_with, normalize_markup, xml_element,
};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// amsppt's roster items close each other where latex_constructs is not loaded (`\input amsppt.sty`): the binding
/// declares latex_constructs' `ltx:item` Tag (latex_constructs.pool.ltxml:1284). Red (INTREGFIX4): each item opened
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

/// An author note written after an e-mail link (`\IUCrAufn`, iucrjournals.cls:212-232) is printed after the title
/// (:293-296): the e-mail a contact, the block is written as items, and the call printing Ann's mark apart runs for its
/// global definition of `\aufnX1` (`emit::printing_kept`, in a box no one uses) — REGFIXI dropped it, a bare "†" after
/// the title; Bob's own note, the second, prints after Ann's, each kept call run at its place. The ORCID link's iD is
/// read without the line end after it (2507.03170). Witnesses 2609.21618,
/// 2511.12314, 2509.01370. Distinct from `iucrjournals_reads_its_own_class`: a call printing apart from the names,
/// read outside the block.
#[test]
fn iucr_author_note_after_email() {
  let (log, xml) = convert_files_with(
    repro!("class-bindings/iucr/iucrjournals_author_note_after_email.tex"),
    &[(
      "iucrjournals.cls",
      repro!("class-bindings/iucr/iucrjournals.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>Ann Able</personname><note mark=\"†\" role=\"footnotemark\" \
     xml:id=\"id1\"/><contact name=\"Email:\u{a0}\" role=\"email\">ann@x.org</contact><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ</contact></creator>",
  );
  // (Bob's whole, his ORCID logo's SVG cut: `\IUCrAufn[1]{}` points at Ann's note, his own the second, in order)
  let bob = xml_element(&xml, "creator", &["before=\"\u{2003}\u{2003}\""])
    .unwrap_or_else(|| panic!("{xml}"));
  let (logo, rest) = bob
    .split_once("<svg:svg")
    .unwrap_or_else(|| panic!("{bob}"));
  let bob = [
    logo,
    &rest[rest
      .find("</svg:svg>")
      .map_or(0, |at| at + "</svg:svg>".len())..],
  ]
  .concat();
  assert_eq!(
    normalize_markup(&bob),
    normalize_markup(
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><note mark=\"†,‡\" \
       role=\"footnotemark\" xml:id=\"id2\"/><contact name=\"Email:\u{a0}\" role=\"email\">bob@y.org</contact><contact \
       role=\"orcid\">\u{2009}\u{2009}<ref class=\"ltx_orcid\" href=\"https://orcid.org/0000-0002-1825-0097\" title=\"ORCID \
       0000-0002-1825-0097\"></ref></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ</contact></creator>"
    ),
    "{xml}"
  );
  for note in [
    "These authors contributed equally.",
    "Present address: CERN.",
  ] {
    assert_eq!(xml.matches(note).count(), 1, "{xml}");
  }
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p align=\"center\" xml:id=\"p1.1\"><sup xml:id=\"p1.1.1\"><text font=\"italic\" \
     xml:id=\"p1.1.1.1\">†</text></sup>These authors contributed equally.</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.2\""],
    "<p align=\"center\" xml:id=\"p1.2\"><sup xml:id=\"p1.2.1\"><text font=\"italic\" \
     xml:id=\"p1.2.1.1\">‡</text></sup>Present address: CERN.</p>",
  );
}

/// A call printing a mark and a contact apart from the names is run once: the contact item the emitter writes holds the
/// call, which is not kept for a reader past the block as well (`emit::printing_apart`'s `written`). Red (REGFIXI2):
/// marks 3 and 4, "Count 4". Distinct from `iucr_author_note_after_email`: the kept call's sibling the item writes
/// itself; the e-mail on its own line and inline.
#[test]
fn contact_printing_call_runs_once() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/author_contact_call_runs_once.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, n, address) in [("id1", 1, "ann@x.org"), ("id2", 2, "bob@y.org")] {
    let contact = format!(
      "<contact class=\"ltx_author_line\" name=\"Email:\u{a0}\" role=\"email\"><sup xml:id=\"{id}\">{n}</sup><ref \
       class=\"ltx_href\" href=\"mailto:{address}\">{address}</ref></contact>"
    );
    assert_eq!(xml.matches(&contact).count(), 1, "{contact} in {xml}");
  }
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Count 2.</p>",
  );
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/author_inline_contact_call_runs_once.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Email:\u{a0}\" role=\"email\"><sup \
     xml:id=\"id1\">1</sup><ref class=\"ltx_href\" href=\"mailto:ann@x.org\">ann@x.org</ref></contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Email:\u{a0}\" role=\"email\"><sup xml:id=\"id2\">2</sup><ref class=\"ltx_href\" \
     href=\"mailto:bob@y.org\">bob@y.org</ref></contact></creator>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Count 2.</p>",
  );
}

/// backref's stub defines `\BR@backref` (backref.sty:245, the back-reference line, not rendered: empty), which a
/// document's bibitem macro calls once `\ifbackrefparscan` is defined. Red (INTREGFIX4): `\BR@backref` undefined, an
/// ERROR element and the key in the bibblock. Witnesses 1211.3435, 1408.3859, 1609.07764.
#[test]
fn backref_document_bibitem_calls_br_backref() {
  let (log, xml) = convert_with(
    repro!("stub-dependencies/backref_document_bibitem_calls_br_backref.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // (the doubled `[[1]]` is the pre-existing `\item[label]` in `{thebibliography}`, BIBITEMLABEL2, pinned here as it
  // stands, not this guard's subject)
  assert_element(
    &xml,
    "bibitem",
    &[],
    "<bibitem xml:id=\"bib.bib1\"><tags><tag>[1]</tag><tag role=\"autoref\">1<text xml:id=\"bib.bib1.1\"/></tag><tag role=\"refnum\">1</tag></tags>\
     <bibblock>[[1]]A. Author, Title.</bibblock></bibitem>",
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

/// A counted affiliation macro whose item the emitter writes from its argument alone: the item does not hold the call,
/// so the call still runs once with its globals (`printing_apart` tests containment, as `unwritten` and
/// `mark_dropped` do). pdflatex "Count 2: Univ A, Univ B."; the overlap test gave "Count 1: Univ B, ." (int/regfix5
/// review).
#[test]
fn affiliation_call_globals_run_once() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/author_affiliation_call_globals_run.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Count 2: Univ A, Univ B.</p>",
  );
}
