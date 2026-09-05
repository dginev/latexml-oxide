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
  // tabularray shorthand: a mandatory argument with NO top-level `=` is
  // interpreted entirely as the colspec (`\begin{tblr}{Q[c]Q[c]}`).
  // (PR_READINESS should-fix 13 — this common form previously fell through
  // to the stub and kept the original alignment-leak failure mode.)
  let mut depth = 0usize;
  let mut has_top_eq = false;
  for ch in inner.chars() {
    match ch {
      '{' | '[' => depth += 1,
      '}' | ']' => depth = depth.saturating_sub(1),
      '=' if depth == 0 => {
        has_top_eq = true;
        break;
      },
      _ => {},
    }
  }
  if !has_top_eq {
    let trimmed = inner.trim();
    if trimmed.is_empty() {
      return None;
    }
    return Some(trimmed.to_string());
  }
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
  let mut total = 0usize;
  parse_colspec_capped(spec, 0, &mut total)
}

/// Caps: recursion depth ≤ 8 and ≤ 512 total columns — nested `*{n}{…}`
/// multiplies past any per-level cap (`*{1000}{*{1000}{c}}`), and deep
/// `*{1}{…}` nesting is otherwise unbounded recursion (PR_READINESS
/// should-fix 13). Exceeding a cap bails to the stub, like any other
/// untranslatable spec.
fn parse_colspec_capped(spec: &str, depth: usize, total: &mut usize) -> Option<String> {
  if depth > 8 {
    return None;
  }
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
        // Count the sub-spec's columns once, then charge n× the delta so the
        // TOTAL cap holds under multiplication.
        let before = *total;
        let sub_cols = parse_colspec_capped(&sub, depth + 1, total)?;
        let per = *total - before;
        let extra = per.checked_mul(n.saturating_sub(1))?;
        *total = total.checked_add(extra)?;
        if *total > 512 {
          return None;
        }
        for _ in 0..n {
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
          cols.push_str(&spec[start..i]);
        } else {
          return None;
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
  }
}

