use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ifdraft.sty.ltxml
  DefConditional!("\\if@draft");
  DefConditional!("\\if@option@draft");
  DefConditional!("\\if@option@final");

  DeclareOption!("draft", sub {
    Let!("\\if@draft", "\\iftrue");
    Let!("\\if@option@draft", "\\iftrue");
  });
  DeclareOption!("final", sub {
    Let!("\\if@draft", "\\iffalse");
    Let!("\\if@option@final", "\\iftrue");
  });

  ProcessOptions!(*);

  // Perl ifdraft.sty.ltxml L21-31: runtime dispatch — each of these
  // expands to \@firstoftwo or \@secondoftwo depending on whether the
  // associated \if@… boolean is currently true.
  DefMacro!("\\ifdraft", sub[_args] {
    Ok(Tokens!(if if_condition(&T_CS!("\\if@draft"))?.unwrap_or(false) {
      T_CS!("\\@firstoftwo")
    } else {
      T_CS!("\\@secondoftwo")
    }))
  });
  DefMacro!("\\ifoptiondraft", sub[_args] {
    Ok(Tokens!(if if_condition(&T_CS!("\\if@option@draft"))?.unwrap_or(false) {
      T_CS!("\\@firstoftwo")
    } else {
      T_CS!("\\@secondoftwo")
    }))
  });
  DefMacro!("\\ifoptionfinal", sub[_args] {
    Ok(Tokens!(if if_condition(&T_CS!("\\if@option@final"))?.unwrap_or(false) {
      T_CS!("\\@firstoftwo")
    } else {
      T_CS!("\\@secondoftwo")
    }))
  });
});
