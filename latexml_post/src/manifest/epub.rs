//! EPUB manifest creation.
//!
//! Port of `LaTeXML::Post::Manifest::Epub` (252 lines of Perl).
//! Creates the EPUB 3.2 package structure:
//! - `mimetype` file
//! - `META-INF/container.xml`
//! - `OPS/content.opf` (spine + manifest)
//! - Indexes all content files with correct media types

use std::{
  fs,
  path::{Path, PathBuf},
};

/// The OPF package namespace (EPUB 3).
const OPF_NS: &str = "http://www.idpf.org/2007/opf";
/// Dublin Core, the metadata vocabulary an OPF's `<metadata>` carries.
const DC_NS: &str = "http://purl.org/dc/elements/1.1/";

/// EPUB 3.2 container.xml content.
const CONTAINER_XML: &str = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="OPS/content.opf" media-type="application/oebps-package+xml"/>
   </rootfiles>
</container>"#;

/// Error context for a libxml call.
///
/// Generic over the error type on purpose: `rust-libxml` returns
/// `Box<dyn Error>` from some constructors and `String` from others, so one
/// fixed-argument closure cannot serve both.
fn ctx<E>(what: impl Into<String>) -> impl Fn(E) -> String {
  let what = what.into();
  move |_| format!("couldn't create {what}")
}

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
  pub fn initialize(&mut self) -> Result<(), String> {
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

  /// Port of `Epub::process`: the package for the content documents `documents` (their destinations, in reading
  /// order) under `OPS/`, every other file there a resource, `content.opf` written unless the user supplied one.
  pub fn process(&mut self, documents: &[PathBuf], title: Option<&str>) -> Result<(), String> {
    self.initialize()?;
    let ops = Path::new(&self.site_directory).join("OPS");
    let spine: Vec<SpineEntry> = documents
      .iter()
      .map(|doc| {
        let (has_math, has_svg, has_nav) = content_properties(doc);
        let relative = doc.strip_prefix(&ops).unwrap_or(doc);
        self.add_document(&relative.to_string_lossy(), has_math, has_svg, has_nav)
      })
      .collect();
    // Epub::finalize: every non-XHTML file under OPS/, in name order.
    let mut files = Vec::new();
    collect_files(&ops, &mut files);
    files.sort();
    let resources: Vec<ResourceEntry> = files
      .iter()
      .filter_map(|path| {
        let href = path.strip_prefix(&ops).ok()?.to_string_lossy().into_owned();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "xhtml" || href == "content.opf" || path.ends_with("LaTeXML.cache") {
          return None;
        }
        let media_type = core_media_type(ext);
        if media_type == "application/octet-stream" {
          crate::Info!(
            "unexpected",
            ext.to_lowercase(),
            "resource '{}' is not of a core media type, assigning type application/octet-stream",
            href
          );
        }
        Some(ResourceEntry {
          id: url_to_id(&href),
          href,
          media_type: media_type.to_string(),
        })
      })
      .collect();
    let content_path = ops.join("content.opf");
    if content_path.is_file() {
      crate::Info!(
        "note",
        "content.opf",
        "using the manifest supplied by the user"
      );
      return Ok(());
    }
    // Perl reads the title from the ObjectDB's SITE_ROOT entry, which carries no authors or language.
    let opf = self.generate_opf(title.unwrap_or("No Title"), &[], "en", &spine, &resources);
    fs::write(&content_path, opf).map_err(|e| format!("Couldn't write content.opf: {e}"))
  }

  /// Add a document to the EPUB spine; `href` is its path relative to `OPS/`.
  ///
  /// Port of `Epub::process` per-document loop.
  pub fn add_document(
    &self,
    href: &str,
    has_math: bool,
    has_svg: bool,
    has_nav: bool,
  ) -> SpineEntry {
    let href = href.replace(std::path::MAIN_SEPARATOR, "/");
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
      id: url_to_id(&href),
      href,
      media_type: "application/xhtml+xml".to_string(),
      properties: if properties.is_empty() {
        None
      } else {
        Some(properties.join(" "))
      },
    }
  }

  /// Generate the content.opf package document.
  ///
  /// Port of `Epub::finalize`. Built through the libxml DOM rather than by
  /// `push_str`, because an OPF is parsed by strict readers: one unescaped `&`
  /// in a single `href` invalidates the whole package, not one entry.
  ///
  /// That was reachable. `href` is `format!("{}.{}", name, ext)` over a split
  /// document's file stem, and `--splitnaming=label` takes that stem from the
  /// author's `\label{...}` — `\label{Fisher&Yates}` really does produce a file
  /// named `Fisher&Yates.xhtml`. The old builder escaped 2 of 12 interpolated
  /// values and `href` was not one of them. libxml escapes on set, so the
  /// question no longer arises for any of them. (Issue 386 item 2.)
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

    match self.build_opf_dom(title, authors, language, uid, spine, resources) {
      Ok(xml) => xml,
      // A DOM allocation failure is not something a caller can act on, and an
      // empty package is a clearer failure than a half-built one.
      Err(e) => {
        crate::Error!(
          "epub",
          "opf",
          "Couldn't build the EPUB package document: {}",
          e
        );
        String::new()
      },
    }
  }

  fn build_opf_dom(
    &self,
    title: &str,
    authors: &[String],
    language: &str,
    uid: &str,
    spine: &[SpineEntry],
    resources: &[ResourceEntry],
  ) -> Result<String, String> {
    use libxml::tree::{Document as XmlDoc, Namespace, Node};

    let mut doc = XmlDoc::new().map_err(|_| "couldn't create the OPF document".to_string())?;

    let mut package = Node::new("package", None, &doc).map_err(ctx("<package>"))?;
    let opf_ns = Namespace::new("", OPF_NS, &mut package).map_err(ctx("the OPF namespace"))?;
    package
      .set_namespace(&opf_ns)
      .map_err(ctx("the OPF namespace"))?;
    package
      .set_attribute("unique-identifier", "pub-id")
      .map_err(ctx("@unique-identifier"))?;
    package
      .set_attribute("version", "3.0")
      .map_err(ctx("@version"))?;
    doc.set_root_element(&package);

    // ---- metadata ----
    let mut metadata = Node::new("metadata", None, &doc).map_err(ctx("<metadata>"))?;
    // `dc:` is declared HERE, matching the shape readers expect, and the
    // Dublin Core children below are created in it.
    let dc_ns = Namespace::new("dc", DC_NS, &mut metadata).map_err(ctx("the dc namespace"))?;
    package
      .add_child(&mut metadata)
      .map_err(ctx("<metadata>"))?;

    // A closure would hold `metadata` borrowed across every call, so the
    // Dublin Core children are appended inline through one small helper.
    fn dc_child(
      doc: &XmlDoc,
      dc_ns: &Namespace,
      metadata: &mut Node,
      name: &str,
      text: &str,
    ) -> Result<Node, String> {
      let mut n = Node::new(name, Some(dc_ns.clone()), doc).map_err(ctx(name))?;
      n.set_content(text).map_err(ctx(name))?;
      metadata.add_child(&mut n).map_err(ctx(name))?;
      Ok(n)
    }
    dc_child(&doc, &dc_ns, &mut metadata, "title", title)?;
    for author in authors {
      dc_child(&doc, &dc_ns, &mut metadata, "creator", author)?;
    }
    dc_child(&doc, &dc_ns, &mut metadata, "language", language)?;

    let mut modified = Node::new("meta", None, &doc).map_err(ctx("<meta>"))?;
    modified
      .set_attribute("property", "dcterms:modified")
      .map_err(ctx("@property"))?;
    modified
      .set_content(&chrono_like_now())
      .map_err(ctx("<meta>"))?;
    metadata.add_child(&mut modified).map_err(ctx("<meta>"))?;

    let mut identifier = dc_child(&doc, &dc_ns, &mut metadata, "identifier", uid)?;
    identifier
      .set_attribute("id", "pub-id")
      .map_err(ctx("@id"))?;

    // ---- manifest ----
    let mut manifest = Node::new("manifest", None, &doc).map_err(ctx("<manifest>"))?;
    package
      .add_child(&mut manifest)
      .map_err(ctx("<manifest>"))?;
    let mut item =
      |id: &str, href: &str, media_type: &str, properties: Option<&str>| -> Result<(), String> {
        let mut n = Node::new("item", None, &doc).map_err(ctx("<item>"))?;
        n.set_attribute("id", id).map_err(ctx("@id"))?;
        n.set_attribute("href", href).map_err(ctx("@href"))?;
        n.set_attribute("media-type", media_type)
          .map_err(ctx("@media-type"))?;
        if let Some(props) = properties {
          n.set_attribute("properties", props)
            .map_err(ctx("@properties"))?;
        }
        manifest.add_child(&mut n).map_err(ctx("<item>"))
      };
    for entry in spine {
      item(
        &entry.id,
        &entry.href,
        &entry.media_type,
        entry.properties.as_deref(),
      )?;
    }
    for res in resources {
      item(&res.id, &res.href, &res.media_type, None)?;
    }

    // ---- spine ----
    let mut spine_el = Node::new("spine", None, &doc).map_err(ctx("<spine>"))?;
    package.add_child(&mut spine_el).map_err(ctx("<spine>"))?;
    for entry in spine {
      let mut itemref = Node::new("itemref", None, &doc).map_err(ctx("<itemref>"))?;
      itemref
        .set_attribute("idref", &entry.id)
        .map_err(ctx("@idref"))?;
      spine_el.add_child(&mut itemref).map_err(ctx("<itemref>"))?;
    }

    Ok(doc.to_string())
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
/// Port of `url_id`: a character outside NCName becomes `_xN_` (N its uppercase hex codepoint), and an `_` that
/// opens such a sequence is itself escaped first.
fn url_to_id(name: &str) -> String {
  let chars: Vec<char> = name.chars().collect();
  let mut result = String::from("_");
  for (i, &ch) in chars.iter().enumerate() {
    if ch == '_' && opens_escape(&chars[i + 1..]) {
      result.push_str("_x5F_");
    } else if is_ncname_char(ch) {
      result.push(ch);
    } else {
      result.push_str(&format!("_x{:X}_", ch as u32));
    }
  }
  result
}

/// Whether `rest` (the text after an `_`) reads `x<HEX>+_`, Perl's `_(x[0-9A-F]+)(?=_)`.
fn opens_escape(rest: &[char]) -> bool {
  let hex = rest
    .iter()
    .skip(1)
    .take_while(|c| matches!(c, '0'..='9' | 'A'..='F'))
    .count();
  rest.first() == Some(&'x') && hex > 0 && rest.get(1 + hex) == Some(&'_')
}

/// The NCName character class of Perl's `url_id`.
fn is_ncname_char(ch: char) -> bool {
  matches!(ch,
    'A'..='Z' | '_' | 'a'..='z' | '-' | '.' | '0'..='9' | '\u{B7}'
    | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{2FF}' | '\u{300}'..='\u{36F}'
    | '\u{370}'..='\u{37D}' | '\u{37F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}'
    | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}'
    | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}')
}

