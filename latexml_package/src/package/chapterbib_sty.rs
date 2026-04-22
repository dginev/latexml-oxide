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
});
