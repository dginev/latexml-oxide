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

/// Lexical relative path from `base` to `target`, with `..` for divergent base
/// components — matching Perl's `File::Spec->abs2rel` (used by
/// `pathname_relative`). Component-based, no symlink resolution. Falls back to
/// the target's string form if either side isn't absolute or has no common root.
fn abs2rel(target: &Path, base: &Path) -> String {
  use std::path::Component;
  if !target.is_absolute() || !base.is_absolute() {
    return target.to_string_lossy().to_string();
  }
  let t: Vec<Component> = target.components().collect();
  let b: Vec<Component> = base.components().collect();
  let common = t.iter().zip(b.iter()).take_while(|(a, c)| a == c).count();
  if common == 0 {
    return target.to_string_lossy().to_string();
  }
  let mut result = PathBuf::new();
  for _ in 0..(b.len() - common) {
    result.push("..");
  }
  for comp in &t[common..] {
    result.push(comp.as_os_str());
  }
  result.to_string_lossy().to_string()
}

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
    // Strip surrounding double-quotes from the search directory, symmetric to
    // the `path.trim_matches('"')` above. A quoted `\graphicspath{{"./dir"}}`
    // (or `\svgpath` / `--graphicspaths`) otherwise joins to a `"…"` path that
    // never resolves. See OXIDIZED_DESIGN #55.
    let dir = dir.trim().trim_matches('"');
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

  // Perl image_candidates (Util/Image.pm L49-53): when the search-dir lookup
  // finds nothing AND the name is extensionless, consult kpsewhich for
  // `<path>.png` / `<path>.pdf` — this resolves TeX Live system images such as
  // `example-image-a` (whose real file is a .pdf). Crucially, kpsewhich returns
  // ONLY files that actually exist, so a missing image yields no candidate. The
  // earlier Rust port instead SYNTHESIZED `<path>.png` unconditionally, so a
  // missing extensionless image got a bogus `candidates="missing.png"` (Perl
  // emits none) and `example-image-a` got the wrong `.png` instead of its `.pdf`.
  if candidates.is_empty() && !has_extension {
    let png = format!("{path}.png");
    let pdf = format!("{path}.pdf");
    if let Some(found) = crate::util::pathname::kpsewhich(&[&png, &pdf]) {
      // Perl relativizes every candidate to SOURCEDIRECTORY via pathname_relative,
      // which yields a `../…`-style path for a kpsewhich hit in the texmf tree
      // (e.g. `../usr/share/texlive/…/example-image-a.png`) — NOT an absolute
      // machine path. `pathname::relative`/`strip_prefix` only handle the
      // descendant case, so use a lexical abs2rel for the non-descendant tree.
      let rel = match &source_path {
        Some(sp) => abs2rel(Path::new(&found), sp),
        None => found,
      };
      candidates.push(rel);
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
    // The raster readers (PNG/JPEG/EPS, like Perl's `imgsize`) couldn't measure
    // the asset. Before giving up, emulate pdfTeX: read the natural size from the
    // file itself. pdfTeX's built-in reader takes a PDF's CropBox (its default)
    // or MediaBox, and an SVG's viewBox — with NO external tool. (Perl-LaTeXML
    // instead shells out to ImageMagick precisely because Image::Size can't read
    // PDF; even then it forces `pdf:use-cropbox` to match pdfTeX. So the faithful,
    // self-contained move is pdfTeX's, not Perl's.) `natural_size_pt` shares the
    // same CropBox→MediaBox reader as `LaTeXML::Post::Graphics`.
    //
    // Whatever we decide, we MUST set `cached_width`: without it, `compute_size`
    // falls through to summing the whatsit's ARGUMENT boxes — and one of them is
    // the Semiverbatim *filename* — so a bare `arrange_panels` would wrap figure
    // rows by path length (arXiv:2409.16471 fig 2: 12 uniform 0.245\textwidth
    // panels split 3/3/2/3/1 by filename, not 3 rows of 4).
    let source_dir = state::lookup_string("SOURCEDIRECTORY");
    let natural = candidates.split(',').find_map(|candidate| {
      let candidate = candidate.trim();
      if candidate.is_empty() {
        return None;
      }
      natural_size_pt(&resolve_candidate(candidate, &source_dir))
    });
    if let Some((nw_pt, nh_pt)) = natural {
      // pdfTeX/graphics.sty box sizing in pt (verified against `\the\wd` under
      // pdflatex): with an explicit `width=`, the box width IS the request and
      // the natural size only fills in the height via the aspect ratio.
      let (bw, bh) = graphicx_box_pt(nw_pt, nh_pt, &options);
      whatsit.set_property("cached_width", Stored::Dimension(bw));
      whatsit.set_property("cached_height", Stored::Dimension(bh));
      whatsit.set_property("cached_depth", Stored::Dimension(Dimension::default()));
      return;
    }
    // Last resort — a PDF whose page box is buried in a compressed object stream
    // (where pdfTeX's full parser would still succeed but our byte reader can't),
    // or an unreadable SVG. Honor an EXPLICIT `width=`/`height=` request (the
    // display size LaTeXML already emits), else 0 (Perl-without-ImageMagick
    // parity). Still set `cached_width` so the filename is never summed.
    let mut ew: Option<Dimension> = None;
    let mut eh: Option<Dimension> = None;
    for opt in options.split(',') {
      let opt = opt.trim();
      if let Some(val) = opt.strip_prefix("width=") {
        ew = <Dimension as std::str::FromStr>::from_str(val.trim()).ok();
      } else if let Some(val) = opt.strip_prefix("height=") {
        eh = <Dimension as std::str::FromStr>::from_str(val.trim()).ok();
      } else if let Some(val) = opt.strip_prefix("totalheight=") {
        eh = <Dimension as std::str::FromStr>::from_str(val.trim()).ok();
      }
    }
    whatsit.set_property("cached_width", Stored::Dimension(ew.unwrap_or_default()));
    whatsit.set_property("cached_height", Stored::Dimension(eh.unwrap_or_default()));
    whatsit.set_property("cached_depth", Stored::Dimension(Dimension::default()));
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

/// Resolve an `image_candidates` entry to a filesystem path, relative to the
/// document's `SOURCEDIRECTORY` when the candidate isn't already absolute.
fn resolve_candidate(candidate: &str, source_dir: &str) -> PathBuf {
  if Path::new(candidate).is_absolute() {
    PathBuf::from(candidate)
  } else if !source_dir.is_empty() {
    PathBuf::from(source_dir).join(candidate)
  } else {
    PathBuf::from(candidate)
  }
}

/// Natural (unscaled) size of a graphic in TeX points, read the way pdfTeX
/// reads it — with no external tool: a PDF's CropBox (default) / MediaBox, or an
/// SVG's width/height / viewBox. `None` for formats the raster readers already
/// handle, or when the geometry can't be recovered (e.g. a PDF whose page box is
/// hidden inside a compressed object stream).
fn natural_size_pt(path: &Path) -> Option<(f64, f64)> {
  if let Some((w_bp, h_bp)) = read_pdf_page_box(path) {
    return Some((bp_to_pt(w_bp), bp_to_pt(h_bp)));
  }
  read_svg_size_pt(path)
}

/// bp (PostScript big point, 1/72") → TeX pt (1/72.27").
fn bp_to_pt(bp: f64) -> f64 { bp * 72.27 / 72.0 }

/// pt (f64) → `Dimension` (scaled points).
fn pt_to_dim(pt: f64) -> Dimension { Dimension::new((pt * 65536.0).round() as i64) }

/// Apply graphicx `width`/`height`/`totalheight`/`scale`/`keepaspectratio` to a
/// natural (pt) size, matching pdfTeX/graphics.sty box sizing. Verified against
/// `\the\wd` under pdflatex: an explicit `width=` sets the box width outright,
/// the natural size only supplying the missing dimension via the aspect ratio.
fn graphicx_box_pt(nw: f64, nh: f64, options: &str) -> (Dimension, Dimension) {
  let dim_pt = |v: &str| {
    <Dimension as std::str::FromStr>::from_str(v.trim())
      .ok()
      .map(|d| d.value_of() as f64 / 65536.0)
  };
  let (mut w_opt, mut h_opt, mut scale, mut keep) = (None, None, None, false);
  for opt in options.split(',') {
    let opt = opt.trim();
    if let Some(v) = opt.strip_prefix("width=") {
      w_opt = dim_pt(v);
    } else if let Some(v) = opt.strip_prefix("height=") {
      h_opt = dim_pt(v);
    } else if let Some(v) = opt.strip_prefix("totalheight=") {
      h_opt = dim_pt(v);
    } else if let Some(v) = opt.strip_prefix("scale=") {
      scale = v.trim().parse::<f64>().ok();
    } else if opt.starts_with("keepaspectratio") {
      keep = true;
    }
  }
  let (bw, bh) = match (w_opt, h_opt) {
    (Some(w), Some(h)) => {
      // Both requested: `keepaspectratio` fits within the box, dropping the more
      // extreme request (Perl `image_graphicx_size` scale-to, a3 branch).
      if keep && nw > 0.0 && nh > 0.0 {
        if w / nw < h / nh {
          (w, nh * w / nw)
        } else {
          (nw * h / nh, h)
        }
      } else {
        (w, h)
      }
    },
    (Some(w), None) => (w, if nw > 0.0 { nh * w / nw } else { 0.0 }),
    (None, Some(h)) => (if nh > 0.0 { nw * h / nh } else { 0.0 }, h),
    (None, None) => match scale {
      Some(s) => (nw * s, nh * s),
      None => (nw, nh),
    },
  };
  (pt_to_dim(bw), pt_to_dim(bh))
}

/// Read a PDF's page box (width, height) in bp — CropBox (pdfTeX's default),
/// else MediaBox. Pure Rust, no external tool (this is what pdfTeX's built-in
/// reader does). Shared with `LaTeXML::Post::Graphics`. `None` when neither box
/// appears as raw bytes (e.g. compressed into an object stream).
pub fn read_pdf_page_box(path: &Path) -> Option<(f64, f64)> {
  let bytes = std::fs::read(path).ok()?;
  // Fast-fail before the (whole-file) UTF-8 conversion: modern PDFs often
  // compress the page dictionary into an object stream, so the box tokens never
  // appear as raw bytes.
  if byte_find(&bytes, b"/CropBox").is_none() && byte_find(&bytes, b"/MediaBox").is_none() {
    return None;
  }
  let content = String::from_utf8_lossy(&bytes);
  parse_pdf_box(&content, "/CropBox").or_else(|| parse_pdf_box(&content, "/MediaBox"))
}

/// Parse `TOKEN [ llx lly urx ury ]` from PDF content, returning `(w, h)`.
fn parse_pdf_box(content: &str, token: &str) -> Option<(f64, f64)> {
  let start = content.find(token)? + token.len();
  let rest = &content[start..];
  let lb = rest.find('[')?;
  let rb = rest[lb..].find(']')? + lb;
  let mut it = rest[lb + 1..rb]
    .split_whitespace()
    .filter_map(|s| s.parse::<f64>().ok());
  let (x0, y0, x1, y1) = (it.next()?, it.next()?, it.next()?, it.next()?);
  Some(((x1 - x0).abs(), (y1 - y0).abs()))
}

/// Byte-level substring search — avoids a UTF-8 conversion for the fast-fail.
fn byte_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  if needle.is_empty() || needle.len() > haystack.len() {
    return None;
  }
  haystack.windows(needle.len()).position(|w| w == needle)
}

/// Natural SVG size in pt, from the root `<svg>` element: prefer absolute
/// `width`/`height` lengths, else fall back to the `viewBox` (user units treated
/// as CSS px). Gives at least a correct aspect ratio, which is all a `width=`-ed
/// inclusion needs. `None` if the file isn't an SVG or has no usable geometry.
fn read_svg_size_pt(path: &Path) -> Option<(f64, f64)> {
  let head = read_head_lossy(path)?;
  let tag = svg_root_tag(&head)?;
  if let (Some(w), Some(h)) = (
    svg_attr_len_pt(tag, "width"),
    svg_attr_len_pt(tag, "height"),
  ) {
    return Some((w, h));
  }
  let (vw, vh) = svg_viewbox_extent(tag)?;
  // viewBox user units ≈ CSS px (1/96"); convert to pt for a plausible scale.
  Some((px_to_pt(vw), px_to_pt(vh)))
}

/// SVG **viewport** size in CSS px, as a browser would take it: the `viewBox`
/// extent when there is one, else the root `width`/`height` lengths.
///
/// This is the sizing basis for `imagewidth`/`imageheight` in
/// `LaTeXML::Post::Graphics` — where Perl asks Image::Magick, which renders the
/// SVG and reports the raster it produced.
///
/// **The viewBox comes first here, and that is deliberate** — the opposite of
/// `read_svg_size_pt`, which wants the natural *typeset* size. Both of our own
/// PDF→SVG converters emit a `viewBox` alongside pt-valued `width`/`height`
/// (`pdftocairo -svg`: `width="612pt" height="792pt" viewBox="0 0 612 792"`;
/// `mutool draw -F svg`: `width="612" height="792" viewBox="0 0 612 792"`), so
/// preferring the lengths would silently rescale every PDF-derived figure in the
/// corpus by 96/72 = 1.33×. The viewBox keeps them at their long-standing pixel
/// size, and for a `width=`-ed inclusion only the aspect ratio matters anyway.
pub fn read_svg_viewport_px(path: &Path) -> Option<(u32, u32)> {
  let head = read_head_lossy(path)?;
  let tag = svg_root_tag(&head)?;
  let (w, h) = svg_viewbox_extent(tag).or_else(|| {
    Some((
      svg_attr_len_px(tag, "width")?,
      svg_attr_len_px(tag, "height")?,
    ))
  })?;
  Some((w.round().max(1.0) as u32, h.round().max(1.0) as u32))
}

/// The leading bytes of a file, decoded lossily. Bounded: an SVG can be
/// hundreds of MB, and every geometry attribute we want lives in the root tag.
/// Lossy rather than strict UTF-8 so a latin-1 preamble still yields a
/// readable root tag (and so a multi-byte sequence split by the read boundary
/// degrades to U+FFFD instead of failing the whole read).
fn read_head_lossy(path: &Path) -> Option<String> {
  use std::io::Read;
  let mut file = std::fs::File::open(path).ok()?;
  let mut buf = [0u8; 8192];
  let n = file.read(&mut buf).ok()?;
  Some(String::from_utf8_lossy(&buf[..n]).into_owned())
}

/// The root `<svg …>` start tag within `head`, quote-aware so a `>` inside an
/// attribute value doesn't end the tag early. Skipping to `<svg` also steps over
/// the `<?xml …?>` prolog, comments and any DOCTYPE — otherwise the prolog's
/// `?>` would be mistaken for the end of the start tag.
pub fn svg_root_tag(head: &str) -> Option<&str> {
  let start = head.find("<svg")?;
  let rest = &head[start..];
  let mut quote: Option<char> = None;
  for (i, c) in rest.char_indices() {
    match quote {
      Some(q) if c == q => quote = None,
      Some(_) => {},
      None if c == '"' || c == '\'' => quote = Some(c),
      None if c == '>' => return Some(&rest[..i]),
      None => {},
    }
  }
  None
}

/// Value of the `name="…"` / `name='…'` attribute in an XML start tag.
///
/// The attribute **name is matched whole**: a bare substring search reads
/// `stroke-width="2"` — legal on a root `<svg>` — as `width`, which is how a
/// 634×805 drawing once measured 2×805.
pub fn svg_attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
  let mut from = 0;
  while let Some(hit) = tag[from..].find(name) {
    let at = from + hit;
    from = at + name.len();
    // Left boundary: the name must start an attribute, not end another one
    // (`stroke-width`) — so what precedes it is whitespace, or the `<svg`.
    let preceded_ok = tag[..at]
      .chars()
      .next_back()
      .is_some_and(|c| c.is_whitespace());
    if !preceded_ok {
      continue;
    }
    // Right boundary: `=` (optionally spaced) then a quoted value.
    let after = tag[from..].trim_start();
    let Some(after) = after.strip_prefix('=') else {
      continue;
    };
    let after = after.trim_start();
    let Some(q) = after.chars().next() else {
      continue;
    };
    if q != '"' && q != '\'' {
      continue;
    }
    let body = &after[q.len_utf8()..];
    let end = body.find(q)?;
    return Some(&body[..end]);
  }
  None
}

