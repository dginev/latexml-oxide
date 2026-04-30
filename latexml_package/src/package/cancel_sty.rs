use crate::prelude::*;

/// Perl: cancelColorProperties — captures font state for cancel color styling.
/// Note: In Perl, Digest(T_CS('\CancelColor')) always returns a truthy object
/// (even for empty macros), so forcefont/cancelfont are ALWAYS set.
fn cancel_color_properties(whatsit: &mut Whatsit) -> Result<()> {
  let inner_font = lookup_font().unwrap();
  // Always set forcefont — Perl always sets this (Digest returns truthy object)
  whatsit.set_property("forcefont", Stored::String(arena::pin("1")));

  // Digest \CancelColor in a temp group to get the cancel font
  bgroup();
  digest(Tokens!(T_CS!("\\CancelColor")))?;
  let cancel_font = lookup_font().unwrap();

  // Set cancel color if CancelColor changed the color
  if cancel_font.color != inner_font.color {
    if let Some(ref cancel_color) = cancel_font.color {
      whatsit.set_property(
        "cancelcolor",
        Stored::String(arena::pin(cancel_color.to_attribute())),
      );
    }
    // Set inner color for text mode (so content stays in original color)
    // None color = inherited default (DEFCOLOR = black)
    let inner_color = inner_font
      .color
      .unwrap_or(latexml_core::common::color::BLACK);
    whatsit.set_property(
      "innercolor",
      Stored::String(arena::pin(inner_color.to_attribute())),
    );
  }
  egroup()?;
  Ok(())
}

LoadDefinitions!({
  // Ignorable options
  for option in &[
    "samesize",
    "smaller",
    "Smaller",
    "makeroom",
    "overlap",
    "thicklines",
  ] {
    DeclareOption!(option, None);
  }

  DefMacro!("\\CancelColor", None);

  // Basic macros — dispatch to math or text mode
  DefMacro!(
    "\\cancel{}",
    "\\ifmmode\\@@math@cancel{#1}\\else\\@@text@cancel{#1}\\fi"
  );
  DefMacro!(
    "\\bcancel{}",
    "\\ifmmode\\@@math@bcancel{#1}\\else\\@@text@bcancel{#1}\\fi"
  );
  DefMacro!(
    "\\xcancel{}",
    "\\ifmmode\\@@math@xcancel{#1}\\else\\@@text@xcancel{#1}\\fi"
  );

  // Math mode constructors
  // _force_font triggers finalize_rec font computation on the empty XMTok,
  // producing font="italic" from the math font's relative_to diff.
  // ?#cancelcolor conditionally adds color from CancelColor.
  DefConstructor!("\\@@math@cancel{}",
  "<ltx:XMApp>\
    <ltx:XMTok role='ENCLOSE' enclose='updiagonalstrike' meaning='cancel'\
      _force_font='#forcefont' ?#cancelcolor(color='#cancelcolor')()/>\
    <ltx:XMWrap>#1</ltx:XMWrap>\
  </ltx:XMApp>",
  alias => "\\cancel",
  after_digest => sub[whatsit] {
    cancel_color_properties(whatsit)?;
  });

  DefConstructor!("\\@@math@bcancel{}",
  "<ltx:XMApp>\
    <ltx:XMTok role='ENCLOSE' enclose='downdiagonalstrike' meaning='cancel'\
      _force_font='#forcefont' ?#cancelcolor(color='#cancelcolor')()/>\
    <ltx:XMWrap>#1</ltx:XMWrap>\
  </ltx:XMApp>",
  alias => "\\bcancel",
  after_digest => sub[whatsit] {
    cancel_color_properties(whatsit)?;
  });

  DefConstructor!("\\@@math@xcancel{}",
  "<ltx:XMApp>\
    <ltx:XMTok role='ENCLOSE' enclose='updiagonalstrike downdiagonalstrike' meaning='cancel'\
      _force_font='#forcefont' ?#cancelcolor(color='#cancelcolor')()/>\
    <ltx:XMWrap>#1</ltx:XMWrap>\
  </ltx:XMApp>",
  alias => "\\xcancel",
  after_digest => sub[whatsit] {
    cancel_color_properties(whatsit)?;
  });

  // \cancelto{value}{expression} — math mode only
  DefConstructor!("\\cancelto{}{}",
  "<ltx:XMApp>\
    <ltx:XMTok role='SUPERSCRIPTOP'/>\
    <ltx:XMApp>\
      <ltx:XMTok role='ENCLOSE' enclose='updiagonalstrike updiagonalarrow' meaning='cancel'\
        _force_font='#forcefont' ?#cancelcolor(color='#cancelcolor')()/>\
      <ltx:XMWrap>#2</ltx:XMWrap>\
    </ltx:XMApp>\
    <ltx:XMWrap>#1</ltx:XMWrap>\
  </ltx:XMApp>",
  after_digest => sub[whatsit] {
    cancel_color_properties(whatsit)?;
  });

  // Text mode constructors. Perl cancel.sty.ltxml uses
  //   mode => 'restricted_horizontal', enterHorizontal => 1
  // on each — `enter_horizontal=>true` triggers an implicit \indent /
  // paragraph start when `\cancel{text}` appears in vertical mode
  // (e.g. between paragraphs at top level). Without it, the
  // <ltx:del> opens before the enclosing <ltx:p>, producing invalid
  // structure (vertical-mode block-level del with no paragraph parent).
  DefConstructor!("\\@@text@cancel{}",
  "<ltx:del class='downdiagonalstrike' ?#cancelcolor(color='#cancelcolor')()>\
    <ltx:text _noautoclose='1' _force_font='#forcefont' ?#innercolor(color='#innercolor')()>#1</ltx:text>\
  </ltx:del>",
  alias => "\\cancel",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    cancel_color_properties(whatsit)?;
  });

  DefConstructor!("\\@@text@bcancel{}",
  "<ltx:del class='updiagonalstrike' ?#cancelcolor(color='#cancelcolor')()>\
    <ltx:text _noautoclose='1' _force_font='#forcefont' ?#innercolor(color='#innercolor')()>#1</ltx:text>\
  </ltx:del>",
  alias => "\\bcancel",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    cancel_color_properties(whatsit)?;
  });

  DefConstructor!("\\@@text@xcancel{}",
  "<ltx:del class='downdiagonalstrike updiagonalstrike' ?#cancelcolor(color='#cancelcolor')()>\
    <ltx:text _noautoclose='1' _force_font='#forcefont' ?#innercolor(color='#innercolor')()>#1</ltx:text>\
  </ltx:del>",
  alias => "\\xcancel",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    cancel_color_properties(whatsit)?;
  });

  ProcessOptions!();
});
