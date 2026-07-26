//! EPUB manifest creation.
//!
//! Port of `LaTeXML::Post::Manifest::Epub` (252 lines of Perl).
//! Creates the EPUB 3.2 package structure:
//! - `mimetype` file
//! - `META-INF/container.xml`
//! - `OPS/content.opf` (spine + manifest)
//! - Indexes all content files with correct media types

use std::{fs, path::Path};

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
      hrefs.contains(&"Fisher&Yates.xhtml".to_string()),
      "the ampersand href did not survive the round-trip: {hrefs:?}"
    );
    assert!(
      hrefs.contains(&"a\"quote\".xhtml".to_string())
        && hrefs.contains(&"a<less>.xhtml".to_string()),
      "a quote/less-than href did not survive the round-trip: {hrefs:?}"
    );
    assert!(
      hrefs.contains(&"LaTeXML&core.css".to_string()),
      "a RESOURCE href is interpolated by the same code path and must be escaped too: {hrefs:?}"
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
