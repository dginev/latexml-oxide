use std::cell::{Cell, RefCell};

use latexml_package::{package::color_sty::parse_color, prelude::*};

use crate::discard_env::discard_env_body;

/// A rectangle of cells to fill, in nicematrix's 1-based MAIN-matrix
/// coordinates (the `first-row`/`first-col` label line is excluded). `r2`/`c2`
/// may be `i32::MAX` to mean "through the last row/column" (`\rowcolor`,
/// `\columncolor`, `\arraycolor`); clamped to the real cell count at apply time.
#[derive(Clone)]
struct NiceRect {
  bg: String,
  r1: i32,
  c1: i32,
  r2: i32,
  c2: i32,
}

thread_local! {
  /// `\CodeBefore` color rectangles recorded for the matrix currently being
  /// built. Reset at each `\begin{<x>NiceMatrix}` (`\lx@nice@setopts`), drained
  /// and painted onto the built `ltx:XMArray` by `\lx@nicematrix@applycolors`.
  /// thread_local + `RefCell` per project rule (never a `Mutex`).
  static NICE_RECTS: RefCell<Vec<NiceRect>> = const { RefCell::new(Vec::new()) };
  /// Whether the current matrix declared `first-row` / `first-col`: shifts the
  /// coordinate origin by one and marks the label line `thead`.
  static NICE_FIRST_ROW: Cell<bool> = const { Cell::new(false) };
  static NICE_FIRST_COL: Cell<bool> = const { Cell::new(false) };
}

