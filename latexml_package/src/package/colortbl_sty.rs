use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("color");
  RequirePackage!("array");

  // DefConditional('\if@@rowcolored', sub { LookupValue('tabular_row_color'); });
  DefConditional!("\\if@@rowcolored", {
    state::lookup_value("tabular_row_color").is_some_and(|v| !matches!(v, Stored::None))
  });

  // DefPrimitive('\@clearrowcolor', sub {
  //   MergeFont(background => undef);
  //   AssignValue(tabular_row_color => undef, 'global'); });
  DefPrimitive!("\\@clearrowcolor", sub [_args] {
    // Clear background color from font
    // Perl: MergeFont(background => undef) — sets bg to undef in font hash
    merge_font(Font { bg: None, ..Font::default() });
    state::assign_value("tabular_row_color", Stored::None, Some(Scope::Global));
  });

  // AddToMacro('\@tabular@row@after', '\lx@hidden@noalign{\@clearrowcolor}');
  AddToMacro!("\\@tabular@row@after", "\\lx@hidden@noalign{\\@clearrowcolor}");

  // AddToMacro('\@tabular@column@before', '\@userowcolor');
  AddToMacro!("\\@tabular@column@before", "\\@userowcolor");

  // DefPrimitive('\@userowcolor', sub {
  //   if (my $rc = LookupValue('tabular_row_color')) {
  //     MergeFont(background => $rc); } });
  DefPrimitive!("\\@userowcolor", sub [_args] {
    if let Some(Stored::String(sym)) = state::lookup_value("tabular_row_color") {
      let color_str = arena::with(sym, |s| s.to_string());
      if let Some(c) = latexml_core::common::color::Color::from_stored(&color_str) {
        merge_font(fontmap!(bg => c));
      }
    }
  });

  // \columncolor[<model>]{<color>}[<left_overhang>][<right_overhang>]
  DefMacro!("\\columncolor[]{}[][]",
    "\\if@@rowcolored\\else\\ifx.#1.\\pagecolor{#2}\\else\\pagecolor[#1]{#2}\\fi\\@setcellcolor\\fi");

  // \rowcolor[<model>]{<color>}
  DefMacro!("\\rowcolor[]{}",
    "\\lx@hidden@noalign{\\ifx.#1.\\pagecolor{#2}\\else\\pagecolor[#1]{#2}\\fi\\@setrowcolor}");

  // \cellcolor[<model>]{<color>}
  DefMacro!("\\cellcolor[]{}",
    "\\ifx.#1.\\pagecolor{#2}\\else\\pagecolor[#1]{#2}\\fi\\@setcellcolor");

  // \@setrowcolor — set tr's backgroundcolor from current font background
  DefConstructor!(T_CS!("\\@setrowcolor"), None, sub[document, _args, _props] {
    let bg_opt = document.get_node_font(document.get_node()).get_background()
      .map(|c| c.to_attribute());
    if let Some(bg) = bg_opt {
      let node = document.get_node().clone();
      if let Some(mut tr) = document.findnode("ancestor-or-self::ltx:tr", Some(&node)) {
        if !tr.has_attribute("backgroundcolor") {
          document.set_attribute(&mut tr, "backgroundcolor", &bg)?;
        }
      }
    }
  },
    after_digest => sub[whatsit] {
      let bg = state::lookup_font().and_then(|f| f.get_background().map(|c| c.to_attribute()));
      if let Some(bg_str) = bg {
        whatsit.set_property("background", Stored::String(arena::pin(&bg_str)));
        state::assign_value(
          "tabular_row_color",
          Stored::String(arena::pin(&bg_str)),
          Some(Scope::Global),
        );
      }
    },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) },
    alias => ""
  );

  // \@setcellcolor — set td's backgroundcolor from current font background
  DefConstructor!(T_CS!("\\@setcellcolor"), None, sub[document, _args, _props] {
    let bg_opt = document.get_node_font(document.get_node()).get_background()
      .map(|c| c.to_attribute());
    if let Some(bg) = bg_opt {
      let node = document.get_node().clone();
      if let Some(mut td) = document.findnode("ancestor-or-self::ltx:td", Some(&node)) {
        document.set_attribute(&mut td, "backgroundcolor", &bg)?;
      }
    }
  },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) },
    alias => ""
  );

  // \arrayrulecolor, \doublerulesepcolor — ignore
  DefMacro!("\\arrayrulecolor[]{}", None);
  DefMacro!("\\doublerulesepcolor[]{}", None);

  // \minrowclearance
  DefRegister!("\\minrowclearance", Dimension::new(0));
});
