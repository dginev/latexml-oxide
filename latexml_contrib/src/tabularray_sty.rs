use latexml_core::keyval::split_keyval_source;
use latexml_package::prelude::*;

/// Translate a tabularray `tblr` inner spec's `colspec={…}` into a classic
/// `\tabular` column template (e.g. `colspec={Q[c]Q[c]}` → `cc`).
///
/// tabularray's `\tblr` is otherwise mapped to `\tabular` (both Rust here and
/// Perl's ar5iv `tabularray.sty.ltxml` are identical `\tblr`→`\tabular` stubs),
/// but the stub hands the WHOLE key-value inner spec (`colspec={…},hlines,…`) to
/// the classic alignment template parser, which char-explodes it ("Unrecognized
/// tabular template" per char, the `\lx@begin@alignment` leak; root-caused
/// 2026-06-30 via the TokenLimit hot-loop study, witness 2605.06284).
///
/// This translator extracts and converts the colspec so the produced `\tabular`
/// gets the right column COUNT and approximate alignment. It is deliberately
/// conservative: it handles the common column producers (`Q[…]`, `X[…]`, bare
/// `c`/`l`/`r`, `p`/`m`/`b{width}`, `|`, and `*{n}{…}` repeats) and **returns
/// `None` on anything it does not fully understand** (e.g. `S` siunitx columns
/// without the library), so the caller hands the colspec as written to the classic
/// template — the column count is therefore either correct or the classic reading.
#[cfg(test)]
fn translate_tblr_colspec(inner: &str) -> Option<String> {
  translate_tblr_colspec_with(inner, &|_| None)
}

/// [`translate_tblr_colspec`] with the column types `\NewColumnType` defined
/// (`types`, by name).
#[cfg(test)]
fn translate_tblr_colspec_with(inner: &str, types: ColumnTypes) -> Option<String> {
  let spec = extract_colspec_value(inner)?;
  parse_colspec(&spec, types)
}

/// A column type `\NewColumnType{<name>}[<n>][<default>]{<body>}` defined
/// (tabularray.sty:3291-3334: an xparse command `tblr_Column_type_<name>`
/// whose `n` arguments, the first optional when a default is given, are put
/// into `body`, which is then read as more colspec).
struct TblrColumnType {
  nargs:   usize,
  default: Option<String>,
  body:    String,
}

/// The `\NewColumnType`s of the document, by name.
type ColumnTypes<'a> = &'a dyn Fn(char) -> Option<TblrColumnType>;

/// The State key of a `\NewColumnType`.
fn tblr_column_type_key(name: &str) -> String { s!("tblr_column_type_{name}") }

/// The `\NewColumnType` named `name`, if the document defined one.
fn tblr_column_type(name: char) -> Option<TblrColumnType> {
  let Some(Stored::Strings(fields)) = lookup_value(&tblr_column_type_key(&name.to_string())) else {
    return None;
  };
  let [nargs, has_default, default, body] = &fields[..] else {
    return None;
  };
  Some(TblrColumnType {
    nargs:   to_string(*nargs).parse().unwrap_or(0),
    default: (to_string(*has_default) == "1").then(|| to_string(*default)),
    body:    to_string(*body),
  })
}

/// Expand a use of `ty` at `*i` (after its name) into its body, advancing past
/// its arguments; `None` when an argument is missing or unbalanced.
fn expand_tblr_column_type(
  ty: &TblrColumnType,
  b: &[u8],
  spec: &str,
  i: &mut usize,
) -> Option<String> {
  let mut args: Vec<String> = Vec::with_capacity(ty.nargs);
  for n in 0..ty.nargs {
    while *i < b.len() && (b[*i] as char).is_ascii_whitespace() {
      *i += 1;
    }
    if n == 0
      && let Some(default) = &ty.default
    {
      if *i < b.len() && b[*i] == b'[' {
        let close = spec[*i..].find(']')? + *i;
        args.push(spec[*i + 1..close].to_string());
        *i = close + 1;
      } else {
        args.push(default.clone());
      }
      continue;
    }
    if *i < b.len() && b[*i] == b'{' {
      args.push(parse_braced_group(b, spec, i)?);
    } else {
      let c = spec[*i..].chars().next()?;
      args.push(c.to_string());
      *i += c.len_utf8();
    }
  }
  let mut body = ty.body.clone();
  for (n, arg) in args.iter().enumerate().rev() {
    body = body.replace(&s!("#{}", n + 1), arg);
  }
  Some(body)
}

/// The `table/inner` keys (tabularray.sty:3986-3997, :3999-4033; the functional and varwidth
/// libraries add `process`, :8308, and `measure`, :8769).
const TBLR_INNER_KEYS: [&str; 32] = [
  "column",
  "row",
  "cell",
  "hline",
  "vline",
  "hborder",
  "vborder",
  "name",
  "colspec",
  "rowspec",
  "width",
  "hspan",
  "vspan",
  "stretch",
  "columns",
  "rows",
  "cells",
  "hlines",
  "vlines",
  "leftsep",
  "rightsep",
  "colsep",
  "abovesep",
  "belowsep",
  "rowsep",
  "rulesep",
  "rowhead",
  "rowfoot",
  "delimiter",
  "baseline",
  "process",
  "measure",
];

/// tabularray.sty:4045-4057 `\__tblr_parse_table_spec:n`: an inner spec whose first key name is
/// a `table/inner` key is a key list, any other spec a colspec. The name is the first comma
/// item's text before `=` and before any `{` (`\__tblr_keyval_extract_first_name:n`, :395-402;
/// `\__tblr_key_split_name:n`, :370-375): `{hlines}` and `{row{1}={c}}` are key lists, `{lcr}`
/// and `{X[l,c]}` colspecs. Read as a colspec, `{hlines}` gave the template `hlines` (whose `h`
/// took `l` as a width).
fn is_tblr_key_list(spec: &str) -> bool {
  let first = spec.split(',').next().unwrap_or_default();
  let name = first.split(['=', '{']).next().unwrap_or_default().trim();
  TBLR_INNER_KEYS.contains(&name)
}

/// The colspec of an inner spec (tabularray.sty:4045-4057): the whole spec when it is not a key
/// list, else its `colspec` key's value.
fn extract_colspec_value(inner: &str) -> Option<String> {
  let inner = inner.trim();
  if inner.is_empty() {
    return None;
  }
  if !is_tblr_key_list(inner) {
    return Some(inner.to_string());
  }
  key_list_colspec(inner)
}

/// The last `colspec` key's value in a key list (keys are set in order, the last one wins).
fn key_list_colspec(keys: &str) -> Option<String> {
  split_keyval_source(keys)
    .into_iter()
    .rev()
    .find(|(key, _)| key == "colspec")
    .map(|(_, value)| value)
}

/// Parse a tabularray colspec body into a classic `\tabular` template, or `None`
/// if it contains a construct we don't translate (bail → stub fallback).
#[cfg(test)]
fn parse_colspec(spec: &str, types: ColumnTypes) -> Option<String> {
  parse_colspec_full(spec, types)
    .filter(|parsed| parsed.unknown.is_none())
    .map(|parsed| parsed.cols)
}

/// A colspec as a classic template: the template, its column count, and the first column type it
/// names that is neither tabularray's nor a `\NewColumnType` — an error, after which tabularray reads
/// no more of the spec as columns (tabularray.sty:3389-3410): the template is the columns before it,
/// and the rest, which tabularray typesets as stray text, is dropped.
struct ParsedColspec {
  cols:    String,
  ncols:   usize,
  unknown: Option<char>,
}

/// [`parse_colspec`] with its column count and unknown column types; `None` for a construct left
/// untranslated.
fn parse_colspec_full(spec: &str, types: ColumnTypes) -> Option<ParsedColspec> {
  let mut ncols = 0usize;
  let mut unknown = None;
  let cols = parse_colspec_capped(spec, 0, &mut ncols, types, &mut unknown)?;
  Some(ParsedColspec { cols, ncols, unknown })
}

/// The widest row of the table body that follows, read ahead and put back: tabularray reads the body
/// whole (`+b`, tabularray.sty:3461-3470) and splits its rows at `\\` and cells at `&` outside
/// braces. A math table has as many columns as that (a math alignment is never column-pruned, so it
/// takes no fixed margin). The body ends at the first `\end{…}` outside braces that closes no
/// `\begin{…}` of the body — the table's own `\end{tblr}`, or the end of an environment that wraps
/// it (`\newenvironment{mymat}{\begin{tblr}{cc}}{\end{tblr}}`, where only `\end{mymat}` is in the
/// source) — and the read stops at `TBLR_BODY_READ_CAP` tokens: it never runs on through the rest of
/// the document, which it would tokenize ahead of any verbatim there. Guard
/// `perfect_kernel_batch57::tabularray_math_tables`.
fn tblr_body_width() -> Result<usize> {
  let scan = tblr_scan_body(TBLR_BODY_READ_CAP)?;
  unread(Tokens::new(scan.tokens));
  Ok(scan.widest)
}

/// The table body that follows, read up to the `\end{…}` that ends it ([`tblr_body_width`]'s rule),
/// which stays in the input: what tabularray's `+b` argument holds (tabularray.sty:3463) for its
/// preprocessing — read whole, as tabularray reads it, without the width pre-read's cap. `None`
/// (everything put back) when the input ends first.
fn tblr_read_body() -> Result<Option<Vec<Token>>> {
  let mut scan = tblr_scan_body(usize::MAX)?;
  match scan.end_at {
    Some(end) => {
      unread(Tokens::new(scan.tokens.split_off(end)));
      Ok(Some(scan.tokens))
    },
    None => {
      unread(Tokens::new(scan.tokens));
      Ok(None)
    },
  }
}

/// What [`tblr_scan_body`] read: every token, where the body's closing `\end` starts in them, and the
/// widest row.
struct TblrBodyScan {
  tokens: Vec<Token>,
  end_at: Option<usize>,
  widest: usize,
}

fn tblr_scan_body(cap: usize) -> Result<TblrBodyScan> {
  let is_cs = |t: &Token, name: &str| t.get_catcode() == Catcode::CS && t.with_str(|s| s == name);
  let mut body = Vec::new();
  let mut end_at = None;
  // The `{name}` after a `\begin`/`\end`, read into the body.
  let read_name = |body: &mut Vec<Token>| -> Result<Option<String>> {
    match read_token()? {
      Some(open) if open.get_catcode() == Catcode::BEGIN => {
        body.push(open);
        let mut name = String::new();
        while body.len() < cap
          && let Some(c) = read_token()?
        {
          body.push(c);
          if c.get_catcode() == Catcode::END {
            return Ok(Some(name));
          }
          name.push_str(&c.to_string());
        }
        Ok(None)
      },
      Some(other) => {
        body.push(other);
        Ok(None)
      },
      None => Ok(None),
    }
  };
  let mut open_envs: Vec<String> = Vec::new();
  let (mut depth, mut widest, mut cells) = (0usize, 0usize, 1usize);
  while body.len() < cap
    && let Some(t) = read_token()?
  {
    body.push(t);
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      // a cell or row of this table, not of an environment in a cell (`\begin{pmatrix} a & b
      // \end{pmatrix}`, which tabularray accepts unbraced)
      Catcode::ALIGN if depth == 0 && open_envs.is_empty() => cells += 1,
      _ if depth == 0
        && open_envs.is_empty()
        && (is_cs(&t, "\\\\") || is_cs(&t, "\\cr") || is_cs(&t, "\\tabularnewline")) =>
      {
        widest = widest.max(cells);
        cells = 1;
      },
      _ if depth == 0 && is_cs(&t, "\\begin") => {
        if let Some(name) = read_name(&mut body)? {
          open_envs.push(name);
        }
      },
      _ if depth == 0 && is_cs(&t, "\\end") => {
        let at = body.len() - 1;
        let name = read_name(&mut body)?;
        match open_envs
          .iter()
          .rposition(|open| Some(open) == name.as_ref())
        {
          Some(opened) => open_envs.truncate(opened),
          None => {
            end_at = Some(at);
            break;
          },
        }
      },
      _ => {},
    }
  }
  Ok(TblrBodyScan {
    tokens: body,
    end_at,
    widest: widest.max(cells),
  })
}
/// How far `tblr_body_width` reads ahead at most (the preprocessing read, `tblr_read_body`, has none).
const TBLR_BODY_READ_CAP: usize = 1 << 16;

