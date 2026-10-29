//! The bindings' frontmatter conventions for `--re-resolve` (OXIDIZED_DESIGN_DIVERGENCES #837): each binding declares
//! its own at load (`declare_frontmatter_conventions`), and offline no binding loads, so the re-resolver holds the
//! declarations itself — each the binding's own `FRONTMATTER_CONVENTIONS`, listed here by reference only. A dump
//! records the conventions each block ran under by name; a block whose name this list does not give back (a binding
//! that declares conventions and is missing here, or a dump of another binary), or that records none, fails its paper
//! loudly ([`re_resolve_dump`]: `stale-conventions`, a summary column of its own that the gate counts, and a non-zero
//! exit), never re-resolved under the kernel's in silence.

use latexml_contrib::{jcappub_sty, jinstpub_sty, jpconf_cls};
use latexml_engine::base_utilities::{FmConventions, fm_conventions_among, fm_resolve_with};
use latexml_package::package::{
  aa_cls, ams_support_sty, authblk_sty, book_cls, ieeetran_cls, inst_support_sty,
  iopart_support_sty, jheppub_sty, llncs_cls, report_cls, revtex4_support_sty, sv_support_sty,
};

use super::*;

/// Every binding's declared conventions.
pub const DECLARED: &[&FmConventions] = &[
  &revtex4_support_sty::FRONTMATTER_CONVENTIONS,
  &authblk_sty::FRONTMATTER_CONVENTIONS,
  &llncs_cls::FRONTMATTER_CONVENTIONS,
  &aa_cls::FRONTMATTER_CONVENTIONS,
  &inst_support_sty::FRONTMATTER_CONVENTIONS,
  &sv_support_sty::FRONTMATTER_CONVENTIONS,
  &ams_support_sty::FRONTMATTER_CONVENTIONS,
  &ieeetran_cls::FRONTMATTER_CONVENTIONS,
  &jheppub_sty::FRONTMATTER_CONVENTIONS,
  &jcappub_sty::FRONTMATTER_CONVENTIONS,
  &jinstpub_sty::FRONTMATTER_CONVENTIONS,
  &iopart_support_sty::FRONTMATTER_CONVENTIONS,
  &report_cls::FRONTMATTER_CONVENTIONS,
  &book_cls::FRONTMATTER_CONVENTIONS,
  &jpconf_cls::FRONTMATTER_CONVENTIONS,
];

/// The conventions for a block of `binding` and `hook`, as the conversion's `policy_for` read the declarations.
pub fn declared_conventions(binding: &str, hook: &str) -> &'static FmConventions {
  fm_conventions_among(DECLARED, binding, hook)
}

/// Resolve a dump's events again, each block under its binding's declared conventions — or the first block whose
/// recorded conventions (`policy`, by name) are not the ones [`DECLARED`] gives its binding and hook, or that records
/// none (a dump of a binary before the record: what it ran under is unknown, so it is not re-resolved either).
pub fn re_resolve_dump(dump: &Value) -> Result<FmResolved, String> {
  for e in dump["entries"].as_array().into_iter().flatten() {
    if e["kind"] != "author" || e["status"] != "events" {
      continue;
    }
    let (binding, hook) = (s(&e["binding"]), s(&e["hook"]));
    let declared = declared_conventions(&binding, &hook).name;
    match e["policy"].as_str() {
      Some(recorded) if recorded == declared => {},
      Some(recorded) => {
        return Err(format!(
          "block {} of {binding} ran under the conventions `{recorded}`, the declared are `{declared}`",
          e["index"]
        ));
      },
      None => {
        return Err(format!(
          "block {} of {binding} records no conventions (no `policy`), the declared are `{declared}`",
          e["index"]
        ));
      },
    }
  }
  Ok(fm_resolve_with(&json_inputs(dump), &declared_conventions))
}
