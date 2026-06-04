//! Engine-log diagnostics: one parser, two output shapes (LSP
//! `Diagnostic` and the ar5iv-editor normalized form).

use serde_json::Value;

use super::*;

// ======================================================================
// Diagnostics — one parser, two output shapes.
// ======================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Severity {
  Error,
  Warning,
  Info,
  Fatal,
}

impl Severity {
  /// LSP `DiagnosticSeverity` numeric code.
  fn lsp_code(self) -> f64 {
    match self {
      Severity::Error | Severity::Fatal => 1.0,
      Severity::Warning => 2.0,
      Severity::Info => 3.0,
    }
  }

  /// ar5iv-editor normalized severity string.
  fn normalized(self) -> &'static str {
    match self {
      Severity::Error => "error",
      Severity::Warning => "warning",
      Severity::Info => "info",
      Severity::Fatal => "fatal",
    }
  }
}

/// A single parsed engine diagnostic. `line`/`col` are 1-based, file-relative
/// (the warm-fork path offsets body lines so this holds — see `run_warm`).
#[derive(Debug, Clone)]
pub(crate) struct Diag {
  pub(crate) severity: Severity,
  pub(crate) line:     Option<usize>,
  pub(crate) col:      Option<usize>,
  /// Optional range end (the locator can carry `- line N col M`).
  pub(crate) to_line:  Option<usize>,
  pub(crate) to_col:   Option<usize>,
  /// The locator's source name as logged (the mouth's SHORT name, e.g.
  /// `ch2` — not a path).
  pub(crate) source:   Option<String>,
  /// Absolute path this diagnostic was attributed to (multi-file model;
  /// `attribute_diag_files`). `None` = attach to the edited buffer.
  pub(crate) file:     Option<String>,
  pub(crate) message:  String,
}

impl Diag {
  pub(crate) fn new(severity: Severity, message: String) -> Self {
    Diag {
      severity,
      line: None,
      col: None,
      to_line: None,
      to_col: None,
      source: None,
      file: None,
      message,
    }
  }

  /// LSP `Diagnostic` (0-based positions).
  pub(crate) fn to_lsp(&self) -> Value {
    let line0 = self.line.map(|l| l.saturating_sub(1)).unwrap_or(0);
    let col0 = self.col.map(|c| c.saturating_sub(1)).unwrap_or(0);
    let start = jobj(vec![
      ("line", jnum(line0 as f64)),
      ("character", jnum(col0 as f64)),
    ]);
    // Range end from the locator when present; else a one-character caret
    // anchor (the message carries the detail).
    let end_line0 = self.to_line.map(|l| l.saturating_sub(1)).unwrap_or(line0);
    let end_col0 = match self.to_col {
      Some(c) if end_line0 > line0 => c.saturating_sub(1),
      Some(c) => c.saturating_sub(1).max(col0 + 1),
      None => col0 + 1,
    };
    let end = jobj(vec![
      ("line", jnum(end_line0 as f64)),
      ("character", jnum(end_col0 as f64)),
    ]);
    jobj(vec![
      ("range", jobj(vec![("start", start), ("end", end)])),
      ("severity", jnum(self.severity.lsp_code())),
      ("source", jstr("latexml")),
      ("message", jstr(self.message.clone())),
    ])
  }

  /// ar5iv-editor normalized diagnostic (1-based `from.{line,column}`).
  pub(crate) fn to_normalized(&self) -> Value {
    let mut pairs = vec![
      ("severity", jstr(self.severity.normalized())),
      ("category", jstr("latexml")),
      ("message", jstr(self.message.clone())),
    ];
    if let Some(line) = self.line {
      let mut from = vec![("line", jnum(line as f64))];
      if let Some(col) = self.col {
        from.push(("column", jnum(col as f64)));
      }
      pairs.push(("from", jobj(from)));
    }
    if let Some(ref file) = self.file {
      pairs.push(("file", jstr(file.clone())));
    }
    jobj(pairs)
  }
}

/// Parse one line/column pair out of a LaTeXML log message. Handles both the
/// `…; line N col M` and the bare `… line N` shapes.
pub(crate) fn parse_line_col(line: &str) -> (Option<usize>, Option<usize>) {
  let take_num = |s: &str| -> Option<usize> {
    let n: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    n.parse::<usize>().ok().filter(|&v| v > 0)
  };
  if let Some(idx) = line.find("; line ") {
    let rest = &line[idx + 7..];
    let l = take_num(rest);
    let c = rest.find(" col ").and_then(|ci| take_num(&rest[ci + 5..]));
    (l, c)
  } else if let Some(idx) = line.find("line ") {
    (take_num(&line[idx + 5..]), None)
  } else {
    (None, None)
  }
}