/// Caps: recursion depth ≤ 8 and ≤ 512 total columns — nested `*{n}{…}`
/// multiplies past any per-level cap (`*{1000}{*{1000}{c}}`), and deep
/// `*{1}{…}` nesting is otherwise unbounded recursion (PR_READINESS
/// should-fix 13). Exceeding a cap bails to the stub, like any other
/// untranslatable spec.
fn parse_colspec_capped(
  spec: &str,
  depth: usize,
  total: &mut usize,
  types: ColumnTypes,
  unknown: &mut Option<char>,
) -> Option<String> {
  if depth > 8 {
    return None;
  }
  let b = spec.as_bytes();
  let mut i = 0;
  let mut cols = String::new();
  while i < b.len() && unknown.is_none() {
    // a character, not a byte: `\NewColumnType{é}` names a column (a byte read `Ã`)
    let Some(c) = spec[i..].chars().next() else {
      break;
    };
    match c {
      ' ' | '\t' | '\n' | '\r' => i += 1,
      // tabularray.sty:3172-3181 `|` takes `O{}` rule options (`|[1pt]`,
      // `|[dashed]`): styling, dropped (logoetalab-doc.tex:143
      // `colspec={|[1pt]X[j]}`; bailing left the keys to be read as columns).
      '|' => {
        cols.push('|');
        i += 1;
        skip_bracket_group(b, &mut i)?;
      },
      // tabularray.sty:3339 `j` is `Q[j]`, justified: the default alignment.
      'j' => {
        cols.push('l');
        *total += 1;
        if *total > 512 {
          return None;
        }
        i += 1;
      },
      'c' | 'l' | 'r' => {
        cols.push(c);
        *total += 1;
        if *total > 512 {
          return None;
        }
        i += 1;
      },
      // Generic (Q) and stretchy (X) columns: one column each, alignment from
      // the optional [..] bracket (c/l/r). X has no classic equivalent → use its
      // alignment (default l); the stretch is dropped (approximate, but the
      // column count is exact).
      'Q' | 'X' => {
        i += 1;
        let mut align = 'l';
        if i < b.len() && b[i] == b'[' {
          let start = i + 1;
          let mut j = start;
          while j < b.len() && b[j] != b']' {
            j += 1;
          }
          if j >= b.len() {
            return None; // unbalanced [..]
          }
          let opts = &spec[start..j];
          // Alignment is a STANDALONE single-letter key (or halign=X) among
          // the comma-separated options — a substring scan misread
          // `bg=cyan` as centered ('c' in "cyan").
          for item in opts.split(',') {
            let item = item.trim();
            let item = item.strip_prefix("halign=").unwrap_or(item);
            match item {
              "c" => align = 'c',
              "r" => align = 'r',
              "l" => align = 'l',
              _ => {},
            }
          }
          i = j + 1;
        }
        cols.push(align);
        *total += 1;
        if *total > 512 {
          return None;
        }
      },
      // p/m/b{width}: copy verbatim (classic understands these); tabularray's
      // t/h/f{width} (:3341-3346, `Q[t,wd=#1]`, …) are top/head/foot-aligned
      // paragraph columns, a `p` here.
      't' | 'h' | 'f' => {
        i += 1;
        let width = parse_braced_group(b, spec, &mut i)?;
        cols.push_str(&s!("p{{{width}}}"));
        *total += 1;
        if *total > 512 {
          return None;
        }
      },
      'p' | 'm' | 'b' => {
        let start = i;
        i += 1;
        if i < b.len() && b[i] == b'{' {
          let mut depth = 0usize;
          let body_start = i;
          while i < b.len() {
            if b[i] == b'{' {
              depth += 1;
            } else if b[i] == b'}' {
              depth -= 1;
              if depth == 0 {
                i += 1;
                break;
              }
            }
            i += 1;
          }
          if depth != 0 {
            return None; // unbalanced {width}
          }
          cols.push_str(&spec[start..i]);
          let _ = body_start;
          *total += 1;
          if *total > 512 {
            return None;
          }
        } else {
          return None; // `p` without a width is not classic-valid
        }
      },
      // *{n}{sub}: repeat the sub-spec n times.
      '*' => {
        i += 1;
        let n = parse_braced_uint(b, spec, &mut i)?;
        let sub = parse_braced_group(b, spec, &mut i)?;
        // Count the sub-spec's columns once, then charge the delta for every
        // further copy so the TOTAL cap holds under multiplication. The spec ends
        // in the first copy at an unknown type: one copy, charged once.
        let before = *total;
        let sub_cols = parse_colspec_capped(&sub, depth + 1, total, types, unknown)?;
        let copies = if unknown.is_some() { 1 } else { n };
        let per = *total - before;
        let extra = per.checked_mul(copies.saturating_sub(1))?;
        *total = total.checked_add(extra)?;
        if *total > 512 {
          return None;
        }
        for _ in 0..copies {
          cols.push_str(&sub_cols);
        }
      },
      // Inter-column material `@{…}`/`!{…}` and the array.sty hooks `>{…}`/
      // `<{…}`: not columns — copy verbatim (classic understands them). A
      // `colspec={@{}Xll@{}}` that bailed here left the WHOLE inner spec as
      // the template, whose `cell{…}={cmd={\BusyPanda…}}` value was then
      // edef-expanded in the alignment preamble and ran an l3fp delimited
      // scan to EOF (panda manual, `Until:\__fp_sep:` Fatal).
      '@' | '!' | '>' | '<' => {
        let start = i;
        i += 1;
        // tabularray.sty:3194, :3234 `>`/`<` take `O{} m`: the optional is the
        // column's inner separation, dropped.
        let classic_start = if matches!(c, '>' | '<') && i < b.len() && b[i] == b'[' {
          skip_bracket_group(b, &mut i)?;
          cols.push(c);
          i
        } else {
          start
        };
        if i < b.len() && b[i] == b'{' {
          let mut depth = 0usize;
          while i < b.len() {
            if b[i] == b'{' {
              depth += 1;
            } else if b[i] == b'}' {
              depth -= 1;
              if depth == 0 {
                i += 1;
                break;
              }
            }
            i += 1;
          }
          if depth != 0 {
            return None;
          }
          cols.push_str(&spec[classic_start..i]);
        } else {
          return None;
        }
      },
      // A `\NewColumnType`: its body, with its arguments, is more colspec
      // (non-decimal-units.sty:778 `\NewColumnType{#1}[2]{Q[r, cmd=…]}`,
      // `U{danish rigsdaler}{add to variable=…}`: bailing left the keys to
      // be read as columns, whose `b` read `l` as a width).
      _ => {
        let Some(ty) = types(c) else {
          *unknown = Some(c);
          break;
        };
        i += c.len_utf8();
        let body = expand_tblr_column_type(&ty, b, spec, &mut i)?;
        cols.push_str(&parse_colspec_capped(
          &body,
          depth + 1,
          total,
          types,
          unknown,
        )?);
      },
    }
  }
  // an unknown type first (`{zz}`) leaves an empty template, the type's error reported
  if cols.is_empty() && unknown.is_none() {
    None
  } else {
    Some(cols)
  }
}

/// Skip a `[…]` group at `*i` (after spaces), if there is one; `None` when it
/// is unclosed.
fn skip_bracket_group(b: &[u8], i: &mut usize) -> Option<()> {
  let mut j = *i;
  while j < b.len() && (b[j] == b' ' || b[j] == b'\t') {
    j += 1;
  }
  if j < b.len() && b[j] == b'[' {
    let close = b[j..].iter().position(|&c| c == b']')? + j;
    *i = close + 1;
  }
  Some(())
}

/// Parse a `{<count>}` group at `*i`, advancing past it. Returns the integer.
fn parse_braced_uint(b: &[u8], spec: &str, i: &mut usize) -> Option<usize> {
  let g = parse_braced_group(b, spec, i)?;
  let g = g.trim();
  g.parse::<usize>()
    .ok()
    .or_else(|| evaluate_count(g))
    .filter(|&n| n > 0 && n <= 1000)
}

/// A count that is not a literal is an integer expression, as tabularray
/// evaluates it (tabularray.sty:3361 `\prg_replicate:nn` → `\int_eval:n`):
/// PixelArtTikz.sty:865's `colspec={*{\ListeCoulCaseslen}{Q[m,c]}}`. Read as
/// `\number\numexpr … \relax` in the current state (the colspec is translated
/// while `\lx@tblr@env` expands); unread, the whole spec had fallen back to the
/// kernel template and its `p` took `e` as a width (PixelArtTikz-doc-fr, sweep
/// #126). Any count that is not a literal goes through the engine, `1+1` as
/// well as `\n+1`. The expression is the spec's text, so a control word keeps
/// no space after it (`\csname n\endcsname` reads `\csnamen`), as for the
/// kernel template the spec falls back to; that fallback expands a bad count
/// again, and its "Missing number" is reported a second time.
fn evaluate_count(expr: &str) -> Option<usize> {
  let tokens = mouth::tokenize_internal(TeXString::assembled(format!(
    "\\number\\numexpr {expr}\\relax"
  )));
  do_expand(tokens).ok()?.to_string().trim().parse().ok()
}

/// Parse a brace-balanced `{…}` group at `*i`, advancing past it. Returns the
/// inner text. Returns `None` if `*i` is not at `{` or the group is unbalanced.
fn parse_braced_group(b: &[u8], spec: &str, i: &mut usize) -> Option<String> {
  while *i < b.len() && (b[*i] == b' ' || b[*i] == b'\t') {
    *i += 1;
  }
  if *i >= b.len() || b[*i] != b'{' {
    return None;
  }
  let start = *i + 1;
  let mut depth = 1usize;
  let mut j = start;
  while j < b.len() {
    match b[j] {
      b'{' => depth += 1,
      b'}' => {
        depth -= 1;
        if depth == 0 {
          *i = j + 1;
          return Some(spec[start..j].to_string());
        }
      },
      _ => {},
    }
    j += 1;
  }
  None
}

#[cfg(test)]
mod tests {
  use super::translate_tblr_colspec;

  #[test]
  fn bare_colspec_shorthand() {
    // A mandatory arg with no top-level `=` IS the colspec.
    assert_eq!(translate_tblr_colspec("Q[c]Q[c]"), Some("cc".to_string()));
    assert_eq!(translate_tblr_colspec("|c|c|"), Some("|c|c|".to_string()));
  }

  #[test]
  fn q_alignment_is_a_standalone_key() {
    // `bg=cyan` must NOT read as centered; halign=r counts.
    assert_eq!(
      translate_tblr_colspec("colspec={Q[l,bg=cyan]Q[halign=r]}"),
      Some("lr".to_string())
    );
  }

  #[test]
  fn nested_repeat_caps() {
    // Multiplied nesting past the total cap bails to the stub (None).
    assert_eq!(
      translate_tblr_colspec("colspec={*{1000}{*{1000}{c}}}"),
      None
    );
    // ...but a legitimate large-ish repeat still translates.
    assert_eq!(
      translate_tblr_colspec("colspec={*{4}{cl}}"),
      Some("clclclcl".to_string())
    );
  }

