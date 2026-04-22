use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pdfpages.sty.ltxml
  RequirePackage!("ifthen");
  RequirePackage!("calc");
  RequirePackage!("eso-pic");
  RequirePackage!("graphicx");

  // Perl pdfpages.sty.ltxml L30-36: `\includepdf OptionalKeyVals{}` —
  // optional keyvals with a `pages` key, then the file path. Constructor
  // emits a `<ltx:resource>` and follows with "See [pages X of ]<ref>".
  // Prior Rust stub used `[]` instead of `OptionalKeyVals` and dropped
  // the `pages` key entirely, so `\includepdf[pages=1-3]{foo}` never got
  // the "pages 1-3 of " prefix.
  DefConstructor!("\\includepdf OptionalKeyVals {}",
    "<ltx:resource src='#src' type='application/pdf'/>See #pages<ltx:ref href='#src'>#src</ltx:ref>",
    properties => sub[args] {
      let pages = args[0].as_ref().and_then(|d| {
        if let DigestedData::KeyVals(ref kvs) = d.data() {
          kvs.get_value("pages").map(|v| v.to_string())
        } else { None }
      });
      let src = args[1].as_ref().map(|d| d.to_string()).unwrap_or_default();
      Ok(stored_map!(
        "src"   => src,
        "pages" => pages.map(|p| format!("pages {} of ", p)).unwrap_or_default()
      ))
    });
});
