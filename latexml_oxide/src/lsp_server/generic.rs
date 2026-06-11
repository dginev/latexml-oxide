//! Non-Unix server: simple blocking in-process loop (no fork, no
//! preemption). Functional fallback so the binary works everywhere;
//! the performance model lives in `unix.rs`.

use std::io::{BufRead, Read};

use serde_json::Value;

use super::*;

fn read_message(reader: &mut impl BufRead) -> Option<String> {
  let mut content_length = 0usize;
  let mut header = String::new();
  loop {
    header.clear();
    match reader.read_line(&mut header) {
      Ok(0) => return None,
      Ok(_) => {},
      Err(_) => return None,
    }
    let trimmed = header.trim_end();
    if trimmed.is_empty() {
      break;
    }
    if trimmed.to_lowercase().starts_with("content-length:") {
      if let Some(v) = trimmed.split(':').nth(1) {
        if let Ok(n) = v.trim().parse::<usize>() {
          content_length = n;
        }
      }
    }
  }
  if content_length == 0 {
    return Some(String::new());
  }
  let mut body = vec![0u8; content_length];
  if reader.read_exact(&mut body).is_err() {
    return None;
  }
  Some(String::from_utf8_lossy(&body).into_owned())
}

pub fn run(timeout_secs: u64, max_rss_kb: u64) -> Result<(), Box<dyn std::error::Error>> {
  let mut stdout = std::io::stdout();
  let mut server = Server::new(timeout_secs, max_rss_kb);
  let stdin = std::io::stdin();
  let mut reader = std::io::BufReader::new(stdin.lock());

  while let Some(body_str) = read_message(&mut reader) {
    if body_str.is_empty() {
      continue;
    }
    let request = match parse_json(&body_str) {
      Ok(v) => v,
      Err(e) => {
        log::error!("Failed to parse incoming JSON: {e}");
        continue;
      },
    };
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

    match method {
      "initialize" => {
        let caps = jobj(vec![(
          "capabilities",
          jobj(vec![("textDocumentSync", jnum(1.0))]),
        )]);
        send_message(&mut stdout, &response(id, caps))?;
      },
      "initialized" => {},
      "textDocument/didOpen" => {
        if let Some((uri, text)) = did_open_params(&request) {
          let (root_uri, out) = convert_root_blocking(&mut server, &uri, &text);
          publish_grouped_diagnostics(&mut server, &root_uri, &uri, &out, &mut stdout)?;
        }
      },
      "textDocument/didChange" => {
        if let Some((uri, text)) = did_change_params(&request) {
          let (root_uri, out) = convert_root_blocking(&mut server, &uri, &text);
          publish_grouped_diagnostics(&mut server, &root_uri, &uri, &out, &mut stdout)?;
        }
      },
      "textDocument/didClose" => {
        if let Some(uri) = request
          .get("params")
          .and_then(|p| p.get("textDocument"))
          .and_then(|d| d.get("uri"))
          .and_then(|u| u.as_str())
        {
          server.open_buffers.remove(&get_file_path(uri));
          send_message(&mut stdout, &publish_diagnostics_notification(uri, &[]))?;
        }
      },
      "shutdown" => {
        send_message(&mut stdout, &response(id, Value::Null))?;
      },
      "latexml/convert" => {
        if let (Some(uri), Some(text)) = (
          request
            .get("params")
            .and_then(|p| p.get("uri"))
            .and_then(|u| u.as_str()),
          request
            .get("params")
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str()),
        ) {
          let (_root_uri, out) = convert_root_blocking(&mut server, uri, text);
          send_message(&mut stdout, &response(id, out.to_result_object()))?;
        } else if id != Value::Null {
          // A request MUST be answered (see unix dispatch).
          send_message(
            &mut stdout,
            &error_response(
              id,
              -32602.0,
              "latexml/convert: missing params.uri/params.text".to_string(),
            ),
          )?;
        }
      },
      "exit" => return Ok(()),
      other => {
        if id != Value::Null {
          send_message(
            &mut stdout,
            &error_response(id, -32601.0, format!("Method '{other}' not found")),
          )?;
        }
      },
    }
  }
  Ok(())
}

/// Blocking (non-fork) variant of the multi-file conversion trigger: record
/// the buffer, resolve the project root, convert the ROOT in-process, and
/// attribute diagnostics. Returns `(root_uri, output)` with `output.root`
/// set.
fn convert_root_blocking(server: &mut Server, uri: &str, text: &str) -> (String, ConvertOutput) {
  let buffer_path = get_file_path(uri);
  server.upsert_buffer(buffer_path.clone(), text.to_string(), None);
  let override_root = server.root_override.clone();
  let root = resolve_root(
    &mut server.root_cache,
    override_root.as_deref(),
    std::path::Path::new(&buffer_path),
    Some(text),
  );
  let root_str = root.to_string_lossy().into_owned();
  let root_text = if root_str == buffer_path {
    text.to_string()
  } else if let Some(buf) = server.open_buffers.get(&root_str) {
    buf.text.clone()
  } else {
    std::fs::read_to_string(&root).unwrap_or_else(|_| text.to_string())
  };
  let root_uri = format!("file://{root_str}");
  let mut out = server.convert_in_process(&root_uri, &root_text);
  attribute_diag_files(&mut out.diags, &root, &server.open_buffers);
  out.root = Some(root_str);
  (root_uri, out)
}
