//! Document-side helpers: uri→path, comment-aware preamble/body
//! split, source-map decoder ring, post-processing to HTML5, engine
//! `Config`, and the dependency snapshot keying the warm cache.

use std::collections::BTreeMap;
use std::rc::Rc;

use latexml_core::common::{Config, DataSize, OutputFormat};

// ======================================================================
// URI / config / dependency helpers.
// ======================================================================

pub(crate) fn get_file_path(uri: &str) -> String {
  let s = uri.strip_prefix("file://").unwrap_or(uri);
  // Percent-decode at the BYTE level, then reassemble as UTF-8. Decoding
  // each %XX to `byte as char` would turn multi-byte sequences (e.g.
  // `%C3%A9` = é) into Latin-1 mojibake — wrong SOURCEDIRECTORY/search
  // paths for any non-ASCII document path.
  let bytes = s.as_bytes();
  let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == b'%' {
      if let Some(hex) = bytes.get(i + 1..i + 3) {
        if hex.is_ascii() {
          if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(hex).unwrap_or(""), 16) {
            decoded.push(byte);
            i += 3;
            continue;
          }
        }
      }
      decoded.push(b'%');
      i += 1;
    } else {
      decoded.push(bytes[i]);
      i += 1;
    }
  }
  String::from_utf8_lossy(&decoded).into_owned()
}

/// Byte ranges of `%`-comments in `text` (from each unescaped `%` to its line
/// end). A `%` is escaped only when immediately preceded by a backslash that
/// is itself a control-sequence escape (`\%`); after `\\` (the line-break
/// control word) a `%` DOES start a comment. Verbatim environments are not
/// modeled — a preamble `verbatim` containing `\begin{document}` is out of
/// scope.
fn comment_spans(text: &str) -> Vec<(usize, usize)> {
  let bytes = text.as_bytes();
  let mut spans = Vec::new();
  let mut i = 0;
  while i < bytes.len() {
    match bytes[i] {
      b'\\' => i += 2, // skip the escaped char (covers \% and \\ alike)
      b'%' => {
        let end = bytes[i..]
          .iter()
          .position(|&b| b == b'\n')
          .map(|p| i + p)
          .unwrap_or(bytes.len());
        spans.push((i, end));
        i = end + 1;
      },
      _ => i += 1,
    }
  }
  spans
}

/// Find the **effective** `\begin{document}` — the first regex match that is
/// not inside a `%`-comment. A commented-out `\begin{document}` (common in
/// templates) must not split the preamble: the old `regex.find` cut the text
/// at the commented occurrence, leaking the rest of the comment line into the
/// body as content.
pub(crate) fn find_begin_document(regex: &regex::Regex, text: &str) -> Option<(usize, usize)> {
  let spans = comment_spans(text);
  regex
    .find_iter(text)
    .find(|m| !spans.iter().any(|&(s, e)| s < m.start() && m.start() < e))
    .map(|m| (m.start(), m.end()))
}

/// Final path component (e.g. `main.tex`). The client lowercases for matching.
fn basename(path: &str) -> String {
  std::path::Path::new(path)
    .file_name()
    .and_then(|s| s.to_str())
    .map(String::from)
    .unwrap_or_else(|| path.to_string())
}

/// Resolve the source-map decoder ring: `sources[tag]` is the basename of the
/// file the integer `tag` (in each `data-sourcepos`) refers to. The main
/// buffer is digested as a `literal:` source named "Anonymous String"; map
/// that to the document's own basename so the client can resolve tag 0 back
/// to the active file. Other tags are real `\input`-ed files. Must be called
/// while the post-conversion thread-local state is still live.
pub(crate) fn collect_sources(uri: &str) -> Vec<String> {
  let self_base = basename(&get_file_path(uri));
  latexml_core::state::source_table_snapshot()
    .iter()
    .map(|sym| {
      let name = latexml_core::common::arena::with(*sym, |s| s.to_string());
      if name == "Anonymous String" {
        self_base.clone()
      } else {
        basename(&name)
      }
    })
    .collect()
}

/// Post-process the core ltx XML into HTML5 — the form the editor renders.
/// This runs the same pipeline the CLI and the ar5iv-editor server use
/// (`run_post_processing` with the embedded `LaTeXML-html5.xsl`), which turns
/// presentation MathML on and rewrites the source-map `data:sourcepos`
/// (foreign-namespaced, colon) attributes into the HTML `data-sourcepos`
/// (dash) the client decodes. Without this the server returned raw core XML.
pub(crate) fn post_process_html(core_xml: &str, uri: &str) -> String {
  let file_path = get_file_path(uri);
  let source_dir = std::path::Path::new(&file_path)
    .parent()
    .and_then(|p| p.to_str())
    .map(String::from);
  crate::post::run_post_processing(core_xml, &crate::post::PostOptions {
    pmml: true,
    cmml: false,
    keep_xmath: false,
    stylesheet: Some("resources/XSLT/LaTeXML-html5.xsl"),
    destination: None,
    source_directory: source_dir.as_deref(),
    // The server returns HTML as a string with no destination — it must never
    // write CSS/JS resource files to disk (would pollute the cwd). The client
    // supplies its own preview styling.
    nodefaultresources: true,
    css_files: &[],
    js_files: &[],
    noinvisibletimes: false,
    mathtex: false,
    navigationtoc: None,
    schemadocs: false,
    split: false,
    split_xpath: None,
    split_naming: None,
    xslt_parameters: &[],
    graphics_svg_threshold_kb: 0,
    whatsout: latexml_post::extract::Whatsout::Document,
  })
}

