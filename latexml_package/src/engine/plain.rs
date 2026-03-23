//! plain TeX
//!
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;

// Match negations of many operators
// our %NOTS
static MATH_CHAR_NEGATIONS: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
  map!("=" => "\u{2260}", "<" => "\u{226E}", ">" => "\u{226F}",
  "\u{2208}" => "\u{2209}",                              //\in=>\notin
  "\u{2264}" => "\u{2270}", "\u{2265}" => "\u{2271}",    // Less eq, greater eq.
  "\u{227A}" => "\u{2280}", "\u{227B}" => "\u{2281}",    // prec, succ
  "\u{2AAF}" => "\u{22E0}", "\u{2AB0}" => "\u{22E1}",    // preceq, succeq
  "\u{2282}" => "\u{2284}", "\u{2283}" => "\u{2285}",    // subset, supset
  "\u{2286}" => "\u{2288}", "\u{2287}" => "\u{2289}",    // subseteq, supseteq
  "\u{2291}" => "\u{22E2}", "\u{2290}" => "\u{22E3}",    // sqsubseteq, sqsupseteq
  "\u{2261}" => "\u{2262}",                              // equiv
  "\u{224D}" => "\u{226D}", "\u{2248}" => "\u{2249}",    // asymp, approx
  "\u{22B2}" => "\u{22EA}", "\u{22B3}" => "\u{22EB}",    // lhd, rhd
  "\u{22B4}" => "\u{22EC}", "\u{22B5}" => "\u{22ED}",    // unlhd, unrhd
  "\u{2203}" => "\u{2204}"                              // Exists
  )
});

/// Delimiter char properties for augmenting delimiter elements.
/// Mirrors Perl's %DELIMITER_MAP (TeX_Math.pool.ltxml L732-823).
struct DelimCharProps {
  left_role:      &'static str,
  right_role:     &'static str,
  name:           Option<&'static str>,
  remove_meaning: bool,
  replace_char:   Option<char>,
}

/// Char-keyed delimiter map for augmentDelimiterProperties lookup.
static DELIM_CHAR_MAP: Lazy<HashMap<char, DelimCharProps>> = Lazy::new(|| {
  raw_map!(
    '(' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    ')' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    '[' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    ']' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    '{' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    '}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    '\u{230A}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("lfloor"), remove_meaning: false, replace_char: None },
    '\u{230B}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("rfloor"), remove_meaning: false, replace_char: None },
    '\u{2308}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("lceil"), remove_meaning: false, replace_char: None },
    '\u{2309}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("rceil"), remove_meaning: false, replace_char: None },
    '\u{27E8}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("langle"), remove_meaning: false, replace_char: None },
    '\u{27E9}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("rangle"), remove_meaning: false, replace_char: None },
    '<' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("langle"), remove_meaning: true, replace_char: Some('\u{27E8}') },
    '>' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("rangle"), remove_meaning: true, replace_char: Some('\u{27E9}') },
    '/' => DelimCharProps { left_role: "MULOP", right_role: "MULOP",
      name: None, remove_meaning: false, replace_char: None },
    '\u{005C}' => DelimCharProps { left_role: "MULOP", right_role: "MULOP",
      name: Some("backslash"), remove_meaning: false, replace_char: None },
    '|' => DelimCharProps { left_role: "VERTBAR", right_role: "VERTBAR",
      name: None, remove_meaning: false, replace_char: None },
    '\u{2225}' => DelimCharProps { left_role: "VERTBAR", right_role: "VERTBAR",
      name: None, remove_meaning: false, replace_char: None },
    '\u{2191}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("uparrow"), remove_meaning: false, replace_char: None },
    '\u{21D1}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("Uparrow"), remove_meaning: false, replace_char: None },
    '\u{2193}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("downarrow"), remove_meaning: false, replace_char: None },
    '\u{21D3}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("Downarrow"), remove_meaning: false, replace_char: None },
    '\u{2195}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("updownarrow"), remove_meaning: false, replace_char: None },
    '\u{21D5}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("Updownarrow"), remove_meaning: false, replace_char: None }
  )
});

/// Set role and augment delimiter properties on the last child element.
/// Mirrors Perl's augmentDelimiterProperties (TeX_Math.pool.ltxml).
fn augment_delimiter_properties(document: &mut Document, role: &str) -> Result<()> {
  let current = document.get_node().clone();
  let delim_opt = current
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .last();
  if let Some(mut delim) = delim_opt {
    let char_content = delim.get_content();
    let first_char = char_content.chars().next();
    // Look up delimiter properties by char content
    if let Some(entry) = first_char.and_then(|c| DELIM_CHAR_MAP.get(&c)) {
      // Role: use lrole if requested role is OPEN, else rrole
      let new_role = if role == "OPEN" { entry.left_role } else { entry.right_role };
      // Only set role if current role is a delimiter role or absent
      let current_role = delim.get_attribute("role");
      match current_role.as_deref() {
        None | Some("OPEN") | Some("MIDDLE") | Some("CLOSE") | Some("VERTBAR") => {
          document.set_attribute(&mut delim, "role", new_role)?;
        },
        _ => {},
      }
      // Set name if entry has one
      if let Some(name) = entry.name {
        document.set_attribute(&mut delim, "name", name)?;
      }
      // Handle meaning
      if entry.remove_meaning {
        let _ = delim.remove_attribute("meaning");
      }
      // Handle char replacement (e.g. < → ⟨)
      if let Some(replacement) = entry.replace_char {
        if let Some(mut first_child) = delim.get_first_child() {
          let _ = first_child.set_content(&replacement.to_string());
        }
      }
    } else {
      // No map entry — just set role as before
      let current_role = delim.get_attribute("role");
      match current_role.as_deref() {
        None | Some("OPEN") | Some("MIDDLE") | Some("CLOSE") | Some("VERTBAR") => {
          document.set_attribute(&mut delim, "role", role)?;
        },
        _ => {},
      }
    }
  }
  Ok(())
}

