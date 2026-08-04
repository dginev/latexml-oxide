//! End-to-end characterization of `\includegraphics` box sizing, one row per
//! source format.
//!
//! **This pins behaviour, not correctness**, and it is the durable half of the
//! image-sizing characterization suite: it names no internal function, so it
//! survives a refactor of the sizing pipeline untouched and will report exactly
//! which format's typeset size moved.
//!
//! The same 200x100 figure is saved as PNG, EPS, PDF and SVG and included with
//! no graphicx options. Four containers, four answers, because the resolution
//! used to turn a source measurement into a TeX dimension is chosen per format
//! rather than once:
//!
//! | source              | pdflatex   | Perl LaTeXML | here       | implied dpi |
//! |---------------------|------------|--------------|------------|-------------|
//! | PNG 200x100 px      | 200.7495pt | 144.54pt     | 144.54pt   | 100         |
//! | EPS BBox 200x100 bp | (n/a)      | (no sizer)   | 144.54pt   | 100, on bp  |
//! | PDF 200x100 bp      | 200.7495pt | (no sizer)   | 200.75pt   | 72          |
//! | SVG viewBox 200x100 | (n/a)      | (no sizer)   | 150.5625pt | 96          |
//!
//! The pdflatex and Perl columns were measured on the same fixtures on
//! 2026-08-04 (Perl 0.8.8 lacks Image::Magick on this host, so it can only size
//! what `Image::Size` reads — hence "no sizer" for EPS/PDF/SVG; the PNG row is
//! exact parity with us). Perl's own convention is the 100 dpi one:
//! `Util/Image.pm:37 our $DPI = 100`, with the box derived as `w * 72.27 / $DPI`
//! (L271) — which is why an unscaled raster typesets 28% smaller than pdflatex
//! makes it.
//!
//! The PDF-with-object-streams row is not a contrived case: `%PDF-1.5` and
//! later put the page tree in a compressed object stream by default, which our
//! byte-level reader cannot see, and the figure then reserves no space at all.

use std::{path::Path, process::Command};

/// A PNG header with the given IHDR dimensions — signature, length, `IHDR`,
/// width, height, then the bit-depth/colour bytes. No CRC and no image data:
/// every reader in the workspace takes the dimensions from this prefix.
fn png_header(w: u32, h: u32) -> Vec<u8> {
  let mut v = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
  v.extend_from_slice(&13u32.to_be_bytes());
  v.extend_from_slice(b"IHDR");
  v.extend_from_slice(&w.to_be_bytes());
  v.extend_from_slice(&h.to_be_bytes());
  v.extend_from_slice(&[0x08, 0x02, 0x00, 0x00, 0x00]);
  v.extend_from_slice(&[0u8; 16]);
  v
}

/// Convert `doc` and return the body text, with the `\the\wd`/`\the\ht` probes
/// left in place for the caller to match on.
fn sizes(probes: &[(&str, &str)]) -> String {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let work = tempfile::tempdir().expect("tempdir");
  let dir = work.path();

  std::fs::write(dir.join("fig.png"), png_header(200, 100)).unwrap();
  std::fs::write(
    dir.join("fig.eps"),
    "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 200 100\n%%EndComments\n",
  )
  .unwrap();
  std::fs::write(
    dir.join("fig.pdf"),
    "%PDF-1.4\n1 0 obj\n<< /Type /Page /MediaBox [0 0 200 100] >>\nendobj\n",
  )
  .unwrap();
  // A page tree hidden in an object stream, as pdflatex emits by default for
  // PDF 1.5+. Nothing here is readable as plaintext `/MediaBox`.
  std::fs::write(
    dir.join("objstm.pdf"),
    "%PDF-1.5\n1 0 obj\n<< /Type /ObjStm /N 8 /First 52 /Filter /FlateDecode >>\nstream\n\
     (compressed payload; no plaintext page box)\nendstream\nendobj\n",
  )
  .unwrap();
  std::fs::write(
    dir.join("fig.svg"),
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100"><rect/></svg>"#,
  )
  .unwrap();

  let mut tex = String::from(
    "\\documentclass{article}\n\\usepackage{graphicx}\n\
     \\newcommand\\probe[3]{\\par\\noindent PROBE #3 \
     \\setbox0\\hbox{\\includegraphics[#2]{#1}}\\the\\wd0\\ \\the\\ht0\\ END\\par}\n\
     \\begin{document}\n",
  );
  for (i, (file, opts)) in probes.iter().enumerate() {
    tex.push_str(&format!("\\probe{{{file}}}{{{opts}}}{{{i}}}\n"));
  }
  tex.push_str("\\end{document}\n");
  std::fs::write(dir.join("p.tex"), tex).unwrap();

  let out = Command::new(bin)
    .args(["p.tex", "--dest", "p.xml", "--nocomments"])
    .current_dir(dir)
    .output()
    .expect("spawn latexml_oxide");
  std::fs::read_to_string(dir.join("p.xml")).unwrap_or_else(|e| {
    let stderr = String::from_utf8_lossy(&out.stderr).replace('\u{1b}', "");
    panic!("no output: {e}\n{stderr}");
  })
}

