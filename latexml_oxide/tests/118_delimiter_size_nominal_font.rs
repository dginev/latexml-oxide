//! Sized math delimiters scale to the document's NOMINAL_FONT_SIZE.
//!
//! Perl keeps the size SYMBOLIC in the font spec (`font => { size => 'big' }`)
//! and rationalizes it inside `Font::new` (`Common/Font.pm` L266), so
//! `rationalizeFontSize` multiplies by `DEFSIZE()` — which reads
//! `NOMINAL_FONT_SIZE` from state — at font-construction time.
//!
//! Rust's `Font::size` is an `Option<f64>`, so the symbolic name cannot reach
//! the struct and the multiplication happens at the binding instead. Hardcoding
//! the product (`size => 12.0`, i.e. `1.2 * 10`) baked in the DEFAULT nominal
//! size, so `a0poster` — which sets it to 25 in both engines — got `\big(` at
//! 12/25 = 48%, i.e. *smaller* than body text, instead of 120%.
//!
//! Ground truth is pdflatex, not just Perl: it renders `\big(` visibly larger
//! than an adjacent plain `(`. Perl agrees at 120%/160%.
mod cluster;
use cluster::convert_expecting_errors;

fn sizes(xml: &str) -> Vec<String> {
  let mut v: Vec<String> = Vec::new();
  for seg in xml.split("fontsize=\"").skip(1) {
    if let Some(end) = seg.find('"') {
      v.push(seg[..end].to_string());
    }
  }
  v.sort();
  v.dedup();
  v
}

#[test]
fn sized_delimiters_scale_to_the_nominal_font_size() {
  // NOTE the expected-error count. `a0poster` currently emits 4
  // `Error:expected:<variable>` in Rust and ZERO in Perl 0.8.8 on this exact
  // input — a Rust-only defect in the class binding, unrelated to delimiter
  // sizing and tracked separately. It is pinned rather than tolerated: when
  // that bug is fixed this assertion fails, which is the intended prompt to
  // drop the count back to 0. Do not "fix" this by loosening the check.
  let xml = convert_expecting_errors(
    "tests/cluster_regressions/delimiter_size_nominal_font.tex",
    4,
  );
  let got = sizes(&xml);
  for want in ["120%", "160%"] {
    assert!(
      got.contains(&want.to_string()),
      "expected a {want} delimiter under a0poster (NOMINAL_FONT_SIZE=25), got {got:?} — \
       the \\big family is using a hardcoded absolute size again, so the \
       delimiters are scaled against the wrong body size"
    );
  }
  // The pre-fix answers. Named so a regression is unmistakable.
  for bad in ["48%", "64%"] {
    assert!(
      !got.contains(&bad.to_string()),
      "delimiter rendered at {bad} — that is 1.2*10 (or 1.6*10) measured \
       against a 25pt body, i.e. the hardcoded-DEFSIZE regression"
    );
  }
}
