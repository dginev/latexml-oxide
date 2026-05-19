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
  // \added[author]{text} — pass-through (#2). No container element in
  // the LaTeXML schema accepts ltx:section, so wrapping in <ltx:text>,
  // <ltx:inline-block>, or <ltx:note> all break on appendix-wide
  // `\added{\section{...}...}` patterns when the wrapper auto-closes
  // and the }-token tries to close it. Pass-through preserves content;
  // semantic class is lost in HTML output. Witness 2404.13783,
  // 2110.12098 (aastex63 with `\added{\section{Model Limitations}...}`).
  DefMacro!("\\added[]{}",    "#2");
  DefMacro!("\\deleted[]{}",  "#2");
  DefMacro!("\\replaced[]{}{}", "#2");
  DefMacro!("\\highlight[]{}", "#2");
  DefMacro!("\\comment[]{}",   "#2");
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
