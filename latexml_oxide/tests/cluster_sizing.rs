//! Box / image / SVG measurement + sizing tests.
//!
//! Auto-consolidated test binary: each former file is an inline `mod`
//! below, body preserved verbatim, merged into one link unit for CI
//! economy. All members are subprocess- or few-conversion tests, so
//! co-locating them in one process stays far under the RSS fuse.

mod cluster;

mod pgfplots_units {
  //! pgfplots `bar shift` + `symbolic x coords` units-flag regression.
  //!
  //! Root cause (2110.14597, +12 Rust-only errors → 0): the native pgfmath
  //! parser (`pgfmath_code_tex.rs`) dropped the `\ifpgfmathunitsdeclared` flag
  //! when evaluating a user-declared 0-arg pgfmath function whose body itself
  //! parses a unit'd value. pgfplots' `\pgfplotbarwidth` resolves through the
  //! `pgfplotsbarwidthgeneric` pseudo-constant, whose body does
  //! `\pgfmathparse{<bar width>pt}` — that nested parse sets the global units
  //! flag, but the outer parse clobbered it back to false on exit, so
  //! `\pgfplots@bar@mathparse@` mis-classified the bar shift as a unitless
  //! coordinate and fed it through the symbolic x-coord trafo:
  //!   `Package pgfplots Error: ... \pgfplots@loc@TMPa has not been defined`.
  //!
  //! Conditional: needs the kernel dump (so expl3/pgf load cleanly, not the
  //! degraded raw path) AND pgf/pgfplots installed in the host TeX tree.
  use latexml::util::test::{convert_fixture, dump_available, error_count, kpse_has};

  #[test]
  fn pgfplots_bar_shift_symbolic_coords_units_flag() {
    if !dump_available() {
      eprintln!(
        "SKIP pgfplots_bar_shift_symbolic_coords_units_flag: no latex kernel dump \
         in resources/dumps/ (run tools/make_formats.sh)"
      );
      return;
    }
    if !kpse_has("pgfplots.sty") || !kpse_has("pgf.sty") {
      eprintln!(
        "SKIP pgfplots_bar_shift_symbolic_coords_units_flag: pgf/pgfplots not \
         installed in the host TeX tree"
      );
      return;
    }

    let r = convert_fixture("tests/cluster_regressions/pgfplots_symbolic_bar_units.tex");
    assert!(r.result.is_some(), "conversion produced no result");

    let n = error_count(&r.log);
    assert_eq!(
      n, 0,
      "expected 0 errors (Perl converts cleanly) but got {n} — the pgfmath \
       units-declared flag is being dropped through the pgfplotsbarwidthgeneric \
       pseudo-constant eval, mis-routing `bar shift` through the symbolic-coord \
       trafo (status_code={})",
      r.status_code
    );
  }
}

mod delimiter_size_nominal_font {
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
  use crate::cluster::convert_expecting_errors;

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
    // `a0poster` used to emit 4 `Error:expected:<variable>` here (its binding's
    // `\setlength { \paperwidth }{…}` with spaces — Perl skipped them, we did
    // not); the `Variable` reader now follows tex.web §1211 (batch 54f), so
    // the pinned count is 0. Keep it pinned, never tolerated.
    let xml = convert_expecting_errors(
      "tests/cluster_regressions/delimiter_size_nominal_font.tex",
      0,
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

  /// #683 (xworld21): a non-default `NOMINAL_FONT_SIZE` is persisted into the XML
  /// as a `<?latexml nominal-font-size="X"?>` processing instruction, so
  /// post-processing can size font-relative (`em`) external SVGs correctly (an
  /// `em` is `NOMINAL_FONT_SIZE`pt, not always 10pt). Perl does not emit this —
  /// `NOMINAL_FONT_SIZE` is digestion-only (`DEFSIZE`) — so this is beyond-Perl.
  /// The PI fires ONLY when the value differs from the 10pt default, so ordinary
  /// documents stay byte-identical. Emitting it also exercised (and fixed) an
  /// `insert_pi` bug: a PI added after the root already exists was queued into
  /// the once-drained `pending` list and silently lost.
  #[test]
  fn nominal_font_size_persisted_as_pi_only_when_non_default() {
    use crate::cluster::{convert_expecting_errors, convert_to_xml};
    // a0poster sets NOMINAL_FONT_SIZE=25 → the PI carries it (its former 4
    // `<variable>` errors are gone — see the sibling delimiter test).
    let a0 = convert_expecting_errors(
      "tests/cluster_regressions/delimiter_size_nominal_font.tex",
      0,
    );
    assert!(
      a0.contains("<?latexml nominal-font-size=\"25\"?>"),
      "a0poster (NOMINAL_FONT_SIZE=25) did not persist the nominal-font-size PI:\n{a0}"
    );
    // A default 10pt document must emit NO such PI (output stays byte-identical).
    let plain = convert_to_xml("tests/cluster_regressions/nominal_font_size_default.tex");
    assert!(
      !plain.contains("nominal-font-size"),
      "a default-size document must not emit a nominal-font-size PI:\n{plain}"
    );
  }
}

mod image_sizing_characterization {
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
  //! A fifth row covers a PDF whose page tree lives in a compressed object
  //! stream — the `%PDF-1.5` default, so most figures pdflatex produces. The
  //! byte-level reader could not see it and the figure reserved no space at all;
  //! since 2026-08-04 the stream is inflated and the row matches the plain PDF.
  //! Measured over 14 real PDFs in this repo: 5 failed that way before, all 14
  //! match `pdfinfo` now.

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

