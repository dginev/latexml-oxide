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
  // \added[author]{text} — pass-through. Match Perl ar5iv-bindings
  // changes.sty.ltxml (DefMacro pass-through). A prior DefConstructor
  // wrapped #2 in `<ltx:text class='ltx_changes_added'>` which is an
  // inline-only element; papers commonly use `\added{multi-paragraph
  // block with \begin{equation}...}` in appendices to mark a whole
  // section, which auto-opened `<ltx:p>` inside `<ltx:text>` and then
  // produced `Error:malformed:ltx:text Attempt to close </ltx:text>,
  // which isn't open`. Witness 2404.13783 (appendix wraps several
  // paragraphs+equations in one `\added{...}`).
  DefMacro!("\\added[]{}", "#2");
  // \deleted[author]{text} — pass-through too. Earlier the body was
  // displayed via strike-through; switch to plain pass-through for
  // parity with Perl and to allow block content.
  DefMacro!("\\deleted[]{}", "#2");
  // \replaced[author]{new}{old} — show new (drop old), pass-through.
  DefMacro!("\\replaced[]{}{}", "#2");
  DefMacro!("\\highlight[]{}", "#2");
  DefMacro!("\\comment[]{}", "#2");
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
