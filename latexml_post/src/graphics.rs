//! Graphics postprocessing.
//!
//! Port of `LaTeXML::Post::Graphics`.
//! Finds `<ltx:graphics>` elements without `imagesrc`, locates the source
//! graphic file, applies transformations (scaling, cropping, format conversion),
//! and sets the `imagesrc`, `imagewidth`, `imageheight` attributes.

use libxml::tree::Node;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::document::PostDocument;
use crate::processor::{ProcessResult, Processor};

/// Properties for a graphics file type.
#[derive(Debug, Clone)]
pub struct TypeProperties {
  pub destination_type: Option<String>,
  pub transparent: bool,
  pub prescale: bool,
  pub ncolors: Option<String>,
  pub quality: Option<u32>,
  pub unit: String,
  pub raster: Option<bool>,
  pub autocrop: bool,
  pub desirability: u32,
}

impl Default for TypeProperties {
  fn default() -> Self {
    TypeProperties {
      destination_type: None,
      transparent: false,
      prescale: false,
      ncolors: None,
      quality: None,
      unit: "pixel".to_string(),
      raster: None,
      autocrop: false,
      desirability: 0,
    }
  }
}

/// Graphics post-processor.
///
/// Port of `LaTeXML::Post::Graphics`.
pub struct Graphics {
  name: String,
  dpi: Option<u32>,
  magnify: f64,
  zoomout: f64,
  trivial_scaling: bool,
  graphics_types: Vec<String>,
  type_properties: HashMap<String, TypeProperties>,
  background: String,
}

impl Graphics {
  pub fn new(dpi: Option<u32>, trivial_scaling: bool) -> Self {
    let mut type_properties = HashMap::new();

    // Default type properties matching Perl
    for ext in &["ai", "pdf", "ps", "eps"] {
      type_properties.insert(
        ext.to_string(),
        TypeProperties {
          destination_type: Some("png".to_string()),
          transparent: true,
          prescale: true,
          ncolors: Some("400%".to_string()),
          quality: Some(90),
          unit: "point".to_string(),
          ..Default::default()
        },
      );
    }
    for ext in &["jpg", "jpeg"] {
      type_properties.insert(
        ext.to_string(),
        TypeProperties {
          destination_type: Some(ext.to_string()),
          ncolors: Some("400%".to_string()),
          unit: "pixel".to_string(),
          ..Default::default()
        },
      );
    }
    type_properties.insert(
      "gif".to_string(),
      TypeProperties {
        destination_type: Some("gif".to_string()),
        transparent: true,
        ncolors: Some("400%".to_string()),
        unit: "pixel".to_string(),
        ..Default::default()
      },
    );
    type_properties.insert(
      "png".to_string(),
      TypeProperties {
        destination_type: Some("png".to_string()),
        transparent: true,
        ncolors: Some("400%".to_string()),
        unit: "pixel".to_string(),
        ..Default::default()
      },
    );
    type_properties.insert(
      "svg".to_string(),
      TypeProperties {
        destination_type: Some("svg".to_string()),
        raster: Some(false),
        desirability: 11,
        ..Default::default()
      },
    );

    Graphics {
      name: "Graphics".to_string(),
      dpi,
      magnify: 1.0,
      zoomout: 1.0,
      trivial_scaling,
      graphics_types: vec![
        "svg", "png", "gif", "jpg", "jpeg", "eps", "ps", "postscript", "ai", "pdf",
      ]
      .into_iter()
      .map(String::from)
      .collect(),
      type_properties,
      background: "#FFFFFF".to_string(),
    }
  }

