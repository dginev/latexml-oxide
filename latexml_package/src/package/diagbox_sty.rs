/// Perl: diagbox.sty.ltxml — diagonal box headers in tabulars
use crate::prelude::*;

fn roundto(v: f64) -> f64 { (v * 100.0).round() / 100.0 }
fn max_f(a: f64, b: f64) -> f64 { if a > b { a } else { b } }

/// Build `\diagbox[dir=<dir>{,width=…}{,font=…}]{#3}{#4}` from slashbox /
/// backslashbox args. Perl diagbox.sty.ltxml L177-188 uses
/// `\if.\detokenize{#i}.` to emit the comma-prefixed keyval segments;
/// we do the same check at the token level to avoid LaTeXML's
/// re-tokenization dropping the segment.
fn build_diagbox_invocation(dir: &str, args: Vec<ArgWrap>) -> Tokens {
  let width_tokens: Tokens = args[0].clone().into();
  let font_tokens: Tokens = args[1].clone().into();
  let a_tokens: Tokens = args[2].clone().into();
  let b_tokens: Tokens = args[3].clone().into();

  let mut out: Vec<Token> = Vec::new();
  out.push(T_CS!("\\diagbox"));
  out.push(T_OTHER!("["));
  out.extend(ExplodeText!("dir="));
  out.extend(ExplodeText!(dir));
  if !width_tokens.unlist_ref().is_empty() {
    out.extend(ExplodeText!(",width="));
    out.extend(width_tokens.unlist());
  }
  if !font_tokens.unlist_ref().is_empty() {
    out.extend(ExplodeText!(",font="));
    out.extend(font_tokens.unlist());
  }
  out.push(T_OTHER!("]"));
  out.push(T_BEGIN!());
  out.extend(a_tokens.unlist());
  out.push(T_END!());
  out.push(T_BEGIN!());
  out.extend(b_tokens.unlist());
  out.push(T_END!());
  Tokens::new(out)
}

