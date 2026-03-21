use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: makecell.sty.ltxml
  // Load raw TeX first
  InputDefinitions!("makecell", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Mark thead et.al as headers (row & column)
  DefPrimitive!("\\lx@makecell@head", sub[_args] {
    if let Some(alignment) = lookup_alignment() {
      if let Some(data) = alignment.alignment_cell() {
        if let Some(col) = data.borrow_mut().current_column() {
          col.thead_in_column = true;
          col.thead_in_row = true;
        }
      }
    }
    Ok(())
  });

  // Redefine \theadfont at BeginDocument to include heading marker
  RawTeX!(r"\AtBeginDocument{\let\lx@orig@theadfont\theadfont\def\theadfont{\lx@orig@theadfont\lx@makecell@head}}");

  // Since we use \thead, disable guessing
  AssignValue!("GUESS_TABULAR_HEADERS" => false, Scope::Global);

  // \rothead: simplified override — delegates to \thead without rotation.
  // TODO: implement rotation via {turn}{90} wrapping. Raw TeX causes stack overflow.
  DefMacro!("\\rothead[]{}",
    "\\thead[#1]{#2}");
  DefMacro!("\\rotcell[]{}",
    "\\makecell[#1]{#2}");

  // \lx@diag@head: wraps content in \theadfont + \shortstack for alignment
  DefMacro!("\\lx@diag@head{}{}",
    "{\\theadfont\\shortstack[#1]{#2}}");

  // \lx@diagheads: constructor producing <picture> with diagonal line and text boxes
  // Perl: DefConstructor('\lx@diagheads {}{} {}{}{}', ...)
  DefConstructor!("\\lx@diagheads{}{} {}{}{}",
    "<ltx:picture width='#pxwidth' height='#pxheight' xml:id='#id'><ltx:g transform='translate(#Atransform)' innerwidth='#Aw' innerheight='#Ah'><ltx:inline-block>#A</ltx:inline-block></ltx:g><ltx:g transform='translate(#Btransform)' innerwidth='#Bw' innerheight='#Bh'><ltx:inline-block>#B</ltx:inline-block></ltx:g></ltx:picture>",
    reversion => r"\diaghead(#1,#2){#3}{#4}{#5}",
    after_construct => sub[document, _whatsit] {
      let mut node = document.get_node().clone();
      document.add_class(&mut node, "ltx_nopad")?;
    },
    after_digest => sub[whatsit] {
      let dh: f64 = whatsit.get_arg(0).map(|a| a.to_attribute().parse().unwrap_or(1.0)).unwrap_or(1.0);
      let dv: f64 = whatsit.get_arg(1).map(|a| a.to_attribute().parse().unwrap_or(1.0)).unwrap_or(1.0);
      let flip = (dh < 0.0) != (dv < 0.0);
      // pxValue conversion: sp / 65536 * DPI / 72.27, DPI=100
      let px = |d: Dimension| -> f64 { d.value_of() as f64 / 65536.0 * 100.0 / 72.27 };
      let round2 = |v: f64| -> String { s!("{:.2}", v) };
      // Get sizes of A and B (args 3 and 4)
      let (mut aw, mut ah) = (0.0_f64, 0.0_f64);
      let (mut bw, mut bh) = (0.0_f64, 0.0_f64);
      if let Some(a) = whatsit.get_arg(3) {
        if let Ok((w,h,d,_,_,_)) = a.clone().get_size(None) {
          aw = px(w); let a_h = px(h); let a_d = px(d); ah = a_h + a_d;
        }
      }
      if let Some(b) = whatsit.get_arg(4) {
        if let Ok((w,h,d,_,_,_)) = b.clone().get_size(None) {
          bw = px(w); let b_h = px(h); let b_d = px(d); bh = b_h + b_d;
        }
      }
      // Get width from space arg (#3 = arg index 2)
      // Perl: $space->getWidth->pxValue
      let w = if let Some(sp) = whatsit.get_arg(2) {
        if let Ok(Some(wd)) = sp.clone().get_width(None) {
          let dim: Dimension = wd.into();
          px(dim)
        } else {
          // Fallback: try get_size
          let (wd,_,_,_,_,_) = sp.clone().get_size(None)?;
          px(wd)
        }
      } else { 0.0 };
      let h = w * (dv / dh).abs();
      let line = if flip { s!("0,{h} {w},0") } else { s!("0,0, {w},{h}") };
      let ax = if flip { 0.0 } else { w - aw };
      let ay = 0.0_f64;
      let bx = if flip { w - bw } else { 0.0 };
      let by = h - bh;
      // Convert px to pt for width/height attributes
      let pxppt = 100.0 / 72.27;
      let to_dim_attr = |v: f64| -> String {
        use latexml_core::common::dimension::attribute_format;
        attribute_format((v / pxppt * 65536.0) as i64, None)
      };
      whatsit.set_property("pxwidth", Stored::from(round2(w)));
      whatsit.set_property("pxheight", Stored::from(round2(h)));
      whatsit.set_property("width", Stored::from(to_dim_attr(w)));
      whatsit.set_property("height", Stored::from(to_dim_attr(h)));
      whatsit.set_property("A", whatsit.get_arg(3).cloned().map(Stored::from).unwrap_or(Stored::None));
      whatsit.set_property("Atransform", Stored::from(s!("{},{ay}", round2(ax))));
      whatsit.set_property("Aw", Stored::from(round2(aw)));
      whatsit.set_property("Ah", Stored::from(round2(ah)));
      whatsit.set_property("B", whatsit.get_arg(4).cloned().map(Stored::from).unwrap_or(Stored::None));
      whatsit.set_property("Btransform", Stored::from(s!("{},{}", round2(bx), round2(by))));
      whatsit.set_property("Bw", Stored::from(round2(bw)));
      whatsit.set_property("Bh", Stored::from(round2(bh)));
      whatsit.set_property("line", Stored::from(line));
      whatsit.set_property("color", Stored::from("#000000".to_string()));
    });

  DefMacro!("\\diaghead OptionalPair {}{}{}", sub[args] {
    let mut it = args.into_iter();
    let diag_arg = it.next().unwrap(); // OptionalPair
    let space: Tokens = it.next().unwrap().into(); // {width}
    let a_content: Tokens = it.next().unwrap().into(); // {item A}
    let b_content: Tokens = it.next().unwrap().into(); // {item B}

    // Parse slope from OptionalPair
    let (dh, dv) = if let ArgWrap::Pair(pair) = diag_arg {
      (pair.get_x().0, pair.get_y().0)
    } else {
      (5.0, -2.0)
    };
    let flip = (dh < 0.0) != (dv < 0.0);
    let (ap, bp) = if flip { ("l", "r") } else { ("r", "l") };

    // Build: \lx@diagheads{dh}{dv}{space}{\lx@diag@head{ap}{A}}{\lx@diag@head{bp}{B}}
    let mut result = Vec::new();
    result.push(T_CS!("\\lx@diagheads"));
    // {dh}
    result.push(T_BEGIN!());
    result.extend(ExplodeText!(s!("{dh}")));
    result.push(T_END!());
    // {dv}
    result.push(T_BEGIN!());
    result.extend(ExplodeText!(s!("{dv}")));
    result.push(T_END!());
    // {space}
    result.push(T_BEGIN!());
    result.extend(space.unlist());
    result.push(T_END!());
    // {\lx@diag@head{ap}{A}}
    result.push(T_BEGIN!());
    result.push(T_CS!("\\lx@diag@head"));
    result.push(T_BEGIN!());
    result.extend(ExplodeText!(ap));
    result.push(T_END!());
    result.push(T_BEGIN!());
    result.extend(a_content.unlist());
    result.push(T_END!());
    result.push(T_END!());
    // {\lx@diag@head{bp}{B}}
    result.push(T_BEGIN!());
    result.push(T_CS!("\\lx@diag@head"));
    result.push(T_BEGIN!());
    result.extend(ExplodeText!(bp));
    result.push(T_END!());
    result.push(T_BEGIN!());
    result.extend(b_content.unlist());
    result.push(T_END!());
    result.push(T_END!());

    Tokens::new(result)
  });
});
