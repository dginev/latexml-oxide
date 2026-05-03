//! PiCTeX definition stubs.
//!
//! Raw PiCTeX expands plotted points and line rules into very large TeX
//! boxes. In HTML this becomes thousands of positioned inline spans, which is
//! slow and usually less useful than keeping the figure/caption structure.
//! These stubs consume common PiCTeX drawing commands without rendering them.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!("\\expandafter\\ifx\\csname plotsymbolspacing\\endcsname\\relax\\newdimen\\plotsymbolspacing\\fi");

  DefMacro!("\\beginpicture", "");
  DefMacro!("\\endpicture", "");
  DefMacro!("\\setsolid", "");
  DefMacro!("\\setlinear", "");
  DefMacro!("\\setquadratic", "");
  DefMacro!("\\stoprotation", "");

  RawTeX!("\\def\\linethickness{\\@ifnextchar={\\lx@pictex@linethickness}{\\lx@pictex@linethickness@arg}}");
  RawTeX!("\\def\\lx@pictex@linethickness=#1{}");
  RawTeX!("\\def\\lx@pictex@linethickness@arg#1{}");
  RawTeX!("\\def\\setcoordinatesystem units <#1,#2>{}");
  RawTeX!("\\def\\setplotsymbol (#1){}");
  // PiCTeX `\setdashes`/`\setdots` accept an OPTIONAL `<spacing>` arg:
  //   \setdashes              % default dash spacing
  //   \setdashes <0.05cm>     % custom spacing
  // Witness: math0107222 uses `\setdots` (no arg). Both forms must work.
  RawTeX!("\\def\\setdashes{\\@ifnextchar<{\\lx@pictex@gobble@angle}{}}");
  RawTeX!("\\def\\setdots{\\@ifnextchar<{\\lx@pictex@gobble@angle}{}}");
  RawTeX!("\\def\\lx@pictex@gobble@angle<#1>{}");
  RawTeX!("\\def\\startrotation by #1 #2{}");
  RawTeX!("\\def\\plot#1/{}");
  RawTeX!("\\def\\putrule from #1 #2 to #3 #4{}");
  RawTeX!("\\def\\circulararc #1 degrees from #2 #3 center at #4 #5{}");
  RawTeX!("\\def\\arrow <#1> [#2] from #3 #4 to #5 #6{}");

  // PiCTeX `\put` and `\multiput` use a different syntax than the LaTeX
  // picture-env counterparts — they read "{<text>} at <x> <y>" instead of
  // "(x,y){<text>}". Witness: math0407515 (`\put{...} at -6 2.5`,
  // `\multiput{\sq} at 0 5  0 4 ... /`). Stub: gobble.
  RawTeX!("\\def\\put#1 at #2 #3 {}");
  RawTeX!("\\def\\multiput#1 at #2/{}");

  // More PiCTeX commands needed by 0910.1304, math0608653.
  RawTeX!("\\def\\setplotarea x from #1 to #2, y from #3 to #4 {}");
  RawTeX!("\\def\\setshadesymbol (#1){}");
  RawTeX!("\\def\\setshadegrid span <#1>{}");
  // \vshade / \hshade: 3 nums then <flags> then triples until `/` terminator.
  // Real PiCTeX uses \!Shadewhat to recursively peek-and-consume; our stub
  // gobbles everything up to the first `/` token. Witness: math0608653
  // (multi-line `\vshade -20 0 0 <z,z,,> -10 0 20\n <z,z,,> ... /`).
  RawTeX!("\\def\\vshade#1/{}");
  RawTeX!("\\def\\hshade#1/{}");
});