  #[test]
  fn colspec_translation() {
    // Common forms → correct classic column template (count + alignment).
    assert_eq!(
      translate_tblr_colspec("colspec={Q[c]Q[c]},hlines").as_deref(),
      Some("cc")
    );
    assert_eq!(
      translate_tblr_colspec("colspec={Q[l]Q[r]}").as_deref(),
      Some("lr")
    );
    // X (stretchy) → its alignment (default l); width dropped, count exact.
    assert_eq!(
      translate_tblr_colspec("colspec={Q[l]X[2]p{3cm}|c}").as_deref(),
      Some("llp{3cm}|c")
    );
    // *{n}{sub} repeat.
    assert_eq!(
      translate_tblr_colspec("colspec={*{3}{c}}").as_deref(),
      Some("ccc")
    );
    assert_eq!(
      translate_tblr_colspec("colspec={*{2}{Q[r]}|l}").as_deref(),
      Some("rr|l")
    );
    // colspec=... value not in braces (until comma).
    assert_eq!(
      translate_tblr_colspec("colspec=ccc,hlines").as_deref(),
      Some("ccc")
    );
    // colspec not first key.
    assert_eq!(
      translate_tblr_colspec("hlines,colspec={cc}").as_deref(),
      Some("cc")
    );
    // Inter-column material is copied through and never counts as a column.
    assert_eq!(
      translate_tblr_colspec("colspec={@{}Xll@{}}").as_deref(),
      Some("@{}lll@{}")
    );
    assert_eq!(
      translate_tblr_colspec("colspec={>{\\bfseries}l!{\\vrule}c}").as_deref(),
      Some(">{\\bfseries}l!{\\vrule}c")
    );
    // Bail (→ None → caller keeps the stub behaviour) on unhandled constructs.
    assert_eq!(
      translate_tblr_colspec("colspec={S[table-format=2.1]c}"),
      None
    ); // siunitx S
    assert_eq!(translate_tblr_colspec("hlines,vlines"), None); // no colspec
    assert_eq!(translate_tblr_colspec("colspec={Q[c]z}"), None); // unknown 'z'
    // Rule options, `>[sep]{…}`, `j` and `t{…}` columns.
    assert_eq!(
      translate_tblr_colspec("width=15cm,colspec={|[1pt]X[j]},cells={font=\\footnotesize}")
        .as_deref(),
      Some("|l")
    );
    assert_eq!(
      translate_tblr_colspec("colspec={>[2pt]{\\bfseries}j|[dashed]t{2cm}}").as_deref(),
      Some(">{\\bfseries}l|p{2cm}")
    );
  }

  /// tabularray.sty:4045-4057: the first key name decides between a key list and a colspec.
  #[test]
  fn first_key_name_decides_key_list_or_colspec() {
    use super::{extract_colspec_value, is_tblr_key_list};
    for keys in [
      "hlines",
      "row{1}={c}",
      "hline{2-3}={dashed},colspec={ll}",
      "width=1em,hlines",
    ] {
      assert!(is_tblr_key_list(keys), "{keys}");
    }
    for colspec in ["lcr", "X[l,c]X", "Q[c]Q[c]", "|[dashed]l|", "*{3}{c}"] {
      assert!(!is_tblr_key_list(colspec), "{colspec}");
    }
    assert_eq!(extract_colspec_value("hlines"), None);
    assert_eq!(extract_colspec_value("X[l,c]X").as_deref(), Some("X[l,c]X"));
    assert_eq!(translate_tblr_colspec("X[l,c]X").as_deref(), Some("cl"));
    // keys are set in order: the last colspec wins
    assert_eq!(
      extract_colspec_value("colspec={l},colspec={cc}").as_deref(),
      Some("cc")
    );
  }

  #[test]
  fn new_column_types_expand_into_colspec() {
    use super::{TblrColumnType, translate_tblr_colspec_with};
    let types = |name: char| match name {
      'U' => Some(TblrColumnType {
        nargs:   2,
        default: None,
        body:    String::from("Q[r, cmd=\\cell {#1} {#2}]"),
      }),
      'Y' => Some(TblrColumnType {
        nargs:   1,
        default: Some(String::from("c")),
        body:    String::from("Q[#1]"),
      }),
      _ => None,
    };
    assert_eq!(
      translate_tblr_colspec_with(
        "r U{danish rigsdaler}{add to variable=total}| U{a}{b}",
        &types
      )
      .as_deref(),
      Some("rr|r")
    );
    assert_eq!(
      translate_tblr_colspec_with("Y Y[l]", &types).as_deref(),
      Some("cl")
    );
    assert_eq!(translate_tblr_colspec_with("U{a}", &types), None); // missing argument
  }

  /// The spec ends at the first unknown column type (tabularray.sty:3389-3410), in a repeat's first
  /// copy too: the columns before it, each counted once (57cp review 5, colz: `*{3}{lz}` charged
  /// three `l`s and reported two extra alignment tabs).
  #[test]
  fn colspec_stops_at_the_first_unknown_type() {
    use super::parse_colspec_full;
    let none = |_: char| None;
    for (spec, cols, ncols, unknown) in [
      ("lzr", "l", 1, 'z'),
      ("*{2}{lz}r", "l", 1, 'z'),
      ("*{3}{lz}", "l", 1, 'z'),
      ("c*{2}{l}zr", "cll", 3, 'z'),
      ("zz", "", 0, 'z'),
    ] {
      let parsed = parse_colspec_full(spec, &none).expect(spec);
      assert_eq!(
        (parsed.cols.as_str(), parsed.ncols, parsed.unknown),
        (cols, ncols, Some(unknown)),
        "{spec}"
      );
    }
    let parsed = parse_colspec_full("*{3}{lc}r", &none).expect("known");
    assert_eq!(
      (parsed.cols.as_str(), parsed.ncols, parsed.unknown),
      ("lclclcr", 7, None)
    );
  }
}

/// One key of a tabularray key list, read as l3keys reads one (tabularray.sty:395-402): the item
/// ends at a depth-0 comma, the key at the first depth-0 `=`, and one pair of braces around the
/// value is stripped. `name` is the key's text before any `{` (`note{a}` names `note`, :370-375),
/// `arg` the braced text after it (`a`); the value keeps its tokens (captions and notes are
/// typeset).
struct TblrKey {
  name:  String,
  arg:   Tokens,
  value: Tokens,
}

/// The State key of the material a table's end prints after it (`\lx@tblr@after`).
const TBLR_AFTER: &str = "tblr_after";

/// The width of the table `\lx@tblr@tablewidth` measured, for its box.
const TBLR_WIDTH: &str = "tblr_width";

/// The keys of a key list, in order.
fn tblr_keys(spec: &Tokens) -> Vec<TblrKey> {
  let is_other = |t: &Token, c: &str| t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == c);
  let mut keys = Vec::new();
  let mut depth = 0usize;
  let (mut key, mut value): (Vec<Token>, Option<Vec<Token>>) = (Vec::new(), None);
  let mut tokens = spec.clone().unlist();
  tokens.push(T_OTHER!(","));
  for t in tokens {
    if depth == 0 && is_other(&t, ",") {
      let key = tblr_strip(std::mem::take(&mut key));
      let split = key
        .iter()
        .position(|t| t.get_catcode() == Catcode::BEGIN)
        .unwrap_or(key.len());
      let name = Tokens::new(key[..split].to_vec())
        .to_string()
        .trim()
        .to_string();
      if !name.is_empty() {
        keys.push(TblrKey {
          name,
          arg: Tokens::new(tblr_strip(key[split..].to_vec())),
          value: Tokens::new(tblr_strip(value.take().unwrap_or_default())),
        });
      }
      value = None;
      continue;
    }
    if depth == 0 && value.is_none() && is_other(&t, "=") {
      value = Some(Vec::new());
      continue;
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    match &mut value {
      Some(v) => v.push(t),
      None => key.push(t),
    }
  }
  keys
}

/// `tokens` without surrounding spaces and without one pair of braces enclosing all of them.
fn tblr_strip(tokens: Vec<Token>) -> Vec<Token> {
  let start = tokens
    .iter()
    .position(|t| t.get_catcode() != Catcode::SPACE)
    .unwrap_or(tokens.len());
  let end = tokens
    .iter()
    .rposition(|t| t.get_catcode() != Catcode::SPACE)
    .map_or(start, |e| e + 1);
  let inner = &tokens[start..end];
  let mut depth = 0usize;
  let enclosed = inner.len() >= 2
    && inner.iter().enumerate().all(|(i, t)| {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth = depth.saturating_sub(1),
        _ => {},
      }
      // the first group closes only at the end
      depth > 0 || i == inner.len() - 1
    })
    && inner[0].get_catcode() == Catcode::BEGIN;
  if enclosed {
    inner[1..inner.len() - 1].to_vec()
  } else {
    inner.to_vec()
  }
}

/// The keys `\SetTblrInner`/`\SetTblrOuter` (`kind` `inner`/`outer`) recorded for `env`.
fn tblr_env_defaults(kind: &str, env: &str) -> Result<Tokens> {
  Ok(
    lookup_definition(&T_CS!(s!("\\lx@tblr@{kind}@{env}")))?
      .and_then(|d| d.get_expansion().cloned())
      .and_then(|b| match b {
        ExpansionBody::Tokens(t) => Some(t),
        _ => None,
      })
      .unwrap_or(Tokens!()),
  )
}

/// The head/foot elements a template command names (tabularray.sty:6261-6271: `head` is the
/// three heads, `foot` the three feet).
fn tblr_template_elements(elements: &Tokens) -> Vec<String> {
  let mut out = Vec::new();
  for element in elements
    .to_string()
    .split(',')
    .map(str::trim)
    .filter(|e| !e.is_empty())
  {
    match element {
      "head" => out.extend(["firsthead", "middlehead", "lasthead"].map(String::from)),
      "foot" => out.extend(["firstfoot", "middlefoot", "lastfoot"].map(String::from)),
      _ => out.push(element.to_string()),
    }
  }
  out
}

/// tabularray's templates (tabularray.sty:5838-6298): the elements, the names declared for them, and
/// the one `\SetTblrTemplate` makes their default (none for `head`/`foot`, whose elements set
/// theirs one by one).
const TBLR_BUILTIN_TEMPLATES: [(&str, &[&str], &str); 29] = [
  ("contfoot-text", &["normal"], "normal"),
  ("contfoot", &["empty", "plain", "normal"], "normal"),
  ("conthead-pre", &["empty", "normal"], "normal"),
  ("conthead-text", &["normal"], "normal"),
  ("conthead", &["empty", "plain", "normal"], "normal"),
  ("caption-lot", &["empty", "normal"], "normal"),
  ("caption-tag", &["empty", "normal"], "normal"),
  ("caption-sep", &["empty", "normal"], "normal"),
  ("caption-text", &["empty", "normal"], "normal"),
  ("caption", &["empty", "plain", "normal", "simple"], "normal"),
  ("capcont", &["empty", "plain", "normal", "simple"], "normal"),
  ("note-border", &["empty", "normal"], "empty"),
  ("note-tag", &["empty", "normal"], "normal"),
  ("note-target", &["normal"], "normal"),
  ("note-sep", &["empty", "normal"], "normal"),
  ("note-text", &["empty", "normal"], "normal"),
  ("note", &["empty", "plain", "normal", "inline"], "normal"),
  ("remark-tag", &["empty", "normal"], "normal"),
  ("remark-sep", &["empty", "normal"], "normal"),
  ("remark-text", &["empty", "normal"], "normal"),
  ("remark", &["empty", "plain", "normal", "inline"], "normal"),
  ("head", &["empty"], ""),
  ("foot", &["empty"], ""),
  ("firsthead", &["normal"], "normal"),
  ("middlehead", &["normal"], "normal"),
  ("lasthead", &["normal"], "normal"),
  ("firstfoot", &["normal"], "normal"),
  ("middlefoot", &["normal"], "normal"),
  ("lastfoot", &["normal"], "normal"),
];

