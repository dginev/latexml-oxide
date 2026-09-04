use std::cell::{Cell as StdCell, RefCell};

use latexml_package::{package::color_sty::parse_color, prelude::*};

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
  static NICE_FIRST_ROW: StdCell<bool> = const { StdCell::new(false) };
  static NICE_FIRST_COL: StdCell<bool> = const { StdCell::new(false) };
  static NICE_NAME: RefCell<Option<String>> = const { RefCell::new(None) };
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

/// Extract the value for a `key=value` option in a comma-separated key list.
fn nice_opts_get(opts: &str, key: &str) -> Option<String> {
  for tok in opts.split(',') {
    let mut parts = tok.splitn(2, '=');
    let k = parts.next().unwrap_or("").trim();
    if k == key {
      return parts.next().map(|v| v.trim().to_string());
    }
  }
  None
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

/// Strip nicematrix's rule-option brackets from a colspec token stream:
/// `|[color=blue,start=2]` → `|` (nicematrix.sty attaches an optional
/// `[keys]` to the `|` specifier for rule color/thickness — styling the
/// standard template reader must not see, or every following letter of the
/// key text is miscounted as a column and the whole tabular desyncs — the
/// nicematrix manual's ×54 `Extra alignment tab` cascade + a readBalanced
/// runaway to EOF).
fn nice_strip_rule_opts(toks: Vec<Token>) -> Vec<Token> {
  let mut out: Vec<Token> = Vec::with_capacity(toks.len());
  let mut i = 0;
  while i < toks.len() {
    let t = toks[i];
    out.push(t);
    let is_bar = t.get_catcode() != Catcode::CS && t.with_str(|s| s == "|");
    if is_bar
      && let Some(next) = toks.get(i + 1)
      && next.get_catcode() != Catcode::CS
      && next.with_str(|s| s == "[")
    {
      // skip to the matching ]
      let mut j = i + 2;
      while j < toks.len()
        && !(toks[j].get_catcode() != Catcode::CS && toks[j].with_str(|s| s == "]"))
      {
        j += 1;
      }
      i = j + 1;
      continue;
    }
    i += 1;
  }
  out
}

/// Merge the two optional key lists of a `[opts]{colspec}[opts]` signature
/// (nicematrix.sty:2007 keys are set from both) into one comma list.
fn nice_merge_opts(a: Option<Tokens>, b: Option<Tokens>) -> Tokens {
  let mut out: Vec<Token> = Vec::new();
  for o in [a, b].into_iter().flatten() {
    let toks = o.unlist();
    if toks.is_empty() {
      continue;
    }
    if !out.is_empty() {
      out.push(T_OTHER!(","));
    }
    out.extend(toks);
  }
  Tokens::new(out)
}

/// `\begin{NiceTabular…}[opts]{colspec}[opts]` → `\lx@nice@setopts{opts}
/// \lx@nice@array@begin{<starter>{colspec'}}` with the rule-only column
/// options stripped from the colspec (see `\NiceTabular` below), and a `c`
/// grown on the side of `first-col`/`last-col` (the label column every
/// source row then carries — nicematrix.tex:2569/2617; without it each row
/// overflows the template: "Extra alignment tab"). Same growth as the array
/// family's `\NiceArrayWithDelims`.
fn nice_tabular_expansion(opts_toks: Tokens, pream: Vec<Token>, starter: Vec<Token>) -> Tokens {
  let opts_str = opts_toks.to_string();
  let mut out: Vec<Token> = Vec::new();
  out.push(T_CS!("\\lx@nice@setopts"));
  out.push(T_BEGIN!());
  out.extend(opts_toks.unlist());
  out.push(T_END!());
  out.push(T_CS!("\\lx@nice@array@begin"));
  out.push(T_BEGIN!());
  out.extend(starter);
  out.push(T_BEGIN!());
  if nice_opts_has(&opts_str, "first-col") {
    out.push(T_LETTER!("c"));
  }
  out.extend(nice_strip_rule_opts(pream));
  if nice_opts_has(&opts_str, "last-col") {
    out.push(T_LETTER!("c"));
  }
  out.push(T_END!());
  out.push(T_END!());
  Tokens::new(out)
}

LoadDefinitions!({
  RequirePackage!("pgfcore");
  RequirePackage!("amsmath");
  RequirePackage!("array");
  RequirePackage!("colortbl");
  // nicematrix's own preamble parser accepts `V{width}` (nicematrix.sty:2541:
  // a varwidth[t] cell when varwidth is loaded — varwidth.sty:308-313 only
  // registers its `\newcolumntype{V}` when array preceded it, which is the
  // uncommon order). Model it as the top-attached paragraph column `p{width}`
  // (tex_tables.rs `\lx@tabular@p t`): natural-width-up-to is print layout.
  // Unregistered, the reader's "safety valve" re-read `3cm` as columns and
  // `m` consumed the template's closing brace — nicematrix/nicematrix
  // exemplar 109 → 1002 errors + Fatal after b33.
  DefColumnType!("V{Dimension}", sub[(width)] {
    let mut before = vec![T_CS!("\\lx@tabular@p"), T_LETTER!("t"), T_BEGIN!()];
    before.extend(width.revert()?.unlist());
    before.push(T_END!());
    before.push(T_BEGIN!());
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens::new(before)),
        after: Some(Tokens!(T_END!())),
        align: Some(Align::Justify),
        vattach: Some("top".to_string()),
        ..Default::default()
      })
    });
  });
  // nicematrix.sty L21-22 defines its own `\myfileversion`/`\myfiledate`
  // (its manual typesets them on the title page). The binding shadows the
  // raw load, so read the REAL values from the installed .sty rather than
  // hardcoding a copy that drifts with the TL version.
  if let Some(path) = find_file("nicematrix.sty", None)
    && let Ok(src) = std::fs::read_to_string(&path)
  {
    for name in ["myfileversion", "myfiledate"] {
      let needle = s!("\\def\\{name}{{");
      if let Some(pos) = src.find(&needle)
        && let Some(end) = src[pos + needle.len()..].find('}')
      {
        let val = &src[pos + needle.len()..pos + needle.len() + end];
        def_macro(T_CS!(s!("\\{name}")), None, Tokens!(Explode!(val)), None)?;
      }
    }
  }

  // The tabular-like environments (`\NiceTabular`, `\NiceArray`, …) still degrade
  // to a placeholder-or-\tabular; the math Nice* MATRIX family below renders as a
  // real math array. NiceTabular[opts]{colspec}[opts] is nicematrix's tabular-like
  // environment; the real nicematrix.sty (L3806-3841) reduces it to \NiceArray
  // under a text-mode tabular flag. Degrade faithfully to \tabular with the SAME
  // colspec, dropping the nicematrix-only [opts] — recovers real tables (witnesses
  // 2605.08776, 2605.13835, 2605.18423) instead of discarding the body +
  // Error:undefined. Beyond-Perl: the ar5iv nicematrix.sty.ltxml stub still errors
  // here; mirror this upgrade there for strict Rust<->ar5iv parity.
  // Routed through the same `\CodeBefore … \Body` grabber as the matrix
  // family: executed INLINE, nicematrix's 2-arg `\rowcolor{color}{rows}`
  // (vs colortbl's 1-arg) desyncs the tabular's cell counting and a later
  // `\cmidrule` lands mid-cell ("\noalign cannot be used here" — the
  // exemplar manual's last error). Recorded rects are simply not painted for
  // text tabulars (color overlay is styling; the content is what matters).
  DefMacro!("\\NiceTabular OptionalBalanced {} OptionalBalanced", sub[(opts, pream, post)] {
    let opts_toks = nice_merge_opts(opts.map(|o| Tokens!(o.revert())), post.map(|o| Tokens!(o.revert())));
    nice_tabular_expansion(opts_toks, pream.revert(), vec![T_CS!("\\tabular")])
  }, locked => true);
  DefMacro!("\\endNiceTabular", "\\endtabular\\lx@nicematrix@materializenodes", locked => true);
  // NiceTabular* {width}[opts]{colspec}[opts] / NiceTabularX {width}[opts]
  // {colspec}[opts] (nicematrix.sty:3788/3801 `{ m O{} m !O{} }`): the SAME
  // reduction; `NiceTabularX` is a tabularx (its `X` columns need the
  // tabularx column engine — `\tabular{l||*{3}{X}}` had dropped every X cell,
  // witness nicematrix.tex `\begin{NiceTabularX}{\linewidth}{l||*{\LastDay}{X}}`)
  // and `NiceTabular*` a `\tabular*` — the fixed total width is print layout.
  RequirePackage!("tabularx");
  DefMacro!(T_CS!("\\NiceTabular*"), "{} OptionalBalanced {} OptionalBalanced", sub[(width, opts, pream, post)] {
    let mut starter = vec![T_CS!("\\tabular*"), T_BEGIN!()];
    starter.extend(width.revert()?.unlist());
    starter.push(T_END!());
    let opts_toks = nice_merge_opts(
      if opts.is_some() { Some(opts.revert()?) } else { None },
      if post.is_some() { Some(post.revert()?) } else { None },
    );
    nice_tabular_expansion(opts_toks, pream.revert()?.unlist(), starter)
  });
  DefMacro!(
    T_CS!("\\endNiceTabular*"),
    None,
    "\\endtabular*\\lx@nicematrix@materializenodes"
  );
  DefMacro!("\\NiceTabularX{} OptionalBalanced {} OptionalBalanced", sub[(width, opts, pream, post)] {
    let mut starter = vec![T_CS!("\\tabularx"), T_BEGIN!()];
    starter.extend(width.revert());
    starter.push(T_END!());
    let opts_toks = nice_merge_opts(
      opts.map(|o| Tokens!(o.revert())),
      post.map(|o| Tokens!(o.revert())),
    );
    nice_tabular_expansion(opts_toks, pream.revert(), starter)
  }, locked => true);
  DefMacro!("\\endNiceTabularX", "\\endtabularx\\lx@nicematrix@materializenodes", locked => true);

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
    let opts = opts.to_string();
    NICE_RECTS.with(|c| c.borrow_mut().clear());
    NICE_FIRST_ROW.with(|c| c.set(nice_opts_has(&opts, "first-row")));
    NICE_FIRST_COL.with(|c| c.set(nice_opts_has(&opts, "first-col")));
    let name = nice_opts_get(&opts, "name").or_else(|| {
      if nice_opts_has(&opts, "create-cell-nodes") {
        Some("NiceMatrix".to_string())
      } else {
        None
      }
    });
    NICE_NAME.with(|c| *c.borrow_mut() = name);
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
  DefMacro!("\\chessboardcolors OptionalBalanced {}{}", "", locked => true);
  DefMacro!("\\lx@nice@chessboardcolors OptionalBalanced {}{}", "", locked => true);
  // The rest of the `\CodeBefore`-scoped surface (nicematrix.sty:1790-1809,
  // `\cs_set_eq:NN` inside `\__nicematrix_exec_code_before:`): `\EmptyColumn{j}`
  // / `\EmptyRow{i}` (:6020/6029 — mark a column/row as empty for the
  // `corners` key; witness nicematrix.tex:2716 was `undefined`),
  // `\roundedrectanglecolor[model]{color}{i-j}{k-l}`, `\rowcolors[model]
  // {i}{c1}{c2}`, `\SubMatrix(…)` / `\ShowCellNames` / `\TikzEveryCell{…}`
  // (TikZ overlays). All print styling: gobble arguments, emit nothing.
  DefMacro!("\\SubMatrix {}{}{}{} OptionalBalanced", "", locked => true);
  DefMacro!("\\lx@nice@submatrix {}{}{}{} OptionalBalanced", "", locked => true);
  DefMacro!("\\EmptyColumn{}", "", locked => true);
  DefMacro!("\\lx@nice@emptycolumn{}", "", locked => true);
  DefMacro!("\\EmptyRow{}", "", locked => true);
  DefMacro!("\\lx@nice@emptyrow{}", "", locked => true);
  DefMacro!("\\roundedrectanglecolor OptionalBalanced {}{}{}", "", locked => true);
  DefMacro!("\\lx@nice@roundedrectanglecolor OptionalBalanced {}{}{}", "", locked => true);
  DefMacro!("\\rowcolors OptionalBalanced {}{}{}", "", locked => true);
  DefMacro!("\\lx@nice@rowcolors OptionalBalanced {}{}{}", "", locked => true);
  DefMacro!("\\ShowCellNames", "", locked => true);
  DefMacro!("\\lx@nice@showcellnames", "", locked => true);
  DefMacro!("\\TikzEveryCell OptionalBalanced {}", "", locked => true);
  DefMacro!("\\lx@nice@tikzeverycell OptionalBalanced {}", "", locked => true);
  DefMacro!("\\rowlistcolors OptionalBalanced {}{}", "", locked => true);
  DefMacro!("\\lx@nice@rowlistcolors OptionalBalanced {}{}", "", locked => true);

  RawTeX!(concat!(
    r"\def\lx@nice@fakenode#1{",
    r"\expandafter\gdef\csname pgf@sh@ns@#1\endcsname{coordinate}",
    r"\expandafter\gdef\csname pgf@sh@np@#1\endcsname{\def\centerpoint{\pgfpointorigin}}",
    r"\expandafter\gdef\csname pgf@sh@nt@#1\endcsname{{1}{0}{0}{1}{0pt}{0pt}}",
    r"\expandafter\global\expandafter\let\csname pgf@sh@ma@#1\endcsname\empty",
    r"\expandafter\gdef\csname pgf@sh@pi@#1\endcsname{1}}",
  ));

  DefPrimitive!("\\lx@nicematrix@materializenodes", {
    let name_opt = NICE_NAME.with(|c| c.borrow_mut().take());
    if let Some(name) = name_opt {
      let n_rows = lookup_int("LAST_ALIGNMENT_ROWS");
      let n_cols = lookup_int("LAST_ALIGNMENT_COLS");
      let mut names = Vec::new();
      names.push(name.clone());
      for i in 1..=n_rows {
        for j in 1..=n_cols {
          names.push(format!("{name}-{i}-{j}"));
          names.push(format!("{i}-{j}"));
        }
        names.push(format!("{name}-{i}-last"));
        names.push(format!("{name}-row-{i}"));
        names.push(format!("{i}-last"));
      }
      for j in 1..=n_cols {
        names.push(format!("{name}-last-{j}"));
        names.push(format!("{name}-col-{j}"));
        names.push(format!("last-{j}"));
      }
      names.push(format!("{name}-last-last"));
      names.push("last-last".to_string());
      let mut cmd = String::new();
      for n in names {
        cmd.push_str("\\lx@nice@fakenode{");
        cmd.push_str(&n);
        cmd.push('}');
      }
      let t = mouth::tokenize_internal(TeXString::assembled(cmd));
      digest(t)?;
    }
    Ok(Vec::new())
  });

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
  // starts. `\CodeBefore`/`\Body` are harmless no-ops with UNIQUE meanings
  // (nicematrix.sty:1642/1765 give them their own `\__nicematrix_…` bodies):
  // `\let` to `\relax`, `\@ifnextchar\CodeBefore` — a meaning comparison —
  // matched any `\relax`-meaning token at a matrix start and the
  // `Until:\Body` grab ran to EoF (the nicematrix manual's Fatal). Guard:
  // `perfect_kernel_batch54::nicematrix_relax_at_matrix_start_is_not_codebefore`.
  DefMacro!("\\CodeBefore OptionalBalanced", "", locked => true);
  DefMacro!("\\Body", "", locked => true);
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
    r"\let\EmptyColumn\lx@nice@emptycolumn",
    r"\let\EmptyRow\lx@nice@emptyrow",
    r"\let\roundedrectanglecolor\lx@nice@roundedrectanglecolor",
    r"\let\rowcolors\lx@nice@rowcolors",
    r"\let\ShowCellNames\lx@nice@showcellnames",
    r"\let\TikzEveryCell\lx@nice@tikzeverycell",
    r"\let\SubMatrix\lx@nice@submatrix",
    r"\lx@nice@codebefore{#2}",
    r"\endgroup",
    r"\lx@ams@matrix{#1}}"
  ));
  // The `\CodeBefore` block is EXECUTED to record its color commands, but a
  // drawing `\begin{tikzpicture}…\end{tikzpicture}`/`{scope}`/`\tikz … ;` inside
  // it (the `create-cell-nodes` overlays, nicematrix-french:6154…) references cell
  // nodes LaTeXML never materializes ("No shape named 'i-j'" ×280) and is
  // pure overlay — drop those environments and inline \tikz commands, keep the rest. Guard:
  // `perfect_kernel_batch54::nicematrix_codebefore_drops_drawing_environments`.
  DefPrimitive!("\\lx@nice@codebefore{}", sub[(block)] {
    let toks = block.unlist();
    let mut kept: Vec<Token> = Vec::with_capacity(toks.len());
    let begin_cs = T_CS!("\\begin");
    let end_cs = T_CS!("\\end");
    let tikz_cs = T_CS!("\\tikz");
    let mut i = 0;
    let mut skip_depth = 0usize;
    let env_name = |toks: &[Token], i: usize| -> String {
      // `\begin{name}` → the tokens between the braces after position i
      let mut j = i + 1;
      if j < toks.len() && toks[j].get_catcode() == Catcode::BEGIN {
        j += 1;
        let mut name = String::new();
        while j < toks.len() && toks[j].get_catcode() != Catcode::END {
          name.push_str(&toks[j].to_string());
          j += 1;
        }
        name
      } else {
        String::new()
      }
    };
    while i < toks.len() {
      let t = &toks[i];
      if *t == begin_cs {
        let name = env_name(&toks, i);
        if skip_depth > 0 || name == "tikzpicture" || name == "scope" {
          skip_depth += 1;
        }
      } else if *t == end_cs && skip_depth > 0 {
        skip_depth -= 1;
        // consume the `{name}` of this \end
        let mut j = i + 1;
        if j < toks.len() && toks[j].get_catcode() == Catcode::BEGIN {
          while j < toks.len() && toks[j].get_catcode() != Catcode::END {
            j += 1;
          }
        }
        i = j + 1;
        continue;
      }
      if skip_depth == 0 && *t == tikz_cs {
        let mut j = i + 1;
        while j < toks.len() && toks[j].get_catcode() == Catcode::SPACE {
          j += 1;
        }
        if j < toks.len() && toks[j].with_str(|s| s == "[") {
          let mut bdepth = 1usize;
          j += 1;
          while j < toks.len() && bdepth > 0 {
            if toks[j].with_str(|s| s == "[") {
              bdepth += 1;
            } else if toks[j].with_str(|s| s == "]") {
              bdepth -= 1;
            }
            j += 1;
          }
        }
        while j < toks.len() && toks[j].get_catcode() == Catcode::SPACE {
          j += 1;
        }
        if j < toks.len() && toks[j].get_catcode() == Catcode::BEGIN {
          let mut bdepth = 1usize;
          j += 1;
          while j < toks.len() && bdepth > 0 {
            if toks[j].get_catcode() == Catcode::BEGIN {
              bdepth += 1;
            } else if toks[j].get_catcode() == Catcode::END {
              bdepth -= 1;
            }
            j += 1;
          }
          i = j;
          continue;
        } else {
          let mut bdepth = 0usize;
          while j < toks.len() {
            if toks[j].get_catcode() == Catcode::BEGIN {
              bdepth += 1;
            } else if toks[j].get_catcode() == Catcode::END && bdepth > 0 {
              bdepth -= 1;
            } else if bdepth == 0 && toks[j].with_str(|s| s == ";") {
              j += 1;
              break;
            }
            j += 1;
          }
          i = j;
          continue;
        }
      }
      if skip_depth == 0 {
        kept.push(*t);
      }
      i += 1;
    }
    digest(Tokens::new(kept))
  });

  // The MATRIX family: `\<x>NiceMatrix[opts]` → set opts, then reduce to the
  // amsmath matrix flavour for delimiter <x>; `\end<x>NiceMatrix` closes the array
  // and paints the recorded colors. Mirrors amsmath_sty.rs:393-427.
  DefMacro!("\\NiceMatrix OptionalBalanced",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=NiceMatrix,datameaning=matrix}",
    locked => true);
  DefMacro!("\\endNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@materializenodes\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\pNiceMatrix OptionalBalanced",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=pNiceMatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right)}",
    locked => true);
  DefMacro!("\\endpNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@materializenodes\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\bNiceMatrix OptionalBalanced",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=bNiceMatrix,datameaning=matrix,left=\\lx@left[,right=\\lx@right]}",
    locked => true);
  DefMacro!("\\endbNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@materializenodes\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\BNiceMatrix OptionalBalanced",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=BNiceMatrix,datameaning=matrix,left=\\lx@left\\{,right=\\lx@right\\}}",
    locked => true);
  DefMacro!("\\endBNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@materializenodes\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\vNiceMatrix OptionalBalanced",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=vNiceMatrix,delimitermeaning=determinant,datameaning=matrix,left=\\lx@left|,right=\\lx@right|}",
    locked => true);
  DefMacro!("\\endvNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@materializenodes\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\VNiceMatrix OptionalBalanced",
    "\\lx@nice@setopts{#1}\\lx@nice@matrix@begin{name=VNiceMatrix,delimitermeaning=norm,datameaning=matrix,left=\\lx@left\\|,right=\\lx@right\\|}",
    locked => true);
  DefMacro!("\\endVNiceMatrix", "\\lx@end@ams@matrix\\lx@nicematrix@materializenodes\\lx@nicematrix@applycolors", locked => true);

  // The ARRAY family — REAL reductions (policy 2026-08-31: content must
  // survive; the former placeholders DISCARDED every body). First principles
  // from nicematrix.sty: `\begin{<x>NiceArray}{colspec}[opts]` is an
  // array-with-delimiters — nicematrix builds every one of them on
  // `{NiceArrayWithDelims}{l}{r}{colspec}` (nicematrix.sty v6:
  // `\NewDocumentEnvironment{pNiceArray}… {NiceArrayWithDelims}{(}{)}`),
  // which is itself a math `{array}` wrapped in `\left l … \right r`.
  // Reduce exactly that way, through the SAME `\CodeBefore … \Body`
  // pre-layer grabber and post-paint hook as the matrix family, so cell
  // colors work identically.
  RawTeX!(concat!(
    r"\def\lx@nice@array@begin#1{",
    r"\@ifnextchar\CodeBefore{\lx@nice@grabcode@arr{#1}}{#1}}",
    r"\long\def\lx@nice@grabcode@arr#1#2\Body{",
    r"\begingroup",
    r"\let\rectanglecolor\lx@nice@rectanglecolor",
    r"\let\cellcolor\lx@nice@cellcolor",
    r"\let\rowcolor\lx@nice@rowcolor",
    r"\let\columncolor\lx@nice@columncolor",
    r"\let\arraycolor\lx@nice@arraycolor",
    r"\let\chessboardcolors\lx@nice@chessboardcolors",
    r"\let\rowlistcolors\lx@nice@rowlistcolors",
    r"\let\EmptyColumn\lx@nice@emptycolumn",
    r"\let\EmptyRow\lx@nice@emptyrow",
    r"\let\roundedrectanglecolor\lx@nice@roundedrectanglecolor",
    r"\let\rowcolors\lx@nice@rowcolors",
    r"\let\ShowCellNames\lx@nice@showcellnames",
    r"\let\TikzEveryCell\lx@nice@tikzeverycell",
    r"\let\SubMatrix\lx@nice@submatrix",
    r"\lx@nice@codebefore{#2}",
    r"\endgroup",
    r"#1}"
  ));
  // nicematrix.sty:1953 `NiceArrayWithDelims { m m O{} m !O{} t\CodeBefore }`
  // and :3665-3672 `NiceArray` → `[opts1]{cols}[opts2]`: the preamble is
  // flanked by TWO option lists (`\begin{NiceArray}[t]{lcccccc}[no-cell-nodes]`,
  // nicematrix.tex:409). A signature without the leading one bound `[t]` as
  // the preamble ("Unrecognized tabular template [", 57 extra `&` + the
  // `Until:\Body` EOF fatal that aborted the rest of the manual).
  DefMacro!("\\NiceArray[]{}[]", "\\NiceArrayWithDelims.{.}[#1]{#2}[#3]", locked => true);
  DefMacro!("\\endNiceArray", "\\endNiceArrayWithDelims", locked => true);
  // Closure form: `[first-col]`/`[last-col]` add LABEL cells to every source
  // row, so the colspec must grow a `c` on that side or each row overflows
  // the template ("Extra alignment tab"). first-row needs no preamble change
  // (row count is unconstrained). nicematrix.sty does the analogue in its
  // preamble parser.
  DefMacro!("\\NiceArrayWithDelims{}{}[]{}[]", sub[(l, r, opts1, pream, opts2)] {
    let opts_toks = nice_merge_opts(opts1.map(|o| Tokens!(o.revert())), opts2.map(|o| Tokens!(o.revert())));
    let opts_str = opts_toks.to_string();
    let mut out: Vec<Token> = Vec::new();
    out.push(T_CS!("\\def"));
    out.push(T_CS!("\\lx@nice@awd@right"));
    out.push(T_BEGIN!());
    out.extend(r.revert());
    out.push(T_END!());
    out.push(T_CS!("\\lx@nice@setopts"));
    out.push(T_BEGIN!());
    out.extend(opts_toks.unlist());
    out.push(T_END!());
    out.push(T_CS!("\\lx@nice@array@begin"));
    out.push(T_BEGIN!());
    out.push(T_CS!("\\left"));
    out.extend(l.revert());
    out.push(T_CS!("\\array"));
    out.push(T_BEGIN!());
    if nice_opts_has(&opts_str, "first-col") {
      out.push(T_LETTER!("c"));
    }
    out.extend(nice_strip_rule_opts(pream.revert()));
    if nice_opts_has(&opts_str, "last-col") {
      out.push(T_LETTER!("c"));
    }
    out.push(T_END!());
    out.push(T_END!());
    Tokens::new(out)
  }, locked => true);
  DefMacro!("\\endNiceArrayWithDelims",
    "\\endarray\\expandafter\\right\\lx@nice@awd@right\\lx@nicematrix@materializenodes\\lx@nicematrix@applycolors", locked => true);
  DefMacro!("\\pNiceArray[]{}[]", "\\NiceArrayWithDelims({)}[#1]{#2}[#3]", locked => true);
  DefMacro!("\\endpNiceArray", "\\endNiceArrayWithDelims", locked => true);
  DefMacro!("\\bNiceArray[]{}[]", "\\NiceArrayWithDelims[{]}[#1]{#2}[#3]", locked => true);
  DefMacro!("\\endbNiceArray", "\\endNiceArrayWithDelims", locked => true);
  DefMacro!("\\BNiceArray[]{}[]", "\\NiceArrayWithDelims\\{{\\}}[#1]{#2}[#3]", locked => true);
  DefMacro!("\\endBNiceArray", "\\endNiceArrayWithDelims", locked => true);
  DefMacro!("\\vNiceArray[]{}[]", "\\NiceArrayWithDelims|{|}[#1]{#2}[#3]", locked => true);
  DefMacro!("\\endvNiceArray", "\\endNiceArrayWithDelims", locked => true);
  DefMacro!("\\VNiceArray[]{}[]", "\\NiceArrayWithDelims\\|{\\|}[#1]{#2}[#3]", locked => true);
  DefMacro!("\\endVNiceArray", "\\endNiceArrayWithDelims", locked => true);

  // AutoNiceMatrix family: automated matrix population with delimiter variants.
  DefMacro!("\\AutoNiceMatrixWithDelims{}{} OptionalBalanced {} OptionalBalanced {} OptionalBalanced", sub[(l, r, opt1, dims, opt2, pat, opt3)] {
    let opts12 = nice_merge_opts(opt1.map(|o| Tokens!(o.revert())), opt2.map(|o| Tokens!(o.revert())));
    let opts_toks = nice_merge_opts(Some(opts12), opt3.map(|o| Tokens!(o.revert())));
    let dims_str = dims.to_string();
    let (n_rows, n_cols) = match dims_str.split_once('-') {
      Some((r_str, c_str)) => {
        let r = r_str.trim().parse::<usize>().unwrap_or(1).clamp(1, 100);
        let c = c_str.trim().parse::<usize>().unwrap_or(1).clamp(1, 100);
        (r, c)
      },
      None => (1, 1),
    };
    let mut out: Vec<Token> = Vec::new();
    out.push(T_CS!("\\begin"));
    out.push(T_BEGIN!());
    out.extend(Tokenize!(TeXString::from("NiceArrayWithDelims")).unlist());
    out.push(T_END!());

    out.push(T_BEGIN!());
    out.extend(l.revert());
    out.push(T_END!());

    out.push(T_BEGIN!());
    out.extend(r.revert());
    out.push(T_END!());

    out.push(T_BEGIN!());
    out.extend(Tokenize!(TeXString::assembled(format!("*{{{n_cols}}}{{c}}"))).unlist());
    out.push(T_END!());

    if !opts_toks.is_empty() {
      out.push(T_OTHER!("["));
      out.extend(opts_toks.unlist());
      out.push(T_OTHER!("]"));
    }

    let pat_tokens = pat.revert();
    for i in 1..=n_rows {
      for j in 1..=n_cols {
        out.extend(Tokenize!(TeXString::assembled(format!("\\setcounter{{iRow}}{{{i}}}\\setcounter{{jCol}}{{{j}}}"))).unlist());
        out.extend(pat_tokens.clone());
        if j < n_cols {
          out.push(T_ALIGN!());
        }
      }
      out.push(T_CS!("\\\\"));
    }

    out.push(T_CS!("\\end"));
    out.push(T_BEGIN!());
    out.extend(Tokenize!(TeXString::from("NiceArrayWithDelims")).unlist());
    out.push(T_END!());

    Tokens::new(out)
  }, locked => true);

  DefMacro!("\\AutoNiceMatrix OptionalBalanced {} OptionalBalanced {} OptionalBalanced",
    "\\AutoNiceMatrixWithDelims{.}{.}[#1]{#2}[#3]{#4}[#5]", locked => true);
  DefMacro!("\\pAutoNiceMatrix OptionalBalanced {} OptionalBalanced {} OptionalBalanced",
    "\\AutoNiceMatrixWithDelims{(}{)}[#1]{#2}[#3]{#4}[#5]", locked => true);
  DefMacro!("\\bAutoNiceMatrix OptionalBalanced {} OptionalBalanced {} OptionalBalanced",
    "\\AutoNiceMatrixWithDelims{[}{]}[#1]{#2}[#3]{#4}[#5]", locked => true);
  DefMacro!("\\vAutoNiceMatrix OptionalBalanced {} OptionalBalanced {} OptionalBalanced",
    "\\AutoNiceMatrixWithDelims{|}{|}[#1]{#2}[#3]{#4}[#5]", locked => true);
  DefMacro!("\\VAutoNiceMatrix OptionalBalanced {} OptionalBalanced {} OptionalBalanced",
    "\\AutoNiceMatrixWithDelims{\\|}{\\|}[#1]{#2}[#3]{#4}[#5]", locked => true);
  DefMacro!("\\BAutoNiceMatrix OptionalBalanced {} OptionalBalanced {} OptionalBalanced",
    "\\AutoNiceMatrixWithDelims{\\{}{\\}}[#1]{#2}[#3]{#4}[#5]", locked => true);

  // In-tabular decoration commands the manuals use pervasively.
  // \Block[opts]{i-j}{content}: nicematrix paints `content` OVER an i×j cell
  // rectangle whose other source cells are empty. The logical position of the
  // content is the anchor cell, so emitting it in place preserves content and
  // reading order; the visual spanning overlay is print styling (a
  // \multicolumn/rowspan rewrite would break the row's cell count, since the
  // covered cells are still present in the source). The starred form and
  // math-mode `$`-wrapped bodies pass through unchanged.
  // Under `ampersand-in-blocks` (nicematrix.sty:7310-7311, `\__nicematrix_Block_vii`
  // :7592) a body holding `&`/`\\` is typeset as a SUB-GRID — a `tabular`
  // in text, `$\begin{array}$` in math (:7398/:7406), split on `&` (:7906).
  // Emitting the body bare re-exposed the inner `&` to the outer `\halign`
  // ("Extra alignment tab '&'", nicematrix.tex:1152, nicematrix-french:1204).
  // The sub-grid is built from the body's widest row; without a depth-0 `&`
  // the body passes through as before. Guard:
  // `perfect_kernel_batch54::nicematrix_block_ampersand_body_is_a_subgrid`.
  DefMacro!("\\Block OptionalMatch:* []{}{}", sub[(_star, _pos, _ij, body)] {
    let toks = body.unlist();
    let mut depth = 0i32;
    let mut cols = 1usize;
    let mut max_cols = 1usize;
    let newline = T_CS!("\\\\");
    for t in &toks {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        Catcode::ALIGN if depth == 0 => cols += 1,
        _ if depth == 0 && *t == newline => {
          max_cols = max_cols.max(cols);
          cols = 1;
        },
        _ => {},
      }
    }
    max_cols = max_cols.max(cols);
    if max_cols == 1 {
      return Ok(Tokens::new(toks));
    }
    let (env_begin, env_end) = if lookup_bool_sym(pin!("IN_MATH")) {
      (s!("\\begin{{array}}{{*{{{max_cols}}}{{c}}}}"), "\\end{array}".to_string())
    } else {
      (s!("\\begin{{tabular}}{{*{{{max_cols}}}{{c}}}}"), "\\end{tabular}".to_string())
    };
    let mut out: Vec<Token> = Tokenize!(TeXString::assembled(env_begin)).unlist();
    out.extend(toks);
    out.extend(Tokenize!(TeXString::assembled(env_end)).unlist());
    Ok(Tokens::new(out))
  }, locked => true);
  // \Hline[opts]: nicematrix's own \hline that survives its internal
  // machinery; opts (color=, thickness=) are rule styling. Reduce to \hline.
  DefMacro!("\\Hline []", "\\hline", locked => true);
  // \rotate: rotates the CELL CONTENT 90° in print — pure presentation, the
  // content itself follows. Justified noop.
  def_macro_noop("\\rotate")?;

  // Continuous-dots commands (nicematrix's \\Cdots family draws dotted lines
  // ACROSS cells). Semantically these are the amsmath ellipses in the anchor
  // cell; the cross-cell line extension is drawing-layer presentation.
  // Optional [line-style] args are styling.
  DefMacro!("\\Cdots []", "\\cdots", locked => true);
  DefMacro!("\\Ldots []", "\\ldots", locked => true);
  DefMacro!("\\Vdots []", "\\vdots", locked => true);
  DefMacro!("\\Ddots []", "\\ddots", locked => true);
  DefMacro!("\\Iddots []", "\\ddots", locked => true);
  DefMacro!("\\Hdotsfor{}", "\\hdotsfor{#1}", locked => true);
  DefMacro!("\\Vdotsfor OptionalBalanced {}", "\\vdots", locked => true);
  DefMacro!("\\Hspace OptionalMatch:* {}", "\\hspace#1{#2}", locked => true);
  DefMacro!("\\Hbrace OptionalBalanced {}{}", "#3", locked => true);
  DefMacro!("\\Vbrace OptionalBalanced {}{}", "#3", locked => true);
  DefMacro!("\\tabularnote OptionalBalanced {}", "\\footnote#1{#2}", locked => true);
  NewCounter!("iRow");
  NewCounter!("jCol");
  NewCounter!("tabularnote");

  // {NiceMatrixBlock}[opts]: a grouping wrapper that equalizes column widths
  // across the matrices INSIDE it — layout-only; the content flows through.
  DefMacro!("\\NiceMatrixBlock []", "", locked => true);
  DefMacro!("\\endNiceMatrixBlock", "", locked => true);
  // \CodeAfter: everything from here to the environment's \end is an
  // overlay-drawing layer (\line, \SubMatrix, \tikz over the built grid) —
  // drawing-only, no document content; grab and drop, keeping the \end so
  // the environment closes normally.
  // (Plain `Until` — unexpanded scan. `XUntil` EXPANDS while scanning, which
  // would EXECUTE the overlay's \begin{tikzpicture}/\SubMatrix mid-grab.)
  // The grab is ENVIRONMENT-BALANCED: a `\begin{tikzpicture}…\end{tikzpicture}`
  // (or `{scope}`) inside `\CodeAfter` has its own `\end`, and a plain
  // `Until:\end` stopped there, re-emitting `\end{tikzpicture}` into a context
  // with no open picture (23 "`\endgroup` Attempt to close non-boxing group"
  // + leaked `\node[fit=(A)]` pgf errors, nicematrix-french). Guard:
  // `perfect_kernel_batch54::nicematrix_codeafter_grab_is_environment_balanced`.
  DefMacro!("\\CodeAfter", sub[_args] {
    let begin_cs = T_CS!("\\begin");
    let end_cs = T_CS!("\\end");
    let mut depth = 0usize;
    while let Some(t) = read_token()? {
      if t == begin_cs {
        depth += 1;
      } else if t == end_cs {
        if depth == 0 {
          unread_one(t);
          break;
        }
        depth -= 1;
      }
    }
    Ok(Tokens!())
  });
  // Decoration/rule commands usable in cells and preambles: dotted/double
  // rules are rule styling (reduce to \hline / nothing); \RowStyle sets
  // per-row styling keys.
  DefMacro!("\\hdottedline", "\\hline", locked => true);
  DefMacro!("\\cdottedline{}", "\\hline", locked => true);
  def_macro_noop("\\DoubleRule")?;
  def_macro_noop("\\RowStyle[]{}")?;
  def_macro_noop("\\rowlistcolors[]{}{}")?;
  // `\NotEmpty` (nicematrix.sty:1644 `\cs_set_eq:NN \NotEmpty
  // \__nicematrix_NotEmpty:`, :3745) only flags the cell non-empty so
  // `hvlines` draws its rules — no content. Witness cahierprof.sty:619
  // `\replicate{\NbColonnes}{&\NotEmpty}`.
  def_macro_noop("\\NotEmpty")?;
  // Public CodeBefore hook (nicematrix.sty:394 `\tl_new:N
  // \g_nicematrix_code_before_tl`) that user packages append tikz drawing
  // to (cahierprof.sty:519/531 `\tl_gput_right:Nx`). The binding drops the
  // CodeBefore drawing layer, so the accumulated content is never painted,
  // but the variable must exist for the append to succeed. Guard:
  // `perfect_kernel_batch54::nicematrix_notempty_and_code_before_hook`.
  RawTeX!(r"\ExplSyntaxOn \tl_new:N \g_nicematrix_code_before_tl \ExplSyntaxOff");
  // \diagbox{lower}{upper} (diagbox-style split cell): both texts are
  // content — keep them, separator as a slash.
  DefMacro!("\\diagbox{}{}", "#1/#2", locked => true);

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