/// Record-based log parser. The engine's log format is:
///
/// ```text
/// {Severity}:{Category}:{Object} {message-first-line}
/// \tat {source}; line N col M[ - line N col M]
/// \t[detail line(s)]
/// \tIn {rust_file}:{line}:{col}
/// ```
///
/// Locators live on the tab-indented CONTINUATION line — the previous
/// line-by-line parser never saw them, so every diagnostic came back
/// position-less. (Same record shape ar5iv-editor's proven
/// `parse_diagnostics` consumes.) Inline `; line N col M` on the severity
/// line itself is still honored as a fallback.
pub(crate) fn parse_log_diagnostics(log_str: &str) -> Vec<Diag> {
  let mut out: Vec<Diag> = Vec::new();
  let mut current: Option<Diag> = None;
  for line in log_str.lines() {
    let severity = if line.starts_with("Error:") {
      Some(Severity::Error)
    } else if line.starts_with("Warn:") {
      Some(Severity::Warning)
    } else if line.starts_with("Fatal:") {
      Some(Severity::Fatal)
    } else if line.starts_with("Info:") {
      Some(Severity::Info)
    } else {
      None
    };
    if let Some(sev) = severity {
      if let Some(d) = current.take() {
        out.push(d);
      }
      let mut d = Diag::new(sev, line.to_string());
      // Inline-locator fallback (synthetic / single-line messages).
      let (l, c) = parse_line_col(line);
      d.line = l;
      d.col = c;
      current = Some(d);
      continue;
    }
    // Continuation lines are tab-indented.
    if let Some(rest) = line.strip_prefix('\t') {
      if let Some(d) = current.as_mut() {
        if rest.starts_with("In ") {
          continue; // internal Rust location
        }
        if let Some(loc) = rest.strip_prefix("at ") {
          fill_location(d, loc);
          continue;
        }
        if !rest.trim().is_empty() {
          d.message.push('\n');
          d.message.push_str(rest);
        }
        continue;
      }
    }
    // Non-continuation, non-severity line: closes any open record.
    if let Some(d) = current.take() {
      out.push(d);
    }
  }
  if let Some(d) = current.take() {
    out.push(d);
  }
  out
}

/// Parse the `at {source}; line N col M[ - line N col M]` locator payload.
fn fill_location(d: &mut Diag, loc: &str) {
  let (source, rest) = match loc.find(';') {
    Some(i) => (loc[..i].trim(), &loc[i + 1..]),
    None => (loc.trim(), ""),
  };
  if !source.is_empty() {
    d.source = Some(source.to_string());
  }
  let mut segments = rest.split('-').map(str::trim);
  if let Some(from) = segments.next() {
    let (l, c) = parse_line_col_words(from);
    if l.is_some() {
      d.line = l;
      d.col = c;
    }
  }
  if let Some(to) = segments.next() {
    let (l, c) = parse_line_col_words(to);
    d.to_line = l;
    d.to_col = c;
  }
}

/// `"line N col M"` / `"line N"` word-pair parse.
fn parse_line_col_words(seg: &str) -> (Option<usize>, Option<usize>) {
  let (mut line, mut col) = (None, None);
  let mut tokens = seg.split_ascii_whitespace();
  while let Some(tok) = tokens.next() {
    match tok {
      "line" => line = tokens.next().and_then(|n| n.parse().ok()),
      "col" => col = tokens.next().and_then(|n| n.parse().ok()),
      _ => {},
    }
  }
  (line, col)
}

