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
  // \added[author]{text} — render as <ltx:text class="ltx_changes_added">…</ltx:text>.
  DefConstructor!("\\added[]{}",
    "<ltx:text class='ltx_changes_added'>#2</ltx:text>");
  // \deleted[author]{text} — render struck-through. The text MUST
  // survive (content-preservation).
  DefConstructor!("\\deleted[]{}",
    "<ltx:text class='ltx_changes_deleted ltx_strike'>#2</ltx:text>");
  // \replaced[author]{new}{old} — show new with class hint; the old
  // text is content too, but Perl/HTML convention is to show only
  // the replacement in the rendered output.
  DefConstructor!("\\replaced[]{}{}",
    "<ltx:text class='ltx_changes_replaced'>#2</ltx:text>");
  DefConstructor!("\\highlight[]{}",
    "<ltx:text class='ltx_changes_highlight'>#2</ltx:text>");
  DefConstructor!("\\comment[]{}",
    "<ltx:text class='ltx_changes_comment'>#2</ltx:text>");
  DefMacro!("\\ChangesListline{}{}{}{}", "");
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
  DefMacro!("\\definechangesauthor[]{}", "");
  DefMacro!("\\listofchanges[]", "");
  DefMacro!("\\origcontentsline", "");
  DefMacro!("\\setaddedmarkup{}", "");
  DefMacro!("\\setauthormarkup{}", "");
  DefMacro!("\\setauthormarkupposition{}", "");
  DefMacro!("\\setanonymousname{}", "");
  DefMacro!("\\setauthormarkuptext{}", "");
  DefMacro!("\\setcommentmarkup{}", "");
  DefMacro!("\\setdeletedmarkup{}", "");
  DefMacro!("\\sethighlightmarkup{}", "");
  DefMacro!("\\setlocextension{}", "");
  DefMacro!("\\setsocextension{}", "");
  DefMacro!("\\setsummarytowidth{}", "");
  DefMacro!("\\setsummarywidth{}", "");
  DefMacro!("\\settruncatewidth{}", "");
});
