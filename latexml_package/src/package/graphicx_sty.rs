use crate::prelude::*;

/// Perl: image_candidates($path) from LaTeXML::Util::Image
/// Searches for files matching `path` (possibly with extensions) in graphics/search paths.
/// Returns comma-separated list of candidate paths, relative to source directory.
pub fn image_candidates(path: &str) -> String {
  use std::path::{Path, PathBuf};
  let path = path.trim().trim_matches('"');
  if path.is_empty() {
    return String::new();
  }
  let mut search_dirs: Vec<String> = state::get_graphics_paths();
  search_dirs.extend(state::get_search_paths());
  let source_dir = state::lookup_string("SOURCEDIRECTORY");
  if !source_dir.is_empty() {
    search_dirs.push(source_dir.clone());
  }
  if search_dirs.is_empty() {
    search_dirs.push(".".to_string());
  }

  let mut candidates: Vec<String> = Vec::new();
  let path_obj = Path::new(path);
  let has_extension = path_obj.extension().is_some();
  let source_path = if source_dir.is_empty() { None } else { Some(PathBuf::from(&source_dir)) };

  for dir in &search_dirs {
    let base = PathBuf::from(dir).join(path);
    if has_extension {
      if base.exists() {
        let rel = match &source_path {
          Some(sp) => base.strip_prefix(sp).unwrap_or(&base).to_string_lossy().to_string(),
          None => base.to_string_lossy().to_string(),
        };
        candidates.push(rel);
      }
    } else {
      // Search for path with any extension
      let parent = base.parent().unwrap_or(Path::new("."));
      let stem = base.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
      if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
          let fname = entry.file_name().to_string_lossy().to_string();
          if let Some(dot_pos) = fname.find('.') {
            if fname[..dot_pos] == stem {
              let full = entry.path();
              let rel = match &source_path {
                Some(sp) => full.strip_prefix(sp).unwrap_or(&full).to_string_lossy().to_string(),
                None => full.to_string_lossy().to_string(),
              };
              candidates.push(rel);
            }
          }
        }
      }
    }
  }

  // If no candidates found and path has no extension, try common image extensions
  // (matching Perl's pathname_findall with types => ['*'] which tries all known types)
  if candidates.is_empty() && !has_extension {
    // Perl typically returns just the first match (png)
    if let Some(ext) = ["png", "jpg", "jpeg", "gif", "pdf", "eps", "svg"].first() {
      let with_ext = format!("{path}.{ext}");
      candidates.push(with_ext);
    }
  }

  // Deduplicate while preserving order
  let mut seen = std::collections::HashSet::new();
  candidates.retain(|c| seen.insert(c.clone()));

  if candidates.is_empty() {
    path.to_string()
  } else {
    candidates.join(",")
  }
}

LoadDefinitions!({
  // graphicx.sty provides alternative argument syntax for graphics inclusion.
  // (See LaTeXML::Post::Graphics for suggested postprocessing)

  // Load the base graphics package
  RequirePackage!("graphics");

  // Raw TeX graphicx.sty redefines \rotatebox with \protected\def which overrides
  // our DefConstructor from graphics_sty.rs. Re-register using Let! to restore
  // the Rust definition. The Rust definition (from graphics_sty.rs) is stored
  // as \rotatebox's meaning before the raw TeX override happens, so we can
  // restore it by re-defining here after graphics is loaded.
  // Perl: graphicx.sty.ltxml doesn't need this because .ltxml bindings prevent
  // raw TeX loading entirely.
  DefConstructor!("\\rotatebox OptionalKeyVals:Grot {Float} {}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#3</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true,
    after_digest => sub[whatsit] {
      let angle = whatsit.get_arg(2).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      if let Some(body) = whatsit.get_arg(3) {
        if let Ok(props) = crate::package::graphics_sty::rotated_properties(body.clone(), angle, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
      }
    });

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
      // Perl: image_candidates searches filesystem for files matching path with any extension
      let candidates = image_candidates(&path);
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