/// `(width, height)` extent of the root `viewBox`, in user units (≈ CSS px).
/// Per the SVG grammar the four numbers are comma-**and/or**-whitespace
/// separated, so `viewBox="0,0,634,805"` must parse like `"0 0 634 805"`.
fn svg_viewbox_extent(tag: &str) -> Option<(f64, f64)> {
  let vb = svg_attr_value(tag, "viewBox")?;
  let mut it = vb
    .split(|c: char| c.is_whitespace() || c == ',')
    .filter(|s| !s.is_empty());
  let (_x, _y) = (it.next()?, it.next()?);
  let vw = it.next()?.parse::<f64>().ok()?;
  let vh = it.next()?.parse::<f64>().ok()?;
  Some((vw, vh))
}

/// An SVG length attribute in CSS px. A unitless value is user units, i.e. px.
/// `None` for anything that isn't an absolute length (`%`, `em`, `ex`, …) —
/// those are resolved against a viewport we don't have, so the caller must fall
/// back to the viewBox rather than treat the bare number as pixels.
pub fn svg_attr_len_px(tag: &str, name: &str) -> Option<f64> {
  svg_len_px(svg_attr_value(tag, name)?)
}

/// An SVG length attribute converted to pt, iff it carries an absolute unit.
/// Unitless/`%` values return `None` (the caller falls back to the viewBox) —
/// unlike [`svg_attr_len_px`], a unitless value is *not* accepted here: without
/// a unit there is no natural typeset size to report, only a scale.
fn svg_attr_len_pt(tag: &str, name: &str) -> Option<f64> {
  let raw = svg_attr_value(tag, name)?.trim();
  if !raw.ends_with(|c: char| c.is_alphabetic()) {
    return None;
  }
  Some(px_to_pt(svg_len_px(raw)?))
}

