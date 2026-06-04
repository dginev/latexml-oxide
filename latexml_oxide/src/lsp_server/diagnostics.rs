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
  pub(crate) message:  String,
}

impl Diag {
  /// LSP `Diagnostic` (0-based positions).
  pub(crate) fn to_lsp(&self) -> Value {
    let line0 = self.line.map(|l| l.saturating_sub(1)).unwrap_or(0);
    let col0 = self.col.map(|c| c.saturating_sub(1)).unwrap_or(0);
    let start = jobj(vec![
      ("line", jnum(line0 as f64)),
      ("character", jnum(col0 as f64)),
    ]);
    // One-character caret anchor; the message carries the detail.
    let end = jobj(vec![
      ("line", jnum(line0 as f64)),
      ("character", jnum((col0 + 1) as f64)),
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

pub(crate) fn parse_log_diagnostics(log_str: &str) -> Vec<Diag> {
  let mut diagnostics = Vec::new();
  for line in log_str.lines() {
    let severity = if line.starts_with("Error:") {
      Severity::Error
    } else if line.starts_with("Warn:") {
      Severity::Warning
    } else if line.starts_with("Fatal:") {
      Severity::Fatal
    } else if line.starts_with("Info:") {
      Severity::Info
    } else {
      continue;
    };
    let (l, c) = parse_line_col(line);
    diagnostics.push(Diag {
      severity,
      line: l,
      col: c,
      message: line.to_string(),
    });
  }
  diagnostics
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
