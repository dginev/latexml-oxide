use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mathtools.sty.ltxml
  // Options: fixamsmath, donotfixamsmathbugs, allowspaces, disallowspaces — all ignored
  for option in ["fixamsmath", "donotfixamsmathbugs", "allowspaces", "disallowspaces"].iter() {
    DeclareOption!(*option, None);
  }
  // Pass all other options to amsmath
  DeclareOption!(None, {
    Digest!("\\PassOptionsToPackage{\\CurrentOption}{amsmath}")?;
  });
  ProcessOptions!();

  RequirePackage!("keyval");
  RequirePackage!("calc");
  // TODO: add support for mhsetup
  // RequirePackage!("mhsetup");
  RequirePackage!("amsmath");
  // Perl: AtBeginDocument(sub { RequirePackage('graphicx'); });
  RawTeX!("\\AtBeginDocument{\\RequirePackage{graphicx}}");

  //======================================================================
  // 3 — Macros
  //======================================================================

  // \mathtoolsset — stores keyval pairs as macros \@mt@mathtoolsset@<key>
  // TODO: complex DefPrimitive with sub{} body — stubbed
  DefMacro!("\\mathtoolsset{}", None);

  // Lookup function for mathtoolsset
  DefMacro!("\\@mt@getmtoption{}",
    "\\ifcsname @mt@mathtoolsset@#1\\endcsname\
     \\expandafter\\let\\expandafter\\@mt@currentvalue\\csname @mt@mathtoolsset@#1\\endcsname\\else\
     \\let\\@mt@currentvalue\\relax\\fi\
     \\@mt@currentvalue");

  //======================================================================
  // 3.1
  //======================================================================

  // Perl: enterHorizontal => 1 — not supported in template form, but #1 pass-through is fine
  DefConstructor!("\\mathmbox{}", "#1");

  // \mathllap — zero-width math overlap (left)
  // TODO: afterDigest with width negation — using simple template for now
  DefConstructor!("\\mathllap[]{}", "<ltx:XMArg width='0pt'>#2</ltx:XMArg>");
  // \mathrlap — zero-width math overlap (right)
  DefConstructor!("\\mathrlap[]{}", "<ltx:XMArg width='0pt'>#2</ltx:XMArg>");
  // \mathclap — zero-width math overlap (center)
  DefConstructor!("\\mathclap[]{}", "<ltx:XMArg width='0pt'>#2</ltx:XMArg>");

  DefConstructor!("\\clap{}", "#1");
  DefConstructor!("\\mathmakebox[][][]{}", "#3");
  // Ignoring cramped, for now
  DefConstructor!("\\cramped[]{}", "#2");

  // Same as \mathllap, etc (but also cramped!)
  Let!("\\crampedllap", "\\mathllap");
  Let!("\\crampedrlap", "\\mathrlap");
  Let!("\\crampedclap", "\\mathclap");

  // \smashoperator — destructures argument to recognize operators and scripts
  DefMacro!("\\smashoperator[]{}", "\\lx@smashoperator{#1}#2{}{}{}{}{}{}\\end");
  // TODO: \lx@smashoperator has complex sub{} body — stubbed to pass through
  DefMacro!("\\lx@smashoperator{}{}{}{}{}{}{}{}", "#2");
  // TODO: \lx@@smashoperator has complex afterDigest — stubbed
  DefMacro!("\\lx@@smashoperator{}{}{}{}", "#2");

  // \adjustlimits — TODO: complex afterDigest with depth/height
  DefMacro!("\\adjustlimits{}{}{}{}{}{}", "#1_{#3}#4_{#6}");

  DefConstructor!("\\SwapAboveDisplaySkip", "");

  //======================================================================
  // 3.2 — Tag forms
  //======================================================================

  // \newtagform, \renewtagform — complex DefPrimitive with sub{} — stubbed
  DefMacro!("\\newtagform{}[]{}{}", None);
  DefMacro!("\\renewtagform{}[]{}{}", None);
  DefMacro!("\\usetagform{}", "\\csname @MTStag@#1\\endcsname");

  // RawTeX('\newtagform{default}{(}{)}');
  // Stubbed — the \newtagform primitive is stubbed above
  // RawTeX!("\\newtagform{default}{(}{)}");

  Let!("\\refeq", "\\ref");
  DefMacro!("\\noeqref{}", None);

  //======================================================================
  // 3.3 — Extensible arrows
  //======================================================================

  DefConstructor!("\\xleftrightarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftrightarrow' role='METARELOP' stretchy='true'>\u{2194}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftrightarrow' role='METARELOP' stretchy='true'>\u{2194}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xLeftarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xLeftarrow' role='ARROW' stretchy='true'>\u{21D0}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xLeftarrow' role='ARROW' stretchy='true'>\u{21D0}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xRightarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xRightarrow' role='ARROW' stretchy='true'>\u{21D2}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xRightarrow' role='ARROW' stretchy='true'>\u{21D2}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xLeftrightarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xLeftrightarrow' role='ARROW' stretchy='true'>\u{21D4}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xLeftrightarrow' role='ARROW' stretchy='true'>\u{21D4}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xhookleftarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xhookleftarrow' role='ARROW' stretchy='true'>\u{21A9}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xhookleftarrow' role='ARROW' stretchy='true'>\u{21A9}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xhookrightarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xhookrightarrow' role='ARROW' stretchy='true'>\u{21AA}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xhookrightarrow' role='ARROW' stretchy='true'>\u{21AA}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xmapsto OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xmapsto' role='ARROW' stretchy='true'>\u{21A6}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xmapsto' role='ARROW' stretchy='true'>\u{21A6}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xrightharpoondown OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightharpoondown' role='ARROW' stretchy='true'>\u{21C1}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightharpoondown' role='ARROW' stretchy='true'>\u{21C1}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xrightharpoonup OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightharpoonup' role='ARROW' stretchy='true'>\u{21C0}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightharpoonup' role='ARROW' stretchy='true'>\u{21C0}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xleftharpoondown OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftharpoondown' role='ARROW' stretchy='true'>\u{21BD}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftharpoondown' role='ARROW' stretchy='true'>\u{21BD}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xleftharpoonup OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftharpoonup' role='ARROW' stretchy='true'>\u{21BC}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftharpoonup' role='ARROW' stretchy='true'>\u{21BC}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xrightleftharpoons OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightleftharpoons' role='METARELOP' stretchy='true'>\u{21CC}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightleftharpoons' role='METARELOP' stretchy='true'>\u{21CC}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xleftrightharpoons OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftrightharpoons' role='METARELOP' stretchy='true'>\u{21CB}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftrightharpoons' role='METARELOP' stretchy='true'>\u{21CB}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  // \overbracket / \underbracket — ignore optional rule thickness and bracket height args
  DefMacro!("\\overbracket[][][]{}",  "\\lx@mt@overbracket{#4}");
  DefMacro!("\\underbracket[][][]{}", "\\lx@mt@underbracket{#4}");
  DefMath!("\\lx@mt@overbracket{}", "\u{FE47}",
    operator_role => "OVERACCENT", scriptpos => "mid",
    alias => "\\overbracket");
  DefMath!("\\lx@mt@underbracket{}", "\u{FE48}",
    operator_role => "UNDERACCENT", scriptpos => "mid",
    alias => "\\underbracket");
  Let!("\\LaTeXunderbrace", "\\underbrace");
  Let!("\\LaTeXoverbrace", "\\overbrace");

  //======================================================================
  // 3.4 — Starred matrix environments
  //======================================================================

  DefMacro!("\\csname matrix*\\endcsname[]",
    "\\lx@ams@matrix{name=matrix,datameaning=matrix,alignment=#1}");
  DefMacro!("\\csname endmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname pmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=pmatrix,datameaning=matrix,alignment=#1,left=\\lx@left(,right=\\lx@right)}");
  DefMacro!("\\csname endpmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname bmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=bmatrix,datameaning=matrix,alignment=#1,left=\\lx@left[,right=\\lx@right]}");
  DefMacro!("\\csname endbmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname Bmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=Bmatrix,datameaning=matrix,alignment=#1,left=\\lx@left\\{,right=\\lx@right\\}}");
  DefMacro!("\\csname endBmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname vmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=vmatrix,delimitermeaning=determinant,datameaning=matrix,alignment=#1,left=\\lx@left|,right=\\lx@right|}");
  DefMacro!("\\csname endvmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname Vmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=Vmatrix,delimitermeaning=norm,datameaning=matrix,alignment=#1,left=\\lx@left\\|,right=\\lx@right\\|}");
  DefMacro!("\\csname endVmatrix*\\endcsname", "\\lx@end@ams@matrix");

  // Starred small matrices — complex \@smallmatrix@star@tmp with sub{} body
  // TODO: \@smallmatrix@star@tmp has complex sub{} body — stubbed to simple forwarding
  DefMacro!("\\csname smallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=matrix,datameaning=matrix,style=\\scriptsize}");
  DefMacro!("\\csname endsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname psmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=pmatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right),style=\\scriptsize}");
  DefMacro!("\\csname endpsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname bsmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=bmatrix,datameaning=matrix,left=\\lx@left[,right=\\lx@right],style=\\scriptsize}");
  DefMacro!("\\csname endbsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname Bsmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=Bmatrix,datameaning=matrix,left=\\lx@left\\{,right=\\lx@right\\},style=\\scriptsize}");
  DefMacro!("\\csname endBsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname vsmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=vmatrix,delimitermeaning=determinant,datameaning=matrix,left=\\lx@left|,right=\\lx@right|,style=\\scriptsize}");
  DefMacro!("\\csname endvsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname Vsmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=Vmatrix,delimitermeaning=norm,datameaning=matrix,left=\\lx@left\\|,right=\\lx@right\\|,style=\\scriptsize}");
  DefMacro!("\\csname endVsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  // Non-starred small matrices
  DefMacro!("\\psmallmatrix",
    "\\lx@ams@matrix{name=pmatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right),style=\\scriptsize}");
  DefMacro!("\\endpsmallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\bsmallmatrix",
    "\\lx@ams@matrix{name=bmatrix,datameaning=matrix,left=\\lx@left[,right=\\lx@right],style=\\scriptsize}");
  DefMacro!("\\endbsmallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\Bsmallmatrix",
    "\\lx@ams@matrix{name=Bmatrix,datameaning=matrix,left=\\lx@left\\{,right=\\lx@right\\},style=\\scriptsize}");
  DefMacro!("\\endBsmallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\vsmallmatrix",
    "\\lx@ams@matrix{name=vmatrix,delimitermeaning=determinant,datameaning=matrix,left=\\lx@left|,right=\\lx@right|,style=\\scriptsize}");
  DefMacro!("\\endvsmallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\Vsmallmatrix",
    "\\lx@ams@matrix{name=Vmatrix,delimitermeaning=norm,datameaning=matrix,left=\\lx@left\\|,right=\\lx@right\\|,style=\\scriptsize}");
  DefMacro!("\\endVsmallmatrix", "\\lx@end@ams@matrix");

  //======================================================================
  // {multlined} environment
  //======================================================================
  // Perl: \@@multlined is DefConstructor with DigestedBody + afterConstruct.
  // Creates <ltx:XMApp role="MULTIRELATION"> with alignment rows.
  // Simplified: \@@multlined opens a group, content flows via alignment.
  DefConstructor!("\\@@multlined",
    "<ltx:XMApp role='MULTIRELATION'>",
    before_digest => { bgroup(); }
  );
  DefMacro!("\\multlined[][]",
    "\\@ams@multirow@bindings{name=multlined}\\@@multlined\\lx@begin@alignment");
  DefMacro!("\\endmultlined", "\\lx@end@alignment\\@end@multlined");
  DefPrimitive!("\\@end@multlined", { egroup()?; });

  // \shoveright / \shoveleft — TODO: \@MT@shove has complex sub{} body
  DefMacro!("\\shoveright[]{}", "#2");
  DefMacro!("\\shoveleft[]{}", "#2");

  //======================================================================
  // Cases variants
  //======================================================================

  DefMacro!("\\dcases",
    "\\lx@ams@cases{name=dcases,meaning=cases,left=\\lx@left\\{,style=\\displaystyle,conditionmode=math}");
  DefMacro!("\\enddcases", "\\lx@end@ams@cases");

  DefMacro!("\\csname dcases*\\endcsname",
    "\\lx@ams@cases{name=dcases*,meaning=cases,left=\\lx@left\\{,style=\\displaystyle,conditionmode=text}");
  DefMacro!("\\csname enddcases*\\endcsname", "\\lx@end@ams@cases");

  DefMacro!("\\rcases",
    "\\lx@ams@cases{name=rcases,meaning=cases,right=\\lx@right\\},style=\\textstyle,conditionmode=math}");
  DefMacro!("\\endrcases", "\\lx@end@ams@cases");

  DefMacro!("\\csname rcases*\\endcsname",
    "\\lx@ams@cases{name=rcases*,meaning=cases,right=\\lx@right\\},style=\\textstyle,conditionmode=text}");
  DefMacro!("\\csname endrcases*\\endcsname", "\\lx@end@ams@cases");

  DefMacro!("\\drcases",
    "\\lx@ams@cases{name=drcases,meaning=cases,right=\\lx@right\\},style=\\displaystyle,conditionmode=math}");
  DefMacro!("\\enddrcases", "\\lx@end@ams@cases");

  DefMacro!("\\csname drcases*\\endcsname",
    "\\lx@ams@cases{name=drcases*,meaning=cases,right=\\lx@right\\},style=\\displaystyle,conditionmode=text}");
  DefMacro!("\\csname enddrcases*\\endcsname", "\\lx@end@ams@cases");

  DefMacro!("\\csname cases*\\endcsname",
    "\\lx@ams@cases{name=cases*,meaning=cases,left=\\lx@left\\{,style=\\textstyle,conditionmode=text}");
  DefMacro!("\\csname endcases*\\endcsname", "\\lx@end@ams@cases");

  // TODO: Make this actually shift the equation
  DefMacro!("\\MoveEqLeft[]", "&");

  // TODO: Properly implement \Aboxed
  DefMacro!("\\Aboxed{}", "#1");

  // TODO: Make these actually do something
  DefMacro!("\\ArrowBetweenLines[]", None);
  DefMacro!("\\csname ArrowBetweenLines*\\endcsname[]", None);
  DefMacro!("\\vdotswithin{}",
    "\\mathmakebox[\\widthof{\\ensuremath{{}#1{}}}][c]{\\vdots}");
  DefMacro!("\\shortvdotswithin{}",
    "\\MTFlushSpaceAbove & \\vdotswithin{#1} \\MTFlushSpaceBelow");
  DefMacro!("\\csname shortvdotswithin*\\endcsname{}",
    "\\MTFlushSpaceAbove \\vdotswithin{#1} & \\MTFlushSpaceBelow");
  DefMacro!("\\MTFlushSpaceAbove", None);
  DefMacro!("\\MTFlushSpaceBelow", "\\\\");

  //======================================================================
  // 3.5 — Short intertext
  //======================================================================
  Let!("\\shortintertext", "\\@ams@intertext");

  //======================================================================
  // 3.6 — Paired delimiters
  //======================================================================

  // \DeclarePairedDelimiter\cmd{left}{right}
  // Perl: DefPrimitive creates runtime macros via DefMacroI.
  // Simplified: \cmd{x} → \left<ldel> x \right<rdel>, \cmd*{x} → same
  DefPrimitive!("\\DeclarePairedDelimiter DefToken {}{}", sub[(cs, ldel, rdel)] {
    let ldel_s = ldel.to_string();
    let rdel_s = rdel.to_string();
    // Simple approach: \cmd{x} → \left<ldel> x \right<rdel>
    let body = format!("\\left{ldel_s}#1\\right{rdel_s}");
    // Use the original cs Token directly (not T_CS! which would double-escape)
    let params = parse_parameters("{}", &cs, true)?;
    def_macro(cs.clone(), params, Tokenize!(&body), None)?;
  });

  // \DeclarePairedDelimiterX\cmd[nargs]{left}{right}{body}
  DefPrimitive!("\\DeclarePairedDelimiterX DefToken [Number] {} {} {}", sub[(cs, nargs, ldel, rdel, _body)] {
    let n = nargs.value_of() as usize;
    let ldel_s = ldel.to_string();
    let rdel_s = rdel.to_string();
    // Build parameter spec: {} repeated n times
    let param_spec: String = (0..n.max(1)).map(|_| "{}").collect();
    let expansion = format!("\\left{ldel_s}#1\\right{rdel_s}");
    let params = parse_parameters(&param_spec, &cs, true)?;
    def_macro(cs.clone(), params, Tokenize!(&expansion), None)?;
  });

  // \DeclarePairedDelimiterXPP — most general form
  DefPrimitive!("\\DeclarePairedDelimiterXPP DefToken [Number] {} {} {} {} {}", sub[(cs, _nargs, _pre, ldel, rdel, _post, _body)] {
    let ldel_s = ldel.to_string();
    let rdel_s = rdel.to_string();
    let body = format!("\\left{ldel_s}#1\\right{rdel_s}");
    let params = parse_parameters("{}", &cs, true)?;
    def_macro(cs.clone(), params, Tokenize!(&body), None)?;
  });

  // \reDeclarePairedDelimiterInnerWrapper — stub (rarely used)
  DefMacro!("\\reDeclarePairedDelimiterInnerWrapper{}{}{}", None);

  //======================================================================
  // 3.7 — Math-mode symbol definitions
  //======================================================================

  DefMath!("\\lparen", "(", role => "OPEN",  stretchy => false);
  DefMath!("\\rparen", ")", role => "CLOSE", stretchy => false);

  DefMath!("\\vcentcolon", None, ":", role => "RELOP");
  DefMath!("\\ordinarycolon", None, ":", role => "RELOP");

  DefMath!("\\dblcolon", "::", role => "RELOP");

  DefMath!("\\coloneqq",    "\u{2254}",   role => "RELOP");
  DefMath!("\\Coloneqq",    "\u{2A74}",   role => "RELOP");
  DefMath!("\\coloneq",     "\u{2254}",   role => "RELOP");
  DefMath!("\\Coloneq",     "\u{2A74}",   role => "RELOP");
  DefMath!("\\eqqcolon",    "\u{2255}",   role => "RELOP");
  DefMath!("\\Eqqcolon",    "=::",        role => "RELOP");
  DefMath!("\\eqcolon",     "\u{2255}",   role => "RELOP");
  DefMath!("\\Eqcolon",     "=::",        role => "RELOP");
  DefMath!("\\colonapprox", ":\u{2248}",  role => "RELOP");
  DefMath!("\\Colonapprox", "::\u{2248}", role => "RELOP");
  DefMath!("\\approxcolon", "\u{2248}:",  role => "RELOP");
  DefMath!("\\Approxcolon", "\u{2248}::", role => "RELOP");
  DefMath!("\\colonsim",    ":\u{223C}",  role => "RELOP");
  DefMath!("\\Colonsim",    "::\u{223C}", role => "RELOP");
  DefMath!("\\simcolon",    "\u{223C}:",  role => "RELOP");
  DefMath!("\\Simcolon",    "\u{223C}::", role => "RELOP");
  DefMath!("\\colondash",   ":-",         role => "RELOP");
  DefMath!("\\Colondash",   "::-",        role => "RELOP");
  DefMath!("\\dashcolon",   "-:",         role => "RELOP");
  DefMath!("\\Dashcolon",   "-::",        role => "RELOP");

  // Perl: UTF(0x2909) — RIGHTWARDS DOUBLE ARROW FROM BAR (approximation)
  DefMath!("\\nuparrow", None, "\u{2909}", role => "ARROW");
  // Perl: UTF(0x2908) — DOWNWARDS DOUBLE ARROW FROM BAR (approximation)
  DefMath!("\\ndownarrow", None, "\u{2908}", role => "ARROW");
  // Perl: UTF(0xD7) = × MULTIPLICATION SIGN
  DefMath!("\\bigtimes", None, "\u{00D7}", role => "MULOP", meaning => "times",
    font => { size => 1.2 },
    dynamic_scriptpos => true);

  //======================================================================
  // 4 — Extended features
  //======================================================================

  // 4.2 — Prescripts
  DefMacro!("\\prescript{}{}{}",
    "\\@ams@prescript{#1}{#2}{#3}{\
     {}^{\\@mt@getmtoption{prescript-sup-format}{#1}}\
     _{\\@mt@getmtoption{prescript-sub-format}{#2}}\
     {\\@mt@getmtoption{prescript-arg-format}{#3}}\
     }");
  // wrapper to get reversion
  DefConstructor!("\\@ams@prescript{}{}{}{}", "#4",
    reversion => "\\prescript{#1}{#2}{#3}");

  // 4.4 — Spread lines
  DefMacro!("\\csname spreadlines\\endcsname{}", "\\begingroup\\jot=#1\\relax");
  DefMacro!("\\csname endspreadlines\\endcsname", "\\endgroup");

  // 4.5 — lgathered / rgathered
  // TODO: @@lgathered/@@rgathered have complex afterDigest/afterConstruct — simplified
  DefMacro!("\\lgathered[]",
    "\\@ams@multirow@bindings{name=lgathered,vattach=#1}\\@@lgathered\\lx@begin@alignment");
  DefMacro!("\\endlgathered", "\\lx@end@alignment\\@end@gathered");

  DefMacro!("\\rgathered[]",
    "\\@ams@multirow@bindings{name=rgathered,vattach=#1}\\@@rgathered\\lx@begin@alignment");
  DefMacro!("\\endrgathered", "\\lx@end@alignment\\@end@gathered");

  // Perl: DefConstructor('\@@lgathered DigestedBody', ...)
  // Forward to the gathered constructor from amsmath
  DefConstructor!("\\@@lgathered DigestedBody", "#1",
    before_digest => { bgroup(); },
    reversion => "\\begin{lgathered}#1\\end{lgathered}");
  DefConstructor!("\\@@rgathered DigestedBody", "#1",
    before_digest => { bgroup(); },
    reversion => "\\begin{rgathered}#1\\end{rgathered}");

  // \newgathered{name}{pre_line}{post_line}{after}
  // Creates \name and \endname environments for gathered-like displays.
  // Perl: DefMacro sub{} body creates runtime macros.
  DefPrimitive!("\\newgathered{}{}{}{}", sub[(name, _pre, _post, _after)] {
    let env_name = name.to_string();
    // Create \name macro → begins gathered alignment
    // Build tokens manually to preserve @ in CS names
    let mut begin_toks = vec![
      T_CS!("\\@ams@multirow@bindings"),
      T_BEGIN!(),
    ];
    begin_toks.extend(ExplodeText!(&format!("name={env_name}")));
    begin_toks.extend(vec![
      T_END!(),
      T_CS!("\\@@newgathered@dummy"),
      T_CS!("\\lx@begin@alignment"),
    ]);
    let begin_cs = T_CS!(&format!("\\{env_name}"));
    def_macro(begin_cs, None, Tokens::new(begin_toks), None)?;
    // Create \endname macro → ends alignment
    let end_toks = Tokens::new(vec![
      T_CS!("\\lx@end@alignment"),
      T_CS!("\\@end@gathered"),
    ]);
    let end_cs = T_CS!(&format!("\\end{env_name}"));
    def_macro(end_cs, None, end_toks, None)?;
  });
  Let!("\\renewgathered", "\\newgathered");

  // \@@newgathered@dummy — simplified gathered constructor
  DefConstructor!("\\@@newgathered@dummy",
    "<ltx:XMApp role='MULTIRELATION'>",
    before_digest => { bgroup(); }
  );
  DefPrimitive!("\\@end@gathered", { egroup()?; });

  // 4.6 — Split fractions
  DefMacro!("\\splitfrac{}{}",
    "\\@ams@multirow@bindings{name=splitfrac}\\@@multlined\\lx@begin@alignment #1 \\\\\\\\ #2 \\lx@end@alignment\\@end@multline");
  DefMacro!("\\splitdfrac{}{}",
    "\\displaystyle\\@ams@multirow@bindings{name=splitdfrac}\\@@multlined\\lx@begin@alignment #1 \\\\\\\\ #2 \\lx@end@alignment\\@end@multline");
});
