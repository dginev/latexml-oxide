//! JSON-RPC / LSP message shaping: result objects, responses,
//! notifications, framing, and the shared conversion-trigger
//! extraction used for preemption + pending-queue coalescing.

use std::collections::VecDeque;

use serde_json::Value;

use super::*;

// ======================================================================
// Conversion output + result/notification builders.
// ======================================================================

/// The platform-independent result of one conversion, before it is shaped
/// into either a `latexml/convert` result object or a `publishDiagnostics`
/// notification.
pub(crate) struct ConvertOutput {
  pub(crate) html:    String,
  pub(crate) log:     String,
  pub(crate) diags:   Vec<Diag>,
  pub(crate) sources: Vec<String>,
  /// Human-facing status label (the engine's status message, or `"timeout"`).
  pub(crate) status:  String,
  /// Engine status code: 0 = no problem, 1 = warning, 2 = error, 3 = fatal.
  pub(crate) status_code: i64,
}

/// Default label for a status code, when the engine message isn't carried.
pub(crate) fn status_label(code: i64) -> &'static str {
  match code {
    0 => "ok",
    1 => "warning",
    2 => "error",
    _ => "fatal",
  }
}

impl ConvertOutput {
  /// A failed conversion carrying a `status` label, a `status_code`
  /// (0/1/2/3), and a single Fatal diagnostic with `message`.
  pub(crate) fn failed(status: &str, status_code: i64, message: String) -> Self {
    ConvertOutput {
      html: String::new(),
      log: message.clone(),
      diags: vec![Diag {
        severity: Severity::Fatal,
        line: None,
        col: None,
        message,
      }],
      sources: Vec::new(),
      status: status.to_string(),
      status_code,
    }
  }

  /// A hard/fatal failure (status code 3).
  pub(crate) fn error(message: String) -> Self { Self::failed("fatal", 3, message) }

  /// The `latexml/convert` result object the ar5iv-editor client consumes.
  pub(crate) fn to_result_object(&self) -> Value {
    jobj(vec![
      ("html", jstr(self.html.clone())),
      ("log", jstr(self.log.clone())),
      (
        "diagnostics",
        Value::Array(self.diags.iter().map(Diag::to_normalized).collect()),
      ),
      (
        "sources",
        Value::Array(self.sources.iter().map(|s| jstr(s.clone())).collect()),
      ),
      ("status", jstr(self.status.clone())),
      ("statusCode", Value::from(self.status_code)),
    ])
  }
}

pub(crate) fn response(id: Value, result: Value) -> Value {
  jobj(vec![
    ("jsonrpc", jstr("2.0")),
    ("id", id),
    ("result", result),
  ])
}

pub(crate) fn error_response(id: Value, code: f64, message: String) -> Value {
  jobj(vec![
    ("jsonrpc", jstr("2.0")),
    ("id", id),
    (
      "error",
      jobj(vec![("code", jnum(code)), ("message", jstr(message))]),
    ),
  ])
}

pub(crate) fn cancelled_result_object() -> Value {
  jobj(vec![
    ("html", jstr("")),
    ("log", jstr("Request cancelled")),
    ("diagnostics", Value::Array(Vec::new())),
    ("sources", Value::Array(Vec::new())),
    ("status", jstr("cancelled")),
    // Integer, matching the int statusCode every other result carries.
    ("statusCode", Value::from(0i64)),
  ])
}

pub(crate) fn publish_diagnostics_notification(uri: &str, diags: &[Diag]) -> Value {
  jobj(vec![
    ("jsonrpc", jstr("2.0")),
    ("method", jstr("textDocument/publishDiagnostics")),
    (
      "params",
      jobj(vec![
        ("uri", jstr(uri)),
        (
          "diagnostics",
          Value::Array(diags.iter().map(Diag::to_lsp).collect()),
        ),
      ]),
    ),
  ])
}

