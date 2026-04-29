use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex3_support.sty.ltxml

  DeclareOption!("amsfonts", {
    RequirePackage!("amsfonts");
  });
  DeclareOption!("amssymb", {
    RequirePackage!("amssymb");
  });
  DeclareOption!("amsmath", {
    RequirePackage!("amsmath");
  });
  ProcessOptions!();

  RequirePackage!("revtex4_support");

  //======================================================================
  // Additional or different definitions from revtex4_support

  RawTeX!(r"\newif\ifpreprintsty\newif\ifsecnumbers\newif\ifsegabssty");

  DefMacro!("\\eqsecnum",     "");
  DefMacro!("\\tightenlines", "");
  DefMacro!("\\wideabs",      ""); // wide abstract — takes an arg, but avoid reading it

  // RevTeX's subequation numbering environment
  DefMacro!("\\mathletters",    "\\lx@equationgroup@subnumbering@begin");
  DefMacro!("\\endmathletters", "\\lx@equationgroup@subnumbering@end");

  //======================================================================
  // Revtex3 equation environments with $ faketext trick
  DefConditional!("\\if@lx@revtex@faketext@");
  DefConditional!("\\if@lx@revtex@nestmath@");
  DefMacro!("\\lx@revtex@faketext", "\\@lx@revtex@faketext@true\\hbox\\bgroup");
  DefMacro!("\\lx@revtex@nestmath", "\\@lx@revtex@nestmath@true\\lx@dollar@default");
  DefMacro!("\\lx@dollar@in@oldrevtex",
    "\\ifmmode\
       \\if@lx@revtex@nestmath@\\let\\@next\\lx@dollar@default\\else\\let\\@next\\lx@revtex@faketext\\fi\
     \\else\
       \\if@lx@revtex@faketext@\\let\\@next\\egroup\\else\\let\\@next\\lx@revtex@nestmath\\fi\
     \\fi\\@next");

  DefEnvironment!("{equation}",
    "<ltx:equation xml:id='#id'>\
     #tags\
     <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    mode => "display_math",
    before_digest => {
      Let!(T_MATH!(), "\\lx@dollar@in@oldrevtex");
    },
    properties => { ref_step_counter("equation", false) },
    locked => true);

  DefEnvironment!("{equation*}",
    "<ltx:equation xml:id='#id'>\
     <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    mode => "display_math",
    before_digest => {
      Let!(T_MATH!(), "\\lx@dollar@in@oldrevtex");
    },
    properties => { ref_step_id("equation") },
    locked => true);

  // Perl revtex3_support.sty.ltxml L90-101: DefConstructorI('\[', undef, ...,
  //   beforeDigest => sub { beginMode('display_math'); Let(T_MATH, '\lx@dollar@in@oldrevtex'); },
  //   captureBody => 1, properties => sub { RefStepID('equation') });
  DefConstructor!("\\[",
    "<ltx:equation xml:id='#id'>\
     <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    before_digest => {
      stomach::begin_mode("display_math")?;
      Let!(T_MATH!(), "\\lx@dollar@in@oldrevtex");
    },
    properties => { ref_step_id("equation") },
    capture_body => true);
});
