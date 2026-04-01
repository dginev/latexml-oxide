//! Graphics postprocessing.
//!
//! Port of `LaTeXML::Post::Graphics`.
//! Finds `<ltx:graphics>` elements without `imagesrc`, locates the source
//! graphic file, applies transformations (scaling, cropping, format conversion),
//! and sets the `imagesrc`, `imagewidth`, `imageheight` attributes.

use libxml::tree::Node;
use std::collections::HashMap;
use std::path::Path;

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

    // Check candidates attribute first
    if let Some(candidates) = node.get_attribute("candidates") {
      for path in candidates.split(',') {
        let path = path.trim();
        if Path::new(path).exists() {
          return Some(path.to_string());
        }
      }
    }

    // Search for the file
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
      // Try without extension
      let candidate = format!("{}/{}", search_path, file_base);
      if Path::new(&candidate).exists() {
        return Some(candidate);
      }
      // Try each extension
      for ext in &types {
        let candidate = format!("{}/{}.{}", search_path, file_base, ext);
        if Path::new(&candidate).exists() {
          let props = self.type_properties.get(&ext.to_lowercase());
          let d = props.map(|p| p.desirability as i32).unwrap_or(0);
          let is_same_type = props
            .and_then(|p| p.destination_type.as_ref())
            .map(|dt| dt == ext)
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

    for node in &nodes {
      let mut node_mut = node.clone();
      if let Some(source) = self.find_graphic_file(&doc, node, &search_paths) {
        // For now, set the source path directly (trivial case)
        // Full image transformation requires image processing library
        let rel_path = if let Some(dest_dir) = doc.get_destination_directory() {
          let p = Path::new(&source);
          let b = Path::new(dest_dir);
          p.strip_prefix(b)
            .map(|r| r.to_string_lossy().to_string())
            .unwrap_or_else(|_| source.clone())
        } else {
          source.clone()
        };
        Self::set_graphic_src(&mut node_mut, &rel_path, None, None);
      } else {
        let graphic = node.get_attribute("graphic").unwrap_or_else(|| "none".to_string());
        log::warn!("No graphic source found for {}", graphic);
        node_mut.set_attribute("imagesrc", &graphic).ok();
      }
    }

    Ok(vec![doc])
  }
}