  /// A minimal `%PDF-1.5` whose only object is a Flate-compressed object stream
  /// carrying `payload` — the shape pdflatex has emitted by default since PDF 1.5.
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
    // A page tree inside a real Flate-compressed object stream, as pdflatex emits
    // by default for PDF 1.5+. The box appears nowhere in the plaintext bytes.
    std::fs::write(
      dir.join("objstm.pdf"),
      objstm_pdf(b"5 0 << /Type /Page /MediaBox [0 0 200 100] >>"),
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
    // Same 200 bp box, reachable only by inflating the object stream. Until
    // 2026-08-04 this row read "0.0pt 0.0pt" — the figure reserved nothing.
    assert_eq!(
      row(&xml, 4),
      "200.75pt 100.375pt",
      "PDF with object streams"
    );
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

  /// graphicx applies a rotation before or after scaling depending on whether the
  /// `angle` key precedes the sizing key in the source string, and Perl replicates
  /// it (`image_graphicx_parse` sets `$rotfirst` from the keys seen up to `angle`,
  /// L168). This is not a quirk to smooth over: pdflatex behaves the same way, so
  /// getting it wrong reserves visibly wrong space for every rotated-and-sized
  /// figure. All four rows were verified against Perl 0.8.8 on the same input;
  /// three match to the digit.
  #[test]
  fn rotation_order_depends_on_key_order_like_perl_and_pdflatex() {
    let xml = sizes(&[
      ("fig.png", "angle=90,width=100pt"),
      ("fig.png", "width=100pt,angle=90"),
      ("fig.png", "scale=0.5,angle=90"),
      ("fig.png", "angle=90,scale=0.5"),
    ]);
    // angle first: rotate 200x100 -> 100x200, then width=100pt scales it (aspect
    // preserved) -> ~100x200pt. Perl: 99.7326 x 199.4652, exact.
    assert_eq!(row(&xml, 0), "99.7326pt 199.4652pt", "angle then width");
    // width first: scale to 100pt (-> ~138x69 px), then rotate -> swapped.
    // Perl: 49.8663 x 99.7326, exact.
    assert_eq!(row(&xml, 1), "49.8663pt 99.7326pt", "width then angle");
    // Uniform scale commutes with rotation; both orders agree with Perl exactly.
    assert_eq!(row(&xml, 2), "36.135pt 72.27pt", "scale then angle");
    // angle then scale: rotate-then-scale. Width matches Perl (36.8577); the
    // height is the one intentional sub-pixel divergence. Perl's truncated pi
    // (3.1415926) leaves a ~2.7e-6 residual on the 200px edge that its ceil
    // rounds UP to 101 px (72.9927pt); Rust's real pi keeps it at 100 px
    // (72.27pt), the geometrically correct, pdflatex-consistent value.
    assert_eq!(
      row(&xml, 3),
      "36.8577pt 72.27pt",
      "angle then scale — real pi, not Perl's 72.9927"
    );
  }
}

mod geometry_svg_sizing {
  //! Regression test: the `geometry` package's page layout must size **measured
  //! SVG graphics** (tcolorbox / tikz / pgf pictures) so their aspect ratio
  //! matches the PDF, while leaving the **reflowable HTML flow** untouched
  //! (OXIDIZED_DESIGN #99).
  //!
  //! LaTeXML (Perl and, faithfully, Rust) ignores `geometry` for the HTML body —
  //! page geometry is meaningless for reflowable output. But a measured picture
  //! that reads `\linewidth` (e.g. `\begin{tcolorbox}[width=0.5\linewidth]`) is
  //! emitted as a *fixed-size* SVG, so ignoring geometry there makes the box
  //! `0.5 x 345pt` (the article default) instead of the `0.5 x 472pt` the PDF uses
  //! (letterpaper minus fairmeta's 2.5cm margins). The narrow interior over-wraps
  //! the content, doubling the box height and pushing text through the border
  //! (arXiv:2605.29955, Figure 1).
  //!
  //! The fix injects the geometry-computed text width into `\linewidth` **only
  //! inside SVG-producing pictures**, never into the main HTML flow. These tests
  //! pin the contract in-process (via `convert_fixture`, no spawned binary):
  //!   1. the tcolorbox SVG widens to the geometry `\linewidth` (was 345-based),
  //!   2. a sibling `\rule{0.5\linewidth}` stays at the 345-based class default
  //!      (proof the HTML flow is NOT rescaled),
  //!   3. the SVG foreignObject `--ltx-fo-*` em sizing is preserved,
  //!   4. a `width=\linewidth` box nested in a reduced-width minipage keeps the
  //!      reduced width (the guard compares the class-default column, not the live
  //!      minipage-reduced `\textwidth`).
  //!
  //! Conditional: needs the kernel dump (so tcolorbox's expl3 loads cleanly) AND
  //! tcolorbox installed in the host TeX tree; self-skips otherwise.

