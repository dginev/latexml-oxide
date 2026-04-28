use latexml_package::prelude::*;

LoadDefinitions!({
  // Perl LaTeXML/lib/LaTeXML/Package/ifdraft.sty.ltxml L19-32:
  //   DefConditional('\if@draft');            # default false
  //   DefConditional('\if@option@draft');     # default false
  //   DefConditional('\if@option@final');     # default false
  //   DeclareOption('draft', sub {
  //     Let('\if@draft',        '\iftrue');
  //     Let('\if@option@draft', '\iftrue'); });
  //   DeclareOption('final', sub {
  //     Let('\if@draft',        '\iffalse');
  //     Let('\if@option@final', '\iftrue'); });
  //   ProcessOptions(inorder => 1);
  //
  // Prior Rust port hardcoded `\if@option@final => true` and silently ignored
  // `\usepackage[draft]{ifdraft}`, so any document that relied on
  // `\ifdraft{…draft-only…}{…else…}` to branch on the package option would
  // always take the else branch regardless of the caller's request.
  DefConditional!("\\if@draft");
  DefConditional!("\\if@option@draft");
  DefConditional!("\\if@option@final");

  DeclareOption!("draft", {
    Let!("\\if@draft", "\\iftrue");
    Let!("\\if@option@draft", "\\iftrue");
  });
  DeclareOption!("final", {
    Let!("\\if@draft", "\\iffalse");
    Let!("\\if@option@final", "\\iftrue");
  });
  ProcessOptions!();

  DefMacro!(
    "\\ifdraft",
    "\\if@draft\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi"
  );
  DefMacro!(
    "\\ifoptiondraft",
    "\\if@option@draft\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi"
  );
  DefMacro!(
    "\\ifoptionfinal",
    "\\if@option@final\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi"
  );
});
