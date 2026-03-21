use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("color");
  RequirePackage!("array");

  // DefConditional('\if@@rowcolored', sub { LookupValue('tabular_row_color'); });
  // Can't use DefConditional! because compile-time tokenizer splits \if@@rowcolored
  // into \if + @@ + rowcolored (@ is "other" in proc macro context).
  // Use a runtime primitive that the RawTeX macros can reference.
  DefConditional!("\\iflx@rowcolored", {
    state::lookup_value("tabular_row_color").is_some_and(|v| !matches!(v, Stored::None))
  });
  // Alias the @ version at runtime
  RawTeX!(r"\let\if@@rowcolored\iflx@rowcolored");

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

  // \columncolor, \cellcolor, \rowcolor — set background color.
  // Must use RawTeX! because the compile-time proc macro tokenizer treats @ as "other",
  // so DefMacro! expansion strings containing \@setcellcolor produce two tokens
  // (\@ + setcellcolor) instead of one CS (\@setcellcolor).
  // RawTeX! tokenizes at package loading time when @ has catcode "letter".
  RawTeX!(r"\def\columncolor{\@ifnextchar[\lx@columncolor@ii{\lx@columncolor@ii[]}}");
  RawTeX!(r"\long\def\lx@columncolor@ii[#1]#2{%
    \if@@rowcolored\else
      \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
      \@setcellcolor
    \fi}");

  RawTeX!(r"\def\cellcolor{\@ifnextchar[\lx@cellcolor@ii{\lx@cellcolor@ii[]}}");
  RawTeX!(r"\long\def\lx@cellcolor@ii[#1]#2{%
    \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
    \@setcellcolor}");

  RawTeX!(r"\def\rowcolor{\@ifnextchar[\lx@rowcolor@ii{\lx@rowcolor@ii[]}}");
  RawTeX!(r"\long\def\lx@rowcolor@ii[#1]#2{%
    \lx@hidden@noalign{%
      \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
      \@setrowcolor}}");

  // \@setrowcolor — store row background color during digestion.
  // Sets tabular_row_color state value and stores bg on alignment row properties.
  // The backgroundcolor is applied to <tr> during row absorption.
  DefPrimitive!("\\@setrowcolor", sub[_args] {
    if let Some(font) = lookup_font() {
      if let Some(bg) = font.get_background() {
        let bg_str = bg.to_attribute();
        state::assign_value(
          "tabular_row_color",
          Stored::String(arena::pin(&bg_str)),
          Some(Scope::Global),
        );
      }
    }
    Ok(())
  });

  // \@setcellcolor — store cell background color during digestion.
  // Uses DefPrimitive to capture background at digestion time (when font is set).
  // The backgroundcolor is stored on the alignment cell and applied to <td> during absorption.
  DefPrimitive!("\\@setcellcolor", sub[_args] {
    if let Some(font) = lookup_font() {
      if let Some(bg) = font.get_background() {
        let bg_str = bg.to_attribute();
        if let Some(alignment) = lookup_alignment() {
          if let Some(data) = alignment.alignment_cell() {
            let mut data_lock = data.borrow_mut();
            if let Some(colspec) = data_lock.current_column() {
              colspec.backgroundcolor = Some(bg_str);
            }
          }
        }
      }
    }
    Ok(())
  });

  // \arrayrulecolor, \doublerulesepcolor — ignore
  DefMacro!("\\arrayrulecolor[]{}", None);
  DefMacro!("\\doublerulesepcolor[]{}", None);

  // \minrowclearance
  DefRegister!("\\minrowclearance", Dimension::new(0));
});
