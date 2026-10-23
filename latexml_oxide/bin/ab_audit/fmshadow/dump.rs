//! The dump's JSON back into the resolver's inputs (`--re-resolve`) and its tuples (`ab_audit fmshadow`, OD #837).

use super::*;

pub(super) fn s(v: &Value) -> String { v.as_str().unwrap_or("").to_string() }

/// A dump event as the lexer's event.
pub fn json_event(v: &Value) -> FmEvent {
  let kind = match v["kind"].as_str().unwrap_or("") {
    "Name" => FmEventKind::Name,
    "Mark" => FmEventKind::Mark,
    "Break" => FmEventKind::Break,
    "Contact" => FmEventKind::Contact,
    "Note" => FmEventKind::Note,
    "ApiCall" => FmEventKind::ApiCall,
    "Sep" => FmEventKind::Sep(match v["sep"].as_str().unwrap_or("") {
      "and" => FmSepKind::And(match v["command"].as_str() {
        Some("\\And") => "\\And",
        Some("\\AND") => "\\AND",
        _ => "\\and",
      }),
      "column" => FmSepKind::Column("&"),
      "comma" => FmSepKind::Comma,
      "word" => FmSepKind::Word(s(&v["word"])),
      _ => FmSepKind::Gap {
        items: v["items"].as_u64().unwrap_or(0) as usize,
        width: v["width"].as_f64().unwrap_or(0.0),
        fill:  v["fill"].as_bool().unwrap_or(false),
      },
    }),
    _ => FmEventKind::Layout,
  };
  let mut event = FmEvent::new(kind);
  event.span = v["span"].as_array().and_then(|a| {
    Some((
      u16::try_from(a.first()?.as_u64()?).ok()?,
      u16::try_from(a.get(1)?.as_u64()?).ok()?,
    ))
  });
  event.text = s(&v["text"]);
  event.label = v["label"].as_str().map(str::to_string);
  event.symbol = v["symbol"].as_bool().unwrap_or(false);
  event.line_first = v["line_first"].as_bool().unwrap_or(false);
  event.role = v["role"].as_str().map(str::to_string);
  event.target = v["target"].as_str().map(str::to_string);
  event.nest = v["nest"].as_u64().unwrap_or(0) as u8;
  event.inner = v["inner"]
    .as_array()
    .map(|a| a.iter().map(json_event).collect())
    .unwrap_or_default();
  event
}

/// The resolver's inputs from a dump (as `resolver::ledger_inputs` makes them from the ledger).
pub fn json_inputs(dump: &Value) -> Vec<FmInput> {
  let mut inputs = Vec::new();
  for e in dump["entries"].as_array().into_iter().flatten() {
    match e["kind"].as_str() {
      Some("author") if e["status"] == "events" => inputs.push(FmInput::Block {
        index:   e["index"].as_u64().unwrap_or(0) as usize,
        binding: s(&e["binding"]),
        hook:    s(&e["hook"]),
        keyvals: e["keyvals"].as_str().map(str::to_string),
        counter: e["counter"].as_i64(),
        replace: e["replace"].as_bool().unwrap_or(false),
        epoch:   e["epoch"].as_u64().unwrap_or(0) as usize,
        events:  e["events"]
          .as_array()
          .into_iter()
          .flatten()
          .filter(|ev| ev["kind"] != "ApiCall")
          .map(json_event)
          .collect(),
      }),
      Some("api")
        if e["inside"] == false
          && fm_creator_call(
            e["tag"].as_str().unwrap_or(""),
            e["parent"].as_str().unwrap_or(""),
          ) =>
      {
        inputs.push(FmInput::Api {
          index:  e["index"].as_u64().unwrap_or(0) as usize,
          tag:    s(&e["tag"]),
          role:   s(&e["role"]),
          attrs:  s(&e["attrs"]),
          events: e["events"]
            .as_array()
            .into_iter()
            .flatten()
            .map(json_event)
            .collect(),
        })
      },
      Some("notetext") => inputs.push(FmInput::NoteText {
        index:  e["index"].as_u64().unwrap_or(0) as usize,
        epoch:  e["epoch"].as_u64().unwrap_or(0) as usize,
        mark:   s(&e["mark"]),
        events: e["events"]
          .as_array()
          .into_iter()
          .flatten()
          .map(json_event)
          .collect(),
      }),
      _ => {},
    }
  }
  inputs
}

/// A dump's `resolved` section as the resolver's value.
pub fn json_resolved(v: &Value) -> FmResolved {
  let assoc = |a: &Value| FmAssoc {
    role:   s(&a["role"]),
    text:   s(&a["text"]),
    label:  a["label"].as_str().map(str::to_string),
    rule:   s(&a["rule"]),
    source: None,
    target: a["target"].as_str().map(str::to_string),
  };
  let assocs = |a: &Value| {
    a.as_array()
      .into_iter()
      .flatten()
      .map(assoc)
      .collect::<Vec<_>>()
  };
  let mut out = FmResolved::default();
  for c in v["creators"].as_array().into_iter().flatten() {
    out
      .creators
      .push(latexml_engine::base_utilities::FmCreator {
        name:         s(&c["name"]),
        requests:     c["requests"]
          .as_array()
          .into_iter()
          .flatten()
          .map(s)
          .collect(),
        rule:         s(&c["rule"]),
        source:       c["source"].as_u64().unwrap_or(0) as usize,
        affiliations: assocs(&c["affiliations"]),
        contacts:     assocs(&c["contacts"]),
        notes:        assocs(&c["notes"]),
      });
  }
  out.block = assocs(&v["block"]);
  out.dropped = assocs(&v["dropped"]);
  // (a dump made before FM S2a.2 r8 has no `continued`: `None`, not an r8 dump's empty list — review revfms2a2_8
  // finding 4)
  out.continued = v.get("continued").and_then(Value::as_array).map(|all| {
    all
      .iter()
      .map(|lines| lines.as_array().into_iter().flatten().map(s).collect())
      .collect()
  });
  out
}
