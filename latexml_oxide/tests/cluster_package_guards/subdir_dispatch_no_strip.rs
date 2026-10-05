//! Neither the package dispatcher (latexml_package::lib) nor `find_file_fallback`
//! (latexml_core::binding::content) strips a leading directory any more, so `subdir/<name>`
//! is a file PATH, not a binding name. Both tests drive the REAL fleet config via
//! `convert_to_xml_ar5iv` — `ar5iv.sty` raw-loads styles (`rawstyles`) and, since 62k, paper-local classes without a
//! binding (`localrawclasses`), the exact `cortex_worker --preload=ar5iv.sty` route.
use crate::cluster::convert_to_xml_ar5iv;

/// A paper-local `subdirdispatch/mathenv.sty`, whose basename collides with the CTAN `mathenv`
/// binding, must raw-load under `localrawstyles` (via SOURCEDIRECTORY), not be shadowed by a
/// directory-stripped binding match — the 2606.02073 cleveref/theorem bug. RED before the drop
/// (strip -> `mathenv` binding no-op -> the local `\subdirstymarker` never defined), GREEN
/// after. Guards both the dispatch strip and the `find_file_fallback` BasenameOnly strip stay
/// gone.
#[test]
fn subdir_sty_raw_loads_not_shadowed() {
  let xml = convert_to_xml_ar5iv("tests/cluster_regressions/subdir_sty_not_shadowed.tex");
  assert!(
    xml.contains("SUBDIRSTYLOADED"),
    "subdirdispatch/mathenv.sty should raw-load its local def (not be shadowed by the CTAN \
       mathenv binding):\n{xml}",
  );
}

/// A paper-local subdir `.cls` with no binding raw-loads under the arXiv profile's `localrawclasses` (62k,
/// OXIDIZED_DESIGN_DIVERGENCES #444): its `\subdirclsmarker` is defined and SUBDIRCLSLOADED reaches the output. Before
/// 62k the profile kept classes off and the class fell to OmniBus. The path is still a path: no `localjournal` binding
/// is name-matched from the stripped basename.
#[test]
fn subdir_cls_raw_loads_with_local_raw_classes() {
  let xml = convert_to_xml_ar5iv("tests/cluster_regressions/subdir_cls_rawloaded.tex");
  assert!(
    xml.contains("<document"),
    "the conversion completes:\n{xml}"
  );
  latexml::util::test::assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">SUBDIRCLSLOADED</p>"#);
}
