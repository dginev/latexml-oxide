use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: makecell.sty.ltxml
  // Load raw TeX first
  InputDefinitions!("makecell", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Mark thead et.al as headers (row & column).
  // Perl is DefMacroI with an imperative sub body (no token return);
  // Rust DefPrimitive runs at stomach time. WISDOM #44: the two kinds
  // differ under expansion — safe here because `\lx@makecell@head` is
  // injected by `\theadfont` inside alignment cells and never observed
  // by `\edef` / `\ifx`.
  // WISDOM #44 verified 2026-04-23: zero `\edef`/`\ifx`/`\expandafter`
  // uses of `\lx@makecell@head` across LaTeXML/lib + ar5iv-bindings.
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
  at_begin_document(TokenizeInternal!(
    r"\let\lx@orig@theadfont\theadfont\def\theadfont{\lx@orig@theadfont\lx@makecell@head}"
  ))?;

  // Since we use \thead, disable guessing
  AssignValue!("GUESS_TABULAR_HEADERS" => false, Scope::Global);

  // \lx@rothead@box: custom rotation box that hardcodes 90° rotation.
  // Avoids Float parameter parsing issues with \turnbox/\rotatebox.
  DefConstructor!("\\lx@rothead@box{}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#1</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true,
    after_digest => sub[whatsit] {
      if let Some(body) = whatsit.get_arg(1) {
        if let Ok(props) = crate::package::graphics_sty::rotated_properties(body.clone(), 90.0, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
      }
    });

  // \rothead and \rotcell: use raw TeX definitions from makecell.sty
  // (loaded via InputDefinitions above). The raw TeX definitions use \turn{\cellrotangle}
  // for rotation and p{\rotheadsize} column format for paragraph width.

  // Generate xml:id for <picture> elements (Perl: Tag('ltx:picture', afterOpen => \&GenerateID))
  Tag!("ltx:picture", after_open => sub[document, node] {
    document.generate_id(node, "pic")?;
  });

  // \lx@diag@head: wraps content in \theadfont + \shortstack for alignment
  DefMacro!("\\lx@diag@head{}{}",
    "{\\theadfont\\shortstack[#1]{#2}}");

  // \lx@diagheads: constructor producing <picture> with diagonal line and text boxes
  // Perl: DefConstructor('\lx@diagheads {}{} {}{}{}', ...)
  DefConstructor!("\\lx@diagheads{}{} {}{}{}",
    "<ltx:picture width='#pxwidth' height='#pxheight' xml:id='#id'><ltx:g transform='translate(#Atransform)' innerwidth='#Aw' innerheight='#Ah'><ltx:inline-block>#A</ltx:inline-block></ltx:g><ltx:g transform='translate(#Btransform)' innerwidth='#Bw' innerheight='#Bh'><ltx:inline-block>#B</ltx:inline-block></ltx:g></ltx:picture>",
    reversion => r"\diaghead(#1,#2){#3}{#4}{#5}",
    after_construct => sub[document, whatsit] {
      let mut node = document.get_node().clone();
      document.add_class(&mut node, "ltx_nopad")?;
      // Insert <line> element as first child of the <picture> we just created
      if let Some(line_pts) = whatsit.get_property("line") {
        let children = node.get_child_nodes();
        if let Some(mut picture) = children.into_iter().rev().find(|c| c.get_name() == "picture") {
          let line_str = line_pts.to_attribute();
          let color_str = whatsit.get_property("color").map(|c| c.to_attribute()).unwrap_or_else(|| "#000000".to_string());
          // Create line node using raw libxml API
          let ns = picture.get_namespace();
          let mut line_node = libxml::tree::Node::new("line", ns, document.get_document()).unwrap();
          let _ = line_node.set_attribute("points", &line_str);
          let _ = line_node.set_attribute("stroke", &color_str);
          let _ = line_node.set_attribute("stroke-width", "0.4");
          // Insert as first child of picture
          let pic_children = picture.get_child_nodes();
          if let Some(mut first_g) = pic_children.into_iter().find(|c| c.get_name() == "g") {
            first_g.add_prev_sibling(&mut line_node)?;
          } else {
            picture.add_child(&mut line_node)?;
          }
          // Set tex= attribute on <picture> from constructor reversion
          let tex_str = whatsit.revert()
            .map(|toks| toks.untex())
            .unwrap_or_default();
          if !tex_str.is_empty() {
            document.set_attribute(&mut picture, "tex", &tex_str)?;
          }
        }
      }
    },
    after_digest => sub[whatsit] {
      // get_arg is 1-based (matching Perl's getArg)
      let dh: f64 = whatsit.get_arg(1).map(|a| a.to_attribute().parse().unwrap_or(1.0)).unwrap_or(1.0);
      let dv: f64 = whatsit.get_arg(2).map(|a| a.to_attribute().parse().unwrap_or(1.0)).unwrap_or(1.0);
      let flip = (dh < 0.0) != (dv < 0.0);
      // Perl: pxValue rounds to 2dp (roundto default precision=2)
      let roundto2 = |v: f64| -> f64 {
        let scale = 100.0_f64;
        let n = v * scale * (1.0 + 100.0 * f64::EPSILON);
        let adj = if n < -f64::EPSILON { n - 0.5 } else if n > f64::EPSILON { n + 0.5 } else { 0.0 };
        adj.trunc() / scale
      };
      let px = |d: Dimension| -> f64 { roundto2(d.value_of() as f64 / 65536.0 * 100.0 / 72.27) };
      // Perl: raw pxValue, NO rounding on dimensions or coordinates
      // Format like Perl's default float-to-string: minimal digits, trim trailing zeros
      let fmtpx = |v: f64| -> String {
        if v == 0.0 { return "0".to_string(); }
        let s = s!("{:.6}", v);
        let s = s.trim_end_matches('0');
        s.trim_end_matches('.').to_string()
      };
      // Get sizes of A and B (args #4 and #5)
      let (mut aw, mut ah) = (0.0_f64, 0.0_f64);
      let (mut bw, mut bh) = (0.0_f64, 0.0_f64);
      let mut ad_px = 0.0_f64;
      let mut bd_px = 0.0_f64;
      if let Some(a) = whatsit.get_arg(4) {
        if let Ok((w,h,d,_,_,_)) = a.clone().get_size(None) {
          aw = px(w); let a_h = px(h); ad_px = px(d); ah = a_h + ad_px;
        }
      }
      if let Some(b) = whatsit.get_arg(5) {
        if let Ok((w,h,d,_,_,_)) = b.clone().get_size(None) {
          bw = px(w); let b_h = px(h); bd_px = px(d); bh = b_h + bd_px;
        }
      }
      // Get width from space arg (#3)
      // Perl: $space->getWidth->pxValue
      let w = if let Some(sp) = whatsit.get_arg(3) {
        if let Ok(Some(wd)) = sp.clone().get_width(None) {
          let dim: Dimension = wd.into();
          px(dim)
        } else {
          let (wd,_,_,_,_,_) = sp.clone().get_size(None)?;
          px(wd)
        }
      } else { 0.0 };
      // Perl: $h = $w * abs($diagV / $diagH); — raw pxValue, no rounding
      let h = w * (dv / dh).abs();
      let line = if flip { s!("0,{} {},0", fmtpx(h), fmtpx(w)) } else { s!("0,0, {},{}", fmtpx(w), fmtpx(h)) };
      let ax = if flip { 0.0 } else { w - aw };
      let ay = 0.0_f64;
      let bx = if flip { w - bw } else { 0.0 };
      let by = h - bh;
      // Perl: Dimension($w / $pxppt . 'pt')
      let pxppt = 100.0 / 72.27;
      let to_dim_attr = |v: f64| -> String {
        use latexml_core::common::dimension::attribute_format;
        use latexml_core::common::numeric_ops::kround;
        attribute_format(kround(v / pxppt * 65536.0), None)
      };
      // Perl: picture width/height use &pxValue(#width) which roundto(2dp)
      let fmtpx2 = |v: f64| -> String {
        let r = roundto2(v);
        if r == 0.0 { return "0".to_string(); }
        let s = s!("{:.2}", r);
        let s = s.trim_end_matches('0');
        s.trim_end_matches('.').to_string()
      };
      whatsit.set_property("pxwidth", Stored::from(fmtpx2(w)));
      whatsit.set_property("pxheight", Stored::from(fmtpx2(h)));
      whatsit.set_property("width", Stored::from(to_dim_attr(w)));
      whatsit.set_property("height", Stored::from(to_dim_attr(h)));
      whatsit.set_property("A", whatsit.get_arg(4).cloned().map(Stored::from).unwrap_or(Stored::None));
      whatsit.set_property("Atransform", Stored::from(s!("{},{}", fmtpx(ax), fmtpx(ay))));
      whatsit.set_property("Aw", Stored::from(fmtpx(aw)));
      whatsit.set_property("Ah", Stored::from(fmtpx(ah)));
      whatsit.set_property("Ad", Stored::from(fmtpx(ad_px)));
      whatsit.set_property("B", whatsit.get_arg(5).cloned().map(Stored::from).unwrap_or(Stored::None));
      whatsit.set_property("Btransform", Stored::from(s!("{},{}", fmtpx(bx), fmtpx(by))));
      whatsit.set_property("Bw", Stored::from(fmtpx(bw)));
      whatsit.set_property("Bh", Stored::from(fmtpx(bh)));
      whatsit.set_property("Bd", Stored::from(fmtpx(bd_px)));
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