LoadDefinitions!({
  //**********************************************************************
  // Plain;  Extracted from Appendix B.
  //**********************************************************************

  // Remember, we're assigning a NUMBER (codepoint) to a CHARACTER!
  {
    for digit in 0..10 {
      assign_mathcode((b'0' + digit) as char, 0x7030 + (digit as u16), Some(Scope::Global));
    }
    for letter in b'A'..=b'Z' {
      //FYI: 0x20 == 32
      assign_lccode(letter, letter + 32, Some(Scope::Global));
      assign_uccode(letter, letter, Some(Scope::Global));
      assign_mathcode(letter as char, 0x7100 + (letter as u16), Some(Scope::Global));
      assign_sfcode(letter as char, 999u16, Some(Scope::Global));

      assign_lccode(letter + 32, letter + 32, Some(Scope::Global));
      assign_uccode(letter + 32, letter, Some(Scope::Global));
      assign_mathcode((letter + 32) as char, 0x7100 + ((letter + 32) as u16), Some(Scope::Global));
    }
  }
  DefRegister!("\\magnification", Number!(1000));
  Let!("\\bye", "\\lx@end@document");

  // Most of these are ignored, but...
  DefMacro!(
    "\\tracingall",
    "\\tracingonline=1 \\tracingcommands=2 \\tracingstats=2 \
     \\tracingpages=1 \\tracingoutput=1 \\tracinglostchars=1 \\tracingmacros=2 \
     \\tracingparagraphs=1 \\tracingrestores=1 \\showboxbreadth=\\maxdimen \
     \\showboxdepth=\\maxdimen \\errorstopmode"
  );
  DefMacro!("\\tracingnone", None);
  DefMacro!("\\hideoutput", None);

  //======================================================================
  // \choose & friends, also need VERY special argument handling

  // Perl: math_common.pool.ltxml L634-642 — no braces around left/right values
  DefMacro!("\\choose",
    "\\lx@generalized@over{\\choose}{meaning=binomial,thickness=0pt,left=\\lx@left(,right=\\lx@right)}");
  DefMacro!("\\brace",
    "\\lx@generalized@over{\\brace}{thickness=0pt,left=\\lx@left\\{,right=\\lx@right\\}}");
  DefMacro!("\\brack",
    "\\lx@generalized@over{\\brack}{thickness=0pt,left=\\lx@left[,right=\\lx@right]}");


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

  // TeX's ligatures handled by rewrite regexps.
  // Note: applied in reverse order of definition (latest defined applied first!)
  // Note also, these area only applied in text content, not in attributes!
  DefPrimitive!("\\@@endash", {
    Tbox::new(
      arena::pin_static("\u{2013}"),
      None,
      None,
      Tokens!(T_CS!("\\@@endash")),
      SymHashMap::default(),
    );
  });
  DefPrimitive!("\\@@emdash", {
    Tbox::new(
      arena::pin_static("\u{2014}"),
      None,
      None,
      Tokens!(T_CS!("\\@@emdash")),
      SymHashMap::default(),
    );
  });

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
    fontTest => sub[arg] {non_typewriter_t1(arg)}); // ? backquote
  DefLigature!("!\u{2018}",       "\u{00A1}",  
    fontTest => sub[arg] {non_typewriter_t1(arg)}); // ! backquote
  // These ligatures are also handled by TeX.
  // However, it appears that decent modern fonts in modern browsers handle these at that level.
  // So it's likely not worth doing it at the conversion level, possibly adversely affecting search.
  // DefLigature(qr{ff},               "\x{FB00}", fontTest => \&nonTypewriterT1);
  // DefLigature(qr{fi},               "\x{FB01}", fontTest => \&nonTypewriterT1);
  // DefLigature(qr{fl},               "\x{FB02}", fontTest => \&nonTypewriterT1);
  // DefLigature(qr{ffi},              "\x{FB03}", fontTest => \&nonTypewriterT1);
  // DefLigature(qr{ffl},              "\x{FB04}", fontTest => \&nonTypewriterT1);

  // Perl: enterHorizontal => 1
  DefConstructor!("\\TeX", "<ltx:text class='ltx_TeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.2em'
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%;' yoffset='-0.2ex'
  >e</ltx:text>X</ltx:text>",
    locked => true,
    enter_horizontal => true,
    sizer => sub[_whatsit] { Ok((Dimension!("1.9em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });
  DefPrimitive!("\\i", "\u{0131}"); // LATIN SMALL LETTER DOTLESS I
  DefPrimitive!("\\j", "\u{0237}");

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Alignment code
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  //======================================================================
  // Low-level bits that appear within alignments or \halign

  // "Initialized" alignment; presets spacing, but since we're ignoring it anyway...
  Let!("\\ialign", "\\halign");
  // Overlapping alignments ???
  DefMacro!(
    "\\oalign{}",
    r"\@@oalign{\lx@begin@alignment#1\lx@end@alignment}"
  );
  // TODO: What are the full arguments to alignment_bindings ?
  // DefConstructor!("\\@@oalign{}", "#1",
  //   reversion    => "\\oalign{#1}", bounded => true, mode => "text",
  //   before_digest => sub { alignment_bindings('l', ); });

  // This is actually different; the lines should lie ontop of each other.
  // How should this be represented?
  // TODO: What are the full arguments to alignment_bindings ?
  // DefMacro("\\ooalign{}",
  //   r"\@@ooalign{\lx@begin@alignment#1\lx@end@alignment}");
  // DefConstructor!("\\@@ooalign{}",
  //   "#1",
  //   reversion    => "\\ooalign{#1}", bounded => true, mode => "text",
  //   before_digest => sub { alignment_bindings('l'); });

  DefConstructor!(
    "\\buildrel Until:\\over {}",
    "<ltx:XMApp role='RELOP'>\
    <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>\
    <ltx:XMArg>#2</ltx:XMArg>\
    <ltx:XMArg>#1</ltx:XMArg>\
    </ltx:XMApp>" /* TODO
                   * properties => { scriptpos => sub { "mid" . $_[0]->getBoxingLevel; } } */
  );
  DefMacro!("\\hidewidth", None);
  DefMacro!("\\multispan{Number}", sub[(span)] {
    let n = span.value_of();
    let mut tks = vec![T_CS!("\\omit")];
    for _ in 1..n {
      tks.push(T_CS!("\\span"));
      tks.push(T_CS!("\\omit"));
    }
    Ok(Tokens::new(tks))
  });

  //======================================================================
  // TeX Book, Appendix B, p. 344
  //======================================================================
  TeX!(r"\outer\def^^L{\par}");
  DefMacro!(
    "\\dospecials",
    r"\do\ \do\\\do\{\do\}\do\$\do\&\do\#\do\^\do\^^K\do\_\do\^^A\do\%\do\~"
  );

  //======================================================================
  // TeX Book, Appendix B, p. 345
  TeX!(
    r"\chardef\active=13
    \chardef\@ne=1
    \chardef\tw@=2
    \chardef\thr@@=3
    \chardef\sixt@@n=16
    \chardef\@cclv=255
    \mathchardef\@cclvi=256
    \mathchardef\@m=1000
    \mathchardef\@M=10000
    \mathchardef\@MM=20000
    \countdef\m@ne=21\relax
    \m@ne=-1"
  );
  //======================================================================
  // TeX Book, Appendix B, p. 346
  TeX!(
    r"\countdef\count@=255
  \toksdef\toks@=0
  \skipdef\skip@=0
  \dimendef\dimen@=0
  \dimendef\dimen@i=1
  \dimendef\dimen@ii=2
  \count10=22
  \count11=9
  \count12=9
  \count13=9
  \count14=9
  \count15=9
  \count16=-1
  \count17=-1
  \count18=3
  \count19=0
  \count20=255
  \countdef\insc@unt=20
  \countdef\allocationnumber=21
  \countdef\m@ne=22 \m@ne=-1"
  );
  // Various \count's are set; should we?
  //======================================================================
  // TeX Book, Appendix B, p. 347
  DefPrimitive!("\\wlog{}", sub[(arg)] {
    eprintln!("{}", Expand!(arg));
    Ok(Vec::new())
  }, locked => true);
  // From plain.tex
  DefPrimitive!("\\newcount DefToken", sub[(name)] {
    DefRegister!(name, None, Number::new(0), allocate=>"\\count");
  });
  DefPrimitive!("\\newdimen DefToken", sub[(name)] {
    DefRegister!(name, None, Dimension::new(0), allocate=>"\\dimen");
  });
  DefPrimitive!("\\newskip DefToken", sub[(name)] {
    DefRegister!(name, None, Glue::new(0), allocate=>"\\skip");
  });
  DefPrimitive!("\\newmuskip DefToken", sub[(name)] {
    DefRegister!(name, None, MuGlue::new(0), allocate=>"\\muskip");
  });
  AssignValue!("allocated_boxes" => 0);
  DefPrimitive!("\\newbox DefToken", sub[(t)] {
    let n = lookup_int("allocated_boxes");
    AssignValue!("allocated_boxes" => n + 1, Some(Scope::Global));
    // Don't store a value — a newly allocated box is void.
    // classify_box returns "" for None, making \ifvoid true.
    DefRegister!(t, None, Number(n), readonly => true);
  });
  DefPrimitive!("\\newhelp DefToken {}", sub[(token,arg)] {
    state::assign_value(&token.to_string(), arg, None);
  });
  DefPrimitive!("\\newtoks DefToken", sub[(name)] {
    DefRegister!(name, None, Tokens!(), allocate=>"\\toks");
  });

  // the next 4 actually work by doing a \chardef instead of \countdef, etc.
  // which means they actually work quite differently
  DefPrimitive!("\\alloc@@ {}", sub[(atype)] {
    let c = s!("allocation @{}", atype);
    let n = state::lookup_int(&c);
    state::assign_value(&c, n + 1, Some(Scope::Global));
    state::assign_register("\\allocationnumber", Number::new(n).into(), Some(Scope::Global), Vec::new())?;
  });
  DefMacro!(
    "\\newread DefToken",
    r"\alloc@@{read}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newwrite DefToken",
    r"\alloc@@{write}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newfam DefToken",
    r"\alloc@@{fam}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newlanguage DefToken",
    r"\alloc@@{language}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\e@alloc{}{}{}{}{}{}",
    r"\global\advance#3\@ne
  \allocationnumber#3\relax
  \global#2#6\allocationnumber"
  );
  DefMacro!("\\alloc@{}{}{}{}", r"\e@alloc#2#3{\count1#1}#4\float@count");
  DefMacro!(
    "\\newread",
    r"\e@alloc\read \chardef{\count16}\m@ne\sixt@@n"
  );
  DefMacro!(
    "\\newwrite",
    r"\e@alloc\write
                  {\ifnum\allocationnumber=18
                      \advance\count17\@ne
                      \allocationnumber\count17 %
                    \fi
                    \global\chardef}%
                    {\count17}%
                    \m@ne
                    {128}"
  );

  // This implementation is quite wrong
  DefPrimitive!("\\newinsert Token", sub[(t)] {
    DefRegister!(t, None, Number::new(0));
  });
  // \alloc@, \ch@ck
  DefMacro!("\\ch@ck{}{}{}", None, locked => true);

  // TeX plain uses \newdimen, etc. for these.
  // Is there any advantage to that?
  // note: rust complains about the 16_383.99999 having excessive precision, hence simplifying
  DefRegister!("\\maxdimen", Dimension::new_f64(16383.99999 * UNITY_F64));
  DefRegister!("\\hideskip", Glue!("-1000pt plus 1fill"));
  DefRegister!("\\centering", Glue!("0pt plus 1000pt minus 1000pt"));
  DefRegister!("\\p@", Dimension::new(UNITY));
  DefRegister!("\\z@", Dimension::new(0));
  DefRegister!("\\z@skip", Glue::new(0));

  // Spacing stuff
  DefConstructor!("\\@", "");

  // First approximation. till I figure out \newbox
  TeX!(r"\newbox\voidb@x");

  //======================================================================
  // TeX Book, Appendix B, p. 348

  DefMacro!("\\newif DefToken", sub[(cs)] {
    def_conditional(cs, None,None, ConditionalOptions::default())
  });

  // See the section Registers & Parameters, above for setting default values.
  //======================================================================
  // TeX Book, Appendix B, p. 349
  // See the section Registers & Parameters, above for setting default values.

  // These are originally defined with \newskip, etc
  DefRegister!("\\smallskipamount", Glue!("3pt plus1pt minus1pt"));
  DefRegister!("\\medskipamount", Glue!("6pt plus2pt minus2pt"));
  DefRegister!("\\bigskipamount", Glue!("12pt plus4pt minus4pt"));
  DefRegister!("\\normalbaselineskip", Glue!("12pt"));
  DefRegister!("\\normallineskip", Glue!("1pt"));
  DefRegister!("\\normallineskiplimit", Dimension!("0pt"));
  DefRegister!("\\jot", Dimension!("3pt"));

  let jot_val = LookupRegister!("\\jot");

  DefRegister!("\\lx@default@jot", jot_val);
  DefRegister!("\\interdisplaylinepenalty", Number(100));
  DefRegister!("\\interfootnotelinepenalty", Number(100));

  DefMacro!("\\magstephalf", "1095");
  DefMacro!("\\magstep{}", sub[(mag)] {
    Explode!(match mag.to_string().as_str() {
      "0" => "1000",
      "1" => "1200",
      "2" => "1440",
      "3" => "1728",
      "4" => "2074",
      "5" => "2488",
      _ => ""
    })
  });

  //======================================================================
  // TeX Book, Appendix B, p. 350

  // Font stuff ...
  TeX!(
    r"\font\tenrm=cmr10
  \font\tenrm=cmr10
  \font\sevenrm=cmr7
  \font\fiverm=cmr5
  \font\teni=cmmi10
  \font\seveni=cmmi7
  \font\fivei=cmmi7
  \font\tensy=cmsy10
  \font\sevensy=cmsy7
  \font\fivesy=cmsy5
  \font\tenex=cmex10
  \font\tenbf=cmbx10
  \font\sevenbf=cmbx7
  \font\fivebf=cmbx5
  \font\tensl=cmsl10
  \font\tentt=cmtt10
  \font\tenit=cmti10
  \newfam\itfam
  \newfam\slfam
  \newfam\bffam
  \newfam\ttfam
 \textfont0=\tenrm\scriptfont0=\sevenrm\scriptscriptfont0=\fiverm
 \textfont1=\teni\scriptfont1=\seveni\scriptscriptfont1=\fivei
 \textfont2=\tensy\scriptfont2=\sevensy\scriptscriptfont2=\fivesy
 \textfont3=\tenex"
  );

  // Note: \newfam in math should be font switching(?)

  //======================================================================
  // TeX Book, Appendix B, p. 351

  // Old style font styles.
  // The trick is to create an empty Whatsit preserved till assimilation (for reversion'ing)
  // but to change the current font used in boxes.
  // (some of these were defined on different pages? or even latex...)
  Tag!("ltx:text", auto_open => true, auto_close => true);

  // Note that these, unlike \rmfamily, should set the other attributes to the defaults!
  DefPrimitive!("\\rm", None,
    font => {family => "serif", series => "medium", shape => "upright"});
  DefPrimitive!("\\sf", None,
    font => {family => "sansserif", series => "medium", shape => "upright"});
  DefPrimitive!("\\bf", None,
    font => {series => "bold", family => "serif", shape => "upright"});
  DefPrimitive!("\\it", None,
    font => {shape => "italic", family => "serif", series => "medium" });
  DefPrimitive!("\\tt", None,
    font => {family => "typewriter", series => "medium", shape => "upright" });
  // No effect in math for the following 2 ?
  DefPrimitive!("\\sl", None,
    font => {shape => "slanted", family => "serif", series => "medium" });
  DefPrimitive!("\\sc", None,
    font => {shape => "smallcaps", family => "serif", series => "medium" });
  // Perl: DefPrimitiveI('\cal', undef, sub {
  //   if (LookupValue('IN_MATH')) {
  //     MergeFont(family=>'caligraphic', series=>'medium', shape=>'upright', encoding=>'OMS');
  //     return Box(undef, undef, undef, T_CS('\cal')); } return; });
  DefPrimitive!("\\cal", {
    if state::lookup_bool("IN_MATH") {
      merge_font(fontmap!(family => "caligraphic", series => "medium",
        shape => "upright", encoding => "OMS"));
    }
    Tbox::new(arena::pin_static(""), None, None, Tokens::from(T_CS!("\\cal")),
      SymHashMap::default())
  });

  // Ideally, we should set these sizes from class files
  AssignValue!("NOMINAL_FONT_SIZE", 10);

  // Perl: \mit is \fam\itfam (plain.tex); LaTeXML doesn't override it.
  // In math mode, the default font is already italic, so \mit is effectively a no-op.
  // Use empty alias so the reversion is empty (Perl's \mit expands as a TeX assignment, no Box).
  DefMacro!("\\mit", None);

  DefPrimitive!("\\frenchspacing", None);
  DefPrimitive!("\\nonfrenchspacing", None);
  DefMacro!(
    "\\normalbaselines",
    r"\lineskip=\normallineskip\baselineskip=\normalbaselineskip\lineskiplimit=\normallineskiplimit"
  );
  DefMacro!(T_CS!("\\space"), None, T_SPACE!());
  DefMacro!(T_CS!("\\lq"), None, T_OTHER!("`"));
  DefMacro!(T_CS!("\\rq"), None, T_OTHER!("'"));
  Let!("\\empty", "\\lx@empty");
  DefMacro!("\\null", "\\hbox{}");
  Let!("\\bgroup", T_BEGIN!());
  Let!("\\egroup", T_END!());
  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");

  DefPrimitive!("\\endline", None);

  // Use \r for the newline from TeX!!!
  DefMacro!(T_CS!("\\\r"), None, T_CS!("\\ ")); // \<cr> == \<space> Interesting (see latex.ltx)
  Let!(&T_ACTIVE!('\r'), T_CS!("\\par")); // (or is this just LaTeX?)

  Let!("\\\t", "\\\r"); // \<tab> == \<space>, also

  //======================================================================
  // TeX Book, Appendix B, p. 352

  DefPrimitive!("\\obeyspaces", {
    AssignCatcode!(' ', Catcode::ACTIVE);
    Let!(&T_ACTIVE!(' '), T_CS!("\\space"));
  });
  // Curiously enough, " " (a space) is ALREADY defined to be the same as "\space"
  // EVEN before it is made active. (see p.380)
  Let!(&T_ACTIVE!(' '), T_CS!("\\space"));

  DefPrimitive!("\\obeylines", {
    AssignCatcode!('\r', Catcode::ACTIVE);
    Let!(&T_ACTIVE!('\r'), T_CS!("\\@break")); // More appropriate than \par, I think?
  });

  DefConstructor!("\\@break", "<ltx:break/>",
    properties => { stored_map!("isBreak" => true) });

  TeX!(
    r"
  \def\loop#1\repeat{\def\body{#1}\iterate}
  \def\iterate{\body \let\next=\iterate \else\let\next=\relax\fi \next}
  \let\repeat=\fi
  "
  );

  DefPrimitive!("\\enskip", {
    Tbox::new(
      arena::pin_static("\u{2002}"),
      None,
      None,
      Tokens!(T_CS!("\\enskip")),
      stored_map!("name" => "enskip", "width" => Dimension::from_str("0.5em")?,
      "isSpace"=>true),
    )
  });

  DefPrimitive!("\\enspace", {
    Tbox::new(
      arena::pin_static("\u{2002}"),
      None,
      None,
      Tokens!(T_CS!("\\enspace")),
      stored_map!("name" => "enskip", "width" => Dimension::from_str("0.5em")?,
        "isSpace"=>true),
    )
  });

  DefPrimitive!("\\quad", {
    Tbox::new(
      arena::pin_static("\u{2003}"),
      None,
      None,
      Tokens!(T_CS!("\\quad")),
      stored_map!("name" => "quad", "width" => Dimension::from_str("1em")?,
        "isSpace"=>true),
    )
  });

  // Conceivably should be treated as punctuation! (but maybe even \quad should !?!)
  DefPrimitive!("\\qquad", {
    Tbox::new(
      arena::pin_static("\u{2003}\u{2003}"),
      None,
      None,
      Tokens!(T_CS!("\\qquad")),
      stored_map!("name" => "qquad", "width" => Dimension::from_str("2em")?,
        "isSpace"=>true),
    )
  });

  DefPrimitive!("\\thinspace", {
    Tbox::new(
      arena::pin_static("\u{2009}"),
      None,
      None,
      Tokens!(T_CS!("\\thinspace")),
      stored_map!("name" => "thinspace", "width" => Dimension::from_str("0.16667em")?,
        "isSpace"=>true),
    )
  });

  DefPrimitive!("\\negthinspace", {
    Tbox::new(
      *EMPTY_SYM,
      None,
      None,
      Tokens!(T_CS!("\\negthinspace")),
      stored_map!("name" => "negthinspace", "width" => Dimension::from_str("-0.16667em")?,
        "isSpace"=>true),
    )
  });

  // Perl: plain_base.pool.ltxml L447
  DefPrimitive!("\\hglue Glue", sub[(length)] {
    let s = dimension_to_spaces(length);
    if s.is_empty() { return Ok(Vec::new()); }
    Tbox::new(arena::pin(&s), None, None,
      Invocation!(T_CS!("\\hglue"), vec![length.revert()?]),
      stored_map!("name" => "hglue", "width" => length, "isSpace" => true))
  });
  DefPrimitive!("\\vglue Glue", None);
  DefPrimitive!("\\topglue", None);
  // Perl: DefMacroI('\nointerlineskip',undef,'\prevdepth-1000\p@');
  DefMacro!("\\nointerlineskip", r"\prevdepth-1000\p@");
  // Perl: DefMacroI('\offinterlineskip',undef, '\baselineskip-1000\p@\lineskip\z@ \lineskiplimit\maxdimen');
  DefMacro!("\\offinterlineskip", r"\baselineskip-1000\p@\lineskip\z@ \lineskiplimit\maxdimen");

  DefMacro!("\\smallskip", "\\vskip\\smallskipamount");
  DefMacro!("\\medskip", "\\vskip\\medskipamount");
  DefMacro!("\\bigskip", "\\vskip\\bigskipamount");

  //======================================================================
  // TeX Book, Appendix B, p. 353

  DefPrimitive!("\\break", None);
  DefPrimitive!("\\nobreak", None);
  DefPrimitive!("\\allowbreak", None);
  DefPrimitive!("\\nobreakspace", {
    Tbox::new(
      arena::pin_static("\u{00A0}"),
      None,
      None,
      Tokens!(T_ACTIVE!('~')),
      stored_map!("isSpace" => true,
      "width" => Dimension::from_str("0.333em")?),
    )
  });
  DefMacro!(T_ACTIVE!('~'), None, "\\nobreakspace{}");

  DefMacro!("\\slash", "/");
  DefPrimitive!("\\filbreak", None);
  DefMacro!("\\goodbreak", "\\par");
  DefMacro!("\\eject", "\\par\\lx@newpage");
  Let!("\\newpage", "\\eject");

  DefConstructor!("\\LTX@newpage", "^<ltx:pagination role='newpage'/>",
  before_digest=>{
    after_assignment();
    Ok(Vec::new())
  });
  DefMacro!("\\supereject", "\\par\\lx@newpage");
  DefPrimitive!("\\removelastskip", None);
  DefMacro!("\\smallbreak", "\\par");
  DefMacro!("\\medbreak", "\\par");
  DefMacro!("\\bigbreak", "\\par");
  DefMacro!("\\line", "\\hbox to \\hsize");
  DefMacro!("\\leftline Undigested", r"\ltx@leftline{\hbox{#1}}");
  DefMacro!("\\rightline Undigested", r"\ltx@rightline{\hbox{#1}}");
  DefMacro!("\\centerline Undigested", r"\ltx@centerline{\hbox{#1}}");
  DefConstructor!("\\ltx@leftline{}", sub[doc,args,_props] {
      align_line(doc,args,"left")?;
    },
    alias => "\\leftline", bounded => true);
  DefConstructor!("\\ltx@rightline{}", sub[doc,args,_props] {
      align_line(doc,args,"right")?;
    },
    alias => "\\rightline", bounded => true);
  DefConstructor!("\\ltx@centerline{}", sub[doc,args,_props] {
      align_line(doc,args,"center")?;
    },
    alias => "\\centerline", bounded => true);

  // These should be 0 width, but perhaps also shifted?
  DefMacro!("\\llap{}", r"\hbox to 0pt{\hss#1}");
  DefMacro!("\\rlap{}", r"\hbox to 0pt{#1\hss}");
  DefMacro!("\\m@th", "\\mathsurround=0pt ");

  // \strutbox
  DefMacro!("\\strut", None);
  TeX!("\\newbox\\strutbox");
  //======================================================================
  // TeX Book, Appendix B. p. 354

  // TODO: Not yet done!!
  // tabbing stuff!!!

  DefMacro!("\\settabs", None);
  //======================================================================
  // TeX Book, Appendix B. p. 355

  DefPrimitive!("\\hang", None);

  // TODO: \item, \itemitem not done!
  // This could probably be adopted from LaTeX, if the <itemize> could auto-open
  // and close!
  DefMacro!("\\hang", r"\hangindent\parindent");
  DefMacro!("\\item", r"\par\hang\textindent");
  DefMacro!(
    "\\itemitem",
    r"\par\indent \hangindent2\parindent \textindent"
  );
  DefMacro!("\\textindent{}", r"\indent\llap{#1\enspace}\ignorespaces");
  DefMacro!(
    "\\narrower",
    r"\advance\leftskip by\parindent\advance\rightskip by\parindent"
  );

  // If folks start using plain TeX macros, and never load LaTeX.pool,
  // they might benefit from a ltx-plain.css?
  DefMacro!("\\beginsection Until:\\par", r"\@beginsection{{\bf #1}}");
  DefConstructor!(
    "\\@beginsection {}",
    "<ltx:section><ltx:title>#1</ltx:title>"
  );

  // POSSIBLY #1 is a name or reference number and  #2 is the theoremm TITLE
  //  If so, how do know when the theorem ends?
  DefMacro!(
    T_CS!("\\proclaim"),
    parse_def_parameters(&T_CS!("\\proclaim"), Tokenize!("#1. #2\\par"))?,
    Some(r"\@proclaim{{\bf #1}}{{\sl #2}}".into())
  );
  DefConstructor!("\\@proclaim{}{}",
  "<ltx:theorem><ltx:title font='#titlefont' _force_font='true' >#title</ltx:title>#2",
  after_construct => sub[doc,_args] { doc.maybe_close_element("ltx:theorem")?; },
  properties     => sub[args] {
    if let Some(ref title) = args[0] {
      Ok(stored_map!("title" => title, "titlefont" => title.get_font()?))
    } else { Ok(SymHashMap::default()) }
  });

  //======================================================================
  // TeX Book, Appendix B. p. 356

  DefPrimitive!("\\raggedright", None);
  DefPrimitive!("\\raggedleft", None); // this is actually LaTeX
  DefPrimitive!("\\ttraggedright", None);
  // Perl: sub { $_[0]->enterHorizontal; }  (plain_bootstrap.pool.ltxml line 43)
  DefPrimitive!("\\leavevmode", { enter_horizontal(); });
  DefMacro!(
    "\\mathhexbox{}{}{}",
    r##"\leavevmode\hbox{$\m@th \mathchar"#1#2#3$}"##
  );
  //----------------------------------------------------------------------
  //  Actually from LaTeX; Table 3.3, Greek, p.41
  //----------------------------------------------------------------------
  DefMath!("\\alpha", None, "\u{03B1}");
  DefMath!("\\beta", None, "\u{03B2}");
  DefMath!("\\gamma", None, "\u{03B3}");
  DefMath!("\\delta", None, "\u{03B4}");
  DefMath!("\\epsilon", None, "\u{03F5}");
  DefMath!("\\varepsilon", None, "\u{03B5}");
  DefMath!("\\zeta", None, "\u{03B6}");
  DefMath!("\\eta", None, "\u{03B7}");
  DefMath!("\\theta", None, "\u{03B8}");
  DefMath!("\\vartheta", None, "\u{03D1}");
  DefMath!("\\iota", None, "\u{03B9}");
  DefMath!("\\kappa", None, "\u{03BA}");
  DefMath!("\\lambda", None, "\u{03BB}");
  DefMath!("\\mu", None, "\u{03BC}");
  DefMath!("\\nu", None, "\u{03BD}");
  DefMath!("\\xi", None, "\u{03BE}");
  DefMath!("\\pi", None, "\u{03C0}");
  DefMath!("\\varpi", None, "\u{03D6}");
  DefMath!("\\rho", None, "\u{03C1}");
  DefMath!("\\varrho", None, "\u{03F1}");
  DefMath!("\\sigma", None, "\u{03C3}");
  DefMath!("\\varsigma", None, "\u{03C2}");
  DefMath!("\\tau", None, "\u{03C4}");
  DefMath!("\\upsilon", None, "\u{03C5}");
  DefMath!("\\phi", None, "\u{03D5}");
  DefMath!("\\varphi", None, "\u{03C6}");
  DefMath!("\\chi", None, "\u{03C7}");
  DefMath!("\\psi", None, "\u{03C8}");
  DefMath!("\\omega", None, "\u{03C9}");
  DefMath!("\\Gamma", None, "\u{0393}");
  DefMath!("\\Delta", None, "\u{0394}");
  DefMath!("\\Theta", None, "\u{0398}");
  DefMath!("\\Lambda", None, "\u{039B}");
  DefMath!("\\Xi", None, "\u{039E}");
  DefMath!("\\Pi", None, "\u{03A0}");
  DefMath!("\\Sigma", None, "\u{03A3}");
  DefMath!("\\Upsilon", None, "\u{03A5}");
  DefMath!("\\Phi", None, "\u{03A6}");
  DefMath!("\\Psi", None, "\u{03A8}");
  DefMath!("\\Omega", None, "\u{03A9}");

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.2. Non-English Symbols, p.39

  // The following shouldn't appear in math.
  DefPrimitive!("\\OE", "\u{0152}"); // LATIN CAPITAL LIGATURE OE
  DefPrimitive!("\\oe", "\u{0153}"); // LATIN SMALL LIGATURE OE
  DefPrimitive!("\\AE", "\u{00C6}"); // LATIN CAPITAL LETTER AE
  DefPrimitive!("\\ae", "\u{00E6}"); // LATIN SMALL LETTER AE
  DefPrimitive!("\\AA", "\u{00C5}"); // LATIN CAPITAL LETTER A WITH RING ABOVE
  DefPrimitive!("\\aa", "\u{00E5}"); // LATIN SMALL LETTER A WITH RING ABOVE
  DefPrimitive!("\\O", "\u{00D8}"); // LATIN CAPITAL LETTER O WITH STROKE
  DefPrimitive!("\\o", "\u{00F8}"); // LATIN SMALL LETTER O WITH STROKE
  DefPrimitive!("\\L", "\u{0141}"); // LATIN CAPITAL LETTER L WITH STROKE
  DefPrimitive!("\\l", "\u{0142}"); // LATIN SMALL LETTER L WITH STROKE
  DefPrimitive!("\\ss", "\u{00DF}"); // LATIN SMALL LETTER SHARP S

  // apparently the rest can appear in math.
  DefPrimitive!("\\lx@sectionsign",   "\u{00a7}", alias=>"\\S"); // SECTION SIGN
  DefPrimitive!("\\lx@paragraphsign", "\u{00B6}", alias=>"\\P"); // PILCROW SIGN
  DefMacro!("\\S", "\\lx@sectionsign");
  DefMacro!("\\P", "\\lx@paragraphsign");
  DefPrimitive!("\\dag", "\u{2020}"); // DAGGER
  DefPrimitive!("\\ddag", "\u{2021}"); // DOUBLE DAGGER
  DefPrimitive!("\\copyright", "\u{00A9}"); // COPYRIGHT SIGN
  DefPrimitive!("\\pounds", "\u{00A3}"); // POUND SIGN

  //======================================================================
  // Specific accents (see TeX-Character)
  //----------------------------------------------------------------------

  DefAccent!("\\`", '\u{0300}', "\u{0060}"); // COMBINING GRAVE ACCENT & GRAVE ACCENT
  DefAccent!("\\'", '\u{0301}', "\u{00B4}"); // COMBINING ACUTE ACCENT & ACUTE ACCENT
  DefAccent!("\\^", '\u{0302}', "\u{02C6}"); // COMBINING CIRCUMFLEX ACCENT & MODIFIER LETTER CIRCUMFLEX ACCENT
  DefAccent!("\\\"", '\u{0308}', "\u{00A8}"); // COMBINING DIAERESIS & DIAERESIS
  DefAccent!("\\~", '\u{0303}', "\u{02DC}"); // COMBINING TILDE & SMALL TILDE
  DefAccent!("\\=", '\u{0304}', "\u{00AF}"); // COMBINING MACRON & MACRON
  DefAccent!("\\.", '\u{0307}', "\u{02D9}"); // COMBINING DOT ABOVE & DOT ABOVE
  DefAccent!("\\u", '\u{0306}', "\u{02D8}"); // COMBINING BREVE & BREVE
  DefAccent!("\\v", '\u{030C}', "\u{02C7}"); // COMBINING CARON & CARON
  DefAccent!("\\@ringaccent", '\u{030A}', "\u{02DA}"); // COMBINING RING ABOVE & RING ABOVE
  DefAccent!("\\r", '\u{030A}', "\u{02DA}"); // COMBINING RING ABOVE & RING ABOVE
  DefAccent!("\\H", '\u{030B}', "\u{02DD}"); // COMBINING DOUBLE ACUTE ACCENT & non-combining
  DefAccent!("\\c", '\u{0327}', "\u{00B8}", below => true); // COMBINING CEDILLA & CEDILLA
  // NOTE: The next two get define for math, as well; See below
  DefAccent!("\\@text@daccent", '\u{0323}', ".",       below => true); // COMBINING DOT BELOW & DOT (?)
  DefAccent!("\\@text@baccent", '\u{0331}', "\u{00AF}", below => true); // COMBINING MACRON BELOW  & MACRON
  // COMBINING DOUBLE INVERTED BREVE & NBSP + combining char as standalone
  DefAccent!("\\t", '\u{0361}', "\u{00A0}\u{0361}");
  // this one"s actually defined in mathscinet.sty, but just stick it here!
  // COMBINING COMMA BELOW
  DefAccent!("\\lfhook", '\u{0326}', ",", below => true);

  // Perl TeX_Character.pool.ltxml: DefPrimitive('\accent Number', sub { ... })
  // \accent <number> <optional assignments> <character>; See TeX Book p.287
  // Reads a number (font position), then optional assignments, then a character.
  // Decodes the font position to a glyph, looks up accent data, applies accent.
  DefPrimitive!("\\accent Number", sub[(num)] {
    use crate::engine::tex_character;
    // 1. Decode the accent glyph from font position (BEFORE processing assignments)
    let n = num.value_of() as i32;
    let (glyph_opt, _font) = font_decode(n, None, None);

    // 2. Process optional assignments (Perl lines 117-123)
    //    <assignments>: (<prefix>) simple assignment or macro assignment
    //    <character> : letter, other, \char, \chardef token, \noboundary
    let mut assignments: Vec<Digested> = Vec::new();
    let mut last_token: Option<Token> = None;
    let mut last_defn: Option<Rc<dyn Definition>> = None;
    loop {
      let token_opt = gullet::read_x_non_space()?;
      let token = match token_opt {
        Some(t) => t,
        None => { break; }
      };
      let defn = if token.get_catcode().is_active_or_cs() {
        state::lookup_definition(&token)?.map(|d| d as Rc<dyn Definition>)
      } else {
        None
      };
      // Perl: isPrefix || isFontDef || (isRegister && !isCharDef)
      //       || token matches \def|\edef|\gdef|\xdef
      let is_assignment = if let Some(ref d) = defn {
        if d.is_prefix() || (
          d.is_register() && !matches!(d.register_type(), Some(RegisterType::CharDef))) {
          true
        } else {
          // Check isFontDef: lookupValue("fontinfo_<cs>")
          let cs_str = token.to_string();
          let fontinfo_key = s!("fontinfo_{}", cs_str);
          if state::lookup_value(&fontinfo_key).is_some() {
            true
          } else {
            // Check \def, \edef, \gdef, \xdef
            matches!(cs_str.as_str(), "\\def" | "\\edef" | "\\gdef" | "\\xdef")
          }
        }
      } else {
        false
      };
      if !is_assignment {
        last_token = Some(token);
        last_defn = defn;
        break;
      }
      // Process the assignment: invoke the token
      let digested = stomach::invoke_token(&token)?;
      assignments.extend(digested);
    }

    // 3. Read the base character token (Perl lines 126-134)
    let letter = if let Some(t) = last_token {
      let cc = t.get_catcode();
      if cc == Catcode::LETTER || cc == Catcode::OTHER
        || last_defn.as_ref().is_some_and(|d|
             matches!(d.register_type(), Some(RegisterType::CharDef))) {
        Tokens!(t)
      } else if t == T_CS!("\\char") {
        Tokens!(t, ExplodeText!(&gullet::read_number()?.to_string()))
      } else if t == T_CS!("\\noboundary") {
        Tokens!() // Treat as empty
      } else {
        gullet::unread_one(t);
        Tokens!()
      }
    } else {
      Tokens!()
    };

    // 4. Enter horizontal mode
    enter_horizontal();

    // 5. Apply accent (Perl lines 137-141)
    let accent_result: Vec<Digested> = if let Some(glyph_char) = glyph_opt {
      let glyph_str = glyph_char.to_string();
      if let Some(entry) = tex_character::unicode_accent(&glyph_str) {
        let mut rev_toks = vec![T_CS!("\\accent")];
        rev_toks.extend(ExplodeText!(&num.to_string()));
        rev_toks.push(T_OTHER!(" "));
        rev_toks.extend(letter.unlist_ref().iter().copied());
        let reversion = Tokens::new(rev_toks);
        let tbox = tex_character::apply_accent(
          letter, entry.combiner, entry.standalone, Some(reversion))?;
        vec![tbox.into()]
      } else {
        // Unknown accent: overlay glyph on letter using \lx@overlay
        let glyph_s = glyph_char.to_string();
        let overlay_toks = Tokens!(
          T_CS!("\\lx@overlay"), T_BEGIN!(),
          letter, T_END!(), T_BEGIN!(),
          ExplodeText!(&glyph_s), T_END!()
        );
        vec![stomach::digest(overlay_toks)?]
      }
    } else {
      // No glyph found: produce empty or just the letter
      let text = if letter.is_empty() {
        String::new()
      } else {
        letter.untex()
      };
      let tbox = Tbox::new(arena::pin(text), None, None, Tokens!(), SymHashMap::default());
      vec![tbox.into()]
    };

    // 6. Return assignments + accent result (Perl line 142)
    let mut result = assignments;
    result.extend(accent_result);
    Ok(result)
  });
  // Note that these two apparently work in Math? BUT the argument is treated as text!!!
  DefMacro!(
    "\\d{}",
    r"\ifmmode\@math@daccent{#1}\else\@text@daccent{#1}\fi"
  );
  DefMacro!(
    "\\b{}",
    r"\ifmmode\@math@baccent{#1}\else\@text@baccent{#1}\fi"
  );

  // Perl: DefConstructor('\@math@daccent {}', "...", mode => 'text', alias => '\d', ...)
  // Since mode => "text", the arg is always text, so textarg is always set.
  DefConstructor!("\\@math@daccent {}",
    "<ltx:XMApp><ltx:XMTok role='UNDERACCENT'>\u{22c5}</ltx:XMTok>\
     ?#textarg(<ltx:XMText>#textarg</ltx:XMText>)(<ltx:XMArg>#matharg</ltx:XMArg>)\
     </ltx:XMApp>",
    mode => "text", alias => "\\d",
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1).cloned() {
        whatsit.set_property("textarg", arg);
      }
    });
  // Perl: DefConstructor('\@math@baccent {}', "...", mode => 'text', alias => '\b', ...)
  DefConstructor!("\\@math@baccent {}",
    "<ltx:XMApp><ltx:XMTok role='UNDERACCENT'>\u{00AF}</ltx:XMTok>\
     ?#textarg(<ltx:XMText>#textarg</ltx:XMText>)(<ltx:XMArg>#matharg</ltx:XMArg>)\
     </ltx:XMApp>",
    mode => "text", alias => "\\b",
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1).cloned() {
        whatsit.set_property("textarg", arg);
      }
    });

  //======================================================================
  // TeX Book, Appendix B. p. 357
  // RIGHTWARDS ARROW??? a bit more explicitly
  DefMath!("\\to", None, "\u{2192}", role => "ARROW");

  // Perl: plain_constructs.pool.ltxml L86-91
  DefMacro!("\\hrulefill", "\\leaders\\hrule\\hfill");
  DefMacro!("\\dotfill", "\\leaders\\hbox{.}\\hfill");
  DefMath!("\\leftarrowfill", None, "\u{2190}", role => "ARROW", stretchy => true);
  DefMath!("\\rightarrowfill", None, "\u{2192}", role => "ARROW", stretchy => true);
  DefMath!("\\upbracefill", None, "\u{23DF}", role => "ARROW", stretchy => true);
  DefMath!("\\downbracefill", None, "\u{23DE}", role => "ARROW", stretchy => true);

  Let!("\\sp", T_SUPER!());
  Let!("\\sb", T_SUB!());

  // Perl: \, in math mode => \mskip\thinmuskip => Box(' ', ..., width => thinmuskip)
  DefPrimitive!("\\lx@thinmuskip", {
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\,")),
      stored_map!("name"  => "thinspace", "isSpace" => true,
      "width" => state::lookup_register("\\thinmuskip", Vec::new())?),
    )
  });
  DefPrimitive!("\\lx@thinspace", {
    Tbox::new(
      arena::pin_static("\u{2009}"),
      None,
      None,
      Tokens!(T_CS!("\\,")),
      stored_map!("name" => "thinspace", "width" => Dimension::from_str("0.16667em")?,
       "isSpace" => true),
    )
  });
  DefMacro!(
    "\\,",
    r"\ifmmode\lx@thinmuskip\else\lx@thinspace\fi",
    protected => true
  );

  DefPrimitive!("\\!", {
    Tbox::new(
      arena::pin_static("\u{200B}"),
      None,
      None,
      Tokens!(T_CS!("\\!")), // zero width space
      stored_map!("name"  => "negthinspace", "isSpace" => true,
      "width" => lookup_dimension("\\thinmuskip").unwrap().negate()),
    )
  });
  // Perl: \> and \; in math mode => Box(' ', ..., width => medmuskip/thickmuskip)
  DefPrimitive!("\\>", {
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\>")),
      stored_map!("name"  => "medspace", "isSpace" => true,
      "width" => state::lookup_register("\\medmuskip", Vec::new())?),
    )
  });
  DefPrimitive!("\\;", {
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\;")),
      stored_map!("name"  => "thickspace", "isSpace" => true,
      "width" => state::lookup_register("\\thickmuskip", Vec::new())?),
    )
  });

  Let!("\\:", "\\>");

  DefPrimitive!("\\\t", {
    Tbox::new(
      arena::pin_static("\u{00A0}"),
      None,
      None,
      Tokens!(T_CS!("\\\t")),
      stored_map!("isSpace" => true, "width" => Dimension::from_str("1em")?),
    )
  });

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.7. Miscellaneous Symbols, p.43
  //----------------------------------------------------------------------
  // Some should be differential operators, qualifiers, ...
  DefMath!("\\aleph", None, "\u{2135}");
  DefMath!("\\hbar",  None, "\u{210F}", role => "ID", meaning => "Planck-constant-over-2-pi");
  DefMath!("\\imath", None, "\u{0131}");
  DefMath!("\\jmath", None, "\u{0237}");
  DefMath!("\\ell", None, "\u{2113}");
  DefMath!("\\wp", None, "\u{2118}", meaning => "Weierstrass-p");
  DefMath!("\\Re", None, "\u{211C}", role    => "OPFUNCTION", meaning => "real-part");
  DefMath!("\\Im", None, "\u{2111}", role    => "OPFUNCTION", meaning => "imaginary-part");
  DefMath!("\\mho", None, "\u{2127}");

  DefMath!("\\prime",    None, "\u{2032}", role => "SUPOP",    locked  => true);
  DefMath!("\\emptyset", None, "\u{2205}", role => "ID",       meaning => "empty-set");
  DefMath!("\\nabla",    None, "\u{2207}", role => "OPERATOR");
  DefMath!("\\surd",     None, "\u{221A}", role => "OPERATOR", meaning => "square-root");
  DefMath!("\\top",      None, "\u{22A4}", role => "ADDOP",    meaning => "top");
  DefMath!("\\bot",      None, "\u{22A5}", role => "ADDOP",    meaning => "bottom");
  DefMath!("\\|", None, "\u{2225}", role => "VERTBAR", name => "||");
  // should get meaning => "parallel"to' when used as infix, but NOT when for OPEN|CLOSE
  DefMath!("\\angle", None, "\u{2220}");

  // NOTE: This is probably the wrong role.
  // Also, should probably carry info about Binding for OpenMath
  DefMath!("\\forall", None, "\u{2200}", role => "BIGOP",    meaning => "for-all");
  DefMath!("\\exists", None, "\u{2203}", role => "BIGOP",    meaning => "exists");
  DefMath!("\\neg",    None, "\u{00AC}",  role => "BIGOP", meaning => "not");
  DefMath!("\\lnot",   None, "\u{00AC}",  role => "BIGOP", meaning => "not");
  DefMath!("\\flat", None, "\u{266D}");
  DefMath!("\\natural", None, "\u{266E}");
  DefMath!("\\sharp", None, "\u{266F}");
  DefMath!("\\backslash", None, "\u{005C}", role => "MULOP");
  DefMath!("\\partial",   None, "\u{2202}", role => "DIFFOP", meaning => "partial-differential");

  DefMath!("\\infty", None, "\u{221E}", role => "ID", meaning => "infinity");
  DefMath!("\\Box", None, "\u{25A1}");
  DefMath!("\\Diamond", None, "\u{25C7}");
  DefMath!("\\triangle", None, "\u{25B3}");
  DefMath!("\\clubsuit", None, "\u{2663}");
  DefMath!("\\diamondsuit", None, "\u{2662}");
  DefMath!("\\heartsuit", None, "\u{2661}");
  DefMath!("\\spadesuit", None, "\u{2660}");

  DefMacro!("\\active@math@prime", {
    let mut sup = vec![T_CS!("\\prime")];
    // Collect up all ', convering to \prime
    let prime_token = T_OTHER!("\'");

    while gullet::if_next(prime_token)? {
      gullet::read_token()?;
      sup.push(T_CS!("\\prime"));
    }
    // Combine with any following superscript!
    // However, this is semantically screwed up!
    // We really need to set up separate superscripts, but at same level!
    if gullet::if_next(T_SUPER!())? {
      gullet::read_token()?;
      let arg = gullet::read_arg(ExpansionLevel::Off)?;
      let arg_tks = arg.unlist();
      sup.extend(arg_tks);
    }
    let mut activated = vec![T_SUPER!(), T_BEGIN!()];
    activated.extend(sup);
    activated.push(T_END!());
    activated
  },
  locked => true); // Only in math!
  assign_mathcode('\'', 0x8000u16, None);
  Let!("'", "\\active@math@prime");

  // Mathcode assignments from plain_base.pool.ltxml (Table 17.2 of TeX Book)
  // Punctuation and operators
  assign_mathcode('!', 0x5021u16, None);
  assign_mathcode('(', 0x4028u16, None);
  assign_mathcode(')', 0x5029u16, None);
  assign_mathcode('*', 0x2203u16, None);  // \ast (class 2 BINOP, family 2 OMS, pos 3)
  assign_mathcode('+', 0x202Bu16, None);
  assign_mathcode(',', 0x613Bu16, None);
  assign_mathcode('-', 0x2200u16, None);
  assign_mathcode('.', 0x013Au16, None);
  assign_mathcode('/', 0x013Du16, None);
  assign_mathcode(':', 0x303Au16, None);
  assign_mathcode('?', 0x503Fu16, None);  // class CLOSE (from plain.tex dump)
  assign_mathcode(';', 0x603Bu16, None);
  assign_mathcode('<', 0x313Cu16, None);
  assign_mathcode('=', 0x303Du16, None);
  assign_mathcode('>', 0x313Eu16, None);
  assign_mathcode('[', 0x405Bu16, None);
  assign_mathcode('\\', 0x026Eu16, None);
  assign_mathcode(']', 0x505Du16, None);
  assign_mathcode('{', 0x4266u16, None);
  assign_mathcode('|', 0x026Au16, None);
  assign_mathcode('}', 0x5267u16, None);
  // _ and ' are active (handled above/below)
  assign_mathcode('_', 0x8000u16, None);

  //----------------------------------------------------------------------
  // Table 3.8. Variable-sized Symbols (from math_common.pool.ltxml)
  // Perl: scriptpos => \&doScriptpos  — "mid" in display, "post" in inline
  //       mathstyle => \&doVariablesizeOp — "display" in display, "text" in inline
  // NOTE: \int and \oint have NO scriptpos (only mathstyle)
  //       \smallint has scriptpos but STATIC mathstyle => 'text'
  //----------------------------------------------------------------------
  DefMath!("\\smallint", None, "\u{222B}",
    meaning => "integral", role => "INTOP",
    dynamic_scriptpos => true, mathstyle => "text");
  // TODO: font => { size => 9 }
  DefMath!("\\sum",    None, "\u{2211}", role => "SUMOP", meaning => "sum",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\prod",   None, "\u{220F}", role => "SUMOP", meaning => "product",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\coprod", None, "\u{2210}", role => "SUMOP", meaning => "coproduct",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\int",    None, "\u{222B}", role => "INTOP", meaning => "integral",
    dynamic_mathstyle => true);
  DefMath!("\\oint",   None, "\u{222E}", role => "INTOP", meaning => "contour-integral",
    dynamic_mathstyle => true);
  DefMath!("\\bigcap",    None, "\u{22C2}", role => "SUMOP", meaning => "intersection",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigcup",    None, "\u{22C3}", role => "SUMOP", meaning => "union",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigsqcup",  None, "\u{2A06}", role => "SUMOP", meaning => "square-union",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigvee",    None, "\u{22C1}", role => "SUMOP", meaning => "or",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigwedge",  None, "\u{22C0}", role => "SUMOP", meaning => "and",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigodot",   None, "\u{2A00}", role => "SUMOP",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigotimes", None, "\u{2A02}", role => "SUMOP", meaning => "tensor-product",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigoplus",  None, "\u{2A01}", role => "SUMOP", meaning => "direct-sum",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\biguplus",  None, "\u{2A04}", role => "SUMOP", meaning => "symmetric-difference",
    dynamic_scriptpos => true, dynamic_mathstyle => true);

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.4. Binary Operation Symbols, p.42
  //----------------------------------------------------------------------
  DefMath!("\\pm",    None, "\u{00B1}",  role => "ADDOP", meaning => "plus-or-minus");
  DefMath!("\\mp",    None, "\u{2213}", role => "ADDOP", meaning => "minus-or-plus");
  DefMath!("\\times", None, "\u{00D7}",  role => "MULOP", meaning => "times");
  DefMath!("\\div",   None, "\u{00F7}",  role => "MULOP", meaning => "divide");
  DefMath!("\\ast",   None, "\u{2217}", role => "MULOP");
  DefMath!("\\star",  None, "\u{22C6}", role => "MULOP");
  DefMath!("\\circ",  None, "\u{2218}", role => "MULOP", meaning => "compose");
  DefMath!("\\bullet", None, "\u{2219}", role => "MULOP");
  DefMath!("\\cdot",   None, "\u{22C5}", role => "MULOP");
  ////  , meaning=>"inner-product");  that"s pushing it a bit far...

  // Need to classify set operations more carefully....
  DefMath!("\\cap", None, "\u{2229}", role => "ADDOP", meaning => "intersection");
  DefMath!("\\cup", None, "\u{222A}", role => "ADDOP", meaning => "union");
  DefMath!("\\uplus",    None, "\u{228E}", role => "ADDOP");
  DefMath!("\\sqcap",    None, "\u{2293}", role => "ADDOP", meaning => "square-intersection");
  DefMath!("\\sqcup",    None, "\u{2294}", role => "ADDOP", meaning => "square-union");
  DefMath!("\\vee",      None, "\u{2228}", role => "ADDOP", meaning => "or");
  DefMath!("\\lor",      None, "\u{2228}", role => "ADDOP", meaning => "or");
  DefMath!("\\wedge",    None, "\u{2227}", role => "ADDOP", meaning => "and");
  DefMath!("\\land",     None, "\u{2227}", role => "ADDOP", meaning => "and");
  DefMath!("\\setminus", None, "\u{2216}", role => "ADDOP", meaning => "set-minus");
  DefMath!("\\wr",       None, "\u{2240}", role => "MULOP");

  // Should this block be ADDOP or something else?
  DefMath!("\\diamond",         None, "\u{22C4}", role => "ADDOP");
  DefMath!("\\bigtriangleup",   None, "\u{25B3}", role => "ADDOP");
  DefMath!("\\bigtriangledown", None, "\u{25BD}", role => "ADDOP");
  DefMath!("\\triangleleft",    None, "\u{22B2}", role => "ADDOP");
  DefMath!("\\triangleright",   None, "\u{22B3}", role => "ADDOP");
  DefMath!("\\lhd",           None, "\u{22B2}", role => "ADDOP", meaning => "subgroup-of");
  DefMath!("\\rhd",           None, "\u{22B3}", role => "ADDOP", meaning => "contains-as-subgroup");
  DefMath!("\\unlhd", None, "\u{22B4}", role => "ADDOP", meaning => "subgroup-of-or-equals");
  DefMath!("\\unrhd", None, "\u{22B5}", role => "ADDOP",
    meaning => "contains-as-subgroup-or-equals");

  DefMath!("\\oplus",  None, "\u{2295}", role => "ADDOP", meaning => "direct-sum");
  DefMath!("\\ominus", None, "\u{2296}", role => "ADDOP", meaning => "symmetric-difference");
  DefMath!("\\otimes", None, "\u{2297}", role => "MULOP", meaning => "tensor-product");
  DefMath!("\\oslash", None, "\u{2298}", role => "MULOP");
  DefMath!("\\odot",   None, "\u{2299}", role => "MULOP", meaning => "direct-product");
  DefMath!("\\bigcirc", None, "\u{25CB}", role => "MULOP");
  DefMath!("\\dagger",  None, "\u{2020}", role => "MULOP");
  DefMath!("\\ddagger", None, "\u{2021}", role => "MULOP");
  DefMath!("\\amalg",   None, "\u{2210}", role => "MULOP", meaning => "coproduct");

  //----------------------------------------------------------------------
  // LaTeX; Table 3.5. Relation Symbols, p.43
  //----------------------------------------------------------------------
  DefMath!("\\leq",        None, "\u{2264}", role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\prec",       None, "\u{227A}", role => "RELOP", meaning => "precedes");
  DefMath!("\\preceq",     None, "\u{2AAF}", role => "RELOP", meaning => "precedes-or-equals");
  DefMath!("\\ll",         None, "\u{226A}", role => "RELOP", meaning => "much-less-than");
  DefMath!("\\subset",     None, "\u{2282}", role => "RELOP", meaning => "subset-of");
  DefMath!("\\subseteq",   None, "\u{2286}", role => "RELOP", meaning => "subset-of-or-equals");
  DefMath!("\\sqsubset",   None, "\u{228F}", role => "RELOP", meaning => "square-image-of");
  DefMath!("\\sqsubseteq", None, "\u{2291}", role => "RELOP",
    meaning => "square-image-of-or-equals");
  DefMath!("\\in",         None, "\u{2208}", role => "RELOP", meaning => "element-of");
  DefMath!("\\vdash", None, "\u{22A2}", role => "METARELOP", meaning => "proves");

  DefMath!("\\geq",      None, "\u{2265}", role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\succ",     None, "\u{227B}", role => "RELOP", meaning => "succeeds");
  DefMath!("\\succeq",   None, "\u{2AB0}", role => "RELOP", meaning => "succeeds-or-equals");
  DefMath!("\\gg",       None, "\u{226B}", role => "RELOP", meaning => "much-greater-than");
  DefMath!("\\supset",   None, "\u{2283}", role => "RELOP", meaning => "superset-of");
  DefMath!("\\supseteq", None, "\u{2287}", role => "RELOP", meaning => "superset-of-or-equals");
  DefMath!("\\sqsupset", None, "\u{2290}", role => "RELOP", meaning => "square-original-of");
  DefMath!("\\sqsupseteq", None, "\u{2292}", role => "RELOP",
    meaning => "square-original-of-or-equals");
  DefMath!("\\ni",    None, "\u{220B}", role => "RELOP",     meaning => "contains");
  DefMath!("\\dashv", None, "\u{22A3}", role => "METARELOP", meaning => "does-not-prove");

  // I have the impression think that "identical" is a stronger notion than "equivalence"
  // Note that the unicode here is called "Identical To",
  // and that the notion of "equivalent to" usually involves the tilde operator.
  DefMath!("\\equiv",  None, "\u{2261}", role => "RELOP", meaning => "equivalent-to");
  DefMath!("\\sim",    None, "\u{223C}", role => "RELOP", meaning => "similar-to");
  DefMath!("\\simeq",  None, "\u{2243}", role => "RELOP", meaning => "similar-to-or-equals");
  DefMath!("\\asymp",  None, "\u{224D}", role => "RELOP", meaning => "asymptotically-equals");
  DefMath!("\\approx", None, "\u{2248}", role => "RELOP", meaning => "approximately-equals");
  DefMath!("\\cong",   None, "\u{2245}", role => "RELOP", meaning => "approximately-equals");
  DefMath!("\\neq",    None, "\u{2260}", role => "RELOP", meaning => "not-equals");
  DefMath!("\\doteq",  None, "\u{2250}", role => "RELOP", meaning => "approaches-limit");
  DefMath!("\\notin",  None, "\u{2209}", role => "RELOP", meaning => "not-element-of");

  DefMath!("\\models", None, "\u{22A7}", role => "RELOP", meaning => "models");
  DefMath!("\\perp",   None, "\u{27C2}", role => "RELOP", meaning => "perpendicular-to");
  DefMath!("\\mid", None, "\u{2223}", role => "VERTBAR"); // DIVIDES (RELOP?) ?? well, sometimes...
  DefMath!("\\parallel", None, "\u{2225}", role => "VERTBAR", meaning => "parallel-to");
  DefMath!("\\bowtie",   None, "\u{22C8}", role => "RELOP"); // BOWTIE
  DefMath!("\\Join", None, "\u{2A1D}", role => "RELOP", meaning => "join");
  DefMath!("\\smile",  None, "\u{2323}", role => "RELOP"); // SMILE
  DefMath!("\\frown",  None, "\u{2322}", role => "RELOP"); // FROWN
  DefMath!("\\propto", None, "\u{221D}", role => "RELOP", meaning => "proportional-to");

  // TeX defines these as alternate names...
  Let!("\\le", "\\leq");
  Let!("\\ge", "\\geq");
  Let!("\\ne", "\\neq");
  // And it defines some others as alternate names, but they seem to
  // potentially imply slightly different meanings???  Leave them out for now..

  //----------------------------------------------------------------------
  // Not;  (Is fullwidth solidus appropriate for when \not appears in isolation?)
  DefMath!("\\not", None, "\u{FF0F}", role => "OPFUNCTION", meaning => "not");

  // For a \not operator that is followed by anything, concoct an appropriate not or cancelation.
  DefRewrite!(select =>
    "descendant-or-self::ltx:XMTok[text()='\u{FF0F}' and @meaning='not'][following-sibling::*]",
  select_count => 2,
  replace =>  sub[document, nodes] {
    // TODO: This argument low-level boilerplate is annoying
    // what is a good design pattern to "destructure" a Vec?
    // should it be another datastructure?
    let thing = nodes.pop().unwrap();
    let not_node = nodes.pop().unwrap();
    let text = model::with_node_qname(thing, |thing_str| match thing_str {
      "ltx:XMTok" => { thing.get_content() },
      _ => String::new()
    });
    // eprintln debug removed
    if text.chars().count() != 1 { // Not simple char token.
      // Wrap with a cancel op
      document.open_element("ltx:XMApp",
        Some(map!("_box" => not_node.to_hashable().to_string())), None)?;
      let mut strike = document.insert_math_token("",
        string_map!("role" => "ENCLOSE", "enclose" => "updiagonalstrike",
        "meaning" => "not", "_box" => not_node.to_hashable()), None)?;
      if let Some(id) = not_node.get_attribute_ns("id",XML_NS) {
        not_node.remove_attribute("xml:id")?;
        document.unrecord_id(&id);
        document.set_attribute(&mut strike, "xml:id", &id)?;
      }
      // Use append_tree to avoid DOM corruption from add_child on detached nodes
      let inner_children = vec![thing.clone()];
      let mut current = document.get_node().clone();
      document.append_tree(&mut current, inner_children)?;
      document.close_element("ltx:XMApp")?;
    } else {
      // For simple tokens, we'll modify the relevant content & attributes
      // [children removed, id's presumably ignorable]
      for mut child in thing.get_child_nodes() {
        child.unbind_node();
      }

      if let Some(meaning) = thing.get_attribute("meaning") {
        document.set_attribute(thing, "meaning",  &format!("not-{meaning}"))?; }
      if let Some(name) = thing.get_attribute("name") {
        document.set_attribute(thing, "name", &format!("not-{name}"))?; }
      else if !text.is_empty() {
        document.set_attribute(thing, "name", &format!("not-{text}"))?; }

      let known_c = MATH_CHAR_NEGATIONS.get(&text);
      let new : Cow<'_, str> = match known_c {
        Some(c) => Cow::Borrowed(c),
        None => Cow::Owned(text + "\u{0338}")
      };
      thing.append_text(&new)?;
      // Put the modified node back in using append_tree
      let inner_children = vec![thing.clone()];
      let mut current = document.get_node().clone();
      document.append_tree(&mut current, inner_children)?;
      // Since the <not> element is disappearing, if it had an id that was referenced...!?!?
      if let Some(id) = not_node.get_attribute_ns("id",XML_NS) {
        let idref_xpath = format!("descendant-or-self::ltx:XMRef[@idref='{id}']");
        for n in document.findnodes(&idref_xpath, None) {
          document.remove_node(n);
        }
      }   // ? Hopefully this is safe.
    }
  });

  //----------------------------------------------------------------------
  // \joinrel
  DefMath!("\\relbar", None, "-", role => "RELOP"); // ???
  DefMath!("\\Relbar", None, "=", role => "RELOP"); // ???

  // \joinrel is \mathrel{\mkern-3\mu}
  // Ah, but the Effect is to join 2 "relations" into one!
  // Perl: \joinrel joins 2 relations (e.g. \longrightarrow = \relbar\joinrel\rightarrow)
  // It pops left, digests right, then creates @@joinrel whatsit.
  // Stub: just read and discard the glue (the \mkern-3mu from \joinrel's definition),
  // then let the next token be digested normally.
  DefPrimitive!("\\joinrel", {
    gullet::skip_spaces()?;
    // Pop left item, read right item, but for now just return left unchanged
    if let Some(left) = pop_box_list() {
      vec![left]
    } else {
      Vec::new()
    }
  });

  DefConstructor!("\\@@joinrel{}{}", sub[document,args] {
    // Stub: just absorb both sides sequentially
    let left = args[0].as_ref().unwrap();
    let right = &args[1].as_ref().unwrap();
    document.absorb(left,None)?;
    document.absorb(right,None)?;
    // TODO: merge last 2 XMTok elements into a single joined token
    },
    reversion => "#1\\joinrel #2");

  //----------------------------------------------------------------------
  // LaTeX; Table 3.6. Arrow Symbols, p.43
  //----------------------------------------------------------------------
  // Arrows get treated somewhat like relations (or meta-relations),
  // but it's hard to associate any particular "meaning" to them.

  DefMath!("\\leftarrow",      "\u{2190}", role => "ARROW"); // LEFTWARDS ARROW
  DefMath!("\\Leftarrow",      "\u{21D0}", role => "ARROW"); // LEFTWARDS DOUBLE ARROW
  DefMath!("\\rightarrow",     "\u{2192}", role => "ARROW"); // RIGHTWARDS ARROW
  DefMath!("\\Rightarrow",     "\u{21D2}", role => "ARROW"); // RIGHTWARDS DOUBLE ARROW
  DefMath!("\\leftrightarrow", "\u{2194}", role => "METARELOP"); // LEFT RIGHT ARROW
  DefMath!("\\Leftrightarrow", "\u{21D4}", role => "METARELOP"); // LEFT RIGHT DOUBLE ARROW
  DefMath!("\\iff", "\u{21D4}", role => "METARELOP", meaning => "iff"); // LEFT RIGHT DOUBLE ARROW
  DefMath!("\\mapsto",        "\u{21A6}", role => "ARROW", meaning => "maps-to");
  DefMath!("\\hookleftarrow", "\u{21A9}", role => "ARROW"); // LEFTWARDS ARROW WITH HOOK
  DefMath!("\\leftharpoonup", "\u{21BC}", role => "ARROW"); // LEFTWARDS HARPOON WITH BARB UPWARDS
  DefMath!("\\leftharpoondown", "\u{21BD}", role => "ARROW"); // LEFTWARDS HARPOON WITH BARB DOWNWARDS
  DefMath!("\\rightleftharpoons", "\u{21CC}", role => "METARELOP"); // RIGHTWARDS HARPOON OVER LEFTWARDS HARPOON
  DefMath!("\\longleftarrow",      "\u{27F5}", role => "ARROW"); // LONG LEFTWARDS ARROW
  DefMath!("\\Longleftarrow",      "\u{27F8}", role => "ARROW"); // LONG LEFTWARDS DOUBLE ARROW
  DefMath!("\\longrightarrow",     "\u{27F6}", role => "ARROW"); // LONG RIGHTWARDS ARROW
  DefMath!("\\Longrightarrow",     "\u{27F9}", role => "ARROW"); // LONG RIGHTWARDS DOUBLE ARROW
  DefMath!("\\longleftrightarrow", "\u{27F7}", role => "METARELOP"); // LONG LEFT RIGHT ARROW
  DefMath!("\\Longleftrightarrow", "\u{27FA}", role => "METARELOP"); // LONG LEFT RIGHT DOUBLE ARROW
  DefMath!("\\longmapsto",     "\u{27FC}", role => "ARROW"); // LONG RIGHTWARDS ARROW FROM BAR
  DefMath!("\\hookrightarrow", "\u{21AA}", role => "ARROW"); // RIGHTWARDS ARROW WITH HOOK
  DefMath!("\\rightharpoonup", "\u{21C0}", role => "ARROW"); // RIGHTWARDS HARPOON WITH BARB UPWARDS
  DefMath!("\\rightharpoondown", "\u{21C1}", role => "ARROW"); // RIGHTWARDS HARPOON WITH BARB DOWNWARDS
  DefMath!("\\leadsto",          "\u{219D}", role => "ARROW", meaning => "leads-to");

  DefMath!("\\uparrow",     "\u{2191}", role => "ARROW"); // UPWARDS ARROW
  DefMath!("\\Uparrow",     "\u{21D1}", role => "ARROW"); // UPWARDS DOUBLE ARROW
  DefMath!("\\downarrow",   "\u{2193}", role => "ARROW"); // DOWNWARDS ARROW
  DefMath!("\\Downarrow",   "\u{21D3}", role => "ARROW"); // DOWNWARDS DOUBLE ARROW
  DefMath!("\\updownarrow", "\u{2195}", role => "ARROW"); // UP DOWN ARROW
  DefMath!("\\Updownarrow", "\u{21D5}", role => "ARROW"); // UP DOWN DOUBLE ARROW
  DefMath!("\\nearrow",     "\u{2197}", role => "ARROW"); // NORTH EAST ARROW
  DefMath!("\\searrow",     "\u{2198}", role => "ARROW"); // SOUTH EAST ARROW
  DefMath!("\\swarrow",     "\u{2199}", role => "ARROW"); // SOUTH WEST ARROW
  DefMath!("\\nwarrow",     "\u{2196}", role => "ARROW"); // NORTH WEST ARROW

  // \mapstochar (3237), \lhook(312C), \rhook(312D)
  // These are really wrong; I can't find the right Unicode Glyphs.
  // These are only fragments intended to be assembled into meaningful(?) symbols.
  DefMath!("\\mapstochar", "\u{2E20}"); // TeX 3237
  DefMath!("\\lhook", "\u{2E26}"); // TeX 312C
  DefMath!("\\rhook", "\u{2E27}"); // TeX 312D

  //======================================================================
  // TeX Book, Appendix B. p. 359

  // Ah, since \ldots can appear in text and math....
  DefMacro!("\\ldots", "\\lx@ldots");
  DefConstructor!(
    "\\vdots",
    "?#isMath(<ltx:XMTok name='vdots' font='#font' role='ID'>\u{22EE}</ltx:XMTok>)(\u{22EE})",
    properties => {
      if lookup_bool("IN_MATH") {
        Ok(stored_map!("font" => lookup_font().unwrap().merge(
          fontmap!(family => "serif", series => "medium", shape => "upright")
            .specialize("\u{22EE}"))))
      } else {
        Ok(SymHashMap::default())
      }
    });
  //                   # But not these!
  // Design note: Perl LaTeXML uses role 'ID' for \cdots, but latexml-oxide intentionally uses
  // 'ELIDEOP' to enable dedicated grammar rules (e.g. `term mulop tight_term elideop`) that
  // produce better-structured math parse trees for elision patterns like a⋅b⋅c⋯.
  DefMath!("\\cdots", None, "\u{22EF}", role => "ELIDEOP"); // MIDLINE HORIZONTAL ELLIPSIS
  DefMath!("\\ddots", None, "\u{22F1}", role => "ID"); // DOWN RIGHT DIAGONAL ELLIPSIS
  DefMath!("\\colon", None, ":",        role => "METARELOP"); // Seems like good default role
  //         # Note that amsmath redefines \dots to be `smart'.
  //         # Aha, also can be in text...
  DefConstructor!(
    "\\dots",
    "?#isMath(<ltx:XMTok name='dots' font='#font' role='ID'>\u{2026}</ltx:XMTok>)(\u{2026})",
    sizer      => "\u{2026}",
    properties => {
      if lookup_bool("IN_MATH") {
        Ok(stored_map!("font" => lookup_font().unwrap().merge(
          fontmap!(family => "serif", series => "medium", shape => "upright")
            .specialize("\u{2026}"))))
      } else {
        Ok(SymHashMap::default())
      }
    });

  // And while we're at it...

  // Pretest for XMath to keep from interpreting math that the DOM may not allow!!

  // Same design note as \cdots above: ELIDEOP is an intentional Rust-specific choice.
  DefMathLigature!("\u{22C5}\u{22C5}\u{22C5}", "\u{22EF}", role => "ELIDEOP", name => "cdots");

  DefLigature!(r"[.][.][.]", "\u{2026}", fontTest => sub[arg] {arg.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter" }); // ldots

  DefMathLigature!("...", "\u{2026}", role => "ID", name => "ldots");

  //----------------------------------------------------------------------
  // Math Accents.
  //----------------------------------------------------------------------
  // LaTeX; Table 3.11. Math Mode Accents, p.50.
  // Are these all TeX (or LaTeX)?
  // Note that most of these should NOT be stretchy, by default!
  DefMath!("\\hat Digested", "\u{005E}",
    operator_role => "OVERACCENT", operator_stretchy => false);
  DefMath!("\\check Digested", "\u{02C7}",
    operator_role => "OVERACCENT", operator_stretchy => false); // CARON
  DefMath!("\\breve Digested", "\u{02D8}", operator_role => "OVERACCENT"); // BREVE
  DefMath!("\\acute Digested", "\u{00B4}",  operator_role => "OVERACCENT"); // ACUTE ACCENT
  DefMath!("\\grave Digested", "\u{0060}",  operator_role => "OVERACCENT"); // GRAVE ACCENT
  DefMath!("\\tilde Digested", "\u{007E}",
    operator_role => "OVERACCENT", operator_stretchy => false); // TILDE
  DefMath!("\\bar Digested", "\u{00AF}",
    operator_role => "OVERACCENT", operator_stretchy => false); // MACRON
  DefMath!("\\vec Digested", "\u{2192}",
    operator_role => "OVERACCENT", operator_stretchy => false); // RIGHTWARDS ARROW
  DefMath!("\\dot Digested",      "\u{02D9}", operator_role => "OVERACCENT"); // DOT ABOVE
  DefMath!("\\ddot Digested",     "\u{00A8}",  operator_role => "OVERACCENT"); // DIAERESIS
  DefMath!("\\mathring Digested", "\u{030A}", operator_role => "OVERACCENT"); // COMBINING RING ABOVE
  DefMath!("\\widehat Digested", "\u{005E}", operator_role => "OVERACCENT"); // CIRCUMFLEX ACCENT [plain? also amsfonts]
  DefMath!("\\widetilde Digested", "\u{007E}", operator_role => "OVERACCENT"); // TILDE [plain? also amsfonts]
  // Perl: math_common.pool.ltxml lines 535-536
  // overbrace/underbrace canonical defs are \lx@math@* in tex_math.rs
  Let!("\\overbrace", "\\lx@math@overbrace");
  Let!("\\underbrace", "\\lx@math@underbrace");

  // NOTE that all the above accents REQUIRE math mode
  // EXCEPT underline, overrightarrow and overleftarrow!
  Let!("\\underbar", "\\underline"); // Will anyone notice?

  DefMacro!(
    "\\overrightarrow{}",
    r"\protect\ifmmode\lx@math@overrightarrow{#1}\else$\lx@math@overrightarrow{#1}$\fi"
  );
  DefMacro!(
    "\\overleftarrow{}",
    r"\protect\ifmmode\lx@math@overleftarrow{#1}\else$\lx@math@overleftarrow{#1}$\fi"
  );

  DefMacro!("\\skew{}{}{}", r"{#2{#3\mkern#1mu}\mkern-#1mu}{}"); // ignore the subtle spacing for now?
  //
  //----------------------------------------------------------------------
  // LaTeX; Table 3.10. Delimiters, p.47
  //----------------------------------------------------------------------
  // The meaning of OPEN/CLOSE tends to depend upon the pairing,
  // rather than the individual tokens.
  // This meaning is handled in MathParser (for now)
  DefMacro!("\\{", r"\ifmmode\lx@math@lbrace\else\lx@text@lbrace\fi", protected => true);
  DefMacro!("\\}", r"\ifmmode\lx@math@rbrace\else\lx@text@rbrace\fi", protected => true);
  DefMath!("\\lx@math@lbrace", None, "{", role => "OPEN",  stretchy => false, alias => "\\{");
  DefMath!("\\lx@math@rbrace", None, "}", role => "CLOSE", stretchy => false, alias => "\\}");
  DefPrimitive!("\\lx@text@lbrace", "{", alias => "\\{",
  font => { shape => "upright" }, bounded => true); // Since not DefMath!
  DefPrimitive!("\\lx@text@rbrace", "}", alias => "\\}",
  font => { shape => "upright" }, bounded => true); // Since not DefMath!
  Let!("\\lbrace", "\\{");
  Let!(&T_CS!("\\lbrack"), T_OTHER!("["));
  Let!("\\rbrace", "\\}");
  Let!(&T_CS!("\\rbrack"), T_OTHER!("]"));
  DefMath!("\\lceil",  None, "\u{2308}", role => "OPEN",  stretchy => false); // LEFT CEILING
  DefMath!("\\rceil",  None, "\u{2309}", role => "CLOSE", stretchy => false); // RIGHT CEILING
  DefMath!("\\lfloor", None, "\u{230A}", role => "OPEN",  stretchy => false); // LEFT FLOOR
  DefMath!("\\rfloor", None, "\u{230B}", role => "CLOSE", stretchy => false); // RIGHT FLOOR

  // Note: We should be using 27E8,27E9, which are "mathematical", not 2329,232A

  DefMath!("\\langle", None, "\u{27E8}", role => "OPEN", stretchy => false); // LEFT-POINTING ANGLE BRACKET
  DefMath!("\\rangle", None, "\u{27E9}", role => "CLOSE", stretchy => false); // RIGHT-POINTING ANGLE BRACKET

  // Not sure these should be defined here, or latex, or even latex compat mode.
  // Updated to use proper codepoints: U+27EE/27EF (MATHEMATICAL LEFT/RIGHT FLATTENED PARENTHESIS)
  // Perl commit "Lrgroup (#2762)": removed bold font, using dedicated Unicode codepoints.
  DefMath!("\\lgroup", None, "\u{27EE}", role => "OPEN",  stretchy => false);
  DefMath!("\\rgroup", None, "\u{27EF}", role => "CLOSE", stretchy => false);
  DefMath!("\\bracevert", None, "|", font => { series => "bold" }, role => "VERTBAR");

  // TeX marks some symbols as delimiters which can be used with \left,\right,
  // but many of which have different grammatical roles otherwise, eg. arrows, <, >.
  // Short of setting up TeX's complicated encoding machinery, I need an explicit
  // mapping.  Unfortunately, this doesn't (yet) support people declaring thier own delimiters!

  // With new treatment of Simple Symbols as just Box's with assigned attributes,
  // we're not getting whatsits, and so we're not looking them up the same way!!!
  // TEMPORARILY (?) hack the Delimiter map
  // foreach my $entry (values %DELIMITER_MAP) {
  //   $DELIMITER_MAP{ $$entry{char} } = $entry; }

  // sub lookup_delimiter {
  //   my ($delim) = @_;
  //   return $DELIMITER_MAP{$delim}; }

  // Perl PR#2596: TeXDelimiter reads like {} (for correct math XMTok digestion)
  // but reverts WITHOUT adding braces: \Big( not \Big{(}
  DefConstructor!("\\big TeXDelimiter",  "#1", bounded => true, font => { scale => 1.2 });
  DefConstructor!("\\Big TeXDelimiter",  "#1", bounded => true, font => { scale => 1.6 });
  DefConstructor!("\\bigg TeXDelimiter", "#1", bounded => true, font => { scale => 2.1 });
  DefConstructor!("\\Bigg TeXDelimiter", "#1", bounded => true, font => { scale => 2.6 });

  // sub addDelimiterRole {
  //   my ($document, $role) = @_;
  //   my $current = $document->getNode;
  //   my $delim   = $document->getLastChildElement($current) || $current;
  //   my $delim_role = (($delim && ($delim->nodeType == XML_ELEMENT_NODE) &&
  // $delim->getAttribute('role')) || '<none>');   # if there is some delimiter-like role on the
  // "delimiter", switch it, otherwise, leave it alone!   if ($delim && ($delim_role =~
  // /^(OPEN|MIDDLE|CLOSE|VERTBAR|<none>)$/)) {     ## Maybe we shouldn't switch VERTBAR ?
  //     ## The catch is that occasionally people use a single \Bigl (or whatever)
  //     ## where they should have used a \Big
  //     $document->setAttribute($delim, role => $role); }
  //   return; }

  // Sized delimiters with role assignment (l=OPEN, m=MIDDLE, r=CLOSE)
  DefConstructor!("\\bigl TeXDelimiter",  "#1", bounded => true, font => { size => 1.2 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\bigm TeXDelimiter",  "#1", bounded => true, font => { size => 1.2 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "MIDDLE")?; });
  DefConstructor!("\\bigr TeXDelimiter",  "#1", bounded => true, font => { size => 1.2 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  DefConstructor!("\\Bigl TeXDelimiter",  "#1", bounded => true, font => { size => 1.6 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\Bigm TeXDelimiter",  "#1", bounded => true, font => { size => 1.6 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "MIDDLE")?; });
  DefConstructor!("\\Bigr TeXDelimiter",  "#1", bounded => true, font => { size => 1.6 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  DefConstructor!("\\biggl TeXDelimiter", "#1", bounded => true, font => { size => 2.1 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\biggm TeXDelimiter", "#1", bounded => true, font => { size => 2.1 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "MIDDLE")?; });
  DefConstructor!("\\biggr TeXDelimiter", "#1", bounded => true, font => { size => 2.1 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  DefConstructor!("\\Biggl TeXDelimiter", "#1", bounded => true, font => { size => 2.6 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\Biggm TeXDelimiter", "#1", bounded => true, font => { size => 2.6 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "MIDDLE")?; });
  DefConstructor!("\\Biggr TeXDelimiter", "#1", bounded => true, font => { size => 2.6 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  Let!(&T_CS!("\\vert"), T_OTHER!("|"));
  Let!("\\Vert", "\\|");
  //======================================================================
  // TeX Book, Appendix B. p. 360
  //
  // Note that in TeX, all 4 args get digested(!)
  // and the choice is made when absorbing!

  DefMacro!(
    "\\mathpalette{}{}",
    r"\mathchoice{#1\displaystyle{#2}}{#1\textstyle{#2}}{#1\scriptstyle{#2}}{#1\scriptscriptstyle{#2}}"
  );

  // Perl: DefConstructor('\phantom{}', "?#isMath(...)(...)", properties => {isSpace=>1}, afterDigest => ...)
  DefConstructor!(
    "\\phantom{}",
    "?#isMath(<ltx:XMHint width='#width' height='#height' depth='#depth' name='phantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)",
    properties => { stored_map!("isSpace" => true) },
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg_mut(1) {
        let (w, h, d, _, _, _) = arg.get_size(None)?;
        whatsit.set_property("width", Stored::Dimension(w));
        whatsit.set_property("height", Stored::Dimension(h));
        whatsit.set_property("depth", Stored::Dimension(d));
      }
    });

  DefConstructor!(
    "\\hphantom{}",
    "?#isMath(<ltx:XMHint width='#width' name='hphantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)",
    properties => { stored_map!("isSpace" => true) },
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg_mut(1) {
        let (w, h, d, _, _, _) = arg.get_size(None)?;
        whatsit.set_property("width", Stored::Dimension(w));
        whatsit.set_property("height", Stored::Dimension(h));
        whatsit.set_property("depth", Stored::Dimension(d));
      }
    });

  DefConstructor!(
    "\\vphantom{}",
    "?#isMath(<ltx:XMHint height='#height' depth='#depth' name='vphantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)",
    properties => { stored_map!("isSpace" => true) },
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg_mut(1) {
        let (w, h, d, _, _, _) = arg.get_size(None)?;
        whatsit.set_property("width", Stored::Dimension(w));
        whatsit.set_property("height", Stored::Dimension(h));
        whatsit.set_property("depth", Stored::Dimension(d));
      }
    });

  DefConstructor!("\\mathstrut", "?#isMath(<ltx:XMHint name='mathstrut'/>)()",
    properties => { stored_map!("isSpace" => true) });
  DefConstructor!("\\smash{}", "#1"); // well, what?

  //======================================================================
  // TeX Book, Appendix B. p. 361

  // This is actually LaTeX's definition, but let's just do it this way.
  DefConstructor!(
    "\\sqrt OptionalInScriptStyle Digested",
    "?#1(<ltx:XMApp><ltx:XMTok meaning='nth-root'/>\
    <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>)\
    (<ltx:XMApp><ltx:XMTok meaning='square-root'/><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>)"
  );

  DefParameterType!(ScriptStyleUntil, sub[_inner,until] {
    gullet::read_until(&until[0]) },
  before_digest => {
    bgroup();
    MergeFont!(mathstyle => "script");
  },
  after_digest => {
    egroup()?; },
  reversion => sub[args,_inner,_extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(args).revert(), T_END!())) });

  DefConstructor!("\\root ScriptStyleUntil:\\of {}",
    "<ltx:XMApp><ltx:XMTok meaning='nth-root'/>\
      <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg>\
      </ltx:XMApp>",
    reversion => "\\root #1 \\of {#2}");

  //----------------------------------------------------------------------
  // LaTeX; Table 3.9. Log-like Functions, p.44.
  //----------------------------------------------------------------------
  // NOTE: Classifying some as TRIGFUNCTION might clarify 'pi' ambiguities ?
  DefMath!("\\arccos", None, "arccos", role => "OPFUNCTION", meaning => "inverse-cosine");
  DefMath!("\\arcsin", None, "arcsin", role => "OPFUNCTION", meaning => "inverse-sine");
  DefMath!("\\arctan", None, "arctan", role => "OPFUNCTION", meaning => "inverse-tangent");
  DefMath!("\\arg",    None, "arg",    role => "OPFUNCTION", meaning => "argument");

  DefMath!("\\cos",  None, "cos",  role => "TRIGFUNCTION", meaning => "cosine");
  DefMath!("\\cosh", None, "cosh", role => "TRIGFUNCTION", meaning => "hyperbolic-cosine");
  DefMath!("\\cot",  None, "cot",  role => "TRIGFUNCTION", meaning => "cotangent");
  DefMath!("\\coth", None, "coth", role => "TRIGFUNCTION", meaning => "hyperbolic-cotangent");

  DefMath!("\\csc", None, "csc", role => "TRIGFUNCTION", meaning => "cosecant");
  DefMath!("\\deg", None, "deg", role => "OPFUNCTION",   meaning => "degree");
  DefMath!("\\det", None, "det", role => "LIMITOP", meaning => "determinant",

  ); //TODO: scriptpos => \&doScriptpos);
  DefMath!("\\dim", None, "dim", role => "LIMITOP", meaning => "dimension");

  DefMath!("\\exp", None, "exp", role => "OPFUNCTION", meaning => "exponential");
  DefMath!("\\gcd", None, "gcd", role => "OPFUNCTION", meaning => "gcd",

  ); //TODO: scriptpos => \&doScriptpos);
  DefMath!("\\hom", None, "hom", role => "OPFUNCTION");
  DefMath!("\\inf", None, "inf", role => "LIMITOP", meaning => "infimum",

  ); //TODO: scriptpos => \&doScriptpos);

  DefMath!("\\ker", None, "ker", role => "OPFUNCTION", meaning => "kernel");
  DefMath!("\\lg", None, "lg", role => "OPFUNCTION");
  DefMath!("\\lim", None, "lim", role => "LIMITOP", meaning => "limit",

  ); //TODO: scriptpos => \&doScriptpos);
  DefMath!("\\liminf", None, "lim inf", role => "LIMITOP", meaning => "limit-infimum",
    scriptpos => "post"); // TODO: \&doScriptpos for display/text distinction
  DefMath!("\\limsup", None, "lim sup", role => "LIMITOP", meaning => "limit-supremum",
    scriptpos => "post"); // TODO: \&doScriptpos for display/text distinction
  DefMath!("\\ln",  None, "ln",  role => "OPFUNCTION", meaning => "natural-logarithm");
  DefMath!("\\log", None, "log", role => "OPFUNCTION", meaning => "logarithm");
  DefMath!("\\max", None, "max", role => "OPFUNCTION", meaning => "maximum",
    scriptpos => "post"); // TODO: \&doScriptpos for display/text distinction
  DefMath!("\\min", None, "min", role => "OPFUNCTION", meaning => "minimum",
    scriptpos => "post"); // TODO: \&doScriptpos for display/text distinction
  DefMath!("\\Pr",  None, "Pr",  role => "OPFUNCTION",
    scriptpos => "post"); // TODO: \&doScriptpos for display/text distinction
  DefMath!("\\sec", None, "sec", role => "TRIGFUNCTION", meaning   => "secant");
  DefMath!("\\sin", None, "sin", role => "TRIGFUNCTION", meaning   => "sine");

  DefMath!("\\sinh", None, "sinh", role => "TRIGFUNCTION", meaning => "hyperbolic-sine");
  DefMath!("\\sup", None, "sup", role => "LIMITOP", meaning => "supremum",
    scriptpos => "post"); // TODO: \&doScriptpos for display/text distinction
  DefMath!("\\tan",  None, "tan",  role => "TRIGFUNCTION", meaning => "tangent");
  DefMath!("\\tanh", None, "tanh", role => "TRIGFUNCTION", meaning => "hyperbolic-tangent");

  //----------------------------------------------------------------------
  // Modulo

  DefMath!("\\pmod{}", r"\;\;(\mathop{{\rm mod}} #1)", role => "MODIFIER"); //  , meaning=>"modulo");
  DefMath!("\\bmod", "mod", role => "MODIFIEROP", meaning => "modulo");

  //======================================================================
  // TeX Book, Appendix B. p. 362
  DefMacro!(
    "\\matrix{}",
    "\\lx@gen@plain@matrix{name=matrix,datameaning=matrix}{#1}"
  );

  DefMacro!(
    "\\bordermatrix{}", // Semantics?
    r"\lx@hack@bordermatrix{\lx@gen@plain@matrix{name=bordermatrix}{#1}}"
  );
  // HACK the newly created border matrix to add columns for the (spanned) parentheses!!!
  // Perl: adds empty XMCell columns for stretchy parens with rowspan
  DefConstructor!("\\lx@hack@bordermatrix{}", sub[document, args, _props] {
      let matrix = args[0].as_ref().unwrap();
      document.absorb(matrix, None)?;
      // DOM manipulation: add paren columns to the border matrix
      let marray = document.get_node().get_last_element_child();
      if let Some(marray) = marray {
        let rows = document.findnodes("ltx:XMRow", Some(&marray));
        let n = rows.len();
        if n >= 2 {
          // Add 2 empty cells to each row; move one to 2nd position
          for mut row in rows.iter().cloned() {
            let mut nopad_attrs = HashMap::default();
            nopad_attrs.insert("class".to_string(), "ltx_nopad".to_string());
            let mut cell1 = document.open_element_at(&mut row, "ltx:XMCell", Some(nopad_attrs.clone()), None)?;
            document.close_element_at(&mut cell1)?;
            let mut cell2 = document.open_element_at(&mut row, "ltx:XMCell", Some(nopad_attrs), None)?;
            document.close_element_at(&mut cell2)?;
            // Move cell2 (last child) to 2nd position (after first child)
            if let Some(mut first_child) = row.get_first_element_child() {
              cell2.unlink_node();
              first_child.add_next_sibling(&mut cell2).ok();
            }
          }
          // Set rowspan and add parens on 2nd and last columns of row 1
          if let Some(row1) = rows.get(1) {
            let cols: Vec<_> = row1.get_child_elements();
            if cols.len() >= 2 {
              let rowspan_str = (n - 1).to_string();
              // 2nd column (index 1): open paren
              let mut col1 = cols[1].clone();
              col1.set_attribute("rowspan", &rowspan_str).ok();
              col1.set_attribute("class", "ltx_nopad").ok();
              // Build XMWrap with open paren
              let mut wrap1 = document.open_element_at(&mut col1, "ltx:XMWrap", None, None)?;
              let mut open_attrs = HashMap::default();
              open_attrs.insert("role".to_string(), "OPEN".to_string());
              open_attrs.insert("stretchy".to_string(), "true".to_string());
              let mut open_tok = document.open_element_at(&mut wrap1, "ltx:XMTok", Some(open_attrs), None)?;
              open_tok.set_content("(");
              document.close_element_at(&mut open_tok)?;
              // Strut for height
              let mut strut = document.open_element_at(&mut wrap1, "ltx:XMTok", None, None)?;
              strut.set_content(" ");
              document.close_element_at(&mut strut)?;
              document.close_element_at(&mut wrap1)?;
              // Last column: close paren
              let mut coln = cols[cols.len() - 1].clone();
              coln.set_attribute("rowspan", &rowspan_str).ok();
              coln.set_attribute("class", "ltx_nopad").ok();
              let mut wrap2 = document.open_element_at(&mut coln, "ltx:XMWrap", None, None)?;
              let mut close_attrs = HashMap::default();
              close_attrs.insert("role".to_string(), "CLOSE".to_string());
              close_attrs.insert("stretchy".to_string(), "true".to_string());
              let mut close_tok = document.open_element_at(&mut wrap2, "ltx:XMTok", Some(close_attrs), None)?;
              close_tok.set_content(")");
              document.close_element_at(&mut close_tok)?;
              let mut strut2 = document.open_element_at(&mut wrap2, "ltx:XMTok", None, None)?;
              strut2.set_content(" ");
              document.close_element_at(&mut strut2)?;
              document.close_element_at(&mut wrap2)?;
            }
          }
        }
      }
    },
    reversion => "#1");
  // DefConstructor('\lx@hack@bordermatrix{}', sub {
  //     my ($document, $matrix) = @_;
  //     $document->absorb($matrix);
  //     my $marray = $document->getNode->lastChild;
  //     my @rows   = $document->findnodes('ltx:XMRow', $marray);
  //     my ($h, $d) = (10.0 * $UNITY, 0);    # 10pts.
  //                                          # Contrived, since $matrix may be a List or...
  //     my ($alignment) = grep { $_ } map { $_->getProperty('alignment') } $matrix->unlist;
  //     if ($alignment) {
  //       my $arrayh = $alignment->getHeight->ptValue;
  //       my ($row0, $row1) = $alignment->rows;    # What's row 0 ?
  //       $h = $$row1{y}->valueOf;
  //       $d = $h - $arrayh; }
  //     my $md = Dimension(-$d);
  //     $h = Dimension($h); $d = Dimension($d);

  //     foreach my $row (@rows) {                  # Add empty cells for 2nd & last colum
  //       $document->openElementAt($row, 'ltx:XMCell');
  //       $document->openElementAt($row, 'ltx:XMCell');
  //       $row->insertAfter($row->lastChild, $row->firstChild);    # Move to 2nd pos!
  //     }
  //     my @cols = element_nodes($rows[1]);
  //     my $col1 = $cols[1];
  //     my $coln = $cols[-1];
  //     my $n    = scalar(@rows) - 1;
  //     $col1->setAttribute(rowspan => $n);
  //     $coln->setAttribute(rowspan => $n);
  //     $document->appendTree($col1,
  //       ['ltx:XMWrap', { depth => $d },
  //         ['ltx:XMTok', { role   => 'OPEN', height  => 0, depth => $d, yoffset => $md }, '('],
  //         ['ltx:XMTok', { height => $h,     yoffset => $md }, ' ']]);    # Effectively, a strut
  //     $document->appendTree($coln,
  //       ['ltx:XMWrap', {},
  //         ['ltx:XMTok', { role   => 'CLOSE', height => 0, depth => $d, yoffset => $md }, ')'],
  //         ['ltx:XMTok', { height => $h, yoffset => $md }, ' ']]);
  //     return; },
  //   reversion => '#1');

  DefMacro!(
    "\\pmatrix{}",
    r"\lx@gen@plain@matrix{name=pmatrix,datameaning=matrix,left=\@left(,right=\@right)}{#1}"
  );

  // Note that 2nd column in \cases is in text mode!
  DefMacro!(
    "\\cases{}",
    r"\lx@gen@plain@cases{meaning=cases,left=\@left\{,conditionmode=text,style=\textstyle}{#1}"
  );

  //----------------------------------------------------------------------
  DefPrimitive!("\\openup Dimension", None);

  // What should this do? (needs to work with alignments..)
  // see https://www.tug.org/TUGboat/tb07-1/tb14beet.pdf
  // Perl: DefMacro('\displaylines{}', '\halign{\hbox to\displaywidth{...}\crcr#1\crcr}')
  DefMacro!(
    "\\displaylines{}",
    r"\halign{\hbox to\displaywidth{$\hfil\displaystyle##\hfil$}\crcr#1\crcr}"
  );

  DefMacro!(
    "\\eqalign{}",
    r"\@@eqalign{\lx@begin@alignment#1\lx@end@alignment}"
  );
  DefConstructor!("\\@@eqalign{}", "#1",
    reversion => "\\eqalign{#1}", bounded => true,
    before_digest => {
      use crate::engine::tex_tables::alignment_bindings;
      use latexml_core::alignment::template::{Align, TemplateConfig};
      use latexml_core::alignment::cell::Cell;
      let template = Template::new(TemplateConfig {
        columns: Some(vec![
          Cell { align: Some(Align::Right), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
        ]),
        ..TemplateConfig::default()
      });
      alignment_bindings(template, String::from("math"),
        SymHashMap::default(), string_map!("vattach" => "baseline"));
    });

  DefMacro!(
    "\\eqalignno{}",
    r"\@@eqalignno{\lx@begin@alignment#1\lx@end@alignment}"
  );
  DefConstructor!("\\@@eqalignno{}", "#1",
    reversion => "\\eqalignno{#1}", bounded => true,
    before_digest => {
      use crate::engine::tex_tables::alignment_bindings;
      use latexml_core::alignment::template::{Align, TemplateConfig};
      use latexml_core::alignment::cell::Cell;
      let template = Template::new(TemplateConfig {
        columns: Some(vec![
          Cell { align: Some(Align::Right), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
        ]),
        ..TemplateConfig::default()
      });
      alignment_bindings(template, String::from("math"),
        SymHashMap::default(), string_map!("vattach" => "baseline"));
    });

  DefMacro!(
    "\\leqalignno{}",
    r"\@@leqalignno{\lx@begin@alignment#1\lx@end@alignment}"
  );
  DefConstructor!("\\@@leqalignno{}", "#1",
    reversion => "\\leqalignno{#1}", bounded => true,
    before_digest => {
      use crate::engine::tex_tables::alignment_bindings;
      use latexml_core::alignment::template::{Align, TemplateConfig};
      use latexml_core::alignment::cell::Cell;
      let template = Template::new(TemplateConfig {
        columns: Some(vec![
          Cell { align: Some(Align::Right), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
          Cell { align: Some(Align::Left), ..Cell::default() },
        ]),
        ..TemplateConfig::default()
      });
      alignment_bindings(template, String::from("math"),
        SymHashMap::default(), string_map!("vattach" => "baseline"));
    });

  DefRegister!("\\pageno"   => Number::new(0));
  DefRegister!("\\headline" => Tokens!());
  DefRegister!("\\footline" => Tokens!());
  DefMacro!("\\folio", "1"); // What else?

  DefPrimitive!("\\nopagenumbers", None);
  DefMacro!("\\advancepageno", "\\advance\\pageno1\\relax");

  //======================================================================
  // TeX Book, Appendix B. p. 363
  DefPrimitive!("\\raggedbottom", None);
  DefPrimitive!("\\normalbottom", None);

  // if the mark is not simple, we add it to the content of the note
  // otherwise, to the attribute.
  DefConstructor!("\\footnote{}{}",
    "^<ltx:note role='footnote' ?#mark(mark='#mark')()>?#prenote(#prenote )()#2</ltx:note>",
    mode => "internal_vertical",
    before_digest => sub { neutralize_font(); },
    after_digest => sub[whatsit] {
      let mark_clone = whatsit.get_arg(1).cloned();
      if let Some(mark) = mark_clone {
        let mark_tks = mark.revert()?.unlist();
        let mut change = false;
        for token in mark_tks {
          if !matches!(token.get_catcode(), Catcode::LETTER | Catcode::SPACE | Catcode::OTHER) {
            change = true;
            break;
          }
        }
        whatsit.set_property(if change { "prenote" } else {"mark"}, mark);
      }
    }
  );

  // Until we can do the "v" properly:
  DefMacro!("\\vfootnote", "\\footnote");
  DefMacro!(
    "\\fo@t",
    r"\ifcat\bgroup\noexpand\next \let\next\f@@t  \else\let\next\f@t\fi \next"
  );
  DefMacro!("\\f@@t", r"\bgroup\aftergroup\@foot\let\next");
  DefMacro!("\\f@t{}", r"#1\@foot");
  DefMacro!("\\@foot", r"\strut\egroup");

  DefPrimitive!("\\footstrut", None);
  DefRegister!("\\footins" => Number::new(0));

  DefPrimitive!("\\topinsert", None);
  DefPrimitive!("\\midinsert", None);
  DefPrimitive!("\\pageinsert", None);
  DefPrimitive!("\\endinsert", None);
  // \topins ?

  //======================================================================
  // TeX Book, Appendix B. p. 364

  // Let's hope nobody is messing with the output routine...
  DefPrimitive!("\\footnoterule", None);

  //======================================================================
  // End of TeX Book definitions.
  //======================================================================

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Stuff that probably doesn't belong here (LaTeX? or nowhere?)
  //DefMacro('\vspace{}', '\vskip#1\relax');

  //======================================================================
  // In principle, <ltx:emph> is a nice markup for emphasized.
  // Unfortunately, TeX really just treats it as a font switch.
  // Something like:  \em et.al. \rm more stuff
  // works in TeX, but in our case, since there is no explicit {},
  // the <ltx:emph> stays open!  Ugh!
  // This could still be made to work, but merge font would
  // need to look at any open <ltx:emph>, and then somehow close it!
  DefPrimitive!("\\em", None,
  before_digest => {
    let font = LookupFont!().unwrap();
    let shape = font.get_shape().unwrap_or(&Cow::Borrowed(""));
    let shapevariant = if shape == "italic" { "normal" } else { "italic" };
    AssignValue!("font", font.merge(fontmap!(shape => shapevariant)), Some(Scope::Local));
  });

  // Change math font while still in text!
  // Perl: AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 1), 'local')
  DefPrimitive!("\\boldmath", None,
    before_digest => {
      let mf = state::lookup_mathfont().unwrap_or_else(|| Rc::new(Font::math_default()));
      let merged = mf.merge(Font { forcebold: Some(true), ..Font::default() });
      state::assign_value("mathfont", Stored::Font(Rc::new(merged)), Some(Scope::Local));
    },
    forbid_math => true);
  // Perl: AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 0), 'local')
  DefPrimitive!("\\unboldmath", None,
    before_digest => {
      let mf = state::lookup_mathfont().unwrap_or_else(|| Rc::new(Font::math_default()));
      let merged = mf.merge(Font { forcebold: Some(false), ..Font::default() });
      state::assign_value("mathfont", Stored::Font(Rc::new(merged)), Some(Scope::Local));
    },
    forbid_math => true);
});

fn non_typewriter(font: &Font) -> bool {
  font.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter"
}

fn non_typewriter_t1(font: &Font) -> bool {
  non_typewriter(font)
    && matches!(
      font
        .get_encoding()
        .unwrap_or(&Cow::Borrowed("OT1"))
        .as_ref(),
      "OT1" | "T1"
    )
}

fn align_line(document: &mut Document, line: &[Option<Digested>], alignment: &str) -> Result<()> {
  if document.is_openable("ltx:p") {
    let line_content = line.iter().filter_map(|c| c.as_ref()).collect();
    document.insert_element(
      "ltx:p",
      line_content,
      Some(string_map!(
      "class" => s!("ltx_align_{alignment}"))),
    )?;
  } else if document.is_openable("ltx:text") {
    let line_content = line.iter().filter_map(|c| c.as_ref()).collect();
    document.insert_element(
      "ltx:text",
      line_content,
      Some(string_map!(
      "class" => s!("ltx_align_{alignment}"))),
    )?;
    document.insert_element("ltx:break", Vec::new(), None)?;
  } else if let Some(Some(line_content)) = line.first() {
    document.absorb(line_content, None)?;
  }
  Ok(())
}
