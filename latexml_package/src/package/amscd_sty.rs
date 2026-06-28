use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("amsgen");

  DefMacro!(
    "\\CD",
    "\\lx@ams@CD{name=CD,datameaning=commutative-diagram}"
  );
  DefMacro!(
    "\\lx@ams@CD RequiredKeyVals:lx@GEN",
    "\\lx@gen@matrix@bindings{#1}\\lx@ams@CD@bindings\\lx@ams@matrix@{#1}\\lx@begin@alignment"
  );
  DefMacro!("\\endCD", "\\lx@end@alignment\\lx@end@gen@matrix");

  // Perl: DefPrimitive('\lx@ams@CD@bindings', sub {
  //     Let("\\\\", '\lx@alignment@newline@noskip');
  //     $STATE->assignMathcode('@' => 0x8000);
  //     Let('@', '\cd@'); });
  DefPrimitive!(T_CS!("\\lx@ams@CD@bindings"), None, {
    Let!("\\\\", "\\lx@alignment@newline@noskip");
    assign_mathcode('@', 0x8000u16, None);
    Let!("@", "\\cd@");
  });

  // Perl: DefMacro('\cd@ Token', sub {
  //     my ($gullet, $token) = @_;
  //     (T_CS('@' . ToString($token))); });
  // Implemented as a primitive that reads a token and unreads the appropriate CS.
  DefPrimitive!(T_CS!("\\cd@"), None, {
    // `\cd@` requires a following token to build the `@<token>` connector CS; on
    // input-exhaustion emit the parity "file ended" error instead of panicking.
    let Some(token) = read_token_required("\\cd@")? else { return Ok(vec![]); };
    let cs_name = token.with_str(|s| format!("@{s}"));
    unread(Tokens::from(T_CS!(&*cs_name)));
  });

  // Horizontal connectors
  // Perl: DefMacroI(T_CS('@>'), 'Until:> Until:>',
  //   '\lx@hidden@align\lx@amscd@stack{>}{\lx@amscd@rightarrow}{#1}{#2}\lx@hidden@align');
  DefMacro!(
    T_CS!("@>"),
    "Until:> Until:>",
    "\\lx@hidden@align\\lx@amscd@stack{>}{\\lx@amscd@rightarrow}{#1}{#2}\\lx@hidden@align"
  );
  DefMacro!(
    T_CS!("@)"),
    "Until:) Until:)",
    "\\lx@hidden@align\\lx@amscd@stack{)}{\\lx@amscd@rightarrow}{#1}{#2}\\lx@hidden@align"
  );
  DefMacro!(
    T_CS!("@<"),
    "Until:< Until:<",
    "\\lx@hidden@align\\lx@amscd@stack{<}{\\lx@amscd@leftarrow}{#1}{#2}\\lx@hidden@align"
  );
  DefMacro!(
    T_CS!("@("),
    "Until:( Until:(",
    "\\lx@hidden@align\\lx@amscd@stack{(}{\\lx@amscd@leftarrow}{#1}{#2}\\lx@hidden@align"
  );
  DefMacro!(
    T_CS!("@="),
    None,
    "\\lx@hidden@align\\lx@amscd@equals\\lx@hidden@align"
  );

  // Vertical connectors
  DefMacro!(
    T_CS!("@A"),
    "Until:A Until:A",
    "\\lx@amscd@adjacent{A}{\\Big\\uparrow}{#1}{#2}\\lx@hidden@align\\lx@hidden@align"
  );
  DefMacro!(
    T_CS!("@V"),
    "Until:V Until:V",
    "\\lx@amscd@adjacent{V}{\\Big\\downarrow}{#1}{#2}\\lx@hidden@align\\lx@hidden@align"
  );

  DefMacro!(
    T_CS!("@|"),
    None,
    "\\Big\\Vert\\lx@hidden@align\\lx@hidden@align"
  );
  DefMacro!(
    T_CS!("@\\vert"),
    None,
    "\\Big\\Vert\\lx@hidden@align\\lx@hidden@align"
  );
  DefMacro!(T_CS!("@."), None, "\\lx@hidden@align\\lx@hidden@align");

  DefRegister!("\\minaw@" => Dimension!("11.111pt"));

  // Perl: DefConstructor('\lx@amscd@stack Undigested {} ScriptStyle ScriptStyle', sub { ... },
  //   properties => { scriptpos => sub { "mid" . $_[0]->getScriptLevel; } },
  //   reversion  => '@#1{#3}#1{#4}#1');
  DefConstructor!("\\lx@amscd@stack Undigested {} ScriptStyle ScriptStyle",
    sub[document, args, props] {
      // args: [0]=reversion(Undigested), [1]=op({}), [2]=over(ScriptStyle), [3]=under(ScriptStyle)
      // Probe scriptpos in place — only resolve to an owned String
      // when the value is non-empty (most amscd cells have no override).
      let scriptpos_attr = props.get("scriptpos").and_then(|v| match v {
        Stored::String(s) if !with(*s, |p| p.is_empty()) => {
          Some(("scriptpos".to_string(), to_string(*s)))
        },
        _ => None,
      });

      let op = args.get(1).and_then(|a| a.as_ref());
      let over = args.get(2).and_then(|a| a.as_ref());
      let under = args.get(3).and_then(|a| a.as_ref());

      let under_empty = under.map(|u| u.is_empty().unwrap_or(true)).unwrap_or(true);
      let over_empty = over.map(|o| o.is_empty().unwrap_or(true)).unwrap_or(true);

      if !under_empty {
        // outer XMApp with SUBSCRIPTOP
        let outer_attrs: HashMap<String, String> = map!("role" => "ARROW".to_string());
        document.open_element("ltx:XMApp", Some(outer_attrs), None)?;
        let mut sub_attrs = map!("role" => "SUBSCRIPTOP".to_string());
        if let Some((ref k, ref v)) = scriptpos_attr { sub_attrs.insert(k.clone(), v.clone()); }
        document.insert_element("ltx:XMTok", Vec::new(), Some(sub_attrs))?;

        if !over_empty {
          // inner XMApp with SUPERSCRIPTOP
          document.open_element("ltx:XMApp", None, None)?;
          let mut sup_attrs = map!("role" => "SUPERSCRIPTOP".to_string());
          if let Some((k, v)) = scriptpos_attr.clone() { sup_attrs.insert(k, v); }
          document.insert_element("ltx:XMTok", Vec::new(), Some(sup_attrs))?;
          if let Some(op_val) = op {
            document.insert_element("ltx:XMArg", vec![op_val], None)?;
          }
          if let Some(over_val) = over {
            document.insert_element("ltx:XMArg", vec![over_val], None)?;
          }
          document.close_element("ltx:XMApp")?;
        } else {
          if let Some(op_val) = op {
            document.insert_element("ltx:XMArg", vec![op_val], None)?;
          }
        }
        if let Some(under_val) = under {
          document.insert_element("ltx:XMArg", vec![under_val], None)?;
        }
        document.close_element("ltx:XMApp")?;
      } else if !over_empty {
        // XMApp with SUPERSCRIPTOP only
        document.open_element("ltx:XMApp", None, None)?;
        let mut sup_attrs = map!("role" => "SUPERSCRIPTOP".to_string());
        if let Some((k, v)) = scriptpos_attr { sup_attrs.insert(k, v); }
        document.insert_element("ltx:XMTok", Vec::new(), Some(sup_attrs))?;
        if let Some(op_val) = op {
          document.insert_element("ltx:XMArg", vec![op_val], None)?;
        }
        if let Some(over_val) = over {
          document.insert_element("ltx:XMArg", vec![over_val], None)?;
        }
        document.close_element("ltx:XMApp")?;
      } else {
        // Just the operator
        if let Some(op_val) = op {
          document.insert_element("ltx:XMArg", vec![op_val], None)?;
        }
      }
    },
    properties => sub[_args] {
      let scriptpos = format!("mid{}", get_script_level());
      Ok(stored_map!("scriptpos" => scriptpos))
    },
    reversion => "@#1{#3}#1{#4}#1"
  );

  // Perl: DefConstructor('\lx@amscd@adjacent Undigested {} ScriptStyle ScriptStyle', sub { ... },
  //   reversion => '@#1{#3}#1{#4}#1');
  DefConstructor!("\\lx@amscd@adjacent Undigested {} ScriptStyle ScriptStyle",
    sub[document, args, _props] {
      // args: [0]=reversion(Undigested), [1]=op({}), [2]=left(ScriptStyle), [3]=right(ScriptStyle)
      let op = args.get(1).and_then(|a| a.as_ref());
      let left = args.get(2).and_then(|a| a.as_ref());
      let right = args.get(3).and_then(|a| a.as_ref());

      let left_empty = left.map(|l| l.is_empty().unwrap_or(true)).unwrap_or(true);
      let right_empty = right.map(|r| r.is_empty().unwrap_or(true)).unwrap_or(true);

      // Make the left & right parts width=0, so they don't affect centering
      document.open_element("ltx:XMWrap",
        Some(map!("role" => "ARROW".to_string())), None)?;
      if !left_empty
        && let Some(left_val) = left {
          document.insert_element("ltx:XMArg", vec![left_val],
            Some(map!("width" => "0.0pt".to_string())))?;
        }
      if let Some(op_val) = op {
        document.insert_element("ltx:XMArg", vec![op_val], None)?;
      }
      if !right_empty
        && let Some(right_val) = right {
          document.insert_element("ltx:XMArg", vec![right_val],
            Some(map!("width" => "0.0pt".to_string())))?;
        }
      document.close_element("ltx:XMWrap")?;
    },
    reversion => "@#1{#3}#1{#4}#1"
  );

  // These, in case used...
  DefMacro!("\\leftarrowfill@ {}", "\\lx@amscd@leftarrow");
  DefMacro!("\\rightarrowfill@ {}", "\\lx@amscd@rightarrow");
  DefMacro!("\\leftrightarrowfill@ {}", "\\lx@amscd@leftrightarrow");

  // These are stretchy, widened version; should be \minCDarrowwidth or \minaw@ wide
  // Perl: DefPrimitive('\lx@amscd@leftarrow', sub {
  //     Box("\x{2190}", undef, undef, T_CS('\leftarrow'),
  //       role => 'ARROW', stretchy => 'true', meaning => 'leftarrow',
  //       class=>'ltx_horizontally_stretchy',  width => Dimension('30pt')); });
  DefPrimitive!(T_CS!("\\lx@amscd@leftarrow"), None, {
    Tbox::new(
      pin_static("\u{2190}"),
      None,
      None,
      Tokens!(T_CS!("\\leftarrow")),
      stored_map!(
        "role" => "ARROW",
        "stretchy" => "true",
        "meaning" => "leftarrow",
        "class" => "ltx_horizontally_stretchy",
        "width" => Dimension!("30pt")
      ),
    )
  });

  DefPrimitive!(T_CS!("\\lx@amscd@rightarrow"), None, {
    Tbox::new(
      pin_static("\u{2192}"),
      None,
      None,
      Tokens!(T_CS!("\\rightarrow")),
      stored_map!(
        "role" => "ARROW",
        "stretchy" => "true",
        "meaning" => "rightarrow",
        "class" => "ltx_horizontally_stretchy",
        "width" => Dimension!("30pt")
      ),
    )
  });

  DefPrimitive!(T_CS!("\\lx@amscd@leftrightarrow"), None, {
    Tbox::new(
      pin_static("\u{2194}"),
      None,
      None,
      Tokens!(T_CS!("\\leftrightarrow")),
      stored_map!(
        "role" => "ARROW",
        "stretchy" => "true",
        "meaning" => "leftrightarrow",
        "class" => "ltx_horizontally_stretchy",
        "width" => Dimension!("30pt")
      ),
    )
  });

  DefPrimitive!(T_CS!("\\lx@amscd@equals"), None, {
    Tbox::new(
      pin_static("="),
      None,
      None,
      Tokens!(T_OTHER!("=")),
      stored_map!(
        "role" => "ARROW",
        "stretchy" => "true",
        "meaning" => "equals",
        "class" => "ltx_horizontally_stretchy",
        "width" => Dimension!("30pt")
      ),
    )
  });

  DefRegister!("\\minCDarrowwidth" => Dimension!("2.5pc"));
});
