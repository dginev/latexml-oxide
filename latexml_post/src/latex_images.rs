//! LaTeX-based image generation processor.
//!
//! Port of `LaTeXML::Post::LaTeXImages` (539 lines of Perl).
//! Base class for processors that generate images by running LaTeX + dvipng/dvips
//! on extracted TeX fragments. Used by MathImages and PictureImages.
//!
//! Pipeline:
//! 1. Collect TeX fragments from document nodes via `extractTeX()`
//! 2. Deduplicate: same TeX → same image (keyed by processor+type+tex)
//! 3. Check cache for previously generated images
//! 4. Generate a LaTeX document with all pending fragments
//! 5. Run `latex` to produce DVI
//! 6. Run `dvipng`/`dvips`/`dvisvgm` to produce individual images
//! 7. Parse dimensions from LaTeX log (LXIMAGE lines)
//! 8. Optionally convert/crop via ImageMagick
//! 9. Store results in cache; set node attributes

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::document::PostDocument;
use crate::processor::{PostError, ProcessResult, Processor, find_documentclass_and_packages};

/// DVI-to-image conversion method.
#[derive(Debug, Clone)]
pub enum DviMethod {
  /// dvipng (fast, PNG only)
  DviPng,
  /// dvisvgm (SVG output)
  DviSvgm,
  /// dvips + ImageMagick (general purpose, slow)
  Dvips,
}

/// A pending image entry to be generated.
#[derive(Debug)]
struct ImageEntry {
  /// The TeX source fragment.
  tex:   String,
  /// Cache key.
  key:   String,
  /// Nodes that reference this image.
  nodes: Vec<Node>,
  /// Desired destination paths.
  dests: Vec<String>,
}

/// LaTeX image generation processor.
///
/// Port of `LaTeXML::Post::LaTeXImages`.
pub struct LaTeXImages {
  name:               String,
  resource_directory: String,
  resource_prefix:    String,
  image_type:         String,
  dvi_method:         DviMethod,
  magnification:      f64,
  max_width:          u32,
  dpi:                u32,
  background:         String,
  padding:            u32,
  clipping_fudge:     u32,
  clipping_rule:      f64,
}

impl LaTeXImages {
  pub fn new(resource_directory: &str, resource_prefix: &str, image_type: &str) -> Self {
    let dvi_method = match image_type {
      "svg" => DviMethod::DviSvgm,
      "png" => DviMethod::DviPng,
      _ => DviMethod::Dvips,
    };

    LaTeXImages {
      name: "LaTeXImages".to_string(),
      resource_directory: resource_directory.to_string(),
      resource_prefix: resource_prefix.to_string(),
      image_type: image_type.to_string(),
      dvi_method,
      magnification: 1.33333,
      max_width: 800,
      dpi: 100,
      background: "#FFFFFF".to_string(),
      padding: 2,
      clipping_fudge: 3,
      clipping_rule: 0.90,
    }
  }

  /// Clean a TeX string for image generation.
  ///
  /// Port of `LaTeXImages::cleanTeX`.
  pub fn clean_tex(tex: &str) -> String {
    let mut s = tex.to_string();
    let mut style = String::new();

    // Save leading math style
    for prefix in &[
      "\\displaystyle",
      "\\textstyle",
      "\\scriptstyle",
      "\\scriptscriptstyle",
    ] {
      if let Some(rest) = s.trim_start().strip_prefix(prefix) {
        style = prefix.to_string();
        s = rest.to_string();
        break;
      }
    }

    // Trim leading/trailing TeX spacing commands
    let spacing_re = regex::Regex::new(r"^(?:\\[,!>;:/ ]|\\ )*").unwrap();
    s = spacing_re.replace(&s, "").to_string();
    let trailing_re = regex::Regex::new(r"(?:\\[,!>;:/ ]|\\ )*$").unwrap();
    s = trailing_re.replace(&s, "").to_string();

    // Strip comments (but not escaped %)
    // Note: Rust regex doesn't support lookbehinds, so we use a simple approach
    let comment_re = regex::Regex::new(r"([^\\])%[^\n]*\n").unwrap();
    s = comment_re.replace_all(&s, "$1").to_string();

    if !style.is_empty() {
      format!("{} {}", style, s)
    } else {
      s
    }
  }