  use latexml::util::test::{convert_fixture, dump_available, kpse_has};

  /// Pull the first `viewBox="minx miny W H"` width (the 3rd number) out of the
  /// serialized markup — the tcolorbox root `<svg>` is the first picture.
  fn first_viewbox_width(html: &str) -> f64 {
    let at = html.find("viewBox=\"").expect("no viewBox in output");
    let vb = &html[at + "viewBox=\"".len()..];
    let vb = &vb[..vb.find('"').expect("unterminated viewBox")];
    vb.split_whitespace()
      .nth(2)
      .and_then(|s| s.parse::<f64>().ok())
      .expect("could not parse viewBox width")
  }

  /// Pull the first `--ltx-fo-width:<N>em` value out of the markup.
  fn fo_width_em(html: &str) -> f64 {
    let at = html
      .find("--ltx-fo-width:")
      .expect("no --ltx-fo-width in SVG output");
    let rest = &html[at + "--ltx-fo-width:".len()..];
    let end = rest.find("em").expect("--ltx-fo-width not in em");
    rest[..end]
      .parse::<f64>()
      .expect("could not parse --ltx-fo-width")
  }

  /// True when this environment can raw-load tcolorbox on the kernel dump.
  fn can_run() -> bool { dump_available() && kpse_has("tcolorbox.sty") }

  const TEX: &str = "literal:\\documentclass{article}\n\
    \\usepackage[left=2.5cm,right=2.5cm,top=2.5cm,bottom=2.5cm]{geometry}\n\
    \\usepackage[most]{tcolorbox}\n\
    \\begin{document}\n\
    \\noindent\\rule{0.5\\linewidth}{2pt}\n\
    \n\
    \\begin{tcolorbox}[width=0.5\\linewidth]\n\
    Some tcolorbox content here.\n\
    \\end{tcolorbox}\n\
    \\end{document}\n";