/// The probe text for row `i`, whitespace-collapsed.
fn row(xml: &str, i: usize) -> String {
  let marker = format!("PROBE {i} ");
  let start = xml
    .find(&marker)
    .unwrap_or_else(|| panic!("row {i} missing from:\n{xml}"))
    + marker.len();
  let rest = &xml[start..];
  let end = rest
    .find(" END")
    .unwrap_or_else(|| panic!("row {i} unterminated"));
  rest[..end].split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn natural_box_size_differs_per_source_format() {
  let xml = sizes(&[
    ("fig.png", ""),
    ("fig.eps", ""),
    ("fig.pdf", ""),
    ("fig.svg", ""),
    ("objstm.pdf", ""),
  ]);
  // 200 px read as 100 dpi.
  assert_eq!(row(&xml, 0), "144.54pt 72.27pt", "PNG");
  // 200 bp read as if it were 200 px, then as 100 dpi — the BoundingBox unit
  // is discarded on the way in.
  assert_eq!(row(&xml, 1), "144.54pt 72.27pt", "EPS");
  // 200 bp converted honestly: 200 * 72.27/72. Exact pdflatex parity.
  assert_eq!(row(&xml, 2), "200.75pt 100.375pt", "PDF");
  // 200 viewBox user units read as CSS px: 200 * 72.27/96.
  assert_eq!(row(&xml, 3), "150.5625pt 75.28125pt", "SVG");
  // Page box unreadable — the figure reserves nothing.
  assert_eq!(row(&xml, 4), "0.0pt 0.0pt", "PDF with object streams");
}

/// With an explicit `width=`, the two sizing algebras still disagree — the
/// raster branch quantizes the box to a whole device pixel at DPI 100
/// (100pt -> 99.6265bp -> 137.85 px -> ceil 138 -> 99.7326pt) while the page-box
/// branch stays in pt and returns the request unchanged. pdflatex: 100.0pt for
/// both. Perl agrees with us exactly on the PNG row (99.7326pt / 49.8663pt).
#[test]
fn an_explicit_width_is_quantized_on_the_raster_branch_only() {
  let xml = sizes(&[
    ("fig.png", "width=100pt"),
    ("fig.pdf", "width=100pt"),
    ("fig.svg", "width=100pt"),
    ("fig.png", "scale=0.5"),
    // `angle=` rotates the reserved box, as Perl has always done
    // (`Util/Image.pm` L238-242) and we did not until 2026-08-04. Perl on
    // these exact inputs: 72.27pt x 144.54pt, and 153.30782pt square at 45.
    ("fig.png", "angle=90"),
    ("fig.png", "angle=45"),
  ]);
  assert_eq!(row(&xml, 0), "99.7326pt 49.8663pt", "PNG width=100pt");
  assert_eq!(row(&xml, 1), "100.0pt 50.0pt", "PDF width=100pt");
  assert_eq!(row(&xml, 2), "100.0pt 50.0pt", "SVG width=100pt");
  assert_eq!(row(&xml, 3), "72.27pt 36.135pt", "PNG scale=0.5");
  assert_eq!(
    row(&xml, 4),
    "72.27pt 144.54pt",
    "PNG angle=90 — Perl parity"
  );
  assert_eq!(
    row(&xml, 5),
    "153.30782pt 153.30782pt",
    "PNG angle=45 — the rotated bounding box, Perl parity"
  );
}