/// Parse an `i-j` cell coordinate ("row-col").
fn nice_parse_cell(s: &str) -> Option<(i32, i32)> {
  let (a, b) = s.trim().split_once('-')?;
  Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// Parse a comma list of `i` / `i-j` items into inclusive `(lo, hi)` ranges
/// (`\rowcolor{2,4-5}` → `[(2,2),(4,5)]`).
fn nice_ranges(list: &str) -> Vec<(i32, i32)> {
  list
    .split(',')
    .filter_map(|item| {
      let item = item.trim();
      if item.is_empty() {
        return None;
      }
      match item.split_once('-') {
        Some((a, b)) => Some((a.trim().parse().ok()?, b.trim().parse().ok()?)),
        None => {
          let n = item.parse().ok()?;
          Some((n, n))
        },
      }
    })
    .collect()
}

/// Record a fill rectangle, normalizing corner order.
fn nice_push_rect(bg: &str, r1: i32, c1: i32, r2: i32, c2: i32) {
  NICE_RECTS.with(|cell| {
    cell.borrow_mut().push(NiceRect {
      bg: bg.to_string(),
      r1: r1.min(r2),
      c1: c1.min(c2),
      r2: r1.max(r2),
      c2: c1.max(c2),
    });
  });
}

// The `\rectanglecolor` recorders run at DIGESTION (DefPrimitive); the color-walk
// runs at CONSTRUCTION (DefConstructor) — two separate phases. So the whole
// document is digested (every matrix's CodeBefore recording into NICE_RECTS,
// cleared between matrices) BEFORE any color-walk runs; by then the thread_local
// holds only the last matrix's rects. To bridge the phase boundary, each matrix's
// `\lx@nicematrix@applycolors` snapshots ITS rects + flags into a whatsit property
// (in the digest-time `properties` closure) and reads them back at construction.
// `#` is disallowed in the encoding fields anyway (hex has no `,`/`|`/`;`).

/// Encode `(first_row, first_col, rects)` as a flat whatsit-property string.
fn nice_encode(fr: bool, fc: bool, rects: &[NiceRect]) -> String {
  let body: Vec<String> = rects
    .iter()
    .map(|r| format!("{},{},{},{},{}", r.r1, r.c1, r.r2, r.c2, r.bg))
    .collect();
  format!("{};{};{}", u8::from(fr), u8::from(fc), body.join("|"))
}

/// Whether a comma-separated key list contains `flag` as its own key — matching
/// the exact key (before any `=`), so `first-row` is NOT satisfied by
/// nicematrix's unrelated `code-for-first-row` styling key.
fn nice_opts_has(opts: &str, flag: &str) -> bool {
  opts.split(',').any(|tok| {
    let key = tok.split('=').next().unwrap_or("").trim();
    key == flag
  })
}

/// Inverse of [`nice_encode`].
fn nice_decode(data: &str) -> (bool, bool, Vec<NiceRect>) {
  let mut it = data.splitn(3, ';');
  let fr = it.next() == Some("1");
  let fc = it.next() == Some("1");
  let rects = it
    .next()
    .unwrap_or("")
    .split('|')
    .filter(|s| !s.is_empty())
    .filter_map(|s| {
      let f: Vec<&str> = s.splitn(5, ',').collect();
      if f.len() != 5 {
        return None;
      }
      Some(NiceRect {
        r1: f[0].parse().ok()?,
        c1: f[1].parse().ok()?,
        r2: f[2].parse().ok()?,
        c2: f[3].parse().ok()?,
        bg: f[4].to_string(),
      })
    })
    .collect();
  (fr, fc, rects)
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("pgfcore");
  RequirePackage!("amsmath");
  RequirePackage!("array");
  Warn!(
    "missing_file",
    "nicematrix.sty",
    "nicematrix.sty is not implemented and will not be interpreted raw."
  );

  // The tabular-like environments (`\NiceTabular`, `\NiceArray`, …) still degrade
  // to a placeholder-or-\tabular; the math Nice* MATRIX family below renders as a
  // real math array. NiceTabular[opts]{colspec}[opts] is nicematrix's tabular-like
  // environment; the real nicematrix.sty (L3806-3841) reduces it to \NiceArray
  // under a text-mode tabular flag. Degrade faithfully to \tabular with the SAME
  // colspec, dropping the nicematrix-only [opts] — recovers real tables (witnesses
  // 2605.08776, 2605.13835, 2605.18423) instead of discarding the body +
  // Error:undefined. Beyond-Perl: the ar5iv nicematrix.sty.ltxml stub still errors
  // here; mirror this upgrade there for strict Rust<->ar5iv parity.
  DefMacro!("\\NiceTabular[]{}[]", "\\tabular{#2}", locked => true);
  DefMacro!("\\endNiceTabular", "\\endtabular", locked => true);

  //======================================================================
  // #6569: the math Nice* MATRIX family renders as real bracketed math arrays,
  // with `\CodeBefore` cell coloring and `first-row`/`first-col` header marking.
  // Beyond-Perl (no Perl nicematrix binding). OXIDIZED_DESIGN divergence.
  //
  // A `\begin{<x>NiceMatrix}[opts] \CodeBefore <color-cmds> \Body <rows> \end{...}`
  // reduces to the amsmath matrix flavour for the delimiter <x> (b->[], p->(),
  // B->{}, v->||, V->‖‖, plain->none), so the entries render through the real
  // math-array engine (amsmath_sty.rs:384-391, base_xmath.rs:1151) instead of a
  // discarded placeholder. `\CodeBefore … \Body` (nicematrix.sty:1772-1780,2042)
  // records background rectangles; `\lx@nicematrix@applycolors` paints them onto
  // the built XMArray afterward. `[first-row,first-col]` (counters 490-492,904-905)
  // shift the origin and mark the label cells `thead`.
  //
  // Limitation: only the color commands in `\CodeBefore` are interpreted; other
  // decorations (`\tikz`, `\SubMatrix`, …) are undefined and may emit
  // Error:undefined (the matrix itself still renders). A Nice matrix NESTED inside
  // another Nice matrix's cell is not fully supported: the inner `\begin` clears
  // the shared thread_local, so the outer loses its recorded `\CodeBefore` rects
  // and both share one first-row/first-col flag pair (inner wins). Sibling
  // matrices in one display ARE handled (each paints its own array).

  // first-row/first-col detection + per-matrix reset. Runs at each
  // \begin{<x>NiceMatrix}: clears the rect list and records whether [opts]
  // requested a label line.
  DefPrimitive!("\\lx@nice@setopts{}", sub[(opts)] {
    let opts = do_expand(opts)?.to_string();
    NICE_RECTS.with(|c| c.borrow_mut().clear());
    NICE_FIRST_ROW.with(|c| c.set(nice_opts_has(&opts, "first-row")));
    NICE_FIRST_COL.with(|c| c.set(nice_opts_has(&opts, "first-col")));
    Ok(Vec::new())
  });

  // \CodeBefore color commands. Each records a fill rectangle (main-matrix 1-based
  // coords) into NICE_RECTS during CodeBefore digestion. Color specs reuse
  // color.sty's parse_color (xcolor `!`-algebra), so `blue!15` resolves exactly as
  // elsewhere (DRY). nicematrix.sty: \rectanglecolor 5752, \cellcolor, \rowcolor,
  // \columncolor, \arraycolor.
  DefPrimitive!("\\lx@nice@rectanglecolor[]{}{}{}", sub[(model, color, corner1, corner2)] {
    let model = model.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let hex = parse_color(model.as_deref(), &do_expand(color)?.to_string()).to_attribute();
    let a = do_expand(corner1)?.to_string();
    let b = do_expand(corner2)?.to_string();
    if let (Some((r1, c1)), Some((r2, c2))) = (nice_parse_cell(&a), nice_parse_cell(&b)) {
      nice_push_rect(&hex, r1, c1, r2, c2);
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\lx@nice@cellcolor[]{}{}", sub[(model, color, cells)] {
    let model = model.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let hex = parse_color(model.as_deref(), &do_expand(color)?.to_string()).to_attribute();
    for item in do_expand(cells)?.to_string().split(',') {
      if let Some((r, c)) = nice_parse_cell(item) {
        nice_push_rect(&hex, r, c, r, c);
      }
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\lx@nice@rowcolor[]{}{}", sub[(model, color, rows)] {
    let model = model.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let hex = parse_color(model.as_deref(), &do_expand(color)?.to_string()).to_attribute();
    for (lo, hi) in nice_ranges(&do_expand(rows)?.to_string()) {
      nice_push_rect(&hex, lo, 1, hi, i32::MAX);
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\lx@nice@columncolor[]{}{}", sub[(model, color, cols)] {
    let model = model.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let hex = parse_color(model.as_deref(), &do_expand(color)?.to_string()).to_attribute();
    for (lo, hi) in nice_ranges(&do_expand(cols)?.to_string()) {
      nice_push_rect(&hex, 1, lo, i32::MAX, hi);
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\lx@nice@arraycolor[]{}", sub[(model, color)] {
    let model = model.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let hex = parse_color(model.as_deref(), &do_expand(color)?.to_string()).to_attribute();
    nice_push_rect(&hex, 1, 1, i32::MAX, i32::MAX);
    Ok(Vec::new())
  });
  // Recognized-but-unpainted \CodeBefore color commands: gobble args (no-op) so a
  // paper using them keeps rendering instead of erroring. nicematrix.sty:
  // \chessboardcolors, \rowlistcolors.
  DefMacro!("\\lx@nice@chessboardcolors[]{}{}", "", locked => true);
  DefMacro!("\\lx@nice@rowlistcolors[]{}{}", "", locked => true);

  // Paint the recorded \CodeBefore rectangles onto the just-built XMArray, and mark
  // first-row/first-col label cells thead. A no-output DefConstructor (like
  // colortbl's \lxsetcellcolor) run right after \lx@end@ams@matrix: the ams matrix
  // constructor has already emitted the whole XMDual>XMWrap>XMArg>XMArray tree
  // synchronously, so the array is in the document and navigable here. The
  // digest-time `properties` closure drains THIS matrix's rects+flags (before the
  // next matrix's \lx@nice@setopts clears the thread_local) and stashes them on the
  // whatsit; the constructor reads them back — the only reliable bridge across the
  // digest→construct phase gap. Cells carry `backgroundcolor`/`thead` on XMCell
  // (schema LaTeXML-math.rnc:330,352). The array is this math's direct
  // XMDual>XMWrap>XMArg>XMArray child (needXMDual always set via datameaning=matrix).
  DefConstructor!("\\lx@nicematrix@applycolors",
    sub[document, _args, props] {
      let data = match props.get("nice_data") {
        Some(Stored::String(s)) => to_string(*s),
        _ => String::new(),
      };
      let (first_row, first_col, rects) = nice_decode(&data);
      // `current` is the shared math parent (ltx:XMath), not the matrix just
      // closed — a display can hold several matrix XMDuals (`\[ A \quad B \]`,
      // or a plain `pmatrix` beside a Nice matrix). The matrix we just built is
      // the LAST matrix-XMDual child at construct time (later matrices aren't
      // constructed yet), so pick `[last()]` — NOT the first, which would paint
      // an earlier (possibly unrelated) matrix. PR-review witness.
      let current = document.get_node().clone();
      let array = document
        .findnode(
          "ltx:XMDual[ltx:XMWrap/ltx:XMArg/ltx:XMArray][last()]/ltx:XMWrap/ltx:XMArg/ltx:XMArray",
          Some(&current),
        )
        .or_else(|| document.findnode("(descendant::ltx:XMArray)[last()]", Some(&current)));
      if let Some(array) = array {
        let rows = document.findnodes("ltx:XMRow", Some(&array));
        let n_rows = rows.len() as i32;
        let n_cols = rows
          .first()
          .map(|r| document.findnodes("ltx:XMCell", Some(r)).len() as i32)
          .unwrap_or(0);
        let row_off = i32::from(first_row);
        let col_off = i32::from(first_col);
        // Main-matrix extent (label line excluded).
        let main_rows = n_rows - row_off;
        let main_cols = n_cols - col_off;
        for rect in &rects {
          let r_hi = rect.r2.min(main_rows);
          let c_hi = rect.c2.min(main_cols);
          for i in rect.r1.max(1)..=r_hi {
            let ri = (i - 1 + row_off) as usize;
            let Some(row) = rows.get(ri) else { continue };
            let cells = document.findnodes("ltx:XMCell", Some(row));
            for j in rect.c1.max(1)..=c_hi {
              let ci = (j - 1 + col_off) as usize;
              if let Some(cell) = cells.get(ci) {
                let mut cell = cell.clone();
                document.set_attribute(&mut cell, "backgroundcolor", &rect.bg)?;
              }
            }
          }
        }
        // first-row → column headers; first-col → row headers; corner → both.
        if first_row || first_col {
          for (ri, row) in rows.iter().enumerate() {
            let cells = document.findnodes("ltx:XMCell", Some(row));
            for (ci, cell) in cells.iter().enumerate() {
              let col_head = first_row && ri == 0;
              let row_head = first_col && ci == 0;
              if col_head || row_head {
                let mut thead = String::new();
                if col_head {
                  thead.push_str("column");
                }
                if col_head && row_head {
                  thead.push(' ');
                }
                if row_head {
                  thead.push_str("row");
                }
                let mut cell = cell.clone();
                document.set_attribute(&mut cell, "thead", &thead)?;
              }
            }
          }
        }
      }
    },
    // Digest-time: snapshot THIS matrix's rects+flags off the thread_local before
    // the next \begin's \lx@nice@setopts clears it.
    properties => {
      let fr = NICE_FIRST_ROW.with(|c| c.get());
      let fc = NICE_FIRST_COL.with(|c| c.get());
      let rects = NICE_RECTS.with(|c| std::mem::take(&mut *c.borrow_mut()));
      let mut props = stored_map!();
      props.insert("nice_data", Stored::String(pin(nice_encode(fr, fc, &rects))));
      Ok(props)
    },
    alias => "");

  // `\lx@nice@matrix@begin{keys}` consumes an optional `\CodeBefore … \Body`
  // pre-layer before starting the matrix. In the grab, the color commands are
  // rebound to the recording primitives inside a group (so colortbl's
  // \cellcolor/\rowcolor/\columncolor in text tabulars are untouched), the
  // CodeBefore block (#2) is executed to record its rectangles, then the matrix
  // starts. `\CodeBefore`/`\Body` are \relax (harmless marker / delimiter).
  Let!("\\CodeBefore", "\\relax");
  Let!("\\Body", "\\relax");
  RawTeX!(concat!(
    r"\def\lx@nice@matrix@begin#1{",
      r"\@ifnextchar\CodeBefore{\lx@nice@grabcode{#1}}{\lx@ams@matrix{#1}}}",
    r"\long\def\lx@nice@grabcode#1#2\Body{",
      r"\begingroup",
        r"\let\rectanglecolor\lx@nice@rectanglecolor",
        r"\let\cellcolor\lx@nice@cellcolor",
        r"\let\rowcolor\lx@nice@rowcolor",
        r"\let\columncolor\lx@nice@columncolor",
        r"\let\arraycolor\lx@nice@arraycolor",
        r"\let\chessboardcolors\lx@nice@chessboardcolors",
        r"\let\rowlistcolors\lx@nice@rowlistcolors",
        r"#2",
      r"\endgroup",
      r"\lx@ams@matrix{#1}}"
  ));

  // The MATRIX family: `\<x>NiceMatrix[opts]` → set opts, then reduce to the
  // amsmath matrix flavour for delimiter <x>; `\end<x>NiceMatrix` closes the array
  // and paints the recorded colors. Mirrors amsmath_sty.rs:393-427.
  DefMacro!("\\NiceMatrix[]",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=NiceMatrix,datameaning=matrix}",
    locked => true);
  DefMacro!("\\endNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\pNiceMatrix[]",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=pNiceMatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right)}",
    locked => true);
  DefMacro!("\\endpNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\bNiceMatrix[]",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=bNiceMatrix,datameaning=matrix,left=\\lx@left[,right=\\lx@right]}",
    locked => true);
  DefMacro!("\\endbNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\BNiceMatrix[]",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=BNiceMatrix,datameaning=matrix,left=\\lx@left\\{,right=\\lx@right\\}}",
    locked => true);
  DefMacro!("\\endBNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\vNiceMatrix[]",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=vNiceMatrix,delimitermeaning=determinant,datameaning=matrix,left=\\lx@left|,right=\\lx@right|}",
    locked => true);
  DefMacro!("\\endvNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\VNiceMatrix[]",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=VNiceMatrix,delimitermeaning=norm,datameaning=matrix,left=\\lx@left\\|,right=\\lx@right\\|}",
    locked => true);
  DefMacro!("\\endVNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@applycolors", locked => true);

  // The ARRAY family (`\NiceArray`/`pNiceArray`/…/`NiceArrayWithDelims`/
  // `NiceTabular*`/`NiceTabularX`) takes a `{colspec}` and stays a placeholder
  // stub for now (no faithful colspec reduction yet).
  DefConstructor!(T_CS!("\\begin{NiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceArrayWithDelims}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceArrayWithDelims (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceArrayWithDelims", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceArrayWithDelims", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceTabular*}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceTabular* (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceTabular*", "nicematrix.sty.ltxml")?; });
  DefConstructor!(T_CS!("\\begin{pNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">pNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("pNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endpNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{NiceTabularX}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">NiceTabularX (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("NiceTabularX", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endNiceTabularX", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{bNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">bNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("bNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endbNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{BNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">BNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("BNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endBNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{vNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">vNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("vNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endvNiceArray", "\\relax", locked => true);
  DefConstructor!(T_CS!("\\begin{VNiceArray}"), None,
    "<ltx:note role=\"nicematrix-placeholder\">VNiceArray (nicematrix)</ltx:note>",
    bounded => true, mode => "text", locked => true,
    before_digest => { discard_env_body("VNiceArray", "nicematrix.sty.ltxml")?; });
  DefMacro!("\\endVNiceArray", "\\relax", locked => true);
  // Configuration entry-points — `\NiceMatrixOptions{...}` /
  // `\NewCollectionOfColumnsType{...}` etc. set internal styling keys that are
  // visually irrelevant to our rendering. No-op stubs prevent Error:undefined for
  // papers that call them in their preamble. Witness 2312.01047.
  def_macro_noop("\\NiceMatrixOptions{}")?;
  def_macro_noop("\\NewCollectionOfColumnsType{}{}")?;
  def_macro_noop("\\RenewCollectionOfColumnsType{}{}")?;
  def_macro_noop("\\nicematrixoptions{}")?;
});

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn opts_flag_matches_key_not_substring() {
    assert!(nice_opts_has("first-row, first-col", "first-row"));
    assert!(nice_opts_has("first-row , first-col", "first-col"));
    assert!(nice_opts_has("first-row = 2", "first-row"));
    // `code-for-first-row` must NOT count as `first-row` (it is a styling key).
    assert!(!nice_opts_has(
      "code-for-first-row = {\\color{red}}",
      "first-row"
    ));
    assert!(!nice_opts_has("code-for-first-col = {}", "first-col"));
    assert!(!nice_opts_has("", "first-row"));
    assert!(!nice_opts_has("columns-width = auto", "first-row"));
  }

  #[test]
  fn encode_decode_round_trips_rects_and_flags() {
    let rects = vec![
      NiceRect {
        bg: "#D9D9FF".into(),
        r1: 1,
        c1: 1,
        r2: 1,
        c2: 1,
      },
      NiceRect {
        bg: "#FF9999".into(),
        r1: 2,
        c1: 1,
        r2: 3,
        c2: i32::MAX,
      },
    ];
    let (fr, fc, back) = nice_decode(&nice_encode(true, false, &rects));
    assert!(fr && !fc);
    assert_eq!(back.len(), 2);
    assert_eq!(
      (back[1].r1, back[1].c2, back[1].bg.as_str()),
      (2, i32::MAX, "#FF9999")
    );
    // Empty round-trips to no rects.
    let (_, _, none) = nice_decode(&nice_encode(false, false, &[]));
    assert!(none.is_empty());
  }
}
