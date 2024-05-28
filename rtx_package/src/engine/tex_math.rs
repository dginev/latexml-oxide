//! TeX Math
//! 
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Math Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  
  //======================================================================
  // NOT YET IMPLEMENTED !?!?!
  //----------------------------------------------------------------------
  // \radical                c  makes a radical atom from the delimiter (27-bit number) and the math field.
  // \muskipdef              c  creates a symbolic name for a \muskip register.
  // \muskip                 iq assigns <muglue> to a \muskip register.
  // \nonscript              c  ignores immediately following glue or kern in script and scriptscript styles.
  
  //======================================================================
  // The next two sections are the basic LaTeXML Infrastructure for math.
  // There are several internal control sequences which need to be renamed!
  //======================================================================
  
  // Decide whether we're going into or out of math, inline or display.
  Tag!("ltx:XMText", auto_open => true, auto_close => true);
  // This really should be T_MATH
  // and it should (or not) check for a second $ only if not in restricted horizontal mode!
  // (and then all the \lx@dollar@in@(text|math|normal)mode defns would not be needed.
  DefPrimitive!(T_CS!("\\lx@dollar@in@normalmode"), None, {
      let mut op = "\\lx@begin@inline@math";
      {
        let mode = state::lookup_string("MODE");
        Debug!("T_MATH primitive current mode: {:?}", mode);
        if mode == "display_math" {
          if gullet::if_next(T_MATH!())? {
            gullet::read_token()?;
            op = "\\lx@end@display@math";
          } else {
            // Avoid a Fatal, but we're likely in trouble.
            // Should we switch to text mode? (LaTeX normally wouldn't)
            // Did we miss something and would should have already been in text mode? Possibly...
            Error!(
              "expected",
              "$",
              "Missing $ closing display math.\nIgnoring; expect to be in wrong math/text mode."
            );
            op = "";
          }
        } else if mode == "inline_math" {
          op = "\\lx@end@inline@math";
        } else if gullet::if_next(T_MATH!())? {
          gullet::read_token()?;
          op = "\\lx@begin@display@math";
        }
      }
      if !op.is_empty() {
        Ok(stomach::invoke_token(&T_CS!(op))?)
      } else {
        Ok(Vec::new())
      }
    });
  // Let this be the default, conventional $
  Let!(T_MATH!(), T_CS!("\\lx@dollar@in@normalmode"));

  //======================================================================
  // Math mode in alignment
  // Special forms for $ appearing within alignments.
  // Note that $ within a math alignment (eg array environment),
  // switches to text mode! There's no $$ for display math.
  //
  // This is the "normal" case: $ appearing with an alignment that is in text mode.
  // It's just like regular $, except it doesn't look for $$ (no display math).
  DefPrimitive!("\\lx@dollar@in@textmode", {
    let mathcs = if lookup_bool("IN_MATH") { T_CS!("\\lx@end@inline@math") }
      else {T_CS!("\\lx@begin@inline@math") };
    stomach::invoke_token(&mathcs)
  });

  // This one is for $ appearing within an alignment that's already math.
  // This should switch to text mode (because it's balancing the hidden $
  // wrapping each alignment cell!!!!!!)
  // However, it should be like a normal $ if it's inside something like \mbox
  // that itself makes a text box!!!!!!
  // Thus, we need to know at what boxing level we started the last math or text.
  // This is all complicated by the need to know _how_ we got into or out of math mode!
  // Gawd, this is awful!
  // NOTE: Probably the most "Right" thing to do would be to process
  // alignments in text mode only (like TeX), sneaking $'s in where needed,
  // but then afterwards, morph them into math arrays?
  // This would be complicated by the need to hide these $ from untex.
  DefPrimitive!(T_CS!("\\lx@dollar@in@mathmode"), None, {
    let level = stomach::get_boxing_level();
    if lookup_int("MATH_ALIGN_$_BEGUN") == (level as i64) { // If we're begun making _something_ with $.
      let l = if lookup_bool("IN_MATH") { // But we're somehow in math?
        stomach::invoke_token(&T_CS!("\\lx@end@inline@math")) 
      } else {
        stomach::invoke_token(&T_CS!("\\lx@end@inmath@text"))
      };
      assign_value("MATH_ALIGN_$_BEGUN", 0, None); // Reset this AFTER finishing the something
      l
    } else {
      assign_value("MATH_ALIGN_$_BEGUN", level + 1, None); // Note that we've begun something
      if lookup_bool("IN_MATH") { // If we're "still" in math
        stomach::invoke_token(&T_CS!("\\lx@begin@inmath@text"))
      } else {
        stomach::invoke_token(&T_CS!("\\lx@begin@inline@math"))
      }
    } 
  });
  //======================================================================
  // For inserting (non-trivial?) text while in math mode
  DefConstructor!("\\lx@begin@inmath@text",
    "<ltx:XMText>#body</ltx:XMText>",
    // alias => T_MATH ? do we support that ?
    alias => "$", 
    before_digest => sub { stomach::begin_mode("text")?; },
    capture_body => true
  );
  DefConstructor!("\\lx@end@inmath@text", "", alias => "$",
    before_digest => sub { stomach::end_mode("text")?; });
  //======================================================================
  // Effectively these are the math hooks, redefine these to do what you want with math?
  DefConstructor!("\\lx@begin@display@math",
  "<ltx:equation>\
    <ltx:Math mode=\"display\">\
    <ltx:XMath>\
    #body\
    </ltx:XMath>\
    </ltx:Math>\
  </ltx:equation>",
    reversion         => Tokens!(T_MATH!(),T_MATH!()),
    before_digest => {
      begin_mode("display_math")?;
      // TODO:
      // if let Some(everymath_toks) = lookup_definition(T_CS!("\\everymath")).value_of().unlist() {
      //   gullet::unread(everymath_toks);
      // }
      // if let Some(everydisplay_toks) = lookup_definition(T_CS!("\\everydisplay")).value_of().unlist() {
      //   gullet::unread(everydisplay_toks);
      // }
    },
    capture_body  => true );

  DefConstructor!(T_CS!("\\lx@end@display@math"), None, None,
    reversion => Tokens!(T_MATH!(),T_MATH!()),
    before_digest => { end_mode("display_math")?; });

  DefConstructor!("\\lx@begin@inline@math",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    reversion    => Tokens!(T_MATH!()),
    before_digest => {
      begin_mode("inline_math")?;
      if let Some(RegisterValue::Tokens(everymath_toks)) = state::lookup_register("\\everymath", Vec::new())? {
        let everymath_toks = everymath_toks.unlist();
        if !everymath_toks.is_empty() {
          gullet::unread(Tokens::new(everymath_toks));
        }
      }
    },
    capture_body => true);
  DefConstructor!(T_CS!("\\lx@end@inline@math"), None, None,
    before_digest => { end_mode("inline_math")?; },
    reversion    => Tokens!(T_MATH!())
  );

  // Same as add_TeX, but add the code from the body of the object.
  Tag!("ltx:Math", after_close => sub[document, node] {
    if !node.has_attribute("tex") {
      // only do this once.

      let tex_opt = if let Some(ref tbox) = document.get_node_box(node) {
        if let Some(body) = tbox.get_body()? {
          set_dual_branch("presentation");
          let tex = body.untex()?;
          expire_dual_branch();
          set_dual_branch("content");
          let ctex = body.untex()?;
          expire_dual_branch();
          if ctex != tex {
            document.set_attribute(node, "content-tex", &ctex)?;
          }
          Some(tex)
        } else {
          None
        }
      } else {
        None
      };
      if let Some(tex_string) = tex_opt {
        document.set_attribute(node, "tex", &tex_string)?;
      }
    }
  });

  Tag!("ltx:Math", after_close => sub[document, node] {
    cleanup_math(document, node.clone())?;
  });
  
  //======================================================================
  // General
  //----------------------------------------------------------------------
  // \everydisplay         pt holds tokens inserted at the start of every switch to display math mode.
  // \everymath            pt holds tokens inserted at the start of every switch to math mode.
  DefRegister!("\\everymath", Tokens!());
  DefRegister!("\\everydisplay", Tokens!());

    
  // Almost like a register (and \countdef), but different...
  // (including the preassignment to \relax!)
  DefConstructor!("\\mathchar Number", "?#glyph(<ltx:XMTok role='#role'>#glyph</ltx:XMTok>)",
    sizer       => "#1",
    after_digest => sub[whatsit] {
      let n = whatsit.get_arg(1).unwrap().value_of();
      let (role_opt, glyph_opt) = decode_math_char(n as u16)?;
      if let Some(glyph) = glyph_opt {
        whatsit.set_property("glyph", glyph);
        whatsit.set_property("font", lookup_font().unwrap().specialize(&glyph.to_string()));
      }
      if let Some(role) = role_opt {
        whatsit.set_property("role", role);
      }
      Ok(Vec::new())
    }
  );

  DefConstructor!("\\delimiter Number",
  "?#glyph(?#isMath(<ltx:XMTok role='#role'>#glyph</ltx:XMTok>)(#glyph))",
  sizer       => "#glyph",
  after_digest => sub[whatsit] {
    let mut n = whatsit.get_arg(1).unwrap().value_of();
    n >>= 12;    // Ignore 3 rightmost digits and treat as \mathchar
    let (role_opt, glyph_opt) = decode_math_char(n as u16)?;
    if let Some(glyph) = glyph_opt {
      whatsit.set_property("glyph",glyph);
      whatsit.set_property("font", lookup_font().unwrap().specialize(&glyph.to_string()));
    }
    if let Some(role) = role_opt {
      whatsit.set_property("role", role);
    }
    Ok(Vec::new())
  });

  // Almost like a register, but different...
  DefPrimitive!("\\mathchardef Token SkipSpaces SkipMatch:=", sub[(newcs)] {
    // Let w/o AfterAssignment
    let means_relax = lookup_meaning(&TOKEN_RELAX).unwrap();
    assign_meaning(&newcs, means_relax, None);
    let value  = gullet::read_number().unwrap_or_default();
    let (role, glyph) = decode_math_char(value.value_of() as u16)?;
    // eprintln!("    role: {:?} + glyph: {:?}", role, glyph);
    state::install_definition(Register::new_chardef(newcs,Some(value.into()), glyph, role.map(arena::pin)), None);
    state::after_assignment();
  });
  

  DefConstructor!("\\mathaccent Number Digested",
  "<ltx:XMApp><ltx:XMTok role='OVERACCENT'>#glyph</ltx:XMTok><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>",
  sizer => "#1",    // Close enough?
  after_digest => sub[whatsit] {
    let n = whatsit.get_arg(1).unwrap().value_of();
    let (_role, glyph_opt) = decode_math_char(n as u16)?;
    if let Some(glyph) = glyph_opt {
      whatsit.set_property("glyph", glyph);

      let mut glyph_buf: [u8; 4] = [0; 4];
      let glyph_str: &str = glyph.encode_utf8(&mut glyph_buf);
      whatsit.set_property("font", lookup_font().unwrap().specialize(glyph_str));
    }
  });

  // Only used for active math characters, so far
  DefRegister!("\\mathcode Number", Number::new(0),
    getter => sub[args] {
      let ch_code   = args.remove(0).expect_number().value_of() as u8;
      let ch : char = ch_code as char;
      let code = match lookup_mathcode(&ch.to_string()) {
        None => ch_code,
        Some(code) => code as u8
      };
      Number!(code)
    },    // defaults to the char's code itself(?)
    setter => sub[value, scope, args] {
      let ch = args.remove(0).expect_number().value_of() as u8;
      let ch : char = ch as char;
      assign_mathcode(ch, value.value_of() as u16, scope);
    }
  );

  // Not used anywhere (yet)
  DefRegister!("\\delcode Number", Number::new(0),
  getter=> sub[args] {
    let code = lookup_delcode(args[0].value_of() as u8 as char);
    Number::new(code.unwrap_or_default() as i64)
  },
  setter => sub[value, scope, args] {
    assign_delcode(args[0].value_of() as u8 as char,
      value.value_of() as u16, scope);
  });
  DefRegister!("\\fam", Number!(-1));

  //======================================================================
  // TeX-level grammatical roles
  //----------------------------------------------------------------------
  // \mathbin                c  assigns class 2 (binary operation) to the following character or subformula.
  // \mathclose              c  assigns class 5 (closing) to the following character or subformula.
  // \mathinner              c  makes an inner atom holding the math field.
  // \mathop                 c  assigns class 1 (large operator) to following character or subformula.
  // \mathopen               c  assigns class 4 (opening) to following character or subformula.
  // \mathord                c  assigns class 0 (ordinary) to following character or subformula.
  // \mathpunct              c  assigns class 6 (punctuation) to following character or subformula.
  // \mathrel                c  assigns class 3 (relation) to following character or subformula.
  
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

  //======================================================================
  // Delimiters
  //----------------------------------------------------------------------
  // \left     c  makes TeX calculate the size of the delimiter needed at the left of a subformula.
  // \right    c  makes TeX calculate the size of the delimiter needed at the right of a subformula.

  // This duplicates in slightly different way what DefMath has put together.
  // [duplication seems like a bad idea!]

  // TODO ?
  // # With new treatment of Simple Symbols as just Box's with assigned attributes,
  // # we're not getting whatsits, and so we're not looking them up the same way!!!
  // # TEMPORARILY (?) hack the Delimiter map
  // foreach my $entry (values %DELIMITER_MAP) {
  //   $DELIMITER_MAP{ $$entry{char} } = $entry; }
  // sub lookup_delimiter {
  //   my ($delim) = @_;
  //   return $DELIMITER_MAP{$delim}; }
  // This is a little messier than you'd think.
  // These effectively create a group between the \left,\right.
  // And this also gives us a single list of things to parse separately.
  // Since \left,\right are TeX, primitives and must be paired up,
  // we use a bit of macro trickery to simulate.
  // [The \@hidden@bgroup/egroup keep from putting a {} into the UnTeX]
  // HOWEVER, an additional complication is that it is a common mistake to omit the balancing
  // \right! Using an \egroup (or hidden) makes it hard to recover, so use a special egroup
  DefMacro!("\\left XToken", r"\@left #1\@hidden@bgroup");
  // Like \@hidden@egroup, but softer about missing \left
  DefConstructor!("\\right@hidden@egroup", "",
    after_digest => {
      if is_value_bound("MODE", Some(0)) // Last stack frame was a mode switch!?!?!
        || lookup_bool("groupNonBoxing") { // or group was opened with \begingroup
        Error!("unexpected", "\\right", "Unbalanced \\right, no balancing \\left."); }
      else {
        egroup()?;
      }
    },
    reversion => None);

  DefMacro!("\\right XToken", r"\right@hidden@egroup\@right #1");

  DefConstructor!("\\@left Token",
    "?#char(<ltx:XMTok role='#role' name='#name' stretchy='#stretchy'>#char</ltx:XMTok>)\
      (?#hint(<ltx:XMHint/>)(#1))",
    after_digest => sub[whatsit] {
      let delim = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      if delim == "." {
        whatsit.set_property("hint", true); }
      else if let Some(entry) = DELIMITER_MAP.get(delim.as_str()) {
        whatsit.set_property("role", entry.left_role);
        whatsit.set_property("char", entry.char);
        whatsit.set_property("name", entry.name);
        whatsit.set_property("stretchy", true);
        // TODO: Should we have more Rc<> wrappers over Font?
        whatsit.set_font(Rc::new(
          whatsit.get_arg(1).unwrap().get_font()?.unwrap().into_owned()
        ));
      }
      else if whatsit.get_arg(1).unwrap().get_property_string("role") == "OPEN" {
        whatsit.get_arg_mut(1).unwrap().set_property("stretchy", true);
      } else {
        Warn!("unexpected", delim,
          "Missing delimiter; '.' inserted");
      }
      Ok(Vec::new())
    },
    alias => "\\left");
  DefConstructor!("\\@right Token",
    "?#char(<ltx:XMTok role='#role' name='#name' stretchy='#stretchy'>#char</ltx:XMTok>)\
      (?#hint(<ltx:XMHint/>)(#1))",
    after_digest => sub[whatsit] {
      let delim = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      if delim == "." {
        whatsit.set_property("hint", true); }
      else if let Some(entry) = DELIMITER_MAP.get(delim.as_str()) {
        whatsit.set_property("role", entry.right_role);
        whatsit.set_property("char", entry.char);
        whatsit.set_property("name", entry.name);
        whatsit.set_property("stretchy", true);
        // TODO: Should we have more Rc<> wrappers over Font?
        whatsit.set_font(Rc::new(
          whatsit.get_arg(1).unwrap().get_font()?.unwrap().into_owned()
        ));
      }
      else if whatsit.get_arg(1).unwrap().get_property_string("role") == "CLOSE" {
        whatsit.get_arg_mut(1).unwrap().set_property("stretchy", true);
      } else {
        Warn!("unexpected", delim,
          "Missing delimiter; '.' inserted");
      }
      Ok(Vec::new())
    },
    alias => "\\right");
  
  //======================================================================
  // Limit placement
  //----------------------------------------------------------------------
  // \limits                 c  displays limits above and below large operators (class 1).
  // \nolimits               c  displays limits to the right of large operators (class 1).
  // \displaylimits          c  restores normal conventions for using limits with operators.
    
  // TODO:
  // DefConstructor('\limits', undef, sub {
  //     my $node = $_[0]->getElement;
  //     $_[0]->setAttribute($_[0]->getLastChildElement($node) || $node, scriptpos => "mid"); });
  // DefConstructor('\nolimits', undef, sub {
  //     my $node = $_[0]->getElement;
  //     $node = $_[0]->getLastChildElement($node) || $node;
  //     $node->removeAttribute('scriptpos'); });    # default is 'post', so we can just remove the
  // attrib.
  //
  // DefConstructor('\displaylimits', undef, sub {
  //     my ($document, %props) = @_;
  //     my $node = $_[0]->getElement;
  //     $node = $_[0]->getLastChildElement($node) || $node;
  //     if (($props{mathstyle} || 'text') eq 'display') {
  //       $document->setAttribute($node, scriptpos => "mid"); }
  //     else {
  //       $node->removeAttribute('scriptpos'); } },
  //   properties => sub { (mathstyle => LookupValue('font')->getMathstyle); });

  //======================================================================
  // Math script fonts
  //----------------------------------------------------------------------
  // \textfont               iq specifies the text font for a family.
  // \scriptfont             iq specifies the script font for a family.
  // \scriptscriptfont       iq specifies the scriptscript font for a family.

  // Doubtful that we can do anything useful with these.
  // These look essentially like Registers, although Knuth doesn't call them that.
  // NOTE: These should just point to a CS token, right????
  // (although it SHOULD be one defined to be a font switch??)
  // NOTE: These should NOT be global(?)
  DefRegister!("\\textfont Number", T_CS!("\\tenrm"),
  getter => sub[args] {
    let fam = args.remove(0).expect_number().value_of();
    lookup_number(&s!("textfont_{fam}")).unwrap_or_default()
  },
  setter => sub[font,scope,args] {
    let fam = args.remove(0).expect_number().value_of();
    state::assign_value(&s!("textfont_{fam}"), font, scope);
  });

  DefRegister!("\\scriptfont Number" => T_CS!("\\sevenrm"),
  getter => sub[args] {
    let fam = args.remove(0).expect_number().value_of();
    lookup_number(&s!("scriptfont_{fam}")).unwrap_or_default()
  },
  setter => sub[font,scope,args] {
    let fam = args.remove(0).expect_number().value_of();
    state::assign_value(&s!("scriptfont_{fam}"), font, scope);
  });

  DefRegister!("\\scriptscriptfont Number" => T_CS!("\\fiverm"),
  getter => sub[args] {
    let fam = args.remove(0).expect_number().value_of();
    lookup_number(&s!("scriptscriptfont_{fam}")).unwrap_or_default()
  },
  setter => sub[font,scope,args] {
    let fam = args.remove(0).expect_number().value_of();
    state::assign_value(&s!("scriptscriptfont_{fam}"), font, scope);
  });

  
});

