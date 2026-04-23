use latexml_package::prelude::*;

LoadDefinitions!({
  // Perl ar5iv-bindings/ifdraft.sty.ltxml L14-21: always behave as `final`
  // (non-draft). Three conditionals + three user-visible ifs.
  // Perl's comment: "for now always final - respect the package options
  // for official latexml support." — matched byte-for-byte.
  DefConditional!("\\if@draft", { false });
  DefConditional!("\\if@option@draft", { false });
  DefConditional!("\\if@option@final", { true });

  DefMacro!("\\ifdraft",
    "\\if@draft\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi");
  DefMacro!("\\ifoptiondraft",
    "\\if@option@draft\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi");
  DefMacro!("\\ifoptionfinal",
    "\\if@option@final\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi");
});
