//! lipics-v2021 and IEEEtaes are OmniBus/IEEEtran stub bindings that replace the
//! paper's real `.cls` and omitted its `\RequirePackage{cleveref}`, so `\cref`/`\Cref`
//! came out `Error:undefined`. Perl (no lipics/IEEEtaes binding) raw-loads the `.cls`
//! and gets cleveref, so this was a Rust-only parity gap. The stubs now require
//! cleveref after hyperref. Witnesses 2606.01187 (lipics), 2606.01169 (IEEEtaes).
use crate::cluster::convert_to_xml_contrib_clean;

#[test]
fn lipics_stub_requires_cleveref() {
  // Red before the fix: `\cref`/`\Cref` come out Error:undefined; green: 0 errors.
  let _ = convert_to_xml_contrib_clean("tests/cluster_regressions/cleveref_lipics_stub.tex");
}