  #[test]
  fn geometry_sizes_svg_but_not_html_flow() {
    if !can_run() {
      eprintln!("skipping: needs kernel dump + tcolorbox.sty in the host TeX tree");
      return;
    }
    let r = convert_fixture(TEX);
    let html = r.result.expect("conversion produced no result");

    // (1) The tcolorbox SVG is sized from the geometry \linewidth (0.5 x 472pt ->
    // ~326.6px), NOT the 345pt article default (~238.7px). letterpaper (8.5in =
    // 614.295pt) minus left+right (2 x 2.5cm = 142.264pt) -> \textwidth 472.03pt;
    // 0.5\linewidth = 236.02pt, emitted in px (x DPI/72.27 = x100/72.27) as ~326.6.
    let vb_w = first_viewbox_width(&html);
    assert!(
      (315.0..=340.0).contains(&vb_w),
      "tcolorbox SVG viewBox width {vb_w} is not geometry-sized (~326.6); \
       a 345pt-default build emits ~238.7. geometry text width was not applied \
       to the SVG picture.\n{html}",
    );

    // (2) The HTML flow is untouched: a sibling \rule{0.5\linewidth} keeps the
    // 345-based class default (0.5 x 345 = 172.5pt). Geometry must NOT rescale
    // the reflowable HTML body.
    assert!(
      html.contains("width=\"172.5pt\""),
      "sibling \\rule{{0.5\\linewidth}} should stay at the 345-based default \
       172.5pt (HTML flow must not be rescaled by geometry).\n{html}",
    );

    // (3) SVG box-content emulation intact: the foreignObject still carries an
    // em-valued --ltx-fo-width, now widened by the geometry linewidth (~20.5em
    // vs the 345-default ~14.1em).
    let fo_em = fo_width_em(&html);
    assert!(
      (18.0..=22.0).contains(&fo_em),
      "SVG foreignObject --ltx-fo-width {fo_em}em is not geometry-widened \
       (~20.5em expected; 345-default ~14.1em). SVG content sizing regressed.\n{html}",
    );
  }

  // A `width=\linewidth` tcolorbox nested in a reduced-width minipage must keep
  // the REDUCED width, not be clobbered to the full geometry width. A minipage
  // sets \textwidth=\linewidth locally, so the SVG-scope guard must compare
  // against the class-default column width, not the live \textwidth.
  const TEX_NESTED: &str = "literal:\\documentclass{article}\n\
    \\usepackage[left=2.5cm,right=2.5cm]{geometry}\n\
    \\usepackage[most]{tcolorbox}\n\
    \\begin{document}\n\
    \\begin{minipage}{0.4\\linewidth}\n\
    \\begin{tcolorbox}[width=\\linewidth]x\\end{tcolorbox}\n\
    \\end{minipage}\n\
    \\end{document}\n";

  #[test]
  fn geometry_does_not_clobber_reduced_linewidth() {
    if !can_run() {
      eprintln!("skipping: needs kernel dump + tcolorbox.sty in the host TeX tree");
      return;
    }
    let r = convert_fixture(TEX_NESTED);
    let html = r.result.expect("conversion produced no result");

    // 0.4 x 345pt = 138pt -> ~190.9px. If the guard mis-fired against the live
    // (minipage-reduced) \textwidth it would jump to the full geometry width
    // (~653px). Must stay in the reduced band.
    let vb_w = first_viewbox_width(&html);
    assert!(
      (170.0..=215.0).contains(&vb_w),
      "nested tcolorbox width {vb_w} was clobbered by geometry; a reduced \
       \\linewidth (minipage) must be preserved (expected ~190.9px).\n{html}",
    );
  }
}

mod tcbline_nointerlineskip {
  //! Regression test: the height measurement of a `\tcbline`-segmented tcolorbox
  //! must not over-count the paragraph BEFORE the rule.
  //!
  //! `\tcbline` (tcolorbox's `tcbskins` segmentation, `\tcbline@`) brackets its
  //! dashed-rule picture with `{\parskip\z@\par\nointerlineskip}` on each side.
  //! The `\par` there fires INSIDE a `{...}` group. Rust digests a text-mode
  //! `{...}` into a localized box-list (a `List`), so the grouped `\par` repacks
  //! only that empty inner list — the outer paragraph's characters ("xxx") stayed
  //! LOOSE (unpacked). Perl's bare `{...}` flows flat, so its enclosing `\par`
  //! packs them; Perl never sees the loose run. `compute_boxes_size` then counted
  //! each loose glyph as its own vertical line, adding one `\baselineskip` per
  //! extra glyph: a `xxx\tcbline yyy` box measured 111.03 where Perl is 77.82.
  //! That inflated tcolorbox height and left an oversized gap after the dashed
  //! rule (arXiv:2605.29955 Figure 1, the Lean code card).
  //!
  //! Ground truth (box viewBox height, `xxx\tcbline yyy` @ width=100pt):
  //!   pdflatex 65.79 · Perl 77.82 · Rust (pre-fix) 111.03.
  //! The fix (`tex_box.rs` `{` primitive, "R2"): when a text-mode `{...}` group
  //! breaks a paragraph (its digested body resumed vertical mode via `\par`), the
  //! `{` primitive repacks the outer paragraph's loose run at DIGESTION — so the
  //! box-list matches Perl's flat-digestion structure (a packed paragraph `List`,
  //! not loose chars) and `compute_boxes_size` measures the pure Perl way. Result:
  //! 77.82, an exact Perl match. (Perl's own 12pt excess over pdflatex is a shared
  //! limitation, not this test's target.)
  //!
  //! The coalescing itself is OXIDIZED_DESIGN #100. NOTE: it does NOT address box
  //! content overflowing the drawn frame when the CLIENT browser substitutes a
  //! wider/taller monospace than cmtt10 — that is the TeX-realm-vs-SVG font
  //! impedance mismatch (shapes frozen at TeX-font metrics; only the
  //! foreignObject text reflows). That is #99's known residual / WISDOM #47.
  //!
  //! Conditional: needs the kernel dump + tcolorbox in the host TeX tree.

