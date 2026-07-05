use latexml_core::common::color::{self, Color};

use crate::prelude::*;

/// Get the current font's background color as a hex attribute string.
fn current_background_hex() -> String {
  lookup_font()
    .and_then(|f| f.get_background().map(|c| c.to_attribute()))
    .unwrap_or_default()
}

/// Look up a named color from state, returning hex attribute string.
fn lookup_color_hex(name: &str) -> String {
  let key = s!("color_{name}");
  // with_value avoids cloning the Stored envelope on the String arm.
  with_value(&key, |v| match v {
    Some(Stored::String(sym)) => {
      let stored_str = with(*sym, |s| s.to_string());
      Color::from_stored(&stored_str)
        .map(|c| c.to_attribute())
        .unwrap_or_else(|| color::BLACK.to_attribute())
    },
    _ => color::BLACK.to_attribute(),
  })
}

/// Perl #2829: the framed.sty environments pass ALL whatsit properties to
/// `insertBlock`, which filters to the attributes `ltx:figure` accepts.
/// Stringify the attribute-capable Stored values for the block-attr map.
fn props_to_attrs(props: &SymHashMap<Stored>) -> HashMap<String, String> {
  let mut attrs: HashMap<String, String> = HashMap::default();
  for (k, v) in props {
    let key = to_string(*k);
    match v {
      Stored::String(s) => {
        attrs.insert(key, to_string(*s));
      },
      Stored::Dimension(d) => {
        attrs.insert(key, d.to_attribute());
      },
      Stored::Int(i) => {
        attrs.insert(key, i.to_string());
      },
      _ => {}, // body (Digested) and other non-attribute payloads
    }
  }
  attrs
}

