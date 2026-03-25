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

  // Perl: DefMacro('\ifdraft', sub { T_CS(IfCondition('\if@draft') ? '\@firstoftwo' : '\@secondoftwo'); });
  // Stub: these macros expand to \@firstoftwo or \@secondoftwo based on conditionals.
  // For now, default to the "final" branch (\@secondoftwo) since that's the common case.
  DefMacro!("\\ifdraft", "\\@secondoftwo");
  DefMacro!("\\ifoptiondraft", "\\@secondoftwo");
  DefMacro!("\\ifoptionfinal", "\\@secondoftwo");
});
