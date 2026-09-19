//! datatool.sty — the raw v3 package plus a native `\DTLloaddb` (CSV load).
//!
//! The raw file loads whole; only the default CSV load is native
//! (`docs/performance/PERFORMANCE.md` "Native datatool load — the design"):
//! `\DTLloaddb[]{name}{file}` = `\DTLread[name={name},format=csv,csv-content=tex]`
//! (datatool.sty:12798), whose l3regex line split (:12730-12790) and
//! per-item store are the whole 5.5× of tikz-network against pdflatex. A
//! loaded database is four global registers and one index macro per key,
//! and every reader is a delimited-macro consumer of them (:3465-3530,
//! :6843, :6932): `\dtldb@<name>` (one row body per row), `\dtlkeys@<name>`
//! (one column body per column), `\dtlrows@`/`\dtlcols@` (ints) and
//! `\dtl@ci@<name>@<key>` (a column index) — so a load that writes
//! byte-identical register contents, the way datatool's own reload
//! (`\@dtl@reconstruct@data`, :12073-12093) does, is transparent to every
//! getter. Any option on the call, an existing database of that name, or a
//! file the resolver cannot find hands the call to the raw `\DTLread`.
//!
//! `LATEXML_DATATOOL_NATIVE=0` keeps the raw loader (the differential harness
//! converts both ways and requires byte-identical core XML).
use std::cell::LazyCell;

use latexml_core::mouth::Mouth;

use crate::prelude::*;

/// The datum types of datatool-base.sty:2279-2286 that the default CSV load
/// can produce (no currency setup, no temporal parsing).
const T_UNKNOWN: i32 = -1;
const T_STRING: i32 = 0;
const T_INTEGER: i32 = 1;
const T_DECIMAL: i32 = 2;

/// `\__datatool_parse_datum:n` (datatool-base.sty:3489) under the default
/// locale (number group `,`, decimal `.`): the scientific form is decimal;
/// an integer of ten or more digits whose tenth-from-last digit is 2-9
/// overflows to string (`\l_datatool_locale_bigint_regex`, :2180); the
/// locale numeric form with optional thousands groups and an optional
/// fraction, or the two fractional forms, are integer/decimal by the
/// presence of a fraction (:2188-2200, `\__datatool_parse_number:w` :3782);
/// anything else is a string; an empty value is unknown (`\tl_if_blank`).
fn datum_type(value: &str) -> i32 {
  thread_local! {
    static SCI: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^[+\-]?[0-9]*\.?[0-9]+\s*[eE][+\-]?[0-9]+$").unwrap());
    static BIGINT: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^[+\-]?[0-9]*[2-9](?:,?[0-9]{3}){3}$").unwrap());
    static NUMERIC: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^[+\-]?[0-9]+(?:,[0-9]{3}){0,3}(\.[0-9]+)?$").unwrap());
    static FRACTIONAL: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^[+\-]?[0-9]*\.[0-9]+$").unwrap());
  }
  if value.is_empty() {
    return T_UNKNOWN;
  }
  if SCI.with(|r| r.is_match(value)) {
    return T_DECIMAL;
  }
  if BIGINT.with(|r| r.is_match(value)) {
    return T_STRING;
  }
  if let Some(c) = NUMERIC.with(|r| r.captures(value)) {
    return if c.get(1).is_some() {
      T_DECIMAL
    } else {
      T_INTEGER
    };
  }
  if FRACTIONAL.with(|r| r.is_match(value)) {
    return T_DECIMAL;
  }
  T_STRING
}

fn is_other(t: &Token, c: char) -> bool {
  t.get_catcode() == Catcode::OTHER && t.with_str(|s| s.len() == c.len_utf8() && s.starts_with(c))
}

fn is_space(t: &Token) -> bool { t.get_catcode() == Catcode::SPACE }

/// `\tl_trim_spaces` (`\@dtl@setnewvalue`, datatool.sty:196).
fn trim_spaces(v: &[Token]) -> &[Token] {
  let start = v.iter().position(|t| !is_space(t)).unwrap_or(v.len());
  let end = v
    .iter()
    .rposition(|t| !is_space(t))
    .map(|i| i + 1)
    .unwrap_or(start);
  &v[start..end.max(start)]
}

