use crate::prelude::*;
// Perl: `use LaTeXML::Util::Image;` (graphicx.sty.ltxml L17).
// image_candidates / image_graphicx_sizer now live in latexml_core::util::image.
pub use latexml_core::util::image::{image_candidates, image_graphicx_sizer};

LoadDefinitions!({
  // graphicx.sty provides alternative argument syntax for graphics inclusion.
  // (See LaTeXML::Post::Graphics for suggested postprocessing)

  // Perl L22: RequirePackage('graphics');
  RequirePackage!("graphics");

  // Perl L24-27: internal length / dimension macros.
  DefMacro!("\\Gin@ewidth", "");
  DefMacro!("\\Gin@eheight", "");
  DefMacro!("\\Gin@eresize", "");
  DefMacro!("\\Gin@esetsize", "");

  // Perl L29-38 uses `GraphixDimension` / `GraphixDimensions` custom parameter
  // types (graphics.sty.ltxml L26-57, ported in graphics_sty.rs). The Rust
  // port of those types currently returns raw scaled-point integers via
  // `dim.value_of().to_string()`, which does NOT match the keyval→options
  // attribute serializer's expected format. Using GraphixDimension here
  // regressed 50_structure::figure_grids_test (emitted `width=11118493`
  // instead of `width=169.65474pt`).
  //
  // Revert to `"Dimension"` / `""` until GraphixDimension's output is made
  // byte-equivalent to the old Dimension path. The parameter types stay
  // registered for any caller that wants to opt in explicitly.
  DefKeyVal!("Gin", "width", "Dimension");
  DefKeyVal!("Gin", "height", "Dimension");
  DefKeyVal!("Gin", "totalheight", "Dimension");
  DefKeyVal!("Gin", "keepaspectratio", "", "true");
  DefKeyVal!("Gin", "clip", "", "true");
  DefKeyVal!("Gin", "scale", "");
  DefKeyVal!("Gin", "angle", "");
  DefKeyVal!("Gin", "alt", "");
  DefKeyVal!("Gin", "trim", "");
  DefKeyVal!("Gin", "viewport", "");
  // NOTE: graphicx defines @angle to actually carry out the rotation (on \box\z@) w/\Gin@erotate
  // rather than to simply record the angle for later use. (also origin redefines)
  // This is used by adjustbox.
  // See \Gin@erotate, \Grot@box

  // Perl L45-46: LaTeXML extensions.
  DefKeyVal!("Gin", "vrml", "Semiverbatim");
  DefKeyVal!("Gin", "magnifiable", "", "true");

  // Perl L49-50: Redefine \includegraphics to dispatch based on bracket
  // syntax: if a second [] follows, fall back to the old graphics.sty
  // `\@includegraphics`, otherwise use the graphicx keyval-style
  // `\@includegraphicx`. `scope => 'global'` ensures the override
  // survives when `\usepackage{graphicx}` runs inside a group.
  DefMacro!(
    "\\includegraphics OptionalMatch:* []",
    "\\@ifnextchar[{\\@includegraphics#1[#2]}{\\@includegraphicx#1[#2]}",
    scope => Some(Scope::Global)
  );

  // Perl L52-72: graphicx-style \includegraphics with keyval options.
  DefConstructor!(
    "\\@includegraphicx OptionalMatch:* OptionalKeyVals:Gin Semiverbatim",
    "<ltx:graphics graphic='#path' candidates='#candidates' options='#options'/>",
    // Perl L72: mode => 'restricted_horizontal', enterHorizontal => 1.
    mode => "restricted_horizontal",
    enter_horizontal => true,
    // Perl L54: alias => '\includegraphics' so the reversion `tex=`
    // attribute serializes back to the author-facing name rather than
    // the internal `\@includegraphicx`.
    alias => "\\includegraphics",
    // Perl L56: scope => 'global'.
    scope => Some(Scope::Global),
    // Perl L63-71: properties callback.
    properties => sub[args] {
      // arg 0: starred, arg 1: keyvals, arg 2: graphic path
      let path = args[2].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      let path = path.trim().to_string();
      // Perl: ($path, @candidates) = image_candidates(ToString($graphic))
      let candidates = image_candidates(&path);
      // Perl: $options = graphicX_options($starred, $kv)
      let starred = args[0].is_some();
      let mut options_vec: Vec<String> = Vec::new();
      if starred {
        options_vec.push(s!("clip=true"));
      }
      let mut saw_w = false;
      let mut saw_h = false;
      let mut has_keepaspectratio = false;
      // Perl extracts `alt` separately as a semantic property — it
      // becomes the `description` attribute via afterConstruct, not
      // a graphics option.
      let mut alt_value: Option<String> = None;
      if let Some(ref kv_digested) = args[1] {
        if let DigestedData::KeyVals(ref kv) = kv_digested.data() {
          for (key, value) in kv.get_pairs() {
            if key == "alt" {
              alt_value = Some(value.to_string());
              continue;
            }
            if key.ends_with("width") { saw_w = true; }
            if key.ends_with("height") { saw_h = true; }
            if key == "keepaspectratio" { has_keepaspectratio = true; }
            let val_str = value.to_string();
            let val_str = val_str.replace(',', "\\,");
            options_vec.push(format!("{key}={val_str}"));
          }
        }
      }
      // Auto-add keepaspectratio if only width or height (not both) specified
      if (saw_w ^ saw_h) && !has_keepaspectratio {
        options_vec.push(s!("keepaspectratio=true"));
      }
      let options = options_vec.join(",");
      let mut props = stored_map!("path" => path, "candidates" => candidates, "options" => options);
      if let Some(alt) = alt_value {
        props.insert("alt", Stored::from(alt));
      }
      Ok(props)
    },
    // Perl L55: sizer => \&image_graphicx_sizer. We port it via
    // after_digest since the sizer callback is consulted for cached
    // dimensions at the same point.
    after_digest => sub[whatsit] {
      image_graphicx_sizer(whatsit);
    },
    // Perl L57-62: afterConstruct — emit `description` attribute from
    // `alt` keyval EVEN IF the value is the empty string (the
    // constructor template's shorthand `?#alt(…)` would skip empty
    // strings, so do it explicitly here).
    after_construct => sub[document, whatsit] {
      if let Some(alt) = whatsit.get_property("alt") {
        let alt_str = alt.to_string();
        if let Some(mut last_child) = document.get_node().get_last_child() {
          document.set_attribute(&mut last_child, "description", &alt_str)?;
        }
      }
    }
  );
});