/// The OPF item properties of a content document: (MathML, SVG, the navigation TOC), Perl `Epub::process`.
fn content_properties(path: &Path) -> (bool, bool, bool) {
  let Ok(doc) = libxml::parser::Parser::default().parse_file(&path.to_string_lossy()) else {
    return (false, false, false);
  };
  let Ok(mut xpath) = libxml::xpath::Context::new(&doc) else {
    return (false, false, false);
  };
  let mut has = |query: &str| {
    xpath
      .findnodes(query, None)
      .is_ok_and(|nodes| !nodes.is_empty())
  };
  (
    has(r#"//*[local-name() = "math"]"#),
    has(r#"//*[local-name() = "svg"]"#),
    has(r#"//*[@class="ltx_toclist"]"#),
  )
}

/// Every file under `dir`, recursively.
fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
  let Ok(entries) = fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      collect_files(&path, files);
    } else if path.is_file() {
      files.push(path);
    }
  }
}

/// The text content of `node` without its `ltx:indexmark`s (Perl Scan.pm `cleanNode`, then `textContent`); the
/// serializer's indentation (blank text in element-only content) is not text.
fn text_without_indexmarks(node: &libxml::tree::Node) -> String {
  use libxml::tree::NodeType;
  let is_text = |n: &libxml::tree::Node| {
    matches!(
      n.get_type(),
      Some(NodeType::TextNode | NodeType::CDataSectionNode)
    )
  };
  let mixed = node
    .get_child_nodes()
    .iter()
    .any(|n| is_text(n) && !n.get_content().trim().is_empty());
  let mut text = String::new();
  let mut child = node.get_first_child();
  while let Some(current) = child {
    if is_text(&current) && !mixed {
      child = current.get_next_sibling();
      continue;
    }
    match current.get_type() {
      Some(NodeType::ElementNode) if current.get_name() != "indexmark" => {
        text.push_str(&text_without_indexmarks(&current));
      },
      Some(NodeType::TextNode | NodeType::CDataSectionNode) => {
        text.push_str(&current.get_content());
      },
      _ => {},
    }
    child = current.get_next_sibling();
  }
  text
}

