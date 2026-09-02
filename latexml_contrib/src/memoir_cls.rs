use latexml_package::prelude::*;

// memoir.cls is raw-interpreted through the engine (tlp/czjphys precedent).
//
// The former stub (`LoadClass!("book")` + ~40 page-geometry no-ops, kept at
// git history e5a46e1443^) hid the real class, whose command surface is
// enormous — `\onelineskip` (memoir.cls L62), `{vplace}` (L11305),
// `\cftbeforechapterskip` (L7429), `\HUGE`, `\setsecnumdepth`,
// `\chapterstyle`, `\xpretocmd`, `\makeoddhead`, the output-stream family
// (L10965-11063, content-bearing) … — so 22 of 24 oracle-clean memoir manuals
// in the perfect-kernel corpus errored on `undefined:\<memoir-CS>`
// (witnesses: titlepages 4→0, dlfltxbmarkup 3→0, memexsupp, the dlfltxb*
// family, biblatex-oxref oxalph/oxnum/oxyear-doc). The real class raw-loads
// with zero errors and yields the correct <chapter>/<section> structure, so
// the complete class beats the stub (policy: complete support over stubs).
// Keeping a binding — rather than deleting the file — makes memoir raw-load
// under BOTH `[rawclasses]` and the default (arXiv) configuration, where a
// bindingless class would otherwise fall to the OmniBus article base.
// Perl LaTeXML ships no memoir.cls.ltxml.
LoadDefinitions!({
  // memoir.cls:8811 redefines ONLY `\endminipage` (the classic latex.ltx box
  // closer plus minipage-footnote flushing), never `\minipage`. Our minipage
  // is a native constructor pair (latex_constructs.rs `\minipage`/`\endminipage`;
  // Perl latex_constructs.pool.ltxml:4771) whose begin sets no `\@mpargs`, so
  // the raw closer would `\egroup` the native mode frame and hand the still-live
  // dump `\@iiiparbox` (latex.ltx:16309) an undefined `\@mpargs` — its `Until:[`
  // scan then swallows the NEXT `[…]` in the document (tcolorbox captures the
  // closer via `\let\endtcb@lrbox=\endminipage`, tcolorbox.sty:1118 — witness
  // biblatex-oxref/oxalph-doc: 983× `\csname bm@bicolor ,colframe = …` + Fatal
  // TooManyErrors; Perl has no `\@iiiparbox` at all and merely errors). Keep
  // the native pair paired: save the closer around the raw class load and
  // restore it. Guard: `perfect_kernel_batch54::memoir_keeps_native_endminipage`.
  Let!("\\lx@memoir@saved@endminipage", "\\endminipage");
  InputDefinitions!("memoir", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  Let!("\\endminipage", "\\lx@memoir@saved@endminipage");

  // memoir's auto-table family (memoir.cls:5477-5719; manual §"Automatic
  // tables", memman.tex:20627-20886). All four build low-level alignments the
  // engine cannot intercept: `\autocols` does `\let\@sharp ##` + `\valign`
  // (:5654-5657; our `\valign` is a no-op — Perl TeX_Tables.pool:555 — so the
  // PARAM tokens reach the Stomach), `\autorows` the `\halign` analogue
  // (:5677), `\ctabular` (:5491) runs `\@mkpream` on array.sty internals
  // (`\@classz`/`\@acol`/`\col@sep`, deliberately absent — TeX_Tables.pool:627)
  // and borrows tabularx's `\TX@cols`/`\TX@col@width` scratch registers.
  // Reduce them to `\tabular`, the way tabularx_sty.rs reduces tabularx:
  //   * `\ctabular[pos]{fmt}` — a tabular that may break across pages; `pos`
  //     is HORIZONTAL (l/c/r), not tabular's vertical slot → dropped;
  //   * `\autorows[width]{pos}{num}{style}{items}` — `num` COLUMNS, filled
  //     row-major (memman.tex:1761-1765), last row short;
  //   * `\autocols[width]{pos}{num}{style}{items}` — `num` COLUMNS, filled
  //     column-major with `\linespercol` heights (:5665-5675: greedy
  //     `ceil(remaining / cols_left)` per column — column 0 tallest), then
  //     transposed to rows; `width` only sizes the columns.
  // Items are a brace-aware comma list (a literal comma needs `\comma`,
  // memman.tex:20852). Witnesses: memman (4 `\autorows`, 2 `\autocols`,
  // 2 `{ctabular}` → ~157 of its errors), AroundTheBend. Guards
  // `perfect_kernel_batch54::memoir_auto_tables_reduce_to_tabular`.
  DefMacro!("\\ctabular[Default:c]{}", "\\tabular{#2}");
  DefMacro!("\\endctabular", "\\endtabular");
  DefMacro!(
    T_CS!("\\ctabular*"),
    "[Default:c]{}{}",
    "\\csname tabular*\\endcsname{#2}{#3}"
  );
  DefMacro!(
    T_CS!("\\endctabular*"),
    None,
    "\\csname endtabular*\\endcsname"
  );
  DefMacro!("\\autorows[Default:0pt]{}{Number}{}{}", sub[(_width, _pos, num, style, items)] {
    let cols = (num.value_of().max(1)) as usize;
    let items = split_top_level_commas(items.into_tokens_result()?);
    let rows: Vec<Vec<Tokens>> = items.chunks(cols).map(|c| c.to_vec()).collect();
    Ok(auto_table(cols, style.into_tokens_result()?, rows))
  });
  DefMacro!("\\autocols[Default:0pt]{}{Number}{}{}", sub[(_width, _pos, num, style, items)] {
    let cols = (num.value_of().max(1)) as usize;
    let items = split_top_level_commas(items.into_tokens_result()?);
    // memoir.cls:5665-5675 `\linespercol`: column heights are assigned
    // greedily left to right, ceil(remaining / columns left).
    let mut heights = Vec::with_capacity(cols);
    let mut remaining = items.len();
    for left in (1..=cols).rev() {
      let h = if remaining > 0 { remaining.div_ceil(left) } else { 0 };
      heights.push(h);
      remaining -= h;
    }
    let mut offsets = Vec::with_capacity(cols);
    let mut acc = 0;
    for h in &heights {
      offsets.push(acc);
      acc += h;
    }
    let nrows = heights.first().copied().unwrap_or(0);
    let rows: Vec<Vec<Tokens>> = (0..nrows)
      .map(|r| {
        (0..cols)
          .map(|c| {
            if r < heights[c] {
              items[offsets[c] + r].clone()
            } else {
              Tokens::new(Vec::new())
            }
          })
          .collect()
      })
      .collect();
    Ok(auto_table(cols, style.into_tokens_result()?, rows))
  });
});

/// Split a brace-aware comma list into its items (top-level catcode-OTHER
/// commas only; a trailing whitespace-only item is dropped, as memoir's
/// `\@for` would yield an empty entry there).
fn split_top_level_commas(arg: Tokens) -> Vec<Tokens> {
  let comma = T_OTHER!(",");
  let mut parts: Vec<Tokens> = Vec::new();
  let mut current: Vec<Token> = Vec::new();
  let mut depth: i32 = 0;
  for t in arg.unlist() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && t == comma {
      parts.push(Tokens::new(std::mem::take(&mut current)));
    } else {
      current.push(t);
    }
  }
  if !current.iter().all(|t| t.get_catcode() == Catcode::SPACE) {
    parts.push(Tokens::new(current));
  }
  parts
}

/// `\tabular{<style>×cols} row & … \\ … \endtabular`, splicing the raw item
/// tokens (never a re-tokenized string, so `\cmd{\footnote}`-style items
/// survive).
fn auto_table(cols: usize, style: Tokens, rows: Vec<Vec<Tokens>>) -> Tokens {
  let style_tok = style
    .unlist()
    .into_iter()
    .find(|t| t.get_catcode() != Catcode::SPACE)
    .unwrap_or_else(|| T_LETTER!("l"));
  let mut out = vec![T_CS!("\\par"), T_CS!("\\tabular"), T_BEGIN!()];
  out.extend(std::iter::repeat_n(style_tok, cols));
  out.push(T_END!());
  for (i, row) in rows.iter().enumerate() {
    if i > 0 {
      out.push(T_CS!("\\\\"));
    }
    for (j, cell) in row.iter().enumerate() {
      if j > 0 {
        out.push(T_ALIGN!());
      }
      out.extend(cell.clone().unlist());
    }
  }
  out.push(T_CS!("\\endtabular"));
  out.push(T_CS!("\\par"));
  Tokens::new(out)
}
