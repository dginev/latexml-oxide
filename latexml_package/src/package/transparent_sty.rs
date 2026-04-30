use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl transparent.sty.ltxml:
  //   DefPrimitive('\transparent{}', sub { MergeFont(opacity => ToString($_[1])); });
  //   DefMacro('\texttransparent{}{}', '{\transparent{#1} #2}');
  // Rust Font has `opacity: Option<Cow<'static, str>>` (font.rs:408)
  // and emits an `opacity=` attribute via the font-diff pipeline
  // (font.rs:1210-1214). Faithful parity.
  DefPrimitive!("\\transparent{}", sub[(value)] {
    merge_font(fontmap!(opacity => value.to_string()));
    Ok(Vec::new())
  });
  DefMacro!("\\texttransparent{}{}", "{\\transparent{#1} #2}");
});
