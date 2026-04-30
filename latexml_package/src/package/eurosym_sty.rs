use crate::prelude::*;

LoadDefinitions!({
  TeX!(r"\newif\if@EURleft\@EURlefttrue");
  DeclareOption!("left", r"\@EURlefttrue");
  DeclareOption!("right", r"\@EURleftfalse");
  DeclareOption!("official", {});
  DeclareOption!("gen", {});
  // Perl eurosym.sty.ltxml L28 declares `gennorrow` (typo for
  // `gennarrow`). Upstream eurosym.sty actually uses `gennarrow`, so we
  // declare both — the Perl typo for log-order parity, the correct form
  // for real-world user input. Both are no-ops in Perl too.
  DeclareOption!("gennarrow", {});
  DeclareOption!("gennorrow", {});
  DeclareOption!("genwide", {});

  ProcessOptions!();

  DefMacro!(
    "\\EUR{}",
    r"{\if@EURleft\euro\,\fi#1\if@EURleft\else\,\euro\fi}"
  );

  DefMacro!("\\officialeuro", None, "\u{20AC}");
  Let!("\\euro", "\\officialeuro");

  // People shouldn't be using these, but let's at least avoid errors.
  DefMacro!("\\eurobars", None, "=");
  DefMacro!("\\eurobarsnarrow", None, "=");
  DefMacro!("\\eurobarswide", None, "=");
  Let!("\\geneuro", "\\officialeuro");
  Let!("\\geneuronarrow", "\\officialeuro");
  Let!("\\geneurowide", "\\officialeuro");
});