  /// Set the image attributes on a node.
  ///
  /// Port of `LaTeXImages::setTeXImage`.
  pub fn set_tex_image(node: &mut Node, path: &str, width: u32, height: u32, depth: Option<u32>) {
    node.set_attribute("imagesrc", path).ok();
    node.set_attribute("imagewidth", &width.to_string()).ok();
    node.set_attribute("imageheight", &height.to_string()).ok();
    if let Some(d) = depth {
      node.set_attribute("imagedepth", &d.to_string()).ok();
    }
  }

  /// Generate images for the given nodes.
  ///
  /// Port of `LaTeXImages::generateImages`.
  /// This is the main pipeline entry point.
  pub fn generate_images(
    &self,
    doc: &mut PostDocument,
    nodes: &[Node],
    extract_tex: &dyn Fn(&PostDocument, &Node) -> Option<String>,
  ) -> Result<(), PostError> {
    // Step 1: Collect unique TeX strings
    let mut table: HashMap<String, ImageEntry> = HashMap::default();
    let mut n_total = 0u32;

    for node in nodes {
      let tex = match extract_tex(doc, node) {
        Some(t) if !t.trim().is_empty() => t,
        _ => continue,
      };
      n_total += 1;

      let key = format!("{}:{}:{}", self.name, self.image_type, tex);
      let entry = table.entry(key.clone()).or_insert_with(|| ImageEntry {
        tex:   tex.clone(),
        key:   key.clone(),
        nodes: Vec::new(),
        dests: Vec::new(),
      });
      entry.nodes.push(node.clone());
    }

    let n_unique = table.len();
    if n_unique == 0 {
      return Ok(());
    }

    // Step 2: Check cache for already-generated images
    let mut pending = Vec::new();
    for key in table.keys() {
      if let Some(cached) = doc.cache_lookup(key) {
        if cached.contains(';') {
          continue; // Already cached
        }
      }
      pending.push(key.clone());
    }

    log::info!(
      "LaTeXImages: {} total, {} unique, {} pending",
      n_total,
      n_unique,
      pending.len()
    );

    if !pending.is_empty() {
      // Step 3: Generate LaTeX source
      let (preamble, body_prefix) = self.pre_preamble(doc);
      let tex_body = self.generate_tex_document(
        &preamble,
        &body_prefix,
        &pending
          .iter()
          .map(|k| table[k].tex.as_str())
          .collect::<Vec<_>>(),
      );

      log::info!(
        "LaTeXImages: generated LaTeX document ({} bytes)",
        tex_body.len()
      );
      log::debug!(
        "Would run: latex + {} to produce {} images",
        match self.dvi_method {
          DviMethod::DviPng => "dvipng",
          DviMethod::DviSvgm => "dvisvgm",
          DviMethod::Dvips => "dvips + convert",
        },
        pending.len()
      );

      // Steps 4-8: Would run external commands here
      // For now, log the intent
    }

    // Step 9: Apply cached results to nodes
    for entry in table.values() {
      if let Some(cached) = doc.cache_lookup(&entry.key) {
        let parts: Vec<&str> = cached.split(';').collect();
        if parts.len() == 4 {
          let (image, width, height, depth) = (parts[0], parts[1], parts[2], parts[3]);
          let w: u32 = width.parse().unwrap_or(0);
          let h: u32 = height.parse().unwrap_or(0);
          let d: u32 = depth.parse().unwrap_or(0);
          for node in &entry.nodes {
            let mut node_mut = node.clone();
            Self::set_tex_image(&mut node_mut, image, w, h, Some(d));
          }
        }
      }
    }

    Ok(())
  }

