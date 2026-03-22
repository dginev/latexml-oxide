/// Perl: diagbox.sty.ltxml — diagonal box headers in tabulars
use crate::prelude::*;

fn roundto(v: f64) -> f64 { (v * 100.0).round() / 100.0 }
fn max_f(a: f64, b: f64) -> f64 { if a > b { a } else { b } }
fn px_value(d: Dimension) -> f64 { d.value_of() as f64 / UNITY as f64 }

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
      let third = gullet::read_arg(ExpansionLevel::Full)?;
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
    // Keyvals arg — convert back to tokens
    let kv_toks: Tokens = kv_arg.into();
    result.push(T_BEGIN!()); result.extend(kv_toks.unlist()); result.push(T_END!());
    // A head
    result.push(T_BEGIN!()); result.push(T_CS!("\\lx@diagbox@head"));
    result.push(T_BEGIN!()); result.push(T_OTHER!("l")); result.push(T_END!());
    result.push(T_BEGIN!()); result.extend(font_tokens.clone().unlist()); result.push(T_END!());
    result.push(T_BEGIN!()); result.extend(a_content.unlist()); result.push(T_END!());
    result.push(T_END!());
    // M head (optional)
    if let Some(m) = m_content {
      result.push(T_OTHER!("["));
      result.push(T_CS!("\\lx@diagbox@head"));
      result.push(T_BEGIN!()); result.extend(ExplodeText!(align_m)); result.push(T_END!());
      result.push(T_BEGIN!()); result.extend(font_tokens.clone().unlist()); result.push(T_END!());
      result.push(T_BEGIN!()); result.extend(m.unlist()); result.push(T_END!());
      result.push(T_OTHER!("]"));
    }
    // B head
    result.push(T_BEGIN!()); result.push(T_CS!("\\lx@diagbox@head"));
    result.push(T_BEGIN!()); result.push(T_OTHER!("r")); result.push(T_END!());
    result.push(T_BEGIN!()); result.extend(font_tokens.clone().unlist()); result.push(T_END!());
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
        for (px, py, pw, ph, content) in groups {
          // Use document's open/close element API instead of raw libxml
          let mut g_attrs = HashMap::default();
          g_attrs.insert("transform".to_string(), s!("translate({px},{py})"));
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

      let (aw, ah, _ad) = if let Some(ref d) = a {
        let (w, h, d, _, _, _) = d.clone().get_size(None)?;
        (roundto(px_value(w)), roundto(px_value(h) + px_value(d)), roundto(px_value(d)))
      } else { (0.0, 0.0, 0.0) };
      let (bw, bh, _bd) = if let Some(ref d) = b {
        let (w, h, d, _, _, _) = d.clone().get_size(None)?;
        (roundto(px_value(w)), roundto(px_value(h) + px_value(d)), roundto(px_value(d)))
      } else { (0.0, 0.0, 0.0) };
      let (mw, mh, _md) = if let Some(ref d) = m {
        let (w, h, d, _, _, _) = d.clone().get_size(None)?;
        (roundto(px_value(w)), roundto(px_value(h) + px_value(d)), roundto(px_value(d)))
      } else { (0.0, 0.0, 0.0) };

      let kv = args.first().and_then(|a| a.clone());
      let dir = kv.as_ref().and_then(|k| k.get_property("dir")).map(|v| v.to_string()).unwrap_or_else(|| "NW".to_string());

      let kv_w: Option<f64> = kv.as_ref().and_then(|k| {
        k.get_property("width").or_else(|| k.get_property("innerwidth"))
      }).and_then(|v| {
        let s = v.to_string();
        Dimension::spec_to_f64(&s).ok().map(|f| f / UNITY as f64)
      });
      let kv_h: Option<f64> = kv.as_ref().and_then(|k| k.get_property("height"))
        .and_then(|v| {
          let s = v.to_string();
          Dimension::spec_to_f64(&s).ok().map(|f| f / UNITY as f64)
        });

      #[allow(unused_assignments)]
      let (mut line1, mut line2) = (String::new(), String::new());
      #[allow(unused_assignments, unused_mut)]
      let (mut ax, mut ay, mut bx, mut by, mut mx, mut my) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
      let has_m = m.is_some() && mw > 0.0;
      let (w, h);

      if has_m {
        w = roundto(kv_w.unwrap_or(2.0 * (max_f(aw, bw) + mw)));
        h = roundto(kv_h.unwrap_or(2.0 * (max_f(ah, bh) + mh)));
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

      let linewidth = kv.as_ref().and_then(|k| k.get_property("linewidth")).map(|v| v.to_string()).unwrap_or_else(|| "0.4".to_string());
      let linecolor = kv.as_ref().and_then(|k| k.get_property("linecolor")).map(|v| v.to_string()).unwrap_or_else(|| "#000000".to_string());

      whatsit.set_property("width", Stored::from(s!("{w}")));
      whatsit.set_property("height", Stored::from(s!("{h}")));
      whatsit.set_property("A", a.map(Stored::from).unwrap_or(Stored::None));
      whatsit.set_property("Ax", Stored::from(s!("{ax}"))); whatsit.set_property("Ay", Stored::from(s!("{ay}")));
      whatsit.set_property("Aw", Stored::from(s!("{aw}"))); whatsit.set_property("Ah", Stored::from(s!("{ah}")));
      whatsit.set_property("B", b.map(Stored::from).unwrap_or(Stored::None));
      whatsit.set_property("Bx", Stored::from(s!("{bx}"))); whatsit.set_property("By", Stored::from(s!("{by}")));
      whatsit.set_property("Bw", Stored::from(s!("{bw}"))); whatsit.set_property("Bh", Stored::from(s!("{bh}")));
      if has_m {
        whatsit.set_property("M", m.map(Stored::from).unwrap_or(Stored::None));
        whatsit.set_property("Mx", Stored::from(s!("{mx}"))); whatsit.set_property("My", Stored::from(s!("{my}")));
        whatsit.set_property("Mw", Stored::from(s!("{mw}"))); whatsit.set_property("Mh", Stored::from(s!("{mh}")));
      }
      whatsit.set_property("line1", Stored::from(line1));
      whatsit.set_property("line2", Stored::from(line2));
      whatsit.set_property("linewidth", Stored::from(linewidth));
      whatsit.set_property("linecolor", Stored::from(linecolor));
    });

  // slashbox/backslashbox compatibility (Perl uses complex \edef to build keyvals)
  RawTeX!(r"
\def\slashbox{\@ifnextchar[{\@slashbox}{\diagbox[dir=SW]}}
\def\@slashbox[#1]{\diagbox[dir=SW,width=#1]}
\def\backslashbox{\@ifnextchar[{\@backslashbox}{\diagbox[dir=NW]}}
\def\@backslashbox[#1]{\diagbox[dir=NW,width=#1]}
");
});
