//! Post-processing pipeline API.
//!
//! Provides a public interface to the LaTeXML post-processing pipeline
//! (Scan → Bibliography → CrossRef → Graphics → Split → MathML → XSLT → HTML5 fixups).
//! Used by both the `latexml_oxide` binary and the `cortex_worker` binary.

use latexml_post::document::{PostDocument, PostDocumentOptions};
use latexml_post::object_db::ObjectDB;
use latexml_post::processor::Processor;

/// Options for the post-processing pipeline.
pub struct PostOptions<'a> {
  pub pmml: bool,
  pub cmml: bool,
  pub keep_xmath: bool,
  pub stylesheet: Option<&'a str>,
  pub destination: Option<&'a str>,
  pub source_directory: Option<&'a str>,
  pub nodefaultresources: bool,
  pub css_files: &'a [String],
  pub js_files: &'a [String],
  pub noinvisibletimes: bool,
  pub mathtex: bool,
  pub navigationtoc: Option<&'a str>,
  pub split: bool,
  pub split_xpath: Option<String>,
  pub split_naming: Option<&'a str>,
  pub xslt_parameters: &'a [String],
}

/// Run the post-processing pipeline on XML output.
///
/// Executes: Scan → MakeBibliography → CrossRef → Graphics → Split → MathML → XSLT → HTML5 fixups.
pub fn run_post_processing(xml: &str, opts: &PostOptions) -> String {
  let PostOptions {
    pmml, cmml, keep_xmath, stylesheet, destination,
    source_directory, nodefaultresources, css_files, js_files, noinvisibletimes,
    mathtex, navigationtoc, split, ref split_xpath, split_naming, xslt_parameters,
  } = *opts;

  let mut doc_opts = PostDocumentOptions::default();
  if let Some(dest) = destination {
    doc_opts.destination = Some(dest.to_string());
  }
  if let Some(src_dir) = source_directory {
    doc_opts.source_directory = Some(src_dir.to_string());
    let mut sp = doc_opts.searchpaths.take().unwrap_or_default();
    sp.push(src_dir.to_string());
    doc_opts.searchpaths = Some(sp);
  }
  let doc = match PostDocument::new_from_string(xml, doc_opts) {
    Ok(d) => d,
    Err(e) => {
      eprintln!("Post-processing: failed to parse XML: {}", e);
      return xml.to_string();
    }
  };

  // Phase 1: Scan
  let db = ObjectDB::new();
  let mut scanner = latexml_post::scan::Scan::new(db);
  let scan_nodes = scanner.to_process(&doc);
  let doc = match scanner.process(doc, scan_nodes) {
    Ok(mut docs) => docs.remove(0),
    Err(e) => {
      eprintln!("Post-processing: Scan failed: {}", e);
      return xml.to_string();
    }
  };

  // Phase 1.5: MakeBibliography
  let db = scanner.db;
  let mut bibmaker = latexml_post::make_bibliography::MakeBibliography::new(db, false);
  let bib_nodes = bibmaker.to_process(&doc);
  let doc = if !bib_nodes.is_empty() {
    match bibmaker.process(doc, bib_nodes) {
      Ok(mut docs) => docs.remove(0),
      Err(e) => {
        eprintln!("Post-processing: MakeBibliography failed: {}", e);
        return xml.to_string();
      }
    }
  } else {
    doc
  };

  // Phase 2: CrossRef
  let db = bibmaker.db;
  let mut crossref = latexml_post::crossref::CrossRef::new(
    db,
    latexml_post::crossref::UrlStyle::File,
    true,
  );
  let xref_nodes = crossref.to_process(&doc);
  let doc = match crossref.process(doc, xref_nodes) {
    Ok(mut docs) => docs.remove(0),
    Err(e) => {
      eprintln!("Post-processing: CrossRef failed: {}", e);
      return xml.to_string();
    }
  };

  // Phase 2.5: Graphics
  let mut graphics_proc = latexml_post::graphics::Graphics::new(None, true);
  let graphics_nodes = graphics_proc.to_process(&doc);
  let doc = if !graphics_nodes.is_empty() {
    match graphics_proc.process(doc, graphics_nodes) {
      Ok(mut docs) => docs.remove(0),
      Err(e) => {
        eprintln!("Post-processing: Graphics failed: {}", e);
        return xml.to_string();
      }
    }
  } else {
    doc
  };

  // Phase 2.75: Split
  let doc = if split {
    if let Some(ref xpath) = split_xpath {
      let naming = match split_naming {
        Some("id") | None => latexml_post::split::SplitNaming::Id,
        Some("idrelative") => latexml_post::split::SplitNaming::IdRelative,
        Some("label") => latexml_post::split::SplitNaming::Label,
        Some("labelrelative") => latexml_post::split::SplitNaming::LabelRelative,
        Some(other) => {
          eprintln!("Unknown splitnaming '{}', using 'id'", other);
          latexml_post::split::SplitNaming::Id
        }
      };
      let mut splitter = latexml_post::split::Split::new(xpath, naming, false);
      let split_nodes = splitter.to_process(&doc);
      match splitter.process(doc, split_nodes) {
        Ok(mut docs) => {
          if docs.len() > 1 {
            eprintln!("Split into {} documents", docs.len());
          }
          docs.remove(0)
        }
        Err(e) => {
          eprintln!("Post-processing: Split failed: {}", e);
          return xml.to_string();
        }
      }
    } else {
      doc
    }
  } else {
    doc
  };

  // Phase 3: MathML + XSLT
  let mut post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  let intent_literal = xml.contains("package=\"ar5iv");

  if pmml {
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_presentation()
        .with_keep_xmath(keep_xmath)
        .with_invisible_times(!noinvisibletimes)
        .with_mathtex(mathtex)
        .with_intent_literal(intent_literal),
    ));
  }
  if cmml {
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_content()
        .with_keep_xmath(keep_xmath)
        .with_invisible_times(!noinvisibletimes),
    ));
  }
  if let Some(xsl_path) = stylesheet {
    let mut searchpaths = vec![".".to_string()];
    if let Ok(exe) = std::env::current_exe() {
      if let Some(project_root) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
        searchpaths.insert(0, project_root.display().to_string());
      }
    }
    let mut xslt_params = std::collections::HashMap::new();
    if !css_files.is_empty() {
      xslt_params.insert("CSS".to_string(), format!("\"{}\"", css_files.join("|")));
    }
    if !js_files.is_empty() {
      xslt_params.insert(
        "JAVASCRIPT".to_string(),
        format!("\"{}\"", js_files.join("|")),
      );
    }
    if let Some(navtoc) = navigationtoc {
      xslt_params.insert("NAVIGATIONTOC".to_string(), format!("\"{}\"", navtoc));
    }
    for param in xslt_parameters {
      if let Some((key, value)) = param.split_once('=') {
        xslt_params.insert(key.to_string(), format!("\"{}\"", value));
      }
    }
    match latexml_post::xslt::XSLT::new(
      xsl_path,
      xslt_params,
      nodefaultresources,
      None,
      searchpaths,
    ) {
      Ok(xslt) => processors.push(Box::new(xslt)),
      Err(e) => eprintln!("Post-processing: XSLT error: {}", e),
    }
  }

  match post.process_chain(doc, &mut processors) {
    Ok(results) => {
      let output = results[0].to_xml_string();
      if stylesheet.is_some_and(|s| s.contains("html")) {
        let re = regex::Regex::new(
          r"<(span|div|p|a|td|th|tr|section|article|figure|figcaption|pre|code|em|strong|b|i|u|sub|sup|small|cite)(\s[^>]*)?/>",
        )
        .unwrap();
        let output = re.replace_all(&output, "<$1$2></$1>").to_string();
        let void_close_re = regex::Regex::new(
          r"</(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)>",
        )
        .unwrap();
        let output = void_close_re.replace_all(&output, "").to_string();
        let void_selfclose_re = regex::Regex::new(
          r"<(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)(\s[^>]*?)\s*/>",
        )
        .unwrap();
        void_selfclose_re
          .replace_all(&output, "<$1$2>")
          .to_string()
      } else {
        output
      }
    }
    Err(e) => {
      eprintln!("Post-processing failed: {}", e);
      xml.to_string()
    }
  }
}
