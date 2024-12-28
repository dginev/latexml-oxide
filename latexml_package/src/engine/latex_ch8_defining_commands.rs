use crate::prelude::*;
LoadDefinitions!({
  //**********************************************************************
  // C.8 Definitions, Numbering and Programming
  //**********************************************************************

  //======================================================================
  // C.8.1 Defining Commands
  //======================================================================

  DefMacro!("\\@tabacckludge {}", "\\csname\\string#1\\endcsname");

  DefPrimitive!("\\newcommand OptionalMatch:* DefToken [Number][]{}",
  sub[(_star,cs_token,nargs,opt,body)] {
    let nargs = nargs.value_of() as usize;
    if !IsDefinable!(&cs_token) {
      if !has_value(&s!("{}:locked", cs_token.to_string())) { // not locked, inform.
        let message = s!("Ignoring redefinition (\\newcommand) of {}", cs_token.stringify());
        Info!("ignore", cs_token, message);
      }
      return Ok(vec![]);
    }
    let macro_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs_token, macro_args, body);
  });

  DefPrimitive!("\\renewcommand OptionalMatch:* DefToken [Number][]{}",
  sub[(_star, cs, nargs_num, opt, body)] {
    let nargs = nargs_num.value_of() as usize;
    let opt = if let Some(ref opt_content) = opt {
      if opt_content.is_empty() { None } else { opt }
    } else { opt };
    let macro_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs, macro_args, body);
  });

  // low-level implementation of both \newcommand and \renewcommand depends on \@argdef
  // and robustness upgrades are often realized via redefining \l@ngrel@x
  //
  // Experiment: use \l@ngrel@x to carry over \protected information from outside, etoolbox-style.
  // DefMacro('\@argdef','\l@ngrel@x\renewcommand');
  //
  // The etoolbox binding now defines \newrobustcmd & friends directly, so \@argdef is not directly
  // needed However, we would need to add support for other packages that may leverage that
  // machinery.

  DefPrimitive!("\\providecommand OptionalMatch:* DefToken [Number][]{}",
  sub[(_star, cs, nargs, opt, body)] {
    // TODO: Consider if we should just treat the empty tokens directly in convert_latex_args ?
    let opt_checked = if let Some(ref opt_content) = opt {
      if opt_content.is_empty() {
        None
      } else { opt }
    } else { opt };
    if IsDefinable!(&cs) {
      let nargs = nargs.value_of() as usize;
      let cs_args = convert_latex_args(nargs, opt_checked)?;
      DefMacro!(cs, cs_args, body);
    }
  });

  // Crazy; define \cs in terms of \cs[space] !!!
  DefPrimitive!("\\DeclareRobustCommand OptionalMatch:* SkipSpaces DefToken [Number][]{}",
  sub[(_star,cs,nargs,opt,body)] {
    let opt_checked = match opt {
      Some(opt_content) if !opt_content.is_empty() => Some(opt_content),
      _ => None
    };
    let nargs = nargs.value_of() as usize;
    let cs_args = convert_latex_args(nargs, opt_checked)?;
    DefMacro!(cs, cs_args, body, robust => true);
  });

  DefPrimitive!("\\MakeRobust DefToken", sub[(cs)] {
    let mungedcs = T_CS!(cs.with_str(|cstr| s!("{cstr} ")));
    // only if defined but not yet robust
    if LookupDefinition!(&cs).is_some() &&
       LookupDefinition!(&mungedcs).is_none() {
      Let!(&mungedcs, &cs);
      DefMacro!(cs, None, Tokens!(T_CS!("\\protect"),mungedcs));
    }
  });

  //------------------------------------------------------------
  // The following commands define encoding-specific expansions
  // or glyphs.  The control-sequence is defined to use the expansion for
  // the current encoding, if any, or the default expansion (for encoding "?").
  // We don't want to redefine control-sequence if it already has a definition:
  // It may be that we've already defined it to expand into the above conditional.
  // But more importantly, we don't want to override a hand-written definition (if any).
  //------------------------------------------------------------
  DefPrimitive!("\\DeclareTextCommand DefToken {}[Number][]{}",
  sub[(cs, encoding, nargs, opts, expansion)] {
    let cs_str = cs.to_string();
    let nargs = nargs.value_of() as usize;

    let encoding = Expand!(encoding);
    if !IsDefined!(&cs) {    // If not already defined...
      DefMacro!(cs, None, Some(s!(r#"""
      \expandafter\ifx\csname\cf@encoding\string{}\endcsname\relax\csname?\string{}\endcsname\else
      \csname\cf@encoding\string{}\endcsname\fi
      """#, cs_str, cs_str, cs_str).into()));
    }
     let ecs = T_CS!(s!("\\{}{}", encoding, cs_str));
     let ecs_args = convert_latex_args(nargs, opts)?;
     DefMacro!(ecs, ecs_args, expansion);
  });

  DefMacro!(
    "\\DeclareTextCommandDefault DefToken",
    "\\DeclareTextCommand{#1}{?}"
  );

  DefPrimitive!("\\ProvideTextCommand DefToken {}[Number][]{}",
  sub[(cs, encoding, nargs, opts, expansion)] {
    let cs_str = cs.to_string();
    let nargs = nargs.value_of() as usize;
    if IsDefinable!(&cs) { // If not already defined...
      DefMacro!(cs, None, Some(s!(
        r"\expandafter\ifx\csname\cf@encoding\string{cs_str}\endcsname\relax\csname?\string{cs_str}\endcsname
        \else\csname\cf@encoding\string{cs_str}\endcsname\fi").into()));
    }
    let ecs = T_CS!(s!("\\{}{}", encoding, cs_str));
    if !IsDefined!(&ecs) { // If not already defined...
      let ecs_args = convert_latex_args(nargs, opts)?;
      DefMacro!(ecs, ecs_args, expansion);
    }
  });

  DefMacro!(
    "\\ProvideTextCommandDefault DefToken",
    "\\ProvideTextCommand{#1}{?}"
  );

  // #------------------------------------------------------------

  DefPrimitive!("\\DeclareTextSymbol DefToken {}{Number}", sub[(cs, encoding, code)] {
    let code_value = code.value_of() as u8;
    let cs_str = cs.to_string();
    let ecs = T_CS!(s!("\\{encoding}{cs_str}"));
    let encoding_str = Expand!(encoding).to_string();
    if IsDefinable!(&cs) {    // If not already defined...
      DefMacro!(cs, None, Some(s!(r"\expandafter\ifx\csname\cf@encoding\string{cs_str}\endcsname\relax
      \csname?\string{cs_str}\endcsname\else\csname\cf@encoding\string{cs_str}\endcsname\fi").into()));
    }
    let replacement_value = font::decode(code_value, Some(encoding_str), false).unwrap();
    let replacement = PrimitiveBody::from(replacement_value);
    def_primitive(ecs, None, Some(replacement), PrimitiveOptions::default())?;
  });

  // hmmm... what needs doing here; basically it means use this encoding as the default for the
  // symbol
  DefMacro!("\\DeclareTextSymbolDefault DefToken {}", None);

  //------------------------------------------------------------
  DefPrimitive!("\\DeclareTextAccent DefToken {}{}", None);
  DefPrimitive!("\\DeclareTextAccentDefault{}{}", None);

  DefMacro!("\\fontencoding{}", "\\lx@fontencoding{#1}");
  DefMacro!("\\f@encoding", {
    ExplodeText!(LookupFont!().unwrap().get_encoding().unwrap())
  });
  DefMacro!("\\cf@encoding", {
    ExplodeText!(LookupFont!().unwrap().get_encoding().unwrap())
  });

  // #------------------------------------------------------------
  DefPrimitive!("\\DeclareTextComposite{}{}{}{}", None);
  // sub { ignoredDefinition("DeclareTextComposite", $_[1]); });
  DefPrimitive!("\\DeclareTextCompositeCommand{}{}{}{}", None);
  // sub { ignoredDefinition("DeclareTextCompositeCommand", $_[1]); });

  DefPrimitive!("\\UndeclareTextCommand{}{}", None);
  DefMacro!("\\UseTextSymbol{}{}", "{\\fontencoding{#1}#2}");
  DefMacro!("\\UseTextAccent{}{}", "{\\fontencoding{#1}#2{#3}}");

  // DefPrimitive('\DeclareMathAccent DefToken {}{} {Number}', sub {
  //     my ($stomach, $cs, $kind, $class, $code) = @_;
  //     $class = ToString($class);
  //     my $info = LookupValue('fontdeclaration@' . $class);
  //     my $glyph = FontDecode($code->valueOf, ($info ? $$info{encoding} : $class));
  //     DefMathI($cs, 'Digested', $glyph, operator_role => 'OVERACCENT');
  //     return; });

  DefPrimitive!("\\DeclareMathDelimiter{}{}{}{}", None);
  // sub { ignoredDefinition("DeclareMathAccent", $_[1]); });
  DefPrimitive!("\\DeclareMathRadical{}{}{}{}{}", None);
  // sub { ignoredDefinition("DeclareMathAccent", $_[1]); });
  DefPrimitive!("\\DeclareMathVersion{}", None);
  DefPrimitive!("\\DeclarePreloadSizes{}{}{}{}{}", None);

  // The next font declaration commands are based on
  // http://tex.loria.fr/general/new/fntguide.html
  // we ignore font encoding
  DefPrimitive!("\\DeclareSymbolFont{}{}{}{}{}",
  sub[(name, enc, family, series, shape)] {
    AssignValue!(&s!("fontdeclaration@{}", name),
      fontmap!(family => family.to_string(),
        series   => series.to_string(),
        shape    => shape.to_string(),
        encoding => enc.to_string()
      )
    );
  });
  DefPrimitive!("\\DeclareSymbolFontAlphabet {Token} {}", sub[(cs, name)] {
    let fontkey = s!("fontdeclarations@{}", name.to_string());
    let font : Option<Font> = if let Some(Stored::Font(value)) = lookup_value(&fontkey) {
      Some((*value).clone())
    } else {
      None
    };
    DefPrimitive!(cs, None, None, font => font);
  });

  DefPrimitive!("\\DeclareFixedFont{}{}{}{}{}{}", None);
  DefPrimitive!("\\DeclareErrorFont{}{}{}{}{}", None);

  DefMacro!("\\cdp@list", "\\@empty");
  Let!("\\cdp@elt", "\\relax");
  DefPrimitive!("\\DeclareFontEncoding{}{}{}", sub[(encoding, x, y)] {
    // TODO:
    // AddToMacro!(T_CS!("\\cdp@list"), T_CS!("\\cdp@elt"),
    //   T_BEGIN!(), encoding.unlist(), T_END,
    //   T_BEGIN!(), T_CS!("\\default@family"), *T_END,
    //   T_BEGIN!(), T_CS!("\\default@series"), *T_END,
    //   T_BEGIN!(), T_CS!("\\default@shape"),  *T_END);

    let e = Expand!(encoding);
    DefMacro!(T_CS!("\\LastDeclaredEncoding"), None, e.clone());
    DefMacro!(T_CS!(s!("\\T@{}", e)), None, x);
    DefMacro!(T_CS!(s!("\\M@{}", e)), None, Tokens!(T_CS!("\\default@M"), y.unlist()));
  });

  DefMacro!("\\LastDeclaredEncoding", None, None);
  DefPrimitive!("\\DeclareFontSubstitution{}{}{}{}", None);
  DefPrimitive!("\\DeclareFontEncodingDefaults{}{}", None);
  DefMacro!("\\LastDeclaredEncoding", None, None);

  DefPrimitive!("\\SetSymbolFont{}{}{}{}{}{}", None);
  DefPrimitive!("\\SetMathAlphabet{}{}{}{}{}{}", None);
  DefPrimitive!("\\addtoversion{}{}", None);
  DefPrimitive!("\\TextSymbolUnavailable{}", None);

  TeX!(
    r#"""
  \DeclareSymbolFont{operators}   {OT1}{cmr} {m}{n}
  \DeclareSymbolFont{letters}     {OML}{cmm} {m}{it}
  \DeclareSymbolFont{symbols}     {OMS}{cmsy}{m}{n}
  \DeclareSymbolFont{largesymbols}{OMX}{cmex}{m}{n}
  """#
  );
  // At least all things on uclclist need to be macros
  DefMacro!("\\lx@utf@OE", None, "\u{0152}", alias => "\\OE"); // LATIN CAPITAL LIGATURE OE
  DefMacro!("\\lx@utf@oe", None, "\u{0153}", alias => "\\oe"); // LATIN SMALL LIGATURE OE
  DefMacro!("\\lx@utf@AE", None, "\u{00C6}", alias => "\\AE"); // LATIN CAPITAL LETTER AE
  DefMacro!("\\lx@utf@ae", None, "\u{00E6}", alias => "\\ae"); // LATIN SMALL LETTER AE

  // LATIN CAPITAL LETTER A WITH RING ABOVE
  DefMacro!("\\lx@utf@AA", None, "\u{00C5}", alias => "\\AA");

  // LATIN SMALL LETTER A WITH RING ABOVE
  DefMacro!("\\lx@utf@aa", None, "\u{00E5}", alias => "\\aa");
  DefMacro!("\\lx@utf@O",  None, "\u{00D8}", alias => "\\O"); // LATIN CAPITAL LETTER O WITH STROKE
  DefMacro!("\\lx@utf@o",  None, "\u{00F8}", alias => "\\o"); // LATIN SMALL LETTER O WITH STROKE
  DefMacro!("\\lx@utf@L",  None, "\u{0141}", alias => "\\L"); // LATIN CAPITAL LETTER L WITH STROKE
  DefMacro!("\\lx@utf@l",  None, "\u{0142}", alias => "\\l"); // LATIN SMALL LETTER L WITH STROKE
  DefMacro!("\\lx@utf@ss", None, "\u{00DF}", alias => "\\ss"); // LATIN SMALL LETTER SHARP S
  DefMacro!("\\lx@utf@dh", None, "\u{00f0}", alias => "\\dh"); // eth
  DefMacro!("\\lx@utf@DH", None, "\u{00d0}", alias => "\\DH"); // Eth (looks same as \DJ!)
  DefMacro!("\\lx@utf@dj", None, "\u{0111}", alias => "\\dj"); // d with stroke
  DefMacro!("\\lx@utf@DJ", None, "\u{0110}", alias => "\\DJ"); // D with stroke (looks sames as \DH!)
  DefMacro!("\\lx@utf@ng", None, "\u{014B}", alias => "\\ng");
  DefMacro!("\\lx@utf@NG", None, "\u{014A}", alias => "\\NG");
  DefMacro!("\\lx@utf@th", None, "\u{00FE}", alias => "\\th");
  DefMacro!("\\lx@utf@TH", None, "\u{00DE}", alias => "\\TH");
  DefMacro!("\\OE", None, "\\lx@utf@OE");
  DefMacro!("\\oe", None, "\\lx@utf@oe");
  DefMacro!("\\AE", None, "\\lx@utf@AE");
  DefMacro!("\\ae", None, "\\lx@utf@ae");
  DefMacro!("\\ae", None, "\\lx@utf@ae");
  DefMacro!("\\AA", None, "\\lx@utf@AA");
  DefMacro!("\\aa", None, "\\lx@utf@aa");
  DefMacro!("\\O", None, "\\lx@utf@O");
  DefMacro!("\\o", None, "\\lx@utf@o");
  DefMacro!("\\L", None, "\\lx@utf@L");
  DefMacro!("\\l", None, "\\lx@utf@l");
  DefMacro!("\\ss", None, "\\lx@utf@ss");
  DefMacro!("\\dh", None, "\\lx@utf@dh"); // in latex?
  DefMacro!("\\DH", None, "\\lx@utf@DH");
  DefMacro!("\\dj", None, "\\lx@utf@dj");
  DefMacro!("\\DJ", None, "\\lx@utf@DJ");
  DefMacro!("\\ng", None, "\\lx@utf@ng");
  DefMacro!("\\NG", None, "\\lx@utf@NG");
  DefMacro!("\\th", None, "\\lx@utf@th");
  DefMacro!("\\TH", None, "\\lx@utf@TH");
});