LoadDefinitions!({
  RequirePackage!("booktabs");
  // `\tblr` maps to `\tabular`, but tabularray's argument is a key-value inner
  // spec (`colspec={Q[c]Q[c]},hlines,…`), NOT a classic column template. Parse
  // out `colspec` and translate it so `\tabular` gets the right column count;
  // fall back to the bare inner spec (the historical stub behaviour) for specs
  // we don't fully translate. `[]{}` captures the optional outer spec (ignored,
  // as before) + the mandatory inner spec. See `translate_tblr_colspec`.
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
  DefMacro!("\\lx@tblr@env{} []{}", sub[(env, _outer, inner)] {
    let env = env.to_string();
    let stored = lookup_definition(&T_CS!(s!("\\lx@tblr@inner@{env}")))?
      .and_then(|d| d.get_expansion().cloned())
      .map(|b| match b { ExpansionBody::Tokens(t) => t.to_string(), _ => String::new() })
      .unwrap_or_default();
    let inner_str = inner.to_string();
    let combined = if stored.trim().is_empty() {
      inner_str.clone()
    } else {
      format!("{stored},{inner_str}")
    };
    let cols = match translate_tblr_colspec(&combined) {
      Some(c) => c,
      None if extract_colspec_value(&combined).is_none() => String::from("*{32}{l}"),
      None => inner_str,
    };
    // tabularray parses its own body and tolerates a row wider than the
    // colspec (circularglyphs-doc.tex:196: `*{13}{X[m,c]}` with a 14-cell
    // last row; pdflatex clean, Perl raw-loads it clean). The kernel template
    // is only a hard cap — the final column count is the widest row and
    // short rows are padded — so a margin of fallback columns is inert on a
    // well-formed table and absorbs a ragged one. Guard:
    // `perfect_kernel_batch54::tblr_row_wider_than_the_colspec_is_tolerated`.
    // The margin continues the LAST column's alignment, as tabularray does.
    let last = cols.trim_end().chars().last().filter(|c| c.is_ascii_alphabetic()).unwrap_or('c');
    // tabularray.sty:2006/2008 `\NewTblrTableCommand \hline [1] []` /
    // `\cline [2] []`: inside a tblr both take an optional `[<style>]`
    // (`\hline[dashed]\hline`, manual :547). The kernel `\hline` the stub
    // reuses has no optional, so `[dashed]` became cell text and the next
    // `\hline`'s `\noalign` fired mid-cell (7-error cascade per demo;
    // RUST-ONLY — Perl raw-loads tabularray). Scoped to the environment's
    // group; the style itself is unrendered. Guard:
    // `perfect_kernel_batch56::tblr_hline_style_optional_is_absorbed`.
    Ok(TokenizeInternal!(TeXString::assembled(format!(
      "\\let\\lx@tblr@saved@hline\\hline\
       \\def\\hline{{\\@ifnextchar[\\lx@tblr@hline@opt\\lx@tblr@saved@hline}}\
       \\def\\lx@tblr@hline@opt[#1]{{\\lx@tblr@saved@hline}}\
       \\let\\lx@tblr@saved@cline\\cline\
       \\def\\cline{{\\@ifnextchar[\\lx@tblr@cline@opt\\lx@tblr@saved@cline}}\
       \\def\\lx@tblr@cline@opt[#1]#2{{\\lx@tblr@saved@cline{{#2}}}}\
       \\tabular{{{cols}*{{16}}{{{last}}}}}"
    ))))
  });
  DefMacro!("\\tblr", "\\lx@tblr@env{tblr}");
  DefMacro!("\\endtblr", "\\endtabular");
  // tabularray.sty:3472-3477 creates `longtblr`/`talltblr` with the same
  // factory (`long`/`tall` outer specs add page-breaking + caption/notes
  // layout the tabular reduction has no slot for). Witness: panda manual
  // (`{longtblr}` undefined → 149 relational-token errors + EoF Fatal).
  DefMacro!("\\longtblr", "\\lx@tblr@env{longtblr}");
  DefMacro!("\\endlongtblr", "\\endtabular");
  DefMacro!("\\talltblr", "\\lx@tblr@env{talltblr}");
  DefMacro!("\\endtalltblr", "\\endtabular");
  DefMacro!("\\SetTblrInner []{}", sub[(envs, keys)] {
    let envs = envs.map(|e| e.to_string()).unwrap_or_else(|| String::from("tblr"));
    let keys = keys.to_string();
    for env in envs.split(',').map(str::trim).filter(|e| !e.is_empty()) {
      let cs = T_CS!(s!("\\lx@tblr@inner@{env}"));
      let prev = lookup_definition(&cs)?
        .and_then(|d| d.get_expansion().cloned())
        .map(|b| match b { ExpansionBody::Tokens(t) => t.to_string(), _ => String::new() })
        .unwrap_or_default();
      let merged = if prev.trim().is_empty() { keys.clone() } else { format!("{prev},{keys}") };
      def_macro(cs, None, ExpansionBody::Tokens(Tokenize!(TeXString::assembled(merged))), None)?;
    }
    Ok(Tokens!())
  });
  DefMacro!("\\booktabs", "\\tabular");
  DefMacro!("\\endbooktabs", "\\endtabular");
  DefMacro!("\\UseTblrLibrary", "\\usepackage");
  def_macro_noop("\\SetCell[]{}")?;
  def_macro_noop("\\SetCells[]{}")?;
  // tabularray styling primitives — no-op stubs.
  // Witness 2406.00523 (\SetTblrInner).
  def_macro_noop("\\SetTblrOuter[]{}")?;
  def_macro_noop("\\SetTblrStyle{}{}")?;
  // tabularray.sty:3461-3470: every tblr-family environment is built by one
  // factory (`\NewDocumentEnvironment{#1}{O{c} m +b}{\__tblr_environ_code…}`),
  // so a user environment is the same thing as `tblr` under another name.
  // ProfSio.sty:98 `\NewTblrEnviron{MPMtache}` then `\begin{MPMtache}{…}`
  // inside tikz pics (:105-134) — as a no-op the env was undefined and its
  // `&`/`\\` cascaded into 396 mode errors. Skip a name that already has a
  // meaning (the base `tblr`/`longtblr` the real package would create).
  DefMacro!("\\NewTblrEnviron{}", sub[(name)] {
    let n = name.to_string();
    // TokenizeInternal!: `\@ifundefined` needs `@` as a letter.
    Ok(TokenizeInternal!(TeXString::assembled(format!(
      "\\@ifundefined{{{n}}}{{\\newenvironment{{{n}}}{{\\lx@tblr@env{{{n}}}}}{{\\endtabular}}}}{{}}"))))
  });
  def_macro_noop("\\NewColumnType{}[]{}")?;
  def_macro_noop("\\NewTblrTheme{}{}")?;
  // Template API (tabularray.sty:5673-5807): `\DeclareTblrTemplate` is the
  // primary and `\DefTblrTemplate` its alias (:5680); `\UseTblrTemplate`
  // (:5783) expands a stored template, `\MapTblrNotes`/`\MapTblrRemarks`
  // (:5792/:5802) iterate the collected notes/remarks binding the
  // `\InsertTblr…Tag`/`…Text` token lists. Templates are print layout the
  // tabular reduction has no slot for, so the whole family is inert here —
  // including the map iterators (nothing is collected) and the `\lTblr…Tl`
  // caption/entry/label lists (:6387-6425). Witness tabularray-abnt
  // (`\DeclareTblrTemplate`/`\UseTblrTemplate`/`\MapTblrRemarks`/
  // `\InsertTblrRemarkTag` undefined); `\NewTblrLibrary` (:8030).
  def_macro_noop("\\DeclareTblrTemplate{}{}{}")?;
  def_macro_noop("\\DefTblrTemplate{}{}{}")?;
  def_macro_noop("\\SetTblrTemplate{}{}")?;
  def_macro_noop("\\UseTblrTemplate{}{}")?;
  def_macro_noop("\\MapTblrNotes{}")?;
  def_macro_noop("\\MapTblrRemarks{}")?;
  def_macro_noop("\\NewTblrLibrary{}{}")?;
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
