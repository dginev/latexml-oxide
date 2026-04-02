//! TeX Math
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
use crate::engine::tex_character;
use latexml_core::common::mathchar::decode_math_char;

/// Perl's mergeLimits (TeX_Math.pool.ltxml): walks backward through the
/// digest list, extracts any existing script level from the previous
/// `scriptpos` value, and sets `scriptpos` to `pos` + level.
fn merge_limits(pos: &str) {
  use crate::engine::tex_scripts::is_script;
  // Compute script level before borrowing the box list mutably,
  // since get_script_level() also borrows the stomach.
  let default_level = get_script_level().to_string();
  stomach::with_box_list_mut(|list| {
    for b in list.iter_mut().rev() {
      // extract trailing level number from existing scriptpos, or use current script level
      let prev = b.get_property("scriptpos").map(|v| v.to_string()).unwrap_or_default();
      let level: String = prev.chars().rev().take_while(|c| c.is_ascii_digit()).collect::<Vec<_>>()
        .into_iter().rev().collect();
      let level = if level.is_empty() { &default_level } else { &level };
      b.set_property("scriptpos", format!("{pos}{level}"));
      // Perl: last unless IsEmpty($box) || IsScript($box)
      // Continue past empty boxes AND script boxes
      if !b.is_empty().unwrap_or(false) && is_script(b).is_none() {
        break;
      }
    }
  });
}

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Math Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // NOT YET IMPLEMENTED !?!?!
  //----------------------------------------------------------------------
  // \radical                c  makes a radical atom from the delimiter (27-bit number) and the math
  // field. \muskipdef              c  creates a symbolic name for a \muskip register.
  // \muskip                 iq assigns <muglue> to a \muskip register.
  // \nonscript              c  ignores immediately following glue or kern in script and
  // scriptscript styles.
  // Should discard following skip/glue; for now a no-op stub.
  DefPrimitive!("\\nonscript", None);

  //======================================================================
  // The next two sections are the basic LaTeXML Infrastructure for math.
  // There are several internal control sequences which need to be renamed!
  //======================================================================

  // Decide whether we're going into or out of math, inline or display.
  Tag!("ltx:XMText", auto_open => true, auto_close => true);
  // This really should be T_MATH
  // and it should (or not) check for a second $ only if not in restricted horizontal mode!
  // (and then all the \lx@dollar@in@(text|math|normal)mode defns would not be needed.
  DefPrimitive!(T_CS!("\\lx@dollar@default"), None, {
    let mut op = "\\lx@begin@inline@math";
    {
      let mode = state::lookup_string("MODE");
      if mode == "display_math" {
        if gullet::if_next(T_MATH!())? {
          gullet::read_token()?;
          op = "\\lx@end@display@math";
        } else {
          Error!(
            "expected",
            "$",
            "Missing $ closing display math.\nIgnoring; expect to be in wrong math/text mode."
          );
          op = "";
        }
      } else if mode == "math" {
        // Perl L62: `elsif ($mode eq 'math')` — because beginMode("inline_math")
        // maps to MODE="math" via bindable_mode. NOT "inline_math".
        op = "\\lx@end@inline@math";
      } else {
        // Perl: only check for $$ when within a vertical bound mode
        let bound = state::lookup_string("BOUND_MODE");
        if bound.ends_with("vertical") && gullet::if_next(T_MATH!())? {
          gullet::read_token()?;
          op = "\\lx@begin@display@math";
        }
      }
    }
    if !op.is_empty() {
      Ok(stomach::invoke_token(&T_CS!(op))?)
    } else {
      Ok(Vec::new())
    }
  });
  // Let this be the default, conventional $
  Let!(T_MATH!(), T_CS!("\\lx@dollar@default"));
  // Obsolete aliases
  Let!("\\lx@dollar@in@normalmode", "\\lx@dollar@default");

  //======================================================================
  // Math mode in special cases: math alignments, or perverse equations for ...$text$...
  // In Perl, \lx@dollar@in@textmode is now aliased to \lx@dollar@default.
  Let!("\\lx@dollar@in@textmode", "\\lx@dollar@default");
  // Note that $ within a math alignment (eg array environment),
  // switches to text mode! There's no $$ for display math.

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
    if lookup_int("MATH_ALIGN_$_BEGUN") == (level as i64) {
      // If we're begun making _something_ with $.
      let l = if lookup_bool("IN_MATH") {
        // But we're somehow in math?
        stomach::invoke_token(&T_CS!("\\lx@end@inline@math"))
      } else {
        stomach::invoke_token(&T_CS!("\\lx@end@inmath@text"))
      };
      assign_value("MATH_ALIGN_$_BEGUN", 0, None); // Reset this AFTER finishing the something
      l
    } else {
      assign_value("MATH_ALIGN_$_BEGUN", level + 1, None); // Note that we've begun something
      if lookup_bool("IN_MATH") {
        // If we're "still" in math
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
    // Perl: beginMode('restricted_horizontal') — NOT 'text'
    before_digest => sub { stomach::begin_mode("restricted_horizontal")?; },
    capture_body => true
  );
  DefConstructor!("\\lx@end@inmath@text", "", alias => "$",
    before_digest => sub { stomach::end_mode("restricted_horizontal")?; });
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
      // Perl: $_[0]->enterHorizontal; (TeX_Math.pool.ltxml line 135)
      enter_horizontal();
      // begin_mode handles \everydisplay injection (Stomach.pm lines 504-507)
      begin_mode("display_math")?;
    },
    capture_body  => true );

  DefConstructor!(T_CS!("\\lx@end@display@math"), None, None,
    reversion => Tokens!(T_MATH!(),T_MATH!()),
    before_digest => { end_mode("display_math")?; });

  DefConstructor!("\\lx@begin@inline@math",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    reversion    => Tokens!(T_MATH!()),
    before_digest => {
      // Perl: $_[0]->enterHorizontal; (TeX_Math.pool.ltxml line 151)
      enter_horizontal();
      // begin_mode handles \everymath injection (Stomach.pm lines 504-507)
      begin_mode("inline_math")?;
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
  // \everydisplay         pt holds tokens inserted at the start of every switch to display math
  // mode. \everymath            pt holds tokens inserted at the start of every switch to math
  // mode.
  DefRegister!("\\everymath", Tokens!());
  DefRegister!("\\everydisplay", Tokens!());

  // Almost like a register (and \countdef), but different...
  // (including the preassignment to \relax!)
  DefConstructor!("\\mathchar Number",
    "?#glyph(<ltx:XMTok role='#role' ?#meaning(meaning='#meaning') ?#name(name='#name')\
     ?#scriptpos(scriptpos='#scriptpos') ?#mathstyle(mathstyle='#mathstyle')\
     ?#stretchy(stretchy='#stretchy')>#glyph</ltx:XMTok>)",
    sizer       => "#1",
    after_digest => sub[whatsit] {
      let n = whatsit.get_arg(1).unwrap().value_of();
      let props = decode_math_char(n as u16, None)?;
      if let Some(glyph) = props.glyph {
        whatsit.set_property("glyph", glyph);
        whatsit.set_property("font", lookup_font().unwrap().specialize(&glyph.to_string()));
      }
      if let Some(ref role) = props.role {
        whatsit.set_property("role", role.clone());
      }
      if let Some(ref meaning) = props.meaning {
        whatsit.set_property("meaning", meaning.clone());
      }
      if let Some(ref name) = props.name {
        whatsit.set_property("name", name.clone());
      }
      if let Some(ref scriptpos) = props.scriptpos {
        whatsit.set_property("scriptpos", scriptpos.clone());
      }
      if let Some(ref mathstyle) = props.mathstyle {
        whatsit.set_property("mathstyle", mathstyle.clone());
      }
      if let Some(ref stretchy) = props.stretchy {
        whatsit.set_property("stretchy", stretchy.clone());
      }
      Ok(Vec::new())
    }
  );

  DefConstructor!("\\delimiter Number",
  "?#glyph(?#isMath(<ltx:XMTok role='#role' ?#name(name='#name')\
   ?#stretchy(stretchy='#stretchy')>#glyph</ltx:XMTok>)(#glyph))",
  sizer       => "#glyph",
  after_digest => sub[whatsit] {
    let mut n = whatsit.get_arg(1).unwrap().value_of();
    n >>= 12;    // Ignore 3 rightmost digits and treat as \mathchar
    let props = decode_math_char(n as u16, None)?;
    if let Some(glyph) = props.glyph {
      whatsit.set_property("glyph", glyph);
      whatsit.set_property("font", lookup_font().unwrap().specialize(&glyph.to_string()));
    }
    if let Some(ref role) = props.role {
      whatsit.set_property("role", role.clone());
    }
    if let Some(ref name) = props.name {
      whatsit.set_property("name", name.clone());
    }
    if let Some(ref stretchy) = props.stretchy {
      whatsit.set_property("stretchy", stretchy.clone());
    }
    Ok(Vec::new())
  });

  // Almost like a register, but different...
  DefPrimitive!("\\mathchardef Token SkipSpaces SkipMatch:=", sub[(newcs)] {
    // Let w/o AfterAssignment
    let means_relax = lookup_meaning(&TOKEN_RELAX).unwrap();
    assign_meaning(&newcs, means_relax, None);
    let value = gullet::read_number().unwrap_or_default();
    let props = decode_math_char(value.value_of() as u16, None)?;
    state::install_definition(
      Register::new_math_chardef(
        newcs,
        Some(value.into()),
        props.glyph,
        props.role.as_deref().map(arena::pin),
        CharDefProps {
          meaning: props.meaning.as_deref().map(arena::pin),
          // chardef_name: synthesized at invoke time from CS name
          stretchy: props.stretchy.as_deref().map(arena::pin),
          scriptpos: props.scriptpos.as_deref().map(arena::pin),
          mathstyle: props.mathstyle.as_deref().map(arena::pin),
          need_scriptpos: props.need_scriptpos,
          need_mathstyle: props.need_mathstyle }
      ), None);
    state::after_assignment();
  });

  // Perl: DefConstructor('\mathaccent Number Digested', ..., afterDigest => sub { ... })
  DefConstructor!("\\mathaccent Number Digested",
  "<ltx:XMApp><ltx:XMTok role='#accrole' name='#name' stretchy='#stretchy'>#glyph</ltx:XMTok><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>",
  sizer => "#2",    // Close enough?
  after_digest => sub[whatsit] {
    let n = whatsit.get_arg(1).unwrap().value_of();
    let props = decode_math_char(n as u16, None)?;
    if let Some(glyph) = props.glyph {
      let glyph_string = glyph.to_string();
      let acc_props = tex_character::unicode_accent(&glyph_string);
      // Perl: $glyph = $acc_props{unwrapped} if $acc_props{unwrapped};
      let display_glyph = if let Some(ap) = acc_props {
        if !ap.unwrapped.is_empty() { ap.unwrapped.to_string() } else { glyph_string.clone() }
      } else { glyph_string.clone() };
      let accrole = acc_props.map(|ap| ap.role).unwrap_or("OVERACCENT");
      let name = acc_props.map(|ap| ap.name);
      // Perl: $$acc_props{stretchy} || 'false'
      let stretchy = "false";
      whatsit.set_property("glyph", arena::pin(&display_glyph));
      whatsit.set_property("font", lookup_font().unwrap().specialize(&display_glyph));
      whatsit.set_property("accrole", accrole);
      if let Some(n) = name {
        whatsit.set_property("name", n);
      }
      whatsit.set_property("stretchy", stretchy);
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

  // Perl default is -1 ("no delimiter code assigned").
  DefRegister!("\\delcode Number", Number::new(-1),
  getter=> sub[args] {
    let code = lookup_delcode(args[0].value_of() as u8 as char);
    Number::new(code.map(|c| c as i64).unwrap_or(-1))
  },
  setter => sub[value, scope, args] {
    assign_delcode(args[0].value_of() as u8 as char,
      value.value_of() as u16, scope);
  });
  // Perl #2772: \fam with getter/setter for fontfamily state
  DefRegister!("\\fam", Number!(-1),
    getter => {
      let fam = state::lookup_value("fontfamily")
        .map(|v| v.to_string().parse::<i64>().unwrap_or(-1))
        .unwrap_or(-1);
      Some(RegisterValue::Number(Number::new(fam)))
    },
    setter => sub[value, scope, _args] {
      state::assign_value("fontfamily", Stored::from(value.value_of()), scope);
    });

  //======================================================================
  // TeX-level grammatical roles
  //----------------------------------------------------------------------
  // \mathbin                c  assigns class 2 (binary operation) to the following character or
  // subformula. \mathclose              c  assigns class 5 (closing) to the following character
  // or subformula. \mathinner              c  makes an inner atom holding the math field.
  // \mathop                 c  assigns class 1 (large operator) to following character or
  // subformula. \mathopen               c  assigns class 4 (opening) to following character or
  // subformula. \mathord                c  assigns class 0 (ordinary) to following character or
  // subformula. \mathpunct              c  assigns class 6 (punctuation) to following character
  // or subformula. \mathrel                c  assigns class 3 (relation) to following character
  // or subformula.

  // Is XMWrap the right thing to wrap with (instead of XMArg)?
  // We can't really assume that the stuff inside is sensible math.
  // NOTE that \mathord and \mathbin aren't really right here.
  // We need a finer granularity than TeX does: an ORD could be several things,
  // a BIN could be a MULOP or ADDOP.
  // AND, rarely, they're empty.... Is it wrong to drop them?
  // Perl: adjustMathRole — wraps content in XMWrap, conditionally sets role
  // If single child already has an acceptable sub-role, DON'T override
  DefConstructor!("\\mathord{}", sub[document, args, _props] {
    adjust_math_role(document, args.first().and_then(|a| a.as_ref()), "ID", None)?;
  }, bounded => true);
  DefConstructor!("\\mathop{}", sub[document, args, props] {
    let sp = props.get("scriptpos").map(|v| v.to_string());
    adjust_math_role(document, args.first().and_then(|a| a.as_ref()), "BIGOP", sp.as_deref())?;
  },
    bounded => true,
    properties => {
      let pos = if lookup_font().is_some_and(|f|
        f.mathstyle.as_deref() == Some("display"))
      { "mid" } else { "post" };
      Ok(stored_map!("scriptpos" => pos))
    });
  DefConstructor!("\\mathbin{}", sub[document, args, _props] {
    adjust_math_role(document, args.first().and_then(|a| a.as_ref()), "BINOP", None)?;
  }, bounded => true);
  DefConstructor!("\\mathrel{}", sub[document, args, _props] {
    adjust_math_role(document, args.first().and_then(|a| a.as_ref()), "RELOP", None)?;
  }, bounded => true);
  DefConstructor!("\\mathopen{}", sub[document, args, _props] {
    adjust_math_role(document, args.first().and_then(|a| a.as_ref()), "OPEN", None)?;
  }, bounded => true);
  DefConstructor!("\\mathclose{}", sub[document, args, _props] {
    adjust_math_role(document, args.first().and_then(|a| a.as_ref()), "CLOSE", None)?;
  }, bounded => true);
  DefConstructor!("\\mathpunct{}", sub[document, args, _props] {
    adjust_math_role(document, args.first().and_then(|a| a.as_ref()), "PUNCT", None)?;
  }, bounded => true);
  DefConstructor!("\\mathinner{}", sub[document, args, _props] {
    adjust_math_role(document, args.first().and_then(|a| a.as_ref()), "ATOM", None)?;
  }, bounded => true);

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
  // Perl: DefMacro('\left TeXDelimiter', ...) where TeXDelimiter invokes the token.
  // When the delimiter is \delimiter<Number>, we must digest it to produce the glyph.
  // For regular tokens (., \{, \langle, etc.), XToken suffices.
  DefMacro!("\\left XToken", sub[(delim)] {
    let delim_str = delim.to_string();
    if delim_str == "\\delimiter" {
      // \delimiter<Number>: read the number, shift, and decode to get the delimiter char
      let n = gullet::read_number()?.value_of() >> 12;
      let props = decode_math_char(n as u16, None)?;
      if let Some(glyph) = props.glyph {
        let glyph_str = glyph.to_string();
        if let Some(entry) = DELIMITER_MAP.get(glyph_str.as_str()) {
          // Found the delimiter — unread it as a token
          let tok = T_OTHER!(entry.char.to_string());
          gullet::unread(Tokens::new(vec![T_CS!("\\@left"), tok, T_CS!("\\lx@hidden@bgroup")]));
        } else {
          // Unknown glyph, use dot delimiter
          gullet::unread(Tokens::new(vec![T_CS!("\\@left"), T_OTHER!("."), T_CS!("\\lx@hidden@bgroup")]));
        }
      } else {
        gullet::unread(Tokens::new(vec![T_CS!("\\@left"), T_OTHER!("."), T_CS!("\\lx@hidden@bgroup")]));
      }
    } else {
      gullet::unread(Tokens::new(vec![T_CS!("\\@left"), delim, T_CS!("\\lx@hidden@bgroup")]));
    }
  });
  // \lx@hidden@egroup@right: like \lx@hidden@egroup, but softer about missing \left
  DefConstructor!("\\lx@hidden@egroup@right", "",
    after_digest => {
      if is_value_bound("MODE", Some(0)) // Last stack frame was a mode switch!?!?!
        || lookup_bool("groupNonBoxing") { // or group was opened with \begingroup
        Error!("unexpected", "\\right", "Unbalanced \\right, no balancing \\left."); }
      else {
        egroup()?;
      }
    },
    reversion => None);

  // \right is a constructor (non-expandable), so that LaTeX3 kernel can use it as a separator
  // in \numexpr contexts. It unreads \lx@hidden@egroup@right and \lx@right into the input stream.
  DefConstructor!("\\right", "",
    before_digest => {
      gullet::unread(Tokens::new(vec![T_CS!("\\lx@hidden@egroup@right"), T_CS!("\\lx@right")]));
    },
    // Empty reversion — \lx@right provides the actual \right reversion via alias
    reversion => Tokens!());

  DefConstructor!("\\@left Token",
    "?#char(<ltx:XMTok role='#role' name='#name' ?#meaning(meaning='#meaning') stretchy='#stretchy'>#char</ltx:XMTok>)\
      (?#hint(<ltx:XMHint/>)(#1))",
    after_digest => sub[whatsit] {
      let delim = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      if delim == "." {
        whatsit.set_property("hint", true); }
      else if let Some(entry) = DELIMITER_MAP.get(delim.as_str()) {
        whatsit.set_property("role", entry.left_role);
        whatsit.set_property("char", entry.char);
        if let Some(name) = entry.name {
          whatsit.set_property("name", name);
        }
        // Preserve meaning from DefMath (e.g. "/" has meaning="divide")
        // Look up math_token_attributes for the delimiter character.
        let char_str = entry.char.to_string();
        state::with_value(&format!("math_token_attributes_{}", char_str), |val| {
          if let Some(Stored::HashString(ref attrs)) = val {
            if let Some(meaning) = attrs.get("meaning") {
              whatsit.set_property("meaning", meaning.to_string());
            }
          }
        });
        whatsit.set_property("stretchy", true);
        whatsit.set_font(Rc::new(
          whatsit.get_arg(1).unwrap().get_font()?.unwrap().into_owned()
        ));
        // Set canonical reversion: \left + the user-facing delimiter token.
        // XToken expands \{ → \lx@text@lbrace during macro reading, so the stored
        // arg reverts to \lx@text@lbrace. Override with the canonical form.
        let canonical = match entry.char {
          '{' => Some("\\left\\{"),
          '}' => Some("\\left\\}"),
          _ => None,
        };
        if let Some(rev_str) = canonical {
          whatsit.set_property("reversion", Stored::Tokens(Tokenize!(rev_str)));
        }
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
    "?#char(<ltx:XMTok role='#role' name='#name' ?#meaning(meaning='#meaning') stretchy='#stretchy'>#char</ltx:XMTok>)\
      (?#hint(<ltx:XMHint/>)(#1))",
    after_digest => sub[whatsit] {
      let delim = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      if delim == "." {
        whatsit.set_property("hint", true); }
      else if let Some(entry) = DELIMITER_MAP.get(delim.as_str()) {
        whatsit.set_property("role", entry.right_role);
        whatsit.set_property("char", entry.char);
        if let Some(name) = entry.name {
          whatsit.set_property("name", name);
        }
        // Preserve meaning from DefMath
        let char_str = entry.char.to_string();
        state::with_value(&format!("math_token_attributes_{}", char_str), |val| {
          if let Some(Stored::HashString(ref attrs)) = val {
            if let Some(meaning) = attrs.get("meaning") {
              whatsit.set_property("meaning", meaning.to_string());
            }
          }
        });
        whatsit.set_property("stretchy", true);
        whatsit.set_font(Rc::new(
          whatsit.get_arg(1).unwrap().get_font()?.unwrap().into_owned()
        ));
        // Set canonical reversion for brace delimiters (XToken expands \} → \lx@text@rbrace)
        let canonical = match entry.char {
          '{' => Some("\\right\\{"),
          '}' => Some("\\right\\}"),
          _ => None,
        };
        if let Some(rev_str) = canonical {
          whatsit.set_property("reversion", Stored::Tokens(Tokenize!(rev_str)));
        }
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

  DefConstructor!("\\limits", "",
    after_digest => { merge_limits("mid"); },
    properties => { Ok(stored_map!("isEmpty" => true)) });
  DefConstructor!("\\nolimits", "",
    after_digest => { merge_limits("post"); },
    properties => { Ok(stored_map!("isEmpty" => true)) });
  DefConstructor!("\\displaylimits", "",
    after_digest => {
      let pos = if lookup_font().is_some_and(|f|
        f.mathstyle.as_deref() == Some("display"))
      { "mid" } else { "post" };
      merge_limits(pos);
    },
    properties => { Ok(stored_map!("isEmpty" => true)) });

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
    lookup_token(&s!("textfont_{fam}")).unwrap_or_else(|| T_CS!("\\tenrm"))
  },
  setter => sub[font,scope,args] {
    let fam = args.remove(0).expect_number().value_of();
    state::assign_value(&s!("textfont_{fam}"), font, scope);
  });

  DefRegister!("\\scriptfont Number" => T_CS!("\\sevenrm"),
  getter => sub[args] {
    let fam = args.remove(0).expect_number().value_of();
    lookup_token(&s!("scriptfont_{fam}")).unwrap_or_else(|| T_CS!("\\sevenrm"))
  },
  setter => sub[font,scope,args] {
    let fam = args.remove(0).expect_number().value_of();
    state::assign_value(&s!("scriptfont_{fam}"), font, scope);
  });

  DefRegister!("\\scriptscriptfont Number" => T_CS!("\\fiverm"),
  getter => sub[args] {
    let fam = args.remove(0).expect_number().value_of();
    lookup_token(&s!("scriptscriptfont_{fam}")).unwrap_or_else(|| T_CS!("\\fiverm"))
  },
  setter => sub[font,scope,args] {
    let fam = args.remove(0).expect_number().value_of();
    state::assign_value(&s!("scriptscriptfont_{fam}"), font, scope);
  });

  //======================================================================
  // Math script styles
  //----------------------------------------------------------------------
  // \displaystyle           c  selects display style: D or D'.
  // \scriptscriptstyle      c  selects scriptscript style: SS or SS'.
  // \scriptstyle            c  selects script style: S or S'.
  // \textstyle              c  selects text style: T or T'.

  // Also record that this explicitly sets the mathstyle (support for \over, etal)
  DefPrimitive!("\\displaystyle", {
    MergeFont!(mathstyle => "display");
    Tbox::new(
      *EMPTY_SYM,
      None,
      None,
      Tokens!(T_CS!("\\displaystyle")),
      stored_map!("explicit_mathstyle" => true),
    )
  });
  DefPrimitive!("\\textstyle", {
    MergeFont!(mathstyle => "text");
    Tbox::new(
      *EMPTY_SYM,
      None,
      None,
      Tokens!(T_CS!("\\textstyle")),
      stored_map!("explicit_mathstyle" => true),
    )
  });
  DefPrimitive!("\\scriptstyle", {
    MergeFont!(mathstyle => "script");
    Tbox::new(
      *EMPTY_SYM,
      None,
      None,
      Tokens!(T_CS!("\\scriptstyle")),
      stored_map!("explicit_mathstyle" => true),
    )
  });
  DefPrimitive!("\\scriptscriptstyle", {
    MergeFont!(mathstyle => "scriptscript");
    Tbox::new(
      *EMPTY_SYM,
      None,
      None,
      Tokens!(T_CS!("\\scriptscriptstyle")),
      stored_map!("explicit_mathstyle" => true),
    )
  });

  //======================================================================
  //
  //----------------------------------------------------------------------
  // \mathchoice             c  specifies specific subformulas for the 4 main styles.
  // \vcenter                c  centers material with respect to the axis.

  // Note that in TeX, all 4 args get digested(!)
  // and the choice is made when absorbing!
  // Perl: TeX_Math.pool.ltxml lines 931-939
  DefConstructor!("\\mathchoice Digested Digested Digested Digested", sub[document, args, props] {
    let style = prop_string!(props, "mathstyle");
    let choice = match style.as_str() {
      "display" => args[0].as_ref(),
      "text"    => args[1].as_ref(),
      "script"  => args[2].as_ref(),
      _         => args[3].as_ref(), // scriptscript or default
    };
    if let Some(c) = choice {
      document.absorb(c, None)?;
    }
  },
    properties => {
      let mathstyle = lookup_font()
        .map(|f| f.get_mathstyle().map(|s| s.to_string()).unwrap_or_default())
        .unwrap_or_default();
      Ok(stored_map!("mathstyle" => mathstyle))
    }
  );
  // THIS IS WRONG!!!!
  Let!("\\vcenter", "\\vbox");
  //======================================================================
  //
  //----------------------------------------------------------------------
  // \overline               c  puts a line over the following character or subformula.
  // \underline              c  puts a line under the following character or subformula.
  // Perl: TeX_Math.pool.ltxml lines 951-987
  // Note that (over|under) brace accents act like \limit, but lines, arrows do NOT!
  DefMath!("\\lx@math@overline{}", "\u{00AF}", operator_role => "OVERACCENT",
    operator_stretchy => true, name => "overline", alias => "\\overline");
  DefConstructor!(
    "\\lx@text@overline{}",
    "<ltx:text framed='overline' _noautoclose='true'>#1</ltx:text>",
    enter_horizontal => true
  );
  DefMath!("\\lx@math@underline{}", "\u{00AF}", operator_role => "UNDERACCENT",
    operator_stretchy => true, name => "underline", alias => "\\underline");
  DefConstructor!(
    "\\lx@text@underline{}",
    "<ltx:text framed='underline' _noautoclose='true'>#1</ltx:text>",
    enter_horizontal => true
  );
  DefMath!("\\lx@math@overrightarrow{}", "\u{2192}", operator_role => "OVERACCENT",
    operator_stretchy => true, name => "overrightarrow", alias => "\\overrightarrow");
  DefMath!("\\lx@math@overleftarrow{}", "\u{2190}", operator_role => "OVERACCENT",
    operator_stretchy => true, name => "overleftarrow", alias => "\\overleftarrow");
  DefMath!("\\lx@math@underrightarrow{}", "\u{2192}", operator_role => "UNDERACCENT",
    operator_stretchy => true, name => "underrightarrow", alias => "\\underrightarrow");
  DefMath!("\\lx@math@underleftarrow{}", "\u{2190}", operator_role => "UNDERACCENT",
    operator_stretchy => true, name => "underleftarrow", alias => "\\underleftarrow");
  DefMath!("\\lx@math@overbrace{}", "\u{23DE}", operator_role => "OVERACCENT",
    scriptpos => "mid", operator_stretchy => true,
    name => "overbrace", alias => "\\overbrace", robust => true);
  DefMath!("\\lx@math@underbrace{}", "\u{23DF}", operator_role => "UNDERACCENT",
    scriptpos => "mid", operator_stretchy => true,
    name => "underbrace", alias => "\\underbrace", robust => true);

  // Careful: Use \protect so that it doesn't expand too early in alignments, etc.
  DefMacro!(
    "\\overline{}",
    r"\protect\ifmmode\lx@math@overline{#1}\else\lx@text@overline{#1}\fi",
    locked => true
  );
  DefMacro!(
    "\\underline{}",
    r"\protect\ifmmode\lx@math@underline{#1}\else\lx@text@underline{#1}\fi",
    locked => true
  );

  //======================================================================
  // fraction-like things
  //----------------------------------------------------------------------
  // \above                  d  is equivalent to `\abovewithdelims..'.
  // \abovewithdelims        c  is a generalized fraction command.
  // \atop                   d  is equivalent to `\atopwithdelims..'.
  // \atopwithdelims         d  is a generalized fraction command with an invisible fraction bar.
  // \over                   d  is equivalent to `\overwithdelims..'.
  // \overwithdelims         d  is a generalized fraction command with preset fraction bar
  // thickness. After digesting the \choose (or whatever), grab the previous and following
  // material and store as args in the whatsit.

  // TODO: adjustMathstyle — recursively adjusts mathstyle on already-digested boxes.
  // Perl walks all Box/List/Whatsit children and shifts mathstyle using
  // mathstyle_adjust_map. Skipped for now; cosmetic effect only.

  // \lx@delimiterdot — empty delimiter hint (replacement for "." in \left. / \right.)
  DefConstructor!("\\lx@delimiterdot", "<ltx:XMHint/>",
    alias => ".",
    properties => { stored_map!("hint" => true) });

  // \lx@left/\lx@right: like \left/\right but without extra grouping.
  // Perl uses TeXDelimiter param type; we handle \delimiter specially.
  Let!("\\lx@left", "\\@left");
  // \lx@right wraps \@right to handle \delimiter<Number> (TeXDelimiter logic)
  DefMacro!("\\lx@right XToken", sub[(delim)] {
    let delim_str = delim.to_string();
    if delim_str == "\\delimiter" {
      let n = gullet::read_number()?.value_of() >> 12;
      let props = decode_math_char(n as u16, None)?;
      if let Some(glyph) = props.glyph {
        let glyph_str = glyph.to_string();
        if let Some(entry) = DELIMITER_MAP.get(glyph_str.as_str()) {
          let tok = T_OTHER!(entry.char.to_string());
          gullet::unread(Tokens::new(vec![T_CS!("\\@right"), tok]));
        } else {
          gullet::unread(Tokens::new(vec![T_CS!("\\@right"), T_OTHER!(".")]));
        }
      } else {
        gullet::unread(Tokens::new(vec![T_CS!("\\@right"), T_OTHER!(".")]));
      }
    } else {
      gullet::unread(Tokens::new(vec![T_CS!("\\@right"), delim]));
    }
  });

  // \lx@generalized@over{reversion}{keyvals}{top}{bottom}
  // keyvals: role, meaning, left, right, thickness
  DefConstructor!("\\lx@generalized@over Undigested RequiredKeyVals",
    "?#needXMDual(\
       <ltx:XMDual>\
         <ltx:XMApp>\
           <ltx:XMRef _xmkey='#xmkey0'/>\
           <ltx:XMRef _xmkey='#xmkey1'/>\
           <ltx:XMRef _xmkey='#xmkey2'/>\
         </ltx:XMApp>\
         <ltx:XMWrap>\
           #left)(\
       )\
       <ltx:XMApp>\
         <ltx:XMTok _xmkey='#xmkey0' role='#role' ?#meaning(meaning='#meaning')\
          ?#mathstyle(mathstyle='#mathstyle') ?#thickness(thickness='#thickness')/>\
         <ltx:XMArg _xmkey='#xmkey1'>#top</ltx:XMArg>\
         <ltx:XMArg _xmkey='#xmkey2'>#bottom</ltx:XMArg>\
       </ltx:XMApp>\
       ?#needXMDual(\
           #right\
         </ltx:XMWrap>\
       </ltx:XMDual>)()",
    after_digest => sub[whatsit] {
      use latexml_core::stomach;
      use latexml_core::binding::content::merge_font;
      use latexml_core::common::font::Font;
      use latexml_core::list::List;
      use latexml_core::binding::def::dialect::get_xmarg_id;

      // Extract key-value pairs from arg 2
      // Store left/right as Stored::Tokens so template #left/#right can absorb them
      let (role_kv, meaning_kv, thickness_kv, has_left, has_right, left_val, right_val) = {
        let arg2 = whatsit.get_arg(2);
        if let Some(d) = arg2 {
          use latexml_core::digested::DigestedData;
          if let DigestedData::KeyVals(kv) = d.data() {
            let role = kv.get_value("role").map(ToString::to_string);
            let meaning = kv.get_value("meaning").map(ToString::to_string);
            let thickness = kv.get_value("thickness").map(ToString::to_string);
            let left_val = kv.get_value("left").cloned();
            let right_val = kv.get_value("right").cloned();
            let has_left = left_val.is_some();
            let has_right = right_val.is_some();
            (role, meaning, thickness, has_left, has_right, left_val, right_val)
          } else {
            (None, None, None, false, false, None::<ArgWrap>, None::<ArgWrap>)
          }
        } else {
          (None, None, None, false, false, None, None)
        }
      };
      // Store left/right as pre-digested Stored::Digested for template #left/#right.
      // The template's #prop lookup converts Stored→Option<Digested> for absorption.
      // Stored::Tokens doesn't convert, but Stored::Digested does.
      use latexml_core::definition::argument::ArgWrap;
      // Digest left/right delimiter tokens.
      // Replace \lx@left/\lx@right (which may resolve to \left/\right with egroup
      // semantics) with \@left/\@right (Constructors without grouping).
      for (key, val_opt) in [("left", &left_val), ("right", &right_val)] {
        if let Some(val) = val_opt {
          if let ArgWrap::Tokens(ref ts) = val {
            // Rewrite tokens: replace any left/right CS with \@left/\@right
            let mut new_tokens = Vec::new();
            for tok in ts.unlist_ref().iter() {
              let s = tok.to_string();
              if s.ends_with("left") && s.starts_with('\\') {
                new_tokens.push(T_CS!("\\@left"));
              } else if s.ends_with("right") && s.starts_with('\\') {
                new_tokens.push(T_CS!("\\@right"));
              } else {
                new_tokens.push(*tok);
              }
            }
            let d = stomach::digest(Tokens::new(new_tokens))?;
            whatsit.set_property(key, Stored::Digested(d));
          } else {
            whatsit.set_property(key, Stored::String(arena::pin(val.to_string())));
          }
        }
      }

      // Determine mathstyle from current font
      let style = lookup_font()
        .and_then(|f| f.get_mathstyle().map(|s| s.to_string()))
        .unwrap_or_else(|| "display".to_string());

      // Determine role: default to FRACOP
      let role = role_kv.unwrap_or_else(|| "FRACOP".to_string());

      // Determine meaning: default to "divide" if thickness is not "0pt"
      let meaning = if let Some(m) = meaning_kv {
        if m.is_empty() { None } else { Some(m) }
      } else if thickness_kv.as_deref() != Some("0pt") {
        Some("divide".to_string())
      } else {
        None
      };

      // Grab the numerator (already digested content)
      let top = stomach::regurgitate();
      // Perl: adjustMathstyle($style, {}, @top) — retroactively adjust font sizes
      adjust_mathstyle(&style, &top);

      // Set fraction font for denominator
      merge_font(Font { fraction: Some(true), ..Font::default() });

      // Digest the denominator
      let mut bot = stomach::digest_next_body(None)?;

      // Pop the closing token (endmath, endgroup, etc.) — leave it for further processing
      let closing = bot.pop();

      // Set properties on the whatsit
      let top_list = Digested::from(List::new(top));
      let bot_list = Digested::from(List::new(bot));
      whatsit.set_property("top", top_list);
      whatsit.set_property("bottom", bot_list);
      whatsit.set_property("role", role);
      if let Some(ref m) = meaning {
        whatsit.set_property("meaning", m.clone());
      }
      if let Some(ref t) = thickness_kv {
        whatsit.set_property("thickness", t.clone());
      }
      whatsit.set_property("mathstyle", style);

      // For delimited variants, set up XMDual keys
      if has_left || has_right {
        whatsit.set_property("needXMDual", "1");
        let key0 = get_xmarg_id()?;
        let key1 = get_xmarg_id()?;
        let key2 = get_xmarg_id()?;
        whatsit.set_property("xmkey0", key0.to_string());
        whatsit.set_property("xmkey1", key1.to_string());
        whatsit.set_property("xmkey2", key2.to_string());
      }

      // Return the closing token to be placed after the whatsit
      let result: Vec<Digested> = closing.into_iter().collect();
      Ok(result)
    },
    reversion => sub[whatsit, _args] {
      use latexml_core::common::object::Object;
      use latexml_core::state::Stored;
      // Perl: (Revert($whatsit->getProperty('top')), $whatsit->getArg(1)->unlist, Revert($whatsit->getProperty('bottom')))
      let mut result = Vec::new();
      if let Some(top) = whatsit.get_property("top") {
        if let Stored::Digested(ref d) = *top {
          result.extend(d.revert()?.unlist());
        }
      }
      if let Some(arg1) = whatsit.get_arg(1) {
        result.extend(arg1.revert()?.unlist());
      }
      if let Some(bottom) = whatsit.get_property("bottom") {
        if let Stored::Digested(ref d) = *bottom {
          result.extend(d.revert()?.unlist());
        }
      }
      Ok(Tokens::new(result))
    }
  );

  DefMacro!("\\above Dimension",
    "\\lx@generalized@over{\\above #1}{meaning=divide,thickness=#1}");
  DefMacro!("\\abovewithdelims Token Token Dimension",
    "\\lx@generalized@over{\\abovewithdelims #1 #2 #3}{left={\\lx@left#1},right={\\lx@right#2},meaning=divide,thickness=#3}");
  DefMacro!("\\atop",
    "\\lx@generalized@over{\\atop}{thickness=0pt}");
  DefMacro!("\\atopwithdelims Token Token",
    "\\lx@generalized@over{\\atopwithdelims #1 #2}{thickness=0pt,left={\\lx@left#1},right={\\lx@right#2}}");
  DefMacro!("\\over",
    "\\lx@generalized@over{\\over}{meaning=divide}");
  DefMacro!("\\overwithdelims Token Token",
    "\\lx@generalized@over{\\overwithdelims #1 #2}{left={\\lx@left#1},right={\\lx@right#2},meaning=divide}");

  //======================================================================
  //
  //----------------------------------------------------------------------
  // \mkern                  c  adds a math kern item to the current math list.
  // \mskip                  c  adds math glue to the current math list.
  // \thinmuskip             pm is ``thin'' math glue inserted into formulas.
  // \medmuskip              pm is ``medium'' math glue inserted into formulas.
  // \thickmuskip            pm is ``thick'' math glue inserted into formulas.
  // \abovedisplayskip       pg is normal glue placed before a displayed equation.
  // \abovedisplayshortskip  pg is alternate glue placed before a displayed equation.
  // \belowdisplayskip       pg is normal glue placed after a displayed equation.
  // \belowdisplayshortskip  pg is alternate glue placed after a displayed equation.

  // Perl: Box(' ', undef, undef, Invocation(...), width => $length, isSpace => 1)
  // Use regular space as content, matching Perl. Width is stored as MuGlue.
  DefPrimitive!("\\mkern MuGlue", sub[(length)] {
    Tbox::new(arena::pin_static(" "), None, None, Invocation!(T_CS!("\\mkern"), vec![length]),
      stored_map!("width" => length, "isSpace" => true)) });
  DefPrimitive!("\\mskip MuGlue", sub[(length)] {
    Tbox::new(arena::pin_static(" "), None, None, Invocation!(T_CS!("\\mskip"), vec![length]),
      stored_map!("width" => length, "isSpace" => true)) });

  // MuGlue registers; TeXBook p.274
  // NOTE: Perl stores these as Glue with "mu" parsed to pt internally,
  // NOT as MuGlue. Changing to MuGlue causes width computation regressions.
  DefRegister!("\\thinmuskip", Glue!("3mu"));
  DefRegister!("\\medmuskip", Glue!("4mu plus 2mu minus 4mu"));
  DefRegister!("\\thickmuskip", Glue!("5mu plus 5mu"));

  DefRegister!("\\abovedisplayskip", Glue!("12pt plus 3pt minus 9pt"));
  DefRegister!("\\abovedisplayshortskip", Glue!("0pt plus 3pt"));
  DefRegister!("\\belowdisplayskip", Glue!("12pt plus 3pt minus 9pt"));
  DefRegister!("\\belowdisplayshortskip", Glue!("0pt plus 3pt"));
  //======================================================================
  //
  //----------------------------------------------------------------------
  // \binoppenalty           pi is the penalty for a line break after a binary operation.
  // \postdisplaypenalty     pi is the penalty added immediately after a math display.
  // \predisplaypenalty      pi is the penalty added immediately before a math display.
  // \relpenalty             pi is the penalty for a line break after a relation.
  // \displaywidowpenalty    pi is the penalty added after the penultimate line immediately
  // preceeding a display. \skewchar               iq is -1 or the character used to fine-tune the
  // positioning of math accents     . \defaultskewchar        pi is -1 or the \skewchar value for
  // a font when it is loaded. \delimitershortfall     pd is the second parameter used to compute
  // the size of delimeters required by \left and \right. \displayindent          pd is the amount
  // to shift a line holding a displayed equation. \displaywidth           pd is the width of the
  // line holding a displayed equation. \mathsurround           pd is extra space added when
  // switching in and out of math mode. \nulldelimiterspace     pd is the width of a null or
  // missing delimiter. \predisplaysize         pd is the effective width of the line preceeding a
  // displayed equation. \scriptspace            pd is extra space added after a subscript or a
  // superscript. \delimiterfactor        pi is the first parameter used to compute the size of
  // delimeters required by \left and \right.
  DefRegister!("\\binoppenalty", Number!(700));
  DefRegister!("\\relpenalty", Number!(500));
  DefRegister!("\\displaywidowpenalty", Number!(50));
  DefRegister!("\\predisplaypenalty", Number!(10000));
  DefRegister!("\\postdisplaypenalty", Number!(0));

  DefRegister!("\\skewchar{}", Number::new(0));
  // TODO:
  //  getter => sub {
  //     my ($font) = @_;
  //     my $info = lookupFontinfo($font);
  //     return ($info && $$info{skewchar}) || Number(0); },
  //   setter => sub {
  //     my ($value, $scope, $font) = @_;
  //     if (my $info = lookupFontinfo($font)) {
  //       $$info{skewchar} = $value; } }
  // );
  DefRegister!("\\defaultskewchar", Number!(-1));

  // Dimen registers; TeXBook p. 274
  DefRegister!("\\delimitershortfall", Dimension!("5pt"));
  DefRegister!("\\nulldelimiterspace", Dimension!("1.2pt"));
  DefRegister!("\\scriptspace", Dimension!("0.5pt"));
  DefRegister!("\\mathsurround", Dimension!("0"));
  DefRegister!("\\predisplaysize", Dimension!("0"));
  DefRegister!("\\displaywidth", Dimension!("0"));
  DefRegister!("\\displayindent", Dimension!("0"));
  DefRegister!("\\delimiterfactor", Number!(0));

  //======================================================================
  // Equation numbers
  //----------------------------------------------------------------------
  // \eqno                   c  puts an equation number at the right-hand margin.
  // \leqno                  c  puts an equation number at the left-hand margin.

  // \eqno & \leqno are really bizzare.
  // They should seemingly digest until $ (or while still in math mode),
  // and use that stuff as the reference number.
  // However, since people abuse this, and we're really not quite TeX,
  // we really can't do it Right.
  // Even a \begin{array} ends up expanding into a $ !!!
  DefMacro!("\\eqno", {
    // my $locator  = $gullet->getLocator;
    let mut stuff = Vec::new();
    // This is risky!!!

    while let Some(t) = gullet::read_x_token(Some(false), false, None)? {
      if t == T_BEGIN!() {
        stuff.push(t);
        let balanced_arg = gullet::read_balanced(ExpansionLevel::Off, false, false)?;
        if !balanced_arg.is_empty() {
          stuff.extend(balanced_arg.unlist());
        }
        stuff.push(T_END!());
      }
      // What do I need to explicitly list here!?!?!? UGGH!
      else if t == T_MATH!()
        || t == T_CS!("\\]")
        // UGH from 2022: also don"t jump over rows
        || t == T_CS!("\\cr")
        // see arXiv:math/0001062, for one example
        || t == T_CS!("\\lx@hidden@cr")
        || t == T_CS!("\\lx@end@display@math")
        || t == T_CS!("\\begingroup") // Totally wrong, but to catch expanded environments
        // any sort of environ begin or end???
        || t.with_str(|tstr| tstr.starts_with("\\begin{") || tstr.starts_with("\\end{"))
      // This seems needed within AmSTeX environs
      {
        let mut invoked = Invocation!(T_CS!("\\lx@eqno"), vec![Tokens::new(stuff)]).unlist();
        invoked.push(t);
        return Ok(Tokens::new(invoked));
      } else {
        stuff.push(t);
      }
    }
    Error!(
      "unexpected",
      "\\eqno",
      "Fell of the end reading tag for \\eqno!"
    );
    // s!("started {locator}"));
    Tokens::new(stuff)
  });

  Let!("\\leqno", "\\eqno");
  // Revert to nothing, since it really doesn't belong in the TeX string(?)
  DefConstructor!("\\lx@eqno{}",
    "^ <ltx:tags><ltx:tag><ltx:Math><ltx:XMath>#1</ltx:XMath></ltx:Math></ltx:tag></ltx:tags>",
    reversion => "");
});

/// A shorthand data structure for delimiter metadata
pub struct DelimiterMeta {
  char:       char,
  left_role:  &'static str,
  right_role: &'static str,
  name:       Option<&'static str>,
}
/// This duplicates in slightly different way what DefMath has put together.
pub static DELIMITER_MAP: Lazy<HashMap<&'static str, DelimiterMeta>> = Lazy::new(|| {
  raw_map!(
    "(" => DelimiterMeta{char: '(', left_role: "OPEN", right_role: "CLOSE", name:None},
    ")" => DelimiterMeta{char: ')', left_role: "OPEN", right_role: "CLOSE", name:None},
    "[" => DelimiterMeta{char: '[', left_role: "OPEN", right_role: "CLOSE", name:None},
    "]" => DelimiterMeta{ char: ']', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\{" => DelimiterMeta{ char: '{', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\}" => DelimiterMeta{ char: '}', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\lbrace" => DelimiterMeta{ char: '{', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\rbrace" => DelimiterMeta{ char: '}', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\lx@math@lbrace" => DelimiterMeta{ char: '{', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\lx@math@rbrace" => DelimiterMeta{ char: '}', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\lx@text@lbrace" => DelimiterMeta{ char: '{', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\lx@text@rbrace" => DelimiterMeta{ char: '}', left_role: "OPEN", right_role: "CLOSE", name:None},
    "{" => DelimiterMeta{ char: '{', left_role: "OPEN", right_role: "CLOSE", name:None},
    "}" => DelimiterMeta{ char: '}', left_role: "OPEN", right_role: "CLOSE", name:None},
    "\\lfloor"=> DelimiterMeta{ char: '\u{230A}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("lfloor") },
    "\u{230A}" => DelimiterMeta{ char: '\u{230A}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("lfloor") },
    "\\rfloor"=> DelimiterMeta{ char: '\u{230B}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("rfloor") },
    "\u{230B}" => DelimiterMeta{ char: '\u{230B}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("rfloor") },
    "\\lceil" => DelimiterMeta{ char: '\u{2308}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("lceil") },
    "\u{2308}" => DelimiterMeta{ char: '\u{2308}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("lceil") },
    "\\rceil" => DelimiterMeta{ char: '\u{2309}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("rceil") },
    "\u{2309}" => DelimiterMeta{ char: '\u{2309}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("rceil") },
    "\\langle"=> DelimiterMeta{ char: '\u{27E8}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("langle") },
    "\u{27E8}" => DelimiterMeta{ char: '\u{27E8}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("langle") },
    "\\rangle"=> DelimiterMeta{ char: '\u{27E9}',
                  left_role: "OPEN",  right_role: "CLOSE", name: Some("rangle") },
    "\u{27E9}" => DelimiterMeta{ char: '\u{27E9}',
                  left_role: "OPEN",  right_role: "CLOSE", name: Some("rangle") },
    "<"      => DelimiterMeta{ char: '\u{27E8}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("langle") },
    ">"      => DelimiterMeta{ char: '\u{27E9}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("rangle") },
    // Perl #2762: \lgroup / \rgroup
    "\\lgroup"=> DelimiterMeta{ char: '\u{27EE}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("lgroup") },
    "\u{27EE}" => DelimiterMeta{ char: '\u{27EE}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("lgroup") },
    "\\rgroup"=> DelimiterMeta{ char: '\u{27EF}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("rgroup") },
    "\u{27EF}" => DelimiterMeta{ char: '\u{27EF}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("rgroup") },
    "/"      => DelimiterMeta{ char: '/', left_role: "MULOP",   right_role: "MULOP", name: None },
    "\\backslash" => DelimiterMeta{ char: '\u{005C}',
                  left_role: "MULOP",   right_role: "MULOP", name: Some("backslash") },
    "|"      => DelimiterMeta{ char: '|',
                  left_role: "VERTBAR", right_role: "VERTBAR", name: None },
    "\\|"     => DelimiterMeta{ char: '\u{2016}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("||") },
    "\\Vert"  => DelimiterMeta{ char: '\u{2016}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("||") },
    "\\vert"  => DelimiterMeta{ char: '|',
                  left_role: "VERTBAR", right_role: "VERTBAR", name: None },
    "\u{2225}" => DelimiterMeta{ char: '\u{2016}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("||") },
    "\u{2016}" => DelimiterMeta{ char: '\u{2016}',
                  left_role: "OPEN", right_role: "CLOSE", name: Some("||") },
    "\\uparrow"   => DelimiterMeta{ char: '\u{2191}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("uparrow") },
    "\u{2191}"    => DelimiterMeta{ char: '\u{2191}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("uparrow") },
    "\\Uparrow"   => DelimiterMeta{ char: '\u{21D1}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("Uparrow") },
    "\u{21D1}"    => DelimiterMeta{ char: '\u{21D1}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("Uparrow") },
    "\\downarrow" => DelimiterMeta{ char: '\u{2193}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("downarrow") },
    "\u{2193}"    => DelimiterMeta{ char: '\u{2193}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("downarrow") },
    "\\Downarrow" =>  DelimiterMeta{ char: '\u{21D3}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("Downarrow") },
    "\u{21D3}"    => DelimiterMeta{ char: '\u{21D3}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("Downarrow") },
    "\\updownarrow" => DelimiterMeta{ char: '\u{2195}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("updownarrow") },
    "\u{2195}"    => DelimiterMeta{ char: '\u{2195}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("updownarrow") },
    "\\Updownarrow" => DelimiterMeta{ char: '\u{21D5}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("Updownarrow") },
    "\u{21D5}"    => DelimiterMeta{ char: '\u{21D5}',
                      left_role: "OPEN", right_role: "CLOSE", name: Some("Updownarrow") },
    // amsmath delimiter CS names (resolved via TeXDelimiter in Perl)
    "\\lvert"  => DelimiterMeta{ char: '|', left_role: "OPEN",  right_role: "OPEN",  name: None },
    "\\rvert"  => DelimiterMeta{ char: '|', left_role: "CLOSE", right_role: "CLOSE", name: None },
    "\\lVert"  => DelimiterMeta{ char: '\u{2016}', left_role: "OPEN",  right_role: "OPEN",  name: Some("||") },
    "\\rVert"  => DelimiterMeta{ char: '\u{2016}', left_role: "CLOSE", right_role: "CLOSE", name: Some("||") }
  )
});

/// Perl TeX_Math.pool.ltxml L1010-1052: adjustMathstyle
/// Recursively adjusts font mathstyle on already-digested boxes.
/// Called from \over handler to retroactively adjust numerator font sizes.
/// Perl: adjustMathRole (TeX_Math.pool.ltxml L669-688)
/// Wraps content in XMWrap, conditionally sets role.
/// If single non-hint child has acceptable sub-role, keeps it.
fn adjust_math_role(
  document: &mut Document,
  content: Option<&Digested>,
  role: &str,
  scriptpos: Option<&str>,
) -> Result<()> {
  use latexml_core::common::xml::element_nodes;

  let content = match content {
    Some(c) => c,
    None => return Ok(()), // Nothing? do nothing!
  };

  // Perl: open XMWrap, absorb, close, inspect children
  document.open_element("ltx:XMWrap", None, None)?;
  document.absorb(content, None)?;
  let wrap_opt = document.close_element("ltx:XMWrap")?;

  if let Some(mut wrapper) = wrap_opt {
    // Filter out XMHint nodes
    let nodes: Vec<_> = element_nodes(&wrapper)
      .into_iter()
      .filter(|n| document::get_node_qname(n) != arena::pin_static("ltx:XMHint"))
      .collect();

    // Perl: %mathclass_subclass lookup
    let acceptable = if role == "ATOM" {
      true // ATOM accepts any role
    } else if nodes.len() == 1 {
      if let Some(got_role) = nodes[0].get_attribute("role") {
        match role {
          "BIGOP" => matches!(got_role.as_str(), "ARROW" | "SUMOP" | "INTOP" | "DIFFOP"),
          "BINOP" => matches!(got_role.as_str(), "ADDOP" | "MULOP"),
          "PUNCT" => matches!(got_role.as_str(), "PERIOD"),
          "ID" => matches!(got_role.as_str(), "NUMBER"),
          _ => false,
        }
      } else {
        false
      }
    } else {
      false
    };

    if !acceptable {
      document.set_attribute(&mut wrapper, "role", role)?;
    }
    if let Some(sp) = scriptpos {
      document.set_attribute(&mut wrapper, "scriptpos", sp)?;
    }
  }
  Ok(())
}

pub fn adjust_mathstyle(outerstyle: &str, boxes: &[Digested]) {
  let mut adjusted: std::collections::HashSet<usize> = std::collections::HashSet::new();
  adjust_mathstyle_rec(outerstyle, &mut adjusted, boxes);
}

fn adjust_mathstyle_rec(
  outerstyle: &str,
  adjusted: &mut std::collections::HashSet<usize>,
  boxes: &[Digested],
) {
  for box_item in boxes {
    // Use the data pointer as identity for dedup (Rc::as_ptr on inner)
    let ptr = box_item.data() as *const DigestedData as usize;
    if adjusted.contains(&ptr) {
      continue; // don't adjust twice (args AND props may share references)
    }
    adjusted.insert(ptr);
    // Perl L1018: return if $box->getProperty('explicit_mathstyle');
    // Checked on ALL box types BEFORE dispatch — `return` stops entire recursion.
    // This preserves \scriptstyle etc. as absolute mathstyle commands.
    if box_item.get_property("explicit_mathstyle").is_some() {
      return;
    }
    // Perl L1019: next if $box->getProperty('own_mathstyle');
    if box_item.get_property("own_mathstyle").is_some() {
      continue;
    }
    match box_item.data() {
      DigestedData::TBox(b) => {
        adjust_mathstyle_internal(outerstyle, &mut b.borrow_mut());
      },
      DigestedData::List(l) => {
        let children: Vec<Digested> = l.borrow().boxes.clone();
        adjust_mathstyle_rec(outerstyle, adjusted, &children);
      },
      DigestedData::Whatsit(w) => {
        // Adjust the whatsit's font and get the new style for recursion
        let style = {
          let mut wb = w.borrow_mut();
          adjust_mathstyle_internal_whatsit(outerstyle, &mut wb)
            .unwrap_or_else(|| outerstyle.to_string())
        };
        // Recurse on args
        let args: Vec<Digested> = w
          .borrow()
          .get_args()
          .iter()
          .filter_map(|a| a.clone())
          .collect();
        adjust_mathstyle_rec(&style, adjusted, &args);
        // Recurse on property values that are Digested
        let prop_digested: Vec<Digested> = w
          .borrow()
          .properties
          .iter()
          .filter_map(|(_k, v)| {
            if let Stored::Digested(d) = v {
              Some(d.clone())
            } else {
              None
            }
          })
          .collect();
        adjust_mathstyle_rec(&style, adjusted, &prop_digested);
      },
      _ => {},
    }
  }
}

/// Perl mathstyle_adjust_map: maps (outerstyle, origstyle) → newstyle
fn mathstyle_adjust(outerstyle: &str, origstyle: &str) -> &'static str {
  match (outerstyle, origstyle) {
    ("display", "display") => "text",
    ("display", "text") => "script",
    ("display", "script") => "script",
    ("display", "scriptscript") => "scriptscript",
    ("text", "display") => "text",
    ("text", "text") => "script",
    ("text", "script") => "scriptscript",
    ("text", "scriptscript") => "scriptscript",
    ("script", "display") => "display",
    ("script", "text") => "text",
    ("script", "script") => "scriptscript",
    ("script", "scriptscript") => "scriptscript",
    ("scriptscript", "display") => "display",
    ("scriptscript", "text") => "text",
    ("scriptscript", "script") => "scriptscript",
    ("scriptscript", "scriptscript") => "scriptscript",
    _ => "display",
  }
}

/// Adjust a TBox's font mathstyle using Font::merge to trigger size recalculation
fn adjust_mathstyle_internal(outerstyle: &str, tbox: &mut Tbox) {
  let origstyle_owned = tbox
    .font
    .mathstyle
    .as_deref()
    .unwrap_or("display")
    .to_string();
  let newstyle = mathstyle_adjust(outerstyle, &origstyle_owned);
  if newstyle != origstyle_owned {
    // Use Font::merge to trigger size recalculation via STYLE_SIZE mapping
    let merge_font = Font {
      mathstyle: Some(Cow::Borrowed(newstyle)),
      ..Font::default()
    };
    let merged = tbox.font.merge(merge_font);
    tbox.font = Rc::new(merged);
  }
}

/// Adjust a Whatsit's font mathstyle, returning the new style if it had a mathstyle property
fn adjust_mathstyle_internal_whatsit(outerstyle: &str, whatsit: &mut Whatsit) -> Option<String> {
  if let Some(Stored::Font(ref font)) = whatsit.properties.get("font") {
    let origstyle = font
      .mathstyle
      .as_deref()
      .unwrap_or("display")
      .to_string();
    let newstyle = mathstyle_adjust(outerstyle, &origstyle);
    if newstyle != origstyle {
      let merge_font = Font {
        mathstyle: Some(Cow::Borrowed(newstyle)),
        ..Font::default()
      };
      let merged = font.merge(merge_font);
      whatsit
        .properties
        .insert("font", Stored::Font(Rc::new(merged)));
    }
  }
  // If whatsit has a recorded mathstyle property, adjust it too
  if let Some(Stored::String(ms)) = whatsit.properties.get("mathstyle") {
    let ms_str = arena::to_string(*ms);
    let newstyle = mathstyle_adjust(outerstyle, &ms_str);
    whatsit
      .properties
      .insert("mathstyle", Stored::String(arena::pin(newstyle)));
    Some(newstyle.to_string())
  } else {
    None
  }
}