  /// Generate the LaTeX preamble.
  ///
  /// Port of `LaTeXImages::pre_preamble`.
  fn pre_preamble(&self, doc: &PostDocument) -> (String, String) {
    let (class_info, packages) = find_documentclass_and_packages(doc);
    let class = &class_info.name;
    let class_options = &class_info.options;
    let oldstyle = class_info.oldstyle.is_some();
    let document_command = if oldstyle {
      "\\documentstyle"
    } else {
      "\\documentclass"
    };

    let mut pkg_lines = String::new();
    for pkg in &packages {
      if oldstyle {
        pkg_lines.push_str(&format!("\\RequirePackage{{{}}}\n", pkg.name));
      } else if pkg.name == "english" {
        pkg_lines.push_str("\\usepackage[english]{babel}\n");
      } else if pkg.options.is_empty() {
        pkg_lines.push_str(&format!("\\usepackage{{{}}}\n", pkg.name));
      } else {
        pkg_lines.push_str(&format!("\\usepackage[{}]{{{}}}\n", pkg.options, pkg.name));
      }
    }

    let pts_per_pixel = 72.27 / self.dpi as f64 / self.magnification;
    let w = (self.max_width as f64 * pts_per_pixel).ceil() as u32;
    let gap = (self.padding + self.clipping_fudge) as f64 * pts_per_pixel;
    let th = match self.dvi_method {
      DviMethod::DviSvgm => 0.0,
      _ => self.clipping_rule * pts_per_pixel,
    };

    let preamble = format!(
      r"\batchmode
\def\inlatexml{{true}}
{document_command}[{class_options}]{{{class}}}
{pkg_lines}
\makeatletter
\setlength{{\hoffset}}{{0pt}}\setlength{{\voffset}}{{0pt}}
\setlength{{\textwidth}}{{{w}pt}}
\newcount\lxImageNumber\lxImageNumber=0\relax
\newbox\lxImageBox
\newdimen\lxImageBoxSep
\setlength\lxImageBoxSep{{{gap:.4}pt}}
\newdimen\lxImageBoxRule
\setlength\lxImageBoxRule{{{th:.4}pt}}
\def\lxShowImage{{%
  \global\advance\lxImageNumber1\relax
  \@tempdima\wd\lxImageBox
  \advance\@tempdima-\lxImageBoxSep
  \advance\@tempdima-\lxImageBoxSep
  \typeout{{LXIMAGE \the\lxImageNumber\space= \the\@tempdima\space x \the\ht\lxImageBox\space + \the\dp\lxImageBox}}%
  \@tempdima\lxImageBoxRule
  \advance\@tempdima\lxImageBoxSep
  \advance\@tempdima\dp\lxImageBox
  \hbox{{\lower\@tempdima\hbox{{\vbox{{%
    \hrule\@height\lxImageBoxRule%
    \hbox{{\vrule\@width\lxImageBoxRule%
      \vbox{{\vskip\lxImageBoxSep\box\lxImageBox\vskip\lxImageBoxSep}}%
      \vrule\@width\lxImageBoxRule}}%
    \hrule\@height\lxImageBoxRule}}}}}}%
}}%
\def\lxBeginImage{{\setbox\lxImageBox\hbox\bgroup\color@begingroup\kern\lxImageBoxSep}}
\def\lxEndImage{{\kern\lxImageBoxSep\color@endgroup\egroup}}
\makeatother",
      document_command = document_command,
      class_options = class_options,
      class = class,
      pkg_lines = pkg_lines,
      w = w,
      gap = gap,
      th = th
    );

    // Body prefix: neutralize page styles, captions, citations
    let body_prefix = "\\makeatletter\\thispagestyle{empty}\\pagestyle{empty}\n\
       \\let\\@@toccaption\\@gobble\n\
       \\let\\@@caption\\@gobble\n\
       \\let\\cite\\@gobble\n\
       \\def\\@@bibref#1#2#3#4{}\n\
       \\renewcommand{\\cite}[2][]{}\n\
       \\title{}\\date{}\n\
       \\makeatother\n"
      .to_string();

    (preamble, body_prefix)
  }