  /// Find the graphics source file for a node.
  ///
  /// Port of `Graphics::findGraphicFile`.
  fn find_graphic_file(
    &self,
    _doc: &PostDocument,
    node: &Node,
    search_paths: &[String],
  ) -> Option<String> {
    let source = node.get_attribute("graphic")?;

    // Check candidates attribute first (comma-separated list of found files)
    // Perl: findGraphicFile checks each candidate path, resolving relative to search paths.
    if let Some(candidates) = node.get_attribute("candidates") {
      // Pick the best candidate by desirability
      let mut best: Option<(String, i32)> = None;
      for path in candidates.split(',') {
        let path = path.trim();
        if path.is_empty() {
          continue;
        }
        // Try the path directly, then in each search directory
        let resolved = if Path::new(path).exists() {
          Some(path.to_string())
        } else {
          search_paths.iter().find_map(|sp| {
            let candidate = format!("{}/{}", sp, path);
            if Path::new(&candidate).exists() { Some(candidate) } else { None }
          })
        };
        if let Some(resolved_path) = resolved {
          let ext = Path::new(&resolved_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
          let props = self.type_properties.get(&ext);
          let d = props.map(|p| p.desirability as i32).unwrap_or(0);
          let is_same_type = props
            .and_then(|p| p.destination_type.as_ref())
            .map(|dt| dt == &ext)
            .unwrap_or(false);
          let desirability = if is_same_type { 10 } else { d };
          if best.as_ref().is_none_or(|(_, bd)| desirability > *bd) {
            best = Some((resolved_path, desirability));
          }
        }
      }
      if let Some((path, _)) = best {
        return Some(path);
      }
    }

    // Search for the file in search paths
    let path = Path::new(&source);
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or(&source);
    let dir = path.parent().and_then(|p| p.to_str()).unwrap_or("");

    let file_base = if dir.is_empty() {
      name.to_string()
    } else {
      format!("{}/{}", dir, name)
    };

    // Try each type in search paths
    let mut best_desirability: i32 = -1;
    let mut best_path: Option<String> = None;

    let types: Vec<String> = self
      .graphics_types
      .iter()
      .flat_map(|t| vec![t.clone(), t.to_uppercase()])
      .collect();

    for search_path in search_paths {
      // Try without extension first (source might already have one)
      let candidate = if search_path.is_empty() {
        source.clone()
      } else {
        format!("{}/{}", search_path, source)
      };
      if Path::new(&candidate).exists() {
        let ext = Path::new(&candidate)
          .extension()
          .and_then(|e| e.to_str())
          .unwrap_or("")
          .to_lowercase();
        let props = self.type_properties.get(&ext);
        let d = props.map(|p| p.desirability as i32).unwrap_or(5);
        if d > best_desirability {
          best_desirability = d;
          best_path = Some(candidate);
        }
      }
      // Try each extension
      for ext in &types {
        let candidate = format!("{}/{}.{}", search_path, file_base, ext);
        if Path::new(&candidate).exists() {
          let props = self.type_properties.get(&ext.to_lowercase());
          let d = props.map(|p| p.desirability as i32).unwrap_or(0);
          let is_same_type = props
            .and_then(|p| p.destination_type.as_ref())
            .map(|dt| dt == &ext.to_lowercase())
            .unwrap_or(false);
          let desirability = if is_same_type { 10 } else { d };
          if desirability > best_desirability {
            best_desirability = desirability;
            best_path = Some(candidate);
          }
        }
      }
    }

    best_path
  }

  /// Set the image source attributes on a graphics node.
  ///
  /// Port of `Graphics::setGraphicSrc`.
  fn set_graphic_src(node: &mut Node, src: &str, width: Option<u32>, height: Option<u32>) {
    node.set_attribute("imagesrc", src).ok();
    // HTML width/height are in pixels (unitless)
    if let Some(w) = width {
      node.set_attribute("imagewidth", &w.to_string()).ok();
    }
    if let Some(h) = height {
      node.set_attribute("imageheight", &h.to_string()).ok();
    }
    // Set aspect ratio class
    if let (Some(w), Some(h)) = (width, height) {
      let class = if w as f64 > 1.24 * h as f64 {
        "ltx_img_landscape"
      } else if h as f64 > 1.24 * w as f64 {
        "ltx_img_portrait"
      } else {
        "ltx_img_square"
      };
      let existing = node.get_attribute("class").unwrap_or_default();
      let new_class = if existing.is_empty() {
        class.to_string()
      } else {
        format!("{} {}", existing, class)
      };
      node.set_attribute("class", &new_class).ok();
    }
  }

  /// Find graphicspath from processing instructions.
  fn find_graphics_paths(&self, doc: &PostDocument) -> Vec<String> {
    let re = regex::Regex::new(r#"^\s*graphicspath\s*=\s*[\"'](.*?)[\"']\s*$"#).unwrap();
    let mut paths = Vec::new();
    for pi in doc.findnodes(".//processing-instruction('latexml')") {
      let text = pi.get_content();
      if let Some(cap) = re.captures(&text) {
        paths.push(cap[1].to_string());
      }
    }
    paths
  }

  /// Read a named parameter from processing instructions.
  /// Port of Perl's `Graphics::getParameter`.
  /// Checks both direct PI (`<?latexml DPI="100"?>`) and
  /// latexml.sty package options (`<?latexml package="latexml" options="magnify=1.2"?>`).
  fn get_parameter(&self, doc: &PostDocument, param: &str) -> Option<f64> {
    let direct_re = regex::Regex::new(
      &format!(r#"^\s*{}\s*=\s*[\"']?([\d.]+)[\"']?\s*$"#, regex::escape(param))
    ).ok()?;
    let options_re = regex::Regex::new(
      r#"package\s*=\s*[\"']latexml[\"'].*options\s*=\s*[\"'](.*?)[\"']"#
    ).ok()?;
    let param_in_options_re = regex::Regex::new(
      &format!(r#"\b{}\s*=\s*([\d.]+)"#, regex::escape(param))
    ).ok()?;

    for pi in doc.findnodes(".//processing-instruction('latexml')") {
      let text = pi.get_content();
      if let Some(cap) = direct_re.captures(&text) {
        return cap[1].parse().ok();
      }
      if let Some(cap) = options_re.captures(&text) {
        if let Some(inner) = param_in_options_re.captures(&cap[1]) {
          return inner[1].parse().ok();
        }
      }
    }
    None
  }

  /// Read image dimensions using imagesize crate.
  /// Returns (width, height) in pixels.
  fn read_image_dimensions(path: &str) -> Option<(u32, u32)> {
    match imagesize::size(path) {
      Ok(dim) => Some((dim.width as u32, dim.height as u32)),
      Err(_) => None,
    }
  }

  /// Parse graphicx options and apply transforms to image dimensions.
  /// Port of Perl's `getTransform` + `image_graphicx_trivial`.
  ///
  /// Handles: scale=N, width=Npt, height=Npt, keepaspectratio
  fn apply_graphicx_transforms(
    raw_w: u32, raw_h: u32, options: &str, dpi: u32,
  ) -> (u32, u32) {
    let dppt = dpi as f64 / 72.27; // dots per point
    let mut w = raw_w as f64;
    let mut h = raw_h as f64;

    // Parse options as key=value pairs
    let mut scale: Option<f64> = None;
    let mut target_width: Option<f64> = None;
    let mut target_height: Option<f64> = None;
    let mut keep_aspect = false;

    for opt in options.split(',') {
      let opt = opt.trim();
      if let Some((key, val)) = opt.split_once('=') {
        let key = key.trim();
        let val = val.trim();
        match key {
          "scale" => { scale = val.parse::<f64>().ok(); },
          "width" => {
            // Parse dimension: "345.0pt" or "345pt" or bare number
            let val = val.trim_end_matches("pt").trim_end_matches("px");
            target_width = val.parse::<f64>().ok();
          },
          "height" => {
            let val = val.trim_end_matches("pt").trim_end_matches("px");
            target_height = val.parse::<f64>().ok();
          },
          "keepaspectratio" => {
            keep_aspect = val == "true" || val == "1" || val.is_empty();
          },
          _ => {},
        }
      } else if opt == "keepaspectratio" {
        keep_aspect = true;
      }
    }

    // Apply transforms (matching Perl's image_graphicx_trivial)
    if let Some(s) = scale {
      w *= s;
      h *= s;
    } else if target_width.is_some() || target_height.is_some() {
      let tw = target_width.unwrap_or(w / dppt);
      let th = target_height.unwrap_or(h / dppt);
      if keep_aspect {
        // Preserve aspect ratio: use the more constraining dimension
        let scale_w = tw / (w / dppt);
        let scale_h = th / (h / dppt);
        let s = scale_w.min(scale_h);
        w = (w / dppt * s * dppt).ceil();
        h = (h / dppt * s * dppt).ceil();
      } else {
        w = (tw * dppt).ceil();
        h = (th * dppt).ceil();
      }
    }

    (w.max(1.0) as u32, h.max(1.0) as u32)
  }

  /// Copy a source image to the destination directory, preserving relative paths.
  /// Returns the destination path (relative to dest_dir).
  fn copy_to_destination(
    source: &str,
    source_dir: &str,
    dest_dir: &str,
  ) -> Option<String> {
    // Compute relative path of source from source_dir
    let source_path = Path::new(source);
    let source_base = Path::new(source_dir);
    let rel_path = source_path
      .strip_prefix(source_base)
      .unwrap_or(source_path);

    // Build absolute destination path
    let abs_dest = PathBuf::from(dest_dir).join(rel_path);

    // Create parent directories if needed
    if let Some(parent) = abs_dest.parent() {
      std::fs::create_dir_all(parent).ok()?;
    }

    // Copy the file (skip if same path)
    let source_canonical = std::fs::canonicalize(source).ok();
    let dest_canonical = std::fs::canonicalize(&abs_dest).ok();
    if source_canonical != dest_canonical || dest_canonical.is_none() {
      std::fs::copy(source, &abs_dest).ok()?;
    }

    // Return relative path for imagesrc attribute
    Some(rel_path.to_string_lossy().to_string())
  }

  /// Extract `page=N` from graphicx options string.
  /// Returns 1-based page number (matching graphicx convention), or None.
  fn parse_page_option(options: &str) -> Option<u32> {
    for opt in options.split(',') {
      let opt = opt.trim();
      if let Some((key, val)) = opt.split_once('=') {
        if key.trim() == "page" {
          // Strip braces: page={2} → 2
          let val = val.trim().trim_matches('{').trim_matches('}');
          return val.parse::<u32>().ok();
        }
      }
    }
    None
  }

  /// Convert a graphics file using ImageMagick's `convert` command.
  /// Perl: image_graphicx_complex via Image::Magick / convert CLI.
  /// `page` is 1-based (graphicx convention); converted to 0-based for ImageMagick.
  fn convert_image(source: &str, dest: &str, _dpi: u32, page: Option<u32>) -> bool {
    // Build the source argument with optional page selector
    // Perl: image_read reads "$source[$page]" where $page = ($page // 1) - 1
    let source_arg = if let Some(p) = page {
      format!("{}[{}]", source, p.saturating_sub(1))
    } else {
      // No page specified: use [0] for PDFs to avoid converting all pages
      if source.to_lowercase().ends_with(".pdf") {
        format!("{}[0]", source)
      } else {
        source.to_string()
      }
    };
    // Shell out to convert (matching Perl's approach)
    // -define pdf:use-cropbox=true matches Perl's Image::Magick option (line 466)
    let result = std::process::Command::new("convert")
      .arg("-define")
      .arg("pdf:use-cropbox=true")
      .arg("-density")
      .arg("150")
      .arg(&source_arg)
      .arg(dest)
      .output();
    match result {
      Ok(output) => output.status.success(),
      Err(_) => false,
    }
  }
}

impl Processor for Graphics {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:graphics[not(@imagesrc)]")
  }

  fn process(&mut self, doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    let mut search_paths = self.find_graphics_paths(&doc);
    search_paths.extend(doc.get_search_paths().iter().cloned());
    // Also add source directory
    let source_dir = doc.get_source_directory().to_string();
    if !source_dir.is_empty() && !search_paths.contains(&source_dir) {
      search_paths.push(source_dir.clone());
    }
    // Add current directory as fallback
    if !search_paths.contains(&".".to_string()) {
      search_paths.push(".".to_string());
    }

    let dest_dir = doc.get_destination_directory().unwrap_or(".").to_string();
    // Read DPI/magnify/zoomout from processing instructions (set by latexml.sty)
    let dpi = self.get_parameter(&doc, "DPI")
      .map(|v| v as u32)
      .or(self.dpi)
      .unwrap_or(100);
    let magnify = self.get_parameter(&doc, "magnify").unwrap_or(self.magnify);
    let _zoomout = self.get_parameter(&doc, "zoomout").unwrap_or(self.zoomout);
    // Perl: effective DPI = DPI * magnify / zoomout (used for scale-to transforms)
    let effective_dpi = ((dpi as f64) * magnify / _zoomout) as u32;
    let n_to_process = nodes.len();

    // Counter for generating unique resource names (like Perl's generateResourcePathname)
    let mut resource_counter: u32 = 0;

    // Two-phase plan so the slow per-image `convert` subprocess and
    // `read_image_dimensions` calls can run in parallel without touching
    // the libxml DOM off-thread.
    //
    // Phase 1 (serial): read each node's attributes, resolve source path,
    // decide the conversion kind, and allocate resource-name counters.
    //   - `Plan::NotFound` — apply fallback on the main thread later
    //   - `Plan::Copy { .. }` — apply on the main thread later (cheap)
    //   - `Plan::Convert { .. }` — independent; run in parallel.
    // Phase 2 (parallel): run convert_image + read_image_dimensions for
    // `Plan::Convert` entries. Produces `JobResult`s keyed by node index.
    // Phase 3 (serial): apply DOM mutations on the main thread in original
    // node order so attribute writes happen on the libxml-owning thread.
    enum Plan {
      NotFound { idx: usize, graphic: String },
      Copy { idx: usize, source: String, options: String },
      Convert { idx: usize, source: String, options: String, page: Option<u32>,
                rel_dest: String, abs_dest_str: String },
    }
    struct ConvertOutcome {
      idx: usize,
      /// Path to write into `imagesrc`; `None` if both convert and copy-fallback failed.
      imagesrc: Option<String>,
      /// Raw (pre-transform) dimensions read from whichever file we ended up with.
      raw_dims: Option<(u32, u32)>,
      /// Options passed to transforms (captured once to avoid a DOM read off-thread).
      options: String,
    }

    let mut plans: Vec<Plan> = Vec::with_capacity(n_to_process);
    for (idx, node) in nodes.iter().enumerate() {
      let options = node.get_attribute("options").unwrap_or_default();
      let page = Self::parse_page_option(&options);
      let Some(source) = self.find_graphic_file(&doc, node, &search_paths) else {
        let graphic = node.get_attribute("graphic").unwrap_or_else(|| "none".to_string());
        plans.push(Plan::NotFound { idx, graphic });
        continue;
      };
      let src_ext = Path::new(&source)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
      let props = self.type_properties.get(&src_ext).cloned();
      let dest_type = props
        .as_ref()
        .and_then(|p| p.destination_type.as_ref())
        .cloned()
        .unwrap_or(src_ext.clone());
      let needs_conversion = dest_type != src_ext;
      let has_page = page.is_some();
      if needs_conversion || has_page {
        let dest_name = if has_page {
          resource_counter += 1;
          format!("x{}", resource_counter)
        } else {
          Path::new(&source)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image")
            .to_string()
        };
        let rel_dest = format!("{}.{}", dest_name, dest_type);
        let abs_dest = PathBuf::from(&dest_dir).join(&rel_dest);
        if let Some(parent) = abs_dest.parent() {
          std::fs::create_dir_all(parent).ok();
        }
        let abs_dest_str = abs_dest.to_string_lossy().to_string();
        plans.push(Plan::Convert { idx, source, options, page, rel_dest, abs_dest_str });
      } else {
        plans.push(Plan::Copy { idx, source, options });
      }
    }

    // Phase 2: parallel conversions. Bounded worker count to avoid
    // oversubscribing when many images are in flight. `convert` itself
    // is single-threaded per invocation, so the ceiling is useful CPU
    // parallelism — capped at a reasonable limit to avoid fork/memory
    // storms with many-image papers.
    let convert_count = plans.iter().filter(|p| matches!(p, Plan::Convert { .. })).count();
    let worker_cap = std::thread::available_parallelism()
      .map(|n| n.get())
      .unwrap_or(4)
      .min(8)
      .max(1);
    let n_workers = convert_count.min(worker_cap).max(1);
    let source_dir_ref = source_dir.as_str();
    let dest_dir_ref = dest_dir.as_str();
    let mut outcomes: Vec<ConvertOutcome> = Vec::with_capacity(convert_count);
    if convert_count > 0 {
      use std::sync::atomic::{AtomicUsize, Ordering};
      use std::sync::Mutex;
      let next = AtomicUsize::new(0);
      let out = Mutex::new(Vec::<ConvertOutcome>::with_capacity(convert_count));
      // Collect just the Convert entries into a fresh Vec so worker index
      // access is O(1).
      let jobs: Vec<&Plan> = plans.iter().filter(|p| matches!(p, Plan::Convert { .. })).collect();
      std::thread::scope(|s| {
        for _ in 0..n_workers {
          s.spawn(|| {
            loop {
              let i = next.fetch_add(1, Ordering::Relaxed);
              if i >= jobs.len() { break; }
              let Plan::Convert { idx, source, options, page, rel_dest, abs_dest_str } = jobs[i]
                else { unreachable!() };
              let outcome = if Self::convert_image(source, abs_dest_str, dpi, *page) {
                ConvertOutcome {
                  idx: *idx,
                  imagesrc: Some(rel_dest.clone()),
                  raw_dims: Self::read_image_dimensions(abs_dest_str),
                  options: options.clone(),
                }
              } else {
                log::warn!("Graphics: Failed to convert {} to {}", source, abs_dest_str);
                if let Some(rel) = Self::copy_to_destination(source, source_dir_ref, dest_dir_ref) {
                  ConvertOutcome {
                    idx: *idx,
                    imagesrc: Some(rel),
                    raw_dims: Self::read_image_dimensions(source),
                    options: options.clone(),
                  }
                } else {
                  ConvertOutcome { idx: *idx, imagesrc: None, raw_dims: None, options: options.clone() }
                }
              };
              out.lock().unwrap().push(outcome);
            }
          });
        }
      });
      outcomes = out.into_inner().unwrap();
      outcomes.sort_by_key(|o| o.idx);
    }
    let mut outcome_iter = outcomes.into_iter().peekable();

    // Phase 3: serial DOM mutations. Preserves original node order.
    let apply_transforms = |options: &str, raw_dims: Option<(u32, u32)>|
      -> (Option<u32>, Option<u32>) {
      match raw_dims {
        Some((w, h)) if !options.is_empty() => {
          let (tw, th) = Self::apply_graphicx_transforms(w, h, options, effective_dpi);
          (Some(tw), Some(th))
        },
        Some((w, h)) => (Some(w), Some(h)),
        None => (None, None),
      }
    };
    for plan in &plans {
      match plan {
        Plan::NotFound { idx, graphic } => {
          log::warn!("Graphics: No source found for {}", graphic);
          let mut node_mut = nodes[*idx].clone();
          node_mut.set_attribute("imagesrc", graphic).ok();
        },
        Plan::Copy { idx, source, options } => {
          let mut node_mut = nodes[*idx].clone();
          if let Some(rel) = Self::copy_to_destination(source, &source_dir, &dest_dir) {
            let (w, h) = apply_transforms(options, Self::read_image_dimensions(source));
            Self::set_graphic_src(&mut node_mut, &rel, w, h);
          } else {
            let rel_path = Path::new(source)
              .strip_prefix(&source_dir)
              .unwrap_or(Path::new(source));
            let rel_str = rel_path.to_string_lossy().to_string();
            let (w, h) = apply_transforms(options, Self::read_image_dimensions(source));
            Self::set_graphic_src(&mut node_mut, &rel_str, w, h);
          }
        },
        Plan::Convert { idx, .. } => {
          // Pull the matching outcome. Outcomes are sorted by idx and each
          // Convert plan has a unique idx, so peek-and-consume is correct.
          if let Some(out) = outcome_iter.peek() {
            if out.idx == *idx {
              let out = outcome_iter.next().unwrap();
              let mut node_mut = nodes[*idx].clone();
              if let Some(imagesrc) = out.imagesrc {
                let (w, h) = apply_transforms(&out.options, out.raw_dims);
                Self::set_graphic_src(&mut node_mut, &imagesrc, w, h);
              }
            }
          }
        },
      }
    }

    log::info!(
      "Graphics {} {} to process",
      doc.get_destination().unwrap_or("?"),
      n_to_process
    );
    Ok(vec![doc])
  }
}
