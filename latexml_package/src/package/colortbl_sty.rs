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
    lookup_value("tabular_row_color").is_some()
  });
  RawTeX!(r"\let\if@@rowcolored\iflxrowcolored");

  // Perl L35-37:
  //   DefPrimitive('\@clearrowcolor', sub {
  //     MergeFont(background => undef);
  //     AssignValue(tabular_row_color => undef, 'global'); });
  DefPrimitive!("\\lxclearrowcolor", sub [_args] {
    merge_font(Font { bg: None, ..Font::default() });
    assign_value("tabular_row_color", Stored::None, Some(Scope::Global));
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
    if let Some(Stored::Font(rc_font)) = lookup_value("tabular_row_color")
      && let Some(bg) = rc_font.get_background().copied() {
        merge_font(fontmap!(bg => bg));
      }
  });
  RawTeX!(r"\let\@userowcolor\lxuserowcolor");

  // \columncolor, \cellcolor, \rowcolor — set background color.
  // Must use RawTeX! because the compile-time proc macro tokenizer treats @ as "other",
  // so DefMacro! expansion strings containing \@setcellcolor produce two tokens
  // (\@ + setcellcolor) instead of one CS (\@setcellcolor).
  // RawTeX! tokenizes at package loading time when @ has catcode "letter".
  // \columncolor[model]{color}[left_overhang][right_overhang] — Perl
  // colortbl.sty.ltxml L50 `DefMacro('\columncolor[]{}[][]', …)`. The two trailing
  // overhang args are layout-only (ignored) but must be consumed. Read them with
  // LaTeXML's optional-arg reader (the `[]{}[][]` signature), NOT TeX's
  // `\@ifnextchar[` + a delimited `[#1]`: `\@ifnextchar[` matched a `\lbrack` (then a CS
  // `\let` to `[`; a macro since batch 66d, as latex.ltx:562 has it, so no `\ifx` match now) via `\ifx`, but the
  // delimited reader then ran forward hunting a
  // LITERAL `[`, swallowing the row's `\\` and cascading the p/m-column mode
  // (`\@end@tabular … internal_vertical`, document + bibliography lost). LaTeXML's
  // `[]` reader matches only a literal `[` char, so a cell that starts with
  // `\lbrack…\rbrack` stays content, as pdflatex renders it. Rust-only; witness
  // arXiv:2606.02077 (booktabs `\toprule[1pt]` + `\lbrack` in a
  // `>{\columncolor{…}}m{…}` column). `\@setcellcolor` carries `@`, which the
  // compile-time DefMacro! tokenizer would split, so a RawTeX helper (where `@` is a
  // letter) holds the body and the DefMacro only supplies the optional signature.
  RawTeX!(
    r"\long\def\lxcolumncolorbody#1#2{%
      \if@@rowcolored\else\ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi\@setcellcolor\fi}"
  );
  DefMacro!("\\columncolor[]{}[][]", "\\lxcolumncolorbody{#1}{#2}");

  RawTeX!(r"\def\cellcolor{\@ifnextchar[\lx@cellcolor@ii{\lx@cellcolor@ii[]}}");
  RawTeX!(
    r"\long\def\lx@cellcolor@ii[#1]#2{%
    \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
    \@setcellcolor}"
  );

  // colortbl.sty:208-231: `\rowcolor[model]{color}` then `\CT@rowc` reads the two overhangs
  // `[left][right]` (right defaults to left); layout only, but unread they became cell text
  // ("[2pt][2pt] c") and, before a `\multicolumn`, a misplaced `\omit` (witnesses 2605.22864,
  // 2605.08915, 2605.04906; Perl colortbl.sty.ltxml:55 `\rowcolor[]{}` shares it). As
  // `\columncolor`, a RawTeX body with a DefMacro supplying the signature.
  RawTeX!(
    r"\long\def\lx@rowcolor@body#1#2{%
    \lx@hidden@noalign{%
      \ifx.#1.\pagecolor{#2}\else\pagecolor[#1]{#2}\fi
      \@setrowcolor}}"
  );
  DefMacro!("\\rowcolor[]{}[][]", "\\lx@rowcolor@body{#1}{#2}");

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
        let bg_str = with(*bg_sym, |s| s.to_string());
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
      if let Some(font) = lookup_font()
        && let Some(bg) = font.get_background() {
          // Constructor body needs a hex string (DOM attribute value)
          let bg_hex = bg.to_attribute();
          whatsit.set_property("background", Stored::String(pin(&bg_hex)));
          // Perl stores the Color object in tabular_row_color; Rust stashes it
          // as a Font carrying only the bg slot so \@userowcolor can merge it.
          let bg_font = Font { bg: Some(*bg), ..Font::default() };
          assign_value(
            "tabular_row_color",
            Stored::Font(Rc::new(bg_font)),
            Some(Scope::Global),
          );
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
      if let Some(mut td_node) = document.findnode("ancestor-or-self::ltx:td", Some(&current))
        && let Some(Stored::Font(font)) = props.get("font")
          && let Some(bg) = font.get_background() {
            let bg_hex = bg.to_attribute();
            document.set_attribute(&mut td_node, "backgroundcolor", &bg_hex)?;
          }
    },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) },
    alias => "");
  RawTeX!(r"\let\@setcellcolor\lxsetcellcolor");

  // colortbl.sty's `\CT@*` internal surface, reached by raw derivatives that
  // require colortbl (tabu.sty:720 `\CT@everycr\expandafter{…\the\CT@everycr…}`,
  // tabulary.sty `\CT@arc@`/`\CT@color`/`\CT@column@color`/`\CT@row@color`/
  // `\CT@cell@color`/`\CT@do@color`/`\CT@setup`/`\CT@start`/`\CT@extract`,
  // tabularht/keyvaltable `\CT@arc@`): the binding stands in for colortbl.sty
  // (Perl colortbl.sty.ltxml omits them too). `\CT@everycr` MUST be the
  // `\everycr` toks register (colortbl.sty:116 `\let\CT@everycr\everycr` —
  // tabu assigns to it and `\the`s it); the cell colour painters are unrendered
  // (:75-166 `\let…\relax`/`\@empty`), the rule colours `\CT@arc@`/`\CT@drsc@` as colortbl's (below). Witness
  // srdp-mathematik (`\multiplechoice` → raw tabu). Guard:
  // `perfect_kernel_batch54::colortbl_internal_surface_is_defined`.
  Let!("\\CT@everycr", "\\everycr");
  RawTeX!(
    r"\let\CT@arc@\relax \let\CT@drsc@\relax \let\CT@do@color\relax
\let\CT@@do@color\relax \let\CT@column@color\@empty \let\CT@row@color\relax
\let\CT@cell@color\relax \let\CT@color\relax \let\CT@setup\relax
\let\CT@LT@sep\relax
\def\CT@extract#1#2#3#4{}"
  );
  // colortbl.sty:119-135: `\CT@start` saves the rule colours as a `\@tabarray` table (our `\@tabular@bindings`,
  // `\@array@bindings`) starts, `\CT@end` restores them after the alignment closes, its last row included; a longtable
  // or IEEEeqnarraybox keeps the change, as in pdflatex. The request is local to the table's group. OD #1025.
  RawTeX!(
    r"\def\CT@start{\let\CT@arc@save\CT@arc@ \let\CT@drsc@save\CT@drsc@}
\def\CT@end{\global\let\CT@arc@\CT@arc@save \global\let\CT@drsc@\CT@drsc@save}
\def\lx@CT@begin@plain{\let\lx@CT@restore\@empty}
\def\lx@CT@begin@save{\CT@start\let\lx@CT@restore\CT@end\let\lx@CT@begin\lx@CT@begin@plain}
\let\lx@CT@begin\lx@CT@begin@plain \let\lx@CT@restore\@empty
\def\lx@CT@tabarray{\let\lx@CT@begin\lx@CT@begin@save}
\let\lx@CT@tabular@bindings\@tabular@bindings \def\@tabular@bindings{\lx@CT@tabarray\lx@CT@tabular@bindings}
\let\lx@CT@array@bindings\@array@bindings \def\@array@bindings{\lx@CT@tabarray\lx@CT@array@bindings}
\let\lx@CT@begin@alignment\lx@begin@alignment \def\lx@begin@alignment{\lx@CT@begin\lx@CT@begin@alignment}
\let\lx@CT@end@alignment\lx@end@alignment \def\lx@end@alignment{\lx@CT@end@alignment\lx@CT@restore}"
  );
  // colortbl.sty:156-164: `\arrayrulecolor` stores `\color[model]{colour}` in `\CT@arc@`, evaluated as each rule is
  // drawn; its argument runs to the first `{` and takes that group, so `\arrayrulecolor\hhline{---~~~}` takes the
  // `\hhline` and its spec, as pdflatex does (1910.02206; Perl's `[]{}` left the spec a cell, KNOWN_PERL_ERRORS #889;
  // guard `perfect_kernel_batch69a::arrayrulecolor_reads_to_the_brace`). Between rows it is `\noalign` material.
  // `\doublerulesepcolor` is stored alike, unrendered (a border has no colour for the gap of `||`).
  RawTeX!(
    r"\def\arrayrulecolor#1#{\CT@arc{#1}}\def\CT@arc#1#2{\lx@hidden@noalign{\gdef\CT@arc@{\color#1{#2}}}}
\def\doublerulesepcolor#1#{\CT@drs{#1}}\def\CT@drs#1#2{\lx@hidden@noalign{\gdef\CT@drsc@{\color#1{#2}}}}"
  );
  // colortbl.sty:166-192 draws `\hline` and `\cline` as `{\CT@arc@ <rule>}`: the colour `\CT@arc@` gives is the
  // rule's border colour (`alignment::RULE_COLOR`, local to the rule's `\noalign`), the default black none.
  // Beyond Perl (a no-op, colortbl.sty.ltxml:85): OD #1025 (#773).
  DefPrimitive!("\\lx@CT@arc@draw", {
    if let Some(color) = arc_color(&T_CS!("\\CT@arc@"))? {
      assign_value(
        alignment::RULE_COLOR,
        Stored::String(pin(&color)),
        Some(Scope::Local),
      );
    }
  });
  RawTeX!(
    r"\let\lx@CT@hline\@@alignment@hline \def\@@alignment@hline{\lx@CT@arc@draw\lx@CT@hline}
\let\lx@CT@cline\lx@cline \def\lx@cline{\lx@CT@arc@draw\lx@CT@cline}"
  );
  // colortbl.sty:166 `\def\@arrayrule{\@addtopreamble{{\CT@arc@\vline}}}`: only a `|` takes the rule colour, `\xdef`ed
  // into the preamble (array.sty:54) as it is built, a `\multicolumn`'s too; an unset `\CT@arc@` (`\relax`) stays,
  // read as the rule is drawn. A `!{\vrule}` or a typed `\vline` keeps the default ink.
  DefColumnType!("|", {
    let arc = T_CS!("\\CT@arc@");
    let rule = if arc.defined_as(&T_RELAX!()) {
      vec![T_CS!("\\vrule"), T_CS!("\\lx@CT@paint@live")]
    } else {
      vec![T_CS!("\\vrule"), T_CS!("\\lx@CT@paint"), frozen_arc(&arc)]
    };
    with_building_template(|template| template.add_between_column(rule));
  });
  // The `|` rule is the template's own `\vrule`, so a cell's reversion trims it (`tex_tables.rs` `trim_column_template`);
  // the token after it sets its colour as `border_color`, which the cell's border takes (`extract_alignment_column`).
  DefPrimitive!("\\lx@CT@paint Token", sub[(arc)] {
    paint_rule(&arc)
  });
  DefPrimitive!("\\lx@CT@paint@live", sub[_args] {
    paint_rule(&T_CS!("\\CT@arc@"))
  });

  // Perl L93: \minrowclearance
  DefRegister!("\\minrowclearance", Dimension::new(0));
  // colortbl.sty:269-275: the count `\rownum` that `\rowcolors` steps, `\c@rownum` the same register, and `\therownum`
  // (colortbl-DE prints it). The xcolor binding's `\rownum` (Perl xcolor.sty.ltxml:744, the alignment's row number) is
  // no definition of xcolor.sty, so it gives way to the count here: `\c@rownum` is a count whatever loaded first, as
  // tabularray's `\int_zero_new:N \c@rownum` and `\int_set:Nn` need (`\usepackage[table]{xcolor}` before tabularray:
  // 2601.01074, 2409.12172). Under `\rowcolors` the xcolor binding's row hook steps the count (colortbl.sty:261).
  if let (Some(rownum), Some(perl)) = (
    lookup_definition(&T_CS!("\\rownum"))?,
    lookup_definition(&T_CS!("\\lx@xcolor@rownum"))?,
  ) && Rc::ptr_eq(&rownum, &perl)
  {
    RawTeX!(r"\let\rownum\@undefined");
  }
  RawTeX!(
    r"\@ifundefined{rownum}{\@ifundefined{c@rownum}{\newcount\rownum\let\c@rownum\rownum}{\let\rownum\c@rownum}}%
{\let\c@rownum\rownum}
\providecommand\therownum{\arabic{rownum}}"
  );
});

