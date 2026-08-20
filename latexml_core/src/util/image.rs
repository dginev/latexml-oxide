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

/// One graphicx transformation, as compiled from the option string.
///
/// Port of the `@transform` list Perl `image_graphicx_parse` builds
/// (`Util/Image.pm` L142-196). Lengths are in **bp**, the unit `to_bp` yields,
/// and angles in degrees counter-clockwise, as graphicx states them.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphicxOp {
  /// `page=N` — which page of a multi-page source to take.
  Page(u32),
  /// `trim=l b r t` — amounts to remove from each edge.
  Trim {
    l: f64,
    b: f64,
    r: f64,
    t: f64,
  },
  /// `viewport=llx lly urx ury` — an absolute box (Perl's `clip` op).
  Clip {
    l: f64,
    b: f64,
    r: f64,
    t: f64,
  },
  /// `angle=N`, counter-clockwise.
  Rotate(f64),
  Reflect,
  /// `scale=`/`xscale=`/`yscale=`.
  Scale {
    x: f64,
    y: f64,
  },
  /// `width=`/`height=`/`totalheight=`. A dimension left `None` is derived
  /// from the other through the aspect ratio; Perl spells that as a 999999
  /// sentinel with `keep_aspect` forced on (L188-189).
  ScaleTo {
    w:           Option<f64>,
    h:           Option<f64>,
    keep_aspect: bool,
  },
}

/// A TeX/graphicx length in **bp**. Port of Perl `to_bp` + `%BP_conversions`
/// (`Util/Image.pm` L198-210), including its `true`-prefix strip (`truept`) and
/// its "unknown unit counts as bp" fallback. A value that is not a length at
/// all yields 1, exactly as Perl's `else { return 1 }` does.
pub fn to_bp(x: &str) -> f64 {
  let x = x.trim();
  let split = x
    .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '+' && c != '-')
    .unwrap_or(x.len());
  let (num, unit) = x.split_at(split);
  let Ok(v) = num.parse::<f64>() else {
    return 1.0;
  };
  let unit = unit.trim().strip_prefix("true").unwrap_or(unit.trim());
  let factor = match unit {
    "" | "bp" => 1.0,
    "pt" => 72.0 / 72.27,
    "pc" => 12.0 * 72.0 / 72.27,
    "in" => 72.0,
    "cm" => 72.0 / 2.54,
    "mm" => 72.0 / 25.4,
    "dd" => (72.0 / 72.27) * (1238.0 / 1157.0),
    "cc" => 12.0 * (72.0 / 72.27) * (1238.0 / 1157.0),
    "sp" => 72.0 / 72.27 / 65536.0,
    // Perl: `($u && $BP_conversions{$u}) || 1` — an unrecognised unit falls
    // back to a factor of 1, i.e. the number is taken as bp.
    _ => 1.0,
  };
  v * factor
}

/// Compile a graphicx option string into the transformation sequence.
///
/// Port of Perl `image_graphicx_parse` (`Util/Image.pm` L142-196). Key order
/// matters and is Perl's, in two ways:
///
/// * A rotation is applied **before** scaling when no sizing option preceded
///   the `angle` in the source string, and after it otherwise. Perl decides
///   this the instant it parses `angle` (`$rotfirst = !($width || $height ||
///   $xscale || $yscale)`, L168), from the keys seen *so far* — so
///   `angle=90,width=100pt` rotates then scales, while `width=100pt,angle=90`
///   scales then rotates. graphicx really behaves this way and pdflatex agrees:
///   the first is ~100x200, the second ~50x100 for a 200x100 source. We capture
///   `rot_first` at the same point, not from the final key set.
///
/// `pc` differs from Perl by design: Perl's table has `pc => 12/72.27`, which
/// is 12 *TeX pt* expressed in bp only if you also drop the pt→bp step — a pica
/// is 12 pt, so the factor is `12 * 72/72.27`. Perl's value makes a 1pc box
/// 0.166bp instead of 11.955bp. Ours is the correct one; no test in the corpus
/// exercised `pc`.
pub fn parse_graphicx_options(options: &str) -> Vec<GraphicxOp> {
  let (mut width, mut height) = (None, None);
  let (mut xscale, mut yscale) = (None, None);
  let (mut aspect, mut angle, mut page) = (false, 0.0f64, None);
  let (mut viewport, mut is_trim) = (None, false);
  // Set the instant `angle` is parsed, from the sizing keys seen so far — NOT
  // recomputed from the final key set. Perl `image_graphicx_parse` L168.
  let mut rot_first = false;
  for opt in options.split(',') {
    let opt = opt.trim();
    if opt.is_empty() {
      continue;
    }
    let (key, val) = match opt.split_once('=') {
      Some((k, v)) => (k.trim(), v.trim()),
      None => (opt, ""),
    };
    let box4 = |v: &str| {
      let n: Vec<f64> = v.split_whitespace().map(to_bp).collect();
      if n.len() == 4 {
        Some((n[0], n[1], n[2], n[3]))
      } else {
        None
      }
    };
    match key {
      "width" => width = Some(to_bp(val)),
      "height" | "totalheight" => height = Some(to_bp(val)),
      "scale" => {
        let s = val.parse::<f64>().ok();
        xscale = s;
        yscale = s;
      },
      "xscale" => xscale = val.parse::<f64>().ok(),
      "yscale" => yscale = val.parse::<f64>().ok(),
      "angle" => {
        angle = val.parse::<f64>().unwrap_or(0.0);
        rot_first = width.is_none() && height.is_none() && xscale.is_none() && yscale.is_none();
      },
      "keepaspectratio" => aspect = val != "false",
      "page" => page = val.parse::<u32>().ok(),
      "viewport" => {
        viewport = box4(val);
        is_trim = false;
      },
      "trim" => {
        viewport = box4(val);
        is_trim = true;
      },
      _ => {},
    }
  }

  let mut ops = Vec::new();
  if let Some(p) = page {
    ops.push(GraphicxOp::Page(p));
  }
  if let Some((a, b, c, d)) = viewport {
    ops.push(if is_trim {
      GraphicxOp::Trim { l: a, b, r: c, t: d }
    } else {
      GraphicxOp::Clip { l: a, b, r: c, t: d }
    });
  }
  if rot_first && angle != 0.0 {
    ops.push(GraphicxOp::Rotate(angle));
  }
  match (width, height, xscale, yscale) {
    // Perl L187-189: a single dimension forces aspect preservation, whatever
    // `keepaspectratio` said.
    (Some(w), Some(h), ..) => ops.push(GraphicxOp::ScaleTo {
      w:           Some(w),
      h:           Some(h),
      keep_aspect: aspect,
    }),
    (Some(w), None, ..) => ops.push(GraphicxOp::ScaleTo {
      w:           Some(w),
      h:           None,
      keep_aspect: true,
    }),
    (None, Some(h), ..) => ops.push(GraphicxOp::ScaleTo {
      w:           None,
      h:           Some(h),
      keep_aspect: true,
    }),
    (None, None, Some(x), Some(y)) => ops.push(GraphicxOp::Scale { x, y }),
    (None, None, Some(x), None) => ops.push(GraphicxOp::Scale { x, y: 1.0 }),
    (None, None, None, Some(y)) => ops.push(GraphicxOp::Scale { x: 1.0, y }),
    (None, None, None, None) => {},
  }
  if !rot_first && angle != 0.0 {
    ops.push(GraphicxOp::Rotate(angle));
  }
  ops
}