LoadDefinitions!({
  // Intentional divergence (WISDOM #44 class: structural-adaptation, applies
  // to the {shaded}/{shaded*}/{snugshade}/{snugshade*}/{titled-frame} envs
  // below): Perl's `beforeDigest => sub { MergeFont(background =>
  // LookupValue('color_shadecolor')); }` wraps the env body in a nested
  // `<ltx:text backgroundcolor="…">` via the font-merge pipeline. Rust's
  // ports look up `shadecolor` directly through `lookup_color_hex()` and
  // set `backgroundcolor` only on the outer block created by `insertBlock`
  // — no nested text wrappers. Matches Perl's visual intent (shaded box
  // with BG colour) while producing a cleaner XML tree. Audit doesn't
  // flag any kind mismatch here since both are DefEnvironment; this
  // umbrella catalogues the `beforeDigest => MergeFont(…)` omission so a
  // future reviewer searching for `color_shadecolor` doesn't re-derive
  // the rationale from scratch.

  // {framed} Normal framed block-level box
  // Perl (#2829): framed.sty.ltxml L21-29
  DefEnvironment!("{framed}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      // Perl #2829: pass ALL properties; insertBlock filters to attributes.
      if let Some(Stored::Digested(body)) = props.get("body") {
        let attrs = props_to_attrs(props);
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      Ok(framed_properties(FramedOptions {
        color: Some(color::BLACK.to_attribute()),
        margin: Some("\\FrameSep".to_string()),
        rule: Some("\\FrameRule".to_string()),
        ..FramedOptions::default()
      }))
    }
  );

  // {oframed} "open" framed box — same as framed for our purposes
  // Perl (#2829): framed.sty.ltxml L31-39
  DefEnvironment!("{oframed}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      // Perl #2829: pass ALL properties; insertBlock filters to attributes.
      if let Some(Stored::Digested(body)) = props.get("body") {
        let attrs = props_to_attrs(props);
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      Ok(framed_properties(FramedOptions {
        color: Some(color::BLACK.to_attribute()),
        margin: Some("\\FrameSep".to_string()),
        rule: Some("\\FrameRule".to_string()),
        ..FramedOptions::default()
      }))
    }
  );

  // {shaded} a shaded box; uses "shadecolor" for background color
  // Perl (#2829): framed.sty.ltxml L42-49
  DefEnvironment!("{shaded}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      // Perl #2829: pass ALL properties; insertBlock filters to attributes.
      if let Some(Stored::Digested(body)) = props.get("body") {
        let attrs = props_to_attrs(props);
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      // Rust divergence (documented above): shadecolor looked up directly
      // instead of the beforeDigest MergeFont pipeline.
      Ok(framed_properties(FramedOptions {
        backgroundcolor: Some(lookup_color_hex("shadecolor")),
        margin: Some("\\FrameSep".to_string()),
        ..FramedOptions::default()
      }))
    }
  );

  // {shaded*} Same as {shaded}
  // Perl (#2829): framed.sty.ltxml L52-60
  DefEnvironment!("{shaded*}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      // Perl #2829: pass ALL properties; insertBlock filters to attributes.
      if let Some(Stored::Digested(body)) = props.get("body") {
        let attrs = props_to_attrs(props);
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      // Rust divergence (documented above): shadecolor looked up directly
      // instead of the beforeDigest MergeFont pipeline.
      Ok(framed_properties(FramedOptions {
        backgroundcolor: Some(lookup_color_hex("shadecolor")),
        margin: Some("\\FrameSep".to_string()),
        ..FramedOptions::default()
      }))
    }
  );

  // {snugshade} Same as {shaded} — #2829 switched it to \FrameSep too
  // Perl (#2829): framed.sty.ltxml L63-71
  DefEnvironment!("{snugshade}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      // Perl #2829: pass ALL properties; insertBlock filters to attributes.
      if let Some(Stored::Digested(body)) = props.get("body") {
        let attrs = props_to_attrs(props);
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      // Rust divergence (documented above): shadecolor looked up directly
      // instead of the beforeDigest MergeFont pipeline.
      Ok(framed_properties(FramedOptions {
        backgroundcolor: Some(lookup_color_hex("shadecolor")),
        margin: Some("\\FrameSep".to_string()),
        ..FramedOptions::default()
      }))
    }
  );

  // {snugshade*}
  // Perl: framed.sty.ltxml lines 85-94
  DefEnvironment!("{snugshade*}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      let bg = props.get("backgroundcolor").map(|v| v.to_string()).unwrap_or_default();
      let cssstyle = props.get("cssstyle").map(|v| v.to_string()).unwrap_or_default();
      let mut attr = string_map!();
      if !bg.is_empty() { attr.insert(s!("backgroundcolor"), bg); }
      if !cssstyle.is_empty() { attr.insert(s!("cssstyle"), cssstyle); }
      if let Some(Stored::Digested(body)) = props.get("body") {
        insert_block(document, body, attr)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      let bg = lookup_color_hex("shadecolor");
      let margin = match LookupRegisterOrDefault!("\\fboxsep") {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 3.0,
      };
      let css = s!("padding:{}pt", margin);
      Ok(stored_map!(
        "backgroundcolor" => bg,
        "cssstyle" => css
      ))
    }
  );

  // {leftbar}
  // Perl (#2829): framed.sty.ltxml L74-80 — direct properties, NOT
  // framedProperties ("Don't overgeneralize framed {leftbar}"): framed=left,
  // color=Black (filtered out by insertBlock — ltx:figure has no `color`
  // attribute, matching the updated Perl fixture), explicit cssstyle, and a
  // padleft Dimension for size computation.
  DefEnvironment!("{leftbar}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      if let Some(Stored::Digested(body)) = props.get("body") {
        let attrs = props_to_attrs(props);
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      let mut props = stored_map!(
        "framed" => "left",
        "color" => color::BLACK.to_attribute(),
        "cssstyle" => "padding-left:10pt; border-left-width:3pt"
      );
      props.insert("padleft", Stored::Dimension(Dimension!("13pt")));
      Ok(props)
    }
  );

  // {titled-frame}
  // Perl: framed.sty.ltxml lines 105-118
  DefEnvironment!("{titled-frame} Undigested",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      // Perl #2829: pass ALL properties; insertBlock filters to attributes.
      if let Some(Stored::Digested(body)) = props.get("body") {
        let attrs = props_to_attrs(props);
        insert_block(document, body, attrs)?;
      }
      Ok(())
    },
    after_digest_begin => sub[whatsit] {
      // Perl: Digest(Invocation(\@titledframe@title, arg1)) — processes title in
      // the current context so it becomes part of the environment body.
      // Use gullet::unread to prepend title tokens to the input stream.
      if let Some(title_arg) = whatsit.get_arg(1) {
        let title_tokens = title_arg.revert()?;
        let mut inv = vec![T_CS!("\\@titledframe@title"), T_BEGIN!()];
        inv.extend(title_tokens.unlist());
        inv.push(T_END!());
        unread(Tokens::new(inv));
      }
    },
    properties => sub[_args] {
      // Perl (#2829): framedProperties(color=>TFFrameColor, backgroundcolor,
      // margin=>'0pt', rule=>'2pt') → padding:0.0pt;border-width:2.0pt.
      Ok(framed_properties(FramedOptions {
        color: Some(lookup_color_hex("TFFrameColor")),
        backgroundcolor: Some(current_background_hex()),
        margin: Some("0pt".to_string()),
        rule: Some("2pt".to_string()),
        ..FramedOptions::default()
      }))
    }
  );
  DefMacro!(
    "\\@titledframe@title{}",
    "\\@@titledframe@title{{\\fboxsep8pt\\fboxrule2pt\\pagecolor{TFFrameColor}\\textcolor{TFTitleColor} {#1}}}"
  );
  DefConstructor!(
    "\\@@titledframe@title{}",
    "<ltx:text cssstyle='display:block;margin:-8pt -8pt 8pt -8pt;padding:8pt'>#1</ltx:text>"
  );

  //======================================================================
  // Customization macros
  // Perl: framed.sty.ltxml lines 126-130
  DefMacro!(
    "\\FrameCommand",
    "\\setlength\\fboxrule{\\FrameRule}\\setlength\\fboxsep{\\FrameSep}\\fbox"
  );
  DefMacro!("\\FirstFrameCommand", "\\FrameCommand");
  DefMacro!("\\MidFrameCommand", "\\FrameCommand");
  DefMacro!("\\LastFrameCommand", "\\FrameCommand");

  // Registers
  // Perl: framed.sty.ltxml lines 134-142
  DefRegister!("\\FrameRule", Dimension!(".4pt"));
  DefRegister!("\\FrameSep", Dimension!("9pt"));
  DefRegister!("\\OuterFrameSep", Dimension!("0pt"));

  // \MakeFramed{settings}...\endMakeFramed — framed.sty primitives that
  // user-defined frame environments wrap (TL framed.sty L86, L114-115).
  // We don't materialize the framing typeset machinery; stub as no-op
  // pass-through. \FrameRestore is invoked inside the settings group
  // and also a no-op.
  // Witness 2405.19660.
  def_macro_noop("\\MakeFramed{}")?;
  def_macro_noop("\\endMakeFramed")?;
  def_macro_noop("\\FrameRestore")?;
});
