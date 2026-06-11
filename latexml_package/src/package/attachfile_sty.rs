use crate::prelude::*;

/// Perl attachfile.sty.ltxml L39-40: `%attachfileicon` name → codepoint
/// table. Keys are case-insensitive in Perl (`lc(ToString(...))`) so
/// this table is stored lowercase and callers should downcase their
/// lookup key.
fn attachfile_icon(name: &str) -> &'static str {
  match name {
    "pushpin" => "\u{1F4CC}",
    "paperclip" => "\u{1F4CE}",
    "tag" => "\u{1F3F7}",
    "graph" => "\u{1F4CA}",
    _ => "\u{1F4CC}", // Perl default: pushpin fallback
  }
}

/// Perl attachfile.sty.ltxml L42-49 `attachfileAttributes`. Extracts the
/// `icon`, `color`, and `file` values from the keyvals (+ optional file
/// arg) and packs them into a stored-map for the constructor XML to
/// reference. `file` is None for the noattach / notextattach variants.
fn attachfile_properties(
  kv: Option<&Digested>,
  file_arg: Option<&Digested>,
) -> latexml_core::common::arena::SymHashMap<Stored> {
  let icon_key = kv
    .and_then(|d| {
      if let DigestedData::KeyVals(kvs) = d.data() {
        kvs.get_value("icon")
      } else {
        None
      }
    })
    .map(|v| v.to_string().to_ascii_lowercase())
    .unwrap_or_else(|| "pushpin".to_string());
  let color = kv
    .and_then(|d| {
      if let DigestedData::KeyVals(kvs) = d.data() {
        kvs.get_value("color")
      } else {
        None
      }
    })
    .map(|v| v.to_string())
    .unwrap_or_default();
  let file = file_arg.map(|f| f.to_string()).unwrap_or_default();
  stored_map!(
    "icon"  => attachfile_icon(&icon_key).to_string(),
    "color" => color,
    "file"  => file
  )
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("keyval");
  RequirePackage!("ifpdf");
  RequirePackage!("calc");
  RequirePackage!("color");
  // Perl L24-27: \attachfilesetup accumulates global keyval options
  DefMacro!("\\lx@attachfile@options", None);
  DefPrimitive!("\\attachfilesetup {}", sub[(opts)] {
    let cs = T_CS!("\\lx@attachfile@options");
    AddToMacro!(cs, opts);
  });
  // Perl attachfile.sty.ltxml L29-32: forward to internal \\lx@… CSes
  // with the accumulated global options list prepended to the per-call
  // keyvals. Previous Rust stubs dropped to raw "#2" / "#3", which
  // silently lost both the icon rendering and the file URL.
  DefMacro!("\\noattachfile []",
    "\\lx@noattachfile{\\lx@attachfile@options,#1}");
  DefMacro!("\\notextattachfile []{}",
    "\\lx@notextattachfile{\\lx@attachfile@options,#1}{#2}");
  DefMacro!("\\attachfile []{}",
    "\\lx@attachfile{\\lx@attachfile@options,#1}{#2}");
  DefMacro!("\\textattachfile []{}{}",
    "\\lx@textattachfile{\\lx@attachfile@options,#1}{#2}{#3}");

  // Perl L51-66 four constructors. Ports the `attachfileAttributes`
  // dispatch so users get their chosen icon (pushpin/paperclip/tag/graph)
  // and color keyval, instead of a hardcoded pushpin with no color.
  // `?#color(…)` emits the color attribute only when the keyval was set
  // (Perl's L52 `color='#color'` always writes it, but a missing value
  // produces an empty attribute that's noise in the XML).
  DefConstructor!(
    "\\lx@noattachfile RequiredKeyVals",
    "<ltx:text ?#color(color='#color')>#icon</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true,
    properties => sub[args] {
      Ok(attachfile_properties(args[0].as_ref(), None))
    });
  DefConstructor!(
    "\\lx@notextattachfile RequiredKeyVals {}",
    "<ltx:text ?#color(color='#color')>#2</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true,
    properties => sub[args] {
      Ok(attachfile_properties(args[0].as_ref(), None))
    });
  DefConstructor!(
    "\\lx@attachfile RequiredKeyVals {}",
    "<ltx:ref href='#file' ?#color(color='#color')>#icon</ltx:ref>",
    mode => "restricted_horizontal", enter_horizontal => true,
    properties => sub[args] {
      Ok(attachfile_properties(args[0].as_ref(), args[1].as_ref()))
    });
  DefConstructor!(
    "\\lx@textattachfile RequiredKeyVals {}{}",
    "<ltx:ref href='#file' ?#color(color='#color')>#3</ltx:ref>",
    mode => "restricted_horizontal", enter_horizontal => true,
    properties => sub[args] {
      Ok(attachfile_properties(args[0].as_ref(), args[1].as_ref()))
    });
});
