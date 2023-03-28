use crate::package::*;

//======================================================================
// TeX Book, Appendix B. p. 364
LoadDefinitions!(state, {
  // Let's hope nobody is messing with the output routine...
  // DefPrimitiveI('\footnoterule', undef, undef);

  // #======================================================================
  // # End of TeX Book definitions.
  // #======================================================================

  // #**********************************************************************
  // # Stray stuff .... where to ?
  // #**********************************************************************

  // # Mostly ignorable, although it could add an attribute to an ancestor
  // # to record the desired justification.
  // # Spacing stuff
  // DefConstructor('\@', '');
  // # Math spacing.

  // # Math style.
  // # Also record that this explicitly sets the mathstyle (support for \over, etal)
  // DefPrimitiveI('\displaystyle', undef, sub {
  //     MergeFont(mathstyle => 'display');
  //     Box(undef, undef, undef, T_CS('\displaystyle'), explicit_mathstyle => 1); });
  // DefPrimitiveI('\textstyle', undef, sub {
  //     MergeFont(mathstyle => 'text');
  //     Box(undef, undef, undef, T_CS('\textstyle'), explicit_mathstyle => 1); });
  // DefPrimitiveI('\scriptstyle', undef, sub {
  //     MergeFont(mathstyle => 'script');
  //     Box(undef, undef, undef, T_CS('\scriptstyle'), explicit_mathstyle => 1); });
  // DefPrimitiveI('\scriptscriptstyle', undef, sub {
  //     MergeFont(mathstyle => 'scriptscript');
  //     Box(undef, undef, undef, T_CS('\scriptscriptstyle'), explicit_mathstyle => 1); });

  // #======================================================================

  // DefMathI('\lx@math@hash',    undef, '#', alias => '\#');
  // DefMathI('\lx@math@amp',     undef, '&', role  => 'ADDOP', meaning => 'and', alias => '\&');
  // DefMathI('\lx@math@percent', undef, '%', role  => 'POSTFIX', meaning => 'percent', alias => '\%');
  // DefMathI('\lx@math@dollar', undef, "\$", role => 'OPERATOR', meaning => 'currency-dollar',
  //   alias => "\\\$");
  // DefMathI('\lx@math@underscore', undef, '_', alias => '\_');

  // # Discretionary times; just treat as invisible ?
  // DefMathI('\*', undef, "\x{2062}", role => 'MULOP', name => '', meaning => 'times'); # INVISIBLE TIMES (or MULTIPLICATION SIGN = 00D7)

  // # These 3 should have some `name' assigned ... but what???

  // Is XMWrap the right thing to wrap with (instead of XMArg)?
  // We can't really assume that the stuff inside is sensible math.
  // NOTE that \mathord and \mathbin aren't really right here.
  // We need a finer granularity than TeX does: an ORD could be several things,
  // a BIN could be a MULOP or ADDOP.
  // AND, rarely, they're empty.... Is it wrong to drop them?
  DefConstructor!("\\mathord{}", "?#1(<ltx:XMWrap role='ID'   >#1</ltx:XMWrap>)()", bounded => true);
  DefConstructor!("\\mathop{}", "?#1(<ltx:XMWrap role='BIGOP' scriptpos='#scriptpos'>#1</ltx:XMWrap>)()",
    bounded => true); // TODO: , properties => { scriptpos => \&doScriptpos });
  DefConstructor!("\\mathbin{}", "?#1(<ltx:XMWrap role='BINOP'>#1</ltx:XMWrap>)()", bounded => true);
  DefConstructor!("\\mathrel{}", "?#1(<ltx:XMWrap role='RELOP'>#1</ltx:XMWrap>)()", bounded => true);
  DefConstructor!("\\mathopen{}", "?#1(<ltx:XMWrap role='OPEN' >#1</ltx:XMWrap>)()", bounded => true);
  DefConstructor!("\\mathclose{}", "?#1(<ltx:XMWrap role='CLOSE'>#1</ltx:XMWrap>)()", bounded => true);
  DefConstructor!("\\mathpunct{}", "?#1(<ltx:XMWrap role='PUNCT'>#1</ltx:XMWrap>)()", bounded => true);
  DefConstructor!("\\mathinner{}", "?#1(<ltx:XMWrap role='ATOM'>#1</ltx:XMWrap>)()",  bounded => true);

  // If an XMWrap (presumably from \mathop, \mathbin, etc)
  // has multiple children, ALL are XMTok, within a restricted set of roles,
  // we want to concatenate the text content into a single XMTok.
  DefMathRewrite!(xpath => concat!("descendant-or-self::ltx:XMWrap[",
    // Only XMWrap's from the above class of operators
    "(@role='OP' or @role='BIGOP' or @role='RELOP' ",
    "or @role='ADDOP' or @role='MULOP' or @role='BINOP'",
    "or @role='OPEN' or @role='CLOSE')",
    " and count(child::*) > 1 ",
    // with only XMTok as children with the roles in (roughly) the same set
    " and not(child::*[local-name() != 'XMTok'])",
    " and not(ltx:XMTok[",
    "@role !='OP' and @role!='BIGOP' and @role!='RELOP' and role!='METARELOP'",
    "and @role!='ADDOP' and @role!='MULOP' and @role!='BINOP'",
    "and @role!='OPEN' and @role!='CLOSE'",
    "])]"),
  replace => sub[document, nodes, _state] {
    let node = nodes.pop().unwrap();
    let mut replacement = node.clone();
    let content     = node.get_content();
    replacement.append_text(&content)?;
    replacement.set_name("ltx:XMTok")?;
    document.get_node_mut().add_child(&mut replacement)?;
  });

  DefMath!('.', None, '.', role => "PERIOD");
  DefMath!(',', None, ',', role => "PUNCT");
  DefMath!(';', None, ';', role => "PUNCT");

  // DefMacro('\hiderel{}', "#1");    # Just ignore, for now...

  // DefMathI('\to', undef, "\x{2192}", role => 'ARROW'); # RIGHTWARDS ARROW??? a bit more explicitly relation-like?

  // # TeX's ligatures handled by rewrite regexps.
  // # Note: applied in reverse order of definition (latest defined applied first!)
  // # Note also, these area only applied in text content, not in attributes!
  // DefPrimitive('\@@endash', sub { Box("\x{2013}", undef, undef, T_CS('\@@endash')); });
  // DefPrimitive('\@@emdash', sub { Box("\x{2014}", undef, undef, T_CS('\@@emdash')); });

  // DefLigature(qr{--}, "\x{2013}",
  //   fontTest => sub { $_[0]->getFamily ne 'typewriter'; }); # EN DASH (NOTE: With digits before & aft => \N{FIGURE DASH})
  // DefLigature(qr{---}, "\x{2014}",
  //   fontTest => sub { $_[0]->getFamily ne 'typewriter'; });    # EM DASH

  // # Ligatures for doubled single left & right quotes to convert to double quotes
  // # [should ligatures be part of a font, in the first place? (it is in TeX!)
  // DefLigature(qr{\x{2018}\x{2018}}, "\x{201C}",
  //   fontTest => sub { ($_[0]->getFamily ne 'typewriter')
  //       && (($_[0]->getEncoding || 'OT1') =~ /^(OT1|T1)$/); });    # is this needed?
  // DefLigature(qr{\x{2019}\x{2019}}, "\x{201D}",
  //   fontTest => sub { ($_[0]->getFamily ne 'typewriter')
  //       && (($_[0]->getEncoding || 'OT1') =~ /^(OT1|T1)$/); });

  DefConstructor!("\\TeX", r###"<ltx:text class='ltx_TeX_logo'
    cssstyle='letter-spacing:-0.2em; margin-right:0.2em'>T<ltx:text yoffset='-0.4ex'>E</ltx:text>X</ltx:text>"###,
    sizer => sub[_whatsit, _state] { Ok((Dimension!("1.9em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  DefPrimitive!("\\i", "\u{0131}"); // LATIN SMALL LETTER DOTLESS I
  DefPrimitive!("\\j", "\u{0237}");

  // DefConstructor('\buildrel Until:\over {}',
  //   "<ltx:XMApp role='RELOP'>"
  //     . "<ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>"
  //     . "<ltx:XMArg>#2</ltx:XMArg>"
  //     . "<ltx:XMArg>#1</ltx:XMArg>"
  //     . "</ltx:XMApp>",
  //   properties => { scriptpos => sub { "mid" . $_[0]->getBoxingLevel; } });
});
