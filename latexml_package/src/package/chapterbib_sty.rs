use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DeclareOption!("rootbib", {
    state::assign_value("CITE_UNIT_GLOBAL", Stored::from(1), None);
  });
  // Perl L20-21: the `sectionbib` option maps back-matter bibliography
  // to a section-level container so chapterbib-generated bibs render as
  // sections rather than top-level ltx:bibliography.
  DeclareOption!("sectionbib", {
    AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");
  });
  DeclareOption!("gather",    {});
  DeclareOption!("duplicate", {});
  ProcessOptions!();
  // Perl L28 comment: "SHOULD adjust BACKMATTER_ELEMENT!" — left as
  // no-op in Perl too.
  DefMacro!("\\sectionbib{}{}", "");

  // Perl L30-33: reset internal unit state between included chapters.
  DefPrimitive!("\\lx@cb@reset", {
    AssignValue!("CHAPTERBIB_UNIT" => Stored::None, Some(Scope::Global));
    AssignValue!("CITE_UNIT"       => Stored::None, Some(Scope::Global));
  });

  // Perl L47: expose the current chapterbib unit name as a token
  // stream. The Perl uses a zero-arg `DefMacro(.., sub { Explode(…) })`
  // closure; the Rust binding language doesn't yet have a typed
  // zero-arg Expandable sub form, so emit a primitive that reads the
  // value and unreads the tokens, wrapped by a regular DefMacro alias
  // so call sites still see it as an expandable CS.
  DefPrimitive!("\\lx@cb@do@unitname", {
    let unit = lookup_value("CHAPTERBIB_UNIT")
      .map(|s| s.to_string())
      .unwrap_or_default();
    if !unit.is_empty() {
      let tokens = Tokenize!(&unit);
      gullet::unread_vec(tokens.unlist().into_iter().collect());
    }
  });
  DefMacro!("\\lx@cb@unitname", "\\lx@cb@do@unitname");

  // Perl L59-60: chapterbib's override of \\bibliography. Branches on
  // \\lx@ifusebbl so that either the .bbl file gets \\input-ed, or
  // \\lx@bibliography receives the current chapter unit name as the
  // optional "per-unit" tag.
  DefMacro!(
    "\\bibliography Semiverbatim",
    "\\lx@ifusebbl{#1}{\\input{\\jobname.bbl}}\
     {\\lx@bibliography[\\lx@cb@unitname]{#1}}"
  );

  // Perl L49-57: {cbunit} environment auto-bumps a `chapbibN` unit
  // per occurrence — same effect as unitbib's bibunit. Using a
  // static atomic counter here matches `our $cbunits = 0` in Perl.
  use std::sync::atomic::{AtomicU64, Ordering};
  static CBUNITS: AtomicU64 = AtomicU64::new(0);
  DefEnvironment!("{cbunit}", "#body",
    after_digest_begin => {
      let n = CBUNITS.fetch_add(1, Ordering::SeqCst) + 1;
      let unit = format!("chapbib{}", n);
      let cite_unit = if lookup_value("CITE_UNIT_GLOBAL").is_some() {
        format!("bibliography {}", unit)
      } else {
        unit.clone()
      };
      AssignValue!("CHAPTERBIB_UNIT" => Stored::from(unit), Some(Scope::Global));
      AssignValue!("CITE_UNIT"       => Stored::from(cite_unit), Some(Scope::Global));
    });
});