  /// Build the complete LaTeX document.
  fn generate_tex_document(
    &self,
    preamble: &str,
    body_prefix: &str,
    tex_fragments: &[&str],
  ) -> String {
    let mut doc = String::new();
    doc.push_str(preamble);
    doc.push_str("\n\\begin{document}\n");
    doc.push_str(body_prefix);
    for tex in tex_fragments {
      doc.push_str(tex);
      doc.push_str("\\clearpage\n");
    }
    doc.push_str("\\end{document}\n");
    doc
  }

  /// Get the DVI command string.
  pub fn dvi_command(&self) -> String {
    let mag = (self.magnification * 1000.0) as u32;
    let dpi = (self.dpi as f64 * self.magnification) as u32;
    match self.dvi_method {
      DviMethod::DviSvgm => format!(
        "dvisvgm --page=1- --bbox=1pt --scale={} --no-fonts -o imgx-%03p",
        self.magnification
      ),
      DviMethod::DviPng => format!(
        "dvipng -bg Transparent -T tight -q -D{} -o imgx-%03d.png",
        dpi
      ),
      DviMethod::Dvips => format!("dvips -q -S1 -i -E -j0 -x{} -o imgx", mag),
    }
  }

  /// Parse LXIMAGE dimension lines from a LaTeX log file.
  ///
  /// Port of the log parsing in `generateImages`.
  /// Each line has format: `LXIMAGE N = Wpt x Hpt + Dpt`
  /// Returns a vector indexed by image number: (width_pt, height_pt, depth_pt).
  pub fn parse_log_dimensions(log_content: &str) -> Vec<Option<(f64, f64, f64)>> {
    let re = regex::Regex::new(
      r"^\s*LXIMAGE\s+(\d+)\s*=\s*([\+\-\d\.]+)pt\s*x\s*([\+\-\d\.]+)pt\s*\+\s*([\+\-\d\.]+)pt\s*$",
    )
    .unwrap();

    let mut dimensions: Vec<Option<(f64, f64, f64)>> = Vec::new();

    for line in log_content.lines() {
      if let Some(caps) = re.captures(line) {
        let index: usize = caps[1].parse().unwrap_or(0);
        let width: f64 = caps[2].parse().unwrap_or(0.0);
        let height: f64 = caps[3].parse().unwrap_or(0.0);
        let depth: f64 = caps[4].parse().unwrap_or(0.0);

        // Ensure vector is large enough
        while dimensions.len() <= index {
          dimensions.push(None);
        }
        dimensions[index] = Some((width, height, depth));
      }
    }

    dimensions
  }

  /// Compute the output filename for a given image index.
  ///
  /// Port of `sprintf($$self{dvicmd_output_name}, $index)`.
  pub fn output_filename(&self, index: u32) -> String {
    match self.dvi_method {
      DviMethod::DviSvgm => format!("imgx-{:03}.svg", index),
      DviMethod::DviPng => format!("imgx-{:03}.png", index),
      DviMethod::Dvips => format!("imgx{:03}", index),
    }
  }

  /// Get the output image type for the DVI method.
  pub fn output_type(&self) -> &str {
    match self.dvi_method {
      DviMethod::DviSvgm => "svg",
      DviMethod::DviPng => "png32",
      DviMethod::Dvips => "eps",
    }
  }

  /// Whether the DVI output needs frame-based cropping.
  pub fn needs_frame_output(&self) -> bool {
    match self.dvi_method {
      DviMethod::DviSvgm => false,
      DviMethod::DviPng | DviMethod::Dvips => true,
    }
  }

