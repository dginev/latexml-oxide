use crate::package::*;

LoadDefinitions!({
  //======================================================================

  // Special Characters.
  // Try to give them some sense in math...
  DefMacro!("\\#", "\\ifmmode\\lx@math@hash\\else\\lx@text@hash\\fi");
  DefMacro!("\\&", "\\ifmmode\\lx@math@amp\\else\\lx@text@amp\\fi");
  DefMacro!(
    "\\%",
    "\\ifmmode\\lx@math@percent\\else\\lx@text@percent\\fi"
  );
  DefMacro!("\\$", "\\ifmmode\\lx@math@dollar\\else\\lx@text@dollar\\fi");
  DefMacro!(
    "\\_",
    "\\ifmmode\\lx@math@underscore\\else\\lx@text@underscore\\fi"
  );
  DefMacro!(T_CS!("\\lx@text@hash"), None, T_OTHER!("#"),  alias => "\\#");
  DefMacro!(T_CS!("\\lx@text@amp"), None, T_OTHER!("&"),  alias => "\\&");
  DefMacro!(T_CS!("\\lx@text@percent"), None, T_OTHER!("%"),  alias => "\\%");
  DefMacro!(T_CS!("\\lx@text@dollar"), None,  T_OTHER!("$"), alias => "\\$");
  DefMacro!(T_CS!("\\lx@text@underscore"), None, T_OTHER!("_"),  alias => "\\_");

  DefMath!("\\lx@math@hash",  None, "#", alias => "\\#");
  DefMath!("\\lx@math@amp",   None, "&", role  => "ADDOP", meaning => "and", alias => "\\&");
  DefMath!("\\lx@math@percent", None, "%", role  => "POSTFIX", meaning => "percent", alias => "\\%");
  DefMath!("\\lx@math@dollar", None, "\\$", role => "OPERATOR", meaning => "currency-dollar",
    alias => "\\$");
  DefMath!("\\lx@math@underscore", None, "_", alias => "\\_");

  // Discretionary times; just treat as invisible ?
  // INVISIBLE TIMES (or MULTIPLICATION SIGN = 00D7)
  DefMath!("\\*", None, "\u{2062}", role => "MULOP", name => "", meaning => "times");

  // These 3 should have some `name' assigned ... but what???

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
  replace => sub[document, nodes] {
    let node = nodes.pop().unwrap();
    let mut replacement = node.clone();
    let content     = node.get_content();
    replacement.append_text(&content)?;
    replacement.set_name("ltx:XMTok")?;
    document.get_node_mut().add_child(&mut replacement)?;
  });

  DefMacro!("\\hiderel{}", "#1");    // Just ignore, for now...

  // RIGHTWARDS ARROW??? a bit more explicitly relation-like?
  DefMath!("\\to", None, "\u{2192}", role => "ARROW");

  // TeX's ligatures handled by rewrite regexps.
  // Note: applied in reverse order of definition (latest defined applied first!)
  // Note also, these area only applied in text content, not in attributes!
  DefPrimitive!("\\@@endash", {
    Tbox::new(arena::pin_static("\u{2013}"), None, None,
      Tokens!(T_CS!("\\@@endash")), HashMap::default()); });
  DefPrimitive!("\\@@emdash", {
    Tbox::new(arena::pin_static("\u{2014}"), None, None,
      Tokens!(T_CS!("\\@@emdash")), HashMap::default()); });


  // EN DASH (NOTE: With digits before & aft => \N{FIGURE DASH})
  DefLigature!(r"--", "\u{2013}",
    fontTest => sub[arg] { arg.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter" });
  // EM DASH
  DefLigature!(r"---", "\u{2014}", fontTest => sub[arg] {arg.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter" });
  // Ligatures for doubled single left & right quotes to convert to double quotes
  // [should ligatures be part of a font, in the first place? (it is in TeX!)
  DefLigature!("\u{2018}\u{2018}", "\u{201C}", fontTest => sub[arg] {
    let family = arg.get_family().unwrap_or(&Cow::Borrowed(""));
    if family != "typewriter" {
      let encoding = arg.get_encoding().unwrap_or(&Cow::Borrowed("OT1"));
      encoding == "OT1" || encoding == "T1" } else {false} });
  DefLigature!("\u{2019}\u{2019}", "\u{201D}", fontTest => sub[arg] {
    let family = arg.get_family().unwrap_or(&Cow::Borrowed(""));
    if family != "typewriter" {
      let encoding = arg.get_encoding().unwrap_or(&Cow::Borrowed("OT1"));
      encoding == "OT1" || encoding == "T1" } else {false} });

  DefConstructor!("\\TeX", r###"<ltx:text class='ltx_TeX_logo'
    cssstyle='letter-spacing:-0.2em; margin-right:0.2em'>T<ltx:text yoffset='-0.4ex'>E</ltx:text>X</ltx:text>"###,
    sizer => sub[_whatsit] { Ok((Dimension!("1.9em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });
  DefPrimitive!("\\i", "\u{0131}"); // LATIN SMALL LETTER DOTLESS I
  DefPrimitive!("\\j", "\u{0237}");

  DefConstructor!("\\buildrel Until:\\over {}",
    "<ltx:XMApp role='RELOP'>\
      <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>\
      <ltx:XMArg>#2</ltx:XMArg>\
      <ltx:XMArg>#1</ltx:XMArg>\
      </ltx:XMApp>"
    // TODO
    // properties => { scriptpos => sub { "mid" . $_[0]->getBoxingLevel; } }
  );

});
