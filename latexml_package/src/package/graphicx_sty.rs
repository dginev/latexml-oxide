use crate::prelude::*;

LoadDefinitions!({
  // graphicx.sty provides alternative argument syntax for graphics inclusion.
  // (See LaTeXML::Post::Graphics for suggested postprocessing)

  // Load the base graphics package
  RequirePackage!("graphics");

  // Internal macros for graphicx sizing
  DefMacro!("\\Gin@ewidth", "");
  DefMacro!("\\Gin@eheight", "");
  DefMacro!("\\Gin@eresize", "");
  DefMacro!("\\Gin@esetsize", "");

  // KeyVal options for the Gin family
  // NOTE: GraphixDimension and GraphixDimensions are custom parameter types
  // defined in graphics.sty.ltxml. We use "Dimension" as a closest approximation
  // for GraphixDimension, and "" (plain text) for GraphixDimensions (sequence of 4 dims).
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

  // LaTeXML extensions:
  DefKeyVal!("Gin", "vrml", "Semiverbatim");
  DefKeyVal!("Gin", "magnifiable", "", "true");

  // Redefine \includegraphics to dispatch based on bracket syntax:
  // If a second [] follows, use the old graphics.sty-style \@includegraphics,
  // otherwise use the graphicx keyval-style \@includegraphicx.
  DefMacro!(
    "\\includegraphics OptionalMatch:* []",
    "\\@ifnextchar[{\\@includegraphics#1[#2]}{\\@includegraphicx#1[#2]}"
  );

  // The graphicx-style \includegraphics with keyval options.
  // Perl: properties callback computes path, candidates, options from keyval args.
  DefConstructor!(
    "\\@includegraphicx OptionalMatch:* OptionalKeyVals:Gin Semiverbatim",
    "<ltx:graphics graphic='#path' candidates='#candidates' options='#options'/>",
    enter_horizontal => true,
    properties => sub[args] {
      // arg 0: starred, arg 1: keyvals, arg 2: graphic path
      let path = args[2].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      let path = path.trim().to_string();
      // Candidates: just the path itself (filesystem search deferred to post-processing)
      let candidates = path.clone();
      // Build options string from keyval pairs, matching Perl's graphicX_options
      let starred = args[0].is_some();
      let mut options_vec: Vec<String> = Vec::new();
      if starred {
        options_vec.push(s!("clip=true"));
      }
      let mut saw_w = false;
      let mut saw_h = false;
      let mut has_keepaspectratio = false;
      if let Some(ref kv_digested) = args[1] {
        if let DigestedData::KeyVals(ref kv) = kv_digested.data() {
          for (key, value) in kv.get_pairs() {
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
      Ok(stored_map!("path" => path, "candidates" => candidates, "options" => options))
    }
  );
});