/// The global copy of `arc`'s colour a preamble freezes (array.sty:54 `\xdef`), one per distinct body.
fn frozen_arc(arc: &Token) -> Token {
  let body = match lookup_meaning(arc) {
    Some(Stored::Expandable(defn)) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => Some(body.to_string()),
      _ => None,
    },
    _ => None,
  };
  let key = body.map(|body| s!("lx@CT@arc@frozen@{body}"));
  if let Some(Stored::Token(frozen)) = key.as_deref().and_then(lookup_value) {
    return frozen;
  }
  let n = lookup_int("lx@CT@arc@frozen") + 1;
  assign_value("lx@CT@arc@frozen", n, Some(Scope::Global));
  let frozen = T_CS!(s!("\\lx@CT@arc@{n}"));
  let_i(&frozen, arc, Some(Scope::Global));
  if let Some(key) = key {
    assign_value(&key, Stored::Token(frozen), Some(Scope::Global));
  }
  frozen
}

/// Gives the rule just drawn, the current list's last box, the colour `arc` gives as its `border_color`.
fn paint_rule(arc: &Token) -> Result<Vec<Digested>> {
  if let Some(color) = arc_color(arc)?
    && let Some(mut rule) = pop_box_list()
  {
    if rule.get_property_bool("isVerticalRule") {
      rule.set_property("border_color", Stored::String(pin(&color)));
    }
    push_box_list(rule);
  }
  Ok(Vec::new())
}

/// The colour `arc` gives (colortbl.sty:161-192 `\CT@arc@`, or a preamble's frozen copy); black and `\relax` none.
fn arc_color(arc: &Token) -> Result<Option<String>> {
  if arc.defined_as(&T_RELAX!()) {
    return Ok(None);
  }
  bgroup();
  digest(Tokens!(*arc))?;
  let color = lookup_font()
    .and_then(|font| font.color)
    .map(|color| color.to_attribute())
    .filter(|color| *color != common::color::BLACK.to_attribute());
  egroup()?;
  Ok(color)
}