pub(crate) fn make_config(uri: &str) -> Config {
  let file_path = get_file_path(uri);
  let dir_path = std::path::Path::new(&file_path).parent();
  let mut search_paths = Vec::new();
  if let Some(parent) = dir_path {
    if let Some(p_str) = parent.to_str() {
      if !p_str.is_empty() {
        search_paths.push(p_str.to_string());
      }
    }
  }

  Config {
    verbosity: 0,
    format: OutputFormat::HTML5,
    whatsin: DataSize::Document,
    whatsout: DataSize::Document,
    preamble: None,
    postamble: None,
    mode: None,
    bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
    // Preload ar5iv.sty: this server backs the ar5iv-editor, and ar5iv.sty
    // enables raw `.sty` handling so a paper's *local, binding-less* packages
    // (e.g. a bundled `mystyle.sty`) load instead of being skipped with a
    // missing-file warning. Mirrors the sandbox/ar5iv conversion workflow.
    preload: Some(vec!["ar5iv.sty".to_string()]),
    search_paths: if search_paths.is_empty() {
      None
    } else {
      Some(search_paths)
    },
    include_comments: None,
    // Math parsing is always ON in the server: the parsed MathML (and its
    // source-mapped tokens) is one of the features this preview showcases, so
    // we deliberately do not expose a disable knob here.
    nomathparse: None,
    source_map: Some(true),
  }
}

pub(crate) fn get_directory_dependencies(uri: &str) -> BTreeMap<String, std::time::SystemTime> {
  let mut deps = BTreeMap::new();
  let file_path = get_file_path(uri);
  if let Some(parent) = std::path::Path::new(&file_path).parent() {
    if let Ok(entries) = std::fs::read_dir(parent) {
      for entry in entries.flatten() {
        let path = entry.path();
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
          continue;
        }
        let is_dep = path
          .extension()
          .and_then(|e| e.to_str())
          .map(|e| {
            matches!(
              e.to_lowercase().as_str(),
              "sty" | "cls" | "tex" | "cfg" | "def" | "bib" | "clo"
            )
          })
          .unwrap_or(false);
        if !is_dep {
          continue;
        }
        if let (Ok(metadata), Some(path_str)) = (entry.metadata(), path.to_str()) {
          if let Ok(mtime) = metadata.modified() {
            deps.insert(path_str.to_string(), mtime);
          }
        }
      }
    }
  }
  deps
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn basename_extraction() {
    assert_eq!(basename("/home/u/proj/main.tex"), "main.tex");
    assert_eq!(basename("main.tex"), "main.tex");
  }

  #[test]
  fn file_path_percent_decodes_utf8() {
    // %C3%A9 = é (two UTF-8 bytes) — the old char-per-byte decode produced
    // "Ã©" mojibake and broke SOURCEDIRECTORY for non-ASCII paths.
    assert_eq!(get_file_path("file:///home/u/caf%C3%A9/main.tex"), "/home/u/café/main.tex");
    assert_eq!(get_file_path("file:///plain/path.tex"), "/plain/path.tex");
    // Malformed escapes pass through unmangled.
    assert_eq!(get_file_path("file:///x%2/y%"), "/x%2/y%");
    assert_eq!(get_file_path("file:///sp%20ace.tex"), "/sp ace.tex");
  }

  #[test]
  fn begin_document_skips_commented_occurrences() {
    let re = regex::Regex::new(r"\\begin\s*\{\s*document\s*\}").unwrap();
    // Commented-out \begin{document} (template style) must not split there.
    let text = "\\documentclass{article}\n% \\begin{document} not yet!\n\\begin{document}\nBody\n\\end{document}\n";
    let (start, end) = find_begin_document(&re, text).unwrap();
    assert_eq!(&text[start..end], "\\begin{document}");
    assert!(text[..start].contains("not yet!"), "split is AFTER the commented line");
    // An escaped \% does not start a comment.
    let text2 = "100\\% sure\\begin{document}x";
    let (s2, _) = find_begin_document(&re, text2).unwrap();
    assert_eq!(s2, "100\\% sure".len());
    // All occurrences commented → no split (in-process fallback path).
    assert_eq!(find_begin_document(&re, "% \\begin{document}\n"), None);
  }
}
