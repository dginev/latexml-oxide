use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: nicefrac.sty.ltxml
  RequirePackage!("ifthen");

  // Handy for cases where macros want to use math, but track the current text font
  DefPrimitive!("\\nf@mathcopytextfont", sub[_args] {
    use latexml_core::binding::content::merge_font;
    use latexml_core::common::font::Font;
    if let Some(Stored::Font(ref f)) = lookup_value("savedfont") {
      merge_font(Font {
        family: f.family.clone(),
        series: f.series.clone(),
        shape: f.shape.clone(),
        ..Font::default()
      });
    }
    Ok(())
  });

  // Inline nicefrac: up-shifted numerator with /
  DefConstructor!("\\ltx@nicefrac@inline InFractionStyle InFractionStyle",
    "<ltx:XMApp>\
       <ltx:XMTok stretchy='true' meaning='divide' role='MULOP'\
         xoffset='-0.1em' width='-0.15em'>/</ltx:XMTok>\
       <ltx:XMArg yoffset='0.3em' rpadding='-0.5em'>#1</ltx:XMArg>\
       <ltx:XMArg>#2</ltx:XMArg>\
     </ltx:XMApp>",
    alias => "\\nicefrac",
    after_construct => sub[document] {
      // Set specialized font on the slash XMTok to prevent italic inheritance.
      // Perl: _font='#slashfont' with font.specialize("/") — returns upright default font.
      let node = document.get_node().clone();
      let slash_toks = document.findnodes("descendant::ltx:XMTok[@meaning='divide']", Some(&node));
      for mut tok in slash_toks {
        let current_font = document.get_node_font(&tok);
        let slash_font = current_font.specialize("/");
        document.set_node_font(&mut tok, &slash_font)?;
      }
    }
  );
  // Bevelled version: MathML mfrac with bevelled='true'
  DefConstructor!("\\ltx@nicefrac@bevelled InFractionStyle InFractionStyle",
    "<ltx:XMApp>\
       <ltx:XMTok meaning='divide' role='FRACOP' mathstyle='#mathstyle' class='ltx_bevelled'/>\
       <ltx:XMArg>#1</ltx:XMArg>\
       <ltx:XMArg>#2</ltx:XMArg>\
     </ltx:XMApp>",
    alias => "\\nicefrac",
    after_digest => sub[whatsit] {
      let style = lookup_font()
        .and_then(|f| f.get_mathstyle().map(|s| s.to_string()))
        .unwrap_or_default();
      if !style.is_empty() {
        whatsit.set_property("mathstyle", style);
      }
      Ok(Vec::new())
    }
  );

  // Note: use math mode, but inherit text font when in text mode
  DefMacro!("\\@UnitsNiceFrac Optional {}{}",
    "\\ifmmode\\ltx@nicefrac@inline{#1{#2}}{#1{#3}}\\else\\if.#1.$\\ltx@nicefrac@inline{\\nf@mathcopytextfont{#2}}{\\nf@mathcopytextfont{#3}}$\\else$\\ltx@nicefrac@inline{#1{#2}}{#1{#3}}$\\fi\\fi");
  DefMacro!("\\@UnitsNiceFrac@bevelled Optional {}{}",
    "\\ifmmode\\ltx@nicefrac@bevelled{#1{#2}}{#1{#3}}\\else\\if.#1.$\\ltx@nicefrac@bevelled{\\nf@mathcopytextfont{#2}}{\\nf@mathcopytextfont{#3}}$\\else$\\ltx@nicefrac@bevelled{#1{#2}}{#1{#3}}$\\fi\\fi");

  Let!("\\@UnitsUglyFrac", "\\@UnitsNiceFrac");

  // Default: nice style
  Let!("\\nicefrac", "\\@UnitsNiceFrac");

  // Perl nicefrac.sty.ltxml L70-73: three options letting the user pick
  // which \nicefrac variant is active. All three were unported, so
  // `\usepackage[bevelled]{nicefrac}` fell through to the default (inline
  // nice fraction) instead of switching to bevelled output. Pure additive
  // — no test exercises these options so zero golden risk.
  DeclareOption!("nice",     { Let!("\\nicefrac", "\\@UnitsNiceFrac"); });
  DeclareOption!("ugly",     { Let!("\\nicefrac", "\\@UnitsUglyFrac"); });
  DeclareOption!("bevelled", { Let!("\\nicefrac", "\\@UnitsNiceFrac@bevelled"); });
  ProcessOptions!();
});
