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

/// Perl: image_graphicx_sizer from LaTeXML::Util::Image
/// Reads image dimensions from candidates, applies graphicx options (height/width/scale),
/// and sets the whatsit's cached size properties so PGF/tikz get correct box dimensions.
fn image_graphicx_sizer(whatsit: &mut Whatsit) {
  use std::path::{Path, PathBuf};

  let dpi = state::lookup_int("DPI").max(72) as f64;
  let candidates = whatsit.get_property("candidates")
    .map(|c| c.to_string()).unwrap_or_default();
  let options = whatsit.get_property("options")
    .map(|c| c.to_string()).unwrap_or_default();

  // Try to read actual image dimensions from file
  let mut img_w: f64 = 0.0;
  let mut img_h: f64 = 0.0;
  let source_dir = state::lookup_string("SOURCEDIRECTORY");
  for candidate in candidates.split(',') {
    let candidate = candidate.trim();
    if candidate.is_empty() { continue; }
    let full_path = if Path::new(candidate).is_absolute() {
      PathBuf::from(candidate)
    } else if !source_dir.is_empty() {
      PathBuf::from(&source_dir).join(candidate)
    } else {
      PathBuf::from(candidate)
    };
    if let Some((w, h)) = read_image_dimensions(&full_path) {
      img_w = w as f64;
      img_h = h as f64;
      break;
    }
  }

  if img_w <= 0.0 || img_h <= 0.0 { return; }

  // Apply graphicx options (height, width, scale, keepaspectratio)
  // Perl: image_graphicx_size applies parsed transformations
  let dppt = dpi / 72.27; // dots per point
  let mut w = img_w;
  let mut h = img_h;

  // Parse options string for simple cases
  // Perl: image_graphicx_parse uses to_bp() to convert dimensions to big points (1/72 inch)
  let mut req_w: Option<f64> = None; // in bp (big points)
  let mut req_h: Option<f64> = None; // in bp
  let mut keep_ratio = false;
  let mut scale: Option<f64> = None;

  for opt in options.split(',') {
    let opt = opt.trim();
    if let Some(val) = opt.strip_prefix("width=") {
      if let Ok(dim) = Dimension::from_str(val.trim()) {
        // to_bp: convert pt to bp (1bp = 1/72 inch, 1pt = 1/72.27 inch)
        req_w = Some(dim.value_of() as f64 / 65536.0 * 72.0 / 72.27);
      }
    } else if let Some(val) = opt.strip_prefix("height=") {
      if let Ok(dim) = Dimension::from_str(val.trim()) {
        req_h = Some(dim.value_of() as f64 / 65536.0 * 72.0 / 72.27);
      }
    } else if let Some(val) = opt.strip_prefix("totalheight=") {
      if let Ok(dim) = Dimension::from_str(val.trim()) {
        req_h = Some(dim.value_of() as f64 / 65536.0 * 72.0 / 72.27);
      }
    } else if opt.starts_with("keepaspectratio") {
      keep_ratio = true;
    } else if let Some(val) = opt.strip_prefix("scale=") {
      scale = val.trim().parse::<f64>().ok();
    }
  }

  // Apply transformations (matching Perl image_graphicx_size logic)
  if let Some(s) = scale {
    w = (w * s).ceil();
    h = (h * s).ceil();
  }
  if req_w.is_some() || req_h.is_some() {
    let target_w = req_w.map(|rw| rw * dppt);
    let target_h = req_h.map(|rh| rh * dppt);
    if keep_ratio {
      match (target_w, target_h) {
        (Some(tw), Some(th)) => {
          // Both specified with keepaspectratio: use the more restrictive
          if w > 0.0 && h > 0.0 {
            if tw / w < th / h {
              let th2 = h * tw / w;
              w = tw;
              h = th2;
            } else {
              let tw2 = w * th / h;
              w = tw2;
              h = th;
            }
          }
        },
        (Some(tw), None) => {
          if w > 0.0 { h = h * tw / w; w = tw; }
        },
        (None, Some(th)) => {
          if h > 0.0 { w = w * th / h; h = th; }
        },
        (None, None) => {},
      }
    } else {
      if let Some(tw) = target_w { w = tw; }
      if let Some(th) = target_h { h = th; }
    }
  }
  // Perl: ceil pixel dimensions after applying transforms
  w = w.ceil();
  h = h.ceil();

  // Convert pixel dimensions back to points, then to scaled points (sp)
  let width_pt = w * 72.27 / dpi;
  let height_pt = h * 72.27 / dpi;

  // Set cached dimensions on the whatsit (Dimension::new takes scaled points = pt * 65536)
  let w_dim = Dimension::new((width_pt * 65536.0) as i64);
  let h_dim = Dimension::new((height_pt * 65536.0) as i64);
  whatsit.set_property("cached_width", Stored::Dimension(w_dim));
  whatsit.set_property("cached_height", Stored::Dimension(h_dim));
  whatsit.set_property("cached_depth", Stored::Dimension(Dimension::new(0)));
}

/// Read image dimensions (width, height) in pixels from a file.
/// Supports PNG and JPEG formats.
fn read_image_dimensions(path: &std::path::Path) -> Option<(u32, u32)> {
  use std::io::Read;
  let mut file = std::fs::File::open(path).ok()?;
  let mut header = [0u8; 32];
  file.read_exact(&mut header).ok()?;

  // PNG: signature + IHDR chunk
  if &header[0..8] == b"\x89PNG\r\n\x1a\n" {
    let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
    let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
    return Some((width, height));
  }

  // JPEG: look for SOF marker
  if header[0] == 0xFF && header[1] == 0xD8 {
    // Read the full file for JPEG parsing
    let mut data = header.to_vec();
    file.read_to_end(&mut data).ok()?;
    let mut i = 2;
    while i + 9 < data.len() {
      if data[i] != 0xFF { break; }
      let marker = data[i + 1];
      // SOF markers: 0xC0-0xCF (except 0xC4 DHT, 0xC8 JPG, 0xCC DAC)
      if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
        let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
        let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
        return Some((width, height));
      }
      let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
      i += 2 + len;
    }
  }

  None
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
  // Perl also has: sizer => \&image_graphicx_sizer — computes box dimensions from image file.
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
    },
    after_digest => sub[whatsit] {
      // Perl: sizer => \&image_graphicx_sizer
      // Compute box dimensions from image file + graphicx options
      image_graphicx_sizer(whatsit);
    }
  );
});