/// Whether the element's default template (`\SetTblrTemplate`) is an empty one.
fn tblr_template_is_empty(element: &str) -> bool {
  lookup_value(&s!("tblr_template:{element}")).is_some_and(|v| v.to_string() == "empty")
}

/// Whether the element's default template is the document's own (`\DeclareTblrTemplate`, which the
/// reduction does not run) and calls `\caption` itself, which writes the List of Tables line.
fn tblr_template_writes_caption(element: &str) -> bool {
  lookup_value(&s!("tblr_template:{element}")).is_some_and(|v| v.to_string() == "custom")
    && matches!(lookup_value(&s!("tblr_template_code:{element}")),
      Some(Stored::Tokens(code)) if CaptionScan::new().calls_caption(code.unlist_ref(), &[], 4))
}

/// A scan of a document template's code for the List of Tables line it writes: a `\caption`, or
/// a `\captionof{table}`, not starred — spelled directly, as a `\let` copy or
/// `\csname caption\endcsname`, through `\UseTblrTemplate{caption}{<name>}` (the document's
/// `caption` template, which a custom head may use: njuthesis.cls:1597-1602), or in the macros the
/// code calls, to four levels. The meanings of `\caption`/`\captionof` are looked up once, and a
/// macro is rescanned only from a level higher than any it was scanned from (per what follows it:
/// a star, a `{table}` group), so the scan stays linear and does not depend on the order the macros
/// are met in. Not seen: a star passed as a macro's argument or following its arguments, a `\let`
/// copy taken before a package redefines `\caption`, an `\endcsname` from a macro. 57cp reviews 6
/// (t4/t5), 7 (e1), 8 (e8), 9 (p1: Deepfirst, Cofmacro).
struct CaptionScan {
  caption:   Option<Stored>,
  captionof: Option<Stored>,
  /// (macro or template, a star follows, a `{table}` group follows) → the deepest level scanned
  visited:   std::collections::HashMap<(String, bool, bool), usize>,
}

enum CaptionCall {
  Caption,
  CaptionOf,
}

impl CaptionScan {
  fn new() -> Self {
    CaptionScan {
      caption:   lookup_meaning(&T_CS!("\\caption")),
      captionof: lookup_meaning(&T_CS!("\\captionof")),
      visited:   std::collections::HashMap::new(),
    }
  }

  /// `\caption`/`\captionof` by name or by meaning (a `\let` copy).
  fn call(&self, t: &Token) -> Option<CaptionCall> {
    let named = |name: &str| t.get_catcode() == Catcode::CS && t.with_str(|s| s == name);
    if named("\\caption") {
      return Some(CaptionCall::Caption);
    }
    if named("\\captionof") {
      return Some(CaptionCall::CaptionOf);
    }
    if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
      return None;
    }
    let meaning = lookup_meaning(t)?;
    if self.caption.as_ref() == Some(&meaning) {
      Some(CaptionCall::Caption)
    } else if self.captionof.as_ref() == Some(&meaning) {
      Some(CaptionCall::CaptionOf)
    } else {
      None
    }
  }

  /// Whether `key` is still to be scanned from `depth` levels (none deeper done yet); records it.
  fn first_scan(&mut self, key: (String, bool, bool), depth: usize) -> bool {
    match self.visited.get(&key) {
      Some(&done) if done >= depth => false,
      _ => {
        self.visited.insert(key, depth);
        true
      },
    }
  }

  /// Whether `tokens` write the line; `after` is what follows them where they are called
  /// (`\def\capx{\caption}` then `\capx*`, `\def\cof{\captionof}` then `\cof{table}`).
  fn calls_caption(&mut self, tokens: &[Token], after: &[Token], depth: usize) -> bool {
    let star = T_OTHER!("*");
    let mut i = 0;
    while i < tokens.len() {
      let t = &tokens[i];
      let mut next = i + 1;
      let call = if *t == T_CS!("\\csname") {
        // the exact name up to `\endcsname` (`caption␣` is another name)
        match tokens[i + 1..]
          .iter()
          .position(|t| *t == T_CS!("\\endcsname"))
        {
          Some(p) => {
            next = i + 2 + p;
            let name: String = tokens[i + 1..i + 1 + p]
              .iter()
              .map(|t| t.to_string())
              .collect();
            match name.as_str() {
              "caption" => Some(CaptionCall::Caption),
              "captionof" => Some(CaptionCall::CaptionOf),
              _ => None,
            }
          },
          None => None,
        }
      } else if *t == T_CS!("\\UseTblrTemplate") {
        // `\UseTblrTemplate{caption}{<name>}`: the document's `caption` template
        if let Some((element, e)) = tblr_braced_text(tokens, i + 1)
          && let Some((name, n)) = tblr_braced_text(tokens, e)
        {
          next = n;
          let key = if name.trim() == "default" {
            String::from("tblr_template_code:caption")
          } else {
            s!("tblr_template_code:caption:{}", name.trim())
          };
          if element.trim() == "caption"
            && depth > 0
            && self.first_scan((key.clone(), false, false), depth)
            && let Some(Stored::Tokens(code)) = lookup_value(&key)
            && self.calls_caption(code.unlist_ref(), &[], depth - 1)
          {
            return true;
          }
        }
        None
      } else {
        self.call(t)
      };
      // what follows here: the rest of these tokens, or of the caller's once these run out
      let rest = if next < tokens.len() {
        &tokens[next..]
      } else {
        after
      };
      let starred = rest.first() == Some(&star);
      // only a table caption writes a List of Tables line (`\captionof{figure}` lists a figure)
      let table_group = tblr_braced_text(rest, 0).is_some_and(|(kind, _)| kind.trim() == "table");
      match call {
        Some(CaptionCall::Caption) if !starred => return true,
        Some(CaptionCall::CaptionOf) if table_group => return true,
        Some(_) => {},
        None => {
          if depth > 0
            && matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE)
            && self.first_scan((t.to_string(), starred, table_group), depth)
            && let Some(ExpansionBody::Tokens(body)) = lookup_definition(t)
              .ok()
              .flatten()
              .and_then(|d| d.get_expansion().cloned())
            && self.calls_caption(body.unlist_ref(), rest, depth - 1)
          {
            return true;
          }
        },
      }
      i = next;
    }
    false
  }
}

/// The text of the brace group starting at `tokens[start]`, and the index after it.
fn tblr_braced_text(tokens: &[Token], start: usize) -> Option<(String, usize)> {
  if tokens.get(start)?.get_catcode() != Catcode::BEGIN {
    return None;
  }
  let mut depth = 0usize;
  for (k, t) in tokens.iter().enumerate().skip(start) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          let text: String = tokens[start + 1..k].iter().map(|t| t.to_string()).collect();
          return Some((text, k + 1));
        }
      },
      _ => {},
    }
  }
  None
}

/// `tokens` in braces.
fn tblr_braced(tokens: &Tokens) -> Vec<Token> {
  let mut out = vec![T_BEGIN!()];
  out.extend(tokens.clone().unlist());
  out.push(T_END!());
  out
}

