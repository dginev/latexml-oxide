//! Neither the package dispatcher (latexml_package::lib) nor `find_file_fallback`
//! (latexml_core::binding::content) strips a leading directory any more, so `subdir/<name>`
//! is a file PATH, not a binding name. Both tests drive the REAL fleet config via
//! `convert_to_xml_ar5iv` — `ar5iv.sty` sets INCLUDE_STYLES="searchpaths" (`localrawstyles`:
//! raw-load local `.sty`, classes OFF), the exact `cortex_worker --preload=ar5iv.sty` route.
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

/// INCLUDE_CLASSES stays disabled under `localrawstyles` (styles-only): a paper-local subdir
/// `.cls` must NOT raw-load — it falls to OmniBus, so its `\subdirclsmarker` stays undefined
/// and SUBDIRCLSLOADED never reaches the output. (Enabling `rawclasses` would flip this.)
#[test]
fn subdir_cls_does_not_raw_load_include_classes_off() {
  let xml = convert_to_xml_ar5iv("tests/cluster_regressions/subdir_cls_not_rawloaded.tex");
  assert!(
    !xml.contains("SUBDIRCLSLOADED"),
    "subdir `.cls` raw-loaded despite INCLUDE_CLASSES off (localrawstyles is styles-only):\n{xml}",
  );
}