/// One line's tokens, read under the document's catcodes with the
/// separator, the delimiter and `%` as OTHER (datatool.sty:12266-12268).
fn line_tokens(line: &str) -> Result<Vec<Token>> {
  push_frame();
  assign_catcode(',', Catcode::OTHER, None);
  assign_catcode('"', Catcode::OTHER, None);
  assign_catcode('%', Catcode::OTHER, None);
  let toks = Mouth::new(line, None)?.read_tokens().unlist();
  pop_frame()?;
  Ok(toks)
}

/// `\__datatool_split_line:` + `\__datatool_map_row_seq:` (:12730-12790):
/// the line split at every brace-depth-0 separator (`\seq_set_split_keep_spaces`),
/// then the pieces walked — a piece that is `\s*"…"\s*` is one item without
/// its quotes; a piece that starts with `\s*"` opens an item that swallows
/// the following separators until a piece ending in `"\s*`; every other
/// piece is an item (doubled quotes are NOT collapsed under
/// `csv-content=tex`, a closing quote alone stays literal, and an item still
/// open at the end of the line is taken as is). Each item is then trimmed.
fn split_fields(toks: &[Token]) -> Vec<Vec<Token>> {
  let mut pieces: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0;
  for t in toks {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && is_other(t, ',') {
      pieces.push(Vec::new());
    } else {
      pieces.last_mut().unwrap().push(*t);
    }
  }
  let starts_quoted = |p: &[Token]| {
    p.iter()
      .position(|t| !is_space(t))
      .is_some_and(|i| is_other(&p[i], '"'))
  };
  let ends_quoted = |p: &[Token]| {
    p.iter()
      .rposition(|t| !is_space(t))
      .is_some_and(|i| is_other(&p[i], '"'))
  };
  let mut items: Vec<Vec<Token>> = Vec::new();
  let mut open: Option<Vec<Token>> = None;
  for piece in pieces {
    match open.take() {
      None => {
        if starts_quoted(&piece) {
          let l = piece.iter().position(|t| !is_space(t)).unwrap();
          if ends_quoted(&piece) && piece.iter().rposition(|t| !is_space(t)).unwrap() > l {
            let r = piece.iter().rposition(|t| !is_space(t)).unwrap();
            items.push(piece[l + 1..r].to_vec());
          } else {
            open = Some(piece[l + 1..].to_vec());
          }
        } else {
          items.push(piece);
        }
      },
      Some(mut acc) => {
        acc.push(Token::new(",", Catcode::OTHER));
        if ends_quoted(&piece) {
          let r = piece.iter().rposition(|t| !is_space(t)).unwrap();
          acc.extend_from_slice(&piece[..r]);
          items.push(acc);
        } else {
          acc.extend(piece);
          open = Some(acc);
        }
      },
    }
  }
  if let Some(acc) = open {
    items.push(acc);
  }
  items
    .into_iter()
    .map(|v| trim_spaces(&v).to_vec())
    .collect()
}

/// The characters of `s` as OTHER-catcode tokens.
fn other_chars(s: &str) -> impl Iterator<Item = Token> + '_ {
  s.chars().map(|c| Token::new(c.to_string(), Catcode::OTHER))
}

fn digits(n: usize) -> Vec<Token> {
  n.to_string()
    .chars()
    .map(|c| Token::new(c.to_string(), Catcode::OTHER))
    .collect()
}

/// `\tl_to_str` of a key: every character OTHER, spaces spaces.
fn str_tokens(v: &[Token]) -> Vec<Token> {
  let text = Tokens::new(v.to_vec()).to_string();
  text
    .chars()
    .map(|c| {
      Token::new(
        c.to_string(),
        if c == ' ' {
          Catcode::SPACE
        } else {
          Catcode::OTHER
        },
      )
    })
    .collect()
}

fn wrap(open: &str, body: &[Token], close: &str) -> Vec<Token> {
  let mut out = vec![T_CS!(open)];
  out.extend_from_slice(body);
  out.push(T_CS!(close));
  out
}

