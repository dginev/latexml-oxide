//! Post-processing pipeline API.
//!
//! Provides a public interface to the LaTeXML post-processing pipeline
//! (Scan → Bibliography → CrossRef → Graphics → Split → MathML → XSLT → HTML5 fixups).
//! Used by both the `latexml_oxide` binary and the `cortex_worker` binary.

use latexml_core::{
  Info, s,
  telemetry::{self, Phase},
};
use latexml_post::{
  document::{PostDocument, PostDocumentOptions},
  object_db::ObjectDB,
  processor::Processor,
};
use once_cell::sync::Lazy;

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static POST_AUDIT: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_POST_AUDIT").is_ok());

/// Options for the post-processing pipeline.
pub struct PostOptions<'a> {
  pub pmml:                      bool,
  pub cmml:                      bool,
  pub keep_xmath:                bool,
  pub stylesheet:                Option<&'a str>,
  pub destination:               Option<&'a str>,
  pub source_directory:          Option<&'a str>,
  /// Extra resource search paths (the `--path` flag): directories searched
  /// for `--css`/`--javascript` files (and other post resources) to copy into
  /// the destination, in addition to the document's own paths, the current
  /// directory, and the binary's embedded resource table.
  pub search_paths:              &'a [String],
  pub nodefaultresources:        bool,
  pub css_files:                 &'a [String],
  pub js_files:                  &'a [String],
  pub noinvisibletimes:          bool,
  pub mathtex:                   bool,
  pub navigationtoc:             Option<&'a str>,
  pub schemadocs:                bool,
  pub split:                     bool,
  pub split_xpath:               Option<String>,
  pub split_naming:              Option<&'a str>,
  pub xslt_parameters:           &'a [String],
  /// If > 0, try `inkscape` for PDF graphics smaller than this many KB
  /// (vector-preservation path). Fall back to ImageMagick `convert` on
  /// failure or timeout. Tracks upstream brucemiller/LaTeXML#902.
  pub graphics_svg_threshold_kb: u32,
  /// Output extraction mode (Perl `LaTeXML::Util::Pack::whatsout`).
  /// `Document` (default) → serialize the full post-processed
  /// document; `Fragment` → embeddable HTML snippet via
  /// `latexml_post::extract::get_embeddable`; `Math` → math subtree
  /// via `get_math`. Applied to each `PostDocument` in the final
  /// serialization loop.
  pub whatsout:                  latexml_post::extract::Whatsout,
}