/// Parse an SVG/CSS length into CSS px (1/96"), or `None` if it carries no
/// absolute unit. Unitless = user units = px.
fn svg_len_px(raw: &str) -> Option<f64> {
  let raw = raw.trim();
  // Split the number from its unit — but `6.34e2` must not split at the
  // exponent's `e`, which would silently read 634 as 6.
  let mut split = raw.len();
  for (i, c) in raw.char_indices() {
    if (c.is_alphabetic() || c == '%') && !is_exponent(&raw[i..]) {
      split = i;
      break;
    }
  }
  let (num, unit) = raw.split_at(split);
  let v = num.trim().parse::<f64>().ok()?;
  match unit.trim() {
    "" | "px" => Some(v),
    "pt" => Some(v * 96.0 / 72.0),
    "in" => Some(v * 96.0),
    "cm" => Some(v * 96.0 / 2.54),
    "mm" => Some(v * 96.0 / 25.4),
    "pc" => Some(v * 16.0),
    "Q" => Some(v * 96.0 / 101.6),
    _ => None, // %, em, ex, rem, vw, … → no absolute length
  }
}

/// Does this trailing fragment start an exponent (`e-3`, `E+10`) rather than a
/// unit?
fn is_exponent(tail: &str) -> bool {
  let mut cs = tail.chars();
  matches!(cs.next(), Some('e') | Some('E'))
    && cs
      .next()
      .is_some_and(|c| c.is_ascii_digit() || c == '+' || c == '-')
}