/// Apply a compiled transformation sequence to a natural size.
///
/// Port of Perl `image_graphicx_size` (`Util/Image.pm` L221-256), generalised
/// over the output unit so the engine and the post-processor share one algebra:
///
/// * `units_per_bp` scales a bp-valued option into the caller's unit —
///   `DPI/72.27` for device pixels (Perl's `$dppt`), `72.27/72` for TeX pt.
/// * `quantize` applies Perl's `ceil` at each sizing step. True in pixel space,
///   where a fractional device pixel is meaningless; false in pt space, where
///   rounding the box to 1/100 inch would be a needless loss of precision.
///
/// `Page` is a selector, not a geometric transform, so it is skipped here —
/// callers read it out separately.
pub fn apply_graphicx_ops(
  mut w: f64,
  mut h: f64,
  ops: &[GraphicxOp],
  units_per_bp: f64,
  quantize: bool,
) -> (f64, f64) {
  let round = |v: f64| if quantize { v.ceil() } else { v };
  for op in ops {
    match *op {
      GraphicxOp::Page(_) | GraphicxOp::Reflect => {},
      GraphicxOp::Scale { x, y } => {
        w = round(w * x);
        h = round(h * y);
      },
      GraphicxOp::ScaleTo { w: rw, h: rh, keep_aspect } => {
        let (tw, th) = (rw.map(|v| v * units_per_bp), rh.map(|v| v * units_per_bp));
        match (tw, th) {
          (Some(tw), Some(th)) if keep_aspect => {
            // Perl L234 `return unless $w && $h` — a degenerate natural size
            // carries no aspect ratio to preserve, and Perl abandons the whole
            // computation rather than guess. The sizer then reports 0.
            if w <= 0.0 || h <= 0.0 {
              return (0.0, 0.0);
            }
            // Perl L233-236: honour the less extreme request, so the result
            // fits inside the requested box.
            if tw / w < th / h {
              h = h * tw / w;
              w = tw;
            } else {
              w = w * th / h;
              h = th;
            }
            w = round(w);
            h = round(h);
          },
          (Some(tw), Some(th)) => {
            w = round(tw);
            h = round(th);
          },
          // A single dimension always preserves aspect (Perl compiles it as a
          // scale-to with a 999999 sentinel and `keep_aspect` forced on), so
          // the same degenerate-size bail applies.
          (Some(tw), None) => {
            if w <= 0.0 || h <= 0.0 {
              return (0.0, 0.0);
            }
            h = round(h * tw / w);
            w = round(tw);
          },
          (None, Some(th)) => {
            if w <= 0.0 || h <= 0.0 {
              return (0.0, 0.0);
            }
            w = round(w * th / h);
            h = round(th);
          },
          (None, None) => {},
        }
      },
      GraphicxOp::Rotate(deg) => {
        // Perl L239-242: `$rad = -$a1 * pi/180`, then the axis-aligned bounding
        // box of the rotated rectangle. Not quantized — Perl does not ceil here.
        let rad = -deg * std::f64::consts::PI / 180.0;
        let (s, c) = (rad.sin(), rad.cos());
        let (nw, nh) = ((w * c).abs() + (h * s).abs(), (w * s).abs() + (h * c).abs());
        w = nw;
        h = nh;
      },
      GraphicxOp::Trim { l, b, r, t } => {
        // Perl L248-250: shrink by the trimmed edges.
        w = round(w - (l + r) * units_per_bp);
        h = round(h - (t + b) * units_per_bp);
      },
      GraphicxOp::Clip { l, b, r, t } => {
        // Perl L252-253: the viewport box IS the new extent.
        w = round((r - l) * units_per_bp);
        h = round((t - b) * units_per_bp);
      },
    }
  }
  (w.max(0.0), h.max(0.0))
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
  // Perl `image_graphicx_size` (Util/Image.pm L221-256) works in device pixels
  // with `$dppt = DPI/72.27`, and derives the box from it at L271.
  let (w, h) = apply_graphicx_ops(
    img_w,
    img_h,
    &parse_graphicx_options(&options),
    dpi / 72.27,
    true,
  );

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

/// The figure's TRUE natural (typeset) size in TeX pt, for the VECTOR formats
/// whose intrinsic size is a real physical dimension: a PDF page box, an EPS/PS
/// `%%BoundingBox` (both bp), or an SVG's lengths/viewBox. `None` for raster
/// formats — a pixel count is not a physical size without a DPI — and when the
/// geometry can't be recovered.
///
/// This is deliberately NOT `image_graphicx_sizer`'s `cached_width`: that runs
/// EPS/raster dimensions through a device-DPI round-trip (`×72.27/DPI`), which is
/// right for the box model's device-pixel sizing but wrong as a physical length.
/// This function is the size a browser should reproduce, used for the
/// font-relative (`em`) sizing of natural-size figure inclusions (#562).
/// Extension-gated so each format is read exactly once; pure Rust, no external
/// tool.
pub fn natural_display_size_pt(path: &Path) -> Option<(f64, f64)> {
  let ext = path
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| e.to_ascii_lowercase());
  match ext.as_deref() {
    Some("pdf") => read_pdf_page_box(path).map(|(w, h)| (bp_to_pt(w), bp_to_pt(h))),
    // read_image_dimensions returns an EPS/PS BoundingBox 1:1 in bp.
    Some("eps" | "ps" | "epsi" | "epsf") => read_image_dimensions(path)
      .filter(|&(w, h)| w > 0 && h > 0)
      .map(|(w, h)| (bp_to_pt(w as f64), bp_to_pt(h as f64))),
    Some("svg" | "svgz") => read_svg_size_pt(path),
    _ => None,
  }
}