/// Run the post-processing pipeline on XML output.
///
/// Executes: Scan → MakeBibliography → CrossRef → Graphics → Split → MathML → XSLT → HTML5 fixups.
pub fn run_post_processing(xml: &str, opts: &PostOptions) -> String {
  let PostOptions {
    pmml,
    cmml,
    keep_xmath,
    stylesheet,
    destination,
    source_directory,
    search_paths,
    nodefaultresources,
    css_files,
    js_files,
    noinvisibletimes,
    mathtex,
    navigationtoc,
    schemadocs,
    split,
    ref split_xpath,
    split_naming,
    xslt_parameters,
    graphics_svg_threshold_kb,
    whatsout,
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
  let audit = *POST_AUDIT;
  let audit_start = |name: &str| -> Option<(String, std::time::Instant)> {
    if audit {
      Some((name.to_string(), std::time::Instant::now()))
    } else {
      None
    }
  };
  let audit_end = |started: Option<(String, std::time::Instant)>| {
    if let Some((name, t0)) = started {
      let ms = t0.elapsed().as_millis();
      Info!("audit", "phase", s!("{} took {}ms", name, ms));
    }
  };

  telemetry::phase_enter(Phase::PostXmlParse);
  let t_parse = audit_start("PostDocument::new_from_string");
  // Perl LaTeXML.pm L330-336: a completely empty core-conversion result
  // (e.g. after a Fatal abort) is still post-processed — Perl sets a bare
  // <document/> root first, "important for utility features such as
  // packing .zip archives for output". Mirror that for the empty-string
  // serialization instead of failing the parse with a libxml "Got a Null
  // pointer" and echoing the empty input through (witness: cortex_worker
  // on any Fatal paper produced a 0-byte .html).
  let xml = if xml.trim().is_empty() {
    "<document/>"
  } else {
    xml
  };
  let doc = match PostDocument::new_from_string(xml, doc_opts) {
    Ok(d) => d,
    Err(e) => {
      eprintln!("Post-processing: failed to parse XML: {}", e);
      telemetry::phase_exit();
      return xml.to_string();
    },
  };
  audit_end(t_parse);
  telemetry::phase_exit();

  // SVG extraction reads the pre-post XML before any in-tree mutation;
  // do it once up-front so the regex-based fragment table is valid for
  // every split sub-document below.
  let t_svg = audit_start("SVG extraction");
  let svg_fragments = extract_svg_fragments(xml);
  audit_end(t_svg);

  // Perl-faithful pipeline order (latexmlpost L223-242):
  //   Split → Scan → MakeBibliography → CrossRef → Graphics → ...
  // Split runs FIRST so each downstream pass sees the per-page
  // destination. With the previous order (Scan before Split), every
  // entry's `location` was the root document and CrossRef built every
  // ref as a within-page anchor — the user-visible TOC links pointed
  // at `#Ch1` instead of `Ch1.html`.
  //
  // Phase 1: Split (only attributes time when --split is on)
  let mut docs: Vec<PostDocument> = if split {
    telemetry::phase_enter(Phase::Split);
    let result = if let Some(xpath) = split_xpath {
      let naming = match split_naming {
        Some("id") | None => latexml_post::split::SplitNaming::Id,
        Some("idrelative") => latexml_post::split::SplitNaming::IdRelative,
        Some("label") => latexml_post::split::SplitNaming::Label,
        Some("labelrelative") => latexml_post::split::SplitNaming::LabelRelative,
        Some(other) => {
          eprintln!("Unknown splitnaming '{}', using 'id'", other);
          latexml_post::split::SplitNaming::Id
        },
      };
      let mut splitter = latexml_post::split::Split::new(xpath, naming, false);
      let split_nodes = splitter.to_process(&doc);
      match splitter.process(doc, split_nodes) {
        Ok(docs) => {
          if docs.len() > 1 {
            eprintln!("Split into {} documents", docs.len());
          }
          Ok(docs)
        },
        Err(e) => {
          eprintln!("Post-processing: Split failed: {}", e);
          Err(())
        },
      }
    } else {
      Ok(vec![doc])
    };
    telemetry::phase_exit();
    match result {
      Ok(ds) => ds,
      Err(()) => return xml.to_string(),
    }
  } else {
    vec![doc]
  };

  // Helper: run one processor across every doc in a Vec, mirroring
  // Perl Post.pm:50-65's `foreach $proc { @newdocs = ... }` loop.
  // Each processor's per-doc `process` may return multiple docs (only
  // Split actually does, and we've already run Split above), so the
  // inner result is flattened back into `docs`.
  fn run_phase<P: Processor + ?Sized>(
    docs: Vec<PostDocument>,
    proc: &mut P,
    label: &'static str,
  ) -> Result<Vec<PostDocument>, ()> {
    let mut out: Vec<PostDocument> = Vec::with_capacity(docs.len());
    for d in docs {
      let nodes = proc.to_process(&d);
      if nodes.is_empty() {
        out.push(d);
        continue;
      }
      match proc.process(d, nodes) {
        Ok(processed) => out.extend(processed),
        Err(e) => {
          eprintln!("Post-processing: {} failed: {}", label, e);
          return Err(());
        },
      }
    }
    Ok(out)
  }

  // Phase 2: Scan — runs on EACH sub-document so its entries register
  // the per-page `location` and `pageid`. Single shared ObjectDB so
  // the later CrossRef pass can resolve cross-doc refs.
  let mut scanner = latexml_post::scan::Scan::new(ObjectDB::new());
  telemetry::phase_enter(Phase::PostScan);
  let t_scan = audit_start("Scan");
  docs = match run_phase(docs, &mut scanner, "Scan") {
    Ok(d) => d,
    Err(()) => {
      telemetry::phase_exit();
      return xml.to_string();
    },
  };
  audit_end(t_scan);
  telemetry::phase_exit();

  // Phase 2b: MakeIndex (Perl LaTeXML.pm L466-470)
  //   Runs BEFORE MakeBibliography in Perl. Populates `<ltx:indexlist>`
  //   inside `<ltx:index>` placeholders and `<ltx:glossarylist>` inside
  //   `<ltx:glossary>` placeholders, using the `GLOSSARY:*` / `INDEX:*`
  //   entries Scan registered in the ObjectDB. Without this pass, glossary
  //   sections render empty in HTML (witness: tests/structure/glossary.tex).
  let mut indexer = latexml_post::make_index::MakeIndex::new(scanner.db, false, false);
  telemetry::phase_enter(Phase::PostScan);
  let t_idx = audit_start("MakeIndex");
  docs = match run_phase(docs, &mut indexer, "MakeIndex") {
    Ok(d) => d,
    Err(()) => {
      telemetry::phase_exit();
      return xml.to_string();
    },
  };
  audit_end(t_idx);
  telemetry::phase_exit();

  // Phase 3: MakeBibliography
  let mut bibmaker = latexml_post::make_bibliography::MakeBibliography::new(indexer.db, false);
  telemetry::phase_enter(Phase::Bibliography);
  let t_bib = audit_start("MakeBibliography");
  docs = match run_phase(docs, &mut bibmaker, "MakeBibliography") {
    Ok(d) => d,
    Err(()) => {
      telemetry::phase_exit();
      return xml.to_string();
    },
  };
  audit_end(t_bib);
  telemetry::phase_exit();

  // Phase 4: CrossRef
  let mut crossref = latexml_post::crossref::CrossRef::new(
    bibmaker.db,
    latexml_post::crossref::UrlStyle::File,
    true,
  );
  if let Some(navtoc) = navigationtoc {
    crossref.set_navigation_toc(navtoc);
  }
  telemetry::phase_enter(Phase::Crossref);
  let t_xref = audit_start("CrossRef");
  docs = match run_phase(docs, &mut crossref, "CrossRef") {
    Ok(d) => d,
    Err(()) => {
      telemetry::phase_exit();
      return xml.to_string();
    },
  };
  audit_end(t_xref);
  telemetry::phase_exit();

  // Phase 5: Graphics
  let mut graphics_proc = latexml_post::graphics::Graphics::new(None, true)
    .with_svg_threshold_kb(graphics_svg_threshold_kb);
  telemetry::phase_enter(Phase::Graphics);
  let t_gfx = audit_start("Graphics");
  docs = match run_phase(docs, &mut graphics_proc, "Graphics") {
    Ok(d) => d,
    Err(()) => {
      telemetry::phase_exit();
      return xml.to_string();
    },
  };
  audit_end(t_gfx);
  telemetry::phase_exit();

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
    if let Ok(exe) = std::env::current_exe()
      && let Some(project_root) = exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
    {
      searchpaths.insert(0, project_root.display().to_string());
    }
    // Prepend the user's `--path` directories so `--css`/`--javascript`
    // resources are found there (and take priority over `.`/project root)
    // when copy_param_resources searches for them.
    for p in search_paths.iter().rev() {
      searchpaths.insert(0, p.clone());
    }
    let mut xslt_params = rustc_hash::FxHashMap::default();

    // When `--schemadocs` is on, auto-prepend the rustdoc-styled
    // theme assets so callers don't need to repeat them on every
    // invocation. Idempotent: skipped if the user has already
    // listed the same basename via `--css` / `--javascript`.
    let prepend = |list: &[String], extra: &str| -> Vec<String> {
      if list
        .iter()
        .any(|p| std::path::Path::new(p).file_name().and_then(|s| s.to_str()) == Some(extra))
      {
        list.to_vec()
      } else {
        let mut out = Vec::with_capacity(list.len() + 1);
        out.push(extra.to_string());
        out.extend_from_slice(list);
        out
      }
    };
    let css_effective: Vec<String> = if schemadocs {
      prepend(css_files, latexml_post::schema_docs::THEME_CSS_BASENAME)
    } else {
      css_files.to_vec()
    };
    let js_effective: Vec<String> = if schemadocs {
      prepend(js_files, latexml_post::schema_docs::THEME_JS_BASENAME)
    } else {
      js_files.to_vec()
    };
    if !css_effective.is_empty() {
      xslt_params.insert(
        "CSS".to_string(),
        format!("\"{}\"", css_effective.join("|")),
      );
    }
    if !js_effective.is_empty() {
      xslt_params.insert(
        "JAVASCRIPT".to_string(),
        format!("\"{}\"", js_effective.join("|")),
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

  // process_chain attributes per-processor inside latexml_post::Post::
  // process_chain (MathmlPres / MathmlCont / Xslt). No outer phase wrap
  // needed; the inner per-processor guards cover their own time.
  //
  // Perl-faithful: pass ALL split docs to ProcessChain at once. Perl's
  // `Post::ProcessChain_internal` (Post.pm L41-67) seeds `@docs = ($doc)`
  // and lets each processor return a (possibly multi-element) list which
  // becomes the input to the next processor. We've already produced the
  // post-Split list above; ProcessChain just needs to fan MathML/XSLT
  // across it.
  let is_html_out = stylesheet.is_some_and(|s| s.contains("html"));
  let t_chain = audit_start("process_chain");
  let chain_result = post.process_chain(docs, &mut processors);
  audit_end(t_chain);
  let results = match chain_result {
    Ok(r) => r,
    Err(e) => {
      eprintln!("Post-processing failed: {}", e);
      return xml.to_string();
    },
  };

  // Serialize + write each doc IMMEDIATELY in the loop (rather than
  // collecting first then writing). Multi-doc PostDocument cleanup has
  // a known libxml2 idcache use-after-free at process exit (see the
  // SVG-processor note above this block) that can intermittently SIGSEGV
  // after the last doc has been serialized. Writing eagerly inside the
  // iteration means even if cleanup later trips, every page that finished
  // post-processing is already on disk. The first doc's content is
  // returned so the caller can also write it to --dest in non-split mode.
  let mut main_output: Option<String> = None;
  let n = results.len();
  for doc in results.into_iter() {
    let dest = doc.get_destination().map(String::from);
    let t_serialize = audit_start("to_xml_string");
    // `serialize_whatsout` is a no-op (returns full doc) for the
    // default `Whatsout::Document`; for Fragment / Math it returns
    // the extracted subtree serialized via libxml `node_to_string`.
    let output = latexml_post::extract::serialize_whatsout(&doc, whatsout);
    audit_end(t_serialize);
    let output = if is_html_out {
      finalize_html5(output, &svg_fragments)
    } else {
      output
    };
    let output = if schemadocs && is_html_out {
      latexml_post::schema_docs::process_page(&output)
    } else {
      output
    };
    // Write each doc to disk inside the loop. The single-file caller
    // also writes the first doc's content to --dest; the redundant write
    // is harmless and ensures the file is on disk even if a later libxml2
    // cleanup tripwire fires.
    if let Some(path) = dest.as_deref() {
      if let Some(parent) = std::path::Path::new(path).parent()
        && !parent.as_os_str().is_empty()
      {
        let _ = std::fs::create_dir_all(parent);
      }
      if let Err(e) = std::fs::write(path, &output) {
        eprintln!("Post-processing: failed to write page {}: {}", path, e);
      }
    }
    if main_output.is_none() {
      main_output = Some(output);
    }
    let _ = n;
  }

  main_output.unwrap_or_else(|| xml.to_string())
}

/// Apply HTML5 cleanup (XML prolog strip, void-element fixes) and inject SVG
/// fragments into empty `ltx_picture` spans. Pulled out of `run_post_processing`
/// so it can run on every split sub-document, not just the first.
fn finalize_html5(output: String, svg_fragments: &[(String, String)]) -> String {
  use std::sync::LazyLock;
  // Cached at first call — regex compile is the slow part of `Regex::new`,
  // and finalize_html5 runs on every (sub-)document in the post-pipeline.
  static XML_PROLOG_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^<\?xml[^?]*\?>\s*").unwrap());
  static NON_VOID_SELF_CLOSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
      r"<(span|div|p|a|td|th|tr|section|article|figure|figcaption|pre|code|em|strong|b|i|u|sub|sup|small|cite)(\s[^>]*)?/>",
    ).unwrap()
  });
  static VOID_CLOSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"</(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)>")
      .unwrap()
  });
  static VOID_SELF_CLOSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
      r"<(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)(\s[^>]*?)\s*/>",
    )
    .unwrap()
  });

  let _gp_html5 = telemetry::phase(Phase::Html5Fixups);
  // Strip <?xml version...?> prolog: HTML5 must NOT have an XML declaration.
  // libxml2's to_string() includes it by default; we strip it here.
  let output = XML_PROLOG_RE.replace(&output, "").to_string();
  let output = NON_VOID_SELF_CLOSE_RE
    .replace_all(&output, "<$1$2></$1>")
    .to_string();
  let output = VOID_CLOSE_RE.replace_all(&output, "").to_string();
  let mut output = VOID_SELF_CLOSE_RE
    .replace_all(&output, "<$1$2>")
    .to_string();
  if !svg_fragments.is_empty() {
    for (pic_id, svg_html) in svg_fragments {
      let pattern = format!(
        r#"<span id="{}" class="ltx_picture"([^>]*)></span>"#,
        regex::escape(pic_id)
      );
      if let Ok(re) = regex::Regex::new(&pattern) {
        output = re
          .replace(&output, |caps: &regex::Captures| {
            format!(
              r#"<span id="{}" class="ltx_picture"{}>{}</span>"#,
              pic_id, &caps[1], svg_html
            )
          })
          .to_string();
      }
    }
  }
  output
}