/// The whole default CSV load, natively. `None` = hand the call to the raw
/// `\DTLread` (the reason traced when `LATEXML_DATATOOL_TRACE` is set).
fn load_csv(name: &str, file: &str) -> Result<Option<()>> {
  let fname = if std::path::Path::new(file).extension().is_some() {
    file.to_string()
  } else {
    format!("{file}.csv")
  };
  let Some(path) = find_file(&fname, None) else {
    return Ok(None);
  };
  let Ok(text) = std::fs::read_to_string(&path) else {
    return Ok(None);
  };
  if has_meaning(&T_CS!(&s!("\\dtldb@{name}"))) {
    return Ok(None);
  }
  // Lines as TeX reads them: a trailing CR and trailing spaces dropped, a
  // blank (empty or spaces-only) line skipped (:12380), the BOM kept.
  let mut rows: Vec<Vec<Vec<Token>>> = Vec::new();
  for raw_line in text.split('\n') {
    let line = raw_line
      .strip_suffix('\r')
      .unwrap_or(raw_line)
      .trim_end_matches(' ');
    if line.trim().is_empty() {
      continue;
    }
    rows.push(split_fields(&line_tokens(line)?));
  }
  let Some(header) = rows.first().cloned() else {
    return Ok(None);
  };
  let data = &rows[1..];
  let ncols = rows.iter().map(Vec::len).max().unwrap_or(0);
  // Column types: the maximum datum type over the non-empty cells (:12562-12585),
  // string for a column with none.
  let mut types = vec![T_STRING; ncols];
  for row in data {
    for (i, cell) in row.iter().enumerate() {
      let t = datum_type(&Tokens::new(cell.clone()).to_string());
      if t != T_UNKNOWN && t > types[i] {
        types[i] = t;
      }
    }
  }
  // The registers, allocated as `\__datatool_new_db:n` (:3538) allocates them.
  digest(mouth::tokenize_internal(TeXString::assembled(format!(
    "\\expandafter\\newtoks\\csname dtldb@{name}\\endcsname\\expandafter\\newtoks\\csname dtlkeys@{name}\\endcsname\\expandafter\\newcount\\csname dtlrows@{name}\\endcsname\\expandafter\\newcount\\csname dtlcols@{name}\\endcsname"
  ))))?;
  // `\dtlkeys@`: one column body per column (:3465, :12292-12313) — the key
  // the header cell's string form (:12441), or `\dtldefaultkey<n>` =
  // `Column<n>` (:10844) as key and header for a column no header cell
  // covers (a data row wider than the header, a trailing separator); the
  // first column to carry a key wins (:12331-12350); `\dtl@ci@<name>@<key>`
  // = its index.
  let mut keys: Vec<Token> = Vec::new();
  let mut seen: Vec<String> = Vec::new();
  let default_cells: Vec<Vec<Token>> = (header.len()..ncols)
    .map(|i| other_chars(&format!("Column{}", i + 1)).collect())
    .collect();
  for i in 0..ncols {
    let cell: &[Token] = match header.get(i) {
      Some(c) => c,
      None => &default_cells[i - header.len()],
    };
    let key = str_tokens(cell);
    let key_text = Tokens::new(key.clone()).to_string();
    if seen.contains(&key_text) {
      continue;
    }
    seen.push(key_text.clone());
    let mut body: Vec<Token> = Vec::new();
    body.extend(wrap("\\db@col@id@w", &digits(i + 1), "\\db@col@id@end@"));
    body.extend(wrap("\\db@key@id@w", &key, "\\db@key@id@end@"));
    body.extend(wrap(
      "\\db@type@id@w",
      &digits(types[i].max(0) as usize),
      "\\db@type@id@end@",
    ));
    body.extend(wrap("\\db@header@id@w", cell, "\\db@header@id@end@"));
    body.extend(wrap("\\db@col@id@w", &digits(i + 1), "\\db@col@id@end@"));
    keys.extend(wrap("\\db@plist@elt@w", &body, "\\db@plist@elt@end@"));
    def_macro(
      T_CS!(&s!("\\dtl@ci@{name}@{key_text}")),
      None,
      ExpansionBody::Tokens(Tokens::new(digits(i + 1))),
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        nopack_parameters: true,
        ..ExpandableOptions::default()
      }),
    )?;
  }
  // `\dtldb@`: one row body per data row (:3486/:3516), ids 1-based, the
  // values verbatim.
  let mut db: Vec<Token> = Vec::new();
  for (r, row) in data.iter().enumerate() {
    let rid = digits(r + 1);
    let mut body: Vec<Token> = Vec::new();
    for (i, cell) in row.iter().enumerate() {
      let cid = digits(i + 1);
      body.extend(wrap("\\db@col@id@w", &cid, "\\db@col@id@end@"));
      body.extend(wrap("\\db@col@elt@w", cell, "\\db@col@elt@end@"));
      body.extend(wrap("\\db@col@id@w", &cid, "\\db@col@id@end@"));
    }
    let mut inner = wrap("\\db@row@id@w", &rid, "\\db@row@id@end@");
    inner.extend(body);
    inner.extend(wrap("\\db@row@id@w", &rid, "\\db@row@id@end@"));
    db.extend(wrap("\\db@row@elt@w", &inner, "\\db@row@elt@end@"));
  }
  assign_register(
    &s!("\\dtldb@{name}"),
    RegisterValue::Tokens(Tokens::new(db)),
    Some(Scope::Global),
    vec![],
  )?;
  assign_register(
    &s!("\\dtlkeys@{name}"),
    RegisterValue::Tokens(Tokens::new(keys)),
    Some(Scope::Global),
    vec![],
  )?;
  assign_register(
    &s!("\\dtlrows@{name}"),
    RegisterValue::Number(Number::new(data.len() as i64)),
    Some(Scope::Global),
    vec![],
  )?;
  assign_register(
    &s!("\\dtlcols@{name}"),
    RegisterValue::Number(Number::new(ncols as i64)),
    Some(Scope::Global),
    vec![],
  )?;
  // After the load (:12252-12261): the last-loaded name, the transient row
  // and column counters at zero.
  def_macro(
    T_CS!("\\dtllastloadeddb"),
    None,
    ExpansionBody::Tokens(mouth::tokenize(TeXString::assembled(name.to_string()))),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      nopack_parameters: true,
      ..ExpandableOptions::default()
    }),
  )?;
  assign_register(
    "\\dtlrownum",
    RegisterValue::Number(Number::new(0)),
    None,
    vec![],
  )?;
  assign_register(
    "\\dtlcolumnnum",
    RegisterValue::Number(Number::new(0)),
    None,
    vec![],
  )?;
  Ok(Some(()))
}

