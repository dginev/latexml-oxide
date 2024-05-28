use crate::prelude::*;

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
  DefPrimitive!(T_CS!("\\lx@text@hash"), None, "#",  alias => "\\#");
  DefPrimitive!(T_CS!("\\lx@text@amp"), None, "&",  alias => "\\&");
  DefPrimitive!(T_CS!("\\lx@text@percent"), None, "%",  alias => "\\%");
  DefPrimitive!(T_CS!("\\lx@text@dollar"), None,  "$", alias => "\\$");
  DefPrimitive!(T_CS!("\\lx@text@underscore"), None, "_",  alias => "\\_");

  DefMath!("\\lx@math@hash",  None, "#", alias => "\\#");
  DefMath!("\\lx@math@amp",   None, "&", role  => "ADDOP", meaning => "and", alias => "\\&");
  DefMath!("\\lx@math@percent", None, "%", role  => "POSTFIX", meaning => "percent", alias => "\\%");
  DefMath!("\\lx@math@dollar", None, "\\$", role => "OPERATOR", meaning => "currency-dollar",
    alias => "\\$");
  DefMath!("\\lx@math@underscore", None, "_", alias => "\\_");

  // Discretionary times; just treat as invisible ?
  // INVISIBLE TIMES (or MULTIPLICATION SIGN = 00D7)
  DefMath!("\\*", None, "\u{2062}", role => "MULOP", name => "", meaning => "times");

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

  // RIGHTWARDS ARROW??? a bit more explicitly relation-like?
  DefMath!("\\to", None, "\u{2192}", role => "ARROW");

  // TeX's ligatures handled by rewrite regexps.
  // Note: applied in reverse order of definition (latest defined applied first!)
  // Note also, these area only applied in text content, not in attributes!
  DefPrimitive!("\\@@endash", {
    Tbox::new(arena::pin_static("\u{2013}"), None, None,
      Tokens!(T_CS!("\\@@endash")), SymHashMap::default()); });
  DefPrimitive!("\\@@emdash", {
    Tbox::new(arena::pin_static("\u{2014}"), None, None,
      Tokens!(T_CS!("\\@@emdash")), SymHashMap::default()); });


  // EN DASH (NOTE: With digits before & aft => \N{FIGURE DASH})
  DefLigature!(r"--", "\u{2013}", fontTest => sub[arg] { non_typewriter(arg) });
  // EM DASH
  DefLigature!(r"---", "\u{2014}", fontTest => sub[arg] {non_typewriter(arg) });

  // Ligatures for doubled single left & right quotes to convert to double quotes
  // [should ligatures be part of a font, in the first place? (it is in TeX!)
  DefLigature!("\u{2018}\u{2018}", "\u{201C}", 
    fontTest => sub[arg] {non_typewriter_t1(arg)}); // double left quote
  DefLigature!("\u{2019}\u{2019}", "\u{201D}", 
    fontTest => sub[arg] {non_typewriter_t1(arg)}); // double right quote
  DefLigature!("[?]\u{2018}",       "\u{00BF}",  
    fontTest => sub[arg] {non_typewriter_t1(arg)});   // ? backquote
  DefLigature!("!\u{2018}",       "\u{00A1}",  
    fontTest => sub[arg] {non_typewriter_t1(arg)});   // ! backquote
  // These ligatures are also handled by TeX.
  // However, it appears that decent modern fonts in modern browsers handle these at that level.
  // So it's likely not worth doing it at the conversion level, possibly adversely affecting search.
  // DefLigature(qr{ff},               "\x{FB00}", fontTest => \&nonTypewriterT1);
  // DefLigature(qr{fi},               "\x{FB01}", fontTest => \&nonTypewriterT1);
  // DefLigature(qr{fl},               "\x{FB02}", fontTest => \&nonTypewriterT1);
  // DefLigature(qr{ffi},              "\x{FB03}", fontTest => \&nonTypewriterT1);
  // DefLigature(qr{ffl},              "\x{FB04}", fontTest => \&nonTypewriterT1);

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

fn non_typewriter(font: &Font) -> bool {
  font.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter"
}

fn non_typewriter_t1(font: &Font) -> bool {
  non_typewriter(font) &&
  matches!(font.get_encoding().unwrap_or(&Cow::Borrowed("OT1")).as_ref(), "OT1" | "T1")
}