  use latexml::util::test::{convert_fixture, dump_available, kpse_has};

  fn first_box_viewbox_height(xml: &str) -> f64 {
    // The tcolorbox root <svg:svg> is the first picture; its viewBox is
    // "minx miny W H" — take H (the 4th number). Skip the nested tcbline
    // sub-picture by taking the FIRST svg:svg (the outer box).
    let at = xml.find("<svg:svg").expect("no <svg:svg> box in output");
    let tag = &xml[at..xml[at..].find('>').expect("unterminated svg:svg") + at];
    let vb = tag.split("viewBox=\"").nth(1).expect("no viewBox");
    let vb = &vb[..vb.find('"').unwrap()];
    vb.split_whitespace()
      .nth(3)
      .and_then(|s| s.parse::<f64>().ok())
      .expect("could not parse viewBox height")
  }

  const TEX: &str = "literal:\\documentclass{article}\n\
    \\usepackage[most]{tcolorbox}\n\
    \\begin{document}\n\
    \\begin{tcolorbox}[width=100pt]xxx\\tcbline yyy\\end{tcolorbox}\n\
    \\end{document}\n";

  #[test]
  fn tcbline_box_not_over_measured() {
    if !(dump_available() && kpse_has("tcolorbox.sty")) {
      eprintln!("skipping: needs kernel dump + tcolorbox.sty in the host TeX tree");
      return;
    }
    let r = convert_fixture(TEX);
    let xml = r.result.expect("conversion produced no result");

    let h = first_box_viewbox_height(&xml);
    // pdflatex 65.79 / Perl 77.82 ground truth; the pre-fix Rust value is 111.03.
    // The paragraph before the rule must pack to one line, not one-per-glyph.
    assert!(
      h < 90.0,
      "\\tcbline box viewBox height {h} is over-measured (pdflatex 65.8 / Perl 77.8; \
       pre-fix Rust 111.0) — the paragraph before the rule was not repacked at \
       digestion, so its characters count as one vertical line each.\n{xml}",
    );
    // Sanity floor: it must still contain the rule + two lines (not collapse).
    assert!(
      h > 55.0,
      "\\tcbline box height {h} collapsed below the rule+content floor.\n{xml}",
    );
  }
}

mod parbox_nested_math {
  //! MathML for a `\parbox` (text box with nested `$...$` math) placed INSIDE
  //! display math — full pipeline, runs the MathML post-processor + XSLT.
  //!
  //! arXiv html_feedback #6847 (arXiv:2608.05024v1): three display formulas in
  //! Sections 3-4 that wrap a two-line `\parbox` of inline-math conditions inside
  //! a set-builder, e.g.
  //! `\Delta_k(S):=\sup\{|\det(BSA)|:\parbox{..}{$A\in\mathcal L(..),\|A\|\le1$,\\ $B\in..$}\}`.
  //! The parbox's inline math is a nested `ltx:Math` that the top-level MathML
  //! pass skips (`//ltx:Math[not(ancestor::ltx:Math)]`). It was then cloned
  //! VERBATIM into the output as raw `<ltx:XMath>` content-MathML, which the
  //! browser renders in operator-first document order — the reporter's garbled
  //! `∈AL(ℓ2k,X),≤‖A‖1`.
  //!
  //! SHARED FAILURE with Perl (`MathML.pm` L1063-1073 clones raw + warns
  //! `unexpected:nested-math`). The fix converts the nested math in place in
  //! `rebuild_text_subtree_with_doc` (and makes `convert_to_pmml` reentrancy-safe)
  //! — a surpass-Perl divergence (OXIDIZED_DESIGN). The in-process `Converter`
  //! (`06_cluster_regressions.rs`) stops at Core XML and the `convert_and_post`
  //! helper runs post with `pmml:false`, so this can only be checked end-to-end
  //! via the binary — like `cluster_xslt_split.rs`.