LoadDefinitions!({
  RequirePackage!("booktabs");
  // `\tblr` maps to `\tabular`, but tabularray's argument is a key-value inner
  // spec (`colspec={Q[c]Q[c]},hlines,…`) or a colspec, NOT a classic column template. Parse
  // out the colspec and translate it so `\tabular` gets the right column count;
  // a colspec we don't fully translate is the template as written. `[]{}` captures the
  // optional outer spec + the mandatory inner spec. See `translate_tblr_colspec`.
  // Every tblr-family environment funnels through `\lx@tblr@env{<env>}[outer]
  // {inner}`: the per-environment defaults recorded by `\SetTblrInner[<env>]`
  // (tabularray.sty:3444, `O{tblr} m` — the optional lists the environments,
  // default `tblr`) are prepended to the inner spec, `colspec` is translated,
  // and when NO colspec exists anywhere the column count is inferred from the
  // rows (tabularray's own rule) — the alignment already normalizes every row
  // to the widest one, so a wide `l` template reproduces that. Witness:
  // pegmatch (`\NewTblrEnviron{spectblr}` + `\SetTblrInner[spectblr]{hlines…}`
  // + `\begin{spectblr}[caption=…]{}` — the empty inner spec became a
  // zero-column template, 52 "Extra alignment tab").
  DefMacro!("\\lx@tblr@env{} []{}", sub[(env, outer, inner)] {
    let env = env.to_string();
    let stored = tblr_env_defaults("inner", &env)?.to_string();
    // The environment's defaults are keys set before the inner spec (tabularray.sty:3975-3979), so
    // the inner spec's colspec wins; a colspec the translation does not read is still the
    // template (a spec with none has columns inferred), never the key list around it — the
    // template `hlines, vlines, colspec={…}` gave a `p` the width `e`.
    let spec = extract_colspec_value(&inner.to_string()).or_else(|| key_list_colspec(&stored));
    let math = lookup_string_from_sym(pin!("MODE")).ends_with("math");
    let body_width = if math { tblr_body_width()? } else { 0 };
    let (cols, ncols) = match spec {
      Some(spec) => match parse_colspec_full(&spec, &tblr_column_type) {
        Some(parsed) => {
          // tabularray.sty:3389-3410: a column type it does not know is an error (`S` without the
          // siunitx library), reported for the first
          if let Some(c) = parsed.unknown {
            Error!("unexpected", &c.to_string(), s!("Unknown Column type {c}!"));
          }
          (parsed.cols, parsed.ncols)
        },
        // a construct the reduction does not translate: the kernel's template reads it as written
        None => (spec, usize::MAX),
      },
      // columns inferred from the rows: in math exactly the widest row
      None if math => (s!("*{{{}}}{{l}}", body_width.max(1)), body_width.max(1)),
      None => (String::from("*{32}{l}"), 32),
    };
    // tabularray parses its own body and tolerates a row wider than the
    // colspec (circularglyphs-doc.tex:196: `*{13}{X[m,c]}` with a 14-cell
    // last row; pdflatex clean, Perl raw-loads it clean). The kernel template
    // is only a hard cap — the final column count is the widest row and
    // short rows are padded — so a margin of fallback columns is inert on a
    // well-formed table and absorbs a ragged one. Guard:
    // `perfect_kernel_batch54::tblr_row_wider_than_the_colspec_is_tolerated`.
    // The margin continues the LAST column's alignment, as tabularray does.
    // (with no column, a cell's initial `halign=j`, :3908-3910, read as `l`)
    let last = cols.trim_end().chars().last().filter(|c| c.is_ascii_alphabetic()).unwrap_or('l');
    // A table in math mode has math cells (tabularray.sty:3490-3492, :4604-4617): `$\begin{tblr}…$`
    // and the amsmath library's `+array` are an `array`, the rest a `tabular`; the environment's end
    // (`\lx@tblr@end`) closes whichever was opened, and `\lx@tblr@mot` wraps a content command's
    // argument in math when the cells are math (`\__tblr_lib_diagbox_math_or_text:n`, :8243-8246).
    // Witness: the manual's math tables (`$\begin{tblr}`, `+array`, :1248-1257).
    // The outer spec: the environment's defaults, then the table's own (tabularray.sty:4100-4110).
    // A long or tall table (:4121-4122) prints a caption, notes and remarks around the tabular,
    // after its theme has set their templates (:4141).
    let mut outer_keys = tblr_env_defaults("outer", &env)?.unlist();
    if let Some(outer) = outer {
      outer_keys.push(T_OTHER!(","));
      outer_keys.extend(outer.unlist());
    }
    let outer_keys = Tokens::new(outer_keys);
    let keys = tblr_keys(&outer_keys);
    let portrait = keys.iter().any(|k| k.name == "long" || k.name == "tall");
    // tabularray.sty:3567-3575 `\__tblr_modify_table_body:`: before it splits the body into cells,
    // tabularray evaluates the functional library's `evaluate=` functions (:8262-8306) and expands each
    // `expand=` macro once (:3579-3591; `expand+=` appends, :4136). The binding hands the body to the
    // kernel `tabular` unread, so with either key it reads the body (`+b`, :3463) and runs that
    // code on it first (`\lx@tblr@modify@body`). Unread, `\makeEmptyTable{3}{7}` ran inside the first
    // cell and its `&` met the cell's group: "Stray alignment" and a 1×6 table for 3×7 (the tabularray
    // manual, tabularray.tex:2859-2870).
    let mut evaluate = Vec::new();
    let mut expand = Vec::new();
    for key in &keys {
      match key.name.as_str() {
        "evaluate" => evaluate = key.value.clone().unlist(),
        "expand" => expand = key.value.clone().unlist(),
        "expand+" => expand.extend(key.value.clone().unlist()),
        _ => {},
      }
    }
    let body = if evaluate.is_empty() && expand.is_empty() { None } else { tblr_read_body()? };
    // A long or tall table in math is a box (tabularray typesets its caption and notes around
    // the math cells, "Table 1: …" in `$…$`): an `\hbox` holding them and a math `array`.
    let (open, end, mot) = match (math, portrait) {
      (true, false) => ("\\array", "\\endarray", "\\lx@tblr@mot@math"),
      // the environment form: a bare `\\array` opened in `$…$` inside the box left its frame open
      (true, true) => ("$\\begin{array}", "\\end{array}$", "\\lx@tblr@mot@math"),
      _ => ("\\tabular", "\\endtabular", "\\@firstofone"),
    };
    let close_box = if math && portrait { "\\egroup" } else { "" };
    // A math alignment is never column-pruned (alignment/normalize.rs:472-476), so a math table
    // takes no margin: 16 empty cells a row followed each one (the manual's `mode=` demo, :1248).
    let margin = match math {
      // a row wider than the colspec (tabularray tolerates it) widens the math table to that row
      true if ncols != usize::MAX && body_width > ncols => s!("*{{{}}}{{{last}}}", body_width - ncols),
      true => String::new(),
      false => s!("*{{16}}{{{last}}}"),
    };
    // The table meanings are macros with optional arguments (`\lx@tblr@hline[]`), expandable as
    // the row-start scan needs (an `\@ifnextchar` definition started a row of empty cells in a
    // math `array`).
    // tabularray.sty:2006/2008 `\NewTblrTableCommand \hline [1] []` /
    // `\cline [2] []`: inside a tblr both take an optional `[<style>]`
    // (`\hline[dashed]\hline`, manual :547). The kernel `\hline` the stub
    // reuses has no optional, so `[dashed]` became cell text and the next
    // `\hline`'s `\noalign` fired mid-cell (7-error cascade per demo;
    // RUST-ONLY — Perl raw-loads tabularray). Scoped to the environment's
    // group; the style itself is unrendered. Guard:
    // `perfect_kernel_batch56::tblr_hline_style_optional_is_absorbed`. A table in a table's cell
    // keeps the outer table's meanings: saving them again made `\hline` call itself (a recursion
    // Fatal).
    // `\lx@tblr@after` follows the environment's end: a long or tall table's notes, remarks and
    // float end (`\lx@tblr@portrait`).
    let mut out = TokenizeInternal!(TeXString::assembled(format!(
      "\\ifx\\hline\\lx@tblr@hline\\else\\let\\lx@tblr@saved@hline\\hline\\let\\hline\\lx@tblr@hline\\fi\
       \\ifx\\cline\\lx@tblr@cline\\else\\let\\lx@tblr@saved@cline\\cline\\let\\cline\\lx@tblr@cline\\fi\
       \\def\\lx@tblr@end{{{end}\\lx@tblr@after{close_box}}}\\let\\lx@tblr@mot{mot}\
       \\let\\lx@tblr@saved@do\\do\\def\\do#1{{\\expandafter\\let\\expandafter#1\\csname lx@tblr@tc@\\string#1\\endcsname}}\
       \\lx@tblr@table@commands\\let\\do\\lx@tblr@saved@do\
       \\ifdefined\\lx@tblr@diagbox\\let\\diagbox\\lx@tblr@diagbox\\let\\diagboxthree\\lx@tblr@diagboxthree\\fi"
    )))
    .unlist();
    // no after-material unless a long or tall table sets it (a nested table must not print its
    // enclosing one's notes)
    assign_value(TBLR_AFTER, Stored::Tokens(Tokens!()), None);
    if !close_box.is_empty() {
      out.extend(TokenizeInternal!("\\hbox\\bgroup").unlist());
    }
    for theme in keys.iter().filter(|k| k.name == "theme") {
      let name = theme.value.to_string().trim().to_string();
      let code = T_CS!(s!("\\lx@tblr@theme@{name}"));
      if lookup_definition(&code)?.is_some() {
        out.push(code);
      } else {
        // an undefined theme is an erroneous variable (:4097 `\tl_use:c`)
        let variable = s!("\\g__tblr_theme_{name}_code_tl");
        Error!("undefined", &variable, s!("Erroneous variable {variable} used"));
      }
    }
    if portrait {
      out.push(T_CS!("\\lx@tblr@portrait"));
      out.extend(tblr_braced(&outer_keys));
    }
    if let Some(body) = &body {
      out.extend([T_CS!("\\tl_set:Nn"), T_CS!("\\l__tblr_body_tl"), T_BEGIN!()]);
      out.extend(body.iter().cloned());
      out.extend([T_END!(), T_CS!("\\lx@tblr@modify@body"), T_BEGIN!()]);
      out.extend(evaluate);
      out.extend([T_END!(), T_BEGIN!()]);
      out.extend(expand);
      out.push(T_END!());
    }
    // tabularray's own alignment keeps the interline values `\\@array` zeroes (tex_tables.rs `array_zeroes_interline`)
    out.extend(
      TokenizeInternal!(TeXString::assembled(format!("\\lx@array@keeps@interline{open}{{{cols}{margin}}}"))).unlist(),
    );
    if body.is_some() {
      out.push(T_CS!("\\l__tblr_body_tl"));
    }
    Ok(Tokens::new(out))
  });
  // A long or tall table's caption, notes and remarks (tabularray.sty:6384-6470, the templates at
  // :5895-6240): its label entry (:6457-6474) steps the table counter, labels it and adds the list
  // of tables line — none of it with `label=none`, which also drops the caption's tag (:6440-6444)
  // — and the head prints the caption, the last foot the notes (`note{<tag>}=<text>`) and the
  // remarks (`remark{<tag>}=<text>`), unless their templates are empty. A tagged caption is the
  // kernel's `\caption` of a `table` (so it is numbered, listed and labelled as LaTeX does); an
  // empty `caption-tag` or `label=none` prints the text alone; a caption-less numbered table only
  // steps the counter. A custom `caption-tag`/`caption-sep` (tabularray-abnt's "Quadro N —") is
  // printed as the kernel's "Table N:" (divergence #384). Unprinted, the texts were lost (16 in the
  // manuals: tabularray, easybook, simplebnf, tabularray-abnt, csvsimple-l3).
  DefMacro!("\\lx@tblr@portrait{}", sub[(outer)] {
    let keys = tblr_keys(&outer);
    let last = |name: &str| keys.iter().rev().find(|k| k.name == name).map(|k| k.value.clone());
    let shown = |element: &str| !tblr_template_is_empty(element);
    // each part prints through its own template (:5895-6240): an empty one prints nothing
    let caption_text = last("caption").unwrap_or(Tokens!());
    let caption = if shown("caption-text") { caption_text.clone() } else { Tokens!() };
    let label = last("label").unwrap_or(Tokens!());
    let numbered = label.to_string().trim() != "none";
    let entry = last("entry").filter(|e| !e.is_empty());
    // The List of Tables line (`caption-lot`, :5896-5905: the entry, else the caption as given)
    // is written unless `entry=none` (:6474-6478) — for a `label=none` table too, with the counter
    // as it stands. A line with no text to it (neither an entry nor a caption) goes on a table that
    // has a float for other reasons; it opens none of its own.
    let entry_none = entry.as_ref().is_some_and(|e| e.to_string().trim() == "none");
    let lot_text = entry.clone().filter(|_| !entry_none).unwrap_or_else(|| caption_text.clone());
    // A document template calling `\caption` writes the line itself for a numbered table:
    // tblr-extras' `caption` library empties `caption-lot` because its `firsthead`/`lastfoot` call
    // `\caption[entry]{…}` (tblr-extras.sty:45-169), which the binding's own caption stands for —
    // except under `label=none`, where its head prints nothing (:47-50). A document template that
    // only formats the caption leaves an empty `caption-lot` unlisted (57cp review 5: tmpl, tex5).
    // The `caption` template is reached through a head that uses it: tabularray's own `firsthead`
    // (`\UseTblrTemplate{caption}{default}`, :6276-6279) or a document head calling
    // `\UseTblrTemplate{caption}` (njuthesis.cls:1597-1602); under an empty head it writes nothing
    // (57cp reviews 7-8, e7, e8).
    let own_caption = || {
      tblr_template_writes_caption("firsthead")
        || tblr_template_writes_caption("lastfoot")
        || (lookup_value("tblr_template:firsthead").is_some_and(|v| v.to_string() == "code")
          && tblr_template_writes_caption("caption"))
    };
    let listed = !entry_none && (shown("caption-lot") || (numbered && own_caption()));
    // the kernel caption's optional argument: the line's text where it is not the printed caption
    let entry = (listed && lot_text.to_string() != caption.to_string()).then_some(lot_text.clone());
    // The tag (`\tablename~\thetable`, :5911) prints for a numbered table unless its template is
    // empty; the caption element prints when there is a tag or a text to print.
    let tagged = numbered && shown("caption-tag");
    let caption_shown =
      shown("firsthead") && shown("caption") && (tagged || !caption.is_empty());
    let texts = |name: &str| -> Vec<&TblrKey> {
      if shown("lastfoot") && shown(name) { keys.iter().filter(|k| k.name == name).collect() } else { Vec::new() }
    };
    let (notes, remarks) = (texts("note"), texts("remark"));
    // Where the caption goes. A long or tall table is one box placed where it is written
    // (tabularray.sty:6384-6474), its caption, notes and remarks in it: the binding opens a `table`
    // inside its own box (`{lx@tblr@box}`, the kernel's `insert_block`, as wide as the table:
    // `\lx@tblr@tablewidth`), which stands as that table where the document holds one — alone
    // between paragraphs the box folds into its float (ruling C), and in a `table` float with no
    // caption of its own the two are one table (`collapse_float`) — and stays in place as an inline
    // block elsewhere: in a quote, an abstract, an algorithm, a cell, a note, `\resizebox`, math. A
    // float opened directly moved out of all of those; the document's float taking the caption
    // put two captions in one float when the float had its own (their labels read one number).
    let labelled = numbered && !label.is_empty();
    let container = caption_shown
      || !notes.is_empty()
      || !remarks.is_empty()
      || labelled
      || (listed && !lot_text.is_empty());
    // A long table is vertical material, as longtable's (it ends the paragraph before it and
    // starts one after, tabularray.sty's long builder outputs page pieces); a tall one is a box
    // in the line. Outside a box, a cell or a note the long table's box stands between paragraphs.
    let long = keys.iter().rev().find(|k| k.name == "long" || k.name == "tall").is_some_and(|k| k.name == "long");
    let par = if long && !lookup_bool_sym(pin!("INNER_BOX")) { "\\par" } else { "" };
    let (open, close) = if container {
      (
        s!("{par}\\begin{{lx@tblr@box}}\\begin{{table}}"),
        s!("\\end{{table}}\\end{{lx@tblr@box}}{par}"),
      )
    } else {
      (String::new(), String::new())
    };
    let internal = |s: &str| TokenizeInternal!(TeXString::assembled(s.to_string())).unlist();
    let mut before = internal(&open);
    // The label entry (:6459-6478) steps the counter, labels it and writes the List of Tables line
    // whether or not the caption prints. With a `table` to carry them the number and the line go on
    // that table (`\@@add@caption@counters`, the kernel caption's own step: the float's tags and
    // list, not a printed caption), so a `\ref` to a table whose theme hides the head still reads
    // its number; without one only the counter steps.
    let step = if container { "\\@@add@caption@counters" } else { "\\refstepcounter{table}" };
    if tagged && caption_shown {
      // the kernel's caption, saved at the document's start: longtable's `\caption` in a cell of
      // one swallowed the table (57cp review)
      before.push(T_CS!("\\lx@tblr@caption"));
      if let Some(entry) = &entry {
        before.push(T_OTHER!("["));
        before.extend(tblr_braced(entry));
        before.push(T_OTHER!("]"));
      }
      before.extend(tblr_braced(&caption));
      if !listed {
        before.push(T_CS!("\\lx@tblr@nolist"));
      }
    } else {
      // a caption the theme hides, or one printed untagged (`label=none`, an empty `caption-tag`):
      // the text alone, the number and the line as the label entry makes them
      if numbered {
        before.extend(internal(step));
      }
      if container && listed {
        if !numbered {
          before.push(T_CS!("\\lx@tblr@list"));
        }
        before.extend(internal("\\@@toccaption{\\lx@format@toctitle@@{table}"));
        before.extend(tblr_braced(&lot_text));
        before.push(T_END!());
      } else if container && numbered {
        before.push(T_CS!("\\lx@tblr@nolist"));
      }
      if caption_shown {
        before.push(T_CS!("\\@@caption"));
        before.extend(tblr_braced(&caption));
      }
    }
    if labelled && container {
      // `\label` itself: the kernel caption takes a trailing `\label` by its token
      // (`\@caption@postlabel`), and a copy under another name printed its argument
      before.push(T_CS!("\\label"));
      before.extend(tblr_braced(&label));
    }
    // note: the tag superscript in sans serif, then the text (:6104-6150); remark: the tag in
    // italics, ": ", the text (:6190-6225)
    let mut after = Vec::new();
    let (note_tag, note_text) = (shown("note-tag"), shown("note-text"));
    for note in notes.iter().filter(|_| note_tag || note_text) {
      after.extend(internal("\\par\\noindent"));
      if note_tag {
        after.extend(internal("\\textsuperscript"));
        after.push(T_BEGIN!());
        after.extend(internal("\\sffamily"));
        after.extend(note.arg.clone().unlist());
        after.push(T_END!());
      }
      if note_tag && note_text {
        after.extend(internal("\\space"));
      }
      if note_text {
        after.extend(note.value.clone().unlist());
      }
    }
    let (remark_tag, remark_sep, remark_text) = (shown("remark-tag"), shown("remark-sep"), shown("remark-text"));
    for remark in remarks.iter().filter(|_| remark_tag || remark_sep || remark_text) {
      after.extend(internal("\\par\\noindent"));
      if remark_tag {
        after.push(T_BEGIN!());
        after.extend(internal("\\itshape"));
        after.extend(remark.arg.clone().unlist());
        after.push(T_END!());
      }
      if remark_sep {
        after.push(T_OTHER!(":"));
        after.extend(internal("\\space"));
      }
      if remark_text {
        after.extend(remark.value.clone().unlist());
      }
    }
    if !after.is_empty() {
      after.extend(internal("\\par"));
    }
    if container {
      after.insert(0, T_CS!("\\lx@tblr@tablewidth"));
    }
    after.extend(internal(&close));
    // kept as tokens, not a macro body: a note's `#` (`\url{…/#1x}`) is not a parameter
    assign_value(TBLR_AFTER, Stored::Tokens(Tokens::new(after)), None);
    Ok(Tokens::new(before))
  });
  // A long or tall table's box (`\lx@tblr@portrait`): the kernel's `insert_block`, as `{minipage}`
  // builds one, as wide as the table.
  DefEnvironment!("{lx@tblr@box}", sub[document, _args, props] {
    if let Some(Stored::Digested(body)) = props.get("body") {
      insert_block_in_paragraph(document, body, string_map!("class" => "ltx_tblr_box"), props)?;
    }
    Ok(())
  },
  // as wide as its table (`\lx@tblr@tablewidth`)
  after_digest => sub[whatsit] {
    if let Some(width) = remove_value(TBLR_WIDTH) {
      whatsit.set_property("width", width);
    }
  },
  mode => "inline_internal_vertical", enter_horizontal => true);
  // The List of Tables line a table's caption leaves pending for its float (`table_inlist`, as
  // `\@@add@caption@counters` records it): dropped for `entry=none`, made for a `label=none` table.
  DefPrimitive!("\\lx@tblr@nolist", {
    remove_value("table_inlist");
  });
  DefPrimitive!("\\lx@tblr@list", {
    let inlist = digest(T_CS!("\\ext@table"))?.to_string();
    assign_value("table_inlist", inlist, Some(Scope::Global));
  });
  // tabularray sets a long or tall table's caption, notes and remarks in a `\hsize` of the table's
  // width (`\__tblr_build_table_head_aux:Nn`/`…foot_aux`, :6480-6508): the notes wrap in it, and the
  // box holding the table (`{lx@tblr@box}`) is as wide as the table — the caption line or a note
  // set at the line's width made `\resizebox{\linewidth}` scale it by 1 or 3 where pdflatex scales
  // by 9. Right after the table, the last box on the list.
  DefPrimitive!("\\lx@tblr@tablewidth", {
    let width = with_box_list(|list| list.last().map(|table| table.get_width(None)))
      .transpose()?
      .flatten()
      .filter(|w| w.clone().value_of() > 0);
    if let Some(width) = width {
      assign_register("\\hsize", width.clone(), None, Vec::new())?;
      assign_value(TBLR_WIDTH, Stored::from(width), Some(Scope::Global));
    }
  });
  // The kernel's `\caption`, as the document has it at its start (with caption.sty loaded), not as
  // an environment around the table rebinds it (longtable's row caption swallowed the table).
  RawTeX!(r"\let\lx@tblr@caption\caption\AtBeginDocument{\let\lx@tblr@caption\caption}");
  // The material after a table's end (`\lx@tblr@portrait`), in the table's group.
  DefMacro!("\\lx@tblr@after", {
    Ok(match lookup_value(TBLR_AFTER) {
      Some(Stored::Tokens(tokens)) => tokens,
      _ => Tokens!(),
    })
  });
  DefMacro!("\\lx@tblr@hline[]", "\\lx@tblr@saved@hline");
  DefMacro!("\\lx@tblr@cline[]{}", "\\lx@tblr@saved@cline{#2}");
  // booktabs library (tabularray.sty:8155): `\cmidrulemore[<keys>]{<range>}`, the `\cmidrule` that
  // follows one; the keys are tabularray's, not booktabs' width.
  DefMacro!("\\lx@tblr@cmidrulemore[]{}", "\\cmidrule{#2}");
  DefMacro!("\\tblr", "\\lx@tblr@env{tblr}");
  DefMacro!("\\endtblr", "\\lx@tblr@end");
  // tabularray.sty:3472-3477 creates `longtblr`/`talltblr` with the same
  // factory (`long`/`tall` outer specs add page-breaking + caption/notes
  // layout the tabular reduction has no slot for). Witness: panda manual
  // (`{longtblr}` undefined → 149 relational-token errors + EoF Fatal).
  DefMacro!("\\longtblr", "\\lx@tblr@env{longtblr}");
  DefMacro!("\\endlongtblr", "\\lx@tblr@end");
  DefMacro!("\\talltblr", "\\lx@tblr@env{talltblr}");
  DefMacro!("\\endtalltblr", "\\lx@tblr@end");
  // tabularray.sty:3443-3457 `\SetTblrInner[<envs>]{<keys>}` / `\SetTblrOuter`: keys appended to
  // the environments' defaults (default `tblr`), set before the table's own.
  // :3443-3457: both are `\NewDocumentCommand`s (assignments, not expandable) ending in
  // `\ignorespaces`; the work is the primitive `\lx@tblr@set@defaults{<kind>}{<envs>}{<keys>}`.
  DefPrimitive!("\\lx@tblr@set@defaults{}{}{}", sub[(kind, envs, keys)] {
    let kind = kind.to_string();
    // `O{tblr}`: the default when the list is absent, none when it is given empty
    let envs = Expand!(envs).to_string();
    for env in envs.split(',').map(str::trim).filter(|e| !e.is_empty()) {
      let mut merged = tblr_env_defaults(&kind, env)?.unlist();
      if !merged.is_empty() {
        merged.push(T_OTHER!(","));
      }
      merged.extend(keys.clone().unlist());
      def_macro(T_CS!(s!("\\lx@tblr@{kind}@{env}")), None, ExpansionBody::Tokens(Tokens::new(merged)), None)?;
    }
    Ok(())
  });
  DefMacro!(
    "\\SetTblrInner[Default:tblr]{}",
    "\\lx@tblr@set@defaults{inner}{#1}{#2}\\ignorespaces"
  );
  DefMacro!(
    "\\SetTblrOuter[Default:tblr]{}",
    "\\lx@tblr@set@defaults{outer}{#1}{#2}\\ignorespaces"
  );

  // :3472-3477, :8168-8169: the long and tall environments are long and tall by their defaults.
  RawTeX!(r"\SetTblrOuter[longtblr,longtabs]{long}\SetTblrOuter[talltblr,talltabs]{tall}");
  // tabularray.sty:8163 `\NewTblrEnviron{booktabs}` (and `longtabs`/`talltabs`)
  // are tblr-family environments whose inner spec is the same key-value list
  // (`{row{2}={c}}`); mapping `\booktabs` straight to `\tabular` bypassed the
  // `\lx@tblr@env` colspec extraction, so the keys became a 0-column template
  // (tabularray manual :2666/:2686, 12 "Extra alignment tab" errors; RUST-ONLY).
  // Guard: `perfect_kernel_batch56::tblr_table_commands_and_booktabs_env`.
  DefMacro!("\\booktabs", "\\lx@tblr@env{booktabs}");
  DefMacro!("\\endbooktabs", "\\lx@tblr@end");
  DefMacro!("\\longtabs", "\\lx@tblr@env{longtabs}");
  DefMacro!("\\endlongtabs", "\\lx@tblr@end");
  DefMacro!("\\talltabs", "\\lx@tblr@env{talltabs}");
  DefMacro!("\\endtalltabs", "\\lx@tblr@end");
  // tabularray.sty:8036-8050 `\UseTblrLibrary{<list>}`: each library's code (`\NewTblrLibrary`,
  // :8030) runs once; a name without one is `\RequirePackage{tblrlib<name>}`. The stub only loaded
  // the same-named package, so the environments and commands the libraries define were undefined
  // (tabularray manual: `{+pmatrix}`, `{+cases}`, `\cmidrulemore`, `\diagboxthree`,
  // `{tblrtikzbelow}` — 14 errors).
  DefMacro!("\\UseTblrLibrary{}", sub[(libs)] {
    let mut out = String::new();
    for lib in libs.to_string().split(',').map(str::trim).filter(|l| !l.is_empty()) {
      let code = s!("lx@tblr@lib@{lib}");
      out.push_str(&if lookup_definition(&T_CS!(s!("\\{code}")))?.is_some() {
        format!("\\csname {code}\\endcsname\\global\\expandafter\\let\\csname {code}\\endcsname\\relax")
      } else {
        format!("\\RequirePackage{{tblrlib{lib}}}")
      });
    }
    Ok(TokenizeInternal!(TeXString::assembled(out)))
  });
  // tabularray.sty:8030-8034 `\NewTblrLibrary{<name>}{<code>}` registers a library for
  // `\UseTblrLibrary` — tabularray's own below, and packages' (tblr-extras.sty:34 `caption`, :184
  // `babel`, which dlrg-templates uses).
  RawTeX!(
    r"\long\def\NewTblrLibrary#1#2{\long\expandafter\def\csname lx@tblr@lib@#1\endcsname{#2}}"
  );
  // The libraries (tabularray.sty:8059-8790), reduced to what the tabular reduction can carry:
  // amsmath's `+array` is a tblr environment (an `array` in math) and `+matrix`… `+cases` its
  // delimited forms — amsmath's own matrices in math, a text table in text (delimiters dropped);
  // booktabs' `\cmidrulemore` is a `\cmidrule`; diagbox's content commands take math arguments in
  // math cells; tikz's overlay environments collect code drawn on cell nodes the reduction has
  // none of (`+b`, dropped with a warning); siunitx's `S`/`s` columns are `Q[si=…,c]` (the `s`
  // column's `cmd=\TblrUnit` and the `si` key's `\TblrNum`, :8416-8425, are not applied: no
  // `cmd=` is); the others load their packages (`hook` varwidth, :8365; `counter` and `html`
  // nothing, :8200, :8384). OXIDIZED_DESIGN_DIVERGENCES #384.
  RawTeX!(
    r#"\def\lx@tblr@lib@amsmath{\RequirePackage{amsmath}\NewTblrEnviron{+array}%
\SetTblrInner[+array]{colsep=5pt}%
\NewDocumentEnvironment{+matrix}{O{}}{\lx@tblr@matrix{matrix}}{\lx@tblr@endmatrix}%
\NewDocumentEnvironment{+bmatrix}{O{}}{\lx@tblr@matrix{bmatrix}}{\lx@tblr@endmatrix}%
\NewDocumentEnvironment{+Bmatrix}{O{}}{\lx@tblr@matrix{Bmatrix}}{\lx@tblr@endmatrix}%
\NewDocumentEnvironment{+pmatrix}{O{}}{\lx@tblr@matrix{pmatrix}}{\lx@tblr@endmatrix}%
\NewDocumentEnvironment{+vmatrix}{O{}}{\lx@tblr@matrix{vmatrix}}{\lx@tblr@endmatrix}%
\NewDocumentEnvironment{+Vmatrix}{O{}}{\lx@tblr@matrix{Vmatrix}}{\lx@tblr@endmatrix}%
\NewDocumentEnvironment{+cases}{O{}}{\lx@tblr@matrix{cases}}{\lx@tblr@endmatrix}}%
\def\lx@tblr@matrix#1{\ifmmode\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\def\lx@tblr@endmatrix{\end{#1}}\begin{#1}}{\def\lx@tblr@endmatrix{\lx@tblr@end}\lx@tblr@env{+#1}{cells={c}}}}%
\def\lx@tblr@lib@booktabs{\RequirePackage{booktabs}%
\let\cmidrulemore\lx@tblr@cmidrulemore}%
\def\lx@tblr@lib@diagbox{\RequirePackage{diagbox}\let\lx@tblr@saved@diagbox\diagbox
\NewDocumentCommand\lx@tblr@diagbox{O{}mm}{\lx@tblr@saved@diagbox[##1]{\lx@tblr@mot{##2}}{\lx@tblr@mot{##3}}}%
\NewDocumentCommand\lx@tblr@diagboxthree{O{}mmm}{\lx@tblr@saved@diagbox[##1]{\lx@tblr@mot{##2}}{\lx@tblr@mot{##3}}{\lx@tblr@mot{##4}}}}%
\def\lx@tblr@mot@math#1{$#1$}%
\def\lx@tblr@lib@tikz{\RequirePackage{tikz}\usetikzlibrary{calc}\lx@tblr@lib@varwidth
\NewDocumentEnvironment{tblrtikzbelow}{+b}{\lx@tblr@tikz@dropped{below}}{}%
\NewDocumentEnvironment{tblrtikzabove}{+b}{\lx@tblr@tikz@dropped{above}}{}}%
\def\lx@tblr@tikz@dropped#1{\PackageWarning{tabularray}{The tblrtikz#1 drawing is not rendered}}%
\def\lx@tblr@lib@siunitx{\RequirePackage{siunitx}\NewTblrColumnType{S}[1][]{Q[si={##1},c]}%
\NewTblrColumnType{s}[1][]{Q[si={##1},c]}}%
\def\lx@tblr@lib@functional{\RequirePackage{functional}\gdef\lx@tblr@lib@functional@used{}}%
\def\lx@tblr@lib@nameref{\RequirePackage{nameref}}%
\def\lx@tblr@lib@varwidth{\RequirePackage{varwidth}}%
\def\lx@tblr@lib@zref{\RequirePackage{zref-user}}%
\def\lx@tblr@lib@hook{\lx@tblr@lib@varwidth}\let\lx@tblr@lib@counter\relax\let\lx@tblr@lib@html\relax"#
  );
  // tabularray.sty:36: ninecolors comes with xcolor (`blue5`, `magenta6`, … — tcolorbox and tikzfill
  // name them, 9 undefined colors in sweep #131; witness 2605.06284, `green6`).
  RawTeX!(r"\AddToHook{package/xcolor/after}{\RequirePackage{ninecolors}}");
  // The public variables (tabularray.sty:1195-1282, :1876-1879, :2705, :3497, :6315-6351, :6509,
  // :6540, :6737-6738, :7160, :7387, :7491-7546), with tabularray's initial values: documents set
  // and read them (`\setlength\lTblrDefaultHruleWidthDim{1pt}`, manual: 4 errors); the caption,
  // entry and label token lists are defined below. `\ExpTblrChildId`/`\ExpTblrChildClass` (:1825-1834)
  // name child selections the reduction does not make.
  RawTeX!(
    r"\ExplSyntaxOn
\clist_new:N \lTblrUsedChildIndexerClist \clist_new:N \lTblrUsedChildSelectorClist
\int_new:N \lTblrChildTotalInt \int_new:N \lTblrChildHtotalInt \int_new:N \lTblrChildVtotalInt
\tl_new:N \lTblrChildIndexTl \clist_new:N \lTblrChildClist
\dim_new:N \lTblrDefaultHruleWidthDim \dim_set:Nn \lTblrDefaultHruleWidthDim {0.4pt}
\dim_new:N \lTblrDefaultVruleWidthDim \dim_set:Nn \lTblrDefaultVruleWidthDim {0.4pt}
\bool_new:N \lTblrCellBreakBool \bool_new:N \lTblrMeasuringBool
\int_new:N \lTblrRowHeadInt \int_new:N \lTblrRowFootInt \dim_new:N \lTblrTableWidthDim
\clist_new:N \lTblrRefMoreClist \tl_new:N \lTblrPortraitTypeTl \int_new:N \lTblrTablePageInt
\int_new:N \lTblrRowFirstInt \int_new:N \lTblrRowLastInt
\tl_new:N \lTblrDefaultHruleColorTl \tl_new:N \lTblrDefaultVruleColorTl
\int_new:N \lTblrCellRowSpanInt \int_new:N \lTblrCellColSpanInt
\tl_new:N \lTblrCellBackgroundTl \bool_new:N \lTblrCellOmittedBool
\tl_new:N \lTblrCellAboveBorderStyleTl \dim_new:N \lTblrCellAboveBorderWidthDim \tl_new:N \lTblrCellAboveBorderColorTl
\tl_new:N \lTblrCellBelowBorderStyleTl \dim_new:N \lTblrCellBelowBorderWidthDim \tl_new:N \lTblrCellBelowBorderColorTl
\tl_new:N \lTblrCellLeftBorderStyleTl \dim_new:N \lTblrCellLeftBorderWidthDim \tl_new:N \lTblrCellLeftBorderColorTl
\tl_new:N \lTblrCellRightBorderStyleTl \dim_new:N \lTblrCellRightBorderWidthDim \tl_new:N \lTblrCellRightBorderColorTl
\cs_new:Npn \ExpTblrChildId #1 {} \cs_new:Npn \ExpTblrChildClass #1 {}
\ExplSyntaxOff"
  );
  // The body preprocessing `\lx@tblr@env` runs when a table has `evaluate=`/`expand=`
  // (`\__tblr_modify_table_body:`, tabularray.sty:3567-3575): tabularray's own code on the body,
  // `\l__tblr_body_tl` — the expansion (:3579-3591) and the functional library's evaluation
  // (:8289-8306, `evaluate=all` :8275-8277). Without `\UseTblrLibrary{functional}` (its flag
  // `\lx@tblr@lib@functional@used`; loading the functional package alone is not enough) `evaluate` is no
  // outer key (:8267): tabularray's "Unknown outer key name" error (:4160-4176), the body as written.
  RawTeX!(
    r"\ExplSyntaxOn
\tl_new:N \l__tblr_body_tl \tl_new:N \l__tblr_expand_tl \tl_new:N \l__tblr_evaluate_tl
\tl_new:N \g__tblr_functional_result_tl
\msg_if_exist:nnF { tabularray } { unknown-outer-key }
  { \msg_new:nnn { tabularray } { unknown-outer-key } { Unknown ~ outer ~ key ~ name ~ '#1'. } }
\cs_new_protected:Npn \__tblr_expand_table_body:NN #1 #2
  {
    \tl_set_eq:NN \l_tmpa_tl #1
    \tl_clear:N #1
    \cs_set_protected:Npn \__tblr_expand_table_body_aux:w ##1 #2
      {
        \tl_put_right:Nn #1 {##1}
        \peek_meaning:NTF \q_stop
          { \use_none:n }
          { \exp_last_unbraced:NV \__tblr_expand_table_body_aux:w #2 }
      }
    \exp_last_unbraced:NV \__tblr_expand_table_body_aux:w \l_tmpa_tl #2 \q_stop
  }
\cs_new_protected:Npn \__tblr_evaluate_table_body:NN #1 #2
  {
    \tl_gclear:N \g__tblr_functional_result_tl
    \cs_set_protected:Npn \__tblr_evaluate_table_body_aux:w ##1 #2
      {
        \tl_gput_right:Nn \g__tblr_functional_result_tl {##1}
        \peek_meaning:NTF \q_stop { \use_none:n } {#2}
      }
    \fun_run_return_processor:nn
      { \exp_last_unbraced:NV \__tblr_evaluate_table_body_aux:w \gResultTl }
      { \exp_last_unbraced:NV \__tblr_evaluate_table_body_aux:w #1 #2 \q_stop }
    \tl_set_eq:NN #1 \g__tblr_functional_result_tl
  }
\cs_new_protected:Npn \lx@tblr@modify@body #1 #2
  {
    \tl_set:Nn \l__tblr_evaluate_tl {#1}
    \tl_if_empty:NF \l__tblr_evaluate_tl
      {
        \cs_if_exist:NTF \lx@tblr@lib@functional@used
          {
            \tl_if_eq:NnTF \l__tblr_evaluate_tl { all }
              { \tlSet \l__tblr_body_tl { \evalWhole {\expValue \l__tblr_body_tl} } }
              {
                \exp_last_unbraced:NNV
                \__tblr_evaluate_table_body:NN \l__tblr_body_tl \l__tblr_evaluate_tl
              }
          }
          { \msg_error:nnn { tabularray } { unknown-outer-key } { evaluate } }
      }
    \tl_set:Nn \l__tblr_expand_tl {#2}
    \tl_map_inline:Nn \l__tblr_expand_tl
      { \__tblr_expand_table_body:NN \l__tblr_body_tl ##1 }
  }
\ExplSyntaxOff"
  );
  // tabularray.sty:6104-6111 `\TblrNote{<tag>}`: the tag, superscript, overlapping to the right
  // (manual: 1 error).
  DefMacro!("\\TblrNote{}", "\\textsuperscript{#1}");
  // tabularray.sty:1603-1656 `\NewTblrTableCommand<cmd>[<n>][<default>]{<body>}` (`m O{0} o m`,
  // alias `\NewTableCommand`, :1644): a command a table's cells use to set its specification —
  // `\NewTblrTableCommand\myhline{\hline[0.1em,red5]}` (manual). It means its body only inside
  // a table (`\__tblr_enable_table_commands:`, :1646-1650, at the table's start, :3516): kept as
  // `\lx@tblr@tc@<cmd>`, listed in `\lx@tblr@table@commands`, and let to the command there
  // (`\lx@tblr@env`); undefined, `\myhline` was an error. The meaning is a `\newcommand`,
  // expandable as the row-start scan needs. tabularray's own `\pagebreak`/`\nopagebreak`
  // (:3129-3137, `[1][4]`) set a row's page break, which has no place here: the kernel's
  // started a cell, and the `\hline` after it came mid-row (4 errors in the manual).
  RawTeX!(
    r"\def\lx@tblr@table@commands{}
\NewDocumentCommand\NewTblrTableCommand{m O{0} o m}{%
\@ifundefined{lx@tblr@tc@\string#1}{}{\PackageError{tabularray}{Table command \string#1 already defined!}{}}%
\IfValueTF{#3}%
{\expandafter\newcommand\csname lx@tblr@tc@\string#1\endcsname[#2][#3]{#4}}%
{\expandafter\newcommand\csname lx@tblr@tc@\string#1\endcsname[#2]{#4}}%
\g@addto@macro\lx@tblr@table@commands{\do#1}}
\let\NewTableCommand\NewTblrTableCommand
\NewTblrTableCommand\pagebreak[1][4]{}\NewTblrTableCommand\nopagebreak[1][4]{}"
  );
  def_macro_noop("\\SetCell[]{}")?;
  def_macro_noop("\\SetCells[]{}")?;
  // The other `\NewTblrTableCommand`s (tabularray.sty:1613; :2990 `\SetRow`,
  // :2962 `\SetRows`, :2824 `\SetColumn`, :2796 `\SetColumns` = `O{} m`;
  // :2013 `\SetHline`, :1964 `\SetHlines`, :2256 `\SetVline`, :2208
  // `\SetVlines` = `O{+} m m`; :1695 `\SetChild` = `m`): the real extractor
  // (:3770-3860) gobbles the command and its arguments out of the cell before
  // alignment; undefined here they leaked an `<ltx:ERROR>` plus their braced
  // arguments into the first cell (manual :580/:591/:908).
  def_macro_noop("\\SetRow[]{}")?;
  def_macro_noop("\\SetRows[]{}")?;
  def_macro_noop("\\SetColumn[]{}")?;
  def_macro_noop("\\SetColumns[]{}")?;
  def_macro_noop("\\SetHline[]{}{}")?;
  def_macro_noop("\\SetHlines[]{}{}")?;
  def_macro_noop("\\SetVline[]{}{}")?;
  def_macro_noop("\\SetVlines[]{}{}")?;
  def_macro_noop("\\SetChild{}")?;
  // tabularray's styling (`\SetTblrStyle`) — a no-op stub.
  // Witness 2406.00523 (\SetTblrInner).
  DefMacro!("\\SetTblrStyle{}{}", "\\ignorespaces");
  // tabularray.sty:3461-3470: every tblr-family environment is built by one
  // factory (`\NewDocumentEnvironment{#1}{O{c} m +b}{\__tblr_environ_code…}`),
  // so a user environment is the same thing as `tblr` under another name.
  // ProfSio.sty:98 `\NewTblrEnviron{MPMtache}` then `\begin{MPMtache}{…}`
  // inside tikz pics (:105-134) — as a no-op the env was undefined and its
  // `&`/`\\` cascaded into 396 mode errors. Skip a name that already has a
  // meaning (the base `tblr`/`longtblr` the real package would create).
  DefMacro!("\\NewTblrEnviron{}", sub[(name)] {
    // The name is expanded (a `\str_use:N`, dlrg), as the real factory's `\NewDocumentEnvironment`
    // takes it.
    let n = Expand!(name).to_string();
    // TokenizeInternal!: `\@ifundefined` needs `@` as a letter.
    Ok(TokenizeInternal!(TeXString::assembled(format!(
      "\\@ifundefined{{{n}}}{{\\newenvironment{{{n}}}{{\\lx@tblr@env{{{n}}}}}{{\\lx@tblr@end}}}}{{}}"))))
  });
  // tabularray.sty:3291-3297 `\NewTblrColumnType{<name>}[<n>][<default>]
  // {<body>}` (`m O{0} o m`) and its alias `\NewColumnType`: recorded for the
  // colspec translation (`expand_tblr_column_type`). Row types have no place
  // in the reduction.
  // `\NewTblrColumnRowType`/`\NewColumnRowType` (:3306-3313) define the type
  // for rows too, which the reduction has no use for.
  for cs in [
    "\\NewTblrColumnType",
    "\\NewColumnType",
    "\\NewTblrColumnRowType",
    "\\NewColumnRowType",
  ] {
    // a `\NewDocumentCommand` (an assignment, not expandable)
    DefPrimitive!(&s!("{cs}{{}}[][]{{}}"), sub[args] {
      let [name, nargs, default, body]: [ArgWrap; 4] =
        [0, 1, 2, 3].map(|i| args.get(i).cloned().unwrap_or_default());
      let nargs = if nargs.is_none() { String::from("0") } else { nargs.to_string().trim().to_string() };
      let (has_default, default) = if default.is_none() {
        ("0", String::new())
      } else {
        ("1", default.to_string())
      };
      let fields: Vec<SymStr> = vec![
        pin(nargs),
        pin(String::from(has_default)),
        pin(default),
        pin(body.to_string()),
      ];
      assign_value(
        &tblr_column_type_key(name.to_string().trim()),
        Stored::Strings(fields.into()),
        Some(Scope::Global),
      );
      Ok(())
    });
  }
  // :4084-4092 `\NewTblrTheme{<name>}{<code>}`: the code runs where a table's outer spec says
  // `theme=<name>` (`\__tblr_use_theme:n`, :4094-4098), setting its templates and styles.
  RawTeX!(
    r"\long\def\NewTblrTheme#1#2{\long\expandafter\def\csname lx@tblr@theme@#1\endcsname{#2}\ignorespaces}"
  );
  // Template API (tabularray.sty:5655-5807): `\DeclareTblrTemplate` is the primary and
  // `\DefTblrTemplate` its alias (:5680); `\UseTblrTemplate` (:5783) expands a stored template,
  // `\MapTblrNotes`/`\MapTblrRemarks` (:5792/:5802) iterate the collected notes/remarks binding
  // the `\InsertTblr…Tag`/`…Text` token lists. Templates are print layout the tabular reduction
  // has no slot for: it prints a long or tall table's caption, notes and remarks itself
  // (`\lx@tblr@portrait`), and keeps only whether an element's template is empty, tabularray's own
  // or the document's, and the document's code, to see whether it calls `\caption` — a theme that
  // empties the head prints no caption (panda-doc.tex:42-47 `naked`). The map iterators and the
  // `\lTblr…Tl` caption/entry/label lists (:6387-6425) are inert. Witness tabularray-abnt
  // (`\DeclareTblrTemplate`/`\UseTblrTemplate`/`\MapTblrRemarks`/`\InsertTblrRemarkTag`
  // undefined).
  DefPrimitive!("\\lx@tblr@declare@template{}{}{}", sub[(elements, name, code)] {
    let name = name.to_string().trim().to_string();
    // the document's own template, which the reduction does not run (`custom`, its code kept: one
    // calling `\caption` writes the List of Tables line itself), or an empty one
    let state = if code.to_string().trim().is_empty() { "empty" } else { "custom" };
    for element in tblr_template_elements(&elements) {
      // the template named `default` is the element's default itself (:5670-5672)
      let suffix = if name == "default" { element.clone() } else { s!("{element}:{name}") };
      assign_value(&s!("tblr_template:{suffix}"), String::from(state), None);
      assign_value(&s!("tblr_template_code:{suffix}"), Stored::Tokens(code.clone()), None);
    }
    Ok(())
  });
  // :5673-5680 (`m m +m`, ending in `\ignorespaces`; `\DefTblrTemplate` its alias)
  DefMacro!(
    "\\DeclareTblrTemplate{}{}{}",
    "\\lx@tblr@declare@template{#1}{#2}{#3}\\ignorespaces"
  );
  Let!("\\DefTblrTemplate", "\\DeclareTblrTemplate");

  // tabularray's own templates (:5838-6298), and each element's default (`\SetTblrTemplate`): an
  // element's templates and its default are recorded by state, `empty` or tabularray's `code`.
  for (elements, names, default) in TBLR_BUILTIN_TEMPLATES {
    for element in tblr_template_elements(&Tokens::new(ExplodeText!(elements))) {
      for name in names {
        let state = if *name == "empty" { "empty" } else { "code" };
        assign_value(
          &s!("tblr_template:{element}:{name}"),
          String::from(state),
          Some(Scope::Global),
        );
      }
      if !default.is_empty() {
        assign_value(
          &s!("tblr_template:{element}"),
          String::from(if default == "empty" { "empty" } else { "code" }),
          Some(Scope::Global),
        );
      }
    }
  }
  // :5697-5722 `\SetTblrTemplate{<elements>}{<name>}` makes the named template the default; a name
  // the element has no template of is an error (`template-undefined`), which changes nothing.
  DefPrimitive!("\\lx@tblr@set@template{}{}", sub[(elements, name)] {
    let name = name.to_string().trim().to_string();
    for element in tblr_template_elements(&elements) {
      let named = if name == "default" { s!("tblr_template:{element}") } else { s!("tblr_template:{element}:{name}") };
      let Some(template) = lookup_value(&named) else {
        Error!("undefined", &name, s!("Undefined template \"{name}\" for element \"{element}\"."));
        continue;
      };
      // the default is the named template's state — `empty`, tabularray's own `code`, or the
      // document's `custom` with its code (a redeclared `empty` template is the document's too)
      if name != "default" {
        assign_value(&s!("tblr_template:{element}"), template.to_string(), None);
        let code = match lookup_value(&s!("tblr_template_code:{element}:{name}")) {
          Some(Stored::Tokens(code)) => code,
          _ => Tokens!(),
        };
        assign_value(&s!("tblr_template_code:{element}"), Stored::Tokens(code), None);
      }
    }
    Ok(())
  });
  DefMacro!(
    "\\SetTblrTemplate{}{}",
    "\\lx@tblr@set@template{#1}{#2}\\ignorespaces"
  );

  def_macro_noop("\\UseTblrTemplate{}{}")?;
  def_macro_noop("\\MapTblrNotes{}")?;
  def_macro_noop("\\MapTblrRemarks{}")?;
  for cs in [
    "\\InsertTblrNoteTag",
    "\\InsertTblrNoteText",
    "\\InsertTblrRemarkTag",
    "\\InsertTblrRemarkText",
    "\\lTblrCaptionTl",
    "\\lTblrEntryTl",
    "\\lTblrLabelTl",
  ] {
    def_macro_noop(cs)?;
  }
});