/// The comparison form of an operand (`\__datatool_get_compare_sort:Nn`,
/// datatool-base.sty:8794-8814, under the default `expand-cs=false`, :935):
/// the operand expanded ONCE (`\exp_args:NNo`, :9053 — its first token, when a
/// parameterless macro such as a loop's `\dtlkey`, replaced by its body),
/// every remaining control sequence mapped to one marker character
/// (`\__datatool_get_compare_sort_fn:n`, :8849, `^^J` — so two operands
/// wrapped in different formatting commands compare equal and an accented
/// `\'e` differs from `é`), `~` and a no-break space a space (:8804), the
/// string form; lowercased for the starred, case-insensitive forms
/// (`\__datatool_get_icompare_sort:Nn`, :8816).
fn compare_text(operand: &Tokens, fold: bool) -> Result<String> {
  let mut toks = operand.clone().unlist();
  if let Some(first) = toks.first().copied()
    && first.get_catcode().is_active_or_cs()
    && let Some(defn) = lookup_expandable(&first, Some(true))?
    && defn.get_parameters().is_none()
    && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
  {
    toks.splice(0..1, body.unlist_ref().iter().copied());
  }
  let mut text = String::new();
  for t in &toks {
    if t.get_catcode() == Catcode::CS {
      text.push('\n');
    } else if t.get_catcode() == Catcode::ACTIVE && t.with_str(|s| s == "~") {
      text.push(' ');
    } else {
      t.with_str(|s| text.push_str(s));
    }
  }
  let text = text.replace('\u{a0}', " ");
  Ok(if fold { text.to_lowercase() } else { text })
}

