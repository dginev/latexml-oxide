use crate::prelude::*;
use latexml_core::document::get_node_qname;

/// Perl: units_assert_units — walks DOM and sets role=ID, class=ltx_unit on UNKNOWN XMToks
fn units_assert_units(document: &mut Document, node: &Node) -> Result<()> {
  let qname = get_node_qname(node);
  let tag = arena::to_string(qname);
  if tag == "ltx:XMTok" {
    let role = node.get_attribute("role").unwrap_or_default();
    if role == "UNKNOWN" || role.is_empty() {
      let mut n = node.clone();
      n.set_attribute("role", "ID")?;
      document.add_class(&mut n, "ltx_unit")?;
    }
  } else if !tag.is_empty() {
    for child in document.findnodes("*", Some(node)) {
      units_assert_units(document, &child)?;
    }
  }
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: units.sty.ltxml
  RequirePackage!("nicefrac");

  // Helper for text mode content in units
  DefConstructor!("\\helper@ams@text {}",
    "<ltx:text _noautoclean='1' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal",
    reversion => "#1"
  );

  DefMacro!("\\unit Optional {}",
    "\\ifx.#1.\\else#1\\ltx@units@spacing\\fi\\lx@units@assertunits{\\ifmmode\\mathrm{#2}\\else #2\\fi}");

  DefMacro!("\\unitfrac Optional {}{}",
    "\\ifx.#1.\\else#1\\ltx@units@spacing\\fi\\lx@units@assertunits{\\ifmmode\\nicefrac[\\mathrm]{#2}{#3}\\else\\nicefrac{#2}{#3}\\fi}");

  DefConstructor!("\\lx@units@assertunits {}",
    "#1",
    after_construct => sub[document] {
      let node = document.get_node().clone();
      units_assert_units(document, &node)?;
    },
    reversion => "#1"
  );

  // Default: tight spacing (\,)
  Let!("\\ltx@units@spacing", "\\,");

  // Perl units.sty.ltxml L55-58: `tight` / `loose` package options
  // toggle the spacing between value and unit (\, vs ~). Rust had the
  // default Let above but neither option nor the ExecuteOptions('tight')
  // call, so `\usepackage[loose]{units}` silently fell through to tight
  // spacing. Pure additive; no units test exercises these options.
  DeclareOption!("tight", { Let!("\\ltx@units@spacing", "\\,"); });
  DeclareOption!("loose", { Let!("\\ltx@units@spacing", "~");   });
  Digest!("\\ExecuteOptions{tight}")?;
  ProcessOptions!();
});