/// [`natural_display_size_pt`] over a comma-joined `candidates` string (the
/// `<ltx:graphics candidates=…>` attribute), resolving each candidate against
/// `source_dir` and returning the first that yields a size.
pub fn natural_display_size_pt_of_candidates(
  candidates: &str,
  source_dir: &str,
) -> Option<(f64, f64)> {
  candidates.split(',').find_map(|c| {
    let c = c.trim();
    (!c.is_empty())
      .then(|| natural_display_size_pt(&resolve_candidate(c, source_dir)))
      .flatten()
  })
}

/// pt (f64) → `Dimension` (scaled points).
fn pt_to_dim(pt: f64) -> Dimension { Dimension::new((pt * 65536.0).round() as i64) }

/// Apply graphicx `width`/`height`/`totalheight`/`scale`/`keepaspectratio` to a
/// natural (pt) size, matching pdfTeX/graphics.sty box sizing. Verified against
/// `\the\wd` under pdflatex: an explicit `width=` sets the box width outright,
/// the natural size only supplying the missing dimension via the aspect ratio.
fn graphicx_box_pt(nw: f64, nh: f64, options: &str) -> (Dimension, Dimension) {
  // The same algebra as the pixel branch, in pt and without quantization:
  // options arrive in bp, and 1bp = 72.27/72 pt. Rounding a typeset box to a
  // whole device pixel — which is what the pixel branch's `ceil` amounts to —
  // would throw away four digits of a TeX dimension for nothing.
  let (bw, bh) = apply_graphicx_ops(
    nw,
    nh,
    &parse_graphicx_options(options),
    72.27 / 72.0,
    false,
  );
  (pt_to_dim(bw), pt_to_dim(bh))
}

/// Read a PDF's page box (width, height) in bp — CropBox (pdfTeX's default),
/// else MediaBox. Pure Rust, no external tool (this is what pdfTeX's built-in
/// reader does). Shared with `LaTeXML::Post::Graphics`.
///
/// Looks in the raw bytes first, then inside object streams. `%PDF-1.5` and
/// later — everything current pdflatex emits — may put the page tree in a
/// `/Type /ObjStm` stream, where the box tokens do not appear as raw bytes at
/// all: measured over 14 real PDFs in this repo, 5 were unreadable without this
/// second pass, and `ObjStm` presence predicted it exactly.
///
/// **First box wins**, in file order, as the raw-byte scan has always done. A
/// correct answer for page N would mean resolving the page tree through the
/// xref stream; for the figures `\includegraphics` pulls in, which are
/// single-page, the first box is the page's own (or the `/Pages` node's, which
/// it inherits).
pub fn read_pdf_page_box(path: &Path) -> Option<(f64, f64)> {
  let bytes = std::fs::read(path).ok()?;
  if byte_find(&bytes, b"/CropBox").is_some() || byte_find(&bytes, b"/MediaBox").is_some() {
    let content = String::from_utf8_lossy(&bytes);
    if let Some(box_) =
      parse_pdf_box(&content, "/CropBox").or_else(|| parse_pdf_box(&content, "/MediaBox"))
    {
      return Some(box_);
    }
  }
  let inflated = inflate_object_streams(&bytes)?;
  parse_pdf_box(&inflated, "/CropBox").or_else(|| parse_pdf_box(&inflated, "/MediaBox"))
}