/// CSS px (1/96") → TeX pt (1/72.27").
fn px_to_pt(px: f64) -> f64 { px * 72.27 / 96.0 }

#[cfg(test)]
mod svg_geometry_tests {
  use super::*;

  /// Write `content` to a uniquely-named temp `.svg` and hand back the path.
  fn svg_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("lximg-{}-{name}.svg", std::process::id()));
    std::fs::write(&path, content).expect("write svg fixture");
    path
  }

  #[test]
  fn root_tag_skips_the_prolog_and_stops_at_the_real_tag_end() {
    let head = "<?xml version=\"1.0\"?>\n<!-- a > in a comment -->\n<svg width=\"3\">\n<rect/>";
    assert_eq!(svg_root_tag(head), Some("<svg width=\"3\""));
    // A `>` inside an attribute value must not end the start tag.
    let quoted = r#"<svg desc="a > b" width="3"><rect/>"#;
    assert_eq!(svg_root_tag(quoted), Some(r#"<svg desc="a > b" width="3""#));
    assert_eq!(svg_root_tag("no svg here"), None);
  }

  /// A bare substring search reads `stroke-width` as `width`. Both attribute
  /// orders, since the bug only bites when the decoy comes first.
  #[test]
  fn attr_value_matches_whole_names_not_substrings() {
    let decoy_first = r#"<svg stroke-width="2" width="634" height="805""#;
    assert_eq!(svg_attr_value(decoy_first, "width"), Some("634"));
    assert_eq!(svg_attr_value(decoy_first, "stroke-width"), Some("2"));
    let decoy_last = r#"<svg width="634" stroke-width="2""#;
    assert_eq!(svg_attr_value(decoy_last, "width"), Some("634"));
    // A name that appears only as a suffix of another attribute is absent.
    assert_eq!(svg_attr_value(r#"<svg stroke-width="2""#, "width"), None);
  }

  #[test]
  fn attr_value_reads_both_quote_styles() {
    let single = r#"<svg xmlns='http://www.w3.org/2000/svg' width='634' height='805'"#;
    assert_eq!(svg_attr_value(single, "width"), Some("634"));
    assert_eq!(svg_attr_value(single, "height"), Some("805"));
    // Spaces around `=` are legal XML.
    assert_eq!(
      svg_attr_value(r#"<svg width = "634""#, "width"),
      Some("634")
    );
  }

  /// The unit table, in CSS px (1in = 96px). Every absolute unit SVG allows,
  /// plus the three shapes that must NOT be read as a pixel count.
  #[test]
  fn len_px_converts_absolute_units_and_rejects_relative_ones() {
    let cases: &[(&str, Option<f64>)] = &[
      ("634", Some(634.0)), // unitless = user units = px
      ("634px", Some(634.0)),
      ("10cm", Some(377.952_755_905_511_8)),
      ("7.5cm", Some(283.464_566_929_133_84)),
      ("100mm", Some(377.952_755_905_511_8)),
      ("4in", Some(384.0)),
      ("72pt", Some(96.0)),
      ("6pc", Some(96.0)),
      ("6.34e2", Some(634.0)), // exponent, not a `e` unit
      ("-5", Some(-5.0)),
      ("100%", None), // resolved against a viewport we don't have
      ("2em", None),
      ("50vw", None),
      ("", None),
      ("wide", None),
    ];
    for (raw, want) in cases {
      match (svg_len_px(raw), want) {
        (Some(got), Some(w)) => assert!(
          (got - w).abs() < 1e-9,
          "svg_len_px({raw:?}) = {got}, want {w}"
        ),
        (got, want) => assert_eq!(
          got.is_none(),
          want.is_none(),
          "svg_len_px({raw:?}) = {got:?}"
        ),
      }
    }
  }

  /// Both of our PDF→SVG converters emit `viewBox` **and** root lengths. The
  /// viewport reader must keep reporting the viewBox extent for them — taking
  /// pdftocairo's `612pt` instead would rescale every PDF-derived figure in the
  /// corpus by 96/72. Root tags copied verbatim from the tools.
  #[test]
  fn viewport_px_is_stable_for_our_own_converter_output() {
    let pdftocairo = svg_file(
      "pdftocairo",
      r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="612pt" height="792pt" viewBox="0 0 612 792">
<defs/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&pdftocairo), Some((612, 792)));
    let mutool = svg_file(
      "mutool",
      r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" version="1.1" width="612" height="792" viewBox="0 0 612 792">
<defs/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&mutool), Some((612, 792)));
    let _ = std::fs::remove_file(pdftocairo);
    let _ = std::fs::remove_file(mutool);
  }

  /// Without a viewBox the root lengths are the viewport — and they must be
  /// *converted*, not truncated. Reading `10cm` as 10 px is how a poster-sized
  /// drawing became a 10-pixel thumbnail (issue 498 follow-up).
  #[test]
  fn viewport_px_converts_unit_bearing_lengths_when_there_is_no_viewbox() {
    let cm = svg_file(
      "cm",
      r#"<svg xmlns="http://www.w3.org/2000/svg" width="10cm" height="7.5cm"><rect/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&cm), Some((378, 283)));
    let inch = svg_file("in", r#"<svg width="4in" height="2in"><rect/></svg>"#);
    assert_eq!(read_svg_viewport_px(&inch), Some((384, 192)));
    let quoted = svg_file(
      "sq",
      r#"<svg xmlns='http://www.w3.org/2000/svg' width='634' height='805'><rect/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&quoted), Some((634, 805)));
    let decoy = svg_file(
      "decoy",
      r#"<svg xmlns="http://www.w3.org/2000/svg" stroke-width="2" width="634" height="805"><rect/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&decoy), Some((634, 805)));
    for p in [cm, inch, quoted, decoy] {
      let _ = std::fs::remove_file(p);
    }
  }

  /// A percentage-sized root with no viewBox has no intrinsic pixel size at
  /// all. `None` is the whole point: the caller then emits no width/height and
  /// the browser sizes the image itself, which is strictly better than
  /// asserting `width="100"`.
  #[test]
  fn viewport_px_declines_relative_lengths_rather_than_inventing_pixels() {
    let pct = svg_file("pct", r#"<svg width="100%" height="100%"><rect/></svg>"#);
    assert_eq!(read_svg_viewport_px(&pct), None);
    let none = svg_file(
      "bare",
      r#"<svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&none), None);
    for p in [pct, none] {
      let _ = std::fs::remove_file(p);
    }
  }

  /// The SVG grammar allows comma-separated viewBox numbers.
  #[test]
  fn viewport_px_parses_a_comma_separated_viewbox() {
    let comma = svg_file("comma", r#"<svg viewBox="0,0,634,805"><rect/></svg>"#);
    assert_eq!(read_svg_viewport_px(&comma), Some((634, 805)));
    let _ = std::fs::remove_file(comma);
  }

  /// `read_svg_size_pt` keeps the opposite precedence — absolute lengths first,
  /// viewBox second — because it answers "how big would this typeset?", not
  /// "how many pixels is the viewport?".
  #[test]
  fn size_pt_prefers_absolute_lengths_then_falls_back_to_the_viewbox() {
    // 4in = 288.something TeX pt (72.27/in).
    let inch = svg_file(
      "pt_in",
      r#"<svg width="4in" height="2in" viewBox="0 0 10 5"><rect/></svg>"#,
    );
    let (w, h) = read_svg_size_pt(&inch).expect("absolute lengths");
    assert!((w - 4.0 * 72.27).abs() < 1e-9, "w = {w}");
    assert!((h - 2.0 * 72.27).abs() < 1e-9, "h = {h}");
    // SVG `pt` is a PostScript big point (1/72"), NOT a TeX pt (1/72.27") — so
    // `72pt` is one inch, i.e. 72.27 TeX pt. The old reader equated the two
    // units and under-reported every pt-sized SVG by 0.375%.
    let bigpt = svg_file("pt_pt", r#"<svg width="72pt" height="36pt"><rect/></svg>"#);
    let (w, h) = read_svg_size_pt(&bigpt).expect("pt lengths");
    assert!((w - 72.27).abs() < 1e-9, "w = {w}");
    assert!((h - 72.27 / 2.0).abs() < 1e-9, "h = {h}");
    let _ = std::fs::remove_file(bigpt);
    // Unitless lengths are only a scale — the viewBox wins.
    let unitless = svg_file(
      "pt_vb",
      r#"<svg width="634" height="805" viewBox="0 0 96 48"><rect/></svg>"#,
    );
    let (w, h) = read_svg_size_pt(&unitless).expect("viewBox fallback");
    assert!((w - 72.27).abs() < 1e-9, "w = {w}");
    assert!((h - 72.27 / 2.0).abs() < 1e-9, "h = {h}");
    for p in [inch, unitless] {
      let _ = std::fs::remove_file(p);
    }
  }
}
