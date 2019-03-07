use crate::package::*;
LoadDefinitions!(state, {
  //**********************************************************************
  // C.8 Definitions, Numbering and Programming
  //**********************************************************************

  //======================================================================
  // C.8.1 Defining Commands
  //======================================================================

  // DefMacro('\@tabacckludge {}', '\csname\string#1\endcsname');

  DefPrimitive!("\\newcommand OptionalMatch:* DefToken [Number][]{}", sub[stomach, args, state] {
    unpack!(args => star, cs, nargs, opt, body);
    let cs_token: Token = cs.into();
    let nargs_token: Token = nargs.into();
    let nargs = nargs_token.to_number().value_of() as usize;
    // TODO:
    // if (!isDefinable(cs)) {
    //   Info('ignore', $cs, $stomach,
    //     "Ignoring redefinition (\\newcommand) of '" . Stringify($cs) . "'")
    //     unless LookupValue(ToString($cs) . ':locked');
    //   return; }
    let opt = if opt.is_empty() { None } else { Some(opt) };
    let macro_args = convert_latex_args(nargs, opt, state)?;
    DefMacroI!(cs_token, macro_args, body);
  });

  DefPrimitive!("\\renewcommand OptionalMatch:* DefToken [Number][]{}", sub[stomach, args, state] {
    unpack!(args => star, cs, nargs, opt, body);
    let cs_token: Token = cs.into();
    let nargs_token: Token = nargs.into();
    let nargs = nargs_token.to_number().value_of() as usize;
    let opt = if opt.is_empty() { None } else { Some(opt) };
    let macro_args = convert_latex_args(nargs, opt, state)?;
    DefMacroI!(cs_token, macro_args, body);
  });


  //------------------------------------------------------------
  // The following commands define encoding-specific expansions
  // or glyphs.  The control-sequence is defined to use the expansion for
  // the current encoding, if any, or the default expansion (for encoding "?").
  // We don't want to redefine control-sequence if it already has a definition:
  // It may be that we've already defined it to expand into the above conditional.
  // But more importantly, we don't want to override a hand-written definition (if any).
  //------------------------------------------------------------
  DefPrimitive!("\\DeclareTextCommand DefToken {}[Number][]{}", sub[stomach, args, state] {
    unpack_to_token!(args => cs, encoding, nargs, opt, expansion);
    // TODO
//     my $css = ToString($cs);
//     $encoding = ToString(Expand($encoding));
//     if (!IsDefined($cs)) {    # If not already defined...
//       DefMacroI($cs, undef,
// '\expandafter\ifx\csname\cf@encoding\string' . $css . '\endcsname\relax\csname?\string' . $css . '\endcsname'
//           . '\else\csname\cf@encoding\string' . $css . '\endcsname\fi'); }
//     my $ecs = T_CS('\\' . $encoding . $css);
//     DefMacroI($ecs, convertLaTeXArgs($nargs, $opt), $expansion);
    
  });

  DefMacro!("\\DeclareTextCommandDefault DefToken", "\\DeclareTextCommand{#1}{?}");

  // DefPrimitive('\ProvideTextCommand DefToken {}[Number][]{}', sub {
  //     my ($gullet, $cs, $encoding, $nargs, $opt, $expansion) = @_;
  //     my $css = ToString($cs);
  //     $encoding = ToString(Expand($encoding));
  //     if (isDefinable($cs)) {    # If not already defined...
  //       DefMacroI($cs, undef,
  // '\expandafter\ifx\csname\cf@encoding\string' . $css . '\endcsname\relax\csname?\string' . $css . '\endcsname'
  //           . '\else\csname\cf@encoding\string' . $css . '\endcsname\fi'); }
  //     my $ecs = T_CS('\\' . $encoding . $css);
  //     if (!IsDefined($ecs)) {    # If not already defined...
  //       DefMacroI($ecs, convertLaTeXArgs($nargs, $opt), $expansion); }
  //     return; });

  // DefMacro('\ProvideTextCommandDefault DefToken', '\ProvideTextCommand{#1}{?}');

  // #------------------------------------------------------------

  DefPrimitive!("\\DeclareTextSymbol DefToken {}{Number}", sub[stomach, args, state] {
    unpack_to_token!(args => cs, encoding, code);
    // TODO:
    //     $code = $code->valueOf;
    //     my $css = ToString($cs);
    //     $encoding = ToString(Expand($encoding));
    //     if (isDefinable($cs)) {    # If not already defined...
    //       DefMacroI($cs, undef,
    // '\expandafter\ifx\csname\cf@encoding\string' . $css . '\endcsname\relax\csname?\string' . $css . '\endcsname'
    //           . '\else\csname\cf@encoding\string' . $css . '\endcsname\fi'); }
    //     my $ecs = T_CS('\\' . $encoding . $css);
    //     DefPrimitiveI($ecs, undef, FontDecode($code, $encoding));
  });
      

  // hmmm... what needs doing here; basically it means use this encoding as the default for the symbol
  DefMacro!("\\DeclareTextSymbolDefault DefToken {}", "");

  //------------------------------------------------------------
  DefPrimitive!("\\DeclareTextAccent DefToken {}{}", None);
  DefPrimitive!("\\DeclareTextAccentDefault{}{}", None);

  // #------------------------------------------------------------
  // DefPrimitive('\DeclareTextComposite{}{}{}{}',
  //   sub { ignoredDefinition('DeclareTextComposite', $_[1]); });
  // DefPrimitive('\DeclareTextCompositeCommand{}{}{}{}',
  //   sub { ignoredDefinition('DeclareTextCompositeCommand', $_[1]); });

  // DefPrimitive('\UndeclareTextCommand{}{}', undef);
  // DefMacro('\UseTextSymbol{}{}', '{\fontencoding{#1}#2}');
  // DefMacro('\UseTextAccent{}{}', '{\fontencoding{#1}#2{#3}}');

  // DefPrimitive('\DeclareMathAccent DefToken {}{} {Number}', sub {
  //     my ($stomach, $cs, $kind, $class, $code) = @_;
  //     $class = ToString($class);
  //     my $info = LookupValue('fontdeclaration@' . $class);
  //     my $glyph = FontDecode($code->valueOf, ($info ? $$info{encoding} : $class));
  //     DefMathI($cs, 'Digested', $glyph, operator_role => 'OVERACCENT');
  //     return; });

  // DefPrimitive('\DeclareMathDelimiter{}{}{}{}',
  //   sub { ignoredDefinition('DeclareMathAccent', $_[1]); });
  // DefPrimitive('\DeclareMathRadical{}{}{}{}{}',
  //   sub { ignoredDefinition('DeclareMathAccent', $_[1]); });
  // DefPrimitive('\DeclareMathVersion{}',          undef);
  // DefPrimitive('\DeclarePreloadSizes{}{}{}{}{}', undef);

  // The next font declaration commands are based on
  // http://tex.loria.fr/general/new/fntguide.html
  // we ignore font encoding
  DefPrimitive!("\\DeclareSymbolFont{}{}{}{}{}", sub[stomach, args, state] {
    unpack_to_token!(args => name, enc, family, series, shape);
    AssignValue!(&s!("fontdeclaration@{}", name),
      fontmap!(family => family.to_string(),
        series   => series.to_string(),
        shape    => shape.to_string(),
        encoding => enc.to_string()
      )
    );
  });
  DefPrimitive!("\\DeclareSymbolFontAlphabet{}{}", sub[stomach, args, state] {
    unpack_to_token!(args => cs, name);
    let fontkey = s!("fontdeclarations@{}", name.to_string());
    let font : Option<Font> = if let Some(Stored::Font(value)) = LookupValue!(&fontkey) {
      Some((**value).clone())
    } else {
      None
    };
    DefPrimitiveII!(cs, None, None, font => font);
  });

  DefPrimitiveI!("\\DeclareFixedFont{}{}{}{}{}{}", None);
  DefPrimitiveI!("\\DeclareErrorFont{}{}{}{}{}", None);

  DefMacro!("\\cdp@list", "\\@empty");
  Let!("\\cdp@elt", "\\relax");
  DefPrimitive!("\\DeclareFontEncoding{}{}{}", sub[stomach, args, state] {
    unpack_to_token!(args => encoding, x, y);
    // TODO:
    // AddToMacro!(T_CS!("\\cdp@list"), T_CS!("\\cdp@elt"),
    //   T_BEGIN!(), encoding.unlist(), T_END,
    //   T_BEGIN!(), T_CS!("\\default@family"), T_END!(),
    //   T_BEGIN!(), T_CS!("\\default@series"), T_END!(),
    //   T_BEGIN!(), T_CS!("\\default@shape"),  T_END!());
    let gullet = stomach.get_gullet_mut();
    let e = Expand!(encoding, gullet, state);
    DefMacroI!(T_CS!("\\LastDeclaredEncoding"), None, e.clone());
    DefMacroI!(T_CS!(s!("\\T@{}", e)), None, x);
    DefMacroI!(T_CS!(s!("\\M@{}", e)), None, Tokens!(T_CS!("\\default@M"), y.unlist()));
  });

  DefMacroI!("\\LastDeclaredEncoding", None, "");
  DefPrimitiveI!("\\DeclareFontSubstitution{}{}{}{}", None);
  DefPrimitiveI!("\\DeclareFontEncodingDefaults{}{}", None);
  DefMacroI!("\\LastDeclaredEncoding", None, "");

  DefPrimitiveI!("\\SetSymbolFont{}{}{}{}{}{}",   None);
  DefPrimitiveI!("\\SetMathAlphabet{}{}{}{}{}{}", None);
  DefPrimitiveI!("\\addtoversion{}{}",            None);
  DefPrimitiveI!("\\TextSymbolUnavailable{}",     None);

  RawTeX!(r#"""
  \DeclareSymbolFont{operators}   {OT1}{cmr} {m}{n}
  \DeclareSymbolFont{letters}     {OML}{cmm} {m}{it}
  \DeclareSymbolFont{symbols}     {OMS}{cmsy}{m}{n}
  \DeclareSymbolFont{largesymbols}{OMX}{cmex}{m}{n}
  """#);
  // At least all things on uclclist need to be macros
  DefMacroI!("\\lx@utf@OE", None, "\u{0152}", alias => "\\OE"); // LATIN CAPITAL LIGATURE OE
  DefMacroI!("\\lx@utf@oe", None, "\u{0153}", alias => "\\oe"); // LATIN SMALL LIGATURE OE
  DefMacroI!("\\lx@utf@AE", None, "\u{00C6}", alias => "\\AE"); // LATIN CAPITAL LETTER AE
  DefMacroI!("\\lx@utf@ae", None, "\u{00E6}", alias => "\\ae"); // LATIN SMALL LETTER AE
  DefMacroI!("\\lx@utf@AA", None, "\u{00C5}", alias => "\\AA"); // LATIN CAPITAL LETTER A WITH RING ABOVE
  DefMacroI!("\\lx@utf@aa", None, "\u{00E5}", alias => "\\aa"); // LATIN SMALL LETTER A WITH RING ABOVE
  DefMacroI!("\\lx@utf@O",  None, "\u{00D8}", alias => "\\O");  // LATIN CAPITAL LETTER O WITH STROKE
  DefMacroI!("\\lx@utf@o",  None, "\u{00F8}", alias => "\\o");  // LATIN SMALL LETTER O WITH STROKE
  DefMacroI!("\\lx@utf@L",  None, "\u{0141}", alias => "\\L");  // LATIN CAPITAL LETTER L WITH STROKE
  DefMacroI!("\\lx@utf@l",  None, "\u{0142}", alias => "\\l");  // LATIN SMALL LETTER L WITH STROKE
  DefMacroI!("\\lx@utf@ss", None, "\u{00DF}", alias => "\\ss"); // LATIN SMALL LETTER SHARP S
  DefMacroI!("\\lx@utf@dh", None, "\u{00f0}", alias => "\\dh"); // eth
  DefMacroI!("\\lx@utf@DH", None, "\u{00d0}", alias => "\\DH"); // Eth (looks same as \DJ!)
  DefMacroI!("\\lx@utf@dj", None, "\u{0111}", alias => "\\dj"); // d with stroke
  DefMacroI!("\\lx@utf@DJ", None, "\u{0110}", alias => "\\DJ"); // D with stroke (looks sames as \DH!)
  DefMacroI!("\\lx@utf@ng", None, "\u{014B}", alias => "\\ng");
  DefMacroI!("\\lx@utf@NG", None, "\u{014A}", alias => "\\NG");
  DefMacroI!("\\lx@utf@th", None, "\u{00FE}", alias => "\\th");
  DefMacroI!("\\lx@utf@TH", None, "\u{00DE}", alias => "\\TH");
  DefMacroI!("\\OE", None, "\\lx@utf@OE");
  DefMacroI!("\\oe", None, "\\lx@utf@oe");
  DefMacroI!("\\AE", None, "\\lx@utf@AE");
  DefMacroI!("\\ae", None, "\\lx@utf@ae");
  DefMacroI!("\\ae", None, "\\lx@utf@ae");
  DefMacroI!("\\AA", None, "\\lx@utf@AA");
  DefMacroI!("\\aa", None, "\\lx@utf@aa");
  DefMacroI!("\\O",  None, "\\lx@utf@O");
  DefMacroI!("\\o",  None, "\\lx@utf@o");
  DefMacroI!("\\L",  None, "\\lx@utf@L");
  DefMacroI!("\\l",  None, "\\lx@utf@l");
  DefMacroI!("\\ss", None, "\\lx@utf@ss");
  DefMacroI!("\\dh", None, "\\lx@utf@dh"); // in latex?
  DefMacroI!("\\DH", None, "\\lx@utf@DH");
  DefMacroI!("\\dj", None, "\\lx@utf@dj");
  DefMacroI!("\\DJ", None, "\\lx@utf@DJ");
  DefMacroI!("\\ng", None, "\\lx@utf@ng");
  DefMacroI!("\\NG", None, "\\lx@utf@NG");
  DefMacroI!("\\th", None, "\\lx@utf@th");
  DefMacroI!("\\TH", None, "\\lx@utf@TH");
});