  use std::{path::Path, process::Command};

  fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(args)
      .current_dir(cwd)
      .output()
      .expect("spawn latexml_oxide")
  }

  const TEX: &str = "\\documentclass{article}\n\
     \\usepackage{amsmath,amssymb}\n\
     \\newcommand{\\norm}[1]{\\|#1\\|}\n\
     \\begin{document}\n\
     \\[\n\
       \\Delta_k(S) := \\sup\\Bigl\\{\\, |\\det(BSA)| :\n\
       \\parbox{42mm}{\n\
       $A \\in \\mathcal{L}(\\ell_2^{k}, X), \\norm{A}\\le1$,\\\\[1mm]\n\
       $B \\in \\mathcal{L}(Y, \\ell_2^{k}), \\norm{B} \\le 1$}\\,\n\
       \\Bigr\\},\n\
     \\]\n\
     \\end{document}\n";

  #[test]
  fn parbox_nested_math_converts_to_presentation_mathml() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("pb.tex"), TEX).unwrap();
    let out = run(work.path(), &["pb.tex", "--dest", "pb.html"]);
    assert!(
      out.status.success(),
      "conversion failed (status {:?}):\n{}",
      out.status.code(),
      String::from_utf8_lossy(&out.stderr)
    );
    let html = std::fs::read_to_string(work.path().join("pb.html")).expect("read pb.html");

    // Primary canary: no content-MathML may survive into the HTML. Before the fix
    // the parbox's nested math leaked as raw `<XMTok>`/`<XMApp>` — the browser
    // then renders those tokens in document (operator-first) order.
    assert!(
      !html.contains("XMTok") && !html.contains("XMApp") && !html.contains("XMDual"),
      "raw content-MathML leaked into the HTML — the \\parbox's nested math was not \
       converted (renders operator-first, garbled):\n{html}"
    );

    // Reading order inside the parbox: the operand `A` must precede the relation
    // `∈` (U+2208). Content-MathML is operator-first (`∈ A`); presentation MathML
    // is `A ∈`. Anchor at the parbox so we do not match the outer formula.
    let pb = &html[html
      .find("ltx_parbox")
      .expect("no ltx_parbox in output — the parbox itself was lost")..];
    let a_pos = pb
      .find("<mi>A</mi>")
      .expect("no presentation <mi>A</mi> in the parbox");
    let in_pos = pb.find('\u{2208}').expect("no ∈ operator in the parbox");
    assert!(
      a_pos < in_pos,
      "the ∈ operator precedes its operand A in the parbox — content-MathML \
       document order leaked into presentation:\n{pb}",
    );

    // The nested math must be a real inline `<math>` element. The parbox becomes
    // an HTML `<span class="ltx_inline-block">` (an HTML5 MathML-breakout tag), so
    // a bare `<mrow>` there would parse as HTML and render as FLAT TEXT — the
    // `<math>` re-enters MathML context so subscripts/superscripts render. `pb`
    // begins after the outer `<math>`, so any `<math` here is a nested one.
    assert!(
      pb.contains("<math"),
      "the parbox's nested math is not wrapped in a <math> element — a bare <mrow> \
       inside the inline-block's HTML renders as flat text, not math:\n{pb}",
    );
  }
}

