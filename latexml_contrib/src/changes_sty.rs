use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
/// Routes inline macro expansion (each ~960 B of .text) through one
/// runtime call. Engine bootstrap pays parse_prototype once per entry.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

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
