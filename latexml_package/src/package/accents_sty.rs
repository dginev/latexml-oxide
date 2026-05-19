use crate::prelude::*;

LoadDefinitions!({
  DefMath!("\\ring{}", "\u{030A}", operator_role => "OVERACCENT");

  DefMacro!("\\lx@acc@size", "\\scriptstyle");

  DefMacro!("\\accentset{}{}", "\\lx@overaccentset{#1}{#2}");
  DefConstructor!("\\lx@overaccentset ScriptStyle {}",
    "<ltx:XMApp><ltx:XMWrap role='OVERACCENT'>#1</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>",
    sizer => "#1",
    alias => "\\accentset");

  DefMath!("\\dddot{}", "\u{02D9}\u{02D9}\u{02D9}", operator_role => "OVERACCENT");
  DefMath!("\\ddddot{}", "\u{02D9}\u{02D9}\u{02D9}\u{02D9}", operator_role => "OVERACCENT");

  // Perl: \lx@if@isaccent{thing}{true}{false}
  // Checks if thing is a single token whose definition has exactly 1 parameter.
  DefMacro!("\\lx@if@isaccent{}{}{}", sub[(thing, true_branch, false_branch)] {
    let toks = thing.unlist();
    // Must be a single token, with a definition that has exactly 1 parameter
    let is_accent = toks.len() == 1 && {
      if let Some(defn) = state::lookup_definition(&toks[0])? {
        defn.get_num_args() == 1
      } else {
        false
      }
    };
    if is_accent {
      Ok(true_branch)
    } else {
      Ok(false_branch)
    }
  });

  // Perl: \underaccent{acc}{base} — checks if acc is an accent command
  DefMacro!(
    "\\underaccent{}{}",
    "\\lx@if@isaccent{#1}{\\lx@converttounder{#1}{#2}{#1{#2}}}{\\lx@underaccentset{#1}{#2}}"
  );

  DefConstructor!("\\lx@underaccentset ScriptStyle {}",
    "<ltx:XMApp><ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>",
    sizer => "#1",
    alias => "\\underaccent");

  // Perl: \lx@converttounder Undigested Undigested {}
  // Absorbs #3 (which is #1{#2}, e.g. \hat{x}), then walks the XML to find
  // XMTok[@role='OVERACCENT'] and changes to UNDERACCENT.
  DefConstructor!("\\lx@converttounder Undigested Undigested {}", sub[document, args, _props] {
    // #3 is already digested and absorbed — absorb it into the document
    let thing = args[2].as_ref().unwrap();
    document.absorb(thing, None)?;
    // Find the OVERACCENT token in the last child and change to UNDERACCENT
    let current = document.get_node().clone();
    if let Some(last_child) = current.get_last_child() {
      let acc_nodes = document.findnodes("ltx:XMTok[@role='OVERACCENT']", Some(&last_child));
      if let Some(mut acc) = acc_nodes.into_iter().next() {
        document.set_attribute(&mut acc, "role", "UNDERACCENT")?;
      }
    }
  },
  reversion => sub[_whatsit, args] {
    let mut rev = vec![T_CS!("\\underaccent")];
    rev.push(T_BEGIN!());
    if let Some(arg0) = &args[0] { rev.extend(arg0.revert()?.unlist()); }
    rev.push(T_END!());
    rev.push(T_BEGIN!());
    if let Some(arg1) = &args[1] { rev.extend(arg1.revert()?.unlist()); }
    rev.push(T_END!());
    Ok(Tokens::new(rev))
  });

  DefMath!("\\undertilde{}", "\u{007E}", operator_role => "UNDERACCENT");
});