/// Extract SVG fragments from intermediate LaTeXML XML.
///
/// Finds `<picture>` elements, converts their children to inline SVG HTML.
/// Uses a lightweight regex+string approach (no libxml2) to avoid the
/// use-after-free crash in PostDocument cleanup.
///
/// Returns (picture_id, svg_html) pairs for post-XSLT injection.
fn extract_svg_fragments(xml: &str) -> Vec<(String, String)> {
  use std::sync::LazyLock;
  // Fast-fail: most documents have no `<picture>` elements (tikz / pgf
  // is uncommon in the canvas). Skip the backtracking lazy-match
  // regex (`(?s)...(.*?)`) when `<picture` doesn't appear as a literal
  // substring. `str::contains` is a SIMD-accelerated byte search and
  // takes microseconds even on ~MB inputs.
  if !xml.contains("<picture") {
    return Vec::new();
  }
  static PICTURE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<picture([^>]*)>(.*?)</picture>"#).unwrap());
  static ID_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"xml:id="([^"]+)""#).unwrap());
  static WIDTH_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"width="([^"]+)""#).unwrap());
  static HEIGHT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"height="([^"]+)""#).unwrap());
  let mut fragments = Vec::new();
  let picture_re = &*PICTURE_RE;
  let id_re = &*ID_RE;
  let width_re = &*WIDTH_RE;
  let height_re = &*HEIGHT_RE;

  for pic_caps in picture_re.captures_iter(xml) {
    let attrs = &pic_caps[1];
    let content = &pic_caps[2];
    let id = id_re
      .captures(attrs)
      .map(|c| c[1].to_string())
      .unwrap_or_default();
    let width = width_re.captures(attrs).and_then(|c| parse_tex_dim(&c[1]));
    let height = height_re.captures(attrs).and_then(|c| parse_tex_dim(&c[1]));

    if id.is_empty() || content.trim().is_empty() {
      continue;
    }

    let w = width.unwrap_or(100.0);
    let h = height.unwrap_or(100.0);

    // Build inline SVG: coordinate system has y-flip (TeX origin bottom-left, SVG top-left)
    let mut svg_content = format!(
      r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{w:.2}" height="{h:.2}" overflow="visible">"#,
    );
    svg_content.push_str(&format!(
      r#"<g transform="translate(0,{h:.2}) scale(1,-1)">"#,
    ));

    // Convert ltx picture children to SVG elements
    svg_content.push_str(&convert_picture_children_to_svg(content));

    svg_content.push_str("</g></svg>");
    fragments.push((id, svg_content));
  }
  fragments
}

/// Convert LaTeXML picture children (g, line, text, circle, etc.) to SVG elements.
fn convert_picture_children_to_svg(content: &str) -> String {
  use std::sync::LazyLock;
  static G_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<g([^>]*)>(.*?)</g>"#).unwrap());
  static TRANSFORM_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"transform="([^"]+)""#).unwrap());
  static LINE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<line\s+points="([^"]+)"([^/]*)/?>"#).unwrap());
  static CIRCLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<circle([^/]*)/?>"#).unwrap());
  static ELLIPSE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<ellipse([^/]*)/?>"#).unwrap());
  static RECT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<rect([^/]*)/?>"#).unwrap());
  static POLYGON_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<polygon([^/]*)/?>"#).unwrap());
  static PATH_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<path([^/]*)/?>"#).unwrap());
  static BEZIER_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<bezier\s+points="([^"]+)"([^/]*)/?>"#).unwrap());
  static TEXT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<text([^>]*)>(.*?)</text>"#).unwrap());

  let mut svg = String::new();

  for g_caps in G_RE.captures_iter(content) {
    let g_attrs = &g_caps[1];
    let g_content = &g_caps[2];

    // Extract transform
    let transform = TRANSFORM_RE.captures(g_attrs).map(|c| c[1].to_string());

    if let Some(t) = &transform {
      svg.push_str(&format!(r#"<g transform="{t}">"#));
    } else {
      svg.push_str("<g>");
    }

    // <line points="x1,y1 x2,y2" stroke="..." stroke-width="..."/>
    for line_caps in LINE_RE.captures_iter(g_content) {
      let points = &line_caps[1];
      let rest_attrs = &line_caps[2];
      let coords: Vec<&str> = points.split_whitespace().collect();
      if coords.len() >= 2 {
        let p1: Vec<&str> = coords[0].split(',').collect();
        let p2: Vec<&str> = coords[1].split(',').collect();
        if p1.len() == 2 && p2.len() == 2 {
          svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}"{}/>"#,
            p1[0], p1[1], p2[0], p2[1], rest_attrs
          ));
        }
      }
    }

    // <circle cx="..." cy="..." r="..." .../>
    for circle_caps in CIRCLE_RE.captures_iter(g_content) {
      svg.push_str(&format!("<circle{}/>", &circle_caps[1]));
    }

    // <ellipse cx="..." cy="..." rx="..." ry="..." .../>
    for ellipse_caps in ELLIPSE_RE.captures_iter(g_content) {
      svg.push_str(&format!("<ellipse{}/>", &ellipse_caps[1]));
    }

    // <rect x="..." y="..." width="..." height="..." .../>
    for rect_caps in RECT_RE.captures_iter(g_content) {
      svg.push_str(&format!("<rect{}/>", &rect_caps[1]));
    }

    // <polygon points="..." .../>
    for polygon_caps in POLYGON_RE.captures_iter(g_content) {
      svg.push_str(&format!("<polygon{}/>", &polygon_caps[1]));
    }

    // <path d="..." .../>
    for path_caps in PATH_RE.captures_iter(g_content) {
      svg.push_str(&format!("<path{}/>", &path_caps[1]));
    }

    // <bezier points="x1,y1 x2,y2 x3,y3 x4,y4" .../>
    // Convert to SVG cubic bezier path
    for bez_caps in BEZIER_RE.captures_iter(g_content) {
      let points = &bez_caps[1];
      let rest = &bez_caps[2];
      let coords: Vec<&str> = points.split_whitespace().collect();
      if coords.len() >= 4 {
        // SVG cubic bezier: M x0,y0 C x1,y1 x2,y2 x3,y3
        let d = format!(
          "M {} C {} {} {}",
          coords[0], coords[1], coords[2], coords[3]
        );
        svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
      } else if coords.len() >= 3 {
        // Quadratic bezier: M x0,y0 Q x1,y1 x2,y2
        let d = format!("M {} Q {} {}", coords[0], coords[1], coords[2]);
        svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
      }
    }

    // <arc .../> — arc segments (rarely used, stub for now)
    // <wedge .../> — filled wedges (rarely used, stub for now)

    // <text>...</text> — wrap in SVG text with y-flip correction
    for text_caps in TEXT_RE.captures_iter(g_content) {
      let text_attrs = &text_caps[1];
      let text_content = &text_caps[2];
      svg.push_str(&format!(
        r#"<g transform="scale(1,-1)"><text{text_attrs}>{text_content}</text></g>"#,
      ));
    }

    svg.push_str("</g>");
  }

  // Also handle direct children not inside <g> (e.g. top-level <bezier>, <line>)
  // These appear directly inside <picture> without a <g> wrapper
  let direct_bezier_re =
    regex::Regex::new(r#"(?m)^\s*<bezier\s+points="([^"]+)"([^/]*)/?>"#).unwrap();
  for bez_caps in direct_bezier_re.captures_iter(content) {
    let points = &bez_caps[1];
    let rest = &bez_caps[2];
    let coords: Vec<&str> = points.split_whitespace().collect();
    if coords.len() >= 4 {
      let d = format!(
        "M {} C {} {} {}",
        coords[0], coords[1], coords[2], coords[3]
      );
      svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
    } else if coords.len() >= 3 {
      let d = format!("M {} Q {} {}", coords[0], coords[1], coords[2]);
      svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
    }
  }

  svg
}

/// Parse a TeX dimension string (e.g. "100.0pt") to pixels.
fn parse_tex_dim(s: &str) -> Option<f64> {
  let s = s.trim();
  if let Some(rest) = s.strip_suffix("pt") {
    rest.parse::<f64>().ok().map(|v| v * 96.0 / 72.27)
  } else if let Some(rest) = s.strip_suffix("px") {
    rest.parse::<f64>().ok()
  } else {
    s.parse::<f64>().ok()
  }
}
