use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("color");
  RequirePackage!("array");

  // Perl L34: DefConditional('\if@@rowcolored', sub { LookupValue('tabular_row_color'); });
  // Perl truthiness: undef => false; any defined value (including Color obj) => true.
  // Rust `state::lookup_value` already returns None for both missing keys and
  // `Stored::None` (see state.rs L780-788), so `.is_some()` alone matches Perl.
  //
  // Note: the `\lx*` indirect CS is a tokenizer-workaround, not a semantic divergence.
  // Can't use DefConditional! directly because compile-time tokenizer splits
  // \if@@rowcolored into \if + @@ + rowcolored (@ is "other" in proc macro context).
  // Use a name without @ that the compile-time tokenizer handles correctly;
  // \let the @ version at runtime when @ has catcode letter.
  DefConditional!("\\iflxrowcolored", {
    state::lookup_value("tabular_row_color").is_some()
  });
  RawTeX!(r"\let\if@@rowcolored\iflxrowcolored");

  // Perl L35-37:
  //   DefPrimitive('\@clearrowcolor', sub {
  //     MergeFont(background => undef);
  //     AssignValue(tabular_row_color => undef, 'global'); });
  DefPrimitive!("\\lxclearrowcolor", sub [_args] {
    merge_font(Font { bg: None, ..Font::default() });
    state::assign_value("tabular_row_color", Stored::None, Some(Scope::Global));
  });
  RawTeX!(r"\let\@clearrowcolor\lxclearrowcolor");

  // Perl L38: AddToMacro('\@tabular@row@after', '\lx@hidden@noalign{\@clearrowcolor}');
  {
    let cs = T_CS!("\\@tabular@row@after");
    let tokens = Tokens!(
      T_CS!("\\lx@hidden@noalign"),
      T_BEGIN!(),
      T_CS!("\\@clearrowcolor"),
      T_END!()
    );
    AddToMacro!(cs, tokens);
  }
  // Perl L40: AddToMacro('\@tabular@column@before', '\@userowcolor');
  {
    let cs = T_CS!("\\@tabular@column@before");
    let tokens = Tokens!(T_CS!("\\@userowcolor"));
    AddToMacro!(cs, tokens);
  }

  // Perl L42-44:
  //   DefPrimitive('\@userowcolor', sub {
  //     if (my $rc = LookupValue('tabular_row_color')) {
  //       MergeFont(background => $rc); } });
  // Perl stores the Color object and passes it through unchanged.
  // Rust stores it as `Stored::Font(Rc<Font>)` carrying only the bg slot
  // (Stored has no Color variant). `\@setrowcolor` populates it; we merge
  // its bg into the current font, matching Perl's direct `MergeFont(background => $rc)`.
  DefPrimitive!("\\lxuserowcolor", sub [_args] {
    if let Some(Stored::Font(rc_font)) = state::lookup_value("tabular_row_color") {
      if let Some(bg) = rc_font.get_background().copied() {
        merge_font(fontmap!(bg => bg));
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
  RawTeX!(
    r"\long\def\lx@columncolor@ii[#1]#2{%
    \if@@rowcolored\else
      \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
      \@setcellcolor
    \fi
    \@ifnextchar[{\lx@gobble@optopt}{}%
  }"
  );
  // Consume up to two optional arguments (overhang params)
  RawTeX!(r"\def\lx@gobble@optopt[#1]{\@ifnextchar[{\lx@gobble@opt}{}}");
  RawTeX!(r"\def\lx@gobble@opt[#1]{}");

  RawTeX!(r"\def\cellcolor{\@ifnextchar[\lx@cellcolor@ii{\lx@cellcolor@ii[]}}");
  RawTeX!(
    r"\long\def\lx@cellcolor@ii[#1]#2{%
    \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
    \@setcellcolor}"
  );

  RawTeX!(r"\def\rowcolor{\@ifnextchar[\lx@rowcolor@ii{\lx@rowcolor@ii[]}}");
  RawTeX!(
    r"\long\def\lx@rowcolor@ii[#1]#2{%
    \lx@hidden@noalign{%
      \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
      \@setrowcolor}}"
  );

  // Perl L64-74: \@setrowcolor — DefConstructor with afterDigest.
  //   afterDigest: captures font background, stores tabular_row_color globally.
  //   constructor body: walks DOM to ancestor <tr>, sets backgroundcolor.
  //
  // RUST DIVERGENCE vs Perl L68's `!$node->hasAttribute('backgroundcolor')`
  // guard. First-principles analysis of where the divergence lives:
  //
  //   Perl has TWO constructors that can write <tr>@backgroundcolor:
  //     1. \@setrowcolor — fired by the user's explicit \rowcolor{…} (colortbl.sty.ltxml L64-74).
  //        hasAttribute-guarded.
  //     2. \@tabular@row@before@xcolor — fired by \rowcolors cycling via \@tabular@row@before
  //        (xcolor.sty.ltxml L757-778). Also hasAttribute-guarded.
  //   In Perl's digested token stream, \@setrowcolor for an explicit
  //   \rowcolor appears BEFORE the cycling constructor in the same row's
  //   tokens; at absorb time it writes first into an unset attribute, the
  //   cycling constructor then sees the set attribute and skips. So Perl
  //   gets "explicit \rowcolor wins" via order, not via guard inversion.
  //
  //   Rust's port injects the cycling and explicit-\rowcolor constructors
  //   in the opposite order in the digested stream, so with the guard
  //   intact the cycling color wins and \rowcolor{blue!25} is dropped
  //   (regressed 65_graphics::xcolors_test on row 3: blue!25 → green!25).
  //
  //   The truly faithful fix is to reorder Rust's token injection so the
  //   explicit-\rowcolor constructor precedes cycling's. That's a deeper
  //   tabular-machinery change. Until then, drop the guard here so
  //   explicit \rowcolor always overrides cycling — this matches the
  //   user-observable TeX semantic (explicit beats cycling) even though
  //   the mechanism diverges from Perl's guard+order pattern.
  DefConstructor!("\\lxsetrowcolor",
    sub[document, _args, props] {
      // Perl L66: if (my $bg = $props{background}) { ... }
      if let Some(Stored::String(bg_sym)) = props.get("background") {
        let bg_str = arena::with(*bg_sym, |s| s.to_string());
        let current = document.get_node().clone();
        if let Some(mut tr_node) = document.findnode("ancestor-or-self::ltx:tr", Some(&current)) {
          document.set_attribute(&mut tr_node, "backgroundcolor", &bg_str)?;
        }
      }
    },
    // Perl L70-72:
    //   afterDigest => sub { my $bg = $_[1]->getProperty('font')->getBackground;
    //     $_[1]->setProperty(background => $bg);
    //     AssignValue(tabular_row_color => $bg, 'global'); },
    after_digest => sub[whatsit] {
      if let Some(font) = lookup_font() {
        if let Some(bg) = font.get_background() {
          // Constructor body needs a hex string (DOM attribute value)
          let bg_hex = bg.to_attribute();
          whatsit.set_property("background", Stored::String(arena::pin(&bg_hex)));
          // Perl stores the Color object in tabular_row_color; Rust stashes it
          // as a Font carrying only the bg slot so \@userowcolor can merge it.
          let bg_font = Font { bg: Some(*bg), ..Font::default() };
          state::assign_value(
            "tabular_row_color",
            Stored::Font(Rc::new(bg_font)),
            Some(Scope::Global),
          );
        }
      }
      Ok(Vec::new())
    },
    // Perl L73-74: properties => { alignmentSkippable => 1 }, alias => ''.
    properties => { Ok(stored_map!("alignmentSkippable" => true)) },
    alias => "");
  RawTeX!(r"\let\@setrowcolor\lxsetrowcolor");

  // Perl L77-83: \@setcellcolor — DefConstructor, no afterDigest.
  //   constructor body reads $props{font}->getBackground INLINE:
  //     DefConstructor('\@setcellcolor', sub {
  //       my ($document, %props) = @_;
  //       if (my $node = $document->findnode('ancestor-or-self::ltx:td', ...)) {
  //         if (my $bg = $props{font} && $props{font}->getBackground) {
  //           $document->setAttribute($node, backgroundcolor => $bg); } } },
  //       properties => { alignmentSkippable => 1 }, alias => '');
  // The constructor framework auto-populates props['font'] from the digestion
  // font state (see definition/constructor.rs L264-269), so we can read it inline.
  DefConstructor!("\\lxsetcellcolor",
    sub[document, _args, props] {
      let current = document.get_node().clone();
      if let Some(mut td_node) = document.findnode("ancestor-or-self::ltx:td", Some(&current)) {
        if let Some(Stored::Font(font)) = props.get("font") {
          if let Some(bg) = font.get_background() {
            let bg_hex = bg.to_attribute();
            document.set_attribute(&mut td_node, "backgroundcolor", &bg_hex)?;
          }
        }
      }
    },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) },
    alias => "");
  RawTeX!(r"\let\@setcellcolor\lxsetcellcolor");

  // Perl L85: \arrayrulecolor — ignored.
  DefMacro!("\\arrayrulecolor[]{}", None);
  // Perl L88: \doublerulesepcolor — ignored.
  DefMacro!("\\doublerulesepcolor[]{}", None);

  // Perl L93: \minrowclearance
  DefRegister!("\\minrowclearance", Dimension::new(0));
});
