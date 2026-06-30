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
/// `None` on anything it does not fully understand** (e.g. `S` siunitx columns),
/// so the caller falls back to the unchanged stub behaviour — the column count
/// is therefore always either correct or exactly as before, never worse.
fn translate_tblr_colspec(inner: &str) -> Option<String> {
  let spec = extract_colspec_value(inner)?;
  parse_colspec(&spec)
}

/// Find `colspec` in the key-value inner spec and return its value text.
/// Handles `colspec={…}` (brace-balanced) and `colspec=…` (until top-level comma).
fn extract_colspec_value(inner: &str) -> Option<String> {
  let idx = inner.find("colspec")?;
  let after = inner[idx + "colspec".len()..].trim_start();
  let after = after.strip_prefix('=')?.trim_start();
  if let Some(rest) = after.strip_prefix('{') {
    let mut depth = 1usize;
    for (i, ch) in rest.char_indices() {
      match ch {
        '{' => depth += 1,
        '}' => {
          depth -= 1;
          if depth == 0 {
            return Some(rest[..i].to_string());
          }
        },
        _ => {},
      }
    }
    None // unbalanced
  } else {
    Some(after.split(',').next().unwrap_or(after).trim().to_string())
  }
}

/// Parse a tabularray colspec body into a classic `\tabular` template, or `None`
/// if it contains a construct we don't translate (bail → stub fallback).
fn parse_colspec(spec: &str) -> Option<String> {
  let b = spec.as_bytes();
  let mut i = 0;
  let mut cols = String::new();
  while i < b.len() {
    let c = b[i] as char;
    match c {
      ' ' | '\t' | '\n' | '\r' => i += 1,
      '|' => {
        cols.push('|');
        i += 1;
      },
      'c' | 'l' | 'r' => {
        cols.push(c);
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
          if opts.contains('c') {
            align = 'c';
          } else if opts.contains('r') {
            align = 'r';
          } else if opts.contains('l') {
            align = 'l';
          }
          i = j + 1;
        }
        cols.push(align);
      },
      // p/m/b{width}: copy verbatim (classic understands these).
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
        } else {
          return None; // `p` without a width is not classic-valid
        }
      },
      // *{n}{sub}: repeat the sub-spec n times.
      '*' => {
        i += 1;
        let n = parse_braced_uint(b, spec, &mut i)?;
        let sub = parse_braced_group(b, spec, &mut i)?;
        let sub_cols = parse_colspec(&sub)?;
        for _ in 0..n {
          cols.push_str(&sub_cols);
        }
      },
      _ => return None, // unknown column type → bail to the stub
    }
  }
  if cols.is_empty() { None } else { Some(cols) }
}

/// Parse a `{<digits>}` group at `*i`, advancing past it. Returns the integer.
fn parse_braced_uint(b: &[u8], spec: &str, i: &mut usize) -> Option<usize> {
  let g = parse_braced_group(b, spec, i)?;
  g.trim()
    .parse::<usize>()
    .ok()
    .filter(|&n| n > 0 && n <= 1000)
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
    // Bail (→ None → caller keeps the stub behaviour) on unhandled constructs.
    assert_eq!(
      translate_tblr_colspec("colspec={S[table-format=2.1]c}"),
      None
    ); // siunitx S
    assert_eq!(translate_tblr_colspec("hlines,vlines"), None); // no colspec
    assert_eq!(translate_tblr_colspec("colspec={Q[c]z}"), None); // unknown 'z'
  }
}

LoadDefinitions!({
  Warn!(
    "missing_file",
    "tabularray.sty",
    "tabularray.sty is not implemented and will not be interpreted raw."
  );
  RequirePackage!("booktabs");
  // `\tblr` maps to `\tabular`, but tabularray's argument is a key-value inner
  // spec (`colspec={Q[c]Q[c]},hlines,…`), NOT a classic column template. Parse
  // out `colspec` and translate it so `\tabular` gets the right column count;
  // fall back to the bare inner spec (the historical stub behaviour) for specs
  // we don't fully translate. `[]{}` captures the optional outer spec (ignored,
  // as before) + the mandatory inner spec. See `translate_tblr_colspec`.
  DefMacro!("\\tblr []{}", sub[(_outer, inner)] {
    let inner_str = inner.to_string();
    let cols = translate_tblr_colspec(&inner_str).unwrap_or(inner_str);
    Ok(Tokenize!(&format!("\\tabular{{{cols}}}")))
  });
  DefMacro!("\\endtblr", "\\endtabular");
  DefMacro!("\\booktabs", "\\tabular");
  DefMacro!("\\endbooktabs", "\\endtabular");
  DefMacro!("\\UseTblrLibrary", "\\usepackage");
  def_macro_noop("\\SetCell[]{}")?;
  def_macro_noop("\\SetCells[]{}")?;
  // tabularray styling primitives — no-op stubs.
  // Witness 2406.00523 (\SetTblrInner).
  def_macro_noop("\\SetTblrInner[]{}")?;
  def_macro_noop("\\SetTblrOuter[]{}")?;
  def_macro_noop("\\SetTblrStyle{}{}")?;
  def_macro_noop("\\NewTblrEnviron{}")?;
  def_macro_noop("\\NewColumnType{}[]{}")?;
  def_macro_noop("\\NewTblrTheme{}{}")?;
  def_macro_noop("\\DefTblrTemplate{}{}{}")?;
  def_macro_noop("\\SetTblrTemplate{}{}")?;
});