#[rustfmt::skip]
LoadDefinitions!({
  // Ensure <ltx:picture> gets xml:id generation (in case makecell isn't loaded)
  Tag!("ltx:picture", after_open => sub[document, node] {
    let _ = document.generate_id(node, "pic");
  });

  DefKeyVal!("diagbox", "dir", "");
  DefKeyVal!("diagbox", "width", "Dimension");
  DefKeyVal!("diagbox", "height", "Dimension");
  DefKeyVal!("diagbox", "innerwidth", "Dimension");
  DefKeyVal!("diagbox", "font", "");
  DefKeyVal!("diagbox", "linewidth", "");
  DefKeyVal!("diagbox", "linecolor", "");
  for key in ["innerleftsep", "innerrightsep", "outerleftsep", "outerrightsep",
              "leftsep", "rightsep"] {
    DefKeyVal!("diagbox", key, "Dimension");
  }
  DefKeyVal!("diagbox", "trim", "");

  // \diagbox sniffs for optional 3rd argument
  DefMacro!("\\diagbox OptionalKeyVals:diagbox {}{}", sub[args] {
    let mut it = args.into_iter();
    let kv_arg = it.next().unwrap();
    let a_arg: Tokens = it.next().unwrap().into();
    let b_arg: Tokens = it.next().unwrap().into();

    let has_third = gullet::if_next(T_BEGIN!())?;
    let (a_content, m_content, b_content) = if has_third {
      // Perl: $gullet->readArg — reads without expansion
      let third = gullet::read_arg(ExpansionLevel::Off)?;
      (a_arg, Some(b_arg), third)
    } else {
      (a_arg, None, b_arg)
    };

    let dir = if let ArgWrap::KV(ref kv) = kv_arg {
      kv.get_value("dir").map(|t| t.to_string()).unwrap_or_else(|| "NW".to_string())
    } else { "NW".to_string() };

    let font_tokens: Tokens = if let ArgWrap::KV(ref kv) = kv_arg {
      kv.get_value("font").map(|t| Tokens::from(t.clone())).unwrap_or(Tokens!())
    } else { Tokens!() };

    let align_m = if dir.contains('W') { "l" } else { "r" };

    let mut result = Vec::new();
    result.push(T_CS!("\\lx@diagbox"));
    // Keyvals arg — revert back to tokens (into() loses KV data, must use revert)
    let kv_toks: Tokens = kv_arg.revert()?;
    result.push(T_BEGIN!()); result.extend(kv_toks.unlist()); result.push(T_END!());
    // A head
    result.push(T_BEGIN!()); result.push(T_CS!("\\lx@diagbox@head"));
    result.push(T_BEGIN!()); result.push(T_OTHER!("l")); result.push(T_END!());
    result.push(T_BEGIN!()); result.extend_from_slice(font_tokens.unlist_ref()); result.push(T_END!());
    result.push(T_BEGIN!()); result.extend(a_content.unlist()); result.push(T_END!());
    result.push(T_END!());
    // M head (optional)
    if let Some(m) = m_content {
      result.push(T_OTHER!("["));
      result.push(T_CS!("\\lx@diagbox@head"));
      result.push(T_BEGIN!()); result.extend(ExplodeText!(align_m)); result.push(T_END!());
      result.push(T_BEGIN!()); result.extend_from_slice(font_tokens.unlist_ref()); result.push(T_END!());
      result.push(T_BEGIN!()); result.extend(m.unlist()); result.push(T_END!());
      result.push(T_OTHER!("]"));
    }
    // B head
    result.push(T_BEGIN!()); result.push(T_CS!("\\lx@diagbox@head"));
    result.push(T_BEGIN!()); result.push(T_OTHER!("r")); result.push(T_END!());
    result.push(T_BEGIN!()); result.extend_from_slice(font_tokens.unlist_ref()); result.push(T_END!());
    result.push(T_BEGIN!()); result.extend(b_content.unlist()); result.push(T_END!());
    result.push(T_END!());

    Tokens::new(result)
  });

  DefMacro!("\\lx@diagbox@head{}{}{}", "{#2\\shortstack[#1]{#3}}");

  // The constructor creates a <picture> with diagonal lines and positioned text.
  // Template creates the picture wrapper with xml:id; afterConstruct fills content.
  DefConstructor!("\\lx@diagbox RequiredKeyVals:diagbox {}[]{}",
    "<ltx:picture xml:id='#id' width='#width' height='#height'></ltx:picture>",
    reversion => "\\diagbox[#1]{#2}{#4}",
    after_construct => sub[document, whatsit] {
      let mut node = document.get_node().clone();
      document.add_class(&mut node, "ltx_nopad")?;
      // Find the picture element we just created
      let children = node.get_child_nodes();
      if let Some(mut picture) = children.into_iter().rev().find(|c| c.get_name() == "picture") {
        let ns = picture.get_namespace();
        let doc_ptr = document.get_document();
        // Add line elements
        for line_key in &["line1", "line2"] {
          if let Some(pts) = whatsit.get_property(line_key) {
            let s = pts.to_string();
            if !s.is_empty() {
              let color = whatsit.get_property("linecolor").map(|c| c.to_string()).unwrap_or_else(|| "#000000".to_string());
              let lw = whatsit.get_property("linewidth").map(|c| c.to_string()).unwrap_or_else(|| "0.4".to_string());
              let mut line_node = libxml::tree::Node::new("line", ns.clone(), doc_ptr).unwrap();
              let _ = line_node.set_attribute("points", &s);
              let _ = line_node.set_attribute("stroke", &color);
              let _ = line_node.set_attribute("stroke-width", &lw);
              picture.add_child(&mut line_node)?;
            }
          }
        }
        // Collect A, B, M content from whatsit properties BEFORE mutating document
        let mut groups: Vec<(String, String, String, String, Digested)> = Vec::new();
        for prefix in &["A", "B", "M"] {
          if let Some(Stored::Digested(content)) = whatsit.get_property(prefix).map(|v| v.into_owned()) {
            let px = whatsit.get_property(&s!("{prefix}x")).map(|v| v.to_string()).unwrap_or_else(|| "0".to_string());
            let py = whatsit.get_property(&s!("{prefix}y")).map(|v| v.to_string()).unwrap_or_else(|| "0".to_string());
            let pw = whatsit.get_property(&s!("{prefix}w")).map(|v| v.to_string()).unwrap_or_default();
            let ph = whatsit.get_property(&s!("{prefix}h")).map(|v| v.to_string()).unwrap_or_default();
            groups.push((px, py, pw, ph, content));
          }
        }
        // Now add groups to DOM
        // Perl: translate coordinates get Perl's default number formatting
        // which naturally rounds float artifacts (18.939999... → "18.94")
        let fmt_coord = |s: &str| -> String {
          if let Ok(v) = s.parse::<f64>() {
            let r = (v * 100.0 * (1.0 + 100.0 * f64::EPSILON)).round() / 100.0;
            if r == 0.0 { return "0".to_string(); }
            let fs = format!("{:.2}", r);
            let fs = fs.trim_end_matches('0');
            fs.trim_end_matches('.').to_string()
          } else { s.to_string() }
        };
        for (px, py, pw, ph, content) in groups {
          let mut g_attrs = HashMap::default();
          g_attrs.insert("transform".to_string(), s!("translate({},{})", fmt_coord(&px), fmt_coord(&py)));
          if !pw.is_empty() { g_attrs.insert("innerwidth".to_string(), pw); }
          if !ph.is_empty() { g_attrs.insert("innerheight".to_string(), ph); }
          g_attrs.insert("class".to_string(), "ltx_svg_fog".to_string());
          // Temporarily set cursor to picture, open g, open inline-block, absorb, close both
          document.set_node(&picture);
          document.open_element("ltx:g", Some(g_attrs), None)?;
          document.open_element("ltx:inline-block", None, None)?;
          document.absorb(&content, None)?;
          document.close_element("ltx:inline-block")?;
          document.close_element("ltx:g")?;
        }
        // Restore cursor
        document.set_node(&node);
      }
    },
    after_digest => sub[whatsit] {
      let args = whatsit.get_args();
      let a = args.get(1).and_then(|a| a.clone());
      let m = args.get(2).and_then(|a| a.clone());
      let b = args.get(3).and_then(|a| a.clone());

      // Perl: map { roundto($_->pxValue) } $X->getSize; $Xh += $Xd;
      let (aw, ah, _ad) = if let Some(ref d) = a {
        let (w, h, d, _, _, _) = d.clone().get_size(None)?;
        (roundto(w.px_value(None)), roundto(h.px_value(None) + d.px_value(None)), roundto(d.px_value(None)))
      } else { (0.0, 0.0, 0.0) };
      let (bw, bh, _bd) = if let Some(ref d) = b {
        let (w, h, d, _, _, _) = d.clone().get_size(None)?;
        (roundto(w.px_value(None)), roundto(h.px_value(None) + d.px_value(None)), roundto(d.px_value(None)))
      } else { (0.0, 0.0, 0.0) };
      let (mw, mh, _md) = if let Some(ref d) = m {
        let (w, h, d, _, _, _) = d.clone().get_size(None)?;
        (roundto(w.px_value(None)), roundto(h.px_value(None) + d.px_value(None)), roundto(d.px_value(None)))
      } else { (0.0, 0.0, 0.0) };

      // Extract KeyVals from the digested first argument (DigestedData::KeyVals)
      let kv_arg = args.first().and_then(|a| a.clone());
      let kv = kv_arg.as_ref().and_then(|d| {
        if let DigestedData::KeyVals(ref kv) = *d.data() { Some(kv) } else { None }
      });

      // Perl: my $dir = $kv && ToString($kv->getValue('dir')) || 'NW';
      let dir = kv.and_then(|kv| kv.get_value_digested("dir"))
        .map(|v| v.to_string())
        .unwrap_or_else(|| "NW".to_string());

      // Perl: $w = $w->pxValue if $w;  (width/innerwidth are Dimension keyvals)
      let kv_w: Option<f64> = kv.and_then(|kv| {
        kv.get_value_digested("width").or_else(|| kv.get_value_digested("innerwidth"))
      }).and_then(|v| v.get_dimension()).map(|d| d.px_value(None));
      let kv_h: Option<f64> = kv.and_then(|kv| kv.get_value_digested("height"))
        .and_then(|v| v.get_dimension()).map(|d| d.px_value(None));

      #[allow(unused_assignments)]
      let (mut line1, mut line2) = (String::new(), String::new());
      #[allow(unused_assignments, unused_mut)]
      let (mut ax, mut ay, mut bx, mut by, mut mx, mut my) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
      let has_m = m.is_some();
      let (w, h);

      if has_m {
        // Perl diagbox.pdf: quadratic formula for optimal 3-part box sizing
        if kv_w.is_none() || kv_h.is_none() {
          let t = ah + mh;
          let s = bw + mw;
          let v = s * t - aw * bh;
          let u = aw * mh - mw * bh;
          let delta = (u + v) * (u + v) + 4.0 * aw * (t - bh) * (mw * (bh - mh) - bw * mh);
          w = roundto(kv_w.unwrap_or_else(|| {
            if bh < t && delta >= 0.0 { (u + v + delta.sqrt()) / (t - bh) / 2.0 }
            else { 2.0 * (max_f(aw, bw) + mw) }
          }));
          h = roundto(kv_h.unwrap_or_else(|| {
            if aw < s && delta >= 0.0 { (u - v - delta.sqrt()) / (aw - s) / 2.0 }
            else { 2.0 * (max_f(ah, bh) + mh) }
          }));
        } else {
          w = roundto(kv_w.unwrap());
          h = roundto(kv_h.unwrap());
        }
        let (wm, hm) = (w * 0.5, h * 0.5);
        match dir.as_str() {
          "SE" => { mx = w - mw; bx = w - bw; by = h - bh;
            line1 = s!("0,{h} {w},{hm}"); line2 = s!("0,{h} {wm},0"); },
          "SW" => { ay = h - ah; bx = w - bw;
            line1 = s!("0,{hm} {w},{h}"); line2 = s!("{wm},0 {w},{h}"); },
          "NE" => { ay = h - ah; mx = w - mw; my = h - mh; bx = w - bw;
            line1 = s!("0,0 {wm},{h}"); line2 = s!("0,0 {w},{hm}"); },
          _ /* NW */ => { my = h - mh; bx = w - bw; by = h - bh;
            line1 = s!("{wm},{h} {w},0"); line2 = s!("0,{hm} {w},0"); },
        }
      } else {
        w = roundto(kv_w.unwrap_or(2.0 * max_f(aw, bw)));
        h = roundto(kv_h.unwrap_or(ah + bh));
        bx = w - bw;
        match dir.as_str() {
          "SW" | "NE" => { ay = h - ah; line1 = s!("0,0, {w},{h}"); },
          _ /* NW | SE */ => { by = h - bh; line1 = s!("0,{h} {w},0"); },
        }
      }

      // Perl: linewidth => ($kv && $kv->getValue('linewidth')) || '0.4'
      let linewidth = kv.and_then(|kv| kv.get_value_digested("linewidth"))
        .map(|v| v.to_string()).unwrap_or_else(|| "0.4".to_string());
      // Perl: linecolor => ($kv && $kv->getValue('linecolor')) || Black
      let linecolor = kv.and_then(|kv| kv.get_value_digested("linecolor"))
        .map(|v| v.to_string()).unwrap_or_else(|| "#000000".to_string());

      // Perl does NOT roundto the coordinates — they inherit precision from h/w/Bh/etc.
      // Only w and h are rounded; coordinates are raw arithmetic results.

      // Perl: setProperties with Dimension($w / $pxppt . 'pt') for width/height
      // In Rust, template uses raw px strings, so store as-is
      whatsit.set_property("width", Stored::from(s!("{w}")));
      whatsit.set_property("height", Stored::from(s!("{h}")));
      whatsit.set_property("A", a.map(Stored::from).unwrap_or(Stored::None));
      whatsit.set_property("Ax", Stored::from(s!("{ax}"))); whatsit.set_property("Ay", Stored::from(s!("{ay}")));
      whatsit.set_property("Aw", Stored::from(s!("{aw}"))); whatsit.set_property("Ah", Stored::from(s!("{ah}")));
      whatsit.set_property("B", b.map(Stored::from).unwrap_or(Stored::None));
      whatsit.set_property("Bx", Stored::from(s!("{bx}"))); whatsit.set_property("By", Stored::from(s!("{by}")));
      whatsit.set_property("Bw", Stored::from(s!("{bw}"))); whatsit.set_property("Bh", Stored::from(s!("{bh}")));
      if has_m {
        // Perl: Mw => $Bw (note: Perl uses $Bw for M's width, not $Mw!)
        whatsit.set_property("M", m.map(Stored::from).unwrap_or(Stored::None));
        whatsit.set_property("Mx", Stored::from(s!("{mx}"))); whatsit.set_property("My", Stored::from(s!("{my}")));
        whatsit.set_property("Mw", Stored::from(s!("{bw}"))); whatsit.set_property("Mh", Stored::from(s!("{mh}")));
      }
      whatsit.set_property("line1", Stored::from(line1));
      whatsit.set_property("line2", Stored::from(line2));
      whatsit.set_property("linewidth", Stored::from(linewidth));
      whatsit.set_property("linecolor", Stored::from(linecolor));
    });

  // slashbox / backslashbox — Perl diagbox.sty.ltxml L177-188.
  //
  // Perl's DefMacro uses `\if.\detokenize{#i}.` to route each of two
  // optional args into a keyval segment. Porting that TeX-level dance
  // through LaTeXML's re-tokenization silently dropped the width arg
  // (diagboxtest:355 got 66.12pt instead of 78.74pt, plus wrong diagonal
  // direction). Driving the dispatch from a Rust closure that inspects
  // the tokens directly is more robust and equally faithful to the
  // per-call behavior Perl specifies.
  DefMacro!("\\slashbox [][]{}{}", sub[args] {
    build_diagbox_invocation("SW", args)
  });
  DefMacro!("\\backslashbox [][]{}{}", sub[args] {
    build_diagbox_invocation("NW", args)
  });
});
