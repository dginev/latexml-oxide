use latexml_package::prelude::*;

LoadDefinitions!({
  // Basic stub binding that only preserves the final content,
  // but silently drops the changes-related metadata.
  RequirePackage!("xcolor");
  RequirePackage!("ulem");
  RequirePackage!("todonotes");
  RequirePackage!("xstring");
  DefMacro!("\\added[]{}", "#2");
  DefMacro!("\\deleted[]{}", "");
  DefMacro!("\\replaced[]{}{}", "#2");
  DefMacro!("\\highlight[]{}", "#2");
  DefMacro!("\\comment[]{}", "#2");
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