/// Attribute each diagnostic to an absolute project file from its locator
/// `source` (the mouth's SHORT name, e.g. `ch2`): match open buffers, then
/// `<project-dir>/**` direct candidates (`<src>`, `<src>.tex`). Ambiguous or
/// unmatched sources stay `None` (the caller attaches them to the edited
/// buffer's uri).
pub(crate) fn attribute_diag_files(
  diags: &mut [Diag],
  root: &std::path::Path,
  buffers: &rustc_hash::FxHashMap<String, Buffer>,
) {
  let dir = project_dir(root);
  for d in diags.iter_mut() {
    let Some(ref src) = d.source else { continue };
    // 1. Unique open-buffer stem/basename match.
    let mut matches = buffers.keys().filter(|path| {
      let p = std::path::Path::new(path.as_str());
      p.file_stem().and_then(|s| s.to_str()) == Some(src.as_str())
        || p.file_name().and_then(|s| s.to_str()) == Some(src.as_str())
    });
    if let (Some(only), None) = (matches.next(), matches.next()) {
      d.file = Some(only.clone());
      continue;
    }
    // 2. Direct project-dir candidates.
    for candidate in [dir.join(src.as_str()), dir.join(format!("{src}.tex"))] {
      if candidate.is_file() {
        d.file = Some(candidate.to_string_lossy().into_owned());
        break;
      }
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_line_col_variants() {
    assert_eq!(parse_line_col("Error:foo:bar baz; line 12 col 7"), (Some(12), Some(7)));
    assert_eq!(parse_line_col("Warn:foo bar at line 3"), (Some(3), None));
    assert_eq!(parse_line_col("Info:foo no position here"), (None, None));
  }

  #[test]
  fn record_parser_reads_continuation_locators() {
    // The REAL engine log shape (probe: /tmp/mf undefined-macro run):
    // locator on a tab-indented continuation line, with a range and a
    // trailing internal-Rust-location line that must be skipped.
    let log = "Error:undefined:\\foo The token is not defined.\n\
               \tat ch2; line 3 col 46 - line 3 col 52\n\
               \t\n\
               \tIn latexml_core/src/state.rs:1165:7\n\
               (Loading something else)\n\
               Warn:unexpected:x lone warning, no locator\n";
    let diags = parse_log_diagnostics(log);
    assert_eq!(diags.len(), 2);
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.source.as_deref(), Some("ch2"));
    assert_eq!((d.line, d.col), (Some(3), Some(46)));
    assert_eq!((d.to_line, d.to_col), (Some(3), Some(52)));
    assert!(!d.message.contains("state.rs"), "internal Rust loc skipped");
    assert_eq!(diags[1].severity, Severity::Warning);
    assert_eq!(diags[1].line, None);
  }

  #[test]
  fn attribution_maps_short_source_to_project_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("main.tex");
    std::fs::write(&root, "x").unwrap();
    let on_disk = tmp.path().join("appendix.tex");
    std::fs::write(&on_disk, "y").unwrap();

    let mut buffers: rustc_hash::FxHashMap<String, Buffer> = rustc_hash::FxHashMap::default();
    buffers.insert("/proj/sections/ch2.tex".to_string(), Buffer {
      version: 1,
      text:    String::new(),
    });

    let mut diags = vec![
      Diag::new(Severity::Error, "e1".into()),
      Diag::new(Severity::Error, "e2".into()),
      Diag::new(Severity::Error, "e3".into()),
    ];
    diags[0].source = Some("ch2".into()); // open buffer, by stem
    diags[1].source = Some("appendix".into()); // disk sibling of the root
    diags[2].source = Some("nowhere".into()); // unattributable

    attribute_diag_files(&mut diags, &root, &buffers);
    assert_eq!(diags[0].file.as_deref(), Some("/proj/sections/ch2.tex"));
    assert_eq!(
      diags[1].file.as_deref(),
      on_disk.to_str(),
      "disk candidate <dir>/<src>.tex"
    );
    assert_eq!(diags[2].file, None, "unmatched stays None (edited-buffer uri)");
  }

  #[test]
  fn diagnostics_severity_and_zero_basing() {
    let log = "Error:x:y problem; line 5 col 2\nWarn:p:q issue at line 9\nrandom line\nInfo:i:j note line 1";
    let diags = parse_log_diagnostics(log);
    assert_eq!(diags.len(), 3);
    assert_eq!(diags[0].severity, Severity::Error);
    // LSP is 0-based: line 5 → 4, col 2 → 1.
    let lsp = diags[0].to_lsp();
    let range = lsp.get("range").unwrap();
    let start = range.get("start").unwrap();
    assert_eq!(start.get("line"), Some(&jnum(4.0)));
    assert_eq!(start.get("character"), Some(&jnum(1.0)));
    assert_eq!(lsp.get("severity"), Some(&jnum(1.0)));
    // Normalized keeps 1-based.
    let norm = diags[0].to_normalized();
    let from = norm.get("from").unwrap();
    assert_eq!(from.get("line"), Some(&jnum(5.0)));
    assert_eq!(from.get("column"), Some(&jnum(2.0)));
    assert_eq!(norm.get("severity"), Some(&Value::String("error".to_string())));
  }
}