/// `\DTLifnumerical` (datatool-base.sty:8534) on the comparison operand: its
/// value when the datum parser types it integer or decimal
/// (`\__datatool_parse_numbers_ii:nnNN` :8640 hands `\l__datatool_datum_value_tl`
/// on, the thousands groups removed and `.5` as `0.5`).
fn numeric_value(text: &str) -> Option<f64> {
  if datum_type(text) > T_STRING {
    text.replace(',', "").parse::<f64>().ok()
  } else {
    None
  }
}

/// `\DTLifeq` (datatool-base.sty:9075-9100): both operands numeric →
/// `\DTLifnumeq` (:8685, an fp equality of literal values); else the string
/// equality of the comparison forms (`\@DTLifstringeq` :9052,
/// `\str_compare:eNeTF`), folded under `*`. `\DTLifstringeq` (:9040) is the string branch alone. The raw
/// numeric branch in this engine misfires (`\DTLifnumerical{5}` says string,
/// so `\DTLifeq{5}{5.0}` is false where pdflatex says true); the native
/// follows pdflatex. The chosen branch goes back into the stream.
fn dtl_ifeq(
  star: bool,
  a: &Tokens,
  b: &Tokens,
  yes: Tokens,
  no: Tokens,
  numeric: bool,
) -> Result<()> {
  let (ta, tb) = (compare_text(a, star)?, compare_text(b, star)?);
  let equal = match (numeric, numeric_value(&ta), numeric_value(&tb)) {
    (true, Some(x), Some(y)) => x == y,
    _ => ta == tb,
  };
  unread(if equal { yes } else { no });
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("datatool", extension => Some(Cow::Borrowed("sty")), noltxml => true);
  if std::env::var("LATEXML_DATATOOL_NATIVE").as_deref() != Ok("0") {
    // :12798 `\DTLloaddb[opts]{name}{file}`; anything beyond the default
    // shape goes to the raw `\DTLread` exactly as the macro would.
    DefPrimitive!("\\DTLloaddb[]{}{}", sub[(opts, name, file)] {
      let name_s = name.to_string();
      let file_s = file.to_string();
      // Options: none, or only ones restating the defaults (tikz-network
      // passes `noheader=false`, tikz-network.sty:786/:940/:1053).
      let default_options = opts.as_ref().is_none_or(|o| {
        o.to_string()
          .split(',')
          .map(str::trim)
          .all(|kv| kv.is_empty() || matches!(kv.replace(' ', "").as_str(), "noheader=false" | "format=csv" | "csv-content=tex"))
      });
      let native = default_options && load_csv(name_s.trim(), file_s.trim())?.is_some();
      if !native {
        let mut toks = mouth::tokenize_internal("\\DTLread[name={").unlist();
        toks.extend(name.unlist());
        toks.extend(mouth::tokenize_internal("},format=csv,csv-content=tex,").unlist());
        if let Some(o) = opts {
          toks.extend(o.unlist());
        }
        toks.extend(mouth::tokenize_internal("]{").unlist());
        toks.extend(file.unlist());
        toks.push(T_END!());
        unread(Tokens::new(toks));
      }
    }, locked => true);
    // datatool-base.sty:9075 / :9040 — the leaf comparators tikz-network's
    // `\Vertices`/`\Edges` run ~20 times per row (tikz-network.sty:804-816):
    // 95 % of a `\Vertices` call, each raw call ~216 M instructions through
    // `\DTLifnumerical` (l3fp/l3regex) twice and two `\text_purify` passes.
    DefPrimitive!("\\DTLifeq OptionalMatch:* {}{}{}{}", sub[(star, a, b, yes, no)] {
      dtl_ifeq(star.is_some(), &a, &b, yes, no, true)?;
    }, locked => true);
    DefPrimitive!("\\DTLifstringeq OptionalMatch:* {}{}{}{}", sub[(star, a, b, yes, no)] {
      dtl_ifeq(star.is_some(), &a, &b, yes, no, false)?;
    }, locked => true);
  }
});