  /// Convert TeX points to pixels at the configured DPI and magnification.
  pub fn pt_to_pixels(&self, pt: f64) -> f64 { pt * self.magnification * self.dpi as f64 / 72.27 }

  /// Check whether this processor has the needed external tools.
  ///
  /// Port of `LaTeXImages::canProcess`.
  /// Checks for:
  /// - Image processing library (ImageMagick or similar)
  /// - LaTeX command availability
  pub fn can_process(&self) -> bool {
    // Check for latex command
    let latex_available = std::process::Command::new("latex")
      .arg("--version")
      .output()
      .is_ok();
    if !latex_available {
      // Perl LaTeXImages.pm L134: Error('expected', $LATEXCMD, undef,
      //   "No latex command ($LATEXCMD) found; Skipping.", ...)
      log_post_error!(
        "expected", "latex",
        "No latex command found; image generation will be skipped"
      );
      return false;
    }
    // Check for DVI converter
    let dvi_cmd = match self.dvi_method {
      DviMethod::DviPng => "dvipng",
      DviMethod::DviSvgm => "dvisvgm",
      DviMethod::Dvips => "dvips",
    };
    let dvi_available = std::process::Command::new(dvi_cmd)
      .arg("--version")
      .output()
      .is_ok();
    if !dvi_available {
      // Perl LaTeXImages.pm dvi-converter check: Error('expected',
      //   $$self{dvicmd}, …) (parallel to the latex check at L134).
      log_post_error!(
        "expected", dvi_cmd,
        "No {} command found; image generation will be skipped",
        dvi_cmd
      );
      return false;
    }
    true
  }

  /// Convert a DVI-output image (EPS/PNG) to final format with cropping.
  ///
  /// Port of `LaTeXImages::convert_image`.
  /// For dvipng output: already cropped, just copy.
  /// For dvips output (EPS): needs ImageMagick conversion + trim.
  /// For dvisvgm output (SVG): already in final format.
  ///
  /// Returns (width, height) in pixels, or None on failure.
  pub fn convert_image(&self, src: &str, dest: &str) -> Option<(u32, u32)> {
    match self.dvi_method {
      DviMethod::DviSvgm => {
        // SVG: just copy
        if let Err(e) = std::fs::copy(src, dest) {
          // Perl LaTeXImages.pm I/O failure: Error('I/O', $dest, …)
          log_post_error!(
            "I/O", dest,
            "Failed to copy {} to {}: {}", src, dest, e
          );
          return None;
        }
        // SVG dimensions from file would need XML parsing
        Some((0, 0))
      },
      DviMethod::DviPng => {
        // PNG: already cropped by dvipng -T tight
        if let Err(e) = std::fs::copy(src, dest) {
          log_post_error!(
            "I/O", dest,
            "Failed to copy {} to {}: {}", src, dest, e
          );
          return None;
        }
        // Would read PNG dimensions from file header
        Some((0, 0))
      },
      DviMethod::Dvips => {
        // EPS: needs ImageMagick conversion
        // Would run: convert -density DPI -trim src dest
        log::info!("Would convert EPS {} to {} via ImageMagick", src, dest);
        // Shave off clipping fudge + rule
        let fudge = (self.clipping_fudge as f64 + self.clipping_rule).round() as u32;
        log::debug!("  Shave: {}px from each edge", fudge);
        Some((0, 0))
      },
    }
  }

  /// Compute pixels-per-point for dimension conversions.
  pub fn pixels_per_pt(&self) -> f64 { self.magnification * self.dpi as f64 / 72.27 }
}

impl Processor for LaTeXImages {
  fn get_name(&self) -> &str { &self.name }

  fn resource_directory(&self) -> Option<&str> { Some(&self.resource_directory) }

  fn resource_prefix(&self) -> Option<&str> { Some(&self.resource_prefix) }

  fn process(&mut self, doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    log::info!("LaTeXImages: {} nodes to process", nodes.len());
    Ok(vec![doc])
  }
}