/// Concatenate the inflated contents of every `/Type /ObjStm` in `bytes`.
///
/// Deliberately not a PDF parser: it finds object-stream dictionaries, takes the
/// `stream`…`endstream` payload that follows each, and inflates it. That is
/// enough to expose the page dictionary, and it stops well short of xref-stream
/// parsing and object resolution — which is what a real page-N lookup would
/// need, and is not what a figure's natural size is worth.
///
/// Only `/FlateDecode` streams are attempted (the only filter pdflatex, Ghost-
/// script, Cairo or matplotlib use for object streams), and only the first
/// [`MAX_OBJSTM_SCAN`] of them, so a pathological file cannot turn a size probe
/// into an unbounded decompression.
fn inflate_object_streams(bytes: &[u8]) -> Option<String> {
  use std::io::Read;

  /// Enough for any real document; a figure PDF has one or two.
  const MAX_OBJSTM_SCAN: usize = 64;
  /// Per-stream inflate ceiling, so a zip bomb cannot be handed to us as a
  /// figure. A page dictionary is a few hundred bytes.
  const MAX_INFLATED: u64 = 8 << 20;

  let mut out = String::new();
  let mut from = 0;
  let mut seen = 0;
  while seen < MAX_OBJSTM_SCAN {
    let Some(hit) = byte_find(&bytes[from..], b"/ObjStm") else {
      break;
    };
    let at = from + hit;
    from = at + b"/ObjStm".len();
    seen += 1;
    // The dictionary ends at `stream`, optionally followed by CR, then LF.
    let Some(rel) = byte_find(&bytes[at..], b"stream") else {
      continue;
    };
    let dict = &bytes[at..at + rel];
    if byte_find(dict, b"/FlateDecode").is_none() {
      continue;
    }
    let mut start = at + rel + b"stream".len();
    if bytes.get(start) == Some(&b'\r') {
      start += 1;
    }
    if bytes.get(start) == Some(&b'\n') {
      start += 1;
    }
    let end = byte_find(&bytes[start..], b"endstream").map_or(bytes.len(), |e| start + e);
    let mut buf = Vec::new();
    if flate2::read::ZlibDecoder::new(&bytes[start..end])
      .take(MAX_INFLATED)
      .read_to_end(&mut buf)
      .is_err()
      && buf.is_empty()
    {
      // A truncated or mis-delimited stream still yields the bytes decoded
      // before the error, and the page dictionary sits at the front — so an
      // error is only fatal when nothing at all came out.
      continue;
    }
    out.push_str(&String::from_utf8_lossy(&buf));
    out.push('\n');
  }
  (!out.is_empty()).then_some(out)
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

/// Natural SVG size in pt, from the root `<svg>` element: the root
/// `width`/`height` lengths (a unitless value is CSS px, exactly as a browser
/// treats it), else the `viewBox` extent (user units ≈ CSS px) as a last
/// resort. Gives at least a correct aspect ratio, which is all a `width=`-ed
/// inclusion needs. `None` if the file isn't an SVG or has no usable geometry.
///
/// Width/height lead and the viewBox only backstops them — see
/// [`read_svg_viewport_px`] for the full rationale (issue #696).
fn read_svg_size_pt(path: &Path) -> Option<(f64, f64)> {
  let head = read_head_lossy(path)?;
  let tag = svg_root_tag(&head)?;
  if let Some((w, h)) = svg_root_lengths_px(tag) {
    return Some((px_to_pt(w), px_to_pt(h)));
  }
  let (vw, vh) = svg_viewbox_extent(tag)?;
  // viewBox user units ≈ CSS px (1/96"); convert to pt for a plausible scale.
  Some((px_to_pt(vw), px_to_pt(vh)))
}

/// SVG **viewport** size in CSS px, the way a browser takes it: the root
/// `width`/`height` (a unitless value is CSS px), falling back to the `viewBox`
/// only when the lengths are absent or relative (`%`). `None` when neither is
/// usable, so the caller omits the dimensions and lets the browser size it.
///
/// Basis for `imagewidth`/`imageheight` in `LaTeXML::Post::Graphics`. The
/// `viewBox` is only a coordinate system, not the rendered size; preferring it
/// under-sized SVGs whose lengths disagreed with it (issue #696, reported by the
/// LaTeXML maintainer). Not parity-relevant: Perl parses no SVG — it renders via
/// Image::Magick, whose raster follows `width`/`height`, not the `viewBox`
/// (`Util/Image.pm:86-97`); `pdftocairo`/`mutool` are our own beyond-Perl
/// PDF→SVG pipeline, absent from Perl.
pub fn read_svg_viewport_px(path: &Path) -> Option<(u32, u32)> {
  let head = read_head_lossy(path)?;
  let tag = svg_root_tag(&head)?;
  let (w, h) = svg_root_lengths_px(tag).or_else(|| svg_viewbox_extent(tag))?;
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

/// The root `<svg>` `width`/`height` as a CSS-px pair — the browser's sizing
/// basis. `Some` only when **both** are absolute lengths (a unitless value is
/// user units = px, a unit-bearing value is converted); `None` if either is
/// missing or relative (`%`, `em`, …), so the caller falls back to the viewBox.
fn svg_root_lengths_px(tag: &str) -> Option<(f64, f64)> {
  Some((
    svg_attr_len_px(tag, "width")?,
    svg_attr_len_px(tag, "height")?,
  ))
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

  /// The viewport reader sizes the way a browser does: the root `width`/`height`
  /// lead, the `viewBox` only backstops them (issue #696). `pdftocairo -svg`
  /// writes `width="612pt"`, which a browser renders at 612·96/72 = 816 px — not
  /// the viewBox's 612. `mutool draw -F svg` writes a unitless `612`, i.e. 612
  /// px, matching its viewBox. Root tags copied verbatim from the tools.
  #[test]
  fn viewport_px_sizes_from_root_lengths_like_a_browser() {
    let pdftocairo = svg_file(
      "pdftocairo",
      r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="612pt" height="792pt" viewBox="0 0 612 792">
<defs/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&pdftocairo), Some((816, 1056)));
    let mutool = svg_file(
      "mutool",
      r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" version="1.1" width="612" height="792" viewBox="0 0 612 792">
<defs/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&mutool), Some((612, 792)));
    let _ = std::fs::remove_file(pdftocairo);
    let _ = std::fs::remove_file(mutool);
  }

  /// The `viewBox` is only a fallback now: it sizes the viewport iff the root
  /// carries no absolute `width`/`height`. When both are present the lengths win
  /// (that is the whole of issue #696), so a viewBox that disagrees with them is
  /// ignored for sizing.
  #[test]
  fn viewport_px_uses_the_viewbox_only_when_lengths_are_absent() {
    let vb_only = svg_file(
      "vb_only",
      r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 480"><rect/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&vb_only), Some((640, 480)));
    // Lengths present and disagreeing with the viewBox → lengths win.
    let both = svg_file(
      "vb_both",
      r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 640 480"><rect/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&both), Some((200, 100)));
    // A percentage width is not absolute → fall through to the viewBox.
    let pct_w = svg_file(
      "vb_pct",
      r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 640 480"><rect/></svg>"#,
    );
    assert_eq!(read_svg_viewport_px(&pct_w), Some((640, 480)));
    for p in [vb_only, both, pct_w] {
      let _ = std::fs::remove_file(p);
    }
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

  /// `read_svg_size_pt` shares the viewport reader's precedence — root lengths
  /// first, viewBox second — differing only in that it answers "how big would
  /// this typeset (pt)?" rather than "how many px is the viewport?". SVG `pt` is
  /// a PostScript big point (1/72"), and a unitless length is CSS px.
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
    // Unitless lengths are CSS px, and they WIN over a disagreeing viewBox
    // (issue #696): 96 px → 72.27 pt, 48 px → 36.135 pt, not the viewBox's 634.
    let unitless = svg_file(
      "pt_len",
      r#"<svg width="96" height="48" viewBox="0 0 634 805"><rect/></svg>"#,
    );
    let (w, h) = read_svg_size_pt(&unitless).expect("root lengths");
    assert!((w - 72.27).abs() < 1e-9, "w = {w}");
    assert!((h - 72.27 / 2.0).abs() < 1e-9, "h = {h}");
    // Only a root without absolute lengths falls back to the viewBox.
    let vb_only = svg_file("pt_vb", r#"<svg viewBox="0 0 96 48"><rect/></svg>"#);
    let (w, h) = read_svg_size_pt(&vb_only).expect("viewBox fallback");
    assert!((w - 72.27).abs() < 1e-9, "w = {w}");
    assert!((h - 72.27 / 2.0).abs() < 1e-9, "h = {h}");
    for p in [inch, unitless, vb_only] {
      let _ = std::fs::remove_file(p);
    }
  }
}

/// Characterization tests for the engine-side image sizing pipeline.
///
/// **These pin behaviour, not correctness.** Several of the numbers below are
/// known to disagree with pdflatex — an EPS BoundingBox is read as pixels, a
/// PNG is assumed to be 100 dpi, an SVG 96 dpi, and a box is quantized to whole
/// device pixels. They are recorded exactly as they are today so that the
/// planned unification of the sizing pipeline (one probe, one resolution
/// policy, one graphicx algebra) has to declare every change it makes instead
/// of drifting silently. When a value here changes, that is a decision, and the
/// comment above it says which way the current number leans.
///
/// Measured references, same 200x100 figure in each format, `\the\wd0` with no
/// graphicx options, recorded 2026-08-04:
///
/// | source              | pdflatex   | Perl LaTeXML | here       |
/// |---------------------|------------|--------------|------------|
/// | PNG 200x100 px      | 200.7495pt | 144.54pt     | 144.54pt   |
/// | EPS BBox 200x100 bp | -          | (no sizer)   | 144.54pt   |
/// | PDF 200x100 bp      | 200.7495pt | (no sizer)   | 200.75pt   |
/// | SVG viewBox 200x100 | -          | (no sizer)   | 150.5625pt |
#[cfg(test)]
mod sizing_characterization_tests {
  use super::*;

  fn fixture(name: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!("lxsize-{}-{name}", std::process::id()));
    std::fs::write(&path, bytes).expect("write fixture");
    path
  }

  /// A minimal `%PDF-1.5` whose only object is a Flate-compressed object stream
  /// carrying `payload` — the shape pdflatex emits for a page tree since 1.5.
  fn objstm_pdf(payload: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(payload).expect("deflate");
    let body = enc.finish().expect("finish");
    let mut pdf = Vec::from(
      &b"%PDF-1.5\n1 0 obj\n<< /Type /ObjStm /N 1 /First 4 /Filter /FlateDecode >>\nstream\n"[..],
    );
    pdf.extend_from_slice(&body);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");
    pdf
  }

  /// A PNG header with the given IHDR dimensions. `read_image_dimensions` reads
  /// a fixed 32-byte prefix and takes bytes 16..24 as width/height, so an
  /// honest signature + IHDR is the whole contract; no CRC is consulted.
  fn png_header(w: u32, h: u32) -> Vec<u8> {
    let mut v = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    v.extend_from_slice(&13u32.to_be_bytes());
    v.extend_from_slice(b"IHDR");
    v.extend_from_slice(&w.to_be_bytes());
    v.extend_from_slice(&h.to_be_bytes());
    v.extend_from_slice(&[0x08, 0x02, 0x00, 0x00, 0x00]);
    v.extend_from_slice(&[0u8; 16]); // pad past the 32-byte read_exact
    v
  }

  /// A JPEG with a single SOF0 frame header. The reader scans markers for
  /// 0xC0..=0xCF (minus DHT/JPG/DAC) and takes height then width, big-endian.
  fn jpeg_header(w: u16, h: u16) -> Vec<u8> {
    let mut v = vec![0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08];
    v.extend_from_slice(&h.to_be_bytes());
    v.extend_from_slice(&w.to_be_bytes());
    v.extend_from_slice(&[0x03, 0x01, 0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01]);
    v.extend_from_slice(&[0xFF, 0xD9]);
    v.extend_from_slice(&[0u8; 16]);
    v
  }

  // ── layer 1: what each format probe returns, and in which unit ────────

  /// PNG and JPEG report true device pixels; EPS reports **bp** through the
  /// same `(u32, u32)` channel. Nothing in the type distinguishes them, which
  /// is the defect the unification is meant to remove — pinned here so the
  /// removal is visible.
  #[test]
  fn read_image_dimensions_returns_pixels_for_raster_and_bp_for_eps() {
    let png = fixture("dims.png", &png_header(200, 100));
    assert_eq!(read_image_dimensions(&png), Some((200, 100)), "PNG IHDR px");

    let jpg = fixture("dims.jpg", &jpeg_header(640, 480));
    assert_eq!(read_image_dimensions(&jpg), Some((640, 480)), "JPEG SOF px");

    // 200 x 100 **bp**, handed back as if it were 200 x 100 pixels.
    let eps = fixture(
      "dims.eps",
      b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 200 100\n%%EndComments\n",
    );
    assert_eq!(
      read_image_dimensions(&eps),
      Some((200, 100)),
      "EPS bp-as-px"
    );

    // HiResBoundingBox wins over BoundingBox when both are present.
    let hires = fixture(
      "hires.eps",
      b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 200 100\n\
        %%HiResBoundingBox: 0 0 199.5 99.4\n%%EndComments\n",
    );
    assert_eq!(
      read_image_dimensions(&hires),
      Some((200, 99)),
      "HiRes wins, rounded"
    );

    // Formats this reader does not know stay `None` — that is what routes a
    // PDF or an SVG to the `natural_size_pt` fallback.
    let pdf = fixture(
      "dims1.pdf",
      b"%PDF-1.4\n1 0 obj\n<< /MediaBox [0 0 200 100] >>\nendobj\n",
    );
    assert_eq!(
      read_image_dimensions(&pdf),
      None,
      "PDF is not this reader's job"
    );
  }

  /// The PDF page box: CropBox is pdfTeX's default and wins over MediaBox, and
  /// a box compressed into an object stream is inflated and read.
  ///
  /// That last case is not hypothetical. Across 14 real PDFs in this repo, 5
  /// returned `None` before the object-stream pass, correlating exactly with
  /// `ObjStm` — the default for `%PDF-1.5` and later, which is what modern
  /// pdflatex emits — and those figures reached the engine with a 0x0 natural
  /// box. All 14 now match `pdfinfo` exactly.
  #[test]
  fn read_pdf_page_box_prefers_cropbox_and_reaches_into_object_streams() {
    let media = fixture("m.pdf", b"%PDF-1.4\n<< /MediaBox [0 0 200 100] >>\n");
    assert_eq!(read_pdf_page_box(&media), Some((200.0, 100.0)));

    let both = fixture(
      "b.pdf",
      b"%PDF-1.4\n<< /MediaBox [0 0 612 792] /CropBox [0 0 200 100] >>\n",
    );
    assert_eq!(
      read_pdf_page_box(&both),
      Some((200.0, 100.0)),
      "CropBox wins"
    );

    // Non-zero origin: the box is the extent, not the corner.
    let offset = fixture("o.pdf", b"%PDF-1.4\n<< /MediaBox [10 20 210 120] >>\n");
    assert_eq!(read_pdf_page_box(&offset), Some((200.0, 100.0)));

    // A real object stream: the box exists only as deflated bytes.
    let objstm = fixture(
      "h.pdf",
      &objstm_pdf(b"5 0 << /Type /Page /MediaBox [0 0 200 100] >>"),
    );
    assert_eq!(read_pdf_page_box(&objstm), Some((200.0, 100.0)));

    // A CropBox inside the stream still wins over a MediaBox beside it.
    let cropped = fixture(
      "hc.pdf",
      &objstm_pdf(b"5 0 << /MediaBox [0 0 612 792] /CropBox [0 0 200 100] >>"),
    );
    assert_eq!(read_pdf_page_box(&cropped), Some((200.0, 100.0)));

    // Nothing readable anywhere: an object stream we cannot inflate.
    let opaque = fixture(
      "ho.pdf",
      b"%PDF-1.5\n<< /Type /ObjStm /N 12 /Filter /FlateDecode >>\nstream\nnot-zlib\nendstream\n",
    );
    assert_eq!(read_pdf_page_box(&opaque), None);
  }

  /// `natural_size_pt` is the only place a file-read number is actually
  /// converted from its own unit into TeX pt — and it uses a different
  /// resolution per format: PDF at 72 (bp), SVG at 96 (CSS px).
  #[test]
  fn natural_size_pt_uses_72_for_pdf_and_96_for_svg() {
    let pdf = fixture("n.pdf", b"%PDF-1.4\n<< /MediaBox [0 0 200 100] >>\n");
    let (w, h) = natural_size_pt(&pdf).expect("pdf box");
    assert!((w - 200.0 * 72.27 / 72.0).abs() < 1e-9, "w = {w}"); // 200.75
    assert!((h - 100.0 * 72.27 / 72.0).abs() < 1e-9, "h = {h}");

    let svg = fixture("n.svg", br#"<svg viewBox="0 0 200 100"><rect/></svg>"#);
    let (w, h) = natural_size_pt(&svg).expect("svg viewport");
    assert!((w - 200.0 * 72.27 / 96.0).abs() < 1e-9, "w = {w}"); // 150.5625
    assert!((h - 100.0 * 72.27 / 96.0).abs() < 1e-9, "h = {h}");

    // A raster file has no page box and no SVG root: `None`, so the caller
    // keeps whatever the pixel reader gave it.
    let png = fixture("n.png", &png_header(200, 100));
    assert_eq!(natural_size_pt(&png), None);
  }

  /// The compiled op *sequence* depends on where `angle` sits relative to the
  /// sizing keys — the ordering graphicx and pdflatex both honour. Pinned at the
  /// parse layer so the rule is guarded without a rasterizer: `angle` before a
  /// sizing key rotates first, after it rotates last, and a rotation with no
  /// sizing key at all rotates first.
  #[test]
  fn parse_orders_rotation_by_key_position() {
    use GraphicxOp::*;
    let w100 = ScaleTo {
      w:           Some(to_bp("100pt")),
      h:           None,
      keep_aspect: true,
    };
    assert_eq!(
      parse_graphicx_options("angle=90,width=100pt"),
      vec![Rotate(90.0), w100.clone()],
      "angle first -> rotate then scale"
    );
    assert_eq!(
      parse_graphicx_options("width=100pt,angle=90"),
      vec![w100, Rotate(90.0)],
      "width first -> scale then rotate"
    );
    assert_eq!(
      parse_graphicx_options("angle=90"),
      vec![Rotate(90.0)],
      "no sizing key -> rotate first (trivially)"
    );
    // scale is a sizing key too, so it flips the order the same way.
    assert_eq!(
      parse_graphicx_options("angle=90,scale=2")[0],
      Rotate(90.0),
      "angle before scale -> rotate first"
    );
    assert_eq!(
      parse_graphicx_options("scale=2,angle=90")[1],
      Rotate(90.0),
      "angle after scale -> rotate last"
    );
  }

  // ── layer 3a: the pt-space algebra (fallback branch) ──────────────────

  /// `graphicx_box_pt` works in pt throughout and never quantizes, so an
  /// explicit `width=100pt` comes out as exactly 100pt. Compare
  /// `sizer_quantizes_the_box_to_whole_device_pixels` below, which is the
  /// px-space algebra answering the *same* request with 99.7326pt.
  #[test]
  fn graphicx_box_pt_table() {
    let pt = |d: Dimension| d.value_of() as f64 / 65536.0;
    let case = |opts: &str| {
      let (w, h) = graphicx_box_pt(200.0, 100.0, opts);
      (pt(w), pt(h))
    };
    let near = |got: (f64, f64), want: (f64, f64), label: &str| {
      assert!(
        (got.0 - want.0).abs() < 1e-3 && (got.1 - want.1).abs() < 1e-3,
        "{label}: got {got:?}, want {want:?}"
      );
    };
    near(case(""), (200.0, 100.0), "no options = natural size");
    near(
      case("width=100pt"),
      (100.0, 50.0),
      "width= drives height by aspect",
    );
    near(
      case("height=25pt"),
      (50.0, 25.0),
      "height= drives width by aspect",
    );
    near(
      case("totalheight=25pt"),
      (50.0, 25.0),
      "totalheight aliases height",
    );
    near(case("scale=0.5"), (100.0, 50.0), "scale=");
    near(
      case("width=100pt,height=80pt"),
      (100.0, 80.0),
      "both, no keepaspect",
    );
    // keepaspectratio drops the more extreme request and fits inside the box.
    near(
      case("width=100pt,height=80pt,keepaspectratio"),
      (100.0, 50.0),
      "keepaspectratio fits width",
    );
    near(
      case("width=400pt,height=80pt,keepaspectratio"),
      (160.0, 80.0),
      "keepaspectratio fits height",
    );
    // An explicit width wins over scale (scale is only consulted when neither
    // width nor height is given).
    near(
      case("scale=2,width=100pt"),
      (100.0, 50.0),
      "width beats scale",
    );
    // Units other than pt parse through `Dimension::from_str`.
    near(case("width=1in"), (72.27, 36.135), "in parses");
    // A degenerate natural size carries no aspect ratio, and a lone `width=`
    // always wants one — Perl abandons the computation rather than guess
    // (`Util/Image.pm` L234), reporting nothing, i.e. a zero box.
    let (w, h) = graphicx_box_pt(0.0, 0.0, "width=100pt");
    near((pt(w), pt(h)), (0.0, 0.0), "zero natural height");
  }

  // ── layer 3b: the px-space algebra, and the whole seam ────────────────

  /// Drive the real entry point, `image_graphicx_sizer`, with an absolute
  /// candidate path so no `SOURCEDIRECTORY` is needed. Returns cached
  /// (width, height) in pt.
  fn sizer_pt(path: &Path, options: &str) -> (f64, f64) {
    let mut w = Whatsit::default();
    w.set_property("candidates", path.to_string_lossy().to_string());
    w.set_property("options", options.to_string());
    image_graphicx_sizer(&mut w);
    let get = |k: &str| match w.get_property(k).map(|c| c.into_owned()) {
      Some(Stored::Dimension(d)) => d.value_of() as f64 / 65536.0,
      other => panic!("{k} was {other:?}"),
    };
    (get("cached_width"), get("cached_height"))
  }

  /// **The whole engine-side seam, as one matrix.** Same 200x100 figure in
  /// four containers, eight option strings, `cached_width`/`cached_height` in
  /// pt. This is the table the unified pipeline has to reproduce, row by row,
  /// or explicitly change.
  ///
  /// What each surprising row records:
  ///
  /// * **Four resolutions.** With no options the same figure is 144.54pt as a
  ///   PNG or EPS (100 dpi), 200.75pt as a PDF (72 dpi, i.e. bp), 150.5625pt as
  ///   an SVG (96 dpi, CSS px), and 0 as a PDF whose page box sits in an object
  ///   stream. pdflatex says 200.7495pt for the PNG and the PDF alike.
  /// * **Two algebras.** PNG/EPS take the px-space branch, PDF/SVG the pt-space
  ///   `graphicx_box_pt` fallback. They answer `width=100pt` differently:
  ///   99.7326 vs 100.0, because the px branch quantizes the box to a whole
  ///   device pixel (100pt -> 99.6265bp -> 137.848 px -> ceil 138 -> 99.7326pt).
  /// * **A single dimension always preserves aspect**, on both branches, as
  ///   Perl does by compiling `width=` alone into a scale-to with a 999999
  ///   sentinel and `keep_aspect` forced on (`Util/Image.pm` L188-189). Until
  ///   2026-08-04 the px branch left the height at its natural value; the
  ///   `keepaspectratio=true` that `graphicx_sty` injects had been hiding it
  ///   from ordinary LaTeX.
  /// * **`angle=` rotates the reserved box** (Perl L238-242). Until 2026-08-04
  ///   neither branch implemented the op, so a sideways figure reserved its
  ///   unrotated width — a Rust-only gap, since Perl has always rotated:
  ///   measured `angle=90` on the PNG gives Perl 72.27 x 144.54, and that is
  ///   now what this matrix pins.
  /// * **The last-resort branch** (unreadable page box) honours an explicit
  ///   `width=`/`height=` and reports 0 for the dimension not asked for.
  #[test]
  fn sizer_matrix_across_formats_and_options() {
    let png = fixture("m.png", &png_header(200, 100));
    let eps = fixture(
      "m.eps",
      b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 200 100\n",
    );
    let pdf = fixture("m.pdf", b"%PDF-1.4\n<< /MediaBox [0 0 200 100] >>\n");
    let svg = fixture("m.svg", br#"<svg viewBox="0 0 200 100"><rect/></svg>"#);
    let objstm = fixture("m2.pdf", b"%PDF-1.5\n<< /Type /ObjStm >>\nstream\n..\n");

    // (source, options, expected width pt, expected height pt)
    #[rustfmt::skip]
    let matrix: &[(&str, &str, f64, f64)] = &[
      // px-space branch: raster pixels, and an EPS BoundingBox read as pixels.
      ("png", "",                                 144.5400,  72.2700),
      ("png", "width=100pt,keepaspectratio=true",  99.7326,  49.8663),
      ("png", "width=100pt",                       99.7326,  49.8663),
      ("png", "scale=0.5",                         72.2700,  36.1350),
      ("png", "height=25pt,keepaspectratio=true",  49.8663,  25.2945),
      ("png", "width=100pt,height=80pt",           99.7326,  80.2197),
      ("png", "width=1in,keepaspectratio=true",    72.2700,  36.1350),
      ("png", "angle=90",                          72.2700, 144.5400),
      ("eps", "",                                 144.5400,  72.2700),
      ("eps", "width=100pt,keepaspectratio=true",  99.7326,  49.8663),
      ("eps", "width=100pt",                       99.7326,  49.8663),
      ("eps", "scale=0.5",                         72.2700,  36.1350),
      ("eps", "height=25pt,keepaspectratio=true",  49.8663,  25.2945),
      ("eps", "width=100pt,height=80pt",           99.7326,  80.2197),
      ("eps", "width=1in,keepaspectratio=true",    72.2700,  36.1350),
      ("eps", "angle=90",                          72.2700, 144.5400),
      // pt-space branch: the `natural_size_pt` fallback, no quantization.
      ("pdf", "",                                 200.7500, 100.3750),
      ("pdf", "width=100pt,keepaspectratio=true", 100.0000,  50.0000),
      ("pdf", "width=100pt",                      100.0000,  50.0000),
      ("pdf", "scale=0.5",                        100.3750,  50.1875),
      ("pdf", "height=25pt,keepaspectratio=true",  50.0000,  25.0000),
      ("pdf", "width=100pt,height=80pt",          100.0000,  80.0000),
      ("pdf", "width=1in,keepaspectratio=true",    72.2700,  36.1350),
      ("pdf", "angle=90",                         100.3750, 200.7500),
      ("svg", "",                                 150.5625,  75.2812),
      ("svg", "width=100pt,keepaspectratio=true", 100.0000,  50.0000),
      ("svg", "width=100pt",                      100.0000,  50.0000),
      ("svg", "scale=0.5",                         75.2812,  37.6406),
      ("svg", "height=25pt,keepaspectratio=true",  50.0000,  25.0000),
      ("svg", "width=100pt,height=80pt",          100.0000,  80.0000),
      ("svg", "width=1in,keepaspectratio=true",    72.2700,  36.1350),
      ("svg", "angle=90",                          75.2812, 150.5625),
      // last resort: nothing measurable, only an explicit request is honoured.
      ("objstm", "",                                0.0000,   0.0000),
      ("objstm", "width=100pt,keepaspectratio=true", 100.0000, 0.0000),
      ("objstm", "width=100pt",                    100.0000,   0.0000),
      ("objstm", "scale=0.5",                        0.0000,   0.0000),
      ("objstm", "height=25pt,keepaspectratio=true", 0.0000,  25.0000),
      ("objstm", "width=100pt,height=80pt",        100.0000,  80.0000),
      ("objstm", "width=1in,keepaspectratio=true",  72.2700,   0.0000),
      ("objstm", "angle=90",                         0.0000,   0.0000),
    ];

    // Report EVERY divergence, not just the first: when this matrix moves it is
    // usually because a shared rule changed, and the whole delta is the useful
    // signal.
    let mut deltas = Vec::new();
    for (src, opts, want_w, want_h) in matrix {
      let path = match *src {
        "png" => &png,
        "eps" => &eps,
        "pdf" => &pdf,
        "svg" => &svg,
        _ => &objstm,
      };
      let (w, h) = sizer_pt(path, opts);
      // 1e-3 pt is ~1/70000 inch: far tighter than any behaviour change, loose
      // enough to survive `Dimension`'s fixed-point round trip.
      if (w - want_w).abs() >= 1e-3 || (h - want_h).abs() >= 1e-3 {
        deltas.push(format!(
          "  {src:<7} [{opts}]\n      pinned ({want_w:.4}, {want_h:.4})  got ({w:.4}, {h:.4})"
        ));
      }
    }
    assert!(
      deltas.is_empty(),
      "{} of {} pinned rows moved:\n{}",
      deltas.len(),
      matrix.len(),
      deltas.join("\n")
    );
  }
}
