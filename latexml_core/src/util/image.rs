//! Image helpers — port of `LaTeXML::Util::Image`.
//!
//! Perl counterpart: `lib/LaTeXML/Util/Image.pm`.
//!
//! Provides filesystem search for image candidates, minimal header-based
//! image size detection (PNG / JPEG / EPS) and the graphicx `sizer` that
//! converts keyval option strings into box dimensions. The Rust port is
//! intentionally narrower than the Perl original — Image::Magick is not
//! used at all; LaTeXML::Post::Graphics carries out any heavy-duty image
//! operations in a post-processing pass.

use std::path::{Path, PathBuf};

use crate::{
  BoxOps,
  common::{dimension::Dimension, numeric_ops::NumericOps, store::Stored},
  state,
  whatsit::Whatsit,
};

/// Perl: `image_candidates($path)` (Util::Image L43-57).
///
/// Returns comma-separated list of candidate paths for `path`, searching
/// GRAPHICSPATHS + SEARCHPATHS + SOURCEDIRECTORY. Paths are returned
/// relative to SOURCEDIRECTORY when possible, matching the Perl
/// `pathname_relative($_, $base)` post-filter.
pub fn image_candidates(path: &str) -> String {
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
  let source_path = if source_dir.is_empty() {
    None
  } else {
    Some(PathBuf::from(&source_dir))
  };

  for dir in &search_dirs {
    let base = PathBuf::from(dir).join(path);
    if has_extension {
      if base.exists() {
        let rel = match &source_path {
          Some(sp) => base
            .strip_prefix(sp)
            .unwrap_or(&base)
            .to_string_lossy()
            .to_string(),
          None => base.to_string_lossy().to_string(),
        };
        candidates.push(rel);
      }
    } else {
      // Search for path with any extension
      let parent = base.parent().unwrap_or_else(|| Path::new("."));
      let stem = base
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
      if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
          let fname = entry.file_name().to_string_lossy().to_string();
          if let Some(dot_pos) = fname.find('.')
            && fname[..dot_pos] == stem
          {
            let full = entry.path();
            let rel = match &source_path {
              Some(sp) => full
                .strip_prefix(sp)
                .unwrap_or(&full)
                .to_string_lossy()
                .to_string(),
              None => full.to_string_lossy().to_string(),
            };
            candidates.push(rel);
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
  let mut seen = rustc_hash::FxHashSet::default();
  candidates.retain(|c| seen.insert(c.clone()));

  // Perl image_candidates (Util/Image.pm) returns ($path, @candidates) where
  // @candidates holds only files actually found (pathname_findall + kpsewhich);
  // graphicx.sty sets `candidates => join(',', @candidates)`, so a missing file
  // yields an EMPTY candidates string (the attribute is then omitted) while the
  // `graphic` attribute still carries the raw path. The earlier Rust port fell
  // back to the raw path here, emitting `candidates="missing.png"` where Perl
  // emits no candidates at all. Return empty to match.
  candidates.join(",")
}

/// Perl: `image_graphicx_sizer($whatsit)` (Util::Image L259-272).
///
/// Reads image dimensions from `candidates`, applies the `options` string
/// (graphicx keyvals: width/height/totalheight/scale/keepaspectratio) and
/// writes back `cached_width`, `cached_height`, `cached_depth` on the
/// whatsit so downstream getSize() consumers (pgf, tikz) see the correct
/// box dimensions.
pub fn image_graphicx_sizer(whatsit: &mut Whatsit) {
  let dpi_val = state::lookup_int("DPI");
  let dpi = if dpi_val > 0 { dpi_val as f64 } else { 100.0 }; // Perl: our $DPI = 100
  let candidates = whatsit
    .get_property("candidates")
    .map(|c| c.to_string())
    .unwrap_or_default();
  let options = whatsit
    .get_property("options")
    .map(|c| c.to_string())
    .unwrap_or_default();

  // Try to read actual image dimensions from file
  let mut img_w: f64 = 0.0;
  let mut img_h: f64 = 0.0;
  let source_dir = state::lookup_string("SOURCEDIRECTORY");
  for candidate in candidates.split(',') {
    let candidate = candidate.trim();
    if candidate.is_empty() {
      continue;
    }
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

  if img_w <= 0.0 || img_h <= 0.0 {
    return;
  }

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
      if let Ok(dim) = <Dimension as std::str::FromStr>::from_str(val.trim()) {
        // to_bp: convert pt to bp (1bp = 1/72 inch, 1pt = 1/72.27 inch)
        req_w = Some(dim.value_of() as f64 / 65536.0 * 72.0 / 72.27);
      }
    } else if let Some(val) = opt.strip_prefix("height=") {
      if let Ok(dim) = <Dimension as std::str::FromStr>::from_str(val.trim()) {
        req_h = Some(dim.value_of() as f64 / 65536.0 * 72.0 / 72.27);
      }
    } else if let Some(val) = opt.strip_prefix("totalheight=") {
      if let Ok(dim) = <Dimension as std::str::FromStr>::from_str(val.trim()) {
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
          if w > 0.0 {
            h = h * tw / w;
            w = tw;
          }
        },
        (None, Some(th)) => {
          if h > 0.0 {
            w = w * th / h;
            h = th;
          }
        },
        (None, None) => {},
      }
    } else {
      if let Some(tw) = target_w {
        w = tw;
      }
      if let Some(th) = target_h {
        h = th;
      }
    }
  }
  // Perl: ceil pixel dimensions after applying transforms
  w = w.ceil();
  h = h.ceil();

  // Convert pixel dimensions back to points, then to scaled points (sp)
  let width_pt = w * 72.27 / dpi;
  let height_pt = h * 72.27 / dpi;

  // Perl: Dimension($w * 72.27 / $dpi . 'pt') — parses via TeX fixed-point arithmetic
  let w_dim =
    <Dimension as std::str::FromStr>::from_str(&format!("{width_pt}pt")).unwrap_or_default();
  let h_dim =
    <Dimension as std::str::FromStr>::from_str(&format!("{height_pt}pt")).unwrap_or_default();
  whatsit.set_property("cached_width", Stored::Dimension(w_dim));
  whatsit.set_property("cached_height", Stored::Dimension(h_dim));
  whatsit.set_property("cached_depth", Stored::Dimension(Dimension::default()));
}

/// Read image dimensions (width, height) in pixels from a file.
/// Supports PNG, JPEG, and EPS (PostScript BoundingBox).
///
/// This is a narrow replacement for `Image::Size::imgsize` (Perl
/// `image_size` at Util::Image L86-97). Only a few formats are needed
/// for typical arXiv graphics inclusions — anything else returns `None`
/// so the caller skips sizing (mirroring Perl's `return unless $w`).
pub fn read_image_dimensions(path: &Path) -> Option<(u32, u32)> {
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
      if data[i] != 0xFF {
        break;
      }
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

  // EPS: PostScript BoundingBox comment. Perl: LaTeXML::Util::Image reads
  // the leading `%%BoundingBox: llx lly urx ury` (values in bp, 1bp=1/72").
  // `%%HiResBoundingBox:` is preferred when present (float precision). We
  // read the first ~8KB since BoundingBox can be deferred (`(atend)` form
  // is also valid but would require scanning the tail; skip that).
  if (header[0] == b'%' && (header[1] == b'!' || header[1] == b'%'))
    || (header.starts_with(b"\xc5\xd0\xd3\xc6"))
  // EPS with binary preview header
  {
    let mut data = header.to_vec();
    // Read up to 32KB — BoundingBox typically in first few hundred bytes
    let mut extra = [0u8; 32768];
    let n = file.read(&mut extra).ok().unwrap_or(0);
    data.extend_from_slice(&extra[..n]);
    // If DOS EPSI binary preview: first 4 bytes are C5 D0 D3 C6, next 4
    // little-endian is offset to the PostScript section. Skip to it.
    let text_start = if data.starts_with(b"\xc5\xd0\xd3\xc6") && data.len() >= 8 {
      u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize
    } else {
      0
    };
    let text = std::str::from_utf8(data.get(text_start..)?).ok()?;
    // Prefer HiResBoundingBox (float) over BoundingBox (int).
    let mut found: Option<(f64, f64, f64, f64)> = None;
    for line in text.lines() {
      let trimmed = line.trim_start();
      let rest = if let Some(r) = trimmed.strip_prefix("%%HiResBoundingBox:") {
        // HiRes wins — take and stop searching.
        parse_bbox(r).inspect(|&b| {
          found = Some(b);
        })
      } else if found.is_none() {
        trimmed
          .strip_prefix("%%BoundingBox:")
          .and_then(parse_bbox)
          .inspect(|&b| {
            found = Some(b);
          })
      } else {
        None
      };
      if rest.is_some() && trimmed.starts_with("%%HiResBoundingBox:") {
        break;
      }
    }
    if let Some((llx, lly, urx, ury)) = found {
      let w = (urx - llx).max(0.0);
      let h = (ury - lly).max(0.0);
      if w > 0.0 && h > 0.0 {
        // EPS BoundingBox is in bp (1bp = 1/72"). Return as pixels at the
        // same bp-per-pixel rate the caller expects (it divides by dppt =
        // dpi/72.27 downstream). Using 1:1 means callers get bp-sized
        // pixels, consistent with Perl's `image_size` returning bp for
        // EPS (LaTeXML::Util::Image::image_size L45-L60).
        return Some((w.round() as u32, h.round() as u32));
      }
    }
  }

  None
}

/// Parse `"llx lly urx ury"` from a BoundingBox comment body.
pub fn parse_bbox(rest: &str) -> Option<(f64, f64, f64, f64)> {
  let mut it = rest.split_whitespace();
  let llx = it.next()?.parse::<f64>().ok()?;
  let lly = it.next()?.parse::<f64>().ok()?;
  let urx = it.next()?.parse::<f64>().ok()?;
  let ury = it.next()?.parse::<f64>().ok()?;
  Some((llx, lly, urx, ury))
}