mod widthof_in_base_dimension_reader {
  //! html_feedback#6869 "Overlapping numbers in tables". `\makebox[\widthof{X}]`
  //! / `\rule{\widthof{X}}{h}` read their length with the BASE gullet dimension
  //! reader, which used to collapse the calc `\widthof` no-op to 0 — so the
  //! boxed content had zero advance width and overlapped its neighbour. Witness
  //! arXiv 2603.23669: its siunitx S-column result tables box every bold value
  //! as `\makebox[\widthof{\tablenum{…}}][r]{…bold…}` and rendered the bold
  //! number on top of its `± unc`. Both LaTeXML engines produced `width="0.0pt"`
  //! (Perl even warns "Missing number (Dimension), treated as zero"); real
  //! pdflatex+calc gives the true width. OXIDIZED_DESIGN #115 makes
  //! `\widthof`&friends Dimension registers, resolvable in every dimension
  //! context (matching real LaTeX, surpassing Perl). Guards the FIX: the widths
  //! are nonzero and agree with the calc-routed reference `\setlength`.
  use crate::cluster::convert_to_xml;

  /// Parse the `pt` value of the first `width="…pt"` at/after `needle`.
  fn width_pt_after(xml: &str, needle: &str) -> f64 {
    let from = xml
      .find(needle)
      .unwrap_or_else(|| panic!("`{needle}` missing from:\n{xml}"));
    let rest = &xml[from..];
    let wstart = rest
      .find("width=\"")
      .unwrap_or_else(|| panic!("no width= after `{needle}` in:\n{xml}"))
      + "width=\"".len();
    let val = &rest[wstart..];
    let end = val.find('"').expect("unterminated width attr");
    val[..end]
      .trim_end_matches("pt")
      .parse::<f64>()
      .unwrap_or_else(|_| panic!("non-numeric width `{}`", &val[..end]))
  }

  #[test]
  fn widthof_resolves_in_makebox_and_rule_length_arguments() {
    let xml = convert_to_xml("tests/cluster_regressions/widthof_base_dimension_reader.tex");

    // Reference: the calc-routed \widthof path (\setlength) has always worked.
    let refw = {
      let i = xml.find("REF ").expect("REF marker missing") + 4;
      let rest = &xml[i..];
      let end = rest.find("pt ENDREF").expect("ENDREF marker missing");
      rest[..end].parse::<f64>().expect("non-numeric REF width")
    };
    assert!(
      refw > 1.0,
      "calc-routed \\widthof reference is ~zero: {refw}pt"
    );

    // The FIX: the base dimension reader now resolves \widthof too. Before
    // OXIDIZED_DESIGN #115 both of these were width="0.0pt" (overlap).
    let rule_w = width_pt_after(&xml, "<rule");
    let makebox_w = width_pt_after(&xml, "<text align=\"right\"");
    assert!(
      rule_w > 1.0,
      "\\rule{{\\widthof{{…}}}} width collapsed to {rule_w}pt"
    );
    assert!(
      makebox_w > 1.0,
      "\\makebox[\\widthof{{…}}] width collapsed to {makebox_w}pt (overlap bug)"
    );

    // All three measure \widthof{WWWW}, so the direct-reader widths must agree
    // with the calc-routed reference (attribute rounding: allow 0.5pt slack).
    assert!(
      (rule_w - refw).abs() < 0.5 && (makebox_w - refw).abs() < 0.5,
      "direct-reader \\widthof disagrees with calc reference: \
       rule={rule_w}pt makebox={makebox_w}pt ref={refw}pt"
    );
  }
}

mod wrapfig_inner_linewidth {
  //! A `wrapfigure`'s inner `\linewidth` must be the declared wrap width, not the
  //! full page. Witness arXiv 2603.23669 Fig. 3: a `wrapfigure{0.296\textwidth}`
  //! whose body was `\includegraphics[width=\linewidth]` rendered the image at
  //! the full page width (479px) inside a 29%-wide right float, so it overran and
  //! collided with the wrapped text. Real LaTeX wrapfig sets `\hsize`/`\linewidth`
  //! to the wrap width; Perl discards the width arg (SHARED-FAILURE). The fix
  //! lives in `wrapfig_sty.rs::set_wrap_width` (OXIDIZED_DESIGN #29).
  use crate::cluster::convert_to_xml;

