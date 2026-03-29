use crate::prelude::*;
//**********************************************************************
// C.12 Line and Page Breaking
//**********************************************************************
LoadDefinitions!({
  //======================================================================
  // C.12.1 Line Breaking
  //======================================================================
  DefPrimitive!("\\linebreak[]");
  DefPrimitive!("\\nolinebreak[]");
  DefPrimitive!("\\-"); // We don't do hyphenation.
  // \hyphenation in TeX.pool
  DefPrimitive!("\\sloppy");
  DefPrimitive!("\\fussy");
  // sloppypar can be used as an environment, or by itself.
  DefMacro!("\\sloppypar", "\\par\\sloppy");
  DefMacro!("\\endsloppypar", "\\par");
  DefMacro!("\\nobreakdashes", "-");
  DefMacro!("\\showhyphens{}", "#1");
  //======================================================================
  // C.12.2 Page Breaking
  //======================================================================
  DefMacro!("\\pagebreak[Default:4]", sub[(arg_opt)] {
      let arg : u32 = if let Some(arg_t) = arg_opt {
        arg_t.to_string().parse::<u32>().unwrap_or(0)
      } else { 0 };
      if arg <= 2 {
        Ok(Tokens!()) }
      else {
        Ok(Invocation!(T_CS!("\\vadjust"), vec![T_CS!("\\clearpage")]))
      }
  });
  DefPrimitive!("\\nopagebreak[]");
  DefPrimitive!("\\columnbreak"); // latex? or multicol?
  DefPrimitive!("\\enlargethispage OptionalMatch:* {}");

  DefMacro!("\\clearpage", "\\lx@newpage");
  DefMacro!("\\cleardoublepage", "\\lx@newpage");
  DefPrimitive!("\\samepage");
});
