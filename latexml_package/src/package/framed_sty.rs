use crate::prelude::*;
use latexml_core::common::color::{self, Color};

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
  state::with_value(&key, |v| match v {
    Some(Stored::String(sym)) => {
      let stored_str = arena::with(*sym, |s| s.to_string());
      Color::from_stored(&stored_str)
        .map(|c| c.to_attribute())
        .unwrap_or_else(|| color::BLACK.to_attribute())
    },
    _ => color::BLACK.to_attribute(),
  })
}


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
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
  // Perl: framed.sty.ltxml lines 21-30
  DefEnvironment!("{framed}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      let framecolor = props.get("framecolor").map(|v| v.to_string()).unwrap_or_default();
      let cssstyle = props.get("cssstyle").map(|v| v.to_string()).unwrap_or_default();
      let mut attr = string_map!("framed" => "rectangle");
      if !framecolor.is_empty() { attr.insert(s!("framecolor"), framecolor); }
      if !cssstyle.is_empty() { attr.insert(s!("cssstyle"), cssstyle); }
      if let Some(Stored::Digested(body)) = props.get("body") {
        insert_block(document, body, attr)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      let margin = match LookupRegisterOrDefault!("\\FrameSep") {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 9.0,
      };
      let border = match LookupRegisterOrDefault!("\\FrameRule") {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 0.4,
      };
      let css = s!("padding:{}pt;border-width:{}pt", margin, border);
      Ok(stored_map!(
        "framecolor" => color::BLACK.to_attribute(),
        "cssstyle" => css
      ))
    }
  );

  // {oframed} "open" framed box — same as framed for our purposes
  // Perl: framed.sty.ltxml lines 34-43
  DefEnvironment!("{oframed}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      let framecolor = props.get("framecolor").map(|v| v.to_string()).unwrap_or_default();
      let cssstyle = props.get("cssstyle").map(|v| v.to_string()).unwrap_or_default();
      let mut attr = string_map!("framed" => "rectangle");
      if !framecolor.is_empty() { attr.insert(s!("framecolor"), framecolor); }
      if !cssstyle.is_empty() { attr.insert(s!("cssstyle"), cssstyle); }
      if let Some(Stored::Digested(body)) = props.get("body") {
        insert_block(document, body, attr)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      let margin = match LookupRegisterOrDefault!("\\FrameSep") {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 9.0,
      };
      let border = match LookupRegisterOrDefault!("\\FrameRule") {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 0.4,
      };
      let css = s!("padding:{}pt;border-width:{}pt", margin, border);
      Ok(stored_map!(
        "framecolor" => color::BLACK.to_attribute(),
        "cssstyle" => css
      ))
    }
  );

  // {shaded} a shaded box; uses "shadecolor" for background color
  // Perl: framed.sty.ltxml lines 50-59
  DefEnvironment!("{shaded}",
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
      // Look up shadecolor directly instead of going through font merge
      // (font merge creates <text backgroundcolor> wrappers we don't want)
      let bg = lookup_color_hex("shadecolor");
      let margin = match LookupRegisterOrDefault!("\\FrameSep") {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 9.0,
      };
      let css = s!("padding:{}pt", margin);
      Ok(stored_map!(
        "backgroundcolor" => bg,
        "cssstyle" => css
      ))
    }
  );

  // {shaded*} Same as {shaded}
  // Perl: framed.sty.ltxml lines 62-72
  DefEnvironment!("{shaded*}",
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
      let margin = match LookupRegisterOrDefault!("\\FrameSep") {
        RegisterValue::Dimension(d) => d.pt_value(None),
        _ => 9.0,
      };
      let css = s!("padding:{}pt", margin);
      Ok(stored_map!(
        "backgroundcolor" => bg,
        "cssstyle" => css
      ))
    }
  );

  // {snugshade} — tighter shading, uses \fboxsep not \FrameSep
  // Perl: framed.sty.ltxml lines 75-84
  DefEnvironment!("{snugshade}",
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
  // Perl: framed.sty.ltxml lines 95-100
  DefEnvironment!("{leftbar}",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      let framecolor = props.get("framecolor").map(|v| v.to_string()).unwrap_or_default();
      let mut attr = string_map!(
        "framed" => "left",
        "cssstyle" => "border-width:3pt;padding-left:10pt"
      );
      if !framecolor.is_empty() { attr.insert(s!("framecolor"), framecolor); }
      if let Some(Stored::Digested(body)) = props.get("body") {
        insert_block(document, body, attr)?;
      }
      Ok(())
    },
    properties => sub[_args] {
      Ok(stored_map!("framecolor" => color::BLACK.to_attribute()))
    }
  );

  // {titled-frame}
  // Perl: framed.sty.ltxml lines 105-118
  DefEnvironment!("{titled-frame} Undigested",
    sub[document, _args, props] {
      document.maybe_close_element("ltx:p")?;
      let framecolor = props.get("framecolor").map(|v| v.to_string()).unwrap_or_default();
      let bg = props.get("backgroundcolor").map(|v| v.to_string()).unwrap_or_default();
      let mut attr = string_map!(
        "framed" => "rectangle",
        "cssstyle" => "padding:8pt;border-width:2pt"
      );
      if !framecolor.is_empty() { attr.insert(s!("framecolor"), framecolor); }
      if !bg.is_empty() { attr.insert(s!("backgroundcolor"), bg); }
      if let Some(Stored::Digested(body)) = props.get("body") {
        insert_block(document, body, attr)?;
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
        gullet::unread(Tokens::new(inv));
      }
    },
    properties => sub[_args] {
      let framecolor = lookup_color_hex("TFFrameColor");
      let bg = current_background_hex();
      Ok(stored_map!("framecolor" => framecolor, "backgroundcolor" => bg))
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
