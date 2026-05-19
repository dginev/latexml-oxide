use latexml_package::prelude::*;


LoadDefinitions!({
  // Preserve change-marked content (author body) as ltx:text with a
  // semantic class. The pre-content-preserving stub gobbled `\deleted`
  // arg #2 entirely; even when authors mark text for deletion in a
  // review-tracking context, that text is article material and the
  // semantic information ("this was marked deleted by author X") is
  // typeset-relevant. Match `\added` / `\replaced` / `\highlight` /
  // `\comment` semantics with named ltx:text classes so HTML/JATS
  // post-processors get the same role information real changes.sty
  // emits in pdf output.
  RequirePackage!("xcolor");
  RequirePackage!("ulem");
  RequirePackage!("todonotes");
  RequirePackage!("xstring");
  // \added[author]{text} — wrap in <ltx:inline-block class='ltx_changes_*'>
  // to delineate review-marked content as metadata. The prior
  // <ltx:text>-wrap variant blew up on `\added{\section{...}}` blocks
  // because ltx:text is inline-only; <ltx:inline-block> can hold both
  // inline and block content, preserving the semantic class while
  // allowing appendix-wide `\added{...multi-paragraph...}` patterns.
  // Witness 2404.13783, 2110.12098 (aastex63 multi-paragraph \added).
  DefConstructor!("\\added[]{}",
    "<ltx:inline-block class='ltx_changes_added'>#2</ltx:inline-block>");
  // \deleted[author]{text} — strike-through so the omitted text remains
  // visible inline (review-mode rendering).
  DefConstructor!("\\deleted[]{}",
    "<ltx:inline-block class='ltx_changes_deleted ltx_strike'>#2</ltx:inline-block>");
  // \replaced[author]{new}{old} — render new with added class, old with
  // strike-through deleted class. Both bodies preserved.
  DefConstructor!("\\replaced[]{}{}",
    "<ltx:inline-block class='ltx_changes_deleted ltx_strike'>#3</ltx:inline-block>\
     <ltx:inline-block class='ltx_changes_added'>#2</ltx:inline-block>");
  DefConstructor!("\\highlight[]{}",
    "<ltx:inline-block class='ltx_changes_highlight'>#2</ltx:inline-block>");
  DefConstructor!("\\comment[]{}",
    "<ltx:inline-block class='ltx_changes_comment'>#2</ltx:inline-block>");
  def_macro_noop("\\ChangesListline{}{}{}{}")?;
  DefMacro!("\\listofchangesname", "List of changes");
  DefMacro!("\\summaryofchangesname", "Changes");
  DefMacro!("\\compactsummaryofchangesname", "Changes (compact)");
  DefMacro!("\\changesaddedname", "Added");
  DefMacro!("\\changesdeletedname", "Deleted");
  DefMacro!("\\changesreplacedname", "Replaced");
  DefMacro!("\\changeshighlightname", "Highlighted");
  DefMacro!("\\changescommentname", "Commented");
  DefMacro!("\\changesauthorname", "Author");
  DefMacro!("\\changesanonymousname", "anonymous");
  DefMacro!("\\changesnochanges", "No changes.");
  DefMacro!(
    "\\changesnoloc",
    "List of changes is available after the next \\LaTeX\\ run."
  );
  DefMacro!(
    "\\changesnosoc",
    "Summary of changes is available after the next \\LaTeX\\ run."
  );
  Let!("\\cleaders", "\\leaders");
  def_macro_noop("\\definechangesauthor[]{}")?;
  def_macro_noop("\\listofchanges[]")?;
  def_macro_noop("\\origcontentsline")?;
  def_macro_noop("\\setaddedmarkup{}")?;
  def_macro_noop("\\setauthormarkup{}")?;
  def_macro_noop("\\setauthormarkupposition{}")?;
  def_macro_noop("\\setanonymousname{}")?;
  def_macro_noop("\\setauthormarkuptext{}")?;
  def_macro_noop("\\setcommentmarkup{}")?;
  def_macro_noop("\\setdeletedmarkup{}")?;
  def_macro_noop("\\sethighlightmarkup{}")?;
  def_macro_noop("\\setlocextension{}")?;
  def_macro_noop("\\setsocextension{}")?;
  def_macro_noop("\\setsummarytowidth{}")?;
  def_macro_noop("\\setsummarywidth{}")?;
  def_macro_noop("\\settruncatewidth{}")?;
});
