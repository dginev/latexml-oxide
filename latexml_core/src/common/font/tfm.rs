//! TeX font metric files: the character dimensions of a `.tfm`, scaled as TeX scales them.
//!
//! A character of a font loaded with `\font` (preload.ltx's `\tenln`, a document's
//! `\font\x=lcircle10`) whose family has no standard metric is measured from its own TFM:
//! the Unicode character a fontmap decodes it to belongs to some other font (line10's
//! slot 45 is an arrowhead 10pt wide; the arrow it maps to is cmmi's 5pt `\vec`).
//! Witness 2605.02221 (diagrams.sty arrows). Guard
//! `perfect_kernel_batch58::line_font_char_has_its_tfm_width`. The size is the font's size; TeX
//! scales by the design size when `\font` has no `at` (tex.web §568; SYNC_STATUS 58f residual).
use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashMap;

use crate::util::pathname;

/// The dimension tables of one TFM (tftopl.web / tex.web §539-545).
pub struct Tfm {
  /// The design size in points (header word 1, a fix_word in points; tex.web §568); `None` when
  /// the header has no such word or it is below 1pt, which TeX rejects as a bad TFM.
  pub design_size: Option<f64>,
  bc:              usize,
  char_info:       Vec<[u8; 4]>,
  widths:          Vec<[u8; 4]>,
  heights:         Vec<[u8; 4]>,
  depths:          Vec<[u8; 4]>,
}

thread_local! {
  static TFM_CACHE: RefCell<FxHashMap<String, Option<Rc<Tfm>>>> =
    RefCell::new(FxHashMap::default());
}

/// Forgets the TFMs read so far, misses included: a document may ship its own `.tfm`, so the
/// next document must not be sized from it (`Converter::prepare_session`, per paper in a
/// persistent worker; `reset_thread_engine`).
pub fn reset_tfm_cache() { TFM_CACHE.with(|cache| cache.borrow_mut().clear()); }

/// The TFM of the font named `name` (`line10`), found through kpathsea; `None` when there is
/// no such file or it is no TFM. Cached per name.
pub fn tfm_for(name: &str) -> Option<Rc<Tfm>> {
  TFM_CACHE.with(|cache| {
    if let Some(hit) = cache.borrow().get(name) {
      return hit.clone();
    }
    let file = format!("{name}.tfm");
    let parsed = pathname::kpsewhich(&[file.as_str()])
      .and_then(|path| std::fs::read(path).ok())
      .and_then(|bytes| Tfm::parse(&bytes))
      .map(Rc::new);
    cache.borrow_mut().insert(name.to_string(), parsed.clone());
    parsed
  })
}

impl Tfm {
  /// tex.web §540: twelve halfwords `lf lh bc ec nw nh nd ni nl nk ne np`, then the `lh`-word
  /// header, the char_info words for `bc..=ec`, and the width, height and depth tables.
  fn parse(b: &[u8]) -> Option<Tfm> {
    if b.len() < 24 {
      return None;
    }
    let hw = |i: usize| u16::from_be_bytes([b[2 * i], b[2 * i + 1]]) as usize;
    let (lf, lh, bc, ec, nw, nh, nd) = (hw(0), hw(1), hw(2), hw(3), hw(4), hw(5), hw(6));
    if lf * 4 != b.len() || bc > ec + 1 || ec > 255 {
      return None; // JFM/OFM headers differ
    }
    let words = |start: usize, n: usize| -> Option<Vec<[u8; 4]>> {
      (0..n)
        .map(|i| {
          let o = start + 4 * i;
          b.get(o..o + 4).map(|w| [w[0], w[1], w[2], w[3]])
        })
        .collect()
    };
    let char_start = 24 + 4 * lh;
    let nc = (ec + 1).saturating_sub(bc);
    let width_start = char_start + 4 * nc;
    let height_start = width_start + 4 * nw;
    let depth_start = height_start + 4 * nh;
    let design = (lh >= 2)
      .then(|| {
        b.get(28..32)
          .map(|w| i32::from_be_bytes([w[0], w[1], w[2], w[3]]))
      })
      .flatten()
      .filter(|&fix| fix >= 1 << 20);
    Some(Tfm {
      design_size: design.map(|fix| f64::from(fix) / f64::from(1 << 20)),
      bc,
      char_info: words(char_start, nc)?,
      widths: words(width_start, nw)?,
      heights: words(height_start, nh)?,
      depths: words(depth_start, nd)?,
    })
  }

  /// Width, height and depth in scaled points of character `code` at size `at` (sp), or `None`
  /// when the font has no such character (a zero width index, tex.web §554).
  pub fn char_dims(&self, code: u32, at: i64) -> Option<(i64, i64, i64)> {
    let info = self.char_info.get((code as usize).checked_sub(self.bc)?)?;
    let (wi, hi, di) = (
      info[0] as usize,
      (info[1] >> 4) as usize,
      (info[1] & 0xF) as usize,
    );
    if wi == 0 {
      return None;
    }
    let scaled = |table: &[[u8; 4]], i: usize| table.get(i).map_or(0, |fix| store_scaled(*fix, at));
    Some((
      scaled(&self.widths, wi),
      scaled(&self.heights, hi),
      scaled(&self.depths, di),
    ))
  }
}

/// tex.web §571-572 `store_scaled`: a fix_word times the size `z`, computed exactly as TeX does
/// (so line10's 10pt arrowhead is 10.0pt, not 10.00002pt).
fn store_scaled(fix: [u8; 4], size: i64) -> i64 {
  let (mut z, mut alpha) = (size, 16_i64);
  while z >= 0o40000000 {
    z /= 2;
    alpha += alpha;
  }
  let beta = 256 / alpha;
  alpha *= z;
  let [a, b, c, d] = fix.map(i64::from);
  let sw = (((d * z) / 0o400 + c * z) / 0o400 + b * z) / beta;
  match a {
    0 => sw,
    255 => sw - alpha,
    _ => 0, // not a valid fix_word (tex.web aborts the load)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn store_scaled_is_exact_for_the_design_size() {
    // 1.0 as a fix_word is 0x00100000; at 10pt it is exactly 655360sp.
    assert_eq!(store_scaled([0, 0x10, 0, 0], 655360), 655360);
    // -0.5 is 0xFFF80000.
    assert_eq!(store_scaled([255, 0xF8, 0, 0], 655360), -327680);
  }
}
