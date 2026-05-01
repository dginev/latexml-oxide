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

  DefConditional!("\\ifpreprintsty");
  DefConditional!("\\ifsecnumbers");
  DefConditional!("\\ifsegabssty");
  // Perl revtex 3 internals — `\iffirstfig` controls float placement on
  // the first page; revtex 3 papers commonly set `\firstfigfalse` BEFORE
  // `\begin{document}` to bypass the kludge. The Perl LaTeXML bindings
  // for revtex don't pre-define it, so the user's `\firstfigfalse` raises
  // an undefined-CS error. Witness: hep-th0109174 (R=1 → 0).
  DefConditional!("\\iffirstfig");

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

  // The earlier latex_constructs.rs `\begin{equation}` is installed
  // with `locked => true`. Under Perl `loadLTXML`'s UNLOCK scope, the
  // redefinitions below override that lock — and Rust's `_load_binding`
  // now wraps the dispatcher in `local_state_unlocked_guard(true)`
  // (Package.pm:2318 parity), so no surgical lock-clear is needed.

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
