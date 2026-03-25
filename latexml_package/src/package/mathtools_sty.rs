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

  // \mathllap — zero-width math overlap (left): xoffset = -width
  DefConstructor!("\\mathllap[]{}",
    "<ltx:XMArg width='0pt' ?#xoffset(xoffset='#xoffset')>#2</ltx:XMArg>",
    after_digest => sub[whatsit] {
      if let Ok(Some(RegisterValue::Dimension(w))) = whatsit.get_width(None) {
        let neg = w.negate();
        whatsit.set_property("xoffset", Stored::String(arena::pin(neg.to_attribute())));
      }
      whatsit.set_width(Stored::String(arena::pin_static("0pt")));
    });
  // \mathrlap — zero-width math overlap (right): no xoffset needed
  DefConstructor!("\\mathrlap[]{}",
    "<ltx:XMArg width='0pt' ?#xoffset(xoffset='#xoffset')>#2</ltx:XMArg>",
    after_digest => sub[whatsit] {
      whatsit.set_width(Stored::String(arena::pin_static("0pt")));
    });
  // \mathclap — zero-width math overlap (center): xoffset = -0.5 * width
  DefConstructor!("\\mathclap[]{}",
    "<ltx:XMArg width='0pt' ?#xoffset(xoffset='#xoffset')>#2</ltx:XMArg>",
    after_digest => sub[whatsit] {
      if let Ok(Some(RegisterValue::Dimension(w))) = whatsit.get_width(None) {
        let half_neg = w.multiply(Float::new_f64(-0.5));
        whatsit.set_property("xoffset", Stored::String(arena::pin(half_neg.to_attribute())));
      }
      whatsit.set_width(Stored::String(arena::pin_static("0pt")));
    });

  DefConstructor!("\\clap{}", "#1");
  DefConstructor!("\\mathmakebox[][]{}", "#3");
  // Ignoring cramped, for now
  DefConstructor!("\\cramped[]{}", "#2");

  // Same as \mathllap, etc (but also cramped!)
  Let!("\\crampedllap", "\\mathllap");
  Let!("\\crampedrlap", "\\mathrlap");
  Let!("\\crampedclap", "\\mathclap");

  // \smashoperator — destructures argument to recognize operators and scripts.
  // Perl: \smashoperator[align]{op_sub_sup} → destructure → \lx@@smashoperator{align}{op}{sub}{sup}
  // The smashing (zero-width padding) is cosmetic only. Perl absorbs scripts into XMApp
  // structure. Our simplified version passes through the bare operator (scripts appear
  // naturally in the math context via normal TeX subscript/superscript processing).
  DefMacro!("\\smashoperator[]{}", "\\lx@smashoperator{#1}#2{}{}{}{}{}{}\\end");
  DefMacro!("\\lx@smashoperator{} {}{}{}{}{}{} Until:\\end", "#2");

  // \adjustlimits — Perl: {lim1} DefToken InScriptStyle {lim2} DefToken InScriptStyle
  // Produces two subscripted limit operators. The afterDigest adjusts depth/height
  // for visual alignment — cosmetic only. Our params read 6 balanced groups where
  // #2 and #5 are _ tokens (consumed silently by TeX when followed by subscript content).
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
  // Perl: DefConstructor('\@@multlined DigestedBody', "#1", ...)
  // DigestedBody absorbs the entire content until the matching end command.
  // Perl: afterDigest sets alignment rule {default=>'center', 0=>'left', -1=>'right'}
  // afterConstruct calls rearrangeAMSMultirow
  DefConstructor!("\\@@multlined DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from("center"));
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_0", Stored::from("left"));
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_LAST", Stored::from("right"));
    },
    after_construct => sub[document, whatsit] {
      if let Some(last) = document.get_node().get_last_child() {
        let align_rule = crate::package::amsmath_sty::get_multirow_alignment_rule(whatsit);
        crate::package::amsmath_sty::rearrange_ams_multirow(document, last, &align_rule)?;
      }
    }
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
  // Perl: creates \cmd with star/optional-size/plain variants:
  //   \cmd*{x}      → \left<ldel> x \right<rdel>
  //   \cmd[\Big]{x}  → \Big<ldel> x \Big<rdel>
  //   \cmd{x}       → <ldel> x <rdel>
  DefPrimitive!("\\DeclarePairedDelimiter DefToken {}{}", sub[(cs, ldel, rdel)] {
    let cmd = cs.to_string();
    let cmd_name = cmd.trim_start_matches('\\');
    let ldel_s = ldel.to_string();
    let rdel_s = rdel.to_string();
    // Star variant: \left...\right
    let star_body = format!("\\left{ldel_s}#1\\right{rdel_s}");
    let star_cs_name = format!("\\MT@delim@{cmd_name}@star");
    let star_cs = T_CS!(&star_cs_name);
    let star_params = parse_parameters("{}", &star_cs, true)?;
    def_macro(star_cs, star_params, Tokenize!(&star_body), None)?;
    // Non-star variant: optional size prefix
    let nostar_body = format!("#1{ldel_s}#2#1{rdel_s}");
    let nostar_cs_name = format!("\\MT@delim@{cmd_name}@nostar");
    let nostar_cs = T_CS!(&nostar_cs_name);
    let nostar_params = parse_parameters("[]{}", &nostar_cs, true)?;
    def_macro(nostar_cs, nostar_params, Tokenize!(&nostar_body), None)?;
    // Main command: \@ifstar dispatches to star or nostar
    let star_cs_tok = T_CS!(&star_cs_name);
    let nostar_cs_tok = T_CS!(&nostar_cs_name);
    let dispatch_toks = Tokens::new(vec![
      T_CS!("\\@ifstar"),
      T_BEGIN!(), star_cs_tok, T_END!(),
      T_BEGIN!(), nostar_cs_tok, T_END!(),
    ]);
    def_macro(cs, None, dispatch_toks, None)?;
  });

  // \DeclarePairedDelimiterX\cmd[nargs]{left}{right}{body}
  // Same star/optional dispatch as DeclarePairedDelimiter but with multi-arg body.
  // For now, body is ignored — content comes from the nargs parameters.
  DefPrimitive!("\\DeclarePairedDelimiterX DefToken [Number] {} {} {}", sub[(cs, nargs, ldel, rdel, _body)] {
    let cmd = cs.to_string();
    let cmd_name = cmd.trim_start_matches('\\');
    let n = nargs.value_of() as usize;
    let ldel_s = ldel.to_string();
    let rdel_s = rdel.to_string();
    let param_spec: String = (0..n.max(1)).map(|_| "{}").collect();
    // Star variant
    let star_body = format!("\\left{ldel_s}#1\\right{rdel_s}");
    let star_cs_name = format!("\\MT@delim@{cmd_name}@star");
    let star_cs = T_CS!(&star_cs_name);
    def_macro(star_cs, parse_parameters(&param_spec, &T_CS!(&star_cs_name), true)?,
      Tokenize!(&star_body), None)?;
    // Non-star variant
    let nostar_cs_name = format!("\\MT@delim@{cmd_name}@nostar");
    let nostar_cs = T_CS!(&nostar_cs_name);
    // Add [] for optional size prefix before the n args
    let nostar_param_spec = format!("[]{param_spec}");
    let nostar_body = format!("#1{ldel_s}#2#1{rdel_s}");
    def_macro(nostar_cs, parse_parameters(&nostar_param_spec, &T_CS!(&nostar_cs_name), true)?,
      Tokenize!(&nostar_body), None)?;
    // Main dispatch
    let dispatch_toks = Tokens::new(vec![
      T_CS!("\\@ifstar"),
      T_BEGIN!(), T_CS!(&star_cs_name), T_END!(),
      T_BEGIN!(), T_CS!(&nostar_cs_name), T_END!(),
    ]);
    def_macro(cs, None, dispatch_toks, None)?;
  });

  // \DeclarePairedDelimiterXPP — most general form
  // Same pattern with star/optional dispatch.
  DefPrimitive!("\\DeclarePairedDelimiterXPP DefToken [Number] {} {} {} {} {}", sub[(cs, _nargs, _pre, ldel, rdel, _post, _body)] {
    let cmd = cs.to_string();
    let cmd_name = cmd.trim_start_matches('\\');
    let ldel_s = ldel.to_string();
    let rdel_s = rdel.to_string();
    // Star variant
    let star_body = format!("\\left{ldel_s}#1\\right{rdel_s}");
    let star_cs_name = format!("\\MT@delim@{cmd_name}@star");
    def_macro(T_CS!(&star_cs_name), parse_parameters("{}", &T_CS!(&star_cs_name), true)?,
      Tokenize!(&star_body), None)?;
    // Non-star variant
    let nostar_body = format!("#1{ldel_s}#2#1{rdel_s}");
    let nostar_cs_name = format!("\\MT@delim@{cmd_name}@nostar");
    def_macro(T_CS!(&nostar_cs_name), parse_parameters("[]{}", &T_CS!(&nostar_cs_name), true)?,
      Tokenize!(&nostar_body), None)?;
    // Main dispatch
    let dispatch_toks = Tokens::new(vec![
      T_CS!("\\@ifstar"),
      T_BEGIN!(), T_CS!(&star_cs_name), T_END!(),
      T_BEGIN!(), T_CS!(&nostar_cs_name), T_END!(),
    ]);
    def_macro(cs, None, dispatch_toks, None)?;
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
  // Perl: afterDigest sets MULTIROW_ALIGNMENT_RULE { default => 'left' }
  // afterConstruct calls rearrangeAMSMultirow
  DefConstructor!("\\@@lgathered DigestedBody", "#1",
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from("left"));
    },
    reversion => "\\begin{lgathered}#1\\end{lgathered}",
    after_construct => sub[document, whatsit] {
      if let Some(last) = document.get_node().get_last_child() {
        let align_rule = crate::package::amsmath_sty::get_multirow_alignment_rule(whatsit);
        crate::package::amsmath_sty::rearrange_ams_multirow(document, last, &align_rule)?;
      }
    });
  DefConstructor!("\\@@rgathered DigestedBody", "#1",
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from("right"));
    },
    reversion => "\\begin{rgathered}#1\\end{rgathered}",
    after_construct => sub[document, whatsit] {
      if let Some(last) = document.get_node().get_last_child() {
        let align_rule = crate::package::amsmath_sty::get_multirow_alignment_rule(whatsit);
        crate::package::amsmath_sty::rearrange_ams_multirow(document, last, &align_rule)?;
      }
    });

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
