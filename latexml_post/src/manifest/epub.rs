//! EPUB manifest creation.
//!
//! Port of `LaTeXML::Post::Manifest::Epub` (252 lines of Perl).
//! Creates the EPUB 3.2 package structure:
//! - `mimetype` file
//! - `META-INF/container.xml`
//! - `OPS/content.opf` (spine + manifest)
//! - Indexes all content files with correct media types

use std::{fs, path::Path};

/// EPUB 3.2 container.xml content.
const CONTAINER_XML: &str = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="OPS/content.opf" media-type="application/oebps-package+xml"/>
   </rootfiles>
</container>"#;

/// Core Media Types as per EPUB 3.2 spec.
fn core_media_type(ext: &str) -> &'static str {
  match ext.to_lowercase().as_str() {
    "gif" => "image/gif",
    "jpg" | "jpeg" => "image/jpeg",
    "png" => "image/png",
    "svg" => "image/svg+xml",
    "mp3" => "audio/mpeg",
    "mp4" | "mpg4" => "audio/mp4",
    "css" => "text/css",
    "ttf" => "font/ttf",
    "otf" => "font/otf",
    "woff" => "font/woff",
    "woff2" => "font/woff2",
    "xhtml" => "application/xhtml+xml",
    "js" => "text/javascript",
    "ncx" => "application/x-dtbncx+xml",
    "smi" | "smil" => "application/smil+xml",
    "pls" => "application/pls+xml",
    _ => "application/octet-stream",
  }
}

/// EPUB manifest builder.
///
/// Port of `LaTeXML::Post::Manifest::Epub`.
pub struct EpubManifest {
  site_directory:    String,
  unique_identifier: Option<String>,
}

impl EpubManifest {
  pub fn new(site_directory: &str) -> Self {
    EpubManifest {
      site_directory:    site_directory.to_string(),
      unique_identifier: None,
    }
  }

  /// Initialize the EPUB directory structure.
  ///
  /// Port of `Epub::initialize`.
  pub fn initialize(
    &mut self,
    _title: &str,
    _authors: &[String],
    _language: &str,
  ) -> Result<(), String> {
    let dir = &self.site_directory;

    // 1. Create mimetype file
    let mime_path = format!("{}/mimetype", dir);
    fs::write(&mime_path, "application/epub+zip")
      .map_err(|e| format!("Couldn't write mimetype: {}", e))?;

    // 2. Create META-INF/container.xml
    let meta_inf = format!("{}/META-INF", dir);
    fs::create_dir_all(&meta_inf).map_err(|e| format!("Couldn't create META-INF: {}", e))?;
    fs::write(format!("{}/container.xml", meta_inf), CONTAINER_XML)
      .map_err(|e| format!("Couldn't write container.xml: {}", e))?;

    // 3. Create OPS directory
    let ops_dir = format!("{}/OPS", dir);
    fs::create_dir_all(&ops_dir).map_err(|e| format!("Couldn't create OPS: {}", e))?;

    // Generate a UUID for the publication
    self.unique_identifier = Some(format!("urn:uuid:{}", generate_uuid()));

    Ok(())
  }

  /// Add a document to the EPUB spine.
  ///
  /// Port of `Epub::process` per-document loop.
  pub fn add_document(
    &self,
    destination: &str,
    has_math: bool,
    has_svg: bool,
    has_nav: bool,
  ) -> SpineEntry {
    let path = Path::new(destination);
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("xhtml");
    let item_id = url_to_id(&format!("{}.{}", name, ext));

    let mut properties = Vec::new();
    if has_math {
      properties.push("mathml");
    }
    if has_svg {
      properties.push("svg");
    }
    if has_nav {
      properties.push("nav");
    }

    SpineEntry {
      id:         item_id,
      href:       format!("{}.{}", name, ext),
      media_type: "application/xhtml+xml".to_string(),
      properties: if properties.is_empty() {
        None
      } else {
        Some(properties.join(" "))
      },
    }
  }

  /// Generate the content.opf XML as a string.
  ///
  /// Port of `Epub::finalize`.
  pub fn generate_opf(
    &self,
    title: &str,
    authors: &[String],
    language: &str,
    spine: &[SpineEntry],
    resources: &[ResourceEntry],
  ) -> String {
    let uid = self
      .unique_identifier
      .as_deref()
      .unwrap_or("urn:uuid:00000000-0000-0000-0000-000000000000");
    let now = chrono_like_now();

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<package xmlns=\"http://www.idpf.org/2007/opf\" unique-identifier=\"pub-id\" version=\"3.0\">\n");