/// A shorthand data structure for delimiter metadata
pub struct DelimiterMeta {
  char: char,
  left_role: &'static str,
  right_role: &'static str,
  name: Option<&'static str>,

}
/// This duplicates in slightly different way what DefMath has put together.
pub static DELIMITER_MAP : Lazy<HashMap<&'static str, DelimiterMeta>> = Lazy::new(|| raw_map!(
  "(" => DelimiterMeta{char: '(', left_role: "OPEN", right_role: "CLOSE", name:None},
  ")" => DelimiterMeta{char: ')', left_role: "OPEN", right_role: "CLOSE", name:None},
  "[" => DelimiterMeta{char: '[', left_role: "OPEN", right_role: "CLOSE", name:None},
  "]" => DelimiterMeta{ char: ']', left_role: "OPEN", right_role: "CLOSE", name:None},
  "\\{" => DelimiterMeta{ char: '{', left_role: "OPEN", right_role: "CLOSE", name:None},
  "\\}" => DelimiterMeta{ char: '}', left_role: "OPEN", right_role: "CLOSE", name:None},
  "\\lfloor"=> DelimiterMeta{ char: '\u{230A}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("lfloor") },
  "\\rfloor"=> DelimiterMeta{ char: '\u{230B}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("rfloor") },
  "\\lceil" => DelimiterMeta{ char: '\u{2308}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("lceil") },
  "\\rceil" => DelimiterMeta{ char: '\u{2309}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("rceil") },
  "\\langle"=> DelimiterMeta{ char: '\u{27E8}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("langle") },
  "\\rangle"=> DelimiterMeta{ char: '\u{27E9}',
                left_role: "OPEN",  right_role: "CLOSE", name: Some("rangle") },
  "<"      => DelimiterMeta{ char: '\u{27E8}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("langle") },
  ">"      => DelimiterMeta{ char: '\u{27E9}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("rangle") },
  "/"      => DelimiterMeta{ char: '/', left_role: "MULOP",   right_role: "MULOP", name: None },
  "\\backslash" => DelimiterMeta{ char: '\u{005C}',
                left_role: "MULOP",   right_role: "MULOP", name: Some("backslash") },
  "|"      => DelimiterMeta{ char: '|',
                left_role: "VERTBAR", right_role: "VERTBAR", name: None },
  "\\|"     => DelimiterMeta{ char: '\u{2225}',
                left_role: "VERTBAR", right_role: "VERTBAR", name: None },
  "\\uparrow"   => DelimiterMeta{ char: '\u{2191}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("uparrow") },
  "\\Uparrow"   => DelimiterMeta{ char: '\u{21D1}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("Uparrow") },
  "\\downarrow" => DelimiterMeta{ char: '\u{2193}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("downarrow") },
  "\\Downarrow" =>  DelimiterMeta{ char: '\u{21D3}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("Downarrow") },
  "\\updownarrow" => DelimiterMeta{ char: '\u{2195}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("updownarrow") },
  "\\Updownarrow" => DelimiterMeta{ char: '\u{21D5}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("Updownarrow") }
));
