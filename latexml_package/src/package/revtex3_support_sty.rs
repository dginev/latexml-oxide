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
  // Perl revtex 3 internals — `\iffirstfig`/`\iffirsttab` control float
  // placement on the first page; revtex 3 papers commonly set
  // `\firstfigfalse`/`\firsttabfalse` BEFORE `\begin{document}` to bypass
  // the kludge. The Perl LaTeXML bindings for revtex don't pre-define
  // these, so users' `\firstfigfalse`/`\firsttabfalse` raise undefined-CS
  // errors. Witness: hep-th0109174, cond-mat0005077.
  DefConditional!("\\iffirstfig");
  DefConditional!("\\iffirsttab");

  def_macro_noop("\\eqsecnum")?;
  def_macro_noop("\\tightenlines")?;
  def_macro_noop("\\wideabs")?; // wide abstract — takes an arg, but avoid reading it

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

  // RevTeX3 decimal-alignment macros for tabular columns.
  //   \setdec 0.000   sets the format (read up to space)
  //   \dec    1.234   aligns the value
  // Used by ~12 cluster papers (nucl-th0002021 etc.) under
  // \documentstyle[prc,aps]{revtex}. Perl LaTeXML's revtex3_support
  // also lacks these — surpass-Perl stub. Decimal alignment is
  // irrelevant for HTML; pass the value through, drop the format spec.
  RawTeX!("\\def\\setdec #1 {}");
  RawTeX!("\\def\\dec #1 {#1}");

  // \CITE — uppercase variant of \cite used in some revtex-era physics
  // papers (~11 cluster: cond-mat0003169, hep-ph0103298, etc.). Author
  // convention; not formally defined anywhere. Stub as alias to \cite
  // so the bib key is still resolved. Surpass-Perl (Perl LaTeXML also
  // errors here).
  Let!("\\CITE", "\\cite");
});