/// The parts of the core XML an EPUB needs before post-processing.
pub struct EpubSource {
  /// The core XML with a navigation TOC, when `check_toc` had to add one.
  pub xml:   Option<String>,
  /// The document's title text (Scan's `cleanNode`: its index marks removed).
  pub title: Option<String>,
}

/// Port of `LaTeXML.pm` `check_TOC` (an EPUB requires a TOC: add a full, undisplayed one unless the document has
/// one) together with the title the manifest reads from the document.
pub fn prepare_source(core_xml: &str) -> Result<EpubSource, String> {
  const LTX_NS: &str = "http://dlmf.nist.gov/LaTeXML";
  let doc = libxml::parser::Parser::default()
    .parse_string(core_xml)
    .map_err(|e| format!("couldn't parse the document: {e:?}"))?;
  let mut root = doc
    .get_root_element()
    .ok_or("the document has no root element")?;
  let mut xpath =
    libxml::xpath::Context::new(&doc).map_err(|_| "couldn't create an XPath context")?;
  xpath
    .register_namespace("ltx", LTX_NS)
    .map_err(|_| "couldn't register the ltx namespace")?;
  let title = xpath
    .findnodes("/*/ltx:title", None)
    .ok()
    .and_then(|nodes| nodes.into_iter().next())
    .map(|title| text_without_indexmarks(&title));
  let has_toc = xpath
    .findnodes(r#"//ltx:TOC[@lists="toc"]"#, None)
    .is_ok_and(|nodes| !nodes.is_empty());
  let xml = if has_toc {
    None
  } else {
    let ns = root.get_namespace();
    let mut toc = libxml::tree::Node::new("TOC", ns, &doc).map_err(ctx("<TOC>"))?;
    for (name, value) in [
      ("lists", "toc"),
      ("scope", "global"),
      (
        "select",
        "ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:paragraph | \
         ltx:subparagraph | ltx:appendix | ltx:index | ltx:bibliography",
      ),
      ("class", "ltx_nodisplay"),
    ] {
      toc
        .set_attribute(name, value)
        .map_err(ctx("a TOC attribute"))?;
    }
    match root.get_first_child() {
      Some(mut first) => first.add_prev_sibling(&mut toc).map_err(ctx("<TOC>"))?,
      None => root.add_child(&mut toc).map(|_| ()).map_err(ctx("<TOC>"))?,
    };
    Some(doc.to_string())
  };
  Ok(EpubSource { xml, title })
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

/// Generate an ISO 8601 timestamp (CCYY-MM-DDThh:mm:ssZ), at `SOURCE_DATE_EPOCH` when set (Perl Epub.pm).
fn chrono_like_now() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  let secs = std::env::var("SOURCE_DATE_EPOCH")
    .ok()
    .and_then(|s| s.trim().parse::<u64>().ok())
    .unwrap_or_else(|| {
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
    });
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

#[cfg(test)]
mod tests {
  use super::*;

  fn spine_entry(id: &str, href: &str, properties: Option<&str>) -> SpineEntry {
    SpineEntry {
      id:         id.to_string(),
      href:       href.to_string(),
      media_type: "application/xhtml+xml".to_string(),
      properties: properties.map(str::to_string),
    }
  }

  /// The package document must be well-formed for **author-controlled** input.
  ///
  /// Issue 386 item 2. An OPF is read by strict parsers, so one unescaped `&`
  /// invalidates the whole book rather than one entry — and `&` is reachable:
  /// `href` is built from a split document's file stem, and `--splitnaming=label`
  /// takes that stem from the author's `\label{...}`, so `\label{Fisher&Yates}`
  /// yields a file named `Fisher&Yates.xhtml`. The old `push_str` builder escaped
  /// 2 of 12 interpolated values and `href` was not among them.
  ///
  /// The assertion is deliberately "it parses", not "it contains `&amp;`":
  /// well-formedness is the property that matters, and it cannot be satisfied by
  /// accident.
  #[test]
  fn opf_stays_well_formed_when_author_input_carries_xml_specials() {
    let mut manifest = EpubManifest::new("/tmp/does-not-need-to-exist");
    manifest.unique_identifier = Some("urn:uuid:test".to_string());

    let spine = vec![
      spine_entry(
        "_Fisher_x26_Yates.xhtml",
        "Fisher&Yates.xhtml",
        Some("mathml"),
      ),
      spine_entry("_quote_x22_.xhtml", "a\"quote\".xhtml", None),
      spine_entry("_lt_x3C_.xhtml", "a<less>.xhtml", None),
    ];
    let resources = vec![ResourceEntry {
      id:         "_css".to_string(),
      href:       "LaTeXML&core.css".to_string(),
      media_type: "text/css".to_string(),
    }];

    let opf = manifest.generate_opf(
      "Tom & Jerry: a <study> of \"conflict\"",
      &["Ampersand & Co.".to_string()],
      "en",
      &spine,
      &resources,
    );

    let parser = libxml::parser::Parser::default();
    let doc = parser.parse_string(&opf).unwrap_or_else(|e| {
      panic!("the OPF is not well-formed XML ({e:?}) — an unescaped special reached it:\n{opf}")
    });
    let root = doc.get_root_element().expect("a root element");
    assert_eq!(root.get_name(), "package");

    // The values must round-trip to their ORIGINAL text, not to an escaped or
    // truncated form — well-formedness alone would also be satisfied by dropping
    // the offending characters.
    let hrefs: Vec<String> = crate::document::element_children(
      &crate::document::element_children(&root)
        .into_iter()
        .find(|n| n.get_name() == "manifest")
        .expect("a <manifest>"),
    )
    .iter()
    .filter_map(|n| n.get_attribute("href"))
    .collect();
    assert!(
      hrefs.iter().any(|h| h == "Fisher&Yates.xhtml"),
      "the ampersand href did not survive the round-trip: {hrefs:?}"
    );
    assert!(
      hrefs.iter().any(|h| h == "a\"quote\".xhtml") && hrefs.iter().any(|h| h == "a<less>.xhtml"),
      "a quote/less-than href did not survive the round-trip: {hrefs:?}"
    );
    assert!(
      hrefs.iter().any(|i| i == "LaTeXML&core.css"),
      "a RESOURCE href is interpolated by the same code path and must be escaped too: {hrefs:?}"
    );
  }

  /// Perl `url_id`: NCName letters beyond ASCII stay, an `_` opening an `_xN_` is escaped.
  #[test]
  fn url_id_follows_perl() {
    assert_eq!(
      url_to_id("Gödel_x41_ a.xhtml"),
      "_Gödel_x5F_x41__x20_a.xhtml"
    );
  }

  /// Structure the readers rely on, so the DOM rewrite cannot quietly drop a
  /// required element or attribute.
  #[test]
  fn opf_carries_the_structure_epub_readers_require() {
    let mut manifest = EpubManifest::new("/tmp/does-not-need-to-exist");
    manifest.unique_identifier = Some("urn:uuid:abc".to_string());
    let spine = vec![spine_entry("_a.xhtml", "a.xhtml", Some("nav"))];

    let opf = manifest.generate_opf("T", &["A".to_string()], "en", &spine, &[]);
    let parser = libxml::parser::Parser::default();
    let doc = parser.parse_string(&opf).expect("well-formed");
    let root = doc.get_root_element().expect("root");

    assert_eq!(root.get_attribute("version").as_deref(), Some("3.0"));
    assert_eq!(
      root.get_attribute("unique-identifier").as_deref(),
      Some("pub-id")
    );

    let kids = crate::document::element_children(&root);
    let names: Vec<String> = kids.iter().map(|n| n.get_name()).collect();
    assert_eq!(
      names,
      vec!["metadata", "manifest", "spine"],
      "OPF child order"
    );

    // The identifier the package points at must actually carry that id.
    let meta = kids.iter().find(|n| n.get_name() == "metadata").unwrap();
    let ident = crate::document::element_children(meta)
      .into_iter()
      .find(|n| n.get_attribute("id").as_deref() == Some("pub-id"))
      .expect("a <dc:identifier id='pub-id'>");
    assert_eq!(ident.get_content(), "urn:uuid:abc");

    // `properties` is optional and must be emitted only when present.
    let item =
      crate::document::element_children(kids.iter().find(|n| n.get_name() == "manifest").unwrap())
        .into_iter()
        .next()
        .expect("an <item>");
    assert_eq!(item.get_attribute("properties").as_deref(), Some("nav"));

    let itemref =
      crate::document::element_children(kids.iter().find(|n| n.get_name() == "spine").unwrap())
        .into_iter()
        .next()
        .expect("an <itemref>");
    assert_eq!(itemref.get_attribute("idref").as_deref(), Some("_a.xhtml"));
  }
}
