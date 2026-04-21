use crate::prelude::*;

LoadDefinitions!({
  TeX!(r"\newif\if@EURleft\@EURlefttrue");
  DeclareOption!("left", r"\@EURlefttrue");
  DeclareOption!("right", r"\@EURleftfalse");
  DeclareOption!("official", {});
  DeclareOption!("gen", {});
  DeclareOption!("gennarrow", {});
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
