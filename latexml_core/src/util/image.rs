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
    // Strip surrounding double-quotes from the search directory, symmetric to
    // the `path.trim_matches('"')` above. A quoted `\graphicspath{{"./dir"}}`
    // (or `\svgpath` / `--graphicspaths`) otherwise joins to a `"…"` path that
    // never resolves. See OXIDIZED_DESIGN #55.
    let dir = dir.trim().trim_matches('"');
    let base = PathBuf::from(dir).join(path);
    if has_extension {
      if base.exists() {
        // Perl relativizes every hit to SOURCEDIRECTORY via pathname_relative
        // (→ File::Spec->abs2rel), which emits a `../…` path for a graphic in a
        // SIBLING directory (issue #698: `\subimport*{../gfx_asset/}` reaching a
        // sideways tree). See `pathname::relative`, which now matches Perl (it
        // used to leak the absolute path on a non-descendant hit).
        let rel = match &source_path {
          Some(sp) => {
            crate::util::pathname::relative(&base.to_string_lossy(), &sp.to_string_lossy())
          },
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
            // Sibling-directory relativization (issue #698) — see the
            // extension branch above: pathname::relative (abs2rel semantics).
            let rel = match &source_path {
              Some(sp) => {
                crate::util::pathname::relative(&full.to_string_lossy(), &sp.to_string_lossy())
              },
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
      // machine path. `pathname::relative` now emits that `../…` form for a
      // non-descendant tree (issue #698 fixed its strip_prefix leak).
      let rel = match &source_path {
        Some(sp) => crate::util::pathname::relative(&found, &sp.to_string_lossy()),
        None => found,
      };
      candidates.push(rel);
    }
  }

  // Beyond Perl (OXIDIZED_DESIGN_DIVERGENCES #230): a name WITH an extension
  // that the search paths do not hold is asked of kpsewhich as-is. Perl (and
  // the branch above) consults kpsewhich only for extensionless names with
  // `.png`/`.pdf` appended (Util/Image.pm:49-53), so a package-shipped asset
  // referenced by its full name — `\includegraphics[page=N]
  // {openmoji-color-all.pdf}` and the other icon galleries, 97.8 % of the
  // corpus's lost figures (~27,700) — never resolved although kpsewhich finds
  // it in the texmf tree. Guard: `cluster_package_guards::graphics_kpsewhich::
  // extensioned_texmf_graphic_is_a_candidate`.
  if candidates.is_empty()
    && has_extension
    && let Some(found) = crate::util::pathname::kpsewhich(&[path])
  {
    let rel = match &source_path {
      Some(sp) => crate::util::pathname::relative(&found, &sp.to_string_lossy()),
      None => found,
    };
    candidates.push(rel);
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

/// The pixel rectangle a `trim`/`viewport` op keeps of a `w`×`h` raster, as
/// `(x, y, width, height)` from its top-left corner; `None` when the box keeps
/// the whole image or no pixel of it. Port of Perl `image_graphicx_complex`'s
/// trim/clip arm (`Util/Image.pm` L400-418): TeX measures the box from the
/// lower-left corner and ImageMagick from the upper-left, and the box is
/// clamped to the image, never padded. `px_per_bp` is the raster's own
/// resolution over 72 (DIVERGENCES #337: Perl divides by 72.27 and reads the
/// resolution without its unit).
pub fn graphicx_crop_rect(
  w: u32,
  h: u32,
  op: &GraphicxOp,
  px_per_bp: f64,
) -> Option<(u32, u32, u32, u32)> {
  let (wf, hf) = (w as f64, h as f64);
  // A length in pt reaches here as bp through 72/72.27, so a whole number of
  // pixels can land a hair off it (`100.375pt` is 100.00000000000001bp):
  // snap within a millionth before rounding, or the box gains a pixel row.
  let floor = |v: f64| (v + 1e-6).floor();
  let ceil = |v: f64| (v - 1e-6).ceil();
  let (x0, y0, ww, hh) = match *op {
    // Amounts to trim: left, bottom, right, top.
    GraphicxOp::Trim { l, b, r, t } => (
      floor(l * px_per_bp),
      floor(t * px_per_bp),
      ceil(wf - (l + r) * px_per_bp),
      ceil(hf - (t + b) * px_per_bp),
    ),
    // The box itself: lower-left and upper-right corners.
    GraphicxOp::Clip { l, b, r, t } => (
      floor(l * px_per_bp),
      floor(hf - t * px_per_bp),
      ceil((r - l) * px_per_bp),
      ceil((t - b) * px_per_bp),
    ),
    _ => return None,
  };
  if !(x0 > 0.0 || y0 > 0.0 || x0 + ww < wf || y0 + hh < hf) {
    return None;
  }
  let (x, y) = (x0.max(0.0), y0.max(0.0));
  let cw = (ww + x0.min(0.0)).min(wf - x);
  let ch = (hh + y0.min(0.0)).min(hf - y);
  (cw >= 1.0 && ch >= 1.0).then_some((x as u32, y as u32, cw as u32, ch as u32))
}

/// A raster's own resolution in dots per inch, `(x, y)`, as pdfTeX reads it
/// to size an image: a PNG's `pHYs` chunk (unit 1 is per metre), rounded to
/// whole dpi, or a JPEG's JFIF `APP0` density (unit 1 per inch; 2 per
/// centimetre, truncated to whole dpi) — pdflatex measures an 11811 px/m PNG
/// at 300 dpi and a 118 px/cm JPEG at 299. `None` when the file states none,
/// or only an aspect ratio (unit 0): pdfTeX then takes 72 dpi, its default
/// `\pdfimageresolution`. Reads the head only: both come before the pixels.
pub fn raster_resolution_dpi(path: &str) -> Option<(f64, f64)> {
  use std::io::Read;
  let mut bytes = Vec::new();
  std::fs::File::open(path)
    .ok()?
    .take(1 << 18)
    .read_to_end(&mut bytes)
    .ok()?;
  if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
    // Chunks: length, type, data, CRC; `pHYs` precedes the first `IDAT`.
    let mut at = 8;
    while at + 8 <= bytes.len() {
      let len = u32::from_be_bytes(bytes[at..at + 4].try_into().ok()?) as usize;
      let kind = &bytes[at + 4..at + 8];
      let data = bytes.get(at + 8..at + 8 + len)?;
      match kind {
        b"pHYs" if len == 9 && data[8] == 1 => {
          let x = u32::from_be_bytes(data[0..4].try_into().ok()?) as f64;
          let y = u32::from_be_bytes(data[4..8].try_into().ok()?) as f64;
          return (x > 0.0 && y > 0.0).then_some(((x * 0.0254).round(), (y * 0.0254).round()));
        },
        b"IDAT" | b"IEND" => return None,
        _ => at += 12 + len,
      }
    }
    None
  } else if bytes.starts_with(&[0xFF, 0xD8]) {
    // Marker segments after SOI; JFIF's APP0 comes before the frame.
    let mut at = 2;
    while at + 4 <= bytes.len() && bytes[at] == 0xFF {
      let marker = bytes[at + 1];
      let len = u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]) as usize;
      let data = bytes.get(at + 4..at + 2 + len)?;
      if marker == 0xE0 && data.len() >= 12 && data.starts_with(b"JFIF\0") {
        let x = u16::from_be_bytes([data[8], data[9]]) as f64;
        let y = u16::from_be_bytes([data[10], data[11]]) as f64;
        let (x, y) = match data[7] {
          1 => (x, y),
          2 => ((x * 2.54).trunc(), (y * 2.54).trunc()),
          _ => return None,
        };
        return (x > 0.0 && y > 0.0).then_some((x, y));
      }
      let frame = (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker);
      if marker == 0xDA || frame {
        return None;
      }
      at += 2 + len;
    }
    None
  } else {
    None
  }
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

/// Run a fallible I/O op, retrying on a *transient* lock. On Windows a
/// just-written file — a figure the converter emitted a moment ago, or a test
/// fixture — can be momentarily locked by another handle (antivirus real-time
/// scanning of the fresh file, or Windows' stricter default file sharing). A
/// bare `op().ok()?` would turn that into a silent `None`, and a figure would
/// reach the engine at 0x0.
///
/// A genuine `NotFound` is not a lock, so it fails fast. Every *other* error is
/// treated as possibly-transient and retried with a widening backoff up to
/// ~0.5 s total. The fresh-file lock usually surfaces as `PermissionDenied`
/// (`ERROR_SHARING_VIOLATION`), but under heavy parallel load (a full `cargo
/// test` with antivirus active) it has shown other kinds and needed longer than
/// a few ms to clear — so this deliberately retries broadly rather than gating on
/// one `ErrorKind`. A permanently-unreadable path pays the full budget once and
/// then fails; the happy path (Ok on the first try) pays nothing.
fn with_transient_retry<T>(mut op: impl FnMut() -> std::io::Result<T>) -> Option<T> {
  let mut tries = 0u32;
  loop {
    match op() {
      Ok(v) => return Some(v),
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
      Err(_) if tries < 10 => {
        tries += 1;
        std::thread::sleep(std::time::Duration::from_millis(u64::from(tries) * 10));
      },
      Err(_) => return None,
    }
  }
}

/// [`std::fs::read`] with the transient-lock retry of [`with_transient_retry`].
fn read_file_resilient(path: &Path) -> Option<Vec<u8>> {
  with_transient_retry(|| std::fs::read(path))
}

/// [`std::fs::File::open`] with the transient-lock retry of
/// [`with_transient_retry`]. Only the open races with the scanner/writer; reads
/// on the returned handle do not, so callers keep their streaming reads.
fn open_file_resilient(path: &Path) -> Option<std::fs::File> {
  with_transient_retry(|| std::fs::File::open(path))
}

/// Read image dimensions (width, height) in pixels from a file.
/// Supports PNG, JPEG, and EPS (PostScript BoundingBox).
///
/// This is a narrow replacement for `Image::Size::imgsize` (Perl
/// `image_size` at Util::Image L86-97). Only a few formats are needed
/// for typical arXiv graphics inclusions — anything else returns `None`
/// so the caller skips sizing (mirroring Perl's `return unless $w`).
pub fn read_image_dimensions(path: &Path) -> Option<(u32, u32)> {
  IMAGE_DIMENSIONS_MEMO.with(|m| {
    if let Some(hit) = m.borrow().get(path) {
      return *hit;
    }
    let value = read_image_dimensions_uncached(path);
    m.borrow_mut().insert(path.to_path_buf(), value);
    value
  })
}

// Per-thread memos of the on-disk size reads, keyed by path and cleared per
// conversion (`clear_image_size_memo`, beside the kpsewhich memo). pdfTeX
// reads an image file's box ONCE and shares the XObject across every
// `\includegraphics` of it (`page=N` selects a page of the read resource);
// without the memo every inclusion re-read the file — hwemoji's manual
// includes its 6.7 MB, 3,677-page `hwemoji-assets.pdf` ~7,800 times through
// `\hwemoji@insert` (hwemoji.sty:11), three reads each, and timed out at
// 420 s once #230 let the asset resolve (2 s before). Guard:
// `cluster_package_guards::graphics_asset_memo::repeated_inclusions_of_one_asset_read_it_once`.
type SizeMemo<T> = std::cell::RefCell<rustc_hash::FxHashMap<PathBuf, Option<T>>>;
std::thread_local! {
  static IMAGE_DIMENSIONS_MEMO: SizeMemo<(u32, u32)> =
    std::cell::RefCell::new(rustc_hash::FxHashMap::default());
  static PDF_PAGE_BOX_MEMO: SizeMemo<(f64, f64)> =
    std::cell::RefCell::new(rustc_hash::FxHashMap::default());
}

/// Clear the per-thread image size memos. Called at the start of every
/// conversion, beside `clear_kpsewhich_memo`: a long-lived worker thread
/// converts many papers, and a file may change between them.
pub fn clear_image_size_memo() {
  IMAGE_DIMENSIONS_MEMO.with(|m| m.borrow_mut().clear());
  PDF_PAGE_BOX_MEMO.with(|m| m.borrow_mut().clear());
  PDF_PAGE_COUNT_MEMO.with(|m| m.borrow_mut().clear());
}

fn read_image_dimensions_uncached(path: &Path) -> Option<(u32, u32)> {
  use std::io::Read;
  let mut file = open_file_resilient(path)?;
  let mut header = [0u8; 32];
  file.read_exact(&mut header).ok()?;

  // PNG: signature + IHDR chunk
  if &header[0..8] == b"\x89PNG\r\n\x1a\n" {
    let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
    let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
    return Some((width, height));
  }

  // JPEG: the frame header (SOF) gives the size; walk the segments to it,
  // seeking past each one rather than reading the file.
  if header[0] == 0xFF && header[1] == 0xD8 {
    return read_jpeg_frame_size(&mut file);
  }

  // EPS: PostScript BoundingBox comment. Perl: LaTeXML::Util::Image reads
  // the leading `%%BoundingBox: llx lly urx ury` (values in bp, 1bp=1/72").
  // `%%HiResBoundingBox:` is preferred when present (float precision).
  if (header[0] == b'%' && (header[1] == b'!' || header[1] == b'%'))
    || header.starts_with(b"\xc5\xd0\xd3\xc6")
  // EPS with binary preview header
  {
    let boxes = read_postscript_boxes(path)?;
    let (llx, lly, urx, ury) = boxes.hires_bounding_box.or(boxes.bounding_box)?;
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

/// The DSC bounding-box comments of a PostScript file (Adobe DSC 3.0:
/// `%%BoundingBox:`, `%%HiResBoundingBox:`, `(atend)`), as `(llx, lly, urx, ury)`
/// in bp.
///
/// A comment line is ASCII, but its neighbours need not be text: a Latin-1
/// `%%Title`, a binary preview, a multi-byte character cut at the window's
/// edge. So the file is scanned as bytes, in lines ending at CR, LF or CRLF.
/// Only the header window (the first 32 KB, `PS_COMMENT_WINDOW`) is read, and
/// the trailer window too when the box is deferred with `(atend)`. The first
/// `%%BoundingBox:` line decides, as in graphics.sty (`\Gread@eps`, 391-399)
/// and epstopdf; after `(atend)` the trailer's last box wins, as DSC and
/// graphics.sty read it. Comments inside an embedded document
/// (`%%BeginDocument` .. `%%EndDocument`) are its own, not the file's, and are
/// skipped. A DOS EPS binary header (`C5 D0 D3 C6`, then the PostScript
/// section's offset and length, little-endian) is followed to its PostScript
/// section. pdflatex (through epstopdf) sizes a Latin-1, a CR-only, an
/// `(atend)` and a DOS EPS alike; repro `graphics-tikz/eps_bounding_box_bytes.tex`.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PostScriptBoxes {
  pub bounding_box:       Option<(f64, f64, f64, f64)>,
  pub hires_bounding_box: Option<(f64, f64, f64, f64)>,
}

/// How much of a PostScript file's head (and, for `(atend)`, its tail) is read.
const PS_COMMENT_WINDOW: u64 = 32 * 1024;

/// See [`PostScriptBoxes`]; `None` when the file cannot be read.
pub fn read_postscript_boxes(path: &Path) -> Option<PostScriptBoxes> {
  use std::io::{Read, Seek, SeekFrom};
  let mut file = open_file_resilient(path)?;
  let file_len = file.metadata().ok()?.len();
  let mut dos = [0u8; 12];
  let dos_len = file.read(&mut dos).ok()?;
  let (start, end) = if dos_len == 12 && dos.starts_with(b"\xc5\xd0\xd3\xc6") {
    let offset = u64::from(u32::from_le_bytes([dos[4], dos[5], dos[6], dos[7]]));
    let length = u64::from(u32::from_le_bytes([dos[8], dos[9], dos[10], dos[11]]));
    (
      offset.min(file_len),
      offset.saturating_add(length).min(file_len),
    )
  } else {
    (0, file_len)
  };
  let mut window = |from: u64, to: u64| -> Option<Vec<u8>> {
    file.seek(SeekFrom::Start(from)).ok()?;
    let mut bytes = Vec::new();
    (&mut file).take(to - from).read_to_end(&mut bytes).ok()?;
    Some(bytes)
  };
  let mut boxes = PostScriptBoxes::default();
  let mut atend = false;
  let mut seen_box = false;
  let mut depth = 0usize;
  for line in dsc_lines(&window(start, (start + PS_COMMENT_WINDOW).min(end))?) {
    if line.starts_with(b"%%BeginDocument") {
      depth += 1;
      continue;
    }
    if line.starts_with(b"%%EndDocument") {
      depth = depth.saturating_sub(1);
      continue;
    }
    if depth > 0 {
      continue;
    }
    if let Some(rest) = line.strip_prefix(b"%%BoundingBox:") {
      // A line that reads as no box is passed over (epstopdf's value match).
      if !seen_box {
        if rest.trim_ascii() == b"(atend)" {
          seen_box = true;
          atend = true;
        } else if let Some(bbox) = parse_bbox_bytes(rest) {
          seen_box = true;
          boxes.bounding_box = Some(bbox);
        }
      }
    } else if let Some(rest) = line.strip_prefix(b"%%HiResBoundingBox:")
      && boxes.hires_bounding_box.is_none()
    {
      boxes.hires_bounding_box = parse_bbox_bytes(rest);
    }
  }
  if atend {
    // The trailer is at the file's top level: read backwards, an
    // `%%EndDocument` opening an embedded document, its `%%BeginDocument`
    // closing it.
    let tail = window(end.saturating_sub(PS_COMMENT_WINDOW).max(start), end)?;
    let mut depth = 0usize;
    for line in dsc_lines(&tail).collect::<Vec<_>>().into_iter().rev() {
      if line.starts_with(b"%%EndDocument") {
        depth += 1;
      } else if line.starts_with(b"%%BeginDocument") {
        depth = depth.saturating_sub(1);
      } else if depth == 0
        && let Some(rest) = line.strip_prefix(b"%%BoundingBox:")
        && let Some(bbox) = parse_bbox_bytes(rest)
      {
        boxes.bounding_box = Some(bbox);
        break;
      }
    }
  }
  Some(boxes)
}

/// The lines of `bytes`, split at CR or LF, leading blanks trimmed.
fn dsc_lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
  bytes
    .split(|&b| b == b'\n' || b == b'\r')
    .map(<[u8]>::trim_ascii_start)
}

fn parse_bbox_bytes(rest: &[u8]) -> Option<(f64, f64, f64, f64)> {
  parse_bbox(std::str::from_utf8(rest.trim_ascii()).ok()?)
}

/// The size of a JPEG's first frame, `(width, height)` in pixels, from its SOF
/// segment (ITU T.81 B.2.2: `FFCn`, length, precision, height, width). The
/// segments before it are skipped by their lengths; the entropy-coded data
/// after SOS (`FFDA`) is never read.
fn read_jpeg_frame_size(file: &mut std::fs::File) -> Option<(u32, u32)> {
  use std::io::{BufReader, Read, Seek, SeekFrom};
  let mut reader = BufReader::new(file);
  reader.seek(SeekFrom::Start(2)).ok()?;
  let mut byte = [0u8; 1];
  loop {
    reader.read_exact(&mut byte).ok()?;
    if byte[0] != 0xFF {
      return None;
    }
    // Fill bytes: any number of FF before the marker code.
    let marker = loop {
      reader.read_exact(&mut byte).ok()?;
      if byte[0] != 0xFF {
        break byte[0];
      }
    };
    match marker {
      // Stand-alone markers carry no length.
      0x01 | 0xD0..=0xD7 => continue,
      // SOS or EOI before any frame: no size.
      0xDA | 0xD9 => return None,
      _ => {},
    }
    let mut length = [0u8; 2];
    reader.read_exact(&mut length).ok()?;
    let length = i64::from(u16::from_be_bytes(length));
    // SOF markers: 0xC0-0xCF (except 0xC4 DHT, 0xC8 JPG, 0xCC DAC)
    if (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
      let mut frame = [0u8; 5];
      reader.read_exact(&mut frame).ok()?;
      let height = u32::from(u16::from_be_bytes([frame[1], frame[2]]));
      let width = u32::from(u16::from_be_bytes([frame[3], frame[4]]));
      return Some((width, height));
    }
    if length < 2 {
      return None;
    }
    reader.seek_relative(length - 2).ok()?;
  }
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
  PDF_PAGE_BOX_MEMO.with(|m| {
    if let Some(hit) = m.borrow().get(path) {
      return *hit;
    }
    let value = read_pdf_page_box_uncached(path);
    m.borrow_mut().insert(path.to_path_buf(), value);
    value
  })
}

fn read_pdf_page_box_uncached(path: &Path) -> Option<(f64, f64)> {
  let bytes = read_file_resilient(path)?;
  if let Some(box_) =
    parse_pdf_box(&bytes, b"/CropBox").or_else(|| parse_pdf_box(&bytes, b"/MediaBox"))
  {
    return Some(box_);
  }
  let inflated = inflate_object_streams(&bytes)?;
  parse_pdf_box(&inflated, b"/CropBox").or_else(|| parse_pdf_box(&inflated, b"/MediaBox"))
}

/// The page count of a PDF, as pdfTeX's `\pdflastximagepages` reports it
/// after `\pdfximage`: the root `/Type /Pages` node's `/Count`. Without a
/// full parser, the root is the `/Pages` dictionary with the LARGEST `/Count`
/// (intermediate nodes count only their subtree); raw bytes first, then the
/// inflated object streams (PDF 1.5+, see [`read_pdf_page_box`]). A PDF with
/// no `/Pages` dictionary at all falls back to counting `/Type /Page` objects.
pub fn read_pdf_page_count(path: &Path) -> Option<u32> {
  PDF_PAGE_COUNT_MEMO.with(|m| {
    if let Some(hit) = m.borrow().get(path) {
      return *hit;
    }
    let value = read_pdf_page_count_uncached(path);
    m.borrow_mut().insert(path.to_path_buf(), value);
    value
  })
}

std::thread_local! {
  static PDF_PAGE_COUNT_MEMO: SizeMemo<u32> =
    std::cell::RefCell::new(rustc_hash::FxHashMap::default());
}

fn read_pdf_page_count_uncached(path: &Path) -> Option<u32> {
  let bytes = read_file_resilient(path)?;
  if let Some(n) = max_pages_count(&bytes) {
    return Some(n);
  }
  let inflated = inflate_object_streams(&bytes);
  if let Some(content) = inflated.as_deref()
    && let Some(n) = max_pages_count(content)
  {
    return Some(n);
  }
  let pages = count_page_objects(&bytes) + inflated.as_deref().map_or(0, count_page_objects);
  if pages > 0 { Some(pages) } else { None }
}

/// The largest `/Count N` of a `/Type /Pages` dictionary. From each
/// `/Type /Pages` (with or without the space) scan forward to the `/Count` at
/// the dictionary's own brace depth — a nested `/Resources << … >>` before it
/// must not end the search — stopping at the dictionary's closing `>>`.
fn max_pages_count(content: &[u8]) -> Option<u32> {
  let mut best: Option<u32> = None;
  for key in [&b"/Type /Pages"[..], b"/Type/Pages"] {
    for at in memchr::memmem::find_iter(content, key) {
      let start = at + key.len();
      let mut depth = 0i32;
      let mut i = start;
      while i < content.len() {
        let rest = &content[i..];
        if rest.starts_with(b"<<") {
          depth += 1;
          i += 2;
          continue;
        }
        if rest.starts_with(b">>") {
          if depth == 0 {
            break;
          }
          depth -= 1;
          i += 2;
          continue;
        }
        if depth == 0 && rest.starts_with(b"/Count") {
          if let Some(n) = leading_u32(trim_start_pdf_whitespace(&rest[6..])) {
            best = Some(best.map_or(n, |b| b.max(n)));
          }
          break;
        }
        i += 1;
      }
    }
  }
  best
}

fn count_page_objects(content: &[u8]) -> u32 {
  let mut n = 0;
  for key in [&b"/Type /Page"[..], b"/Type/Page"] {
    for at in memchr::memmem::find_iter(content, key) {
      // `/Page` followed by a delimiter, not `/Pages`.
      if content.get(at + key.len()) != Some(&b's') {
        n += 1;
      }
    }
  }
  n
}

/// PDF's white-space characters: NUL, HT, LF, FF, CR and SP (ISO 32000-1
/// §7.2.2, Table 1). They separate an array's numbers; nothing else does — a
/// non-ASCII space next to a digit (a UTF-8 NBSP) is not one, where the earlier
/// reading of a UTF-8 copy split on any Unicode white space.
fn is_pdf_whitespace(b: u8) -> bool { matches!(b, 0 | b'\t' | b'\n' | 0x0c | b'\r' | b' ') }

fn trim_start_pdf_whitespace(bytes: &[u8]) -> &[u8] {
  let skip = bytes.iter().take_while(|&&b| is_pdf_whitespace(b)).count();
  &bytes[skip..]
}

/// The run of ASCII digits `bytes` opens with, as a `u32`.
fn leading_u32(bytes: &[u8]) -> Option<u32> {
  let len = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
  std::str::from_utf8(&bytes[..len]).ok()?.parse().ok()
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
fn inflate_object_streams(bytes: &[u8]) -> Option<Vec<u8>> {
  use std::io::Read;

  /// Enough for any real document; a figure PDF has one or two.
  const MAX_OBJSTM_SCAN: usize = 64;
  /// Per-stream inflate ceiling, so a zip bomb cannot be handed to us as a
  /// figure. A page dictionary is a few hundred bytes.
  const MAX_INFLATED: u64 = 8 << 20;

  let mut out = Vec::new();
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
    out.extend_from_slice(&buf);
    out.push(b'\n');
  }
  (!out.is_empty()).then_some(out)
}

/// Parse `TOKEN [ llx lly urx ury ]` from PDF content, returning `(w, h)`.
///
/// The PDF's bytes are searched as they are. The key, the brackets and the
/// numbers are ASCII, which a lossy UTF-8 copy keeps in the same order, so the
/// same box is found; the separators are PDF's own white space
/// ([`is_pdf_whitespace`]). Making that copy of a whole figure PDF cost, in
/// release instructions with `--preload=ar5iv.sty`, 18.7 % of arXiv
/// 2605.13583's conversion, 14.5 % of 2605.09543's and 8.0 % of 2605.08504's.
fn parse_pdf_box(content: &[u8], token: &[u8]) -> Option<(f64, f64)> {
  let start = byte_find(content, token)? + token.len();
  let rest = &content[start..];
  let lb = memchr::memchr(b'[', rest)?;
  let rb = memchr::memchr(b']', &rest[lb..])? + lb;
  let mut it = rest[lb + 1..rb]
    .split(|&b| is_pdf_whitespace(b))
    .filter(|s| !s.is_empty())
    .filter_map(|s| std::str::from_utf8(s).ok()?.parse::<f64>().ok());
  let (x0, y0, x1, y1) = (it.next()?, it.next()?, it.next()?, it.next()?);
  Some(((x1 - x0).abs(), (y1 - y0).abs()))
}

/// Byte-level substring search.
fn byte_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  if needle.is_empty() || needle.len() > haystack.len() {
    return None;
  }
  memchr::memmem::find(haystack, needle)
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
    // A per-call sequence makes every fixture path unique. Two tests reused the
    // same `name` ("m.pdf"), so keying only on pid+name let them race on one temp
    // file when run in parallel: whichever wrote last won, and the other test's
    // reader saw the wrong bytes -> a flaky `None` under full-suite load (it only
    // surfaced when scheduling made the two writes overlap).
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("lxsize-{}-{seq}-{name}", std::process::id()));
    std::fs::write(&path, bytes).expect("write fixture");
    path
  }

  fn io_err(kind: std::io::ErrorKind) -> std::io::Error { std::io::Error::from(kind) }

  /// A genuine `NotFound` is a missing file, not a lock: it must map straight to
  /// `None` on the first attempt, with no retry and no sleep. The "no sleep"
  /// half is what keeps `read_pdf_page_box`/`read_image_dimensions` cheap for
  /// the common missing-figure case, so the perf argument depends on it.
  #[test]
  fn transient_retry_notfound_is_immediate_none() {
    let mut calls = 0u32;
    let got: Option<()> = with_transient_retry(|| {
      calls += 1;
      Err(io_err(std::io::ErrorKind::NotFound))
    });
    assert!(got.is_none(), "NotFound must map to None");
    assert_eq!(calls, 1, "NotFound must not be retried");
  }

  /// A lock that clears after a couple of tries: the op is retried and its
  /// eventual `Ok` is returned — a fresh figure is not silently dropped to 0x0.
  #[test]
  fn transient_retry_recovers_after_transient_errors() {
    let mut calls = 0u32;
    let got = with_transient_retry(|| {
      calls += 1;
      if calls < 3 {
        Err(io_err(std::io::ErrorKind::PermissionDenied))
      } else {
        Ok(42u32)
      }
    });
    assert_eq!(
      got,
      Some(42),
      "a clearing lock should be retried then succeed"
    );
    assert_eq!(calls, 3, "should retry until the op succeeds");
  }

  /// A persistent non-`NotFound` error gives up with `None` after the retry cap
  /// (1 initial attempt + 10 retries = 11 invocations) instead of looping.
  #[test]
  fn transient_retry_gives_up_after_cap() {
    let mut calls = 0u32;
    let got: Option<()> = with_transient_retry(|| {
      calls += 1;
      Err(io_err(std::io::ErrorKind::PermissionDenied))
    });
    assert!(got.is_none(), "a persistent error must give up with None");
    assert_eq!(calls, 11, "one attempt then retries up to the cap");
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

  /// The DSC comments are read as bytes, in CR/LF lines, in a head window and,
  /// for `(atend)`, a tail window; a DOS EPS header is followed to its
  /// PostScript section. Each shape lost its box before (strict UTF-8 over the
  /// whole window, `str::lines`, no tail, no seek past the first 32 KB).
  #[test]
  fn postscript_boxes_are_read_as_bytes() {
    let bbox = Some((0.0, 0.0, 144.0, 72.0));
    let latin1 = fixture(
      "latin1.eps",
      b"%!PS-Adobe-3.0 EPSF-3.0\n%%Title: caf\xe9\n%%BoundingBox: 0 0 144 72\n%%EndComments\n",
    );
    assert_eq!(
      read_postscript_boxes(&latin1).unwrap().bounding_box,
      bbox,
      "Latin-1 title"
    );
    assert_eq!(
      read_image_dimensions(&latin1),
      Some((144, 72)),
      "Latin-1 title"
    );

    let cr = fixture(
      "cr.eps",
      b"%!PS-Adobe-3.0 EPSF-3.0\r%%BoundingBox: 0 0 144 72\r%%EndComments\r",
    );
    assert_eq!(read_image_dimensions(&cr), Some((144, 72)), "CR line ends");

    // The trailer lies past the head window; an embedded document's box before
    // it does not count (the trailer's, the last, wins).
    let mut atend = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: (atend)\n%%EndComments\n".to_vec();
    atend
      .extend_from_slice(b"%%BeginDocument: inner.eps\n%%BoundingBox: 0 0 10 10\n%%EndDocument\n");
    atend.extend(std::iter::repeat_n(b"% filler\n".as_slice(), 5000).flatten());
    atend.extend_from_slice(b"%%Trailer\n%%BoundingBox: 0 0 144 72\n%%EOF\n");
    let atend = fixture("atend.eps", &atend);
    assert_eq!(
      read_postscript_boxes(&atend).unwrap().bounding_box,
      bbox,
      "(atend) trailer"
    );

    // Within one window: the trailer's box is the last at the top level; the
    // embedded document's own trailer, earlier in the tail, does not count.
    let two_trailers = fixture(
      "two_trailers.eps",
      b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: (atend)\n%%EndComments\n\
        %%BeginDocument: inner.eps\n%%BoundingBox: (atend)\n%%Trailer\n%%BoundingBox: 0 0 10 10\n\
        %%EndDocument\n%%Trailer\n%%BoundingBox: 0 0 144 72\n%%EOF\n",
    );
    assert_eq!(
      read_postscript_boxes(&two_trailers).unwrap().bounding_box,
      bbox,
      "outer trailer"
    );

    // The first box decides: an embedded document's `(atend)` or HiRes box is its
    // own (graphics.sty `\Gread@eps`, epstopdf keep the first real box).
    let embedded = fixture(
      "embedded.eps",
      b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 144 72\n%%EndComments\n\
        %%BeginDocument: inner.eps\n%%BoundingBox: (atend)\n%%HiResBoundingBox: 0 0 10 10\n\
        %%Trailer\n%%BoundingBox: 0 0 10 10\n%%EndDocument\n",
    );
    let bad_first = fixture(
      "bad_first.eps",
      b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: none\n%%BoundingBox: 0 0 144 72\n",
    );
    assert_eq!(
      read_postscript_boxes(&bad_first).unwrap().bounding_box,
      bbox,
      "unreadable first"
    );
    let boxes = read_postscript_boxes(&embedded).unwrap();
    assert_eq!(
      (boxes.bounding_box, boxes.hires_bounding_box),
      (bbox, None),
      "embedded"
    );
    assert_eq!(
      read_image_dimensions(&embedded),
      Some((144, 72)),
      "embedded"
    );

    // A DOS EPS whose preview comes first, putting the PostScript past 32 KB.
    let ps = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 144 72\n%%EndComments\n";
    let preview = vec![0xFFu8; 40_000];
    let ps_offset = 30 + preview.len() as u32;
    let mut dos = b"\xc5\xd0\xd3\xc6".to_vec();
    for field in [ps_offset, ps.len() as u32, 0, 0, 30, preview.len() as u32] {
      dos.extend_from_slice(&field.to_le_bytes());
    }
    dos.extend_from_slice(&[0xFF, 0xFF]);
    dos.extend_from_slice(&preview);
    dos.extend_from_slice(ps);
    let dos = fixture("dos.eps", &dos);
    assert_eq!(
      read_image_dimensions(&dos),
      Some((144, 72)),
      "DOS EPS header"
    );
  }

  /// The JPEG reader seeks from segment to segment to the frame header, past
  /// application segments and fill bytes, and never reads the scan data.
  #[test]
  fn jpeg_frame_size_skips_segments() {
    let mut jpg = vec![0xFF, 0xD8];
    // APP1 of 60,000 bytes (an Exif thumbnail), then a fill byte before SOF2.
    jpg.extend_from_slice(&[0xFF, 0xE1]);
    jpg.extend_from_slice(&60_000u16.to_be_bytes());
    jpg.extend(std::iter::repeat_n(0u8, 60_000 - 2));
    jpg.extend_from_slice(&[0xFF, 0xFF, 0xC2, 0x00, 0x11, 0x08]);
    jpg.extend_from_slice(&480u16.to_be_bytes());
    jpg.extend_from_slice(&640u16.to_be_bytes());
    jpg.extend_from_slice(&[
      0x03, 0x01, 0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01, 0xFF, 0xD9,
    ]);
    assert_eq!(
      read_image_dimensions(&fixture("app1.jpg", &jpg)),
      Some((640, 480))
    );
    // Scan data before any frame: no size.
    let no_frame = [0xFF, 0xD8, 0xFF, 0xDA, 0x00, 0x02, 0x12, 0x34, 0xFF, 0xD9];
    let mut no_frame = no_frame.to_vec();
    no_frame.extend_from_slice(&[0u8; 32]);
    assert_eq!(
      read_image_dimensions(&fixture("noframe.jpg", &no_frame)),
      None
    );
  }

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

  /// The size and page-count probes search the PDF's bytes, which need not be
  /// UTF-8 (binary comment line, text strings in a dictionary). A byte that is
  /// not UTF-8 inside the `/Pages` dictionary, before its `/Count`: the count
  /// scan sliced a lossy UTF-8 copy of the file at every byte index and
  /// panicked inside the U+FFFD that byte had become.
  #[test]
  fn pdf_probes_read_through_non_utf8_bytes() {
    let pages = fixture(
      "p8.pdf",
      b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n<< /Type /Pages /Title (\xE9t\xE9) /Count 3 >>\n",
    );
    assert_eq!(read_pdf_page_count(&pages), Some(3));

    // The box's numbers may be separated by any PDF white space, here FF and HT.
    let boxed = fixture(
      "b8.pdf",
      b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n<< /Title (\xE9) /MediaBox [0\x0c0 200\t100] >>\n",
    );
    assert_eq!(read_pdf_page_box(&boxed), Some((200.0, 100.0)));

    // The same through an object stream, and a token that is not UTF-8 in the box.
    let streamed = fixture(
      "s8.pdf",
      &objstm_pdf(b"2 0 << /Type /Pages /T (\xE9) /Count 4 >>"),
    );
    assert_eq!(read_pdf_page_count(&streamed), Some(4));
    let odd = fixture("o8.pdf", b"%PDF-1.4\n<< /MediaBox [0 0 \xFF 200 100] >>\n");
    assert_eq!(read_pdf_page_box(&odd), Some((200.0, 100.0)));
    // NUL is PDF white space; a UTF-8 NBSP between digits is not.
    let nul = fixture("n8.pdf", b"%PDF-1.4\n<< /MediaBox [0\x000 200 100] >>\n");
    assert_eq!(read_pdf_page_box(&nul), Some((200.0, 100.0)));
    let nbsp = fixture(
      "nb8.pdf",
      b"%PDF-1.4\n<< /MediaBox [0 0 200\xC2\xA0100 50] >>\n",
    );
    assert_eq!(read_pdf_page_box(&nbsp), None);

    // No `/Pages` dictionary: the `/Type /Page` objects are counted, not `/Pages`.
    let loose = fixture(
      "l8.pdf",
      b"%PDF-1.4\n<< /Type /Page /T (\xFF) >>\n<< /Type/Page >>\n<< /Type /Pagesx >>\n",
    );
    assert_eq!(read_pdf_page_count(&loose), Some(2));
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

  /// Perl `image_graphicx_complex`'s trim/clip arm (`Util/Image.pm` L400-418),
  /// on a 200×100 raster at 72 dpi (1 px = 1 bp): `trim=100 0 0 50` keeps the
  /// bottom-right quadrant and `viewport=0 0 100 50` the bottom-left one, as
  /// pdflatex shows them; a box over the whole image crops nothing.
  #[test]
  fn graphicx_crop_rect_follows_perl_trim_and_viewport() {
    let trim = GraphicxOp::Trim {
      l: 100.0,
      b: 0.0,
      r: 0.0,
      t: 50.0,
    };
    assert_eq!(
      graphicx_crop_rect(200, 100, &trim, 1.0),
      Some((100, 50, 100, 50))
    );
    let viewport = GraphicxOp::Clip {
      l: 0.0,
      b: 0.0,
      r: 100.0,
      t: 50.0,
    };
    assert_eq!(
      graphicx_crop_rect(200, 100, &viewport, 1.0),
      Some((0, 50, 100, 50))
    );
    let nothing = GraphicxOp::Trim { l: 0.0, b: 0.0, r: 0.0, t: 0.0 };
    assert_eq!(graphicx_crop_rect(200, 100, &nothing, 1.0), None);
    // A viewport reaching past the image is clamped, never padded.
    let past = GraphicxOp::Clip {
      l: -20.0,
      b: 0.0,
      r: 50.0,
      t: 100.0,
    };
    assert_eq!(
      graphicx_crop_rect(200, 100, &past, 1.0),
      Some((0, 0, 50, 100))
    );
    // 2510.17772 Fig 7: `trim=90 30 50 50` on a 3000×1500 raster at 300 dpi.
    let witness = GraphicxOp::Trim {
      l: 90.0,
      b: 30.0,
      r: 50.0,
      t: 50.0,
    };
    assert_eq!(
      graphicx_crop_rect(3000, 1500, &witness, 300.0 / 72.0),
      Some((375, 208, 2417, 1167))
    );
  }

  /// A PNG's `pHYs` counts per metre, a JFIF density per inch or centimetre,
  /// rounded and truncated to whole dpi as pdfTeX takes them; an aspect-only
  /// density (unit 0) states no resolution.
  #[test]
  fn raster_resolution_reads_png_and_jfif_units() {
    let dir = std::env::temp_dir().join(format!("raster_dpi_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut png = png_header(10, 10)[..33].to_vec();
    png.extend_from_slice(&9u32.to_be_bytes());
    png.extend_from_slice(b"pHYs");
    png.extend_from_slice(&11811u32.to_be_bytes());
    png.extend_from_slice(&11811u32.to_be_bytes());
    png.extend_from_slice(&[1, 0, 0, 0, 0]); // unit, CRC
    let png_path = dir.join("r.png");
    std::fs::write(&png_path, &png).unwrap();
    assert_eq!(
      raster_resolution_dpi(png_path.to_str().unwrap()),
      Some((300.0, 300.0))
    );
    let jfif = |unit: u8| {
      let mut v = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
      v.extend_from_slice(b"JFIF\0\x01\x02");
      v.push(unit);
      v.extend_from_slice(&[0x00, 0x76, 0x00, 0x76, 0, 0]); // 118 × 118
      v
    };
    let jpg_path = dir.join("r.jpg");
    std::fs::write(&jpg_path, jfif(2)).unwrap();
    assert_eq!(
      raster_resolution_dpi(jpg_path.to_str().unwrap()),
      Some((299.0, 299.0))
    );
    std::fs::write(&jpg_path, jfif(0)).unwrap();
    assert_eq!(raster_resolution_dpi(jpg_path.to_str().unwrap()), None);
    std::fs::remove_dir_all(&dir).ok();
  }
}