  /// Widths (pt) of every `<rule …>` in the core XML, in document order.
  fn rule_widths(xml: &str) -> Vec<f64> {
    xml
      .match_indices("<rule")
      .filter_map(|(i, _)| {
        let rest = &xml[i..];
        let ws = rest.find("width=\"")? + "width=\"".len();
        let val = &rest[ws..];
        val[..val.find('"')?]
          .trim_end_matches("pt")
          .parse::<f64>()
          .ok()
      })
      .collect()
  }

  #[test]
  fn wrapfigure_reduces_inner_linewidth_to_wrap_width() {
    // One conversion, two `\rule{\linewidth}`s: a full-width control and one
    // inside a 0.3\textwidth wrapfigure (fixture wrapfig_inner_linewidth.tex).
    let x = convert_to_xml("tests/cluster_regressions/wrapfig_inner_linewidth.tex");
    let widths = rule_widths(&x);
    assert!(
      widths.len() >= 2,
      "expected >=2 <rule> widths, got {widths:?}:\n{x}"
    );
    let full = widths.iter().cloned().fold(0.0_f64, f64::max);
    let inner = widths.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(
      inner > 1.0 && inner < full * 0.5,
      "wrapfigure did not reduce inner \\linewidth: inner={inner}pt full={full}pt"
    );
  }
}

/// html_feedback#6632 (arXiv:2605.03143): code in a figure/minipage lost its
/// indentation and newlines. When a `lstlisting` (or a `minted` block, which
/// routes through the same listings display) is the SOLE content of a `minipage`,
/// `insert_block` (`base_utilities.rs`) absorbs the minipage onto the single child
/// and set the child's attributes — but `setAttribute("class", …)` OVERWROTE the
/// child's `ltx_lstlisting` with `ltx_minipage`, so the whitespace-preserving CSS
/// keyed on `.ltx_lstlisting` stopped applying (leading-space indentation
/// collapsed). Fix: the `class` key MERGES (`add_class`) like LaTeXML's own
/// `addClass`, so the listing keeps `ltx_lstlisting` AND gains `ltx_minipage`.
///
/// SURPASS-PERL (OXIDIZED_DESIGN #125): Perl's `insertBlock` (`TeX_Box.pool.ltxml`
/// L492) uses `setAttribute(class => …)`, which overwrites — verified same-host,
/// Perl 0.8.8 also emits `<listing class="ltx_minipage">` (SHARED-FAILURE). The
/// merged class matches the PDF's rendered, indented code.
mod listing_in_minipage_keeps_class {
  use crate::cluster::convert_to_xml;

  /// The `class` attribute of the `<listing>` element whose base64 `data` starts
  /// with `data_prefix` (distinguishes the boxed copy from the control — same
  /// source, same data).
  fn listing_class(xml: &str, which: usize) -> String {
    let opens: Vec<&str> = xml
      .match_indices("<listing ")
      .map(|(i, _)| {
        let tag = &xml[i..];
        &tag[..tag.find('>').unwrap_or(tag.len())]
      })
      .collect();
    let tag = opens
      .get(which)
      .unwrap_or_else(|| panic!("fewer than {} <listing> elements:\n{xml}", which + 1));
    let ci = tag
      .find("class=\"")
      .unwrap_or_else(|| panic!("no class on <listing>: {tag}"))
      + "class=\"".len();
    tag[ci..][..tag[ci..].find('"').unwrap()].to_string()
  }

  #[test]
  fn listing_sole_content_of_minipage_keeps_lstlisting_class() {
    let xml = convert_to_xml("tests/cluster_regressions/listing_in_minipage_keeps_class.tex");

    // Control: the un-boxed listing has always carried its own class.
    assert_eq!(
      listing_class(&xml, 0),
      "ltx_lstlisting",
      "control listing lost its class:\n{xml}"
    );

    // The FIX: the minipage-absorbed listing keeps `ltx_lstlisting` (for the
    // whitespace CSS) and GAINS `ltx_minipage` (for the sizing) — not one or the
    // other. `add_class` sorts, so the merged value is `ltx_lstlisting ltx_minipage`.
    assert_eq!(
      listing_class(&xml, 1),
      "ltx_lstlisting ltx_minipage",
      "listing inside a minipage must keep ltx_lstlisting AND gain ltx_minipage:\n{xml}"
    );
  }
}
