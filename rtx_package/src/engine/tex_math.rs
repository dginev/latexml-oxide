//! TeX Math
//! 
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({

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
});