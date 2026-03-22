use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("color");
  RequirePackage!("array");

  // DefConditional('\if@@rowcolored', sub { LookupValue('tabular_row_color'); });
  // Can't use DefConditional! because compile-time tokenizer splits \if@@rowcolored
  // into \if + @@ + rowcolored (@ is "other" in proc macro context).
  // Use a name without @ that the compile-time tokenizer handles correctly.
  DefConditional!("\\iflxrowcolored", {
    state::lookup_value("tabular_row_color").is_some_and(|v| !matches!(v, Stored::None))
  });
  // Alias the @ version at runtime (@ is letter during package loading)
  RawTeX!(r"\let\if@@rowcolored\iflxrowcolored");

  // DefPrimitive('\@clearrowcolor', sub {
  //   MergeFont(background => undef);
  //   AssignValue(tabular_row_color => undef, 'global'); });
  DefPrimitive!("\\lxclearrowcolor", sub [_args] {
    merge_font(Font { bg: None, ..Font::default() });
    state::assign_value("tabular_row_color", Stored::None, Some(Scope::Global));
  });
  RawTeX!(r"\let\@clearrowcolor\lxclearrowcolor");

  // AddToMacro('\@tabular@row@after', '\lx@hidden@noalign{\@clearrowcolor}');
  RawTeX!(r"\expandafter\def\expandafter\@tabular@row@after\expandafter{\@tabular@row@after\lx@hidden@noalign{\@clearrowcolor}}");

  // AddToMacro('\@tabular@column@before', '\@userowcolor');
  // Use RawTeX because AddToMacro! compile-time tokenizer splits \@userowcolor
  RawTeX!(r"\expandafter\def\expandafter\@tabular@column@before\expandafter{\@tabular@column@before\@userowcolor}");

  // DefPrimitive('\@userowcolor', sub {
  //   if (my $rc = LookupValue('tabular_row_color')) {
  //     MergeFont(background => $rc); } });
  // Use name without @ for compile-time tokenization, then alias at runtime
  // Perl: only calls MergeFont(background => $rc). Does NOT set cell backgroundcolor.
  // The font background propagates to <text backgroundcolor="..."> wrappers.
  // The <tr> backgroundcolor is set by \@setrowcolor's constructor body.
  DefPrimitive!("\\lxuserowcolor", sub [_args] {
    if let Some(Stored::String(sym)) = state::lookup_value("tabular_row_color") {
      let color_str = arena::with(sym, |s| s.to_string());
      if let Some(c) = latexml_core::common::color::Color::from_stored(&color_str) {
        merge_font(fontmap!(bg => c));
      }
    }
  });
  RawTeX!(r"\let\@userowcolor\lxuserowcolor");

  // \columncolor, \cellcolor, \rowcolor — set background color.
  // Must use RawTeX! because the compile-time proc macro tokenizer treats @ as "other",
  // so DefMacro! expansion strings containing \@setcellcolor produce two tokens
  // (\@ + setcellcolor) instead of one CS (\@setcellcolor).
  // RawTeX! tokenizes at package loading time when @ has catcode "letter".
  // \columncolor[model]{color}[left_overhang][right_overhang]
  // The overhang args are layout-only (ignored by LaTeXML) but must be consumed.
  RawTeX!(r"\def\columncolor{\@ifnextchar[\lx@columncolor@ii{\lx@columncolor@ii[]}}");
  RawTeX!(r"\long\def\lx@columncolor@ii[#1]#2{%
    \if@@rowcolored\else
      \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
      \@setcellcolor
    \fi
    \@ifnextchar[{\lx@gobble@optopt}{}%
  }");
  // Consume up to two optional arguments (overhang params)
  RawTeX!(r"\def\lx@gobble@optopt[#1]{\@ifnextchar[{\lx@gobble@opt}{}}");
  RawTeX!(r"\def\lx@gobble@opt[#1]{}");

  RawTeX!(r"\def\cellcolor{\@ifnextchar[\lx@cellcolor@ii{\lx@cellcolor@ii[]}}");
  RawTeX!(r"\long\def\lx@cellcolor@ii[#1]#2{%
    \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
    \@setcellcolor}");

  RawTeX!(r"\def\rowcolor{\@ifnextchar[\lx@rowcolor@ii{\lx@rowcolor@ii[]}}");
  RawTeX!(r"\long\def\lx@rowcolor@ii[#1]#2{%
    \lx@hidden@noalign{%
      \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
      \@setrowcolor}}");

  // \@setrowcolor — Perl: DefConstructor('\@setrowcolor', sub { ... },
  //   afterDigest => sub { ... }, properties => { alignmentSkippable => 1 }, alias => '');
  // During digestion (afterDigest): captures font background, stores tabular_row_color.
  // During absorption (constructor body): walks DOM to find ancestor <tr>, sets backgroundcolor.
  DefConstructor!("\\lxsetrowcolor",
    sub[document, _args, props] {
      if let Some(Stored::String(bg_sym)) = props.get("background") {
        let bg_str = arena::with(*bg_sym, |s| s.to_string());
        let current = document.get_node().clone();
        if let Some(mut tr_node) = document.findnode("ancestor-or-self::ltx:tr", Some(&current)) {
          if !tr_node.has_attribute("backgroundcolor") {
            document.set_attribute(&mut tr_node, "backgroundcolor", &bg_str)?;
          }
        }
      }
    },
    after_digest => sub[whatsit] {
      if let Some(font) = lookup_font() {
        if let Some(bg) = font.get_background() {
          // Store hex format for the constructor body (DOM attribute value)
          let bg_hex = bg.to_attribute();
          whatsit.set_property("background", Stored::String(arena::pin(&bg_hex)));
          // Store "model c1 c2 ..." format for tabular_row_color state
          // (used by \@userowcolor via Color::from_stored)
          let bg_stored = bg.to_stored();
          state::assign_value(
            "tabular_row_color",
            Stored::String(arena::pin(&bg_stored)),
            Some(Scope::Global),
          );
        }
      }
      Ok(Vec::new())
    },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) },
    alias => "");
  RawTeX!(r"\let\@setrowcolor\lxsetrowcolor");

  // \@setcellcolor — Perl: DefConstructor('\@setcellcolor', sub { ... },
  //   properties => { alignmentSkippable => 1 }, alias => '');
  // During absorption: walks DOM to find ancestor <td>, sets backgroundcolor from font.
  DefConstructor!("\\lxsetcellcolor",
    sub[document, _args, props] {
      if let Some(Stored::String(bg_sym)) = props.get("background") {
        let bg_str = arena::with(*bg_sym, |s| s.to_string());
        let current = document.get_node().clone();
        if let Some(mut td_node) = document.findnode("ancestor-or-self::ltx:td", Some(&current)) {
          if !bg_str.is_empty() {
            document.set_attribute(&mut td_node, "backgroundcolor", &bg_str)?;
          }
        }
      }
    },
    after_digest => sub[whatsit] {
      // Capture font background during digestion for use in constructor body
      if let Some(font) = lookup_font() {
        if let Some(bg) = font.get_background() {
          let bg_hex = bg.to_attribute();
          whatsit.set_property("background", Stored::String(arena::pin(&bg_hex)));
        }
      }
      Ok(Vec::new())
    },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) },
    alias => "");
  RawTeX!(r"\let\@setcellcolor\lxsetcellcolor");

  // \arrayrulecolor, \doublerulesepcolor — ignore
  DefMacro!("\\arrayrulecolor[]{}", None);
  DefMacro!("\\doublerulesepcolor[]{}", None);

  // \minrowclearance
  DefRegister!("\\minrowclearance", Dimension::new(0));
});