pub(crate) fn send_message(writer: &mut impl std::io::Write, val: &Value) -> std::io::Result<()> {
  let body = val.to_string();
  let msg = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
  writer.write_all(msg.as_bytes())?;
  writer.flush()?;
  Ok(())
}


/// The document uri of a *conversion-triggering* message — `latexml/convert`
/// (`params.uri`) or `textDocument/didOpen`/`didChange`
/// (`params.textDocument.uri`). `None` for every other method. Used both for
/// preemption (a newer trigger for the same doc supersedes the in-flight
/// child) and for pending-queue coalescing (only the newest trigger per doc
/// runs).
pub(crate) fn message_doc_uri(request: &Value) -> Option<String> {
  match request.get("method").and_then(|m| m.as_str()) {
    Some("latexml/convert") => request
      .get("params")
      .and_then(|p| p.get("uri"))
      .and_then(|u| u.as_str())
      .map(String::from),
    Some("textDocument/didOpen") | Some("textDocument/didChange") => request
      .get("params")
      .and_then(|p| p.get("textDocument"))
      .and_then(|d| d.get("uri"))
      .and_then(|u| u.as_str())
      .map(String::from),
    _ => None,
  }
}

/// Is a newer conversion trigger for `uri` already waiting in `pending`?
/// (Everything in `pending` arrived AFTER the message currently being
/// dispatched, so any match supersedes it.) Running the current conversion
/// anyway would serialize a stale compile in front of the fresh one — the
/// didChange-flurry snowball.
pub(crate) fn superseded_in_pending(pending: &VecDeque<String>, uri: &str) -> bool {
  pending.iter().any(|body| {
    parse_json(body)
      .ok()
      .and_then(|req| message_doc_uri(&req))
      .as_deref()
      == Some(uri)
  })
}

pub(crate) fn did_open_params(request: &Value) -> Option<(String, String)> {
  let td = request.get("params")?.get("textDocument")?;
  let uri = td.get("uri")?.as_str()?.to_string();
  let text = td.get("text")?.as_str()?.to_string();
  Some((uri, text))
}

pub(crate) fn did_change_params(request: &Value) -> Option<(String, String)> {
  let params = request.get("params")?;
  let uri = params
    .get("textDocument")?
    .get("uri")?
    .as_str()?
    .to_string();
  let changes = match params.get("contentChanges")? {
    Value::Array(c) => c,
    _ => return None,
  };
  let text = changes.first()?.get("text")?.as_str()?.to_string();
  Some((uri, text))
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cancelled_object_shape() {
    let v = cancelled_result_object();
    assert_eq!(v.get("status"), Some(&Value::String("cancelled".to_string())));
    assert_eq!(v.get("html"), Some(&Value::String(String::new())));
    // Integer statusCode, consistent with every other result object.
    assert_eq!(v.get("statusCode").and_then(Value::as_i64), Some(0));
  }

  #[test]
  fn message_doc_uri_extraction_and_coalescing() {
    let conv = r#"{"method":"latexml/convert","params":{"uri":"file:///a.tex","text":"x"}}"#;
    let chg = r#"{"method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///a.tex"},"contentChanges":[{"text":"y"}]}}"#;
    let close = r#"{"method":"textDocument/didClose","params":{"textDocument":{"uri":"file:///a.tex"}}}"#;
    assert_eq!(
      message_doc_uri(&parse_json(conv).unwrap()).as_deref(),
      Some("file:///a.tex")
    );
    assert_eq!(
      message_doc_uri(&parse_json(chg).unwrap()).as_deref(),
      Some("file:///a.tex")
    );
    assert_eq!(message_doc_uri(&parse_json(close).unwrap()), None);

    let mut pending = VecDeque::new();
    pending.push_back(close.to_string());
    assert!(!superseded_in_pending(&pending, "file:///a.tex"), "didClose does not supersede");
    pending.push_back(chg.to_string());
    assert!(superseded_in_pending(&pending, "file:///a.tex"), "queued didChange supersedes");
    assert!(!superseded_in_pending(&pending, "file:///b.tex"), "other docs unaffected");
  }
}
