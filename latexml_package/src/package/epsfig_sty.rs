use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("graphicx");
  state::assign_value("epsfclip", Stored::from(0), None);
  DefKeyVal!("epsGin", "width",           "Dimension");
  DefKeyVal!("epsGin", "height",          "Dimension");
  DefKeyVal!("epsGin", "keepaspectratio", "", "true");
  DefKeyVal!("epsGin", "clip",            "", "true");
  DefKeyVal!("epsGin", "figure", "Semiverbatim");
  DefKeyVal!("epsGin", "file",   "Semiverbatim");
  DefKeyVal!("epsGin", "prolog", "Semiverbatim");
  DefKeyVal!("epsGin", "silent", "");
  // epsfig also accepts standard graphicx-like options. Perl
  // epsfig.sty.ltxml leaves them unregistered; Rust-only divergence
  // paired with `21e730e71e`. Driver: 2101.10980 uses
  // `\psfig{file=...,angle=180,...}`.
  for key in [
    "angle", "scale", "totalheight", "trim", "viewport",
    "bb", "bbllx", "bblly", "bburx", "bbury",
    "hiresbb", "natwidth", "natheight",
    "draft", "final", "type", "ext", "read",
  ] {
    DefKeyVal!("epsGin", key, "");
  }
  // Perl epsfig.sty.ltxml L35-52: \psfig RequiredKeyVals:epsGin emits
  // an ltx:graphics element. properties closure extracts 'file' or
  // 'figure' key as the graphic path, removes those keys from the kv
  // set, injects clip=true if epsfclip was toggled on, and serializes
  // remaining key=value pairs as the options string. Prior Rust was a
  // \psfig{} primitive that discarded everything, so every \psfig call
  // produced no graphic node at all.
  DefConstructor!("\\psfig RequiredKeyVals:epsGin",
    "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>",
    enter_horizontal => true,
    properties => sub[args] {
      use DigestedData::*;
      let file = if let Some(kv_arg) = args[0].as_ref() {
        if let KeyVals(ref kv) = kv_arg.data() {
          let raw = kv.get_value("file")
            .or_else(|| kv.get_value("figure"))
            .map(|v| v.to_string())
            .unwrap_or_default();
          raw.trim().to_string()
        } else { String::new() }
      } else { String::new() };
      let candidates = latexml_core::util::image::image_candidates(&file);
      // Serialize remaining keyvals as "k=v,k=v" (skip file/figure,
      // inject clip=true if epsfclip was toggled on).
      let clip_on = state::lookup_value("epsfclip")
        .map(|v| v.to_string() != "0" && !v.to_string().is_empty())
        .unwrap_or(false);
      let mut opts = Vec::<String>::new();
      if let Some(kv_arg) = args[0].as_ref() {
        if let KeyVals(ref kv) = kv_arg.data() {
          let mut saw_clip = false;
          for (k, v) in kv.get_pairs() {
            if k == "file" || k == "figure" { continue; }
            if k == "clip" { saw_clip = true; }
            let v_str = v.to_string();
            if v_str.is_empty() { opts.push(k.to_string()); }
            else { opts.push(format!("{}={}", k, v_str)); }
          }
          if clip_on && !saw_clip { opts.push("clip=true".into()); }
        }
      }
      let options = opts.join(",");
      Ok(stored_map!("graphic" => file, "candidates" => candidates, "options" => options))
    });
  Let!("\\epsfig", "\\psfig");
  DefConstructor!("\\DeclareGraphicsExtensions{}", "");
  DefConstructor!("\\DeclareGraphicsRule{}{}{} Undigested", "");
  def_primitive_noop("\\psdraft")?;
  def_primitive_noop("\\psfull")?;
  def_primitive_noop("\\pssilent")?;
  def_primitive_noop("\\psnoisy")?;
  def_primitive_noop("\\psfigdriver{}")?;
  def_primitive_noop("\\epsfbox[]{}")?;
  Let!("\\epsffile", "\\epsfbox");
  DefPrimitive!("\\epsfclipon", {
    state::assign_value("epsfclip", Stored::from(1), None);
  });
  DefPrimitive!("\\epsfclipoff", {
    state::assign_value("epsfclip", Stored::from(0), None);
  });
  def_primitive_noop("\\epsfverbosetrue")?;
  def_primitive_noop("\\epsfverbosefalse")?;
  DefRegister!("\\epsfxsize" => Dimension::new(0));
  DefRegister!("\\epsfysize" => Dimension::new(0));
  def_primitive_noop("\\epsfsize{}{}")?;
});