    // Metadata
    xml.push_str("  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n");
    xml.push_str(&format!("    <dc:title>{}</dc:title>\n", escape_xml(title)));
    for author in authors {
      xml.push_str(&format!(
        "    <dc:creator>{}</dc:creator>\n",
        escape_xml(author)
      ));
    }
    xml.push_str(&format!("    <dc:language>{}</dc:language>\n", language));
    xml.push_str(&format!(
      "    <meta property=\"dcterms:modified\">{}</meta>\n",
      now
    ));
    xml.push_str(&format!(
      "    <dc:identifier id=\"pub-id\">{}</dc:identifier>\n",
      uid
    ));
    xml.push_str("  </metadata>\n");

    // Manifest
    xml.push_str("  <manifest>\n");
    for entry in spine {
      xml.push_str(&format!(
        "    <item id=\"{}\" href=\"{}\" media-type=\"{}\"",
        entry.id, entry.href, entry.media_type
      ));
      if let Some(ref props) = entry.properties {
        xml.push_str(&format!(" properties=\"{}\"", props));
      }
      xml.push_str("/>\n");
    }
    for res in resources {
      xml.push_str(&format!(
        "    <item id=\"{}\" href=\"{}\" media-type=\"{}\"/>\n",
        res.id, res.href, res.media_type
      ));
    }
    xml.push_str("  </manifest>\n");

    // Spine
    xml.push_str("  <spine>\n");
    for entry in spine {
      xml.push_str(&format!("    <itemref idref=\"{}\"/>\n", entry.id));
    }
    xml.push_str("  </spine>\n");
    xml.push_str("</package>\n");

    xml
  }
}

/// An entry in the EPUB spine (content document).
#[derive(Debug)]
pub struct SpineEntry {
  pub id:         String,
  pub href:       String,
  pub media_type: String,
  pub properties: Option<String>,
}

/// A resource entry (CSS, images, fonts).
#[derive(Debug)]
pub struct ResourceEntry {
  pub id:         String,
  pub href:       String,
  pub media_type: String,
}

/// Convert a URL/filename to a valid NCName for use as an XML id.
///
/// Port of `url_id`.
fn url_to_id(name: &str) -> String {
  let mut result = String::from("_");
  for ch in name.chars() {
    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
      result.push(ch);
    } else {
      result.push_str(&format!("_x{:X}_", ch as u32));
    }
  }
  result
}

/// Generate a UUID v4 string.
fn generate_uuid() -> String {
  // Simple random UUID v4 (no external dependency)
  use std::time::{SystemTime, UNIX_EPOCH};
  let seed = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_nanos();
  // LCG-based pseudo-random for simplicity
  let mut state = seed as u64;
  let mut bytes = [0u8; 16];
  for b in &mut bytes {
    state = state
      .wrapping_mul(6364136223846793005)
      .wrapping_add(1442695040888963407);
    *b = (state >> 33) as u8;
  }
  bytes[6] = (bytes[6] & 0x0F) | 0x40; // version 4
  bytes[8] = (bytes[8] & 0x3F) | 0x80; // variant 1
  format!(
    "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
    bytes[0],
    bytes[1],
    bytes[2],
    bytes[3],
    bytes[4],
    bytes[5],
    bytes[6],
    bytes[7],
    bytes[8],
    bytes[9],
    bytes[10],
    bytes[11],
    bytes[12],
    bytes[13],
    bytes[14],
    bytes[15]
  )
}

/// Simple XML escaping.
fn escape_xml(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
}

/// Generate an ISO 8601 timestamp (CCYY-MM-DDThh:mm:ssZ).
fn chrono_like_now() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  let secs = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();
  // Simple UTC timestamp computation (no chrono dependency)
  let days = secs / 86400;
  let time_of_day = secs % 86400;
  let hours = time_of_day / 3600;
  let minutes = (time_of_day % 3600) / 60;
  let seconds = time_of_day % 60;
  // Rough date from epoch days (good enough for timestamps)
  let (year, month, day) = days_to_ymd(days as i64);
  format!(
    "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
    year, month, day, hours, minutes, seconds
  )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(days: i64) -> (i64, u32, u32) {
  // Algorithm from https://howardhinnant.github.io/date_algorithms.html
  let z = days + 719468;
  let era = if z >= 0 { z } else { z - 146096 } / 146097;
  let doe = (z - era * 146097) as u32;
  let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
  let y = yoe as i64 + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  let mp = (5 * doy + 2) / 153;
  let d = doy - (153 * mp + 2) / 5 + 1;
  let m = if mp < 10 { mp + 3 } else { mp - 9 };
  (y + if m <= 2 { 1 } else { 0 }, m, d)
}